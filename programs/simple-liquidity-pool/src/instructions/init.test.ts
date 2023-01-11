import * as anchor from "@project-serum/anchor";
import {Program} from "@project-serum/anchor";
import {SimpleLiquidityPool, IDL as SimpleLiquidityPoolIdl} from "../../../../target/types/simple_liquidity_pool";
import {sleep} from "../../../../tests/helpers/time";
import {getProgramConstant, getProviderWallet} from "../../../../tests/helpers/test-env";
import {assert, expect} from "chai";
import {NATIVE_MINT, NATIVE_MINT_2022} from "@solana/spl-token"
import {getPrevMintTokenInfoFromTmpData} from "../../../move-token/src/instructions/create_token.test";


export default function test__init(program: Program<SimpleLiquidityPool>) {
  it("can init lp and can init only once", async () => test_init_lp_only_once(program));
}

/**
 *
 * @param program
 */
async function test_init_lp_only_once(program: Program<SimpleLiquidityPool>) {
  console.log('{test_init_lp} : ', Date.now());

  const wallet = getProviderWallet();

  const LP_SEED_PREFIX_RAW = getProgramConstant("LP_SEED_PREFIX", program);
  const LP_SEED_PREFIX = Buffer.from(JSON.parse(LP_SEED_PREFIX_RAW), "utf8");
  expect(LP_SEED_PREFIX).is.not.empty;
  const LP_RATE_DECIMAL_RAW = getProgramConstant("LP_RATE_DECIMAL", program);
  expect(LP_RATE_DECIMAL_RAW).to.be.not.null;
  const LP_RATE_DECIMAL = parseInt(LP_RATE_DECIMAL_RAW);



  const tokenBasePubKey = NATIVE_MINT;  // Sol
  const prevMintToken = getPrevMintTokenInfoFromTmpData(); // This test must run after mint test; Test run async but mochajs test case will run once by one
  const tokenQuotePubKey = new anchor.web3.PublicKey(prevMintToken.mintKeypair.publicKey)
  const [liquidityPoolPubKey] = (anchor.web3.PublicKey.findProgramAddressSync(
    [
      LP_SEED_PREFIX,
      tokenBasePubKey.toBuffer(),
      tokenQuotePubKey.toBuffer(),
    ],
    program.programId
  ))

  const baseAta = await anchor.utils.token.associatedAddress({
    mint: tokenBasePubKey,
    owner: liquidityPoolPubKey
  });
  const quoteAta = await anchor.utils.token.associatedAddress({
    mint: tokenQuotePubKey,
    owner: liquidityPoolPubKey
  });

  const fixedRate = 10 * Math.pow(10, LP_RATE_DECIMAL);
  const getRequest = () => program.methods.initialize(fixedRate)
    .accounts({
      liquidityPool: liquidityPoolPubKey,
      tokenBase: tokenBasePubKey,
      tokenQuote: tokenQuotePubKey,
      baseAta: baseAta,
      quoteAta: quoteAta,
      authority: wallet.payer.publicKey,
      rent: anchor.web3.SYSVAR_RENT_PUBKEY,
      systemProgram: anchor.web3.SystemProgram.programId,
      tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
      associatedTokenProgram: anchor.utils.token.ASSOCIATED_PROGRAM_ID,
    })
    .signers([wallet.payer])
    .rpc();

  const tx = await getRequest();
  console.log('{test_init_lp} tx: ', tx);
  assert(!!tx, "Tx should not be empty");

  let tx2 = "";
  try {
    tx2 = await getRequest();
  } catch (e) {
    // console.log('{test_init_lp} e: ', e);
    // e.logs will show program logs
    expect(e.message.endsWith("error: 0x0"))
  }
  // tx2 should fail
  expect(tx2).to.be.empty;

  // Account must be created
  const lpAccount = await program.account.fixedRateLp.fetch(liquidityPoolPubKey);
  expect(lpAccount.rate).to.be.eq(fixedRate);
  expect(lpAccount.amountBaseAta.toBase58()).to.has.lengthOf(44);
}

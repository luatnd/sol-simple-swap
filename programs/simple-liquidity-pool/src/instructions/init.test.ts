import * as anchor from "@project-serum/anchor";
import {Program} from "@project-serum/anchor";
import {SimpleLiquidityPool, IDL as SimpleLiquidityPoolIdl} from "../../../../target/types/simple_liquidity_pool";
import {sleep} from "../../../../tests/helpers/time";
import {getProgramConstant, getProviderWallet} from "../../../../tests/helpers/test-env";
import {assert, expect} from "chai";
import {NATIVE_MINT, NATIVE_MINT_2022} from "@solana/spl-token"
import {getPrevMintTokenInfoFromTmpData} from "../../../move-token/src/instructions/create_token.test";
import {airDropSolIfBalanceLowerThan} from "../../../../tests/helpers/token";


export default function test__init(program: Program<SimpleLiquidityPool>) {
  // NOTE: This test must run only once per liquidity pair// TODO: Uncomment this to test init new LP, run only once
  // TODO: Uncomment this to test init new LP, run only once
  // it("can init lp and can init only once", async () => test_init_lp_only_once(program));

  it("Other wallet cannot init same pair", async () => test_reinit_lp_by_other_wallet(program));
}

/**
 * This test must run only once per liquidity pair
 * Default Pair: Sol - Your Token
 * Your Token is stored here: tests/tmp/create-token.json
 * It's the token minted with `move-token` tests
 */
async function test_init_lp_only_once(program: Program<SimpleLiquidityPool>) {
  console.log('{test_init_lp_only_once} : ', Date.now());

  const wallet = getProviderWallet();

  const tokenBasePubKey = NATIVE_MINT;  // Sol
  const prevMintToken = getPrevMintTokenInfoFromTmpData(); // This test must run after mint test; Test run async but mochajs test case will run once by one
  const tokenQuotePubKey = new anchor.web3.PublicKey(prevMintToken.mintKeypair.publicKey)
  const {tx, liquidityPoolPubKey} = await init_new_lp(
    program,
    tokenBasePubKey, tokenQuotePubKey,
    wallet.payer,
  );
  assert(!!tx, "Tx should not be empty");

  let tx2 = "";
  try {
    const {tx} = await init_new_lp(
      program,
      tokenBasePubKey, tokenQuotePubKey,
      wallet.payer,
    );
    tx2 = tx;
  } catch (e) {
    // console.log('{test_init_lp} e: ', e);
    // e.logs will show program logs
    expect(e.message.endsWith("error: 0x0"))
  }
  // tx2 should fail
  expect(tx2).to.be.empty;

  // Account must be created
  const lpAccount = await program.account.fixedRateLp.fetch(liquidityPoolPubKey);
  expect(lpAccount.amountBaseAta.toBase58()).to.has.lengthOf(44);
}

async function test_reinit_lp_by_other_wallet(program: Program<SimpleLiquidityPool>) {
  console.log('{test_reinit_lp_by_other_wallet} : ', Date.now());

  // new wallet
  const walletKeyPair = anchor.web3.Keypair.generate();
  await airDropSolIfBalanceLowerThan(0.1, walletKeyPair.publicKey);

  const tokenBasePubKey = NATIVE_MINT;  // Sol
  const prevMintToken = getPrevMintTokenInfoFromTmpData(); // This test must run after mint test; Test run async but mochajs test case will run once by one
  const tokenQuotePubKey = new anchor.web3.PublicKey(prevMintToken.mintKeypair.publicKey)

  let tx2 = "";
  try {
    const {tx} = await init_new_lp(
      program,
      tokenBasePubKey, tokenQuotePubKey,
      walletKeyPair,
    );
    tx2 = tx;
  } catch (e) {
    // console.log('{test_init_lp} e: ', e);
    // e.logs will show program logs
    expect(e.message.endsWith("error: 0x0"))
  }
  // tx2 should fail
  expect(tx2).to.be.empty;
}

async function init_new_lp(
  program: Program<SimpleLiquidityPool>,
  base: anchor.web3.PublicKey,
  quote: anchor.web3.PublicKey,
  authority: anchor.web3.Keypair,
) {
  const LP_SEED_PREFIX_RAW = getProgramConstant("LP_SEED_PREFIX", program);
  const LP_SEED_PREFIX = Buffer.from(JSON.parse(LP_SEED_PREFIX_RAW), "utf8");
  expect(LP_SEED_PREFIX).is.not.empty;
  const LP_RATE_DECIMAL_RAW = getProgramConstant("LP_RATE_DECIMAL", program);
  expect(LP_RATE_DECIMAL_RAW).to.be.not.null;
  const LP_RATE_DECIMAL = parseInt(LP_RATE_DECIMAL_RAW);


  const [liquidityPoolPubKey] = (anchor.web3.PublicKey.findProgramAddressSync(
    [
      LP_SEED_PREFIX,
      base.toBuffer(),
      quote.toBuffer(),
    ],
    program.programId
  ))
  console.log('{init_new_lp} liquidityPoolPubKey: ', liquidityPoolPubKey.toString());

  const baseAta = await anchor.utils.token.associatedAddress({
    mint: base,
    owner: liquidityPoolPubKey
  });
  const quoteAta = await anchor.utils.token.associatedAddress({
    mint: quote,
    owner: liquidityPoolPubKey
  });

  const fixedRate = 10 * Math.pow(10, LP_RATE_DECIMAL);
  const tx = await program.methods.initialize(fixedRate)
    .accounts({
      liquidityPool: liquidityPoolPubKey,
      tokenBase: base,
      tokenQuote: quote,
      baseAta: baseAta,
      quoteAta: quoteAta,
      authority: authority.publicKey,
      rent: anchor.web3.SYSVAR_RENT_PUBKEY,
      systemProgram: anchor.web3.SystemProgram.programId,
      tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
      associatedTokenProgram: anchor.utils.token.ASSOCIATED_PROGRAM_ID,
    })
    .signers([authority])
    .rpc();
  console.log('{init_new_lp} tx: ', tx);

  return {
    tx,
    liquidityPoolPubKey,
  };
}

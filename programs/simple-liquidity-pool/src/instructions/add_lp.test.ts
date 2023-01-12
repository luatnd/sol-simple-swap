import * as anchor from "@project-serum/anchor";
import {getProvider, Program} from "@project-serum/anchor";
import {SimpleLiquidityPool, IDL as SimpleLiquidityPoolIdl} from "../../../../target/types/simple_liquidity_pool";
import {IDL as MoveTokenIdl} from "../../../../target/types/move_token";
import {sleep} from "../../../../tests/helpers/time";
import {getCurrentProvider, getProgramConstant, getProgramIdlConstant, getProviderWallet} from "../../../../tests/helpers/test-env";
import {assert, expect} from "chai";
import {NATIVE_MINT, NATIVE_MINT_2022} from "@solana/spl-token"
import {getPrevMintTokenInfoFromTmpData} from "../../../move-token/src/instructions/create_token.test";


export default function test__add_liquidity(program: Program<SimpleLiquidityPool>) {
  it("Anyone with enough SOL balance can add liquidity to a LP", async () => test___add_liquidity_to_exist_lp(program));
}

async function test___add_liquidity_to_exist_lp(program: Program<SimpleLiquidityPool>) {
  console.log('{test___add_liquidity_to_exist_lp} : ', Date.now());


  const baseDepositAmount = 0.1;

  const provider = getCurrentProvider();
  const wallet = getProviderWallet();

  const LP_SEED_PREFIX_RAW = getProgramConstant("LP_SEED_PREFIX", program);
  const LP_SEED_PREFIX = Buffer.from(JSON.parse(LP_SEED_PREFIX_RAW), "utf8");
  expect(LP_SEED_PREFIX).is.not.empty;
  const LP_RATE_DECIMAL_RAW = getProgramConstant("LP_RATE_DECIMAL", program);
  expect(LP_RATE_DECIMAL_RAW).to.be.not.null;
  const LP_RATE_DECIMAL = parseInt(LP_RATE_DECIMAL_RAW);
  const TOKEN_DECIMAL_RAW = getProgramIdlConstant("TOKEN_DECIMAL", MoveTokenIdl);
  expect(TOKEN_DECIMAL_RAW).to.be.not.null;
  const TOKEN_DECIMAL = parseInt(TOKEN_DECIMAL_RAW);



  // const tokenBasePubKey = NATIVE_MINT;  // Sol
  const prevMintToken = getPrevMintTokenInfoFromTmpData(); // This test must run after mint test; Test run async but mochajs test case will run once by one
  const tokenQuotePubKey = new anchor.web3.PublicKey(prevMintToken.mintKeypair.publicKey)
  const [liquidityPoolPubKey] = (anchor.web3.PublicKey.findProgramAddressSync(
    [
      LP_SEED_PREFIX,
      // tokenBasePubKey.toBuffer(),
      tokenQuotePubKey.toBuffer(),
    ],
    program.programId
  ))

  // const baseAta = await anchor.utils.token.associatedAddress({
  //   mint: tokenBasePubKey,
  //   owner: liquidityPoolPubKey
  // });
  const quoteAta = await anchor.utils.token.associatedAddress({
    mint: tokenQuotePubKey,
    owner: liquidityPoolPubKey
  });
  // const userBaseAta = await anchor.utils.token.associatedAddress({
  //   mint: tokenBasePubKey,
  //   owner: wallet.payer.publicKey
  // });
  const userQuoteAta = await anchor.utils.token.associatedAddress({
    mint: tokenQuotePubKey,
    owner: wallet.payer.publicKey
  });

  const lpBalances = {
    before: {quote: 0, base: 0},
    after: {quote: 0, base: 0},
  }
  lpBalances.before.base = await provider.connection.getBalance(liquidityPoolPubKey);
  lpBalances.before.quote = new anchor.BN((await provider.connection.getTokenAccountBalance(quoteAta)).value.amount).toNumber();
  // console.log('{test___add_liquidity_to_exist_lp} lpBalances before: ', lpBalances);

  const fixedRateDecimal = 10;
  const baseAmount = baseDepositAmount * 1e9;
  const quoteAmount = baseDepositAmount * fixedRateDecimal * Math.pow(10, TOKEN_DECIMAL);

  const tx = await program.methods.addLiquidity(
    new anchor.BN(baseAmount),  // Solana decimal is 9
    new anchor.BN(quoteAmount), // My token
  )
    .accounts({
      liquidityPool: liquidityPoolPubKey,
      // tokenBase: tokenBasePubKey,
      tokenQuote: tokenQuotePubKey,
      // baseAta: baseAta,
      quoteAta: quoteAta,
      // userBaseAta: userBaseAta,
      userQuoteAta: userQuoteAta,
      authority: wallet.payer.publicKey,
      // rent: anchor.web3.SYSVAR_RENT_PUBKEY,
      systemProgram: anchor.web3.SystemProgram.programId,
      tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
      associatedTokenProgram: anchor.utils.token.ASSOCIATED_PROGRAM_ID,
    })
    .signers([wallet.payer])
    .rpc()
    .catch(e => {
      console.log('Error: ', e); // show on-chain logs
      throw e;
    });
  console.log('{test___add_liquidity_to_exist_lp} tx: ', tx);

  lpBalances.after.base = await provider.connection.getBalance(liquidityPoolPubKey);
  lpBalances.after.quote = new anchor.BN((await provider.connection.getTokenAccountBalance(quoteAta)).value.amount).toNumber();

  // lpBalances must increase
  // console.log('{test___add_liquidity_to_exist_lp} lpBalances after: ', lpBalances);
  expect(lpBalances.after.base).to.be.eq(lpBalances.before.base + baseAmount);
  expect(lpBalances.after.quote).to.be.eq(lpBalances.before.quote + quoteAmount);
}

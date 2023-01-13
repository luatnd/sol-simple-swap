import * as anchor from "@project-serum/anchor";
import {Program} from "@project-serum/anchor";
import {SimpleLiquidityPool} from "../../../../target/types/simple_liquidity_pool";
import {IDL as MoveTokenIdl} from "../../../../target/types/move_token";
import {getCurrentProvider, getProgramConstant, getProgramIdlConstant, getProviderWallet, VERBOSE} from "../../../../tests/helpers/test-env";
import {assert, expect} from "chai";
import {getPrevMintTokenInfoFromTmpData} from "../../../move-token/src/instructions/create_token.test";
import {Keypair, PublicKey} from "@solana/web3.js";
import {NATIVE_MINT} from "@solana/spl-token";
import {add_liquidity_to_exist_lp} from "./add_lp.test";


export default function test__swap(program: Program<SimpleLiquidityPool>) {
  it("Can swap SOL to token with fee deducted on token", async () => test__swap_sol_to_token(program));
  // it("Can swap token to SOL with fee deducted on SOL", async () => test__swap_token_to_sol(program));
  // it("Cannot swap more than liquidity", async () => test__swap_over_liquidity(program));
  // it("Can swap by everyone", async () => TODO(program));
  // it("Only liquidity provider can withdraw profit", async () => TODO(program));
}

async function test__swap_sol_to_token(program: Program<SimpleLiquidityPool>) {
  const wallet = getProviderWallet();
  const prevMintToken = getPrevMintTokenInfoFromTmpData(); // This test must run after mint test; Test run async but mochajs test case will run once by one
  const myTokenPubKey = new anchor.web3.PublicKey(prevMintToken.mintKeypair.publicKey)

  // For dev cycle only: Might add some liquidity first if NEEDED
  // await add_liquidity_to_exist_lp(program, {
  //   solAmount: 3,
  //   tokenAmount: 40,
  // })

  return test__swap_token(program, {
    from: NATIVE_MINT,
    to: myTokenPubKey,
    fromAmount: 0.0512345,
    payer: wallet.payer,
  });
}

async function test__swap_token_to_sol(program: Program<SimpleLiquidityPool>) {
  const wallet = getProviderWallet();
  const prevMintToken = getPrevMintTokenInfoFromTmpData(); // This test must run after mint test; Test run async but mochajs test case will run once by one
  const myTokenPubKey = new anchor.web3.PublicKey(prevMintToken.mintKeypair.publicKey)

  return test__swap_token(program, {
    from: myTokenPubKey,
    to: NATIVE_MINT,
    fromAmount: 1.2345,
    payer: wallet.payer,
  });
}

async function test__swap_over_liquidity(program: Program<SimpleLiquidityPool>) {
  const wallet = getProviderWallet();
  const prevMintToken = getPrevMintTokenInfoFromTmpData(); // This test must run after mint test; Test run async but mochajs test case will run once by one
  const myTokenPubKey = new anchor.web3.PublicKey(prevMintToken.mintKeypair.publicKey)

  let tx = "";
  try {
    tx = await test__swap_token(program, {
      from: NATIVE_MINT,
      to: myTokenPubKey,
      fromAmount: 30, // large enough to be over liquidity
      payer: wallet.payer,
    });
  } catch (e) {
    assert(
      e.message.indexOf("InsufficientQuoteAmount") > -1
      || e.message.indexOf("InsufficientBaseAmount") > -1,
      "Should throw error when swap over liquidity"
    );
  }
  expect(tx).to.be.empty;
}

async function test__swap_token(program: Program<SimpleLiquidityPool>, option: {
  from: PublicKey,
  to: PublicKey,
  fromAmount: number,
  payer: Keypair,
}) {
  console.log('{test__swap_token} : ', Date.now());

  const {
    from: fromPubKey,
    to: toPubKey,
    fromAmount,
    payer,
  } = option;

  const provider = getCurrentProvider();

  const LP_SEED_PREFIX = Buffer.from(JSON.parse(getProgramConstant("LP_SEED_PREFIX", program)), "utf8");
  assert(LP_SEED_PREFIX.toString().length > 0, "LP_FEE_SEED_PREFIX empty")
  const LP_FEE_SEED_PREFIX = Buffer.from(JSON.parse(getProgramConstant("LP_FEE_SEED_PREFIX", program)), "utf8");
  assert(LP_FEE_SEED_PREFIX.toString().length > 0, "LP_FEE_SEED_PREFIX empty")
  const LP_RATE_DECIMAL = parseInt(getProgramConstant("LP_RATE_DECIMAL", program));
  // expect(LP_RATE_DECIMAL).to.be.not.NaN.and.gt(0); // ==> This syntax has Bug in assertion
  assert(LP_RATE_DECIMAL > 0, "LP_RATE_DECIMAL must > 0");
  const LP_SWAP_FEE_PERMIL = parseInt(getProgramConstant("LP_SWAP_FEE_PERMIL", program));
  assert(LP_SWAP_FEE_PERMIL > 0, "LP_SWAP_FEE_PERMIL must > 0");
  const TOKEN_DECIMAL = parseInt(getProgramIdlConstant("TOKEN_DECIMAL", MoveTokenIdl));
  assert(TOKEN_DECIMAL > 0, "TOKEN_DECIMAL must > 0");

  const NATIVE_SOL_DECIMAL = 9;
  const PRICE_RATE = 10;
  const fromDecimals = fromPubKey.equals(NATIVE_MINT) ? NATIVE_SOL_DECIMAL : TOKEN_DECIMAL;
  // const toDecimals = fromPubKey.equals(NATIVE_MINT) ? TOKEN_DECIMAL: NATIVE_SOL_DECIMAL;


  /*
  swap Base(SOL) to quote(token):
    - LP inc fromAmount of SOL
    - LP dec fromAmount * rate of token
    - LP FEE inc % of `LP dec` in token: +0 SOL, +x Token
  swap quote(token) to Base(SOL):
    - LP dec fromAmount / rate of SOL
    - LP inc fromAmount of token
    - LP FEE inc % of `LP dec` in SOL: +x SOL, +0 Token
   */
  // NOTE: This fee logic must sync with Smart contract
  const swap_fee = (toAmount) => toAmount * (LP_SWAP_FEE_PERMIL / 1000);
  const changeMatrix = {
    // [Sol change, token change, fee Sol change, fee token change]
    baseToQuote: [
      +fromAmount,
      -(fromAmount * PRICE_RATE - swap_fee(fromAmount * PRICE_RATE)),
      0,
      swap_fee(fromAmount * PRICE_RATE),
    ],
    quoteToBase: [
      -(fromAmount / PRICE_RATE - swap_fee(fromAmount / PRICE_RATE)),
      +fromAmount,
      swap_fee(fromAmount / PRICE_RATE),
      0,
    ],
  }
  const v = (idx) => fromPubKey.equals(NATIVE_MINT)
    ? changeMatrix.baseToQuote[idx]
    : changeMatrix.quoteToBase[idx]
  const baseIncAmount = v(0) * Math.pow(10, NATIVE_SOL_DECIMAL);
  const quoteIncAmount = v(1) * Math.pow(10, TOKEN_DECIMAL);
  const baseFeeIncAmount = v(2) * Math.pow(10, NATIVE_SOL_DECIMAL);
  const quoteFeeIncAmount = v(3) * Math.pow(10, TOKEN_DECIMAL);
  VERBOSE && console.log('{test__swap_token} {}: ', {
    baseIncAmount,
    quoteIncAmount,
    baseFeeIncAmount,
    quoteFeeIncAmount,
  });


  const [liquidityPoolPubKey] = (anchor.web3.PublicKey.findProgramAddressSync(
    [
      LP_SEED_PREFIX,
      // tokenBasePubKey.toBuffer(),
      toPubKey.toBuffer(),
    ],
    program.programId
  ))
  const [liquidityPoolFeePubKey] = (anchor.web3.PublicKey.findProgramAddressSync(
    [
      LP_FEE_SEED_PREFIX,
      toPubKey.toBuffer(),
    ],
    program.programId
  ))

  const quoteAta = await anchor.utils.token.associatedAddress({
    mint: toPubKey,
    owner: liquidityPoolPubKey
  });
  const feeAta = await anchor.utils.token.associatedAddress({
    mint: toPubKey,
    owner: liquidityPoolFeePubKey
  });
  const userQuoteAta = await anchor.utils.token.associatedAddress({
    mint: toPubKey,
    owner: payer.publicKey
  });

  const lpBalances = {
    before: {quote: 0, base: 0},
    after: {quote: 0, base: 0},
  }
  const lpFeeBalances = {
    before: {quote: 0, base: 0},
    after: {quote: 0, base: 0},
  }
  lpBalances.before.base = await provider.connection.getBalance(liquidityPoolPubKey);
  lpBalances.before.quote = new anchor.BN((await provider.connection.getTokenAccountBalance(quoteAta)).value.amount).toNumber();
  lpFeeBalances.before.base = await provider.connection.getBalance(liquidityPoolFeePubKey);
  lpFeeBalances.before.quote = new anchor.BN((await provider.connection.getTokenAccountBalance(feeAta)).value.amount).toNumber();

  VERBOSE && console.log('{test__swap_token} : ', {
    liquidityPoolPubKey: liquidityPoolPubKey.toString(),
    liquidityPoolFeePubKey: liquidityPoolFeePubKey.toString(),
    feeAta: feeAta.toString(),
    quoteAta: quoteAta.toString(),
    lpBalances,
    lpFeeBalances,
    fromAmountBN: fromAmount * Math.pow(10, fromDecimals),
  });

  const tx = await program.methods.swap(
    fromPubKey,
    toPubKey,
    new anchor.BN(fromAmount * Math.pow(10, fromDecimals)),
  )
    .accounts({
      liquidityPool: liquidityPoolPubKey,
      liquidityPoolFee: liquidityPoolFeePubKey,
      feeAta: feeAta,
      tokenQuote: toPubKey,
      quoteAta: quoteAta,
      userQuoteAta: userQuoteAta,
      authority: payer.publicKey,
      // rent: anchor.web3.SYSVAR_RENT_PUBKEY,
      systemProgram: anchor.web3.SystemProgram.programId,
      tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
      associatedTokenProgram: anchor.utils.token.ASSOCIATED_PROGRAM_ID,
    })
    .signers([])
    // .signers([payer])
    .rpc()
    .catch(e => {
      VERBOSE && console.log('Error: ', e); // show on-chain logs
      throw e;
    });
  VERBOSE && console.log('{test__swap_token} tx: ', tx);

  lpBalances.after.base = await provider.connection.getBalance(liquidityPoolPubKey);
  lpBalances.after.quote = new anchor.BN((await provider.connection.getTokenAccountBalance(quoteAta)).value.amount).toNumber();
  lpFeeBalances.after.base = await provider.connection.getBalance(liquidityPoolFeePubKey);
  lpFeeBalances.after.quote = new anchor.BN((await provider.connection.getTokenAccountBalance(feeAta)).value.amount).toNumber();

  // lpBalances must increase
  VERBOSE && console.log('{test__swap_token} lpBalances & fee after: ', lpBalances, lpFeeBalances);
  expect(lpBalances.after.base).to.be.approximately(lpBalances.before.base + baseIncAmount, 1e6, `LP base balance must increase ${baseIncAmount}`);
  expect(lpBalances.after.quote).to.be.approximately(lpBalances.before.quote + quoteIncAmount, 1e6, `LP quote balance must increase ${quoteIncAmount}`);
  expect(lpFeeBalances.after.base).to.be.approximately(lpFeeBalances.before.base + baseFeeIncAmount, 1e6, `LP fee base balance must increase ${baseFeeIncAmount}`);
  expect(lpFeeBalances.after.quote).to.be.approximately(lpFeeBalances.before.quote + quoteFeeIncAmount, 1e6, `LP fee quote balance must increase ${quoteFeeIncAmount}`);

  return tx;
}

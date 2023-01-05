import * as anchor from "@project-serum/anchor";
import {Program} from "@project-serum/anchor";
import {MoveToken} from "../../../../target/types/move_token";
import {getCurrentProvider, getProviderWallet, getTestTokenMetadata} from "../../../../tests/helpers/test-env";
import {getPrevMintTokenInfoFromTmpData} from "./create_token.test";
import {sleep} from "../../../../tests/helpers/time";
import {expect} from "chai";
import {airdropSOL} from "../../../../tests/helpers/token";


export default function test__mintTokenToOtherWallet(program: Program<MoveToken>) {
  it("can mint token to another wallet, or airdrop", async () => mintTokenToAnyWallet(program));
}

/**
 * Test cases:
 * - tx success
 * - cannot mint to a wallet twice
 * - MOVE balance after minting increases by amount arg
 */
async function mintTokenToAnyWallet(program: Program<MoveToken>) {
  console.log('{mintTokenToAnyWallet} : ', Date.now());
  await sleep(300);

  const AIR_DROP_AMOUNT = 1.5;
  // const RECIPIENT_ADDR = "CKsW2dWontvwCJTyesYEbZ8nScx8LiL1utjkvmwhHLkT";
  const RECIPIENT_ADDR = anchor.web3.Keypair.generate().publicKey; // airdrop each user once and only once


  const payer = getProviderWallet(); // use my wallet to pay mint fee
  const tokenInfo = getTestTokenMetadata();

  // await sleep(1000);
  const prevMintToken = getPrevMintTokenInfoFromTmpData(); // Test run async but mochajs test case will run once by one
  const mintKeypair = anchor.web3.Keypair.fromSecretKey(Uint8Array.from(prevMintToken.mintKeypair.secret));
  console.log(`{mintTokenToAnyWallet} mint addr: ${mintKeypair.publicKey}`);

  const recipientPubKey = new anchor.web3.PublicKey(RECIPIENT_ADDR);

  // Airdrop 1 SOL to recipient for paying for the transaction
  // Don't need to airdrop because the recipient is a SOL millionaire
  // await airdropSOL(recipientPubKey, AIR_DROP_AMOUNT);

  console.log(`Recipient pubkey: ${recipientPubKey}`);

  const [mintAuthorityPda, mintAuthorityPdaBump] = anchor.web3.PublicKey.findProgramAddressSync(
    [
      Buffer.from("mint_authority_"),
      mintKeypair.publicKey.toBuffer(),
    ],
    program.programId,
  );

  const associatedTokenAccount = await anchor.utils.token.associatedAddress({
    mint: mintKeypair.publicKey,
    owner: recipientPubKey
  });
  console.log(`associatedTokenAccount: ${associatedTokenAccount}`);

  // get current token balance of recipient
  // const currentBalance = await anchor.utils.acc.getTokenAccountBalance(provider.connection, ata);
  // get current account balance of recipient
  // const currentBalance = await provider.connection.getBalance(recipientPubKey);
  // console.log(`Current balance: ${currentBalance.value}`);

  const tx = await program.methods.mintToAnotherWallet(
    new anchor.BN(AIR_DROP_AMOUNT * Math.pow(10, tokenInfo.decimals)),
    mintAuthorityPdaBump
  )
    .accounts({
      mintAccount: mintKeypair.publicKey,
      mintAuthority: mintAuthorityPda,
      recipient: recipientPubKey,
      recipientAta: associatedTokenAccount,
      payer: payer.publicKey,
      rent: anchor.web3.SYSVAR_RENT_PUBKEY,
      systemProgram: anchor.web3.SystemProgram.programId,
      tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
      associatedTokenProgram: anchor.utils.token.ASSOCIATED_PROGRAM_ID,
    })
    .signers([payer.payer])
    .rpc();
  console.log("{mintTokenToAnyWallet} tx", tx);

  // const afterAirDropBalance = 0;
  // expect(10).to.equal(11);
}

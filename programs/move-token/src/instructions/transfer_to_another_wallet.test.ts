import * as anchor from "@project-serum/anchor";
import {Program} from "@project-serum/anchor";
import {MoveToken} from "../../../../target/types/move_token";
import {getCurrentProvider, getProviderWallet, getTestTokenMetadata} from "../../../../tests/helpers/test-env";
import {getPrevMintTokenInfoFromTmpData} from "./create_token.test";
import {airdropSOL, airDropSolIfBalanceLowerThan} from "../../../../tests/helpers/token";


export default function test__transferTokenToOtherWallet(program: Program<MoveToken>) {
  it("can transfer token to another wallet", async () => testTransferToOtherWallet(program));
}

async function testTransferToOtherWallet(program: Program<MoveToken>) {
  const provider = getCurrentProvider();
  const payer = getProviderWallet(); // use my wallet to pay mint fee
  const tokenInfo = getTestTokenMetadata();
  const prevMintToken = getPrevMintTokenInfoFromTmpData(); // Test run async but mochajs test case will run once by one
  const mintKeypair = anchor.web3.Keypair.fromSecretKey(Uint8Array.from(prevMintToken.mintKeypair.secret));
  console.log(`{testTransferToOtherWallet} mint addr: ${mintKeypair.publicKey}`);

  const TRANSFER_AMOUNT = 1.1;
  const RECIPIENT_ADDR = "CKsW2dWontvwCJTyesYEbZ8nScx8LiL1utjkvmwhHLkT";
  // const RECIPIENT_ADDR = anchor.web3.Keypair.generate().publicKey; // airdrop each user once and only once
  const recipientPubKey = new anchor.web3.PublicKey(RECIPIENT_ADDR);
  console.log(`Recipient pubkey: ${recipientPubKey}`);

  // Airdrop 1 SOL to recipient because payer=owner
  // await airDropSolIfBalanceLowerThan(0.1, recipientPubKey);

  const ownerTokenAddress = await anchor.utils.token.associatedAddress({
    mint: mintKeypair.publicKey,
    owner: payer.publicKey
  });
  console.log(`Owner Token Address: ${ownerTokenAddress}`);
  const recipientTokenAddress = await anchor.utils.token.associatedAddress({
    mint: mintKeypair.publicKey,
    owner: recipientPubKey
  });
  console.log(`Recipient Token Address: ${recipientTokenAddress}`);

  const tx = await program.methods.transferToAnotherWallet(
    new anchor.BN(TRANSFER_AMOUNT * Math.pow(10, tokenInfo.decimals)),
  )
    .accounts({
      mintAccount: mintKeypair.publicKey,
      ownerTokenAccount: ownerTokenAddress,
      recipientAta: recipientTokenAddress,
      owner: payer.publicKey,
      recipient: recipientPubKey,
      rent: anchor.web3.SYSVAR_RENT_PUBKEY,
      systemProgram: anchor.web3.SystemProgram.programId,
      tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
      associatedTokenProgram: anchor.utils.token.ASSOCIATED_PROGRAM_ID,
    })
    .signers([payer.payer])
    .rpc();
  console.log("{testTransferToOtherWallet} tx", tx);
}

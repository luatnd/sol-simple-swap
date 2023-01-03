import * as anchor from "@project-serum/anchor";
import {Program} from "@project-serum/anchor";
import {MoveToken} from "../../../../target/types/move_token";
import {getProviderWallet, getTestTokenMetadata} from "../../../../tests/helpers/test-env";
import {sleep} from "../../../../tests/helpers/time";


export default function test__create_token(program: Program<MoveToken>) {
  it("canCreateNewToken", async () => canCreateNewToken(program));
  it("Test 2", async () => {
    console.log('{Test 2} : ', );
    await sleep(300);
  })
}


async function canCreateNewToken(program: Program<MoveToken>) {
  console.log('{canCreateNewToken} : ', );
  await sleep(1000);


  const METAPLEX_PROGRAM_ID = "metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s";
  const tokenMetadataProgramId = new anchor.web3.PublicKey(METAPLEX_PROGRAM_ID);
  const payer = getProviderWallet();
  const mintKeypair: anchor.web3.Keypair = anchor.web3.Keypair.generate();
  console.log(`{canCreateNewToken} New random token addr: ${mintKeypair.publicKey}`);



  const {uri, metadata} = getTestTokenMetadata();
  const [mintAuthorityPda, mintAuthorityPdaBump] = await anchor.web3.PublicKey.findProgramAddress(
    [
      Buffer.from("mint_authority_"), // must match the program in rust
      mintKeypair.publicKey.toBuffer(),
    ],
    program.programId,
  );
  const metadataAddress = (await anchor.web3.PublicKey.findProgramAddress(
    [
      Buffer.from("metadata"), // must match the metaplex program source code
      tokenMetadataProgramId.toBuffer(),
      mintKeypair.publicKey.toBuffer(),
    ],
    tokenMetadataProgramId
  ))[0];


  // Add your test here.
  const tx = await program.methods.createToken(
    metadata.name,
    metadata.symbol,
    uri,
    mintAuthorityPdaBump
  )
    .accounts({
      metadataAccount: metadataAddress,
      mintAccount: mintKeypair.publicKey,
      mintAuthority: mintAuthorityPda,
      payer: payer.publicKey,
      rent: anchor.web3.SYSVAR_RENT_PUBKEY,
      systemProgram: anchor.web3.SystemProgram.programId,
      tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
      tokenMetadataProgram: tokenMetadataProgramId,
    })
    .signers([mintKeypair, payer.payer])
    .rpc();
  console.log("{canCreateNewToken} Your transaction signature", tx);
}

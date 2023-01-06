import * as anchor from "@project-serum/anchor";
import {getCurrentProvider} from "./test-env";

export async function airdropSOL(recipientPubKey: anchor.web3.PublicKey, amountOfSol: number) {
  // @deprecated
  // await provider.connection.confirmTransaction(
  //   await provider.connection.requestAirdrop(recipientPubKey, 1 * anchor.web3.LAMPORTS_PER_SOL)
  // );
  const provider = getCurrentProvider();
  const airdropSignature = await provider.connection.requestAirdrop(recipientPubKey, amountOfSol * anchor.web3.LAMPORTS_PER_SOL);
  const latestBlockHash = await provider.connection.getLatestBlockhash();
  return provider.connection.confirmTransaction({
    blockhash: latestBlockHash.blockhash,
    lastValidBlockHeight: latestBlockHash.lastValidBlockHeight,
    signature: airdropSignature,
  });
}

export async function airDropSolIfBalanceLowerThan(amountOfSol: number, recipientPubKey: anchor.web3.PublicKey) {
  const provider = getCurrentProvider();
  const balance = await provider.connection.getBalance(recipientPubKey);
  if (balance < amountOfSol * anchor.web3.LAMPORTS_PER_SOL) {
    console.log('{airDropSolIfBalanceLowerThan} ' + `${recipientPubKey} has small balance ${balance / anchor.web3.LAMPORTS_PER_SOL} SOL, airdrop ${amountOfSol} SOL`);
    await airdropSOL(recipientPubKey, amountOfSol);
  }
}

export async function getSqlTokenBalanceOf(
  owner: anchor.web3.PublicKey,
  tokenMintAddress: anchor.web3.PublicKey
) {
  const provider = getCurrentProvider();
  // ata stands for Associated Token Address
  const ata = await anchor.utils.token.associatedAddress({
    mint: tokenMintAddress,
    owner: owner,
  });
  const res = await provider.connection.getTokenAccountBalance(ata);
  return res.value;
}

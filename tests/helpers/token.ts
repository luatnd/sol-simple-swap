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

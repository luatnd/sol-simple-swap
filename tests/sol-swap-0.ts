import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { SolSwap0 } from "../target/types/sol_swap_0";

describe("sol-swap-0", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.SolSwap0 as Program<SolSwap0>;

  it("Is initialized!", async () => {
    // Add your test here.
    const tx = await program.methods.initialize().rpc();
    console.log("Your transaction signature", tx);
  });
});

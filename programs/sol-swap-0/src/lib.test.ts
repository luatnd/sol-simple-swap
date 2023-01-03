import { SolSwap0 } from "../../../target/types/sol_swap_0";
import testProgram from "../../../tests/helpers/testProgram";

const tests = [
  isInitialized,
];
testProgram<SolSwap0>("SolSwap0", tests)


function isInitialized(program) {
  it("Is initialized!", async () => {
    // Add your test here.
    const tx = await program.methods.initialize().rpc();
    console.log("Your transaction signature", tx);
  })
}

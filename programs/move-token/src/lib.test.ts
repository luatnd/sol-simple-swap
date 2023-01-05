import { MoveToken } from "../../../target/types/move_token";
import testProgram from "../../../tests/helpers/testProgram";
import test__create_token from "./instructions/create_token.test"
import test__mintTokenToOtherWallet from "./instructions/mint_to_another_wallet.test";

const tests = [
  // test__create_token, // TODO: run it once then comment it out when you don't wanna generate any new token
  test__mintTokenToOtherWallet,
];
testProgram<MoveToken>("MoveToken", tests)

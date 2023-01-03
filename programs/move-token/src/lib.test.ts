import { MoveToken } from "../../../target/types/move_token";
import testProgram from "../../../tests/helpers/testProgram";
import test__create_token from "./instructions/create_token.test"

const tests = [
  test__create_token,
];
testProgram<MoveToken>("MoveToken", tests)

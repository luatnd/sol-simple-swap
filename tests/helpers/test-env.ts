import * as anchor from "@project-serum/anchor";
import {AnchorProvider, Idl, Program} from "@project-serum/anchor";

export function getTestProgram<TProgram extends Idl>(program_name: string) {
  /**
   * The [provider] was configured in Anchor.toml
   */
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());
  const program = anchor.workspace[program_name] as Program<TProgram>;
  return program;
}

export function getCurrentProvider() {
  // NOTE: We already anchor.setProvider at the beginning of the describe block
  return anchor.getProvider() as AnchorProvider;
}

export function getProviderWallet() {
  const provider = getCurrentProvider();
  const wallet = provider.wallet as anchor.Wallet;

  // console.log('{getProviderWallet} wallet: ', wallet.publicKey.toString());

  return wallet;
}

export function getTestTokenMetadata() {
  return {
    uri: "https://gist.githubusercontent.com/luatnd/f28c2da59b2eea505e7d8bf9631dcc17/raw/feb550c81d82262442b0d2cddc14e1013eae5211/sol-token-luat.json",
    decimals: 9,
    initialSupply: 10000,
    metadata: {
      "name": "Luat Dev",
      "symbol": "LUAT",
      "description": "Just a non-prod ready token",
      "image": "https://avatars.githubusercontent.com/u/1859127?v=4",
      "external_url": "https://luatnd.github.io/aframe-react-demo/",
      "attributes": [{"trait_type": "Speed", "value": "Rapid"}],
    }
  };
}

export function getProgramConstant(constant_name: string, program): string {
  const constants = program.idl.constants.filter(i => i.name === constant_name);
  if (constants[0]) {
    return constants[0].value;
  }

  return undefined;
}

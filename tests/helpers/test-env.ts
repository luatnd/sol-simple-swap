import * as anchor from "@project-serum/anchor";
import {AnchorProvider} from "@project-serum/anchor";

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
    uri: "https://gist.githubusercontent.com/luatnd/f28c2da59b2eea505e7d8bf9631dcc17/raw/1872678838d4e6ee779e5938e83938629c6a308c/sol-token-luat.json",
    metadata: {
      "name": "Luat Foo Bar",
      "symbol": "iLuat",
      "description": "Just a non-prod ready token",
      "image": "https://avatars.githubusercontent.com/u/1859127?v=4",
      "external_url": "https://luatnd.github.io/aframe-react-demo/",
      "attributes": [{"trait_type": "Speed", "value": "Rapid"}],
    }
  };
}

use anchor_lang::prelude::*;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod move_token {
  use super::*;

  // pub fn create_token_mint(
  //   ctx: Context<CreateTokenMint>,
  //   metadata_title: String,
  //   metadata_symbol: String,
  //   metadata_uri: String,
  //   mint_authority_pda_bump: u8,
  // ) -> Result<()> {
  //
  //   create_token_mint::create_token_mint(
  //     ctx,
  //     metadata_title,
  //     metadata_symbol,
  //     metadata_uri,
  //     mint_authority_pda_bump,
  //   )
  // }
  //
  // pub fn mint_to_your_wallet(
  //   ctx: Context<MintToYourWallet>,
  //   amount: u64,
  //   mint_authority_pda_bump: u8,
  // ) -> Result<()> {
  //
  //   mint_to_your_wallet::mint_to_your_wallet(
  //     ctx,
  //     amount,
  //     mint_authority_pda_bump,
  //   )
  // }
}

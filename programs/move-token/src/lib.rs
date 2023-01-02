use anchor_lang::prelude::*;

mod instructions;
use instructions::*; // Must import as * to avoid error

declare_id!("4fQVnLWKKKYxtxgGn7Haw8v2g2Hzbu8K61JvWKvqAi7W");

#[program]
pub mod move_token {
  use super::*;

  pub fn create_token(
    ctx: Context<CreateTokenMint>,
    metadata_title: String,
    metadata_symbol: String,
    metadata_uri: String,
    mint_authority_pda_bump: u8,
  ) -> Result<()> {
    create_token::create_token(
      ctx,
      metadata_title,
      metadata_symbol,
      metadata_uri,
      mint_authority_pda_bump,
    )
  }

  pub fn mint_to(
    ctx: Context<MintToYourWallet>,
    amount: u64,
    mint_authority_pda_bump: u8,
  ) -> Result<()> {
    mint_token::mint_to_your_wallet(
      ctx,
      amount,
      mint_authority_pda_bump,
    )
  }
}

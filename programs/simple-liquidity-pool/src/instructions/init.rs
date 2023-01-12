use anchor_lang::prelude::*;
use anchor_spl::{
  token,
  associated_token,
};
use crate::state::{FixedRateLP, LP_SEED_PREFIX};


pub fn init(ctx: Context<LpInit>, fixed_rate: u32) -> Result<()> {
  let lp = &mut ctx.accounts.liquidity_pool;
  // msg!("Initializing liquidity pool {:?}", lp);

  lp.init(
    ctx.accounts.token_base.key(),
    ctx.accounts.token_quote.key(),
    ctx.accounts.base_ata.key(),
    ctx.accounts.quote_ata.key(),
    fixed_rate,
  )?;

  Ok(())
}


#[derive(Accounts)]
pub struct LpInit<'info> {
  #[account(
    init,
    payer = authority,
    space = 8 + FixedRateLP::MAXIMUM_SIZE,
    seeds = [
      LP_SEED_PREFIX,
      token_base.key().as_ref(),
      token_quote.key().as_ref()
    ],
    bump,
  )]
  pub liquidity_pool: Account<'info, FixedRateLP>,

  // base Token Mint Address: Read more in README.md
  #[account()]
  pub token_base: Account<'info, token::Mint>,
  #[account()]
  pub token_quote: Account<'info, token::Mint>,

  #[account(
    init,
    payer = authority,
    associated_token::mint = token_base,
    associated_token::authority = liquidity_pool,
  )]
  pub base_ata: Account<'info, token::TokenAccount>,
  #[account(
    init,
    payer = authority,
    associated_token::mint = token_quote,
    associated_token::authority = liquidity_pool,
  )]
  pub quote_ata: Account<'info, token::TokenAccount>,


  #[account(mut)]
  pub authority: Signer<'info>,

  pub rent: Sysvar<'info, Rent>,
  pub system_program: Program<'info, System>,
  pub token_program: Program<'info, token::Token>,
  pub associated_token_program: Program<'info, associated_token::AssociatedToken>,
}

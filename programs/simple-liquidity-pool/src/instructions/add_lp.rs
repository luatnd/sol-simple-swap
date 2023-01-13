use anchor_lang::prelude::*;
use anchor_spl::{
  token,
  token::spl_token,
  associated_token,
};
use crate::state::{FixedRateLP, FixedRateLpFee, LP_SEED_PREFIX, LP_FEE_SEED_PREFIX, transfer_token_into_pool};

pub fn add_liquidity(
  ctx: Context<LpAddLiquidity>,
  base_amount: u64,
  quote_amount: u64,
) -> Result<()> {
  let lp = &mut ctx.accounts.liquidity_pool;
  lp.add_liquidity(base_amount, quote_amount)?;

  transfer_token_into_pool(&ctx, spl_token::native_mint::id(), base_amount, None)?;
  transfer_token_into_pool(&ctx, ctx.accounts.token_quote.key(), quote_amount, None)?;

  Ok(())
}


#[derive(Accounts)]
#[instruction(lp_bump: u8)]
pub struct LpAddLiquidity<'info> {
  #[account(
    mut,
    seeds = [
      LP_SEED_PREFIX,
      token_quote.key().as_ref()
    ],
    bump = liquidity_pool.bump,
  )]
  pub liquidity_pool: Account<'info, FixedRateLP>,

  #[account(
    mut,
    seeds = [LP_FEE_SEED_PREFIX, token_quote.key().as_ref()],
    bump,
  )]
  pub liquidity_pool_fee: Account<'info, FixedRateLpFee>,

  #[account(mut)]
  pub token_quote: Account<'info, token::Mint>,

  // LP's quotes Token Mint Address: Read more in README.md
  #[account(
    mut,
    associated_token::mint = token_quote,
    associated_token::authority = liquidity_pool,
  )]
  pub quote_ata: Account<'info, token::TokenAccount>,

  #[account(
    init_if_needed,
    payer = authority,
    associated_token::mint = token_quote,
    associated_token::authority = authority,
  )]
  pub user_quote_ata: Account<'info, token::TokenAccount>,

  #[account(
    mut,
    associated_token::mint = token_quote,
    associated_token::authority = liquidity_pool_fee,
  )]
  pub fee_ata: Account<'info, token::TokenAccount>,


  #[account(mut)]
  pub authority: Signer<'info>,

  // pub rent: Sysvar<'info, Rent>,
  pub system_program: Program<'info, System>,
  pub token_program: Program<'info, token::Token>,
  pub associated_token_program: Program<'info, associated_token::AssociatedToken>,
}

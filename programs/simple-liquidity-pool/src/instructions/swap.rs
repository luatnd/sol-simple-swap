use anchor_lang::prelude::*;
use anchor_spl::{
  token,
  associated_token,
};
use crate::state::{
  FixedRateLP, FixedRateLpFee,
  LP_SEED_PREFIX, LP_FEE_SEED_PREFIX,
  transfer_token_into_pool, transfer_token_into_pool_fee, transfer_token_out_of_pool,
};


pub fn swap(
  ctx: Context<LpSwap>,
  from: Pubkey,
  to: Pubkey,
  from_amount: u64,
) -> Result<()> {
  let lp = &mut ctx.accounts.liquidity_pool;

  let current_base_liquidity: u64 = lp.to_account_info().lamports();
  let current_quote_liquidity: u64 = ctx.accounts.quote_ata.amount;

  let (
    _,
    from_amount,
    to_amount_without_fee,
    fee_of_to_token
  ) = lp.preview_swap(from, to, from_amount, current_base_liquidity, current_quote_liquidity)?;

  let to_amount = to_amount_without_fee - fee_of_to_token;

  transfer_token_into_pool(&ctx, from, from_amount, None)?;
  transfer_token_into_pool_fee(&ctx, to, fee_of_to_token)?;
  transfer_token_out_of_pool(&ctx, to, to_amount)?;

  Ok(())
}


#[derive(Accounts)]
pub struct LpSwap<'info> {
  #[account(
    mut,
    seeds = [
      LP_SEED_PREFIX,
      token_quote.key().as_ref()
    ],
    bump,
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
    mut,
    associated_token::mint = token_quote,
    associated_token::authority = user,
  )]
  pub user_quote_ata: Account<'info, token::TokenAccount>,

  #[account(
  mut,
  associated_token::mint = token_quote,
  associated_token::authority = liquidity_pool_fee,
  )]
  pub fee_ata: Account<'info, token::TokenAccount>,


  #[account(mut)]
  pub user: UncheckedAccount<'info>,

  // pub rent: Sysvar<'info, Rent>,
  pub system_program: Program<'info, System>,
  pub token_program: Program<'info, token::Token>,
  pub associated_token_program: Program<'info, associated_token::AssociatedToken>,
}

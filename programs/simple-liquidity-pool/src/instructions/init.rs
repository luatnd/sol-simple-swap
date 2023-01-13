use anchor_lang::prelude::*;
use anchor_spl::{
  token,
  token::spl_token,
  associated_token,
};
use crate::state::{FixedRateLP, FixedRateLpFee, LP_SEED_PREFIX, LP_FEE_SEED_PREFIX};


pub fn init(ctx: Context<LpInit>, fixed_rate: u32) -> Result<()> {
  let lp = &mut ctx.accounts.liquidity_pool;
  // msg!("Initializing liquidity pool {:?}", lp);

  let lp_bump = *ctx.bumps.get("liquidity_pool").unwrap();
  lp.init(
    spl_token::native_mint::id(),
    ctx.accounts.token_quote.key(),
    // ctx.accounts.base_ata.key(),
    ctx.accounts.quote_ata.key(),
    fixed_rate,
    lp_bump,
  )?;

  Ok(())
}


#[derive(Accounts)]
pub struct LpInit<'info> {
  // Can be hacked? because public auth here
  #[account(
    init,
    payer = authority,
    space = 8 + FixedRateLP::MAXIMUM_SIZE,
    seeds = [
      LP_SEED_PREFIX,
      token_quote.key().as_ref()
    ],
    bump,
  )]
  pub liquidity_pool: Account<'info, FixedRateLP>,

  #[account(
    init,
    payer = authority,
    space = 8 + 0,
    seeds = [LP_FEE_SEED_PREFIX, token_quote.key().as_ref()],
    bump,
  )]
  pub liquidity_pool_fee: Account<'info, FixedRateLpFee>,

  // base Token Mint Address: Read more in README.md
  // #[account()]
  // pub token_base: Account<'info, token::Mint>,
  #[account()]
  pub token_quote: Account<'info, token::Mint>,

  // #[account(
  //   init,
  //   payer = authority,
  //   associated_token::mint = token_base,
  //   associated_token::authority = liquidity_pool,
  // )]
  // pub base_ata: Account<'info, token::TokenAccount>,
  #[account(
    init,
    payer = authority,
    associated_token::mint = token_quote,
    associated_token::authority = liquidity_pool,
  )]
  pub quote_ata: Account<'info, token::TokenAccount>,

  #[account(
    init,
    payer = authority,
    associated_token::mint = token_quote,
    associated_token::authority = liquidity_pool_fee,
  )]
  pub fee_ata: Account<'info, token::TokenAccount>,


  #[account(mut)]
  pub authority: Signer<'info>,

  pub rent: Sysvar<'info, Rent>,
  pub system_program: Program<'info, System>,
  pub token_program: Program<'info, token::Token>,
  pub associated_token_program: Program<'info, associated_token::AssociatedToken>,
}

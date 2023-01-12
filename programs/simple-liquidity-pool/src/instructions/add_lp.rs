use anchor_lang::prelude::*;
use anchor_lang::system_program;
use anchor_spl::{
  token,
  token::spl_token,
  associated_token,
};
use crate::state::{FixedRateLP, LP_SEED_PREFIX};

pub fn add_liquidity(
  ctx: Context<LpAddLiquidity>,
  base_amount: u64,
  quote_amount: u64,
) -> Result<()> {
  transfer_token_into_pool(&ctx, spl_token::native_mint::id(), base_amount)?;
  transfer_token_into_pool(&ctx, ctx.accounts.token_quote.key(), quote_amount)?;

  Ok(())
}

///
/// Transfer token from user wallet into pool
///
fn transfer_token_into_pool<'info>(
  ctx: &Context<LpAddLiquidity<'info>>,
  for_token: Pubkey,
  amount: u64,
) -> Result<()> {
  msg!("[transfer_token_into_pool] Transferring {} {} tokens ...", amount, for_token.key().to_string());

  let is_native_and_base_token = for_token == spl_token::native_mint::id();

  if is_native_and_base_token {
    // case native SOL
    system_program::transfer(
      CpiContext::new(
        ctx.accounts.system_program.to_account_info(),
        system_program::Transfer {
          from: ctx.accounts.authority.to_account_info(),
          to: ctx.accounts.liquidity_pool.to_account_info(),
        },
      ),
      amount,
    )
  } else {
    // case SPL token
    token::transfer(
      CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        token::Transfer {
          from: ctx.accounts.user_quote_ata.to_account_info(),
          to: ctx.accounts.quote_ata.to_account_info(),
          authority: ctx.accounts.authority.to_account_info(),
        },
      ),
      amount,
    )
  }
}

fn transfer_token_out_of_pool<'info>(
  ctx: &Context<LpAddLiquidity<'info>>,
  for_token: Pubkey,
  amount: u64,
) -> Result<()> {
  msg!("[transfer_token_into_pool] Transferring {} {} tokens ...", amount, for_token.key().to_string());

  let is_native_and_base_token = for_token == spl_token::native_mint::id();

  if is_native_and_base_token {
    // case native SOL
    system_program::transfer(
      CpiContext::new(
        ctx.accounts.system_program.to_account_info(),
        system_program::Transfer {
          from: ctx.accounts.liquidity_pool.to_account_info(),
          to: ctx.accounts.authority.to_account_info(),
        },
      ),
      amount,
    )
  } else {
    // case SPL token
    token::transfer(
      CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        token::Transfer {
          from: ctx.accounts.quote_ata.to_account_info(),
          to: ctx.accounts.user_quote_ata.to_account_info(),
          authority: ctx.accounts.liquidity_pool.to_account_info(),
        },
      ),
      amount,
    )
  }
}


#[derive(Accounts)]
pub struct LpAddLiquidity<'info> {
  #[account(
    mut,
    seeds = [
      LP_SEED_PREFIX,
      // token_base.key().as_ref(),
      token_quote.key().as_ref()
    ],
    bump,
  )]
  pub liquidity_pool: Account<'info, FixedRateLP>,

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
    associated_token::authority = authority,
  )]
  pub user_quote_ata: Account<'info, token::TokenAccount>,


  #[account(mut)]
  pub authority: Signer<'info>,

  // pub rent: Sysvar<'info, Rent>,
  pub system_program: Program<'info, System>,
  pub token_program: Program<'info, token::Token>,
  pub associated_token_program: Program<'info, associated_token::AssociatedToken>,
}

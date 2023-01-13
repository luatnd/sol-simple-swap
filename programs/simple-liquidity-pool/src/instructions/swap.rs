use anchor_lang::prelude::*;
use anchor_lang::system_program;
use anchor_spl::{
  token,
  token::spl_token,
  associated_token,
};
use crate::state::{
  FixedRateLP,
  LP_SEED_PREFIX, LP_LIQUIDITY_PREFIX, LP_FEE_SEED_PREFIX,
};


pub fn swap(
  ctx: Context<LpSwap>,
  from: Pubkey,
  to: Pubkey,
  from_amount: u64,
) -> Result<()> {
  let lp = &mut ctx.accounts.lp;

  let current_base_liquidity: u64 = ctx.accounts.lp_liquidity.lamports();
  let current_quote_liquidity: u64 = ctx.accounts.lp_liquidity_quote_ata.amount;

  let (
    _,
    from_amount,
    to_amount_without_fee,
    fee_of_to_token
  ) = lp.preview_swap(from, to, from_amount, current_base_liquidity, current_quote_liquidity)?;

  let to_amount = to_amount_without_fee - fee_of_to_token;

  transfer_token_into_pool(&ctx, from, from_amount, true)?;
  transfer_token_into_pool(&ctx, to, fee_of_to_token, false)?;
  transfer_token_out_of_pool(&ctx, to, to_amount)?;

  Ok(())
}

#[derive(Accounts)]
pub struct LpSwap<'info> {
  // lp state data
  #[account(
    mut,
    seeds = [
      LP_SEED_PREFIX,
      token_quote.key().as_ref()
    ],
    bump = lp.bump,
  )]
  pub lp: Account<'info, FixedRateLP>,

  #[account(mut)]
  pub token_quote: Account<'info, token::Mint>,


  // lp liquidity: store SOL liquidity
  // `lp` account is state, contain data so cannot be used as SOL sender
  /// CHECK: Just to store SOL
  #[account(
    mut,
    seeds = [
      LP_LIQUIDITY_PREFIX,
      token_quote.key().as_ref()
    ],
    bump = lp.liquidity_bump,
  )]
  pub lp_liquidity: UncheckedAccount<'info>,

  // lp liquidity: store SPL liquidity
  #[account(
    mut,
    associated_token::mint = token_quote,
    associated_token::authority = lp_liquidity,
  )]
  pub lp_liquidity_quote_ata: Account<'info, token::TokenAccount>,

  // lp fee: store SOL fee collected, for profit sharing later
  /// CHECK: Just to store SOL
  #[account(
    mut,
    seeds = [LP_FEE_SEED_PREFIX, token_quote.key().as_ref()],
    bump = lp.fee_bump,
  )]
  pub lp_fee: UncheckedAccount<'info>,

  // lp fee: store SPL fee collected, for profit sharing later
  #[account(
    mut,
    associated_token::mint = token_quote,
    associated_token::authority = lp_fee,
  )]
  pub lp_fee_quote_ata: Account<'info, token::TokenAccount>,


  #[account(
    init_if_needed,
    payer = user,
    associated_token::mint = token_quote,
    associated_token::authority = user,
  )]
  pub user_quote_ata: Account<'info, token::TokenAccount>,


  #[account(mut)]
  pub user: Signer<'info>,
  // pub user: UncheckedAccount<'info>,

  // pub rent: Sysvar<'info, Rent>,
  pub system_program: Program<'info, System>,
  pub token_program: Program<'info, token::Token>,
  pub associated_token_program: Program<'info, associated_token::AssociatedToken>,
}


///
/// Transfer token from user wallet into pool
///
fn transfer_token_into_pool<'info>(
  ctx: &Context<LpSwap<'info>>,
  for_token: Pubkey,
  amount: u64,
  store_in_liquidity: bool, // liquidity or fee, will expand to enum if have some more destinations
) -> Result<()> {
  msg!("[transfer_token_into_pool] Transferring {} {} tokens ...", amount, for_token.key().to_string());

  let is_native_and_base_token = for_token == spl_token::native_mint::id();

  if is_native_and_base_token {
    // case native SOL
    system_program::transfer(
      CpiContext::new(
        ctx.accounts.system_program.to_account_info(),
        system_program::Transfer {
          from: ctx.accounts.user.to_account_info(),
          to: if store_in_liquidity {
            ctx.accounts.lp_fee.to_account_info()
          } else {
            ctx.accounts.lp_liquidity.to_account_info()
          },
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
          to: if store_in_liquidity {
            ctx.accounts.lp_fee_quote_ata.to_account_info()
          } else {
            ctx.accounts.lp_liquidity_quote_ata.to_account_info()
          },
          authority: ctx.accounts.user.to_account_info(),
        },
      ),
      amount,
    )
  }
}

fn transfer_token_out_of_pool<'info>(
  ctx: &Context<LpSwap<'info>>,
  for_token: Pubkey,
  amount: u64,
) -> Result<()> {
  msg!("[transfer_token_out_of_pool] Transferring {} {} tokens ...", amount, for_token.key().to_string());

  let token_quote_pubkey = ctx.accounts.token_quote.key().clone();
  let bump = ctx.accounts.lp.bump;
  msg!("[transfer_token_out_of_pool] liquidity_pool.bump: {}", bump);

  let signer_seeds: &[&[&[u8]]] = &[&[
    LP_LIQUIDITY_PREFIX,
    token_quote_pubkey.as_ref(),
    &[bump],
  ]];

  let is_native_and_base_token = for_token == spl_token::native_mint::id();

  if is_native_and_base_token {
    // case native SOL
    system_program::transfer(
      CpiContext::new(
        ctx.accounts.system_program.to_account_info(),
        system_program::Transfer {
          from: ctx.accounts.lp_liquidity.to_account_info(),
          to: ctx.accounts.user.to_account_info(),
        },
      ).with_signer(signer_seeds),
      amount,
    )
  } else {
    // case SPL token
    token::transfer(
      CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        token::Transfer {
          from: ctx.accounts.lp_liquidity_quote_ata.to_account_info(),
          to: ctx.accounts.user_quote_ata.to_account_info(),
          authority: ctx.accounts.lp_liquidity_quote_ata.to_account_info(),
        },
        signer_seeds,
      ),
      amount,
    )
  }
}

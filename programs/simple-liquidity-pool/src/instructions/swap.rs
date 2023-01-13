use anchor_lang::prelude::*;
use anchor_lang::system_program;
use anchor_spl::{
  token,
  associated_token,
};
use anchor_spl::token::spl_token;
use crate::instructions::LpAddLiquidity;
use crate::state::{
  FixedRateLP, FixedRateLpFee,
  LP_SEED_PREFIX, LP_FEE_SEED_PREFIX,
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
    mut,
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
  pub authority: UncheckedAccount<'info>,

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
  override_destination: Option<Account<'info, token::TokenAccount>>,
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
          to: if override_destination.is_some() {
            override_destination.unwrap().to_account_info()
          } else {
            ctx.accounts.liquidity_pool.to_account_info()
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
          to: if override_destination.is_some() {
            override_destination.unwrap().to_account_info()
          } else {
            ctx.accounts.quote_ata.to_account_info()
          },
          authority: ctx.accounts.authority.to_account_info(),
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
    let token_quote_pubkey = ctx.accounts.token_quote.key().clone();

    // get bump method 1
    let (auth, seed) = Pubkey::find_program_address(
      &[
        LP_SEED_PREFIX,
        token_quote_pubkey.as_ref(),
      ],
      ctx.program_id,
    );
    msg!("[transfer_token_out_of_pool] auth: {}, seed {}", auth.key().to_string(), seed);

    // get bump method 2
    let bump = ctx.accounts.liquidity_pool.bump;
    msg!("[transfer_token_out_of_pool] liquidity_pool.bump: {}", bump);

    let signer_seeds: &[&[&[u8]]] = &[&[
      LP_SEED_PREFIX,
      token_quote_pubkey.as_ref(),
      &[bump],
    ]];


    // let bump = ctx.accounts.liquidity_pool.bump;
    // let signer_seeds: &[&[&[u8]]] = &[&[&[bump][..]]];

    token::transfer(
      CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        token::Transfer {
          from: ctx.accounts.quote_ata.to_account_info(),
          to: ctx.accounts.user_quote_ata.to_account_info(),
          authority: ctx.accounts.liquidity_pool.to_account_info(),
        },
        // signer_seeds,
        signer_seeds,
      ),
      amount,
    )
  }
}

fn transfer_token_into_pool_fee<'info>(
  ctx: &Context<LpSwap<'info>>,
  for_token: Pubkey,
  amount: u64,
) -> Result<()> {
  transfer_token_into_pool(&ctx, for_token, amount, Some(ctx.accounts.fee_ata.clone()))
}

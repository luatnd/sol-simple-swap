use anchor_lang::prelude::*;
use anchor_lang::system_program;
use anchor_spl::{
  token,
  token::spl_token,
};
use crate::instructions::LpAddLiquidity;
use crate::state::{errors::*, SwapDir};

///
/// this LP is for <Sol, SplToken>
/// base token is hard coded to SOL
/// If we need to swap <Sol/SplToken, Sol/SplToken>,
/// We need to use Wrapped Sol,
/// which is not a requirement of this test
/// So I skip it for now
///
/// Base: SOL
/// Quote: SPL token
///
/// Base,Quote is a term in trading that represents the BASE/QUOTE trading pair
///
#[account]
#[derive(Default)]
pub struct FixedRateLP {
  /// 1 base = ? quote
  /// if 1 base = 10 quote then rate = 10,000 because of RATE_DECIMAL=3
  /// max rate = 2^(32-RATE_DECIMAL)
  pub rate: u32,            // 4

  // NOTE: base token is hardcoded to be native SOL
  pub token_base: Pubkey,   // 32

  // Use pool address to store base token SOL
  // pub amount_base_ata: Pubkey,  // 32

  pub token_quote: Pubkey,  // 32
  pub amount_quote_ata: Pubkey, // 32

  // NOTE: At this time: we read balance inside amount_base_ATA as the single source of truth
  // For better performance, need another complex implementation, such as cache the balance here
  // pub amount_base: u128,    // 16: current base token amount in this pool
  // pub amount_quote: u128,   // 16

  // profit tracking for all liquidity provider: Ignore this feature

  // misc
  pub bump: u8,                 // 1
}


#[account]
pub struct FixedRateLpFee {}

// pub enum LpType {
//   FixedRate, // 1A always = nB, n is fixed
//   ConstantProduct, // AMM
// }


//
// Mark as constant to Expose to Idl
// constant macro inside struct scope will not be exposed to Idl
//
#[constant]
pub const LP_SEED_PREFIX: &[u8] = b"FixedRateLP_";
#[constant]
pub const LP_FEE_SEED_PREFIX: &[u8] = b"FixedRateLP_fee_";

#[constant]
pub const LP_RATE_DECIMAL: u8 = 3;

/// Swap fee will be deducted directly on to_amount, not from_amount
#[constant]
pub const LP_SWAP_FEE_PERMIL: u8 = 50; // 50/1000 = 5.0%

impl FixedRateLP {
  // pub const SEED_PREFIX: &'static [u8] = b"FixedRateLP_";
  pub const MAXIMUM_SIZE: usize = 4 + 32 + 32 + 32 + 1;


  pub fn get_swap_dir(&self, from_token: Pubkey, to_token: Pubkey) -> Option<SwapDir> {
    let mut swap_dir: Option<SwapDir> = None;
    // if from_token is base token and to_token is quote token then swap_dir=BaseToQuote
    if from_token == self.token_base && to_token == self.token_quote {
      swap_dir = Option::from(SwapDir::BaseToQuote)
    } else if from_token == self.token_quote && to_token == self.token_base {
      swap_dir = Option::from(SwapDir::QuoteToBase)
    }

    swap_dir
  }
}

// impl LP for FixedRateLP {
impl FixedRateLP {
  pub fn init(
    &mut self,
    token_base: Pubkey,
    token_quote: Pubkey,
    // amount_base_ata: Pubkey,
    amount_quote_ata: Pubkey,
    fixed_rate: u32,
    bump: u8,
  ) -> Result<()> {
    require_gt!(fixed_rate, 0, LpBaseError::InvalidRate);
    require!(fixed_rate <= 2_u32.pow(32 - LP_RATE_DECIMAL as u32), LpBaseError::InvalidRate);

    self.rate = fixed_rate;
    self.token_base = token_base;
    self.token_quote = token_quote;
    // self.amount_base_ata = amount_base_ata;
    self.amount_quote_ata = amount_quote_ata;
    // self.amount_base = 0;
    // self.amount_quote = 0;

    self.bump = bump;

    Ok(())
  }

  pub fn add_liquidity(&mut self, token_base_amount: u64, token_quote_amount: u64) -> Result<()> {
    require_gte!(token_base_amount, 0, LpBaseError::InvalidAmount);
    require_gte!(token_quote_amount, 0, LpBaseError::InvalidAmount);

    // TODO: Validate current amount over upper range of u64

    // self.amount_base += token_base_amount;
    // self.amount_quote += token_quote_amount;

    Ok(())
  }

  ///
  /// Return (
  ///   from_amount: base token change amount,
  ///   to_amount_without_fee: quote token change amount,
  ///   fee: the fee deducted on to_amount,
  /// )
  ///
  pub fn preview_swap(
    &mut self,
    from_token: Pubkey,
    to_token: Pubkey,
    from_amount: u64,
    current_base_liquidity: u64,
    current_quote_liquidity: u64,
  ) -> Result<(SwapDir, u64, u64, u64)> {
    require_gt!(from_amount, 0, LpBaseError::InvalidSwapAmount);

    let swap_direction = self.get_swap_dir(from_token, to_token);
    require!(swap_direction.is_some(), LpBaseError::InvalidSwapToken);

    let swap_dir = swap_direction.unwrap();
    // let (current_base_liquidity, current_quote_liquidity) = FixedRateLP::get_current_liquidity();

    let verbose = false;
    if verbose { msg!("[preview_swap] current base, quote liquidity: {}, {}", current_base_liquidity, current_quote_liquidity); }

    let rate: f64 = self.rate as f64 / 1000_f64;
    if verbose { msg!("[preview_swap] rate: {}", rate); }

    let to_amount = match swap_dir {
      SwapDir::BaseToQuote => {
        let to_amount = (from_amount as f64 * rate) as u64;
        require!(to_amount <= current_quote_liquidity, LpBaseError::InsufficientQuoteAmount);
        to_amount
      },
      SwapDir::QuoteToBase => {
        let to_amount = (from_amount as f64 / rate) as u64;
        require!(to_amount <= current_base_liquidity, LpBaseError::InsufficientBaseAmount);
        to_amount
      }
    };
    if verbose { msg!("[preview_swap] to_amount: {}", to_amount); }

    let fee = FixedRateLP::get_swap_fee(to_amount);

    Ok((swap_dir, from_amount, to_amount, fee))
  }

  fn get_swap_fee(to_amount: u64) -> u64 {
    return to_amount * (LP_SWAP_FEE_PERMIL as u64) / 1000;
  }

  // /// @return (current_base_amount_available, current_quote_amount_available)
  // fn get_current_liquidity() -> (u64, u64) {
  //   return (0, 0);
  //   // todo!()
  // }
}


// pub fn get_current_pool_liquidity<'info>(ctx: &Context<LpAddLiquidity<'info>>) -> Result<(u64, u64)> {
//   let current_base_liquidity: u64 = ctx.accounts.liquidity_pool.to_account_info().lamports();
//   let current_quote_liquidity: u64 = ctx.accounts.quote_ata.amount;
//
//   Ok((current_base_liquidity, current_quote_liquidity))
// }

///
/// Transfer token from user wallet into pool
///
pub fn transfer_token_into_pool<'info>(
  ctx: &Context<LpAddLiquidity<'info>>,
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

pub fn transfer_token_out_of_pool<'info>(
  ctx: &Context<LpAddLiquidity<'info>>,
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

    let (auth, seed) = Pubkey::find_program_address(
      &[
        LP_SEED_PREFIX,
        token_quote_pubkey.as_ref(),
      ],
      ctx.program_id,
    );
    msg!("[transfer_token_out_of_pool] auth: {}, seed {}", auth.key().to_string(), seed);

    let signer_seeds: &[&[&[u8]]] = &[&[
      LP_SEED_PREFIX,
      token_quote_pubkey.as_ref(),
      &[seed],
    ]];

    token::transfer(
      CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        token::Transfer {
          from: ctx.accounts.quote_ata.to_account_info(),
          to: ctx.accounts.user_quote_ata.to_account_info(),
          authority: ctx.accounts.liquidity_pool.to_account_info(),
        },
        signer_seeds,
      ),
      amount,
    )
  }
}

pub fn transfer_token_into_pool_fee<'info>(
  ctx: &Context<LpAddLiquidity<'info>>,
  for_token: Pubkey,
  amount: u64,
) -> Result<()> {
  transfer_token_into_pool(&ctx, for_token, amount, Some(ctx.accounts.fee_ata.clone()))
}

// #[cfg(test)]
// mod tests {
//   #[test]
//   fn exploration() {
//     assert_eq!(2 + 2, 4);
//   }
// }

use anchor_lang::prelude::*;
use crate::state::{errors::*, SwapDir};

#[derive(Debug)]
#[account]
pub struct FixedRateLP {
  /// 1 base = ? quote
  /// if 1 base = 10 quote then rate = 10,000 because of RATE_DECIMAL=3
  /// max rate = 2^(32-RATE_DECIMAL)
  pub rate: u32,            // 4
  pub token_base: Pubkey,   // 32
  pub token_quote: Pubkey,  // 32
  pub amount_base_ata: Pubkey,  // 32
  pub amount_quote_ata: Pubkey, // 32

  // NOTE: At this time: we read balance inside amount_base_ATA as the single source of truth
  // For better performance, need another complex implementation, such as cache the balance here
  // pub amount_base: u128,    // 16: current base token amount in this pool
  // pub amount_quote: u128,   // 16

  // profit tracking for all liquidity provider: Ignore this feature
}

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
pub const LP_RATE_DECIMAL: u8 = 3;

/// Swap fee will be deducted directly on to_amount, not from_amount
#[constant]
pub const LP_SWAP_FEE_PERMIL: u8 = 30; // 25/1000 = 2.5%

impl FixedRateLP {
  // pub const SEED_PREFIX: &'static [u8] = b"FixedRateLP_";
  pub const MAXIMUM_SIZE: usize = 4 + 32 + 32 + 32 + 32;// + 16 + 16;


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
    token_base: Pubkey, token_quote: Pubkey,
    amount_base_ata: Pubkey, amount_quote_ata: Pubkey,
    fixed_rate: u32
  ) -> Result<()> {
    require_gt!(fixed_rate, 0, LpBaseError::InvalidRate);
    require!(fixed_rate <= 2_u32.pow(32 - LP_RATE_DECIMAL as u32), LpBaseError::InvalidRate);

    self.rate = fixed_rate;
    self.token_base = token_base;
    self.token_quote = token_quote;
    self.amount_base_ata = amount_base_ata;
    self.amount_quote_ata = amount_quote_ata;
    // self.amount_base = 0;
    // self.amount_quote = 0;

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
  pub fn get_swap_data(&mut self, from_token: Pubkey, to_token: Pubkey, from_amount: u64) -> Result<(u64, u64, u64)> {
    require_gt!(from_amount, 0, LpBaseError::InvalidSwapAmount);

    let swap_direction = self.get_swap_dir(from_token, to_token);
    require!(swap_direction.is_some(), LpBaseError::InvalidSwapToken);

    let swap_dir = swap_direction.unwrap();
    let (amount_base, amount_quote) = FixedRateLP::get_current_liquidity();

    let to_amount = match swap_dir {
      SwapDir::BaseToQuote => {
        let to_amount = from_amount * self.rate as u64;
        require!(to_amount <= amount_quote, LpBaseError::InsufficientQuoteAmount);
        to_amount
      },
      SwapDir::QuoteToBase => {
        let to_amount = from_amount / self.rate as u64;
        require!(to_amount <= amount_base, LpBaseError::InsufficientBaseAmount);
        to_amount
      }
    };

    let fee = FixedRateLP::get_swap_fee(to_amount);
    require!(to_amount - fee <= 2_u64.pow(64), LpBaseError::LargeSwapAmount);


    // swap
    // let (mut lp_base_change, mut lp_quote_change): (i128, i128) = (0_i128, 0_i128);
    // match swap_dir {
    //   SwapDir::BaseToQuote => {
    //     lp_base_change = from_amount as i128;           // in
    //     lp_quote_change = -((to_amount + fee) as i128); // out
    //   },
    //   SwapDir::QuoteToBase => {
    //     lp_base_change = -((to_amount - fee) as i128); // out
    //     lp_quote_change = from_amount as i128;         // in
    //   }
    // };

    Ok((from_amount, to_amount, fee))
  }

  fn get_swap_fee(to_amount: u64) -> u64 {
    return to_amount * (LP_SWAP_FEE_PERMIL as u64) / 1000;
  }

  /// @return (current_base_amount_available, current_quote_amount_available)
  fn get_current_liquidity() -> (u64, u64) {
    return (0, 0);
    // todo!()
  }
}

// #[cfg(test)]
// mod tests {
//   #[test]
//   fn exploration() {
//     assert_eq!(2 + 2, 4);
//   }
// }

mod instructions;
mod state;
mod errors;

use anchor_lang::prelude::*;
use instructions::*; // Must import as * to avoid error


declare_id!("GMDA6SqHUFzctniBczeBSsoLEfd3HaW161wwyAms2buL");

#[program]
pub mod simple_liquidity_pool {
  use super::*;

  pub fn initialize(ctx: Context<LpInit>, fixed_rate: u32) -> Result<()> {
    init::init(ctx, fixed_rate)
  }

  pub fn add_liquidity(ctx: Context<LpAddLiquidity>, base_amount: u64, quote_amount: u64) -> Result<()> {
    add_lp::add_liquidity(ctx, base_amount, quote_amount)
  }

  // pub fn swap(ctx: Context<Tmp>) -> Result<()> {
  //   todo!()
  // }
  //
  // pub fn withdraw_liquidity(ctx: Context<LpAddLiquidity>, base_amount: u64, quote_amount: u64) -> Result<()> {
  //   todo!()
  // }
  //
  // pub fn destruct(ctx: Context<Tmp>, from: Token, to: Token, fromAmount: int) -> Result<()> {
  //   todo!()
  // }

}

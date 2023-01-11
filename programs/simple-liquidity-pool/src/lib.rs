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

  // pub fn add_liquidity(ctx: Context<Tmp>) -> Result<()> {
  //   Ok(())
  // }
  //
  // pub fn swap(ctx: Context<Tmp>) -> Result<()> {
  //   Ok(())
  // }

  // pub fn withdraw_liquidity(from: Token, to: Token, fromAmount: int) -> Result<()> {
  //   Ok(())
  // }

  // pub fn destruct(from: Token, to: Token, fromAmount: int) -> Result<()> {

}

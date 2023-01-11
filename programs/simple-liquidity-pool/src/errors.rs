use anchor_lang::prelude::*;

#[error_code]
pub enum LpError {
  #[msg("Fixed rate must be > 0")]
  InvalidRate,
}

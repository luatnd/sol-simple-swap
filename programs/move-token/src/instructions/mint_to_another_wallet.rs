use anchor_lang::prelude::*;
use anchor_spl::{
  token,
  associated_token,
};
use crate::instructions::MintAuthorityPda;


pub fn mint_to_another_wallet(
  ctx: Context<MintToAnotherWallet>,
  amount: u64,
  mint_authority_pda_bump: u8,
) -> Result<()> {

  msg!("Minting token to token account...");
  msg!("Mint: {}", &ctx.accounts.mint_account.to_account_info().key());
  msg!("Associated Token Address (ATA): {}", &ctx.accounts.recipient_ata.key());
  token::mint_to(
    CpiContext::new_with_signer(
      ctx.accounts.token_program.to_account_info(),
      token::MintTo {
        mint: ctx.accounts.mint_account.to_account_info(),
        to: ctx.accounts.recipient_ata.to_account_info(),
        authority: ctx.accounts.mint_authority.to_account_info(),
      },
      &[&[
        b"mint_authority_",
        ctx.accounts.mint_account.key().as_ref(),
        &[mint_authority_pda_bump],
      ]]
    ),
    amount,
  )?;

  msg!("Token minted to wallet successfully.");

  Ok(())
}


#[derive(Accounts)]
#[instruction(amount: u64, mint_authority_pda_bump: u8)]
pub struct MintToAnotherWallet<'info> {
  #[account(
    mut,
    mint::decimals = 9,
    mint::authority = mint_authority.key(),
  )]
  pub mint_account: Account<'info, token::Mint>,

  #[account(
    mut,
    seeds = [
      b"mint_authority_",
      mint_account.key().as_ref()
    ],
    bump = mint_authority_pda_bump
  )]
  pub mint_authority: Account<'info, MintAuthorityPda>,

  /// CHECK: This is for airdrops
  pub recipient: UncheckedAccount<'info>,

  // TODO: init always create ATA,
  // so if this is not first time, the program will failed to create ATA
  // Then tx was failed
  // But it's good in this case because the logic is we only airdrop to a user once
  #[account(
    init,
    payer = payer,
    associated_token::mint = mint_account,
    associated_token::authority = recipient,
  )]
  pub recipient_ata: Account<'info, token::TokenAccount>,

  #[account(mut)]
  pub payer: Signer<'info>,
  pub rent: Sysvar<'info, Rent>,
  pub system_program: Program<'info, System>,
  pub token_program: Program<'info, token::Token>,

  pub associated_token_program: Program<'info, associated_token::AssociatedToken>,
}

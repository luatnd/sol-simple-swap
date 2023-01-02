use anchor_lang::{
  prelude::*,
  solana_program::program::invoke_signed,
};
use anchor_spl::{
  token,
};
use mpl_token_metadata::{
  instruction as mpl_instruction,
};


pub fn create_token(
  ctx: Context<CreateTokenMint>,
  title: String,
  symbol: String,
  metadata_uri: String,
  mint_authority_pda_bump: u8,
) -> Result<()> {
  msg!("[move_token.create_token] Metadata account address: {}", &ctx.accounts.metadata_account.key());

  // More detail: https://docs.metaplex.com/programs/token-metadata/instructions
  // Cross Program Call Depth index: 0
  let ix = mpl_instruction::create_metadata_accounts_v3(
    ctx.accounts.token_metadata_program.key(),
    ctx.accounts.metadata_account.key(),
    ctx.accounts.mint_account.key(),
    ctx.accounts.mint_authority.key(),
    ctx.accounts.payer.key(),
    ctx.accounts.mint_authority.key(),
    title,
    symbol,
    metadata_uri,
    None,
    0,
    true,
    false,
    None,
    None,
    None,
  );
  let accounts = [
    ctx.accounts.metadata_account.to_account_info(),
    ctx.accounts.mint_account.to_account_info(),
    ctx.accounts.payer.to_account_info(),   // Mint Authority
    ctx.accounts.payer.to_account_info(),   // payer
    ctx.accounts.payer.to_account_info(),   // Update Authority
    ctx.accounts.rent.to_account_info(),
  ];

  let binding = ctx.accounts.mint_account.key();
  let seeds: &[&[&[u8]]] = &[&[
    b"mint_authority_",
    &binding.as_ref(),
    &[mint_authority_pda_bump],
  ]];

  invoke_signed(&ix, &accounts, seeds)?;
  msg!("[move_token.create_token] Done");

  Ok(())
}


#[derive(Accounts)]
pub struct CreateTokenMint<'info> {
  // can I rm some of this props?
  #[account(
    init,
    payer = payer,
    mint::decimals = 9,
    mint::authority = mint_authority,
    mint::freeze_authority = payer,
  )]
  pub mint_account: Account<'info, token::Mint>,

  #[account(
    init,
    payer = payer,
    space = 8 + 32,
    seeds = [
      b"mint_authority_",
      mint_account.key().as_ref(),
    ],
    bump
  )]
  pub mint_authority: Account<'info, MintAuthorityPda>,

  #[account(mut)]
  pub payer: Signer<'info>,
  pub rent: Sysvar<'info, Rent>,

  /// _CHECK: We're about to create this with Metaplex
  #[account(mut)]
  pub metadata_account: UncheckedAccount<'info>,

  pub system_program: Program<'info, System>,
  pub token_program: Program<'info, token::Token>,
  /// _CHECK: Metaplex will check this
  pub token_metadata_program: UncheckedAccount<'info>,
}

#[account]
pub struct MintAuthorityPda {}

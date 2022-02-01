use anchor_lang::prelude::*;
use solana_program;
use solana_program::{
    account_info::AccountInfo,
    entrypoint::ProgramResult, program::invoke, system_instruction,
};

declare_id!("DBhPne1rBZNjSMVvPM6gkUzMH486vSZVxoSomzQmaqkq");

#[program]
pub mod hello {
    use super::*;
    pub fn init_house(ctx: Context<InitHouse>, bump: u8) -> ProgramResult {
        // Debit from_account and credit to_account
        let user = &mut ctx.accounts.user;
        let system_program = &ctx.accounts.system_program;
        let base_account = &ctx.accounts.base_account;
        msg!("Base account");
        msg!(&base_account.to_account_info().owner.to_string());
        msg!("User account");
        msg!(&user.to_account_info().owner.to_string());

        invoke(
            &system_instruction::transfer(
                &user.to_account_info().key,
                &base_account.to_account_info().key,
                100_000, // 0.001 SOL
            ),
            &[
                user.to_account_info().clone(),
                base_account.to_account_info().clone(),
                system_program.to_account_info().clone(),
            ],
        )?;
        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(bump: u8)]
pub struct InitHouse<'info> {
  #[account(init, payer = user, space = 9000, seeds = [], bump = bump)]
  pub base_account: Account<'info, BaseAccount>,
  #[account(mut)]
  pub user: Signer<'info>,
  pub system_program: Program<'info, System>,
}

#[account]
pub struct BaseAccount {
}
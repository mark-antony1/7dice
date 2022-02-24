use anchor_lang::prelude::*;
// use anchor_spl::token;
use solana_program;
use solana_program::{
    // clock::Clock,
    account_info::AccountInfo,
    entrypoint::ProgramResult, program::invoke, system_instruction,
};
pub use switchboard_v2::{VrfAccountData, VrfRequestRandomness};
const MAX_VALUE: u64 = 100;


declare_id!("GJmxJGYZETm142yHTQVasceWxWSVzC1Zi86UrCEpgrhK");

#[program]
pub mod hello {
    use super::*;
    pub fn init_house(ctx: Context<InitHouse>, bump: u8) -> ProgramResult {
        // Debit from_account and credit to_account
        let user = &mut ctx.accounts.user;
        let system_program = &ctx.accounts.system_program;
        let house_vault = &ctx.accounts.house_vault;
        invoke(
            &system_instruction::transfer(
                &user.to_account_info().key,
                &house_vault.to_account_info().key,
                1_000_000, // 0.01 SOL
            ),
            &[
                user.to_account_info().clone(),
                house_vault.to_account_info().clone(),
                system_program.to_account_info().clone(),
            ],
        )?;
        Ok(())
    }

    pub fn gamble(ctx: Context<Gamble>, _params: RequestResultParams) -> ProgramResult {
        // Debit from_account and credit to_account
        msg!("in gamble");
        let user = &mut ctx.accounts.user;
        let system_program = &ctx.accounts.system_program;
        let house_vault = &ctx.accounts.house_vault;
        invoke(
            &system_instruction::transfer(
                &user.to_account_info().key,
                &house_vault.to_account_info().key,
                100_000, // 0.001 SOL
            ),
            &[
                user.to_account_info().clone(),
                house_vault.to_account_info().clone(),
                system_program.to_account_info().clone(),
            ],
        )?;

        let switchboard_program = ctx.accounts.switchboard_program.to_account_info();

        let vrf_request_randomness = VrfRequestRandomness {
            authority: ctx.accounts.authority.to_account_info(),
            vrf: ctx.accounts.vrf.to_account_info(),
            oracle_queue: ctx.accounts.oracle_queue.to_account_info(),
            queue_authority: ctx.accounts.queue_authority.to_account_info(),
            data_buffer: ctx.accounts.data_buffer.to_account_info(),
            permission: ctx.accounts.permission.to_account_info(),
            escrow: ctx.accounts.escrow.to_account_info(),
            payer_wallet: ctx.accounts.payer_wallet.to_account_info(),
            payer_authority: ctx.accounts.payer_authority.to_account_info(),
            recent_blockhashes: ctx.accounts.recent_blockhashes.to_account_info(),
            program_state: ctx.accounts.program_state.to_account_info(),
            token_program: ctx.accounts.token_program.to_account_info(),
        };

        msg!("requesting randomness");
        vrf_request_randomness.invoke(
            switchboard_program,
            _params.state_bump,
            _params.permission_bump,
        )?;
        Ok(())
    }
}

#[derive(Clone, AnchorSerialize, AnchorDeserialize)]
pub struct RequestResultParams {
    pub permission_bump: u8,
    pub state_bump: u8,
}

#[derive(Accounts)]
pub struct Gamble<'info> {
    #[account(mut)]
    pub house_vault: UncheckedAccount<'info>,
    pub vrf_account: UncheckedAccount<'info>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
    pub switchboard_program: AccountInfo<'info>,
    #[account(mut)]
    pub authority: AccountInfo<'info>,
    #[account(mut)]
    pub vrf: AccountInfo<'info>,
    #[account(mut)]
    pub oracle_queue: AccountInfo<'info>,
    pub queue_authority: AccountInfo<'info>,
    pub data_buffer: AccountInfo<'info>,
    #[account(mut)]
    pub permission: AccountInfo<'info>,
    #[account(mut)]
    pub escrow: AccountInfo<'info>,
    #[account(mut)]
    pub payer_wallet: AccountInfo<'info>,
    #[account(mut)]
    pub payer_authority: AccountInfo<'info>,
    #[account(address = solana_program::sysvar::recent_blockhashes::ID)]
    pub recent_blockhashes: AccountInfo<'info>,
    pub program_state: AccountInfo<'info>,
    #[account(address = anchor_spl::token::ID)]
    pub token_program: AccountInfo<'info>,
}

#[derive(Accounts)]
#[instruction(bump: u8)]
pub struct InitHouse<'info> {
    #[account(mut)]
    pub house_vault: UncheckedAccount<'info>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}

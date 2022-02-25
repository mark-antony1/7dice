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
// const ZERO_ADDRESS: Pubkey = Pubkey::new(&[0;32]);

declare_id!("GJmxJGYZETm142yHTQVasceWxWSVzC1Zi86UrCEpgrhK");

#[program]
pub mod hello {
    use switchboard_protos::protos::vrf;

    use super::*;
    pub fn init_house(ctx: Context<InitHouse>, bump: u8) -> ProgramResult {
        // Debit from_account and credit to_account
        let user = &mut ctx.accounts.user;
        let system_program = &ctx.accounts.system_program;
        let house_vault = &ctx.accounts.house_vault;

        let house_state = &mut ctx.accounts.house_state;
        house_state.vrf_account = ctx.accounts.vrf.key.clone();

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
        let zero = Pubkey::new(&[0;32]);

        if ctx.accounts.house_state.reward_address == zero {
            return Err(ErrorCode::MaxResultExceedsMaximum.into());
        }

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
            payer_wallet: ctx.accounts.user.to_account_info(),
            payer_authority: ctx.accounts.user.to_account_info(),
            recent_blockhashes: ctx.accounts.recent_blockhashes.to_account_info(),
            program_state: ctx.accounts.program_state.to_account_info(),
            token_program: ctx.accounts.token_program.to_account_info(),
        };
        let vrf = VrfAccountData::new(&ctx.accounts.vrf)?;
        let next_counter = vrf.counter + 1;
        let house_state = &mut ctx.accounts.house_state;
        house_state.vrf_counter = next_counter as u64;
        house_state.reward_address = ctx.accounts.user.key.clone();

        msg!("requesting randomness");
        vrf_request_randomness.invoke(
            switchboard_program,
            _params.state_bump,
            _params.permission_bump,
        )?;
        Ok(())
    }

    pub fn settle_gamble(ctx: Context<SettleGamble>) -> ProgramResult {
        // Debit from_account and credit to_account
        msg!("in gamble");
        // let user = &mut ctx.accounts.user;
        let system_program = &ctx.accounts.system_program;
        let house_vault = &ctx.accounts.house_vault;
        let vrf_account_info = &ctx.accounts.vrf;
        let vrf_data = VrfAccountData::new(vrf_account_info)?;
        let result_buffer = vrf_data.get_result()?;
        // how to convert result_buffer to a number
        // let result_as_number = fresult_buffer
        // modulo number by 100
        // check if number is greater than 49
        // if so send user money
        // if not do nothing
        let house_state = &mut ctx.accounts.house_state;

        invoke(
            &system_instruction::transfer(
                &house_vault.to_account_info().key,
                &ctx.accounts.house_state.reward_address,
                100_000, // 0.001 SOL
            ),
            &[
                // why is to account info unavailable? How to convert to account info?
                ctx.accounts.house_state.reward_address.to_account_info().clone(),
                house_vault.to_account_info().clone(),
                system_program.to_account_info().clone(),
            ],
        )?;

        house_state.reward_address = Pubkey::new(&[0;32]);
        Ok(())
    }
}

#[derive(Clone, AnchorSerialize, AnchorDeserialize)]
pub struct RequestResultParams {
    pub permission_bump: u8,
    pub state_bump: u8,
    pub house_vault_bump: u8,
    pub house_state_bump: u8,
}

#[derive(Accounts)]
#[instruction(bumps: RequestResultParams)]
pub struct Gamble<'info> {
    #[account(
        mut,
        seeds=[b"house-state"],
        bump=bumps.house_state_bump,
        constraint =(house_state.vrf_account ==  vrf.key())
    )]
    pub house_state: Account<'info, HouseState>,
    #[account(mut)]
    pub house_vault: UncheckedAccount<'info>,
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
    #[account(
        mut,
        seeds = [],
        bump=bumps.house_vault_bump
    )]
    pub payer_wallet: AccountInfo<'info>,
    #[account(mut)]
    pub payer_authority: AccountInfo<'info>,
    #[account(address = solana_program::sysvar::recent_blockhashes::ID)]
    pub recent_blockhashes: AccountInfo<'info>,
    pub program_state: AccountInfo<'info>,
    #[account(address = anchor_spl::token::ID)]
    pub token_program: AccountInfo<'info>,
}

impl Default for HouseState {
    fn default() -> Self {
        unsafe { std::mem::zeroed() }
    }
}

#[derive(Accounts)]
#[instruction(bump: u8)]
pub struct InitHouse<'info> {
    #[account(mut)]
    pub vrf: AccountInfo<'info>,
    #[account(
        init,
        seeds=[b"house-state"],
        bump,
        payer=user
    )]
    pub house_state: Account<'info, HouseState>,
    #[account(mut)]
    pub house_vault: UncheckedAccount<'info>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}

/// How can the caller sign for the house vault so the settle gamble function can pay out a winner?
/// How Can I pass bumps into the invoke method? It looks like it just takes
/// a state_bump and a permission_bump
#[derive(Accounts)]
#[instruction(bump: u8)]
pub struct SettleGamble<'info> {
    #[account(mut)]
    pub vrf: AccountInfo<'info>,
    #[account(mut)]
    pub state: Account<'info, VrfState>,
    #[account(
        mut,
        seeds=[b"house-state"],
        bump=bump,
    )]
    pub house_state: Account<'info, HouseState>,
    #[account(mut)]
    pub house_vault: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
}

#[account]
pub struct VrfState {
    pub authority: Pubkey,
    pub max_result: u64,
    pub vrf_account: Pubkey,
    pub result_buffer: [u8; 32],
    pub result: u128,
    pub last_timestamp: i64,
}


#[account]
pub struct HouseState {
    pub vrf_account: Pubkey,
    pub house_vault: Pubkey,
    pub reward_address: Pubkey,
    pub vrf_counter: u64,
}

#[error]
pub enum ErrorCode {
    #[msg("Not a valid Switchboard VRF account")]
    InvalidSwitchboardVrfAccount,
    #[msg("The max result must not exceed u64")]
    MaxResultExceedsMaximum,
    #[msg("Current round result is empty")]
    EmptyCurrentRoundResult,
    #[msg("Invalid authority account provided.")]
    InvalidAuthorityError,
}
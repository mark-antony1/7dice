use std::thread::AccessError;
use anchor_lang::prelude::*;
use anchor_spl::token::{TokenAccount, Transfer};
// use anchor_spl::token;
use solana_program;
use solana_program::{
    // clock::Clock,
    account_info::AccountInfo,
    entrypoint::ProgramResult, program::invoke, system_instruction,
};
pub use switchboard_v2::{VrfAccountData, VrfRequestRandomness};
const MAX_VALUE: u64 = 100;
const ZERO_ADDRESS: Pubkey = Pubkey::new_from_array([0; 32]);

declare_id!("GJmxJGYZETm142yHTQVasceWxWSVzC1Zi86UrCEpgrhK");

pub fn transfer<'a>(
    token_program: &AccountInfo<'a>,
    from: &Account<'a, TokenAccount>,
    to: &Account<'a, TokenAccount>,
    authority: &AccountInfo<'a>,
    auth_seed: &[&[&[u8]]],
    amount: u64,
) -> ProgramResult {
    let cpi_program = token_program.clone();
    let cpi_accounts = Transfer {
        from: from.to_account_info(),
        to: to.to_account_info(),
        authority: authority.clone(),
    };
    let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, auth_seed);
    anchor_spl::token::transfer(cpi_ctx, amount)?;
    Ok(())
}

#[program]
pub mod hello {
    use super::*;
    pub fn init_house(ctx: Context<InitHouse>, _bump: u8, vault_name: String) -> ProgramResult {
        // Debit from_account and credit to_account
        let user = &mut ctx.accounts.user;
        let system_program = &ctx.accounts.system_program;
        let house_vault = &ctx.accounts.house_vault;

        let house_state = &mut ctx.accounts.house_state;
        house_state.vrf_account = ctx.accounts.vrf.key.clone();
        house_state.reward_address = ZERO_ADDRESS;

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

    pub fn gamble(ctx: Context<Gamble>, _params: RequestResultParams, vault_name: String) -> ProgramResult {
        // Debit from_account and credit to_account
        msg!("in gamble");
        let user = &mut ctx.accounts.user;
        let system_program = &ctx.accounts.system_program;
        let house_vault = &ctx.accounts.house_vault;

        if ctx.accounts.house_state.reward_address != ZERO_ADDRESS {
            return Err(ErrorCode::MaxResultExceedsMaximum.into());
        }

        {
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
        }

        {
            msg!("invoked transfer");
            transfer(
                &ctx.accounts.token_program,
                &ctx.accounts.user_ata, 
                &ctx.accounts.house_vault,
                &ctx.accounts.user,
                &[],
                100_000_000
            )?;
        }

        {
            let switchboard_program = ctx.accounts.switchboard_program.to_account_info();

            
            let vrf_request_randomness = VrfRequestRandomness {
                authority: ctx.accounts.house_state.to_account_info(),
                vrf: ctx.accounts.vrf.to_account_info(),
                oracle_queue: ctx.accounts.oracle_queue.to_account_info(),
                queue_authority: ctx.accounts.queue_authority.to_account_info(),
                data_buffer: ctx.accounts.data_buffer.to_account_info(),
                permission: ctx.accounts.permission.to_account_info(),
                escrow: ctx.accounts.escrow.clone(),
                payer_wallet: ctx.accounts.house_vault.clone(),
                payer_authority: ctx.accounts.house_state.to_account_info(),
                recent_blockhashes: ctx.accounts.recent_blockhashes.to_account_info(),
                program_state: ctx.accounts.program_state.to_account_info(),
                token_program: ctx.accounts.token_program.to_account_info(),
            };

            msg!("requesting randomness {} {} {}", ctx.accounts.vrf.key, ctx.accounts.permission.key, ctx.accounts.oracle_queue.key );
            let state_seeds: &[&[&[u8]]] = &[&[
                &vault_name.as_bytes(),
                &[_params.house_state_bump],
            ]];
            vrf_request_randomness.invoke_signed(
                switchboard_program,
                _params.state_bump,
                _params.permission_bump,
                state_seeds
            )?;
        }

        let vrf = VrfAccountData::new(&ctx.accounts.vrf)?;
        let next_counter = vrf.counter + 1;
        let house_state = &mut ctx.accounts.house_state;
        house_state.vrf_counter = next_counter as u64;
        house_state.reward_address = ctx.accounts.user.key.clone();
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

        // invoke(
        //     &system_instruction::transfer(
        //         &house_vault.to_account_info().key,
        //         &ctx.accounts.house_state.reward_address,
        //         100_000, // 0.001 SOL
        //     ),
        //     &[
        //         // why is to account info unavailable? How to convert to account info?
        //         ctx.accounts.house_state.reward_address.to_account_info(),
        //         house_vault.to_account_info().clone(),
        //         system_program.to_account_info().clone(),
        //     ],
        // )?;

        house_state.reward_address = Pubkey::new(&[0;32]);
        Ok(())
    }
}

#[derive(Clone, AnchorSerialize, AnchorDeserialize)]
pub struct RequestResultParams {
    pub permission_bump: u8,
    pub state_bump: u8,
    pub house_state_bump: u8,
}

#[derive(Accounts)]
#[instruction(bumps: RequestResultParams, vault_name: String)]
pub struct Gamble<'info> {
    #[account(
        mut,
        seeds=[vault_name.as_bytes()],
        bump=bumps.house_state_bump,
        // constraint =house_state.vrf_account.as_ref().strip() ==  vrf.key()
    )]
    pub house_state: Box<Account<'info, HouseState>>,
    #[account(mut)]
    pub house_vault: Account<'info, TokenAccount>,
    // switchboard accounts
    pub switchboard_program: AccountInfo<'info>,
    #[account(mut)]
    pub authority: AccountInfo<'info>,
    pub program_state: AccountInfo<'info>,
    #[account(mut)]
    pub vrf: AccountInfo<'info>,
    #[account(mut)]
    pub oracle_queue: AccountInfo<'info>,
    pub queue_authority: AccountInfo<'info>,
    pub data_buffer: AccountInfo<'info>,
    #[account(mut)]
    pub permission: AccountInfo<'info>,
    // user accounts    
    #[account(mut)]
    pub user: Signer<'info>,    
    #[account(mut)]
    pub user_ata: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub escrow: Account<'info, TokenAccount>,
    #[account(mut)]
    pub payer_authority: AccountInfo<'info>,
    #[account(address = solana_program::sysvar::recent_blockhashes::ID)]
    pub recent_blockhashes: AccountInfo<'info>,
    #[account(address = anchor_spl::token::ID)]
    pub token_program: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
}

impl Default for HouseState {
    fn default() -> Self {
        unsafe { std::mem::zeroed() }
    }
}

#[derive(Accounts)]
#[instruction(bump: u8, vault_name: String)]
pub struct InitHouse<'info> {
    #[account(mut)]
    pub vrf: AccountInfo<'info>,
    #[account(
        init,
        seeds=[vault_name.as_bytes()],
        bump=bump,
        payer=user
    )]
    pub house_state: Account<'info, HouseState>,
    #[account(mut)]
    pub house_vault: Account<'info, TokenAccount>,
    #[account(mut)]
    pub house_authority: AccountInfo<'info>,
    // #[account(
    //     init,
    //     seeds=[b"escrow-vault"],
    //     token::mint = redeemable_mint,
    //     token::authority = user_authority,
    //     bump,
    //     payer=user
    // )]
    // pub house_vault: Account<'info, TokenAccount>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}

// #[derive(Clone, AnchorSerialize, AnchorDeserialize)]
// pub struct InitHouseBumps {
//     pub bump: u8,
//     pub vault_name: [u8;32],
// }

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
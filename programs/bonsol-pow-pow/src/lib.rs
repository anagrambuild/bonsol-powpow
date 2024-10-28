use anchor_lang::prelude::*;
use anchor_lang::solana_program::sysvar;
use anchor_lang::solana_program::sysvar::Sysvar;
use anchor_spl::token_interface::{token_metadata_initialize, TokenMetadataInitialize};
use anchor_spl::{
    token_2022::Token2022,
    token_interface::{Mint, TokenAccount},
};
use bonsol_interface::anchor::{Bonsol, DeployV1Account, ExecutionRequestV1Account};
use bonsol_interface::instructions::{execute_v1, CallbackConfig, ExecutionConfig};

use anchor_lang::solana_program::keccak;
use anchor_spl::token_2022::{mint_to, MintTo};
use bonsol_interface::callback::handle_callback;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");
const MINE_IMAGE_ID: &str = "ec8b92b02509d174a1a07dbe228d40ea13ff4b4b71b84bdc690064dfea2b6f86";

#[error_code]
pub enum PowError {
    #[msg("Mine Request failed")]
    MineRequestFailed,
    #[msg("Mine Too Fast")]
    MineTooFast,
    #[msg("Invalid Callback")]
    InvalidCallback,
    #[msg("Invalid Output")]
    InvalidOutput,
}
#[program]
pub mod bonsol_pow_pow {

    use bonsol_interface::instructions::InputRef;

    use super::*;
    pub fn initialize(ctx: Context<Initialize>, args: InitializeArgs) -> Result<()> {
        let cpi_accounts = TokenMetadataInitialize {
            token_program_id: ctx.accounts.token_program.to_account_info(),
            mint: ctx.accounts.mint.to_account_info(),
            metadata: ctx.accounts.mint.to_account_info(),
            mint_authority: ctx.accounts.pow_config.to_account_info(),
            update_authority: ctx.accounts.pow_config.to_account_info(),
        };
        let cpi_ctx = CpiContext::new(ctx.accounts.token_program.to_account_info(), cpi_accounts);
        token_metadata_initialize(cpi_ctx, args.name, args.symbol, args.uri)?;
        ctx.accounts.pow_config.mint = ctx.accounts.mint.key();
        ctx.accounts.pow_config.init_slot = sysvar::clock::Clock::get()?.slot;
        Ok(())
    }

    pub fn mine_token(ctx: Context<MineToken>, args: MineTokenArgs) -> Result<()> {
        let slot = sysvar::clock::Clock::get()?.slot;
        let pkbytes = ctx.accounts.pow_config.mint.to_bytes();
        let input_hash = keccak::hashv(&[&args.num, &pkbytes]);
        if slot - ctx.accounts.pow_mint_log.slot < 100 {
            return Err(PowError::MineTooFast.into());
        }
        if slot - ctx.accounts.pow_config.last_mined < 2 {
            return Err(PowError::MineTooFast.into());
        }
        ctx.accounts.pow_mint_log.current_execution_account =
            Some(ctx.accounts.execution_request.key());
        execute_v1(
            ctx.accounts.miner.key,
            MINE_IMAGE_ID,
            &args.current_req_id,
            vec![InputRef::url(&pkbytes), InputRef::public(&args.num)],
            args.tip,
            slot + 100,
            ExecutionConfig {
                verify_input_hash: true,
                input_hash: Some(&input_hash.to_bytes()),
                forward_output: true,
            },
            Some(CallbackConfig {
                program_id: crate::id(),
                instruction_prefix: vec![0],
                extra_accounts: vec![
                    AccountMeta::new_readonly(ctx.accounts.pow_config.key(), false),
                    AccountMeta::new(ctx.accounts.pow_mint_log.key(), false),
                    AccountMeta::new(ctx.accounts.mint.key(), false),
                    AccountMeta::new(ctx.accounts.token_account.key(), false),
                    AccountMeta::new_readonly(ctx.accounts.token_program.key(), false),
                ],
            }),
        )
        .map_err(|_| PowError::MineRequestFailed)?;
        Ok(())
    }

    pub fn bonsol_callback(ctx: Context<BonsolCallback>, data: Vec<u8>) -> Result<()> {
        let slot = sysvar::clock::Clock::get()?.slot;
        if let Some(epub) = ctx.accounts.pow_mint_log.current_execution_account {
            if ctx.accounts.execution_request.key() != epub {
                return Err(PowError::InvalidCallback.into());
            }
            let ainfos = ctx.accounts.to_account_infos();
            let output = handle_callback(MINE_IMAGE_ID, &epub, &ainfos.as_slice(), &data)?;
            // this is application specific

            let difficulty = u64::from_le_bytes(
                output
                    .committed_outputs
                    .try_into()
                    .map_err(|_| PowError::InvalidOutput)?,
            );
            //mint tokens to token account based on difficulty
            ctx.accounts.pow_mint_log.slot = slot;
            ctx.accounts.pow_mint_log.amount_mined += difficulty;
            ctx.accounts.pow_mint_log.current_execution_account = None;
            // mint tokens

            mint_to(
                CpiContext::new(
                    ctx.accounts.token_program.to_account_info(),
                    MintTo {
                        mint: ctx.accounts.mint.to_account_info(),
                        to: ctx.accounts.token_account.to_account_info(),
                        authority: ctx.accounts.pow_config.to_account_info(),
                    },
                ),
                difficulty,
            )?;
            Ok(())
        } else {
            Err(PowError::InvalidCallback.into())
        }
    }
}

#[account]
#[derive(InitSpace)]
pub struct PoWConfig {
    pub mint: Pubkey,
    pub init_slot: u64,
    pub last_mined: u64,
    pub total_mined: u64,
}

#[account]
#[derive(InitSpace)]
pub struct PowMintLog {
    pub miner: Pubkey,
    pub amount_mined: u64,
    pub slot: u64,
    pub current_execution_account: Option<Pubkey>,
}

#[derive(AnchorDeserialize, AnchorSerialize)]
pub struct InitializeArgs {
    pub name: String,
    pub symbol: String,
    pub uri: String,
}

#[derive(AnchorDeserialize, AnchorSerialize)]
pub struct MineTokenArgs {
    pub current_req_id: String,
    pub num: [u8; 64],
    pub tip: u64,
}

#[derive(Accounts)]
#[instruction(args: InitializeArgs)]
pub struct Initialize<'info> {
    #[account(
        seeds = [b"powconfig"],
        init,
        payer = payer,
        bump,
        space = 8 + PoWConfig::INIT_SPACE,
    )]
    pub pow_config: Account<'info, PoWConfig>,
    #[account(
        init,
        payer = payer,
        seeds = [b"mint"],
        bump,
        mint::token_program = token_program,
        mint::decimals = 9,
        mint::authority = pow_config,
        mint::freeze_authority = pow_config,
        extensions::metadata_pointer::authority = authority,
        extensions::metadata_pointer::metadata_address = mint,
    )]
    pub mint: InterfaceAccount<'info, Mint>,
    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token2022>,
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
#[instruction(args: MineTokenArgs)]
pub struct MineToken<'info> {
    #[account(
        seeds = [b"powconfig"],
        bump
    )]
    pub pow_config: Account<'info, PoWConfig>,
    #[account(
        init_if_needed,
        space = 8 + PowMintLog::INIT_SPACE,
        payer = miner,
        seeds = [b"powmintlog", miner.key().as_ref()],
        bump,
    )]
    pub pow_mint_log: Account<'info, PowMintLog>,
    #[account(mut,
        constraint = pow_mint_log.miner == miner.key()
    )]
    pub miner: Signer<'info>,
    #[account(mut,
        constraint = mint.key() == pow_config.mint,
    )]
    pub mint: InterfaceAccount<'info, Mint>,
    #[account(
        mut,
        owner = token_program.key(),
        associated_token::mint = mint,
        associated_token::authority = miner,
        associated_token::token_program = token_program,
    )]
    pub token_account: InterfaceAccount<'info, TokenAccount>,
    pub token_program: Program<'info, Token2022>,
    pub bonsol_program: Program<'info, Bonsol>,
    pub execution_request: Account<'info, ExecutionRequestV1Account<'info>>,
    pub deployment_account: Account<'info, DeployV1Account<'info>>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct BonsolCallback<'info> {
    /// CHECK: This is the raw ER account, checked in the callback handler
    pub execution_request: UncheckedAccount<'info>,
    #[account(
        mut,
        seeds = [b"powconfig"],
        bump
    )]
    pub pow_config: Account<'info, PoWConfig>,
    #[account(mut, seeds = [b"powmintlog"], bump)]
    pub pow_mint_log: Account<'info, PowMintLog>,
    #[account(mut,
        constraint = pow_mint_log.miner == miner.key()
    )]
    /// CHECK: Checked via constraint
    pub miner: UncheckedAccount<'info>,
    #[account(mut)]
    pub mint: InterfaceAccount<'info, Mint>,
    #[account(
        mut,
        owner = token_program.key(),
        associated_token::mint = mint,
        associated_token::authority = miner,
        associated_token::token_program = token_program,
    )]
    pub token_account: InterfaceAccount<'info, TokenAccount>,
    pub token_program: Program<'info, Token2022>,
}

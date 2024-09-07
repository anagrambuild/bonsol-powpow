use anchor_lang::prelude::*;
use anchor_spl::{
    token_interface::{
        Mint, TokenAccount, TokenInterface
    }
};
use anchor_spl::token_2022::spl_token_2022::{
    extension::ExtensionType
};
use anchor_lang::solana_program::sysvar::Sysvar;
use anagram_bonsol_channel_interface::{
    anchor::DeployV1Account,
    anchor::ExecutionRequestV1Account,
    execute,
    // anchor::{
    //     BonsolChannel,
    //     DeploymentAccountV1, 
    //     ExecutionRequestV1,
    // },
};

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");
const MINE_IMAGE_ID: &str = "ec8b92b02509d174a1a07dbe228d40ea13ff4b4b71b84bdc690064dfea2b6f86";


#[account]
#[derive(InitSpace)]
pub struct PoWConfig {
    pub pow_authority: Pubkey, // will be set to system program
    pub mint: Pubkey,
    pub init_slot: u64,
}
#[derive(AnchorDeserialize, AnchorSerialize)]
pub struct InitializeArgs {
    pub name: String,
    pub symbol: String,
    pub uri: String,
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
        signer,
        seeds = [b"mint"],
        bump,
        mint::token_program = token_program,
        mint::decimals = 9,
        mint::authority = pow_config,
        mint::freeze_authority = pow_config,
        extensions::metadata_pointer::authority = authority,
        extensions::metadata_pointer::metadata_address = mint,
    )]
    /// CHECK will become the mint
    pub mint: InterfaceAccount<'info, Mint>,
    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,
    pub token_program: Interface<'info, TokenInterface>,
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct MineToken<'info> {
    #[account(
        seeds = [b"powconfig"],
        bump
    )]
    pub pow_config: Account<'info, PoWConfig>,
    
    #[account(mut,
    )]
    pub mint:  InterfaceAccount<'info, Mint>,
    #[account(mut)]
    pub token_account: InterfaceAccount<'info, TokenAccount>,
    pub token_program: Interface<'info, TokenInterface>,
    #[account(

    )]
    pub deployment_account:  InterfaceAccount<'info, DeployV1Account<'info>>,
}


#[program]
pub mod bonsol_pow_pow {
    use anchor_lang::accounts::sysvar;

    use super::*;

    pub fn initialize(
        ctx: Context<Initialize>,
        name: String,
        symbol: String,
        uri: String,
    ) -> Result<()> {
        // Initialize the mint
        let mint_address = ctx.accounts.mint.key();
        let mint_authority = ctx.accounts.pow_authority.key();
        let decimals = 9;

        

        let initialize_mint_ix = initialize_mint2(
            &spl_token_2022::id(),
            &mint_address,
            &mint_authority,
            Some(&mint_authority), // freeze_authority
            decimals,
        )?;

        anchor_lang::solana_program::program::invoke(
            &initialize_mint_ix,
            &[
                ctx.accounts.mint.to_account_info(),
                ctx.accounts.rent.to_account_info(),
            ],
        )?;

        // Initialize metadata
        let metadata = Metadata {
            name,
            symbol,
            uri,
            update_authority: Some(ctx.accounts.pow_authority.key()),
            mint: ctx.accounts.mint.key(),
            ..Default::default()
        };

        let metadata_bytes = metadata.try_to_vec()?;
        initialize_metadata(ctx.accounts.mint.to_account_info(), metadata_bytes)?;
        ctx.accounts.pow_config.mint = ctx.accounts.mint.key();
        ctx.accounts.pow_config.pow_authority = ctx.accounts.pow_authority.key();
        ctx.accounts.pow_config.init_slot = sysvar::clock::Clock::get()?.slot;
        Ok(())
    }
   


    
}

// #[derive(Accounts)]
// pub struct MineToken<'info> {
//     #[account(
//         seeds = [b"powconfig"],
//         bump
        
//     )]

    
// }

// #[derive(Accounts)]
// pub struct BonsolCallback<'info> {
//     #[account(mut)]
//     pub mint: Account<'info, Mint>,
//     #[account(mut)]
//     pub token_account: Account<'info, TokenAccount>,
//     pub token_program: Interface<'info, TokenInterface>,
//     #[account(
//         seeds = [b"powauthority"],
//         bump
//     )]
//     /// CHECK: This is a PDA, safe to use as mint authority
//     pub pow_authority: UncheckedAccount<'info>,
// }

// #[derive(Account)]


// fn initialize_metadata(
//     mint_account_info: AccountInfo,
//     metadata: Vec<u8>,
// ) -> Result<()> {
//     let mut metadata_account = mint_account_info.try_borrow_mut_data()?;
//     let metadata_account = BaseMetadataAccountExt::init_metadata(metadata_account.as_ref_mut(), &metadata)?;
    
//     Ok(())
// }
// fn mint_token(
//     mint_account_info: AccountInfo,
//     token_account_info: AccountInfo,
//     amount: u64,
// ) -> Result<()> {
//     let mint_account = Mint::unpack(&mint_account_info.data.borrow())?;
//     let token_account = TokenAccount::unpack(&token_account_info.data.borrow())?;
//     let cpi_accounts = MintTo {
//         mint: mint_account_info.clone(),
//         to: token_account_info.clone(),
//         authority: mint_account_info.clone(),
//     };

//     let cpi_program = token_program_info.clone();
//     let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, &[]);

//     token::mint_to(cpi_ctx, amount)?;

//     Ok(())
// }

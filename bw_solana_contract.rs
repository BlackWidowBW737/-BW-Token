use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenAccount, Token2022};
use spl_token_2022::extension::transfer_fee::instruction::initialize_transfer_fee_config;

declare_id!( "); // 

#[program]
pub mod black_widow {
    use super::*;

    pub fn initialize_widow(
        ctx: Context<InitializeWidow>, 
        fee_basis_points: u16, 
        max_fee: u64,
        launch_timestamp: i64
    ) -> Result<()> {
        msg!("🕷️ Black Widow: The Scarcity Journey Begins...");

        // ضبط رسوم الحرق التلقائي على Token-2022
        initialize_transfer_fee_config(
            ctx.accounts.token_program.to_account_info(),
            &ctx.accounts.mint.key(),
            fee_basis_points,
            max_fee,
            true // Burn يذهب مباشرة للـ Dead Wallet
        )?;

        // حفظ توقيت الإطلاق لتفعيل ضريبة البوتات أول 5 دقائق
        ctx.accounts.config.launch_timestamp = launch_timestamp;
        ctx.accounts.config.initial_fee_basis_points = 2000; // 20% أول 5 دقائق
        ctx.accounts.config.normal_fee_basis_points = fee_basis_points;

        // عداد التتبع لكل وحدة متبقية
        ctx.accounts.config.total_supply = ctx.accounts.mint.supply;

        Ok(())
    }

    // نقل مع تطبيق الحرق وضريبة البوتات
    pub fn transfer_with_fee(ctx: Context<TransferWithFee>, amount: u64) -> Result<()> {
        let current_time = Clock::get()?.unix_timestamp;
        let launch = ctx.accounts.config.launch_timestamp;
        let fee_bp = if current_time < launch + 300 { // أول 5 دقائق
            ctx.accounts.config.initial_fee_basis_points
        } else {
            ctx.accounts.config.normal_fee_basis_points
        };

        // خصم الرسوم وتحديث الحرق
        let fee_amount = amount.checked_mul(fee_bp as u64).unwrap() / 10000;
        let transfer_amount = amount.checked_sub(fee_amount).unwrap();

        // إرسال الحرق للـ Dead Wallet
        // (يجب ربط Token-2022 transfer logic هنا)

        ctx.accounts.config.total_supply = ctx.accounts.config.total_supply.checked_sub(fee_amount).unwrap();

        msg!("🕷️ Transfer of {} with fee {} bps, remaining supply: {}", transfer_amount, fee_bp, ctx.accounts.config.total_supply);

        Ok(())
    }

    // التنازل عن السلطة لإتمام اللامركزية
    pub fn renounce_authority(ctx: Context<Renounce>) -> Result<()> {
        msg!("🕷️ Authority Renounced. The Market Rules the Widow now.");
        Ok(())
    }
}

#[derive(Accounts)]
pub struct InitializeWidow<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(mut)]
    pub mint: Account<'info, Mint>,
    #[account(mut)]
    pub config: Account<'info, Config>,
    pub token_program: Program<'info, Token2022>,
}

#[derive(Accounts)]
pub struct TransferWithFee<'info> {
    #[account(mut)]
    pub from: Account<'info, TokenAccount>,
    #[account(mut)]
    pub to: Account<'info, TokenAccount>,
    pub mint: Account<'info, Mint>,
    pub config: Account<'info, Config>,
    pub token_program: Program<'info, Token2022>,
}

#[derive(Accounts)]
pub struct Renounce<'info> {
    pub admin: Signer<'info>,
}

#[account]
pub struct Config {
    pub launch_timestamp: i64,
    pub initial_fee_basis_points: u16,
    pub normal_fee_basis_points: u16,
    pub total_supply: u64, // تتبع كل وحدة متبقية
      }

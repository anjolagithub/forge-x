use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};

declare_id!("CxSwnvkjvWwQhD2RW4LgvzUjkB3wXptNgYw78Wc2y598");

#[program]
pub mod forge_x {
    use super::*;

    pub fn initialize_pool(ctx: Context<InitializePool>, fee: u64, bump: u8) -> Result<()> {
        require!(fee <= 10000, CustomError::InvalidFee); // Fee cannot exceed 100%
        let pool = &mut ctx.accounts.pool;
        pool.fee = fee;
        pool.bump = bump;
        pool.token_a_reserve = 0;
        pool.token_b_reserve = 0;

        emit!(PoolInitialized {
            fee,
            pool_address: ctx.accounts.pool.key(),
        });

        Ok(())
    }

    pub fn add_liquidity(ctx: Context<AddLiquidity>, amount_a: u64, amount_b: u64) -> Result<()> {
        require!(amount_a > 0 && amount_b > 0, CustomError::InvalidLiquidityAmount);

        // Transfer tokens from user to pool
        token::transfer(ctx.accounts.into_transfer_to_pool_a_context()?, amount_a)?;
        token::transfer(ctx.accounts.into_transfer_to_pool_b_context()?, amount_b)?;

        let pool = &mut ctx.accounts.pool;
        pool.token_a_reserve = pool
            .token_a_reserve
            .checked_add(amount_a)
            .ok_or(CustomError::Overflow)?;
        pool.token_b_reserve = pool
            .token_b_reserve
            .checked_add(amount_b)
            .ok_or(CustomError::Overflow)?;

        emit!(LiquidityAdded {
            amount_a,
            amount_b,
            new_reserve_a: pool.token_a_reserve,
            new_reserve_b: pool.token_b_reserve,
            pool_address: pool.key(),
        });

        Ok(())
    }

    pub fn swap(ctx: Context<Swap>, amount_in: u64) -> Result<()> {
        require!(amount_in > 0, CustomError::InvalidSwapAmount);

        let pool = &ctx.accounts.pool;

        // Perform validation checks
        require!(pool.token_a_reserve > 0 && pool.token_b_reserve > 0, CustomError::InsufficientLiquidity);

        // Calculate swap output and check constraints
        let amount_in_with_fee = amount_in.checked_mul(10000 - pool.fee).ok_or(CustomError::Overflow)? / 10000;
        let amount_out = amount_in_with_fee
            .checked_mul(pool.token_b_reserve)
            .ok_or(CustomError::Overflow)?
            / (pool.token_a_reserve.checked_add(amount_in_with_fee).ok_or(CustomError::Overflow)?);

        require!(amount_out > 0 && amount_out <= pool.token_b_reserve, CustomError::InsufficientOutput);

        // Perform CPI transfers
        token::transfer(ctx.accounts.into_transfer_in_context()?, amount_in)?;
        token::transfer(ctx.accounts.into_transfer_out_context()?, amount_out)?;

        // Update pool reserves
        let pool = &mut ctx.accounts.pool;
        pool.token_a_reserve = pool
            .token_a_reserve
            .checked_add(amount_in)
            .ok_or(CustomError::Overflow)?;
        pool.token_b_reserve = pool
            .token_b_reserve
            .checked_sub(amount_out)
            .ok_or(CustomError::Underflow)?;

        emit!(SwapExecuted {
            user: ctx.accounts.user.key(),
            amount_in,
            amount_out,
            new_reserve_a: pool.token_a_reserve,
            new_reserve_b: pool.token_b_reserve,
            fee: pool.fee,
        });

        Ok(())
    }
}

#[derive(Accounts)]
pub struct InitializePool<'info> {
    #[account(
        init,
        seeds = [b"pool".as_ref()],
        bump,
        payer = user,
        space = 8 + std::mem::size_of::<Pool>()
    )]
    pub pool: Account<'info, Pool>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct AddLiquidity<'info> {
    #[account(mut)]
    pub pool: Account<'info, Pool>,
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(mut, constraint = token_a.owner == *user.key)]
    pub token_a: Account<'info, TokenAccount>, // Use TokenAccount directly
    #[account(mut, constraint = token_b.owner == *user.key)]
    pub token_b: Account<'info, TokenAccount>, // Use TokenAccount directly
    #[account(mut)]
    pub pool_a: Account<'info, TokenAccount>, // Use TokenAccount directly
    #[account(mut)]
    pub pool_b: Account<'info, TokenAccount>, // Use TokenAccount directly
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct Swap<'info> {
    #[account(mut)]
    pub pool: Account<'info, Pool>,
    #[account(mut, constraint = user_token.owner == *user.key)]
    pub user_token: Account<'info, TokenAccount>, // Use TokenAccount directly
    #[account(mut)]
    pub pool_token: Account<'info, TokenAccount>, // Use TokenAccount directly
    pub user: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

#[account]
pub struct Pool {
    pub token_a_reserve: u64,
    pub token_b_reserve: u64,
    pub fee: u64,
    pub bump: u8,
}

impl<'info> AddLiquidity<'info> {
    fn into_transfer_to_pool_a_context(&self) -> Result<CpiContext<'_, '_, '_, 'info, Transfer<'info>>> {
        Ok(CpiContext::new(
            self.token_program.to_account_info(),
            Transfer {
                from: self.token_a.to_account_info(),
                to: self.pool_a.to_account_info(),
                authority: self.user.to_account_info(),
            },
        ))
    }

    fn into_transfer_to_pool_b_context(&self) -> Result<CpiContext<'_, '_, '_, 'info, Transfer<'info>>> {
        Ok(CpiContext::new(
            self.token_program.to_account_info(),
            Transfer {
                from: self.token_b.to_account_info(),
                to: self.pool_b.to_account_info(),
                authority: self.user.to_account_info(),
            },
        ))
    }
}

impl<'info> Swap<'info> {
    fn into_transfer_out_context(&self) -> Result<CpiContext<'_, '_, '_, 'info, Transfer<'info>>> {
        Ok(CpiContext::new(
            self.token_program.to_account_info(),
            Transfer {
                from: self.pool_token.to_account_info(),
                to: self.user_token.to_account_info(),
                authority: self.pool.to_account_info(),
            },
        ))
    }

    fn into_transfer_in_context(&self) -> Result<CpiContext<'_, '_, '_, 'info, Transfer<'info>>> {
        Ok(CpiContext::new(
            self.token_program.to_account_info(),
            Transfer {
                from: self.user_token.to_account_info(),
                to: self.pool_token.to_account_info(),
                authority: self.user.to_account_info(),
            },
        ))
    }
}

#[event]
pub struct PoolInitialized {
    pub fee: u64,
    pub pool_address: Pubkey,
}

#[event]
pub struct LiquidityAdded {
    pub amount_a: u64,
    pub amount_b: u64,
    pub new_reserve_a: u64,
    pub new_reserve_b: u64,
    pub pool_address: Pubkey,
}

#[event]
pub struct SwapExecuted {
    pub user: Pubkey,
    pub amount_in: u64,
    pub amount_out: u64,
    pub new_reserve_a: u64,
    pub new_reserve_b: u64,
    pub fee: u64,
}

#[error_code]
pub enum CustomError {
    #[msg("Insufficient liquidity in the pool.")]
    InsufficientLiquidity,
    #[msg("Insufficient output amount.")]
    InsufficientOutput,
    #[msg("Invalid swap amount.")]
    InvalidSwapAmount,
    #[msg("Invalid liquidity amount.")]
    InvalidLiquidityAmount,
    #[msg("Fee exceeds the allowable limit.")]
    InvalidFee,
    #[msg("Arithmetic overflow occurred.")]
    Overflow,
    #[msg("Arithmetic underflow occurred.")]
    Underflow,
}

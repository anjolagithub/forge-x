use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};

declare_id!("CxSwnvkjvWwQhD2RW4LgvzUjkB3wXptNgYw78Wc2y598");

#[program]
pub mod forge_x {
    use super::*;

pub fn initialize_pool(ctx: Context<InitializePool>, fee: u64, bump: u8) -> Result<()> {
    let pool = &mut ctx.accounts.pool;
    pool.fee = fee;
    pool.bump = bump; // Directly set it
    Ok(())
}


}


    pub fn add_liquidity(ctx: Context<AddLiquidity>, amount_a: u64, amount_b: u64) -> Result<()> {
    // Perform all immutable operations first
    token::transfer(
        ctx.accounts.into_transfer_to_pool_a_context(),
        amount_a,
    )?;
    token::transfer(
        ctx.accounts.into_transfer_to_pool_b_context(),
        amount_b,
    )?;

    // Create the mutable borrow after immutable borrows are done
    let pool = &mut ctx.accounts.pool;

    // Update reserves in the pool
    pool.token_a_reserve += amount_a;
    pool.token_b_reserve += amount_b;

    Ok(())
}

 pub fn swap(ctx: Context<Swap>, amount_in: u64) -> Result<()> {
    // Perform all immutable operations first
    let transfer_out_ctx = ctx.accounts.into_transfer_out_context();
    token::transfer(transfer_out_ctx, amount_in)?;

    // Now create a mutable reference
    let pool = &mut ctx.accounts.pool;
    pool.token_a_reserve += amount_in;

    Ok(())
}


#[derive(Accounts)]
pub struct InitializePool<'info> {
    #[account(
        init,
        seeds = [b"pool".as_ref()],
        bump,
        payer = user,
        space = 8 + 32 // Add the required space
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
    pub token_a: Account<'info, TokenAccount>,
    #[account(mut, constraint = token_b.owner == *user.key)]
    pub token_b: Account<'info, TokenAccount>,
    #[account(mut)]
    pub pool_a: Account<'info, TokenAccount>,
    #[account(mut)]
    pub pool_b: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct Swap<'info> {
    #[account(mut)]
    pub pool: Account<'info, Pool>,
    #[account(mut, constraint = user_token.owner == *user.key)]
    pub user_token: Account<'info, TokenAccount>,
    #[account(mut)]
    pub pool_token: Account<'info, TokenAccount>,
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
    fn into_transfer_to_pool_a_context(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        CpiContext::new(
            self.token_program.to_account_info(),
            Transfer {
                from: self.token_a.to_account_info(),
                to: self.pool_a.to_account_info(),
                authority: self.user.to_account_info(),
            },
        )
    }

    fn into_transfer_to_pool_b_context(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        CpiContext::new(
            self.token_program.to_account_info(),
            Transfer {
                from: self.token_b.to_account_info(),
                to: self.pool_b.to_account_info(),
                authority: self.user.to_account_info(),
            },
        )
    }
}

impl<'info> Swap<'info> {
    fn into_transfer_out_context(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        CpiContext::new(
            self.token_program.to_account_info(),
            Transfer {
                from: self.pool_token.to_account_info(),
                to: self.user_token.to_account_info(),
                authority: self.pool.to_account_info(),
            },
        )
    }
}

#[error_code]
pub enum CustomError {
    #[msg("Insufficient output amount.")]
    InsufficientOutput,
}

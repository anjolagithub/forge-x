# Forge-X: Solana Liquidity Pool Program

## Overview
**Forge-X** is a decentralized liquidity pool program built on Solana using the Anchor framework. It enables users to:
- Initialize a pool with token reserves.
- Add liquidity to the pool.
- Swap tokens with a fee mechanism to ensure the liquidity provider earns a reward.

This program is ideal for decentralized finance (DeFi) applications needing token exchange functionality, liquidity management, or an automated market maker (AMM).

---

## Features
- **Pool Initialization**: Create a liquidity pool with customizable fees.
- **Liquidity Provision**: Add reserves for token pairs.
- **Token Swaps**: Swap tokens with efficient calculations and fees.
- **Event Emissions**: Tracks operations like pool creation, liquidity additions, and swaps for easy frontend integration.

---

## Program Workflow
1. **Initialize Pool**: Sets up a new pool with a fee and initializes token reserves.
2. **Add Liquidity**: Users can deposit tokens to increase the pool's liquidity reserves.
3. **Swap Tokens**: Perform token exchanges with fees applied, using the constant product formula for automated market making.

---

## Instructions

### Initialize a Pool
The `initialize_pool` instruction creates a new pool.  
**Parameters**:
- `fee`: Fee in basis points (max 10000 = 100%).
- `bump`: Derived PDA bump.

**Accounts**:
- `pool`: The liquidity pool account.
- `user`: The pool creator (payer).
- `system_program`: Solana's system program.

Example:
```bash
solana program initialize_pool <fee> <bump>
```

---

### Add Liquidity
The `add_liquidity` instruction deposits tokens into the pool to increase its reserves.  
**Parameters**:
- `amount_a`: Amount of token A to add.
- `amount_b`: Amount of token B to add.

**Accounts**:
- `pool`: The liquidity pool account.
- `user`: Liquidity provider.
- `token_a`/`token_b`: User's token accounts.
- `pool_a`/`pool_b`: Pool's token accounts.
- `token_program`: SPL token program.

Example:
```bash
solana program add_liquidity <amount_a> <amount_b>
```

---

### Swap Tokens
The `swap` instruction allows users to exchange one token for another.  
**Parameters**:
- `amount_in`: The input token amount.

**Accounts**:
- `pool`: The liquidity pool account.
- `user`: The token swapper.
- `user_token`: User's token account for input/output.
- `pool_token`: Pool's token account for input/output.
- `token_program`: SPL token program.

Example:
```bash
solana program swap <amount_in>
```

---

## Code Highlights

### Pool Initialization
The pool is initialized with:
- **Fee**: Basis points for swaps (max 10,000 = 100%).
- **Token Reserves**: Tracks balances for token A and B.

```rust
pub fn initialize_pool(ctx: Context<InitializePool>, fee: u64, bump: u8) -> Result<()> {
    require!(fee <= 10000, CustomError::InvalidFee);
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
```

### Adding Liquidity
Liquidity is added by transferring tokens from the user to the pool. The reserves are updated accordingly.

```rust
pub fn add_liquidity(ctx: Context<AddLiquidity>, amount_a: u64, amount_b: u64) -> Result<()> {
    require!(amount_a > 0 && amount_b > 0, CustomError::InvalidLiquidityAmount);

    token::transfer(ctx.accounts.into_transfer_to_pool_a_context()?, amount_a)?;
    token::transfer(ctx.accounts.into_transfer_to_pool_b_context()?, amount_b)?;

    let pool = &mut ctx.accounts.pool;
    pool.token_a_reserve = pool.token_a_reserve.checked_add(amount_a).ok_or(CustomError::Overflow)?;
    pool.token_b_reserve = pool.token_b_reserve.checked_add(amount_b).ok_or(CustomError::Overflow)?;

    emit!(LiquidityAdded {
        amount_a,
        amount_b,
        new_reserve_a: pool.token_a_reserve,
        new_reserve_b: pool.token_b_reserve,
        pool_address: pool.key(),
    });

    Ok(())
}
```

### Token Swaps
Swaps use the **constant product formula** to calculate the output amount. Fees are deducted from the input amount.

```rust
pub fn swap(ctx: Context<Swap>, amount_in: u64) -> Result<()> {
    require!(amount_in > 0, CustomError::InvalidSwapAmount);

    let pool = &ctx.accounts.pool;
    require!(pool.token_a_reserve > 0 && pool.token_b_reserve > 0, CustomError::InsufficientLiquidity);

    let amount_in_with_fee = amount_in.checked_mul(10000 - pool.fee).ok_or(CustomError::Overflow)? / 10000;
    let amount_out = amount_in_with_fee
        .checked_mul(pool.token_b_reserve)
        .ok_or(CustomError::Overflow)?
        / (pool.token_a_reserve.checked_add(amount_in_with_fee).ok_or(CustomError::Overflow)?);

    require!(amount_out > 0 && amount_out <= pool.token_b_reserve, CustomError::InsufficientOutput);

    token::transfer(ctx.accounts.into_transfer_in_context()?, amount_in)?;
    token::transfer(ctx.accounts.into_transfer_out_context()?, amount_out)?;

    let pool = &mut ctx.accounts.pool;
    pool.token_a_reserve = pool.token_a_reserve.checked_add(amount_in).ok_or(CustomError::Overflow)?;
    pool.token_b_reserve = pool.token_b_reserve.checked_sub(amount_out).ok_or(CustomError::Underflow)?;

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
```

---

## Events
Events are emitted to track significant operations:
- **PoolInitialized**: Tracks pool creation.
- **LiquidityAdded**: Tracks liquidity additions.
- **SwapExecuted**: Tracks swaps and their outcomes.

---

## Error Handling
Custom error codes ensure proper validation and debugging:
- `InsufficientLiquidity`
- `InvalidSwapAmount`
- `Overflow`
- `Underflow`
- `InvalidFee`

---

## Integration
To integrate this program into a frontend:
1. **Deploy** the program to Solana.
2. Use a Solana SDK (e.g., [Solana Web3.js](https://solana-labs.github.io/solana-web3.js/)) to interact with the program.
3. Listen to program events for real-time updates.

--- 
## Deployment Address
https://explorer.solana.com/tx/sbLv8fxCJgDtdtkEVrhTs8M9EiLbSV2ZXVGp47Up9uiSUmC2qQs6ryjGFJjZTsMUEvhmKtr4ULVpSXbNUf8W1zi?cluster=devnet

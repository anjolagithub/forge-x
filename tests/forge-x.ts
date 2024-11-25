import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { ForgeX } from "../target/types/forge_x";
import { Keypair, SystemProgram } from "@solana/web3.js";

describe("Forge_X", () => {
  // Set the provider
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.ForgeX as Program<ForgeX>;

  // Keypair for the pool account
  const pool = Keypair.generate();

  it("Initializes the pool", async () => {
    // Call the initializePool instruction
    await program.methods
      .initializePool(new anchor.BN(5)) // Set the fee as 5
      .accounts({
        pool: pool.publicKey,
        user: provider.wallet.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .signers([pool])
      .rpc();

    console.log("Pool initialized:", pool.publicKey.toBase58());

    // Fetch the pool account to verify
    const poolAccount = await program.account.pool.fetch(pool.publicKey);
    console.log("Pool state:", poolAccount);

    // Assert that the pool was initialized with the correct values
    expect(poolAccount.tokenAReserve.toNumber()).toBe(0);
    expect(poolAccount.tokenBReserve.toNumber()).toBe(0);
    expect(poolAccount.fee.toNumber()).toBe(5);
  });
});
function expect(actual: any) {
  return {
    toBe(expected: any) {
      if (actual !== expected) {
        throw new Error(`Expected ${actual} to be ${expected}`);
      }
    }
  };
}

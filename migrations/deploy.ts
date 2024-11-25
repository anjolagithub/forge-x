import * as anchor from "@coral-xyz/anchor";
import { Keypair, PublicKey, SystemProgram } from "@solana/web3.js";

// Migration function to deploy and initialize the program
module.exports = async function (provider: anchor.Provider) {
  // Set the provider for the Anchor client
  anchor.setProvider(provider);

  // Load the program workspace
  const program = anchor.workspace.ForgeX as anchor.Program;

  console.log("Deploying Forge_X program...");

  // Generate a new keypair for the pool account
  const pool = Keypair.generate();

  try {
    // Initialize the pool by invoking the program's initializePool method
    const tx = await program.methods
      .initializePool(new anchor.BN(5)) // Example: Fee is 5
      .accounts({
        pool: pool.publicKey, // The generated pool account
        user: provider.publicKey, // The wallet running the transaction
        systemProgram: SystemProgram.programId, // Required Solana system program
      })
      .signers([pool]) // Sign transaction with pool account
      .rpc();

    console.log("Forge_X pool initialized successfully.");
    console.log("Pool public key:", pool.publicKey.toBase58());
    console.log("Transaction signature:", tx);
  } catch (error) {
    console.error("Failed to deploy Forge_X program:", error);
  }
};

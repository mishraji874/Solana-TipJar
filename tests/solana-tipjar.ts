import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { SolanaTipjar } from "../target/types/solana_tipjar";
import { PublicKey, SystemProgram, LAMPORTS_PER_SOL } from "@solana/web3.js";
import { expect } from "chai";

describe("solana-tipjar", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.SolanaTipjar as Program<SolanaTipjar>;
  const owner = provider.wallet;

  let tipjarPDA: PublicKey;
  let tipjarBump: number;

  before(async () => {
    // Generate PDA for tipjar
    [tipjarPDA, tipjarBump] = await PublicKey.findProgramAddress(
      [Buffer.from("tipjar"), owner.publicKey.toBuffer()],
      program.programId
    );
  });

  it("Initialize TipJar", async () => {
    const description = "My First Tip Jar";
    const category = "Testing";
    const goal = new anchor.BN(5 * LAMPORTS_PER_SOL);

    await program.methods
      .initializeTipjar(description, category, goal)
      .accounts({
        tipjar: tipjarPDA,
        user: owner.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .rpc();

    const tipjarAccount = await program.account.tipJar.fetch(tipjarPDA);
    expect(tipjarAccount.description).to.equal(description);
    expect(tipjarAccount.category).to.equal(category);
    expect(tipjarAccount.goal.toString()).to.equal(goal.toString());
    expect(tipjarAccount.owner.toString()).to.equal(owner.publicKey.toString());
    expect(tipjarAccount.isActive).to.be.true;
  });

  // More test cases will be added after seeing state.rs
});

/**
import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Tipjar } from "../target/types/tipjar";
import { assert } from "chai";

describe("tipjar", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.Tipjar as Program<Tipjar>;

  let tipJarPda: anchor.web3.PublicKey;
  let bump: number;
  const user = provider.wallet;
  const systemProgram = anchor.web3.SystemProgram.programId;

  const description = "Support my open-source project";
  const category = "Development";
  const goal = new anchor.BN(5_000_000); // 0.005 SOL

  before(async () => {
    [tipJarPda, bump] = await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from("tipjar"), user.publicKey.toBuffer()],
      program.programId
    );
  });

  it("Initializes a new TipJar", async () => {
    await program.methods
      .initializeTipjar(description, category, goal)
      .accounts({
        tipjar: tipJarPda,
        user: user.publicKey,
        systemProgram,
      })
      .signers([])
      .rpc();

    const tipJarAccount = await program.account.tipJar.fetch(tipJarPda);
    assert.equal(tipJarAccount.description, description);
    assert.equal(tipJarAccount.category, category);
    assert.ok(tipJarAccount.goal.eq(goal));
    assert.ok(tipJarAccount.isActive);
    assert.equal(tipJarAccount.owner.toBase58(), user.publicKey.toBase58());
  });

  it("Sends a tip to the TipJar", async () => {
    const tipAmount = new anchor.BN(1_000_000); // 0.001 SOL
    const memo = "Great work!";
    const visibility = { public: {} };

    await program.methods
      .sendTip(tipAmount, visibility, memo)
      .accounts({
        tipjar: tipJarPda,
        sender: user.publicKey,
        systemProgram,
      })
      .signers([])
      .rpc();

    const tipJarAccount = await program.account.tipJar.fetch(tipJarPda);
    assert.ok(tipJarAccount.totalReceived.eq(tipAmount));
    assert.equal(tipJarAccount.tipsHistory.length, 1);
    assert.equal(tipJarAccount.tipsHistory[0].memo, memo);
  });

  it("Retrieves tip statistics", async () => {
    const tx = await program.methods
      .getTipStats()
      .accounts({
        tipjar: tipJarPda,
      })
      .rpc();

    // Since getTipStats emits an event, you can verify it via logs if needed
    console.log("Tip statistics retrieved successfully.");
  });

  it("Clears the tip history", async () => {
    await program.methods
      .clearTipHistory()
      .accounts({
        tipjar: tipJarPda,
        owner: user.publicKey,
      })
      .signers([])
      .rpc();

    const tipJarAccount = await program.account.tipJar.fetch(tipJarPda);
    assert.equal(tipJarAccount.tipsHistory.length, 0);
  });

  it("Toggles the TipJar status", async () => {
    await program.methods
      .toggleTipjarStatus()
      .accounts({
        tipjar: tipJarPda,
        owner: user.publicKey,
      })
      .signers([])
      .rpc();

    const tipJarAccount = await program.account.tipJar.fetch(tipJarPda);
    assert.isFalse(tipJarAccount.isActive);

    // Toggle back to active
    await program.methods
      .toggleTipjarStatus()
      .accounts({
        tipjar: tipJarPda,
        owner: user.publicKey,
      })
      .signers([])
      .rpc();

    const updatedTipJarAccount = await program.account.tipJar.fetch(tipJarPda);
    assert.isTrue(updatedTipJarAccount.isActive);
  });

  it("Updates TipJar metadata", async () => {
    const newDescription = "Updated description";
    const newCategory = "Updated category";
    const newGoal = new anchor.BN(10_000_000); // 0.01 SOL

    await program.methods
      .updateTipjar(newDescription, newCategory, newGoal)
      .accounts({
        tipjar: tipJarPda,
        owner: user.publicKey,
      })
      .signers([])
      .rpc();

    const tipJarAccount = await program.account.tipJar.fetch(tipJarPda);
    assert.equal(tipJarAccount.description, newDescription);
    assert.equal(tipJarAccount.category, newCategory);
    assert.ok(tipJarAccount.goal.eq(newGoal));
  });

  it("Withdraws tips from the TipJar", async () => {
    const withdrawAmount = new anchor.BN(500_000); // 0.0005 SOL

    await program.methods
      .withdrawTip(withdrawAmount)
      .accounts({
        tipjar: tipJarPda,
        owner: user.publicKey,
        systemProgram,
      })
      .signers([])
      .rpc();

    const tipJarAccount = await program.account.tipJar.fetch(tipJarPda);
    assert.ok(tipJarAccount.totalReceived.lt(goal));
  });

  it("Pauses and resumes the TipJar", async () => {
    // Pause
    await program.methods
      .pauseTipjar()
      .accounts({
        tipjar: tipJarPda,
        owner: user.publicKey,
      })
      .signers([])
      .rpc();

    let tipJarAccount = await program.account.tipJar.fetch(tipJarPda);
    assert.isFalse(tipJarAccount.isActive);

    // Resume
    await program.methods
      .resumeTipjar()
      .accounts({
        tipjar: tipJarPda,
        owner: user.publicKey,
      })
      .signers([])
      .rpc();

    tipJarAccount = await program.account.tipJar.fetch(tipJarPda);
    assert.isTrue(tipJarAccount.isActive);
  });

  it("Closes the TipJar", async () => {
    await program.methods
      .closeTipjar()
      .accounts({
        tipjar: tipJarPda,
        owner: user.publicKey,
        systemProgram,
      })
      .signers([])
      .rpc();

    try {
      await program.account.tipJar.fetch(tipJarPda);
      assert.fail("TipJar account should be closed");
    } catch (err) {
      assert.include(err.message, "Account does not exist");
    }
  });
});

 */
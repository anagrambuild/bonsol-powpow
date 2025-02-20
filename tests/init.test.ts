import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { assert } from 'chai';
import { BonsolPowPow } from '../target/types/bonsol_pow_pow';
import { expect, test, describe } from "bun:test";


test('Initializes the program', async () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const program = anchor.workspace.BonsolPowPow as Program<BonsolPowPow>;
  // Add your test here.
  const tx = await program.methods
    .initialize({
      name: "initialize",
      symbol: "INIT",
      uri: "https://arweave.net/1234",
    })
    .accounts({
      payer: provider.wallet.publicKey,
    })
    .rpc();
  console.log("Your transaction signature", tx);
  // Fetch the account to check if it was initialized correctly
  const account = await program.account.poWConfig.fetch(provider.wallet.publicKey);
  assert.ok(account.mint);
});
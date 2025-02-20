import * as anchor from '@project-serum/anchor';
import { Program } from '@project-serum/anchor';
import { assert } from 'chai';
import { BonsolPowPow } from '../target/types/bonsol_pow_pow';

describe('initialize', () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.MyProgram as Program<BonsolPowPow>;

  it('Initializes the program', async () => {
    // Define the arguments for the initialize method
    const arg1 = new anchor.BN(1234); // Example argument
    const arg2 = provider.wallet.publicKey; // Example argument

    // Add your test here.
    const tx = await program.methods.initialize(arg1, arg2).rpc();
    console.log("Your transaction signature", tx);

    // Fetch the account to check if it was initialized correctly
    const account = await program.account.myAccount.fetch(provider.wallet.publicKey);
    assert.ok(account.initialized);
  });
});
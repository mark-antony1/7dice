import * as anchor from '@project-serum/anchor';
import { Program } from '@project-serum/anchor';
import { Hello } from '../target/types/hello';
const { SystemProgram } = anchor.web3;
import { PublicKey } from "@solana/web3.js";

describe('hello', () => {

  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.Provider.env());
  const program = anchor.workspace.Hello as Program<Hello>;

  it('Is initialized!', async () => {
    // Add your test here.
    console.log("🚀 Starting test...")

    const provider = anchor.Provider.env();
    anchor.setProvider(provider);  
      
    let [pda, bump] = await PublicKey.findProgramAddress([], program.programId);
    console.log(pda.toBase58());
  
    console.log('initializing house')
    const initTx = await program.rpc.initHouse(bump, {
      accounts: {
        baseAccount: pda,
        user: provider.wallet.publicKey,
        systemProgram: SystemProgram.programId,
      },
    });
  });
});

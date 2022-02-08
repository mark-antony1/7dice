import * as anchor from '@project-serum/anchor';
import { Program } from '@project-serum/anchor';
import { Hello } from '../target/types/hello';
const { SystemProgram } = anchor.web3;
import { PublicKey, Keypair } from "@solana/web3.js";

describe('hello', () => {

  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.Provider.env());
  const program = anchor.workspace.Hello as Program<Hello>;
  console.log("program.programId", program.programId)

  it('Is initialized!', async () => {
    // Add your test here.
    console.log("🚀 Starting test...")

    const provider = anchor.Provider.env();
    anchor.setProvider(provider);  
      
    let [pda, bump] = await PublicKey.findProgramAddress([], program.programId);
    console.log(pda.toBase58());

    let vrfAddress = new Keypair().publicKey
    console.log(pda.toBase58());
  
    console.log('initializing house')
    const initTx = await program.rpc.initHouse(bump, {
      accounts: {
        baseAccount: pda,
        user: provider.wallet.publicKey,
        systemProgram: SystemProgram.programId,
      },
    });
    console.log("init tx", initTx)
    const gambleTx = await program.rpc.gamble({
      accounts: {
        baseAccount: pda,
        vrfAccount: vrfAddress,
        user: provider.wallet.publicKey,
        systemProgram: SystemProgram.programId,
      },
    });
    console.log("gamble", gambleTx)
  });
});

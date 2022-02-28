import * as anchor from '@project-serum/anchor';
import { TOKEN_PROGRAM_ID, Token } from "@solana/spl-token";
import { NodeWallet } from './utils/node-wallet'
import { Hello } from '../target/types/hello';
import fs from "node:fs";
import path from "node:path";
const { SystemProgram } = anchor.web3;
import { Cluster, Connection, Keypair, PublicKey, SYSVAR_RECENT_BLOCKHASHES_PUBKEY, clusterApiUrl } from "@solana/web3.js";
import {
  Callback,
  SBV2_DEVNET_PID,
  SBV2_MAINNET_PID,
  SwitchboardPermissionValue,
  OracleQueueAccount,
  PermissionAccount,
  ProgramStateAccount,
  VrfAccount,
} from "@switchboard-xyz/switchboard-v2";
import * as spl from "@solana/spl-token";

describe('hello', () => {
  // Configure the client to use the local cluster.
  const provider = anchor.Provider.env();
  anchor.setProvider(provider);
  const program = anchor.workspace.Hello as anchor.Program<Hello>;

  const vrfKey = '8yFdD7qLRrFuzED2mvPDAj5G1opyWjJhdStESbex67pM'
  const vaultName = "test_vault_" + Math.random().toString(16).substring(2, 8); // "sol_put_sell";
  let house_escrow

  it('Is initialized!', async () => {  
    const nodeWallet = new NodeWallet(provider.connection, provider.wallet as anchor.Wallet)
    const newFundedWallet = await nodeWallet.createFundedWallet(13705000)
    
    let [houseStatePda, houseStateBump] = await PublicKey.findProgramAddress([Buffer.from(vaultName)], program.programId);
    const wrappedSolMint = new Token(
      program.provider.connection,
      new anchor.web3.PublicKey("So11111111111111111111111111111111111111112"),
      TOKEN_PROGRAM_ID,
      newFundedWallet
    );
    const vaultKeypair = anchor.web3.Keypair.generate();

    const house_vault = await spl.Token.createWrappedNativeAccount(
      provider.connection,
      spl.TOKEN_PROGRAM_ID,
      newFundedWallet.publicKey,
      newFundedWallet,
      1,
    );
    house_escrow = house_vault;

    await wrappedSolMint.setAuthority(
      house_escrow,
      houseStatePda,
      "AccountOwner",
      newFundedWallet,
      []
    );

    const initTx = await program.rpc.initHouse(
      houseStateBump, Buffer.from(vaultName), {
      accounts: {
        houseAuthority: newFundedWallet.publicKey,
        houseState: houseStatePda,
        houseVault: house_escrow,
        user: newFundedWallet.publicKey,
        systemProgram: SystemProgram.programId,
        vrf: new PublicKey(vrfKey)
      },
      signers: [newFundedWallet]
    });
  })


  it('Gambles properly!', async () => {
    // Add your test here.
    console.log("🚀 Starting test...")
    

    let [houseAuthorityPda, houseAuthorityBump] = await PublicKey.findProgramAddress([], program.programId);
    let [houseStatePda, houseStateBump] = await PublicKey.findProgramAddress([Buffer.from(vaultName)], program.programId);
    console.log("house state", houseStatePda.toString())

    const nodeWallet = new NodeWallet(provider.connection, provider.wallet as anchor.Wallet)
    const newFundedWallet = await nodeWallet.createFundedWallet(134_905_000)
    const vrfSecret = anchor.web3.Keypair.generate()

    const cluster = 'devnet'

    const switchboardProgram = await loadSwitchboardProgram(provider, cluster);

    const queueAccount = new OracleQueueAccount({
      program: switchboardProgram,
      publicKey: new anchor.web3.PublicKey("F8ce7MsckeZAbAGmxjJNetxYXQa9mKr9nnrC3qKubyYy"),
    });

    const queue = await queueAccount.loadData()

    const vrfExampleProgram = await loadVrfExampleProgram(
      newFundedWallet,
      cluster,
      "https://api.devnet.solana.com"
    );

    const [stateAccount, stateBump] = VrfState.fromSeed(
      vrfExampleProgram,
      vrfSecret.publicKey,
      houseAuthorityPda
    );

    const ixCoder = new anchor.InstructionCoder(vrfExampleProgram.idl);

    const callback: Callback = {
      programId: vrfExampleProgram.programId,
      accounts: [
        { pubkey: new PublicKey(vrfKey), isSigner: false, isWritable: true },
        { pubkey: vrfSecret.publicKey, isSigner: false, isWritable: false },
        { pubkey: houseStatePda, isSigner: true, isWritable: true },
        { pubkey: newFundedWallet.publicKey, isSigner: true, isWritable: true },
        { pubkey: house_escrow, isSigner: false, isWritable: true},
        { pubkey: SystemProgram.programId, isSigner: false, isWritable: false }
      ],
      ixData: ixCoder.encode("settleGamble", ""),
    };

    const vrfAccount = await VrfAccount.create(switchboardProgram, {
      queue: queueAccount,
      callback,
      authority: houseStatePda,
      keypair: vrfSecret,
    });

    const vrf = await vrfAccount.loadData();
    const queueAuthority = queue.authority;
    const dataBuffer = queue.dataBuffer;
    const escrow = vrf.escrow;
    const [programStateAccount, programStateBump] =
      ProgramStateAccount.fromSeed(switchboardProgram);
    console.log(`authority ${queueAuthority}\ngrantee: ${queueAccount.publicKey}\ngranter: ${vrfAccount.publicKey}`);
    const createdPermissionAccount = await PermissionAccount.create(switchboardProgram, {
      authority: queueAuthority, 
      granter: queueAccount.publicKey, 
      grantee: vrfAccount.publicKey
    });
    try {
      await createdPermissionAccount.loadData();
    } catch {
      throw new Error(
        "A requested permission pda account has not been initialized."
      );
    }
    const [permissionAccount, permissionBump] = PermissionAccount.fromSeed(
      switchboardProgram,
      queueAuthority,
      queueAccount.publicKey,
      vrfAccount.publicKey
    );

    console.log("permissionAccount", permissionAccount.publicKey.toString(), 'created permission ', createdPermissionAccount.publicKey.toString())


    const tokenProgram = TOKEN_PROGRAM_ID;
    const recentBlockhashes = SYSVAR_RECENT_BLOCKHASHES_PUBKEY;
    console.log(
      `Sending Txn\nstateBump: ${programStateBump}\npermissionBump: ${permissionBump}`
    );

    console.log("herere create warapped")
    const fundedWalletWSolAta = await spl.Token.createWrappedNativeAccount(
      program.provider.connection,
      TOKEN_PROGRAM_ID,
      newFundedWallet.publicKey,
      newFundedWallet,
      100_000_001,
    );

    console.log('gambling now')
    const gambleTx = await program.rpc.gamble(
      {
        permissionBump: permissionBump,
        stateBump: programStateBump,
        houseStateBump: houseStateBump
      },
      vaultName,
      {
        accounts: {
          switchboardProgram: switchboardProgram.programId,
          vrf: vrfAccount.publicKey,
          queueAuthority,
          authority: vrf.authority,
          houseVault: house_escrow,
          dataBuffer,
          houseState: houseStatePda,
          escrow,
          oracleQueue: vrf.oracleQueue,
          permission: createdPermissionAccount.publicKey,
          payerAuthority: houseStatePda,
          user: newFundedWallet.publicKey,
          userAta: fundedWalletWSolAta,
          recentBlockhashes,
          systemProgram: SystemProgram.programId,
          programState: programStateAccount.publicKey,
          tokenProgram
        },
        signers: [newFundedWallet]
      },
    );
    console.log("gamble", gambleTx)

      setTimeout(async () => {
        const data = await anchor.workspace.Hello.account.houseState.fetch(houseStatePda)
        console.log("newFundedWallet.publicKey", newFundedWallet.publicKey.toString())
        console.log("data data", data.rewardAddress.toString())
      }, 10000)
  });
});

export async function loadSwitchboardProgram(
  provider: anchor.Provider,
  cluster: Cluster,
): Promise<anchor.Program> {
  const programId = getSwitchboardPid(cluster);


  const anchorIdl = await anchor.Program.fetchIdl(programId, provider);
  if (!anchorIdl) {
    throw new Error(`failed to read idl for ${programId}`);
  }

  return new anchor.Program(anchorIdl, programId, provider);
}

export const getSwitchboardPid = (cluster: Cluster): PublicKey => {
  if (cluster === "mainnet-beta") {
    return SBV2_MAINNET_PID;
  }
  return SBV2_DEVNET_PID;
};


export async function loadVrfExampleProgram(
  payer: Keypair,
  cluster: Cluster, // should verify example has been deployed
  rpcUrl: string
): Promise<anchor.Program> {
  const programId = loadVrfExamplePid();
  const connection = new Connection(rpcUrl, {
    commitment: "confirmed",
  });
  const program = await connection.getAccountInfo(programId);
  if (!program) {
    throw new Error(
      `failed to find example program for cluster ${cluster}. did you run 'anchor build && anchor deploy' with the Anchor.toml pointed to cluster ${cluster}`
    );
  }

  // load anchor program from local IDL file
  if (!fs.existsSync(PROGRAM_IDL_PATH)) {
    throw new Error(`Could not find program IDL. Have you run 'anchor build'?`);
  }
  const idl: anchor.Idl = JSON.parse(
    fs.readFileSync(PROGRAM_IDL_PATH, "utf-8")
  );

  const wallet = new anchor.Wallet(payer);
  const provider = new anchor.Provider(connection, wallet, {
    commitment: "confirmed",
  });

  return new anchor.Program(idl, programId, provider);
}

export function loadVrfExamplePid(): PublicKey {
  if (!fs.existsSync(PROGRAM_KEYPAIR_PATH)) {
    throw new Error(`Could not find keypair. Have you run 'anchor build'?`);
  }
  const programKeypair = Keypair.fromSecretKey(
    new Uint8Array(JSON.parse(fs.readFileSync(PROGRAM_KEYPAIR_PATH, "utf8")))
  );
  return programKeypair.publicKey;
}

// VRF Example program keypair
const PROGRAM_KEYPAIR_PATH = path.join(
  __dirname,
  "../target/deploy/hello-keypair.json"
);

// VRF Example program IDL
const PROGRAM_IDL_PATH = path.join(
  __dirname,
  "../target/idl/hello.json"
);


export class VrfState {
  program: anchor.Program;

  publicKey: PublicKey;

  constructor(program: anchor.Program, publicKey: PublicKey) {
    this.program = program;
    this.publicKey = publicKey;
  }

  /**
   * @return account size of the global ProgramStateAccount.
   */
  size(): number {
    return this.program.account.sbState.size;
  }

  async loadData(): Promise<any> {
    const state: any = await this.program.account.vrfState.fetch(
      this.publicKey
    );
    state.ebuf = undefined;
    return state;
  }

  async print(): Promise<void> {
    console.log(JSON.stringify(await this.loadData(), undefined, 2));
  }

  public static fromSeed(
    program: anchor.Program,
    vrfPubkey: PublicKey,
    authority: PublicKey
  ): [VrfState, number] {
    const [statePubkey, stateBump] =
      anchor.utils.publicKey.findProgramAddressSync(
        [Buffer.from("STATE"), vrfPubkey.toBytes(), authority.toBytes()],
        program.programId
      );
    return [new VrfState(program, statePubkey), stateBump];
  }
}
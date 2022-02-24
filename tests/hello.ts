import * as anchor from '@project-serum/anchor';
import { TOKEN_PROGRAM_ID } from "@solana/spl-token";
import { NodeWallet } from './utils/node-wallet'
import { Hello } from '../target/types/hello';
const { SystemProgram } = anchor.web3;
import { Cluster, Connection, Keypair, PublicKey, SYSVAR_RECENT_BLOCKHASHES_PUBKEY, clusterApiUrl } from "@solana/web3.js";
import {
  SBV2_DEVNET_PID,
  SBV2_MAINNET_PID,
  SwitchboardPermissionValue,
  OracleQueueAccount,
  PermissionAccount,
  ProgramStateAccount,
  VrfAccount,
} from "@switchboard-xyz/switchboard-v2";

describe('hello', () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.Provider.env());
  const program = anchor.workspace.Hello as anchor.Program<Hello>;
  console.log("program.programId", program.programId)

  it('Is initialized!', async () => {
    // Add your test here.
    console.log("🚀 Starting test...")

    const provider = anchor.Provider.env();
    anchor.setProvider(provider);  
      
    let [houseVaultPda, houseVaultBump] = await PublicKey.findProgramAddress([], program.programId);
    console.log(houseVaultPda.toBase58());

    // let vrfAddress = new Keypair().publicKey
    // console.log("secretkey address", new Keypair().secretKey)
    console.log(houseVaultPda.toBase58());

    const secret = new Uint8Array( [
      132,  85,  82, 179, 193, 202,  77,  53, 131, 146, 223,
      195, 159, 227, 128,  63,  71,  88, 160,  51, 108,  90,
      200, 156,  30, 213,  89, 235, 101,  37,  29, 255, 201,
       64,  17,  96, 249,  78, 146,  31,  20, 186, 159, 164,
       19, 104, 175, 223, 241,   4,  41,  72,  93,  93,  76,
       75,  26,  70,  54, 106, 141,  10,  29,  57
    ])

    const connection = new Connection(clusterApiUrl('devnet'), 'confirmed')

    const nodeWallet = new NodeWallet(connection, provider.wallet as anchor.Wallet)
    const newFundedWallet = await nodeWallet.createFundedWallet(1105000)

    const initTx = await program.rpc.initHouse(houseVaultBump, {
      accounts: {
        houseVault: houseVaultPda,
        user: newFundedWallet.publicKey,
        systemProgram: SystemProgram.programId,
      },
      signers: [newFundedWallet]
    });

    const rpcUrl = 'https://api.devnet.solana.com'
    const cluster = 'devnet'
    const vrfKey = '8yFdD7qLRrFuzED2mvPDAj5G1opyWjJhdStESbex67pM'
    const payer = provider.wallet.publicKey
    const vrfPubkey = new PublicKey(vrfKey);

    const payerKeypair = Keypair.fromSecretKey(secret);
    // const exampleProgram = await loadVrfExampleProgram(
    //   payerKeypair,
    //   cluster,
    //   rpcUrl
    // );
    const switchboardProgram = await loadSwitchboardProgram(provider, cluster);

    const vrfAccount = new VrfAccount({
      program: switchboardProgram as anchor.Program,
      publicKey: vrfPubkey,
    });
  
    const vrf = await vrfAccount.loadData();
    const queueAccount = new OracleQueueAccount({
      program: switchboardProgram,
      publicKey: vrf.oracleQueue,
    });
    const queue = await queueAccount.loadData();
    const queueAuthority = queue.authority;
    const dataBuffer = queue.dataBuffer;
    const escrow = vrf.escrow;
    const [programStateAccount, programStateBump] =
      ProgramStateAccount.fromSeed(switchboardProgram);
    const [permissionAccount, permissionBump] = PermissionAccount.fromSeed(
      switchboardProgram,
      queueAuthority,
      queueAccount.publicKey,
      vrfPubkey
    );
    try {
      await permissionAccount.loadData();
    } catch {
      throw new Error(
        "A requested permission pda account has not been initialized."
      );
    }
    const switchTokenMint = await programStateAccount.getTokenMint();
    const payerTokenAccount =
      await switchTokenMint.getOrCreateAssociatedAccountInfo(
        payerKeypair.publicKey
      );
    const tokenProgram = TOKEN_PROGRAM_ID;
    const recentBlockhashes = SYSVAR_RECENT_BLOCKHASHES_PUBKEY;
    console.log(
      `Sending Txn\nstateBump: ${programStateBump}\npermissionBump: ${permissionBump}`
    );

    console.log("payerKeypair", payerKeypair.publicKey.toString())
    console.log("vrf.authority", vrf.authority)


    const gambleTx = await program.rpc.gamble(
      {
        permissionBump: permissionBump,
        stateBump: programStateBump
      },
      {
        accounts: {
          switchboardProgram: switchboardProgram.programId,
          vrf: vrfPubkey,
          queueAuthority,
          authority: vrf.authority,
          houseVault: houseVaultPda,
          dataBuffer,
          escrow,
          oracleQueue: new anchor.web3.PublicKey("7Ra2SzwJUeFBHKzryZ8hApAMCmajJdo89K1CshS9yvfY"),
          permission: new anchor.web3.PublicKey("9AuCqRVXeTPViWiPyzB2uVBRQdaDuGDyb9yy18Tyd3HY"),
          vrfAccount: new anchor.web3.PublicKey("3rXbmjGutCPutYnX8rvk8jDgRN62zBuwXoHt1pBJjiLr"),
          payerWallet: houseVaultPda,
          payerAuthority: houseVaultPda,
          user: newFundedWallet.publicKey,
          recentBlockhashes,
          systemProgram: SystemProgram.programId,
          programState: programStateAccount.publicKey,
          tokenProgram
        },
        signers: [newFundedWallet]
      },
    );
    console.log("gamble", gambleTx)
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
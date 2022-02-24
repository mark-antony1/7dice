import * as anchor from '@project-serum/anchor';
import {
  Connection,
  Keypair,
  PublicKey,
  sendAndConfirmTransaction,
  SystemProgram,
  Transaction,
} from '@solana/web3.js';
import { BN } from '@project-serum/anchor';
import { AccountUtils,  } from './account-utils';
import { NATIVE_MINT, Token, TOKEN_PROGRAM_ID } from '@solana/spl-token';

export class NodeWallet extends AccountUtils {
  // @ts-ignore
  wallet: anchor.Wallet; //node wallet

  // @ts-ignore
  constructor(conn: Connection, wallet: anchor.Wallet) {
    super(conn);
    this.wallet = wallet;
  }

  async createFundedWallet(lamports: number): Promise<Keypair> {
    const wallet = Keypair.generate();
    const tx = new Transaction().add(
      SystemProgram.transfer({
        fromPubkey: this.wallet.publicKey,
        toPubkey: wallet.publicKey,
        lamports,
      })
    );
    await sendAndConfirmTransaction(this.conn, tx, [this.wallet.payer]);
    return wallet;
  }
}
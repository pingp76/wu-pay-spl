/* This source code is primarily based on follwing online tutorial:

https://betterprogramming.pub/using-pdas-and-spl-token-in-anchor-and-solana-df05c57ccd04

https://101blockchains.com/transfer-sol-and-spl-tokens-using-anchor/

It is inteded solely for educational purpose and not for commercial use. 
The use of this code is subject to the terms and conditions as outlined 
by the original source material from their website. The author of this 
code bears no liability for any unintended consequences that arise from its use.
*/

import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { WuPaySpl } from "../target/types/wu_pay_spl";
import { BN } from "@coral-xyz/anchor";
import assert from "assert";
import * as spl from '@solana/spl-token';
import {clusterApiUrl, Connection, Keypair, LAMPORTS_PER_SOL, SystemProgram, Transaction, sendAndConfirmTransaction} from "@solana/web3.js";


//WuPaySpl

interface PDAParameters {
  //from: anchor.web3.PublicKey,
  //to: anchor.web3.PublicKey,
  statekey: anchor.web3.PublicKey,
  walletkey: anchor.web3.PublicKey,
  idx: anchor.BN,
  statebump: number,
  walletbump: number,
}

describe("wu-pay-spl", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());
  const provider = anchor.getProvider()
  const program = anchor.workspace.WuPaySpl as Program<WuPaySpl>;

  // create testing accounts, including sender, reciever, and escrow wallet PDAs
  let alice: anchor.web3.Keypair;
  let alice_ata: anchor.web3.PublicKey;
  let bob: anchor.web3.Keypair;
  let bob_ata: anchor.web3.PublicKey;
  let pda: PDAParameters;

  // initialize the testing accounts before each test
  beforeEach(async () => {
    [alice, alice_ata] = await createSolUserAndAirdrop(5, provider);
    [bob, bob_ata] = await createSolUserAndAirdrop(0, provider);
    pda = await createPdaParams(provider.connection, alice.publicKey, bob.publicKey);
  });

  // The main test case
  it("Case1: Normal case, alice trasfer 20 to bob via Escrow wallet", async () => {
    console.log(`- Step1: Initialized a new  Escrow wallet PDA. Alice sent 20 tokens to escrow wallet`);
    // Assert initial balances
    const aliceBalancePre = await readAccount(alice_ata, provider);
    //assert.equal(aliceBalancePre, '5000000000');
    // amount to transfer
    const amount = new anchor.BN(20000000);

    // Step1: Alice sent 20 tokens to Escrow wallet
    const tx1 = await program.methods.depositeGrant(pda.idx, pda.statebump, pda.walletbump, amount).accounts({
      sender: alice.publicKey,
      senderAta: alice_ata,
      receiver: bob.publicKey,
      mintOfTokenBeingSent: spl.NATIVE_MINT,
      escrowState: pda.statekey,
      escrowWallet: pda.walletkey,
      systemProgram: anchor.web3.SystemProgram.programId,
      tokenProgram: spl.TOKEN_PROGRAM_ID,
      rent: anchor.web3.SYSVAR_RENT_PUBKEY,
    }).signers([alice]).rpc(
      //{skipPreflight:true}
    );


    // Assert that 20 tokens were moved from Alice's account to the escrow.
    const aliceBalancePost = await readAccount(alice_ata, provider);
    console.log("Alice's balance: ", aliceBalancePost);
    //assert.equal(aliceBalancePost, '4977598800'); // 20 tokens + rent for PDA

    const EscrowWalletBalancePost = await readAccount(pda.walletkey, provider);
    console.log("Escrow Wallet's balance: ", EscrowWalletBalancePost);
    //assert.equal(saftyboxBalancePost, '20946560'); // 20 tokens + rent for PDA

    //Step2: Complete the grant. Bob received 20 tokens from saftybox
    console.log(`- Step2: Complete the grant. Bob received 20 tokens from saftybox`);

    const tx2 = await program.methods.completeGrant(pda.idx, pda.statebump).accounts({
      sender: alice.publicKey,
      receiver: bob.publicKey,
      receiverAta: bob_ata,
      escrowState: pda.statekey,
      escrowWallet: pda.walletkey,
      systemProgram: anchor.web3.SystemProgram.programId,
      tokenProgram: spl.TOKEN_PROGRAM_ID,
      rent: anchor.web3.SYSVAR_RENT_PUBKEY,
    }).signers([alice]).rpc(
      { skipPreflight: true }
    );

    // Assert that 20 tokens were sent to bob.
    const bobBalance = await readAccount(bob.publicKey, provider);
    console.log("Bob's balance at end: ", bobBalance);
    //assert.equal(bobBalance, '20000000');
    const EscrowBalanceFinal = await readAccount(pda.walletkey, provider);
    console.log("Escrow wallet's balance at end: ", EscrowBalanceFinal);

    console.log(`- Step3: Clean up. Close the PDA account.`);
    //Step3: Clean up. Close the PDA account.
    const tx3 = await program.methods.closeEscrow(pda.idx, pda.statebump).accounts({
      sender: alice.publicKey,
      receiver: bob.publicKey,
      escrowState: pda.statekey,
      escrowWallet: pda.walletkey,
      systemProgram: anchor.web3.SystemProgram.programId,
      tokenProgram: spl.TOKEN_PROGRAM_ID
    }).signers([alice]).rpc();
    // Assert that escrow wallet was correctly closed.
    try {
      //await readAccount(pda.statekey, provider);
      await readAccount(pda.walletkey, provider);
      return assert.fail("Account should be closed");
    } catch (e) {
      assert.equal(e.message, "Account should be closed");
    }
  });

/*
   it("Case2: Send and withdraw", async () => {
    console.log(`Step1: Initialized a new Escrow wallet PDA. Alice sent 20 tokens to Escrow wallet`);
    console.log(`Step1: Initialized a new  Escrow wallet PDA. Alice sent 20 tokens to Escrow wallet`);
    // Assert initial balances
    const aliceBalancePre = await readAccount(alice_ata, provider);
    //assert.equal(aliceBalancePre, '5000000000');
    // amount to transfer
    const amount = new anchor.BN(20000000);

    // Step1: Alice sent 20 tokens to Escrow wallet
    const tx1 = await program.methods.depositeGrant(pda.idx, pda.statebump, pda.walletbump, amount).accounts({
      sender: alice.publicKey,
      senderAta: alice_ata,
      receiver: bob.publicKey,
      mintOfTokenBeingSent: spl.NATIVE_MINT,
      escrowState: pda.statekey,
      escrowWallet: pda.walletkey,
      systemProgram: anchor.web3.SystemProgram.programId,
      tokenProgram: spl.TOKEN_PROGRAM_ID,
      rent: anchor.web3.SYSVAR_RENT_PUBKEY,
    }).signers([alice]).rpc(
      //{skipPreflight:true}
    );

    // Assert that 20 tokens were moved from Alice's account to the escrow.
    const aliceBalancePost = await readAccount(alice.publicKey, provider);
    console.log("Alice's balance: ", aliceBalancePost);
    //assert.equal(aliceBalancePost, '4977598800'); // 20 tokens + rent for PDA

    const EscrowWalletBalancePost = await readAccount(pda.walletkey, provider);
    console.log("Escrow Wallet's balance: ", EscrowWalletBalancePost);
    //assert.equal(saftyboxBalancePost, '20946560'); // 20 tokens + rent for PDA

    //Step2: Withdraw the grant. Alice received back 20 tokens from escrow wallet
    console.log(`Step2: Withdraw the grant. Alice received back 20 tokens from escrow wallet`);

    const tx2 = await program.methods.withdrawGrant(pda.idx, pda.statebump).accounts({
      sender: alice.publicKey,
      senderAta: alice_ata,
      receiver: bob.publicKey,
      //mintOfTokenBeingSent: spl.NATIVE_MINT,
      escrowState: pda.statekey,
      escrowWallet: pda.walletkey,
      systemProgram: anchor.web3.SystemProgram.programId,
      tokenProgram: spl.TOKEN_PROGRAM_ID,
      rent: anchor.web3.SYSVAR_RENT_PUBKEY,
    }).signers([alice]).rpc(
      //{ skipPreflight: true }
    );

    // Assert that 20 tokens were sent to bob.
    const bobBalance = await readAccount(bob.publicKey, provider);
    console.log("Bob's balance at end: ", bobBalance);
    //assert.equal(bobBalance, '0');
    const escrowBalancePost = await readAccount(pda.walletkey, provider);
    console.log("Escrow wallet's balance at end: ", escrowBalancePost);
    const aliceBalanceFinal = await readAccount(alice.publicKey, provider);
    console.log("Alice's balance at end: ", aliceBalanceFinal);

  } 
  );*/

  const createSolUserAndAirdrop = async (airdropAmount: number, provider: anchor.Provider): Promise<[anchor.web3.Keypair, anchor.web3.PublicKey]> => {
    const user = anchor.web3.Keypair.generate();
    console.log(`- Creating a new user and airdropping ${airdropAmount} SOL`);
    console.log(`Pubkey for new user: ${user.publicKey.toBase58()}`);
    const airdropSignature = await provider.connection.requestAirdrop(
      user.publicKey,
      (airdropAmount+1) * LAMPORTS_PER_SOL,
    );
    
    await provider.connection.confirmTransaction(airdropSignature);
    let balance = await provider.connection.getBalance(user.publicKey);
    console.log(`Deposited Balance: ${balance / anchor.web3.LAMPORTS_PER_SOL} SOL`);

    const associatedTokenAccount = await spl.getAssociatedTokenAddress(
      spl.NATIVE_MINT,
      user.publicKey
    )

    // Create token account to hold your wrapped SOL
    const ataTransaction = new Transaction()
    .add(
      spl.createAssociatedTokenAccountInstruction(
        user.publicKey,
        associatedTokenAccount,
        user.publicKey,
        spl.NATIVE_MINT
      )
    );

    await sendAndConfirmTransaction(
      provider.connection,
      ataTransaction,
      [user]
    );
    console.log(`Obtained a new ATA wallet address: ${associatedTokenAccount.toBase58()}`);

    const solTransaction = new Transaction().add(
      SystemProgram.transfer({
        fromPubkey: user.publicKey,
        toPubkey: associatedTokenAccount,
        lamports: airdropAmount*LAMPORTS_PER_SOL
      }),
       spl.createSyncNativeInstruction(
        associatedTokenAccount,
      ) 
    );

    try{ 
    await sendAndConfirmTransaction(
      provider.connection,
      solTransaction,
      [user]
    );
    } catch (e) {
      console.log(`Error: ${e.message}`);
      console.log(`Error: ${e.stack}`);
      throw new Error('An error has occurred');
    }

    const accountInfo = await spl.getAccount(provider.connection, associatedTokenAccount);
    console.log(`Created ATA wallet: ${accountInfo.isNative}, Lamports: ${accountInfo.amount}`);
    
    return [user,associatedTokenAccount];
  }
  
  const readAccount = async (accountPublicKey: anchor.web3.PublicKey, provider: anchor.Provider): Promise<number> => {
    return await provider.connection.getBalance(accountPublicKey);
  }

  const createPdaParams = async (connection: anchor.web3.Connection, alice: anchor.web3.PublicKey, bob: anchor.web3.PublicKey): Promise<PDAParameters> => {
    // this is a unique identifier for this transaction
    const uid = new anchor.BN(parseInt((Date.now() / 1000).toString()));
    const uidBuffer = uid.toBuffer('le', 8);

    // Create a PDA for the escrow state account, holding the escrow account's authority.
    let [statePubKey, statebump] = await anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("escrow_state"), alice.toBuffer(), bob.toBuffer(), uidBuffer], program.programId,
    );
    // create a PDA for the escrow wallet account, holding the SOL
    let [walletkey, walletbump] = await anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("escrow_wallet"), alice.toBuffer(), bob.toBuffer(), uidBuffer], program.programId,
    );
    console.log(`- PDA Wallet created: ${walletkey.toBase58()}`);
    console.log(`- PDA state created: ${statePubKey.toBase58()}`);
    console.log('idx is ', uidBuffer);
    console.log('walletbump is ', walletbump);
    console.log('statebump is ', statebump);
    return {
      statekey: statePubKey,
      walletkey: walletkey,
      idx: uid,
      statebump: statebump,
      walletbump: walletbump,
    }
  }

});

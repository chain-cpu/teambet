import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { Teambet } from "../target/types/teambet";

import {TOKEN_PROGRAM_ID, createMint, mintToChecked, transfer, setAuthority, createAssociatedTokenAccount, getOrCreateAssociatedTokenAccount, transferChecked, revokeInstructionData, getAssociatedTokenAddress} from "@solana/spl-token"
import {PublicKey, Transaction, SystemProgram} from "@solana/web3.js"
import { assert, config, expect } from "chai";
import { BN } from "bn.js";


describe("teambet", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const provider = anchor.getProvider() as anchor.AnchorProvider;
  
  const program = anchor.workspace.Teambet as Program<Teambet>;
  const authorityKeyPair = anchor.web3.Keypair.generate();
  const newAuthorityKeyPair = anchor.web3.Keypair.generate();
  const TEAM1 = 1;
  const TEAM2 = 2;

  let team1_players = [];
  let team2_players = [];
  let team3_players = [];
  
  const TEAM1_PLAYER_NUM = 3;
  const TEAM2_PLAYER_NUM = 3;
  const TEAM3_PLAYER_NUM = 3;

  const INIT_FUND = 10000000;
  const BET_AMOUNT = 5000000; 
  before(async () => {
    console.log("started...")
    await internal_airdrop(authorityKeyPair.publicKey, INIT_FUND);

    for(let i=0;i<TEAM1_PLAYER_NUM;i++) {
      team1_players.push(await newPlayer()); 
    }
    // return;

    for(let i=0;i<TEAM2_PLAYER_NUM;i++) {
      team2_players.push(await newPlayer()); 
    }

    for(let i=0;i<TEAM3_PLAYER_NUM;i++) {
      team3_players.push(await newPlayer()); 
    }

    console.log("finished");
    
  });
  it("Initialize Contract!", async () => {
    // Add your test here.
    // return;

    const _betStatusPda = await findBetStatusPda();

    try {
      const tx = await program.methods.initialize()
      .accounts({betStatus: _betStatusPda, authority: authorityKeyPair.publicKey})
      .signers([authorityKeyPair])
      .rpc(); 

    } catch (error) {
      console.log(error);
    }
  });

  // return;

  it("Initialize Bet!", async () => {
    // Add your test here.
    const _betStatusPda = await findBetStatusPda();
    try {
      const tx = await program.methods.newBet(timeStamp('2022-9-2'), timeStamp('2022-9-30'), )
      .accounts({betStatus: _betStatusPda, authority: authorityKeyPair.publicKey})
      .signers([authorityKeyPair])
      .rpc(); 
    } catch (error) {
      console.log(error);
    }
  });
  it("Should Transfer Ownership", async () => {
    const _betStatusPda = await findBetStatusPda();
    try {
      const tx = await program.methods.transferOwnership()
      .accounts({betStatus: _betStatusPda, authority: authorityKeyPair.publicKey, newAuthority: newAuthorityKeyPair.publicKey})
      .signers([authorityKeyPair])
      .rpc(); 
    } catch (error) {
      console.log(error);
    }
    const newAuthority = await (await program.account.betStatus.fetch(_betStatusPda)).authority as anchor.web3.PublicKey;
    expect(newAuthority.toString()).to.be.equal(newAuthorityKeyPair.publicKey.toString())   
    console.log("pubkey of newAuthorityKeyPair: ", newAuthorityKeyPair.publicKey.toString());
    console.log("pubkey of PDA: ", newAuthority.toString());
  })
  return;
  it("Bet", async () => {
    const _betStatusPda = await findBetStatusPda();
    const _betStatus = await findBetStatus();

    for(let i=0;i<TEAM1_PLAYER_NUM;i++) {
      const player = team1_players[i] as anchor.web3.Keypair;
      const _betInfoPda = await findBetInfoPda(player.publicKey, _betStatus.id);
      console.log(`player ${i} balance before bet: ${await balance(player.publicKey)}`);
      try {
          const tx = await program.methods.bet(1, new BN(BET_AMOUNT))
        .accounts({betStatus: _betStatusPda, payer: player.publicKey, betInfo: _betInfoPda, clock: anchor.web3.SYSVAR_CLOCK_PUBKEY})
        .signers([player])
        .rpc();
      } catch (error) {
        console.log(error)
      }
      console.log(`TeamL player ${i} balance after bet: ${await balance(player.publicKey)}`);
      
    }

    for(let i=0;i<TEAM2_PLAYER_NUM;i++) {
      const player = team2_players[i] as anchor.web3.Keypair;
      const _betInfoPda = await findBetInfoPda(player.publicKey, _betStatus.id);
      console.log(`player ${i} balance before bet: ${await balance(player.publicKey)}`);
      try {
          const tx = await program.methods.bet(2, new BN(BET_AMOUNT))
        .accounts({betStatus: _betStatusPda, payer: player.publicKey, betInfo: _betInfoPda, clock: anchor.web3.SYSVAR_CLOCK_PUBKEY})
        .signers([player])
        .rpc();
      } catch (error) {
        console.log(error)
      }
      console.log(`TeamR player ${i} balance after bet: ${await balance(player.publicKey)}`);
    }

    for(let i=0;i<TEAM3_PLAYER_NUM;i++) {
      const player = team3_players[i] as anchor.web3.Keypair;
      const _betInfoPda = await findBetInfoPda(player.publicKey, _betStatus.id);
      console.log(`player ${i} balance before bet: ${await balance(player.publicKey)}`);
      try {
          const tx = await program.methods.bet(3, new BN(BET_AMOUNT))
        .accounts({betStatus: _betStatusPda, payer: player.publicKey, betInfo: _betInfoPda, clock: anchor.web3.SYSVAR_CLOCK_PUBKEY})
        .signers([player])
        .rpc();
      } catch (error) {
        console.log(error)
      }
      console.log(`TeamM player ${i} balance after bet: ${await balance(player.publicKey)}`);
    }
  })


  it("Finalize", async () => {
    // Add your test here.
    const _betStatusPda = await findBetStatusPda();
    
    const tx = await program.methods.finalize()
    .accounts({betStatus: _betStatusPda, authority: authorityKeyPair.publicKey})
    .signers([authorityKeyPair])
    .rpc();
  });

  // return;
  it("Claim", async () => {
    // return;
    const _betStatusPda = await findBetStatusPda();
    const _betStatus = await findBetStatus();

    for(let i=0;i<2;i++) {
      const player = team1_players[i];
      const _betInfoPda = await findBetInfoPda(player.publicKey, _betStatus.id);
      
      // const _betInfo = await findBetInfo(player.publicKey, _betStatus.id);
      // console.log("betInfo before claim", _betInfo);

      try {
          const tx = await program.methods.claim()
        .accounts({betStatus: _betStatusPda, betInfo: _betInfoPda, payer: player.publicKey})
        .signers([player])
        .rpc();
      } catch (error) {
        console.log(error)
      }
      console.log(`TeamL player ${i} balance after claim: ${await balance(player.publicKey)}`);
    }

    for(let i=0;i<TEAM2_PLAYER_NUM;i++) {
      const player = team2_players[i];
      const _betInfoPda = await findBetInfoPda(player.publicKey, _betStatus.id);
      try {
          const tx = await program.methods.claim()
        .accounts({betStatus: _betStatusPda, betInfo: _betInfoPda, payer: player.publicKey})
        .signers([player])
        .rpc();
      } catch (error) {
        // console.log(error)
        // assert.fail()
      }
      console.log(`TeamR player ${i} balance after claim: ${await balance(player.publicKey)}`);
    }
    
  });

  async function airdrop(pubKey, amount) {
    await provider.connection.confirmTransaction(
      await provider.connection.requestAirdrop(pubKey, amount),
      "processed"
    );
  }

  async function internal_airdrop(pubKey, amount) {

    const tx = new Transaction().add(
      SystemProgram.transfer(
        {
          fromPubkey: provider.wallet.publicKey,
          toPubkey: pubKey,
          lamports: amount
        }
      )
    )
    await provider.sendAndConfirm(tx);
  }

  async function newPlayer() {
    const player = anchor.web3.Keypair.generate();
    await internal_airdrop(player.publicKey, INIT_FUND);
    return player;
  }

  async function findBetStatusPda(){
    const [_betStatus, _1] = await PublicKey.findProgramAddress(
      [Buffer.from(anchor.utils.bytes.utf8.encode("bet-status"))],
      program.programId
    );
    return _betStatus;
  }

  async function findBetStatus(){
    const _betStatusPda = await findBetStatusPda();

    const _betStatus = await program.account.betStatus.fetch(_betStatusPda);
    return _betStatus;
  }

  async function findBetInfoPda(pubKey, betId){
    const [_betInfo, _1] = await PublicKey.findProgramAddress(
      [Buffer.from(anchor.utils.bytes.utf8.encode("bet-info")), pubKey.toBuffer(), Buffer.from([betId])],
      program.programId
    );
    return _betInfo;
  }

  async function findBetInfo(pubKey, betId){
    const _betInfoPda = await findBetInfoPda(pubKey, betId);

    const _betInfo = await program.account.betInfo.fetch(_betInfoPda);
    return _betInfo;
  }

  function timeStamp(dateStr) {
    const date = new Date(dateStr); 
    const timestampInSeconds = Math.floor(date.getTime() / 1000);
    return new BN(timestampInSeconds);
  }

  async function balance(pubkey) {
    return await provider.connection.getBalance(pubkey);
  }

});

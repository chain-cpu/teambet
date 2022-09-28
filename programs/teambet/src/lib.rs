use anchor_lang::{prelude::*, system_program};
use anchor_spl::token::{self, Mint, Token, TokenAccount};
use borsh::{BorshDeserialize, BorshSerialize};

declare_id!("FeF3pjJPvu2SxWYnhpDwGrRBSLy73k5aTsFv9c4PgbxQ");

#[program]
pub mod teambet {
    use core::panic;

    use anchor_spl::token::accessor::amount;

    use super::*;

    pub fn initialize(ctx: Context<Initialize>,) -> Result<()> {
        msg!("Initialize Contract");
        ctx.accounts.bet_status.authority = ctx.accounts.authority.key();
        ctx.accounts.bet_status.bump = *ctx.bumps.get("bet_status").unwrap();
        Ok(())
    }

    pub fn new_bet(ctx: Context<NewBet>, start_date: i64, end_date: i64,) -> Result<()> {
        msg!("New Bet Initialize");
        ctx.accounts.bet_status.init(start_date, end_date);
        Ok(())
    }

    pub fn bet(ctx: Context<Bet>, team_id: u8, amount: u64) -> Result<()> {
        msg!("team {} bet, amount: {}", team_id, amount);
        ctx.accounts.bet_status.bet(team_id, amount);
        ctx.accounts.bet_info.bet(ctx.accounts.bet_status.id, ctx.accounts.payer.key().clone(), team_id, amount);
        let time_stamp = ctx.accounts.clock.unix_timestamp;
        assert!(time_stamp >= ctx.accounts.bet_status.start_date, "bet not started");
        assert!(time_stamp <= ctx.accounts.bet_status.end_date, "bet over end {}, clock {}", ctx.accounts.bet_status.end_date, time_stamp);

        system_program::transfer(CpiContext::new(
            ctx.accounts.system_program.to_account_info(), system_program::Transfer {
                from: ctx.accounts.payer.to_account_info(),
                to: ctx.accounts.bet_status.to_account_info(),
            }), amount);
        // ctx.accounts.bet_status.to_account_info().try_borrow_mut_lamports()?.checked_sub(amount);
        // ctx.accounts.payer.to_account_info().try_borrow_mut_lamports()?.checked_add(amount);
        
        ctx.accounts.bet_info.bump = *ctx.bumps.get("bet_info").unwrap();

        Ok(())
    }

    // finalize
    pub fn finalize(ctx: Context<Finalize>) -> Result<()> {
        msg!("finalize!");
        ctx.accounts.bet_status.finalize();
        // transfer sol to the vault
        Ok(())
    }

    pub fn transfer_ownership(ctx: Context<TransferOwnership>) -> Result<()> {
        msg!("transfer_ownership!");
        ctx.accounts.bet_status.tranfer_ownership(ctx.accounts.new_authority.key());
        Ok(())
    }
    // claim
    pub fn claim(ctx: Context<Claim>) -> Result<()> {
        let bet_status = &mut ctx.accounts.bet_status;
        let bet_info = &mut ctx.accounts.bet_info;
        assert_eq!(bet_info.team_id, bet_status.winner, "Not winner~!");
        assert_eq!(bet_info.claimed, 0, "Already claimed~!");
        msg!("total: {} amount_l: {}, amount_r: {}", bet_status.total(), bet_status.amount_l, bet_status.amount_r);
        bet_info.claim();
        let share = bet_status.share(bet_info.amount, bet_info.team_id);
        msg!("claim team {} amount: {}", bet_info.team_id, share);
        **ctx.accounts.bet_status.to_account_info().try_borrow_mut_lamports()? -= share;
        **ctx.accounts.payer.to_account_info().try_borrow_mut_lamports()? += share;
        Ok(())
    }

}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer=authority,
        space= 8 + 1 + 8 + 8 + 8 + 8 + 8 + 1 + 32 + 1,
        seeds = [b"bet-status".as_ref()],
        bump,
    )]
    bet_status: Account<'info, BetStatus>,
    #[account(mut)]
    pub authority: Signer<'info>,
    system_program: Program<'info, System>,
    token_program: Program<'info, Token>,
    rent: Sysvar<'info, Rent>,
}

// should check authority
// (program_sol_vault, payer, betstatus, )
#[derive(Accounts)]
pub struct NewBet<'info> {
    #[account(
        mut,
        seeds = [b"bet-status".as_ref()],
        bump=bet_status.bump,
    )]
    bet_status: Account<'info, BetStatus>,
    #[account(mut)]
    pub authority: Signer<'info>,
    system_program: Program<'info, System>,
    token_program: Program<'info, Token>,
    rent: Sysvar<'info, Rent>,
}

// bet (betstatus, team_id, amount, betinfo, user_sol_account, program_sol_vault)
#[derive(Accounts)]
pub struct Bet<'info> {
    #[account(        
        mut,
        seeds = [b"bet-status".as_ref()],
        bump = bet_status.bump,
    )]
    bet_status: Account<'info, BetStatus>,
    #[account(        
        init,
        payer = payer,
        space = 8 + 8 + 32 + 1 + 8 + 1 + 1, 
        seeds = [b"bet-info".as_ref(), payer.key().as_ref(), &[bet_status.id]],
        bump
    )]
    bet_info: Account<'info, BetInfo>,
    #[account(mut)]
    pub payer: Signer<'info>,
    system_program: Program<'info, System>,
    token_program: Program<'info, Token>,
    rent: Sysvar<'info, Rent>,
    clock: Sysvar<'info, Clock>,
}

// finalize (betstatus, authority)
#[derive(Accounts)]
pub struct Finalize<'info> {
    #[account(        
        mut,
        seeds = [b"bet-status".as_ref()],
        has_one=authority,
        bump = bet_status.bump,
    )]
    bet_status: Account<'info, BetStatus>,
    pub authority: Signer<'info>,
    system_program: Program<'info, System>,
    token_program: Program<'info, Token>,
    rent: Sysvar<'info, Rent>,
}


// finalize (betstatus, authority)
#[derive(Accounts)]
pub struct TransferOwnership<'info> {
    #[account(        
        mut,
        seeds = [b"bet-status".as_ref()],
        has_one=authority,
        bump = bet_status.bump,
    )]
    bet_status: Account<'info, BetStatus>,
    pub authority: Signer<'info>,
    /// CHECK:` doc comment explaining why no checks through types are necessary.
    pub new_authority: AccountInfo<'info>,
    system_program: Program<'info, System>,
    token_program: Program<'info, Token>,
    rent: Sysvar<'info, Rent>,
}

// should check authority

// claim (betstatus, user_sol_account, betinfo, payer)

#[derive(Accounts)]
pub struct Claim<'info> {
    #[account(        
        mut,
        seeds = [b"bet-status".as_ref()],
        bump = bet_status.bump,
    )]
    bet_status: Account<'info, BetStatus>,
    #[account(
        mut,
        seeds = [b"bet-info".as_ref(), payer.key().as_ref(), &[bet_status.id]],
        bump
    )]
    bet_info: Account<'info, BetInfo>,
    #[account(mut)]
    pub payer: Signer<'info>,
    system_program: Program<'info, System>,
    token_program: Program<'info, Token>,
    rent: Sysvar<'info, Rent>,
}

// space= 1 + 1 + 8 + 8 + 8 + 8 + 8 + 1 + 32 + 1,
// BetStatus (id, start, end, amount_l, amount_r, winner, authority)
#[account]
pub struct BetStatus {
    pub id: u8, // This is the id of the Bet
    pub amount_l: u64,
    pub amount_r: u64,
    pub amount_m: u64,
    pub start_date: i64,
    pub end_date: i64,
    pub winner: u8,
    pub authority: Pubkey,
    pub bump: u8,
}

// BetInfo (player_pubkey, bet_id, amount, claimed, team_info)
#[account]
pub struct BetInfo {
    pub bet_id: u8, // This is the id of the Bet
    pub payer: Pubkey,
    pub team_id: u8,
    pub amount: u64,
    pub claimed: u8,
    pub bump: u8,
}
impl BetStatus {
    pub fn init(&mut self, start_date: i64, end_date: i64) {
        self.id = self.id + 1;
        self.amount_l = 0;
        self.amount_r = 0;
        self.amount_m = 0;
        self.winner = Team::NOTYET as u8;
        self.start_date = start_date;
        self.end_date = end_date;

    }

    pub fn bet(&mut self, team_id: u8, amount: u64) {
        match team_id {
            1 => self.amount_l += amount,
            2 => self.amount_r += amount,
            3 => self.amount_m += amount,
            _ => (),
        };
    }

    pub fn total(&self) -> u64 {
        self.amount_l + self.amount_r + self.amount_m
    }

    pub fn fee(&self) -> u64 {
        self.total() * 10 / 100

    }
    pub fn tranfer_ownership(&mut self, new_owner: Pubkey) {
        self.authority = new_owner;
    }
    pub fn finalize(&mut self) {
        self.winner = get_random();
    }

    pub fn share(&self, amount: u64, team_id: u8) -> u64 {
        (self.total() - self.fee()) * amount / ( match team_id {
            1 => self.amount_l,
            2 => self.amount_r,
            3 => self.amount_m,
            _ => 0,
        })
    }

}

impl BetInfo {
    pub fn bet(&mut self, bet_id: u8, pub_key: Pubkey, team_id: u8, amount: u64) {
        self.bet_id = bet_id;
        self.team_id = team_id;
        self.amount = amount;
        self.payer = pub_key;
    }

    pub fn claim(&mut self) {
        assert_eq!(self.claimed, 0);
        self.claimed = 1;
    }
}

#[derive(BorshSerialize, BorshDeserialize, Clone, Copy)]
pub enum Team {
    NOTYET = 0,
    TEAM_L = 1,
    TEAM_R = 2,
    TEAM_M = 3,
}

fn get_random() -> u8 {
    1
}



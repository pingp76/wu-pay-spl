use anchor_lang::prelude::*;
use anchor_spl::token::{CloseAccount, Mint, Token, TokenAccount, Transfer, transfer,SyncNative, sync_native};

declare_id!("HnHcveP6wj7wV1vwcXEHCm7792P2b6fr6VqGjB1yVWfv");


const SEEDSTATE:&[u8] = b"escrow_state";
const SEEDWALLET:&[u8] = b"escrow_wallet";

#[error_code]
pub enum ErrorCode {
    #[msg("The amount to withdrwa does not match the amount in the escrow state")]
    InvalidAmount,
    #[msg("The bump seed for the wallet does not match the escrow state's wallet bump seed")]
    InvalidWalletBump,
    #[msg("The index does not match the trasaction index in the escrow state")]
    InvalidIndex,
}

#[program]
pub mod wu_pay_spl {

    use super::*;

    pub fn deposite_grant(_ctx: Context<DepositeGrant>, _deposite_idx:u64, _state_bump:u8, _wallet_bump:u8, _amount:u64) -> Result<()> {
        let escrow_state = &mut _ctx.accounts.escrow_state;
        escrow_state.sender = _ctx.accounts.sender.key();
        escrow_state.receiver = _ctx.accounts.receiver.key();
        escrow_state.escrow_wallet = _ctx.accounts.escrow_wallet.key();
        escrow_state.idx = _deposite_idx;
        escrow_state.amount = _amount;
        escrow_state.wallet_bump = _wallet_bump;

        /*let seeds = [
            SEEDSTATE,
            _ctx.accounts.sender.key.as_ref(),
            _ctx.accounts.receiver.key.as_ref(),
            _deposite_idx.to_le_bytes().as_ref(),
            _ctx.accounts.mint_of_token_being_sent.key().as_ref(),
        ];*/

        let transfer_instruction = Transfer {
            from: _ctx.accounts.sender_ata.to_account_info(),
            to: _ctx.accounts.escrow_wallet.to_account_info(),
            authority: _ctx.accounts.sender.to_account_info(),
        };
        
        let cpictx = CpiContext::new(
            _ctx.accounts.token_program.to_account_info(),
            transfer_instruction,
        );
        
        transfer(cpictx, _amount)?;
        Ok(())
    }

    pub fn complete_grant(_ctx: Context<CompleteGrant>, _deposite_idx:u64, _state_bump:u8) -> Result<()> {
        let escrow_state = &_ctx.accounts.escrow_state;
        let escrow_wallet = &mut _ctx.accounts.escrow_wallet;
        let sender = &_ctx.accounts.sender;
        let receiver = &_ctx.accounts.receiver;
        let receiver_wallet = &mut _ctx.accounts.receiver_ata;
        let _deposite_idx_bytes = _deposite_idx.to_le_bytes();
        //let _wallet_bump_bytes = _ctx.accounts.escrow_state.wallet_bump.to_le_bytes();
        let _state_bump_bytes = _state_bump.to_le_bytes();

        // Rest of the code...
        let inner = vec![
            SEEDSTATE.as_ref(),
            sender.key.as_ref(),
            receiver.key.as_ref(),
            &_deposite_idx_bytes.as_ref(),
            &_state_bump_bytes.as_ref(),
        ];
        let outer = vec![inner.as_slice()];

        let transfer_instruction = Transfer {
            from: escrow_wallet.to_account_info(),
            to: receiver_wallet.to_account_info(),
            authority: escrow_state.to_account_info(),
        };

        let cpictx = CpiContext::new_with_signer(
            _ctx.accounts.token_program.to_account_info(),
            transfer_instruction,
            outer.as_slice(),
        );
        msg!("Transferring {} to receiver's wallet...", escrow_state.amount);
        transfer(cpictx, escrow_state.amount)?;

        // Sync the native token to reflect the new SOL balance as wSOL
        let cpi_accounts = SyncNative {
            account: receiver_wallet.to_account_info(),
        };
        let cpictx = CpiContext::new(_ctx.accounts.token_program.to_account_info(), cpi_accounts);
        sync_native(cpictx)?;
        Ok(())
    }

    pub fn withdraw_grant(_ctx: Context<WithdrwaGrant>, _deposite_idx:u64, _state_bump:u8) -> Result<()> {
        let escrow_state = &_ctx.accounts.escrow_state;
        let escrow_wallet = &mut _ctx.accounts.escrow_wallet;
        let sender = &_ctx.accounts.sender;
        let receiver = &_ctx.accounts.receiver;
        let sender_wallet = &mut _ctx.accounts.sender_ata;
        let _deposite_idx_bytes = _deposite_idx.to_le_bytes();
        //let _wallet_bump_bytes = _ctx.accounts.escrow_state.wallet_bump.to_le_bytes();

        msg!("Creating seeds for escrow wallet and state...");
        let _state_bump_bytes = _state_bump.to_le_bytes();

        // Rest of the code...
        let inner = vec![
            SEEDSTATE.as_ref(),
            sender.key.as_ref(),
            receiver.key.as_ref(),
            &_deposite_idx_bytes.as_ref(),
            &_state_bump_bytes.as_ref(),
        ];
        let outer = vec![inner.as_slice()];

        let transfer_instruction = Transfer {
            from: escrow_wallet.to_account_info(),
            to: sender_wallet.to_account_info(),
            authority: escrow_state.to_account_info(),
        };

        let cpictx = CpiContext::new_with_signer(
            _ctx.accounts.token_program.to_account_info(),
            transfer_instruction,
            outer.as_slice(),
        );
        msg!("Transferring {} back to sender's wallet...", escrow_state.amount);
        transfer(cpictx, escrow_state.amount)?;

        // Sync the native token to reflect the new SOL balance as wSOL
        let cpi_accounts = SyncNative {
            account: sender_wallet.to_account_info(),
        };
        let cpictx = CpiContext::new(_ctx.accounts.token_program.to_account_info(), cpi_accounts);
        sync_native(cpictx)?;
        Ok(())
    }

    pub fn close_escrow(_ctx: Context<CloseEscrow>, _deposite_idx:u64, _state_bump:u8) -> Result<()> {
        let escrow_state = &_ctx.accounts.escrow_state;
        let escrow_wallet = &_ctx.accounts.escrow_wallet;
        let sender = &_ctx.accounts.sender;
        let receiver = &_ctx.accounts.receiver;
        let _deposite_idx_bytes = _deposite_idx.to_le_bytes();

        let _state_bump_bytes = _state_bump.to_le_bytes();
        // Rest of the code...
        let inner = vec![
            SEEDSTATE.as_ref(),
            sender.key.as_ref(),
            receiver.key.as_ref(),
            &_deposite_idx_bytes.as_ref(),
            &_state_bump_bytes.as_ref(),
        ];
        let outer = vec![inner.as_slice()];

        let ca = CloseAccount {
            account: escrow_wallet.to_account_info(),
            destination: sender.to_account_info(),
            authority: escrow_state.to_account_info(),
        };
        let cpi_ctx =
            CpiContext::new_with_signer(_ctx.accounts.token_program.to_account_info(), ca, outer.as_slice());
        anchor_spl::token::close_account(cpi_ctx)?;
        Ok(())
    }

}


#[derive(Accounts)]
#[instruction(deposite_idx:u64, state_bump:u8, wallet_bump:u8)]
pub struct DepositeGrant<'info> {
    /// CHECK: this is safe, will not write to this account
    #[account(mut)]
    sender: Signer<'info>,
    #[account(
        mut,
        constraint = sender_ata.owner == sender.key(),
        constraint = sender_ata.mint == mint_of_token_being_sent.key(), 
    )]
    sender_ata: Account<'info,TokenAccount>,

    /// CHECK: this is safe, will not write to this account
    receiver: AccountInfo<'info>, 
    //receiver_ata: Account<'info,TokenAccount>,

    mint_of_token_being_sent: Account<'info, Mint>, 

    #[account(
        init,
        payer = sender,
        space = EscrowState::LEN,
        seeds = [
            SEEDSTATE,
            sender.key.as_ref(),
            receiver.key.as_ref(),
            deposite_idx.to_le_bytes().as_ref(),
        ],
        bump 
    )]
    escrow_state: Account<'info, EscrowState>,

    #[account(
        init,
        payer = sender,
        seeds = [
            SEEDWALLET,
            sender.key.as_ref(),
            receiver.key.as_ref(),
            deposite_idx.to_le_bytes().as_ref(),
        ],
        token::mint = mint_of_token_being_sent,
        token::authority = escrow_state,
        bump,
    )]
    escrow_wallet: Account<'info,TokenAccount>,

    system_program: Program<'info, System>,
    token_program: Program<'info, Token>,
    rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
#[instruction(deposite_idx:u64, state_bump:u8)]
pub struct CompleteGrant<'info> {
    /// CHECK: this is safe, will not write to this account
    #[account(mut)]
    sender: Signer<'info>,

    /// CHECK: this is safe, will not write to this account
    #[account(mut)]
    receiver: AccountInfo<'info>, 
    #[account(
        mut,
        constraint = receiver_ata.owner == receiver.key(),
        //constraint = receiver_ata.mint == mint_of_token_being_sent.key(), 
    )]
    receiver_ata: Account<'info,TokenAccount>,

    //mint_of_token_being_sent: Account<'info, Mint>, 

    #[account(
        mut,
        seeds = [
            SEEDSTATE,
            sender.key.as_ref(),
            receiver.key.as_ref(),
            deposite_idx.to_le_bytes().as_ref(),
        ],
        bump = state_bump,
    )]
    escrow_state: Account<'info, EscrowState>,

    #[account(
        mut,
        seeds = [
            SEEDWALLET,
            sender.key.as_ref(),
            receiver.key.as_ref(),
            deposite_idx.to_le_bytes().as_ref(),
        ],
        bump = escrow_state.wallet_bump,
    )]
    escrow_wallet: Account<'info,TokenAccount>,

    system_program: Program<'info, System>,
    token_program: Program<'info, Token>,
    //associated_token_program: Program<'info, AssociatedToken>,
    rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
#[instruction(deposite_idx:u64, state_bump:u8)]
pub struct WithdrwaGrant<'info> {
    /// CHECK: this is safe, will not write to this account
    #[account(mut)]
    sender: Signer<'info>,
    #[account(
        mut,
        constraint = sender_ata.owner == sender.key(),
        constraint = sender_ata.mint == escrow_state.mint_of_token_being_sent.key(), 
    )]
    sender_ata: Account<'info,TokenAccount>,
    /// CHECK: this is safe, will not write to this account
    receiver: AccountInfo<'info>, 

    #[account(
        mut,
        seeds = [
            SEEDSTATE,
            sender.key.as_ref(),
            receiver.key.as_ref(),
            deposite_idx.to_le_bytes().as_ref(),
        ],
        bump = state_bump,
    )]
    escrow_state: Account<'info, EscrowState>,

    #[account(
        mut,
        seeds = [
            SEEDWALLET,
            sender.key.as_ref(),
            receiver.key.as_ref(),
            deposite_idx.to_le_bytes().as_ref(),
        ],
        bump = escrow_state.wallet_bump,
    )]
    escrow_wallet: Account<'info,TokenAccount>,

    system_program: Program<'info, System>,
    token_program: Program<'info, Token>,
    rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
#[instruction(deposite_idx:u64, state_bump:u8)]
pub struct CloseEscrow<'info> {
    /// CHECK: this is safe, will not write to this account
    #[account(mut)]
    sender: Signer<'info>,
    
    /// CHECK: this is safe, will not write to this account
    receiver: AccountInfo<'info>, 

    #[account(
        mut,
        seeds = [
            SEEDSTATE,
            sender.key.as_ref(),
            receiver.key.as_ref(),
            deposite_idx.to_le_bytes().as_ref(),
        ],
        close=sender,
        bump = state_bump,
    )]
    escrow_state: Account<'info, EscrowState>,

    #[account(
        mut,
        seeds = [
            SEEDWALLET,
            sender.key.as_ref(),
            receiver.key.as_ref(),
            deposite_idx.to_le_bytes().as_ref(),
        ],
        bump = escrow_state.wallet_bump,
    )]
    escrow_wallet: Account<'info,TokenAccount>,
    system_program: Program<'info, System>,
    token_program: Program<'info, Token>
}

#[account]
#[derive(Default)]
pub struct EscrowState {
    sender: Pubkey,
    receiver: Pubkey,
    escrow_wallet: Pubkey,
    wallet_bump: u8,
    mint_of_token_being_sent: Pubkey,
    idx: u64,
    amount: u64,
}

impl EscrowState {
    const LEN:usize = 8 + 32*4 + 8*2 + 1;
}



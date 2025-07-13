use crate::{error::EscrowError, state::Escrow};
use pinocchio::{
    account_info::AccountInfo,
    program::{invoke, invoke_signed},
    msg,
    program_error::ProgramError,
    pubkey::Pubkey,
    ProgramResult,
    spl_token,
};

use super::make::{TOKEN_PROGRAM_ID, find_vault_address};

// Accounts needed for the Take instruction
pub struct TakeAccounts<'a> {
    pub taker: &'a AccountInfo,
    pub maker: &'a AccountInfo,
    pub escrow: &'a AccountInfo,
    pub vault: &'a AccountInfo,
    pub mint_a: &'a AccountInfo,
    pub mint_b: &'a AccountInfo,
    pub taker_ata_a: &'a AccountInfo,
    pub taker_ata_b: &'a AccountInfo,
    pub maker_ata_b: &'a AccountInfo,
    pub token_program: &'a AccountInfo,
}

// complete an escrow by taking the offer
pub fn take(
    program_id: &Pubkey,
    accounts: TakeAccounts,
    amount: u64,
    seed: u64,
) -> ProgramResult {
    msg!(&format!("Take instruction: amount={}, seed={}", amount, seed));
    
    // verify the taker is a signer
    if !accounts.taker.is_signer() {
        return Err(ProgramError::MissingRequiredSignature);
    }
    
    // verify token program
    if accounts.token_program.key() != &TOKEN_PROGRAM_ID {
        return Err(EscrowError::InvalidTokenProgram.into());
    }
    
    // verify the escrow account (and load it)
    let escrow = Escrow::from_account(accounts.escrow)?;
    
    // verify the maker matches
    if escrow.maker != *accounts.maker.key() {
        return Err(EscrowError::InvalidAuthority.into());
    }
    
    // verify mints match
    if escrow.mint_a != *accounts.mint_a.key() || escrow.mint_b != *accounts.mint_b.key() {
        return Err(EscrowError::InvalidTokenMint.into());
    }
    
    // verify the maker's receive account
    if escrow.receive_account != *accounts.maker_ata_b.key() {
        return Err(ProgramError::InvalidAccountData);
    }

    // verify the amount matches
    if escrow.amount != amount {
        return Err(EscrowError::ExpectedAmountMismatch.into());
    }
    
    // derive and verify vault address
    let (vault_key, vault_bump) = find_vault_address(
        accounts.escrow.key(),
        program_id,
    );
    if vault_key != *accounts.vault.key() {
        return Err(EscrowError::InvalidEscrowAccount.into());
    }
    
    // transfer token B from Taker to Maker
    let transfer_b_ix = spl_token::transfer(
        &TOKEN_PROGRAM_ID,
        &[
            spl_token::TransferParams {
                from: accounts.taker_ata_b.key(),
                to: accounts.maker_ata_b.key(),
                authority: accounts.taker.key(),
                amount: escrow.amount,
            },
        ],
    )?;
    
    invoke(
        &transfer_b_ix,
        &[
            accounts.taker_ata_b,
            accounts.maker_ata_b,
            accounts.taker,
        ],
    )?;
    
    // transfer token A from vault to Taker
    let transfer_a_ix = spl_token::transfer(
        &TOKEN_PROGRAM_ID,
        &[
            spl_token::TransferParams {
                from: accounts.vault.key(),
                to: accounts.taker_ata_a.key(),
                authority: accounts.escrow.key(),
                amount: escrow.amount,
            },
        ],
    )?;
    
    let vault_signer_seeds = &[
        b"vault" as &[u8],
        accounts.escrow.key().as_ref(),
        &[vault_bump],
    ];
    
    invoke_signed(
        &transfer_a_ix,
        &[
            accounts.vault,
            accounts.taker_ata_a,
            accounts.escrow,
        ],
        &[vault_signer_seeds],
    )?;
    
    // close the vault account
    let close_vault_ix = spl_token::close_account(
        &TOKEN_PROGRAM_ID,
        &[
            spl_token::CloseAccountParams {
                account: accounts.vault.key(),
                destination: accounts.taker.key(),
                authority: accounts.escrow.key(),
            },
        ],
    )?;
    
    invoke_signed(
        &close_vault_ix,
        &[
            accounts.vault,
            accounts.taker,
            accounts.escrow,
        ],
        &[vault_signer_seeds],
    )?;
    
    // close the escrow account and return lamports to Taker
    let escrow_lamports = accounts.escrow.lamports();
    *accounts.escrow.try_borrow_mut_lamports()? = 0;
    *accounts.taker.try_borrow_mut_lamports()? += escrow_lamports;
    
    // clear the escrow data
    let mut escrow_data = accounts.escrow.try_borrow_mut_data()?;
    escrow_data.fill(0);
    
    msg!("Escrow completed successfully");
    Ok(())
} 
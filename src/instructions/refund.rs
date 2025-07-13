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

// Accounts for the fefund instruction
pub struct RefundAccounts<'a> {
    pub maker: &'a AccountInfo,
    pub escrow: &'a AccountInfo,
    pub vault: &'a AccountInfo,
    pub maker_ata_a: &'a AccountInfo,
    pub token_program: &'a AccountInfo,
}

// Refund escrow, cancel and return tokens to maker
pub fn refund(
    program_id: &Pubkey,
    accounts: RefundAccounts,
    amount: u64,
    seed: u64,
) -> ProgramResult {
    msg!(&format!("Refund instruction: amount={}, seed={}", amount, seed));
    
    // Verify the maker is a signer
    if !accounts.maker.is_signer() {
        return Err(ProgramError::MissingRequiredSignature);
    }
    
    // Verify token program
    if accounts.token_program.key() != &TOKEN_PROGRAM_ID {
        return Err(EscrowError::InvalidTokenProgram.into());
    }

    // verify the escrow account (and load it)
    let escrow = Escrow::from_account(accounts.escrow)?;
    
    // verify if the maker matches
    if escrow.maker != *accounts.maker.key() {
        return Err(EscrowError::InvalidAuthority.into());
    }

    // verify if the amount matches
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
    
    // transfer tokens from vault back to maker
    let transfer_ix = spl_token::transfer(
        &TOKEN_PROGRAM_ID,
        &[
            spl_token::TransferParams {
                from: accounts.vault.key(),
                to: accounts.maker_ata_a.key(),
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
        &transfer_ix,
        &[
            accounts.vault,
            accounts.maker_ata_a,
            accounts.escrow,
        ],
        &[vault_signer_seeds],
    )?;
    
    //close the vault account
    let close_vault_ix = spl_token::close_account(
        &TOKEN_PROGRAM_ID,
        &[
            spl_token::CloseAccountParams {
                account: accounts.vault.key(),
                destination: accounts.maker.key(),
                authority: accounts.escrow.key(),
            },
        ],
    )?;
    
    invoke_signed(
        &close_vault_ix,
        &[
            accounts.vault,
            accounts.maker,
            accounts.escrow,
        ],
        &[vault_signer_seeds],
    )?;
    
    // close the escrow account and return lamports to maker
    let escrow_lamports = accounts.escrow.lamports();
    *accounts.escrow.try_borrow_mut_lamports()? = 0;
    *accounts.maker.try_borrow_mut_lamports()? += escrow_lamports;
    
    // clear escrow data
    let mut escrow_data = accounts.escrow.try_borrow_mut_data()?;
    escrow_data.fill(0);
    
    msg!("Escrow refunded successfully");
    Ok(())
} 
use crate::{error::EscrowError, state::Escrow};
use pinocchio::{
    account_info::AccountInfo,
    program::{invoke, invoke_signed},
    msg,
    program_error::ProgramError,
    pubkey::Pubkey,
    ProgramResult,
    system_program,
    spl_token,
};

// Pinocchio constants
pub use spl_token::ID as TOKEN_PROGRAM_ID;
pub use system_program::ID as SYSTEM_PROGRAM_ID;

// find the escrow account PDA
pub fn find_escrow_address(
    maker: &Pubkey,
    seed: u64,
    program_id: &Pubkey,
) -> (Pubkey, u8) {
    let seed_bytes = seed.to_le_bytes();
    Pubkey::find_program_address(
        &[
            b"escrow",
            maker.as_ref(),
            &seed_bytes,
        ],
        program_id,
    )
}

// find the vault account PDA
pub fn find_vault_address(
    escrow: &Pubkey,
    program_id: &Pubkey,
) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            b"vault",
            escrow.as_ref(),
        ],
        program_id,
    )
}

// accounts for Make instruction
pub struct MakeAccounts<'a> {
    pub maker: &'a AccountInfo,
    pub mint_a: &'a AccountInfo,
    pub mint_b: &'a AccountInfo,
    pub maker_ata_a: &'a AccountInfo,
    pub escrow: &'a AccountInfo,
    pub vault: &'a AccountInfo,
    pub token_program: &'a AccountInfo,
    pub system_program: &'a AccountInfo,
}

//create an escrow
pub fn make(
    program_id: &Pubkey,
    accounts: MakeAccounts,
    amount: u64,
    seed: u64,
) -> ProgramResult {
    msg!(&format!("Make instruction: amount={}, seed={}", amount, seed));
    
    // Verify the maker is a signer
    if !accounts.maker.is_signer() {
        return Err(ProgramError::MissingRequiredSignature);
    }
    
    // verify programs
    if accounts.system_program.key().as_ref() != &SYSTEM_PROGRAM_ID {
        return Err(ProgramError::IncorrectProgramId);
    }
    
    if accounts.token_program.key().as_ref() != &TOKEN_PROGRAM_ID {
        return Err(EscrowError::InvalidTokenProgram.into());
    }
    
    // derive and verify escrow address
    let (escrow_key, escrow_bump) = find_escrow_address(
        accounts.maker.key(),
        seed,
        program_id,
    );
    if escrow_key != *accounts.escrow.key() {
        return Err(EscrowError::InvalidEscrowAccount.into());
    }
    
    // create the escrow account
    let escrow_size = Escrow::LEN;
    // Calculate minimum balance for rent exemption (1.5x the size in lamports as approximation)
    let lamports = ((escrow_size as u64) * 3564480) / 165;
    
    // create account instruction data
    let mut create_account_data = vec![0u8]; // CreateAccount discriminator
    create_account_data.extend_from_slice(&lamports.to_le_bytes());
    create_account_data.extend_from_slice(&(escrow_size as u64).to_le_bytes());
    create_account_data.extend_from_slice(program_id.as_ref());
    
    let create_account_ix = system_program::create_account(
        &SYSTEM_PROGRAM_ID,
        &[
            system_program::CreateAccountParams {
                from: accounts.maker.key(),
                new_account: accounts.escrow.key(),
                lamports,
                space: escrow_size,
                owner: program_id,
            },
        ],
    )?;
    
    let seed_bytes = seed.to_le_bytes();
    let escrow_signer_seeds = &[
        b"escrow" as &[u8],
        accounts.maker.key().as_ref(),
        &seed_bytes,
        &[escrow_bump],
    ];
    
    invoke_signed(
        &create_account_ix,
        &[
            accounts.maker,
            accounts.escrow,
            accounts.system_program,
        ],
        &[escrow_signer_seeds],
    )?;
    
    // Initialize the escrow state
    Escrow::init(
        accounts.escrow,
        *accounts.maker.key(),
        *accounts.mint_a.key(),
        *accounts.mint_b.key(),
        *accounts.maker_ata_a.key(), // This will be the receive account for token B
        amount,
        escrow_bump,
    )?;
    
    // derive and verify vault address
    let (vault_key, vault_bump) = find_vault_address(
        accounts.escrow.key(),
        program_id,
    );
    if vault_key != *accounts.vault.key() {
        return Err(EscrowError::InvalidEscrowAccount.into());
    }
    
    // Create vault token account
    let vault_size = 165; // SPL Token account size
    let vault_lamports = ((vault_size as u64) * 3564480) / 165;
    
    // create vault account instruction data
    let mut create_vault_data = vec![0u8]; // CreateAccount discriminator
    create_vault_data.extend_from_slice(&vault_lamports.to_le_bytes());
    create_vault_data.extend_from_slice(&(vault_size as u64).to_le_bytes());
    create_vault_data.extend_from_slice(&TOKEN_PROGRAM_ID);
    
    let create_vault_ix = system_program::create_account(
        &SYSTEM_PROGRAM_ID,
        &[
            system_program::CreateAccountParams {
                from: accounts.maker.key(),
                new_account: accounts.vault.key(),
                lamports: vault_lamports,
                space: vault_size,
                owner: program_id,
            },
        ],
    )?;
    
    let vault_signer_seeds = &[
        b"vault" as &[u8],
        accounts.escrow.key().as_ref(),
        &[vault_bump],
    ];
    
    invoke_signed(
        &create_vault_ix,
        &[
            accounts.maker,
            accounts.vault,
            accounts.system_program,
        ],
        &[vault_signer_seeds],
    )?;
    
    // Initialize vault token account
    // and InitializeAccount3 instruction discriminator
    let init_data = vec![18u8]; // InitializeAccount3 discriminator
    
    let init_vault_ix = spl_token::initialize_account(
        &TOKEN_PROGRAM_ID,
        &[
            spl_token::InitializeAccountParams {
                account: accounts.vault.key(),
                mint: accounts.mint_a.key(),
                owner: program_id,
            },
        ],
    )?;
    
    invoke_signed(
        &init_vault_ix,
        &[
            accounts.vault,
            accounts.mint_a,
        ],
        &[vault_signer_seeds],
    )?;
    
    // transfer tokens from maker to vault
    // transfer instruction: discriminator (1) + amount (8)
    let mut transfer_data = vec![3u8]; // Transfer discriminator
    transfer_data.extend_from_slice(&amount.to_le_bytes());
    
    let transfer_ix = spl_token::transfer(
        &TOKEN_PROGRAM_ID,
        &[
            spl_token::TransferParams {
                from: accounts.maker_ata_a.key(),
                to: accounts.vault.key(),
                authority: accounts.maker.key(),
                amount: amount,
            },
        ],
    )?;
    
    invoke(
        &transfer_ix,
        &[
            accounts.maker_ata_a,
            accounts.vault,
            accounts.maker,
        ],
    )?;
    
    msg!("Escrow created successfully");
    Ok(())
} 
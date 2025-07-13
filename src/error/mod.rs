use pinocchio::{msg, program_error::ProgramError};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum EscrowError {
    #[error("Invalid Instruction")]
    InvalidInstruction,
    
    #[error("Not Rent Exempt")]
    NotRentExempt,
    
    #[error("Expected Amount Mismatch")]
    ExpectedAmountMismatch,
    
    #[error("Amount Overflow")]
    AmountOverflow,
    
    #[error("Invalid State")]
    InvalidState,
    
    #[error("Invalid Authority")]
    InvalidAuthority,
    
    #[error("Invalid Token Program")]
    InvalidTokenProgram,
    
    #[error("Invalid Token Mint")]
    InvalidTokenMint,
    
    #[error("Invalid Escrow Account")]
    InvalidEscrowAccount,
}

impl From<EscrowError> for ProgramError {
    fn from(e: EscrowError) -> Self {
        msg!(&format!("Escrow error: {}", e));
        ProgramError::Custom(e as u32)
    }
} 
use pinocchio::{
    account_info::AccountInfo,
    pubkey::Pubkey,
    program_error::ProgramError,
    account_validation::{AccountValidation, ValidateAccount},
};

// Escrow account structure
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct Escrow {
    // discriminator to identify account type
    pub discriminator: [u8; 8],
    
    // the maker who created the escrow
    pub maker: Pubkey,
    
    // the mint of token A (Maker's token)
    pub mint_a: Pubkey,
    
    // the mint of token B (Taker's token)
    pub mint_b: Pubkey,
    
    // the maker's token account for receiving token B
    pub receive_account: Pubkey,
    
    // the amount of token A the maker deposits
    pub amount: u64,
    
    // bump seed for the escrow PDA
    pub bump: u8,
}

impl AccountValidation for Escrow {
    fn validate_account<'a>(account: &'a AccountInfo) -> Result<&'a mut Self, ProgramError> {
        let escrow = unsafe {
            let mut data = account.try_borrow_mut_data()?;
            let escrow = &mut *(data.as_mut_ptr() as *mut Escrow);
            
            // Verify discriminator
            if escrow.discriminator != Self::DISCRIMINATOR {
                return Err(ProgramError::InvalidAccountData);
            }
            
            escrow
        };
        
        Ok(escrow)
    }
}

impl Escrow {
    pub const LEN: usize = 8 + 32 + 32 + 32 + 32 + 8 + 1;
    pub const DISCRIMINATOR: [u8; 8] = [139, 11, 230, 78, 92, 65, 103, 116];
    
    // initialize a new Escrow account
    pub fn init(
        account: &AccountInfo,
        maker: Pubkey,
        mint_a: Pubkey,
        mint_b: Pubkey,
        receive_account: Pubkey,
        amount: u64,
        bump: u8,
    ) -> Result<(), ProgramError> {
        let escrow = Escrow {
            discriminator: Self::DISCRIMINATOR,
            maker,
            mint_a,
            mint_b,
            receive_account,
            amount,
            bump,
        };
        
        unsafe {
            let mut data = account.try_borrow_mut_data()?;
            let dst = data.as_mut_ptr() as *mut Escrow;
            *dst = escrow;
        }
        
        Ok(())
    }
    
    // load an Escrow account from the AccountInfo
    pub fn from_account(account: &AccountInfo) -> Result<&mut Self, ProgramError> {
        Self::validate_account(account)
    }
    
    //check if the account has been initialized
    pub fn is_initialized(&self) -> bool {
        self.discriminator == Self::DISCRIMINATOR
    }
} 
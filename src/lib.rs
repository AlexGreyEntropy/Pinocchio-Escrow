//Pinocchio Escrow Library
use pinocchio::{
    declare_id,
    program::{
        invoke,
        invoke_signed,
    },
    account_info::AccountInfo,
    entrypoint,
    msg,
    program_error::ProgramError,
    pubkey::Pubkey,
    ProgramResult,
};

pub mod error;
pub mod instructions;
pub mod state;

pub use error::EscrowError;
pub use instructions::{
    make::{make, MakeAccounts},
    refund::{refund, RefundAccounts},
    take::{take, TakeAccounts},
};
pub use state::Escrow;

// declare program ID
declare_id!("DVVd1pDf9TaTyhep1iYh7S111Hir4SQeqhhAG65m2CFB");

// instruction enum for the escrow program
#[derive(Debug)]
pub enum EscrowInstruction {
    // Make instruction accounts:
    // 0. `[signer]` Maker
    // 1. `[]` Mint A
    // 2. `[]` Mint B  
    // 3. `[writable]` Maker ATA A
    // 4. `[writable]` escrow account (PDA)
    // 5. `[writable]` vault account (PDA)
    // 6. `[]` token program
    // 7. `[]` system program
    Make { amount: u64, seed: u64 },
    
    // Take an escrow offer 
    // 0. `[signer]` Taker
    // 1. `[]` Maker
    // 2. `[writable]` escrow account
    // 3. `[writable]` vault account
    // 4. `[]` Mint A
    // 5. `[]` Mint B
    // 6. `[writable]` Taker ATA A
    // 7. `[writable]` Taker ATA B
    // 8. `[writable]` Maker ATA B
    // 9. `[]` token program
    Take { amount: u64 },

    // refund an escrow
    // accounts:
    // 0. `[signer]` Maker
    // 1. `[writable]` Escrow account
    // 2. `[writable]` Vault account
    // 3. `[writable]` Maker's ATA A
    // 4. `[]` token program
    Refund { amount: u64 },
}

impl EscrowInstruction {
    //unpack instruction data
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        if input.is_empty() {
            return Err(EscrowError::InvalidInstruction.into());
        }
        
        match input[0] {
            0 => {
                if input.len() < 17 {
                    return Err(EscrowError::InvalidInstruction.into());
                }
                let amount = u64::from_le_bytes(input[1..9].try_into().unwrap());
                let seed = u64::from_le_bytes(input[9..17].try_into().unwrap());
                Ok(EscrowInstruction::Make { amount, seed })
            }
            1 => {
                if input.len() < 17 {
                    return Err(EscrowError::InvalidInstruction.into());
                }
                let amount = u64::from_le_bytes(input[1..9].try_into().unwrap());
                let seed = u64::from_le_bytes(input[9..17].try_into().unwrap());
                Ok(EscrowInstruction::Take { amount, seed })
            }
            2 => {
                if input.len() < 17 {
                    return Err(EscrowError::InvalidInstruction.into());
                }
                let amount = u64::from_le_bytes(input[1..9].try_into().unwrap());
                let seed = u64::from_le_bytes(input[9..17].try_into().unwrap());
                Ok(EscrowInstruction::Refund { amount, seed })
            }
            _ => Err(EscrowError::InvalidInstruction.into()),
        }
    }
}

// process instruction.. main entry point for the program
pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let instruction = EscrowInstruction::unpack(instruction_data)?;
    
    match instruction {
        EscrowInstruction::Make { amount, seed } => {
            msg!(&format!("Processing Make instruction"));
            let accounts = MakeAccounts {
                maker: &accounts[0],
                mint_a: &accounts[1],
                mint_b: &accounts[2],
                maker_ata_a: &accounts[3],
                escrow: &accounts[4],
                vault: &accounts[5],
                token_program: &accounts[6],
                system_program: &accounts[7],
            };
            make(program_id, accounts, amount, seed)
        }
        EscrowInstruction::Take { amount, seed } => {
            msg!(&format!("Processing Take instruction"));
            let accounts = TakeAccounts {
                taker: &accounts[0],
                maker: &accounts[1],
                escrow: &accounts[2],
                vault: &accounts[3],
                mint_a: &accounts[4],
                mint_b: &accounts[5],
                taker_ata_a: &accounts[6],
                taker_ata_b: &accounts[7],
                maker_ata_b: &accounts[8],
                token_program: &accounts[9],
            };
            take(program_id, accounts, amount, seed)
        }
        EscrowInstruction::Refund { amount, seed } => {
            msg!(&format!("Processing Refund instruction"));
            let accounts = RefundAccounts {
                maker: &accounts[0],
                escrow: &accounts[1],
                vault: &accounts[2],
                maker_ata_a: &accounts[3],
                token_program: &accounts[4],
            };
            refund(program_id, accounts, amount, seed)
        }
    }
}

// declare entrypoint if building as a program
#[cfg(not(feature = "no-entrypoint"))]
#[cfg(target_os = "solana")]
entrypoint!(process_instruction);

// helper function for creating instruction data
pub fn pack_instruction_data(instruction: &EscrowInstruction) -> Vec<u8> {
    match instruction {
        EscrowInstruction::Make { amount, seed } => {
            let mut data = vec![0u8]; // Make discriminator
            data.extend_from_slice(&amount.to_le_bytes());
            data.extend_from_slice(&seed.to_le_bytes());
            data
        }
        EscrowInstruction::Take { amount, seed } => {
            let mut data = vec![1u8]; // Take discriminator
            data.extend_from_slice(&amount.to_le_bytes());
            data.extend_from_slice(&seed.to_le_bytes());
            data
        }
        EscrowInstruction::Refund { amount, seed } => {
            let mut data = vec![2u8]; // Refund discriminator
            data.extend_from_slice(&amount.to_le_bytes());
            data.extend_from_slice(&seed.to_le_bytes());
            data
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_instruction_packing() {
        // test Make instruction
        let make_instruction = EscrowInstruction::Make { amount: 1000, seed: 12345 };
        let packed = pack_instruction_data(&make_instruction);
        
        let expected = {
            let mut data = vec![0u8]; // discriminator
            data.extend_from_slice(&1000u64.to_le_bytes());
            data.extend_from_slice(&12345u64.to_le_bytes());
            data
        };
        
        assert_eq!(packed, expected);
        
        // test Take instruction
        let take_instruction = EscrowInstruction::Take { amount: 2000, seed: 67890 };
        let packed = pack_instruction_data(&take_instruction);
        let expected = {
            let mut data = vec![1u8]; // discriminator
            data.extend_from_slice(&2000u64.to_le_bytes());
            data.extend_from_slice(&67890u64.to_le_bytes());
            data
        };
        assert_eq!(packed, expected);
        
        // test Refund instruction
        let refund_instruction = EscrowInstruction::Refund { amount: 3000, seed: 112233 };
        let packed = pack_instruction_data(&refund_instruction);
        let expected = {
            let mut data = vec![2u8]; // discriminator
            data.extend_from_slice(&3000u64.to_le_bytes());
            data.extend_from_slice(&112233u64.to_le_bytes());
            data
        };
        assert_eq!(packed, expected);
    }

    #[test]
    fn test_instruction_unpacking() {
        // test Make instruction unpacking
        let data = {
            let mut data = vec![0u8]; // discriminator
            data.extend_from_slice(&1000u64.to_le_bytes());
            data.extend_from_slice(&12345u64.to_le_bytes());
            data
        };
        
        let instruction = EscrowInstruction::unpack(&data).unwrap();
        match instruction {
            EscrowInstruction::Make { amount, seed } => {
                assert_eq!(amount, 1000);
                assert_eq!(seed, 12345);
            }
            _ => panic!("Wrong instruction type"),
        }
        
        // test Take instruction unpacking
        let take_data = {
            let mut data = vec![1u8]; // discriminator
            data.extend_from_slice(&2000u64.to_le_bytes());
            data.extend_from_slice(&67890u64.to_le_bytes());
            data
        };
        let instruction = EscrowInstruction::unpack(&take_data).unwrap();
        match instruction {
            EscrowInstruction::Take { amount, seed } => {
                assert_eq!(amount, 2000);
                assert_eq!(seed, 67890);
            }
            _ => panic!("Wrong instruction type"),
        }
        
        // test Refund instruction unpacking
        let refund_data = {
            let mut data = vec![2u8]; // discriminator
            data.extend_from_slice(&3000u64.to_le_bytes());
            data.extend_from_slice(&112233u64.to_le_bytes());
            data
        };
        let instruction = EscrowInstruction::unpack(&refund_data).unwrap();
        match instruction {
            EscrowInstruction::Refund { amount, seed } => {
                assert_eq!(amount, 3000);
                assert_eq!(seed, 112233);
            }
            _ => panic!("Wrong instruction type"),
        }
        
        // test invalid instruction
        let invalid_data = vec![3u8];
        assert!(EscrowInstruction::unpack(&invalid_data).is_err());
        
        // test empty data
        let empty_data = vec![];
        assert!(EscrowInstruction::unpack(&empty_data).is_err());
        
        // test insufficient data for Make instruction
        let insufficient_data = vec![0u8, 1u8]; // Only discriminator + 1 byte
        assert!(EscrowInstruction::unpack(&insufficient_data).is_err());
    }

    #[test]
    fn test_escrow_constants() {
        // test that our constants are properly defined
        assert_eq!(ID.len(), 32);
        assert_eq!(TOKEN_PROGRAM_ID.len(), 32);
        
        // test escrow size is reasonable
        assert!(Escrow::LEN > 0);
        assert!(Escrow::LEN < 1000); // Reasonable size limit
    }

    #[test]
    fn test_escrow_discriminator() {
        // test that discriminator is properly set
        assert_eq!(Escrow::DISCRIMINATOR.len(), 8);
        
        // test discriminator is not all zeros (which would be invalid)
        assert_ne!(Escrow::DISCRIMINATOR, [0u8; 8]);
    }

    #[test]
    fn test_error_conversion() {
        // test that custom errors properly convert to ProgramError
        let escrow_error = EscrowError::InvalidInstruction;
        let program_error: ProgramError = escrow_error.into();
        
        // should be a custom error
        matches!(program_error, ProgramError::Custom(_));
    }

    #[test]
    fn test_instruction_round_trip() {
        // test that pack/unpack is symmetric
        let original = EscrowInstruction::Make { amount: 999, seed: 777 };
        let packed = pack_instruction_data(&original);
        let unpacked = EscrowInstruction::unpack(&packed).unwrap();
        
        match (original, unpacked) {
            (EscrowInstruction::Make { amount: a1, seed: s1 }, 
             EscrowInstruction::Make { amount: a2, seed: s2 }) => {
                assert_eq!(a1, a2);
                assert_eq!(s1, s2);
            }
            _ => panic!("Round trip failed"),
        }
    }

    #[test] 
    fn test_boundary_values() {
        // test with maximum values
        let max_instruction = EscrowInstruction::Make { 
            amount: u64::MAX, 
            seed: u64::MAX 
        };
        let packed = pack_instruction_data(&max_instruction);
        let unpacked = EscrowInstruction::unpack(&packed).unwrap();
        
        match unpacked {
            EscrowInstruction::Make { amount, seed } => {
                assert_eq!(amount, u64::MAX);
                assert_eq!(seed, u64::MAX);
            }
            _ => panic!("Failed to handle max values"),
        }
        
        // test with zero values
        let zero_instruction = EscrowInstruction::Make { amount: 0, seed: 0 };
        let packed = pack_instruction_data(&zero_instruction);
        let unpacked = EscrowInstruction::unpack(&packed).unwrap();
        
        match unpacked {
            EscrowInstruction::Make { amount, seed } => {
                assert_eq!(amount, 0);
                assert_eq!(seed, 0);
            }
            _ => panic!("Failed to handle zero values"),
        }
    }
}

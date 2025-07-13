
use pinocchio::{
    account_info::AccountInfo,
    entrypoint,
    entrypoint::ProgramResult,
    msg,
    pubkey::Pubkey,
};
use pinocchio_escrow_lib::{
    instructions::{
        make::{make, MakeAccounts},
        take::{take, TakeAccounts},
        refund::{refund, RefundAccounts},
    },
    EscrowInstruction,
};

entrypoint!(process_instruction);

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    msg!("Example program using Pinocchio Escrow Library");
    
    // parse instruction
    let instruction = EscrowInstruction::unpack(instruction_data)?;
    
    //process based on instruction type
    match instruction {
        EscrowInstruction::Make { amount, seed } => {
            msg!("Creating escrow with amount: {} and seed: {}", amount, seed);
            
            // accounts for make handler
            let make_accounts = MakeAccounts {
                maker: &accounts[0],
                mint_a: &accounts[1],
                mint_b: &accounts[2],
                maker_ata_a: &accounts[3],
                escrow: &accounts[4],
                vault: &accounts[5],
                token_program: &accounts[6],
                system_program: &accounts[7],
            };
            
            // library make handler
            make(program_id, make_accounts, amount, seed)?;
            
            msg!("Escrow created successfully!");
        }
        
        EscrowInstruction::Take { amount, seed } => {
            msg!("Taking escrow offer with amount: {} and seed: {}", amount, seed);
            
            //accounts for take handler
            let take_accounts = TakeAccounts {
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
            
            // library take handler
            take(program_id, take_accounts, amount, seed)?;
            
            msg!("Escrow completed successfully!");
        }
        
        EscrowInstruction::Refund { amount, seed } => {
            msg!("Refunding escrow with amount: {} and seed: {}", amount, seed);
            
            // accounts for refund handler
            let refund_accounts = RefundAccounts {
                maker: &accounts[0],
                escrow: &accounts[1],
                vault: &accounts[2],
                maker_ata_a: &accounts[3],
                token_program: &accounts[4],
            };
            
            // library refund handler
            refund(program_id, refund_accounts, amount, seed)?;
            
            msg!("Escrow refunded successfully!");
        }
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_instruction_parsing() {
        // test make instruction
        let make_data = {
            let mut data = vec![0u8]; // Make discriminator
            data.extend_from_slice(&100u64.to_le_bytes()); // amount
            data.extend_from_slice(&1u64.to_le_bytes()); // seed
            data
        };
        let instruction = EscrowInstruction::unpack(&make_data).unwrap();
        match instruction {
            EscrowInstruction::Make { amount, seed } => {
                assert_eq!(amount, 100);
                assert_eq!(seed, 1);
            }
            _ => panic!("Wrong instruction type"),
        }
        
        // test take instruction
        let take_data = {
            let mut data = vec![1u8]; // Take discriminator
            data.extend_from_slice(&200u64.to_le_bytes()); // amount
            data.extend_from_slice(&2u64.to_le_bytes()); // seed
            data
        };
        let instruction = EscrowInstruction::unpack(&take_data).unwrap();
        match instruction {
            EscrowInstruction::Take { amount, seed } => {
                assert_eq!(amount, 200);
                assert_eq!(seed, 2);
            }
            _ => panic!("Wrong instruction type"),
        }
        
        // test refund instruction
        let refund_data = {
            let mut data = vec![2u8]; // Refund discriminator
            data.extend_from_slice(&300u64.to_le_bytes()); // amount
            data.extend_from_slice(&3u64.to_le_bytes()); // seed
            data
        };
        let instruction = EscrowInstruction::unpack(&refund_data).unwrap();
        match instruction {
            EscrowInstruction::Refund { amount, seed } => {
                assert_eq!(amount, 300);
                assert_eq!(seed, 3);
            }
            _ => panic!("Wrong instruction type"),
        }
    }
} 
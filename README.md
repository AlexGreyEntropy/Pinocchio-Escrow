<<<<<<< HEAD
# Pinocchio-Escrow
Pinocchio Escrow example
=======
# Pinocchio Escrow Library

## Features

- **Secure way forToken swaps**: Exchange SPL tokens safely through escrow accounts
- **3 main operations**:
  - `Make`: Create an escrow by depositing tokens
  - `Take`: Complete an escrow by swapping tokens
  - `Refund`: Cancel an escrow and return tokens to maker
- **PDA based security**: Uses Program Derived Addresses for secure vault management
- **Less compute units needed**: Built with Pinocchio for optimal performance

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
pinocchio_escrow_lib = { path = "../pinocchio_escrow_lib" }
```

## Usage

### As a Library

You can use this library in your own Solana programs:

```rust
use pinocchio_escrow_lib::{
    instructions::{
        make::{make, MakeAccounts},
        take::{take, TakeAccounts},
        refund::{refund, RefundAccounts},
    },
    Escrow, EscrowError,
};

// creating an escrow
pub fn create_escrow(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    amount: u64,
    seed: u64,
) -> ProgramResult {
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
    
    make(program_id, make_accounts, amount, seed)
}
```

### This is a standalone Program

Build and deploy the program:

```bash
# Build the program
cargo build-sbf

# Deploy to devnet (don't forget to replace with your keypair path)
solana program deploy target/deploy/pinocchio_escrow_lib.so
```

## Instruction Format

### Make Instruction (0)
Creates a new escrow.

**Data Layout:**
- `[0]` - Instruction discriminator (0)
- `[1..9]` - Amount (u64, little-endian)
- `[9..17]` - Seed (u64, little-endian)

**Accounts:**
1. `[signer]` Maker
2. `[]` Mint A (token being offered)
3. `[]` Mint B (token being requested)
4. `[writable]` Maker ATA A
5. `[writable]` Escrow account (PDA)
6. `[writable]` Vault account (PDA)
7. `[]` Token program
8. `[]` System program

### Take Instruction (1)
Completes an escrow by swapping tokens.

**Data Layout:**
- `[0]` - Instruction discriminator (1)
- `[1..9]` - Amount (u64, little-endian)
- `[9..17]` - Seed (u64, little-endian)

**Accounts:**
1. `[signer]` Taker
2. `[]` Maker
3. `[writable]` Escrow account
4. `[writable]` Vault account
5. `[]` Mint A
6. `[]` Mint B
7. `[writable]` Taker ATA A (to receive)
8. `[writable]` Taker ATA B (to send)
9. `[writable]` Maker ATA B (to receive)
10. `[]` Token program

### Refund Instruction (2)
Cancels an escrow and returns tokens to maker.

**Data Layout:**
- `[0]` - Instruction discriminator (2)
- `[1..9]` - Amount (u64, little-endian)
- `[9..17]` - Seed (u64, little-endian)

**Accounts:**
1. `[signer]` Maker
2. `[writable]` Escrow account
3. `[writable]` Vault account
4. `[writable]` Maker ATA A
5. `[]` Token program

## PDAs

The program uses two types of PDAs

1. **Escrow PDA**: `["escrow", maker_pubkey, seed_bytes]`
2. **Vault PDA**: `["vault", escrow_pubkey]`

## State

### Escrow Account Structure
```rust
pub struct Escrow {
    pub discriminator: [u8; 8],    // account type identifier
    pub maker: Pubkey,             // creator of the escrow
    pub mint_a: Pubkey,            // token being offered
    pub mint_b: Pubkey,            // token being requested
    pub receive_account: Pubkey,   // maker ATA B to receive token B
    pub amount: u64,               // amount of token A in escrow
    pub bump: u8,                  // PDA bump seed
}
```

estimated total size: 145 bytes

## Error Codes

`InvalidInstruction` for Invalid instruction data
`NotRentExempt` for Account is not rent exempt
`ExpectedAmountMismatch` for Token amounts don't match
`AmountOverflow` for Arithmetic overflow
`InvalidState` for Invalid account state
`InvalidAuthority` for Unauthorized operation
`InvalidTokenProgram` for Wrong token program
`InvalidTokenMint` for Wrong token mint
`InvalidEscrowAccount` for Invalid escrow account

## Examples

### Client side use (JavaScript/TypeScript)

```typescript
import { PublicKey, Transaction, TransactionInstruction } from '@solana/web3.js';

// create Make instruction
function createMakeInstruction(
  programId: PublicKey,
  maker: PublicKey,
  mintA: PublicKey,
  mintB: PublicKey,
  makerAtaA: PublicKey,
  amount: bigint,
  seed: bigint
): TransactionInstruction {
  const escrow = PublicKey.findProgramAddressSync(
    [Buffer.from('escrow'), maker.toBuffer(), Buffer.from(seed.toString())],
    programId
  )[0];
  
  const vault = PublicKey.findProgramAddressSync(
    [Buffer.from('vault'), escrow.toBuffer()],
    programId
  )[0];
  
  const data = Buffer.concat([
    Buffer.from([0]), // Make instruction
    Buffer.from(amount.toString(), 'hex').reverse(),
    Buffer.from(seed.toString(), 'hex').reverse(),
  ]);
  
  return new TransactionInstruction({
    keys: [
      { pubkey: maker, isSigner: true, isWritable: true },
      { pubkey: mintA, isSigner: false, isWritable: false },
      { pubkey: mintB, isSigner: false, isWritable: false },
      { pubkey: makerAtaA, isSigner: false, isWritable: true },
      { pubkey: escrow, isSigner: false, isWritable: true },
      { pubkey: vault, isSigner: false, isWritable: true },
      { pubkey: TOKEN_PROGRAM_ID, isSigner: false, isWritable: false },
      { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
    ],
    programId,
    data,
  });
}
```

## Testing

Run tests with

```bash
cargo test
```

integration tests with a local validator

```bash
cargo test-sbf
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## Acknowledgments

Built with [Pinocchio](https://github.com/anza-xyz/pinocchio)
Follow [blueshift.gg](https://blueshift.gg), [febo](https://github.com/febo), [turbin3](https://github.com/turbin3)

give a star if you like it!
>>>>>>> master

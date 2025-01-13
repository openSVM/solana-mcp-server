# Transactions

A transaction contains a list of instructions that are processed atomically. If any of the instructions fail, the entire transaction fails.

## Transaction Format
A Solana transaction consists of:
1. Signatures: An array of signatures included on the transaction
2. Message: Contains the following components:
   - Header: Metadata about the transaction including number of signers and read-only accounts
   - Account addresses: List of accounts referenced by instructions
   - Recent blockhash: Used to prevent duplicate transactions and ensure transaction freshness
   - Instructions: List of instructions to execute

## Transaction Size
The maximum size of a transaction is 1232 bytes, which includes:
- Signatures (64 bytes each)
- Message (accounts, instructions, and metadata)
This limit ensures fast and reliable transmission over the network.

## Instructions
Each instruction in a transaction specifies:
- Program ID: The program that will execute the instruction
- Accounts: List of accounts the instruction will read or write
- Instruction Data: Opaque byte array containing instruction-specific data

## Best Practices
- Group related operations into a single transaction
- Use recent blockhashes (valid for 150 blocks or ~1 minute)
- Handle transaction failures gracefully
- Monitor transaction costs and size limits
- Consider account privileges (signer/writable) carefully
- Test transactions thoroughly before mainnet deployment

## Transaction Lifecycle
1. Client creates transaction
2. Transaction is signed by required signers
3. Transaction is sent to cluster
4. Transaction is processed by leader
5. Transaction is confirmed by validators
6. Client receives confirmation

## Error Handling
- Check transaction status after submission
- Implement proper retry logic for failed transactions
- Handle common error cases:
  - Invalid blockhash
  - Insufficient funds
  - Account not found
  - Program errors

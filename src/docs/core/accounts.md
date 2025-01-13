# Accounts

Accounts are a fundamental building block of the Solana programming model. They serve as the primary data structure for storing state on the Solana blockchain.

## Account Model
- Each account has a unique address (public key)
- Accounts can store data and SOL (lamports)
- Only the owner program can modify account data
- Account data is stored in a serialized format
- Accounts pay rent to maintain their data on-chain

## Account Types
1. System Accounts (User Wallets)
   - Owned by the System Program
   - Store SOL balances
   - Can be transaction signers
   - Used for basic transfers and account creation

2. Program Accounts
   - Store executable program code
   - Immutable after deployment
   - Cannot be transaction signers
   - Loaded into memory when referenced

3. Program Derived Addresses (PDAs)
   - Accounts owned by programs
   - Addresses derived deterministically
   - Cannot sign transactions directly
   - Used for program-controlled state
   - Enable cross-program invocation

## Account Storage
- Data is stored as a serialized byte array
- Maximum account size is 10 megabytes
- Account size is fixed at creation
- Rent is charged based on size and time
- Rent-exempt accounts maintain minimum balance

## Account Ownership
- System Program owns new accounts
- Programs can own accounts
- Only owner can modify data
- Only owner can assign new owner
- Anyone can read account data

## Best Practices
- Minimize account size to reduce rent costs
- Properly validate account ownership and permissions
- Use PDAs for program-controlled state
- Consider account data serialization format
- Handle account initialization properly
- Implement proper access controls

## Account Lifecycle
1. Account creation
2. Data initialization
3. State updates
4. Ownership transfers (optional)
5. Account closure (optional)

## Security Considerations
- Validate account ownership
- Check account permissions
- Verify signer privileges
- Protect against reentrancy
- Handle account initialization safely
- Implement proper access controls

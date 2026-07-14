# 👻 StellarGhost Contract

**Soroban Smart Contract for Decentralized Estate Planning**

The trustless escrow engine powering StellarGhost. This Rust-based Soroban contract manages asset lockboxes, beneficiary configurations, and inheritance execution with zero intermediaries.

## Overview

The StellarGhost contract implements a "Dead Man's Switch" for digital assets on Stellar. When deployed, it allows users to:

- Lock digital assets in an escrow contract
- Configure multiple beneficiaries with allocation percentages
- Set inactivity thresholds (e.g., 180 days without a "ping")
- Automatically release assets to beneficiaries after threshold expiration
- Integrate with path payments for automatic stablecoin conversion

## Features

- **Trustless Escrow** - No intermediaries; logic is immutable on-chain
- **Multiple Beneficiaries** - Support for unlimited beneficiaries with flexible allocation percentages
- **Encrypted Payloads** - Store IPFS CIDs for encrypted files (wills, letters, credentials)
- **Path Payment Integration** - Automatic asset conversion to beneficiary-preferred stablecoins
- **Time-Locked Release** - Threshold-based logic prevents premature or missed releases
- **Atomic State Transitions** - All-or-nothing contract state changes ensure consistency
- **Event Emission** - Full audit trail via contract events

## Core Functions

### Owner Functions

**`ping(owner: Address) -> Result<(), String>`**
- Updates `last_ping_timestamp` to current ledger time
- Resets the inactivity countdown
- Only callable by contract owner
- Cost: ~0.00001 XLM (network fees only)

**`add_beneficiary(owner: Address, address: Address, percentage: u32) -> Result<(), String>`**
- Adds or updates a beneficiary allocation
- Validates percentage sum doesn't exceed 100%
- Emits `BeneficiaryAdded` event
- Only callable by owner

**`remove_beneficiary(owner: Address, address: Address) -> Result<(), String>`**
- Removes beneficiary from the will
- Redistributes percentage allocation
- Emits `BeneficiaryRemoved` event
- Only callable by owner

**`set_inactivity_threshold(owner: Address, duration: u64) -> Result<(), String>`**
- Updates the inactivity threshold (in seconds)
- Allows owner to adjust timeline without redeployment
- Only callable by owner

**`deposit_assets(owner: Address, amount: i128, asset: String) -> Result<(), String>`**
- Locks assets into the contract
- Accepts native XLM or custom Stellar assets
- Updates total asset balance atomically
- Emits `DepositReceived` event

**`withdraw_assets(owner: Address, amount: i128) -> Result<(), String>`**
- Allows owner to withdraw assets before threshold
- Prevents withdrawal after `trigger_release()` is called
- Emits `WithdrawalInitiated` event

### Public Functions

**`trigger_release() -> Result<(), String>`**
- Publicly callable by anyone (decentralized)
- Checks: `current_time > last_ping_timestamp + inactivity_threshold`
- Sets `is_released = true` and records `release_timestamp`
- Emits `InheritanceTriggered` event
- Triggers automated keeper/relayer operations
- Cannot be reversed once called

**`claim_inheritance(beneficiary: Address) -> Result<(), String>`**
- Beneficiary claims their proportional share
- Validates beneficiary exists and hasn't already claimed
- Calculates share based on percentage allocation
- Initiates path payment to beneficiary's preferred asset
- Marks beneficiary as claimed (prevents double-claiming)
- Emits `InheritanceClaimed` event with payout details

### Query Functions

**`get_contract_state() -> Result<LockboxState, String>`**
- Returns full contract state without modifications
- Used for UI polling, threshold verification, and auditing
- Zero-cost read operation
- Returns:
  - Owner address
  - Last ping timestamp
  - Inactivity threshold
  - Total assets held
  - Release status and timestamp
  - All beneficiaries and claim status

**`get_days_until_trigger() -> Result<i64, String>`**
- Calculates remaining days before inheritance triggers
- Returns negative value if already triggered
- Useful for UI countdown timers

**`get_beneficiary_share(beneficiary: Address) -> Result<BeneficiaryInfo, String>`**
- Returns specific beneficiary information
- Includes allocation percentage and claim status
- Returns error if beneficiary doesn't exist

## State Variables

```rust
pub struct Lockbox {
    pub owner: Address,
    pub last_ping_timestamp: u64,
    pub inactivity_threshold: u64,
    pub beneficiaries: Map<Address, BeneficiaryShare>,
    pub encrypted_payload_cid: String,
    pub total_assets: i128,
    pub is_released: bool,
    pub release_timestamp: u64,
}

pub struct BeneficiaryShare {
    pub address: Address,
    pub percentage: u32,
    pub claimed: bool,
    pub preferred_asset: String,
}
```

## Events

The contract emits detailed events for off-chain tracking and indexing:

- **`PingEvent`** - Owner pinged; timestamp updated
- **`BeneficiaryAdded`** - New beneficiary configured
- **`BeneficiaryRemoved`** - Beneficiary removed from will
- **`DepositReceived`** - Assets deposited into contract
- **`WithdrawalInitiated`** - Owner withdrew assets pre-threshold
- **`InheritanceTriggered`** - Threshold exceeded; release initiated
- **`InheritanceClaimed`** - Beneficiary claimed their share
- **`ThresholdUpdated`** - Inactivity threshold changed

## Security Model

### Threat Mitigation

1. **Ownership Verification** - All owner functions verify caller is contract owner
2. **Timestamp Immutability** - Once `release_timestamp` is set, release status cannot change
3. **Atomic State Transitions** - Contract state updates are all-or-nothing
4. **Reentrancy Prevention** - Soroban SDK handles reentrancy protections
5. **Overflow Protection** - Rust's checked arithmetic prevents integer overflow
6. **Access Control** - Clear separation between owner, beneficiary, and public functions

### Recommended Audits

Before mainnet deployment:
- Security audit by Stellar/Soroban specialists
- Formal verification of time-lock logic
- Fuzzing of threshold calculations
- Review of path payment integration

## Prerequisites

- Rust 1.70+
- Soroban SDK 20.0+
- Stellar CLI tools
- wasm32-unknown-unknown target

## Installation & Setup

1. **Clone the repository:**
   ```bash
   git clone https://github.com/stellar-ghost/stellar-ghost-contract.git
   cd stellar-ghost-contract
   ```

2. **Install dependencies:**
   ```bash
   cargo build
   ```

3. **Run tests:**
   ```bash
   cargo test
   ```

4. **Build WASM binary:**
   ```bash
   cargo build --target wasm32-unknown-unknown --release
   ```

## Deployment

### Testnet Deployment

1. **Build the contract:**
   ```bash
   cargo build --target wasm32-unknown-unknown --release
   ```

2. **Deploy using Stellar CLI:**
   ```bash
   soroban contract deploy \
     --network testnet \
     --source <YOUR_ACCOUNT> \
     --wasm target/wasm32-unknown-unknown/release/stellar_ghost_contract.wasm
   ```

3. **Initialize contract (if needed):**
   ```bash
   soroban contract invoke \
     --network testnet \
     --source <YOUR_ACCOUNT> \
     --id <CONTRACT_ID> \
     -- initialize \
     --owner <OWNER_ADDRESS> \
     --threshold 15552000  # 180 days in seconds
   ```

### Mainnet Deployment

Before deploying to mainnet:
- Complete security audit
- Test extensively on testnet
- Review all state transitions
- Verify path payment integrations
- Have rollback plan in place

```bash
soroban contract deploy \
  --network public \
  --source <YOUR_ACCOUNT> \
  --wasm target/wasm32-unknown-unknown/release/stellar_ghost_contract.wasm
```

## Contract Interface (JavaScript)

The contract can be invoked via Stellar SDK in JavaScript:

```typescript
import { Server, TransactionBuilder, SorobanRpc } from '@stellar/js-stellar-sdk';

const server = new Server('https://horizon-testnet.stellar.org');
const contractId = 'CXXXX...';

// Ping the contract
async function ping(signer: string, ownerAddress: string) {
  const account = await server.loadAccount(ownerAddress);
  
  const tx = new TransactionBuilder(account, {
    fee: '1000',
    networkPassphrase: StellarSDK.Networks.TESTNET_NETWORK_PASSPHRASE
  })
    .addOperation(
      SorobanRpc.invokeContractOp({
        contractId,
        method: 'ping',
        args: [SorobanRpc.Address.fromString(ownerAddress)]
      })
    )
    .build();

  // Sign and submit
  return await signAndSubmit(tx, signer);
}

// Get contract state
async function getContractState() {
  return await server.sorobanRpc().getContractData(contractId, ...);
}
```

## Testing

The contract includes comprehensive tests:

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_ping_updates_timestamp

# Run tests with output
cargo test -- --nocapture
```

### Test Coverage

- State initialization and validation
- Ping timestamp updates
- Beneficiary addition and removal
- Inactivity threshold logic
- Asset deposit and withdrawal
- Inheritance release triggering
- Claim processing
- Edge cases and error conditions

## File Structure

```
stellar-ghost-contract/
├── Cargo.toml
├── src/
│   ├── lib.rs              # Contract entry points & main logic
│   ├── state.rs            # State structures and persistence
│   ├── events.rs           # Event definitions
│   ├── errors.rs           # Custom error types
│   ├── utils.rs            # Helper functions
│   └── tests.rs            # Unit tests
├── target/
│   └── wasm32-unknown-unknown/
│       └── release/
│           └── stellar_ghost_contract.wasm
└── README.md
```

## Contributing

We welcome contributions! Please:

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/improvement`)
3. Write tests for any new functionality
4. Ensure all tests pass (`cargo test`)
5. Commit with clear messages (`git commit -m 'Add feature'`)
6. Push and create a Pull Request

## Development Guidelines

- **Code Style** - Follow Rust conventions (use `rustfmt`)
- **Documentation** - All public functions must have doc comments
- **Testing** - Aim for >90% code coverage
- **Security** - Avoid unsafe code; use checked arithmetic
- **Performance** - Optimize for testnet gas costs

## License

MIT License - see LICENSE file for details.

## Support & Community

- **Issues:** Report bugs and request features on GitHub
- **Discussions:** Participate in GitHub Discussions
- **Discord:** Join the Stellar Developer Community
- **Security:** Report vulnerabilities to security@stellarghost.dev

---

**Part of the 👻 StellarGhost ecosystem**

For more information on the full StellarGhost project, visit the [main monorepo](https://github.com/stellar-ghost/stellar-ghost).

# Security Model

## Authorization

All owner functions require owner authentication via `require_auth()`.

## Validation

- Percentages: 0-100 range enforced
- Thresholds: Minimum 86400 seconds (1 day)
- Balances: Checked before transfers
- Claims: Tracked to prevent double-claiming

## State Integrity

- All state transitions are atomic
- Immutable timestamps prevent replay attacks
- Boolean release flag prevents re-triggering

## Tested Security

- Owner authorization checks
- Beneficiary verification
- Threshold validation
- Double-claim prevention
- Balance verification

## Audit Recommendations

Before mainnet:
1. Formal security audit by Soroban specialists
2. Formal verification of time-lock logic
3. Fuzzing of threshold calculations
4. Testnet stress testing

## Deployment Safety

- Test thoroughly on testnet
- Monitor for first 30 days on mainnet
- Have upgrade mechanism for critical bugs

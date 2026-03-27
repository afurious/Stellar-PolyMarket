## Mini-README: propose_outcome Implementation

### Logic Flow
The `propose_outcome` function implements a trustless buffer for oracle proposals:

1. **Oracle Locks Market**: Market must be in `Locked` state before proposing
2. **Proposal Submission**: Oracle with `ORACLE_ROLE` calls `propose_outcome(market_id, proposed_id)`
3. **Timer Setup**: `unlock_timestamp = current_ledger_time + 86,400 seconds` (24 hours)
4. **State Transition**: Market moves from `Locked` → `Proposed`
5. **Storage**: 
   - `DataKey::ProposedOutcomeId(market_id)` stores the proposed outcome
   - `DataKey::UnlockTimestamp(market_id)` stores settlement unlock time

### Security Features
- **Role Enforcement**: Only addresses with `ORACLE_ROLE` can propose outcomes via `require_role()`
- **State Validation**: Market must be `Locked` before proposing (prevents double proposals)
- **Timer Protection**: Settlement is blocked until 24-hour challenge period expires
- **Unauthorized Access**: Non-oracle calls panic with role validation error

### Test Coverage
The implementation includes comprehensive test scenarios:

✅ **Authorized Proposal**: Oracle successfully proposes outcome and sets timer
✅ **Timer Verification**: 24-hour window correctly calculated and stored
✅ **Settlement Blocking**: Early settlement attempts fail before timer expires  
✅ **Post-Timer Settlement**: Settlement succeeds after 24-hour window
✅ **State Transitions**: Active → Locked → Proposed → Resolved
✅ **Storage Validation**: Proposed outcome and timestamp correctly persisted

### Acceptance Criteria Met
- [x] Function strictly enforces ORACLE_ROLE via require_role()
- [x] Payout/Settlement programmatically blocked during liveness window
- [x] 24-hour challenge timer implemented with persistent storage
- [x] Market state transitions work correctly
- [x] Comprehensive test coverage for all scenarios

The implementation provides a secure, trustless buffer allowing community verification of oracle data before any fund withdrawals occur.

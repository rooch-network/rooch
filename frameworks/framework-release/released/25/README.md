# Rooch Move Framework v25

This release introduces significant enhancements to the payment channel infrastructure, resource management, and event handling systems.

## Major Features

### [rooch-framework] Payment Channel Enhancements
- **Enhanced Payment Channel Flexibility (#3836)**: Support payment hub transfer operations within payment channels
- **Same Address Channel Support**: Allow payment channel sender and receiver to use the same address for account internal operations
- **X402 Payment Channel Protocol (#3758)**: Implemented X402 payment channel protocol with improved channel reopen functionality
- **Locked Unit Reserve System (#3791)**: Added locked unit reserve mechanism for payment hub withdrawals with monotonic tracking
- **Per-Hub Revenue Events (#3780)**: Emit payment revenue events per hub using custom event handles for better event indexing

### [rooch-framework] Resource Management & Limits
- **DID Resource Limits (#3796)**: Added comprehensive resource limits for DID operations
  - Maximum 32 DID services per document
  - Maximum 16 properties per service
  - 128-byte fragment length limit and 512-byte string length limit
  - Validation for property keys and values against string limits
- **Payment Channel Limits (#3796)**: Added proof count limits (64 max) for close/cancel operations

### [rooch-framework] Event System Improvements
- **Custom Event Handles (#3793)**: Refactored DID and payment channel events to use custom event handles
  - DID modification events now use `did_event_handle_id` for efficient retrieval
  - Payment hub events use `hub_event_handle_id`
  - Payment channel events use `channel_event_handle_id`
  - Improved event indexing performance for mainnet deployment

## Technical Improvements
- Deprecated WebAuthn validator in favor of modern authentication methods (#3825)
- Enhanced payment hub configuration with update events
- Improved payment channel internal validation and error handling
- Refactored authentication validator architecture for better maintainability
- Fixed numerous spelling issues in code comments (#3750)

## Testing
- Added comprehensive resource limit tests for DID operations
- Enhanced payment channel testing with transfer operations and same-address scenarios
- New DID validator test coverage improvements (#3754)
- Added payment revenue event testing with custom handles
- Comprehensive test coverage for locked unit reserve mechanisms

This release significantly improves Rooch's payment infrastructure reliability and resource management while maintaining full backward compatibility and preparing for mainnet deployment.

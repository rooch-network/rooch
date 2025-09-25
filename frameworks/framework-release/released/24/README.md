# Rooch Move Framework v24

This release introduces significant enhancements to the DID (Decentralized Identity) system, payment infrastructure, and authentication mechanisms.

## Major Features

### [rooch-framework] DID System Enhancements
- **DID Bitcoin Controller Support (#3717)**: Added support for `did:bitcoin` as controller, enabling Bitcoin addresses to control DID documents
- **DID Validator Implementation (#3701)**: Introduced comprehensive DID authentication validator with session key compatibility
  - New DID auth payload structure using BCS encoding
  - WebAuthn envelope support for secure authentication
  - Enhanced session signing envelope mechanism
  - Refactored authentication validator architecture

### [rooch-framework] Payment Infrastructure
- **Payment Revenue System (#3714)**: Implemented payment revenue distribution mechanism
  - Auto-detection of DID addresses and wallet addresses
  - Revenue sharing and distribution capabilities
- **Gas Payment Hub (#3713)**: Enhanced gas payment system with payment hub integration
  - Pay gas fees from payment hub
  - Improved transaction gas handling
  - Updated transaction validator for gas payment flows

## Documentation Updates
- Enhanced DID system documentation with new validator patterns
- Updated authentication validator guides
- Improved payment channel and revenue documentation
- Added comprehensive transaction gas and validator documentation

## Technical Improvements
- Refactored authentication validator error codes for better debugging
- Enhanced session key management and compatibility
- Improved Bitcoin message handling and envelope processing
- Optimized DID virtual machine and session key interactions

## Testing
- Added comprehensive test suites for DID controllers and limits
- Enhanced payment channel testing
- New DID validator test coverage
- Improved authentication validator test cases

This release significantly strengthens Rooch's identity and payment infrastructure, providing more robust and flexible authentication mechanisms while maintaining backward compatibility.

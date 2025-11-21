# Allow Custom Session Key Scope Configuration for DID Creation

## Issue Description

Currently, the DID module uses hardcoded wildcard scope (`*::*::*`) when creating session keys for authentication verification methods. This approach presents security risks and lacks flexibility for different application scenarios.

## Problem Statement

1. **Security Risk**: Wildcard permissions are overly broad, violating the principle of least privilege
2. **Lack of Flexibility**: All DIDs get the same session key permissions regardless of their intended use case
3. **No Customization**: Users cannot specify appropriate permission scopes based on their application requirements

## Current Behavior

When a verification method is added to the `authentication` relationship in a DID document, it automatically creates a session key with these hardcoded scopes:
- `@rooch_framework::*::*` (all rooch framework functions)
- `{did_address}::*::*` (all functions for the DID's account)

## Desired Behavior

Users should be able to:
1. **Specify custom scopes** when creating DIDs using string array format (`"address::module::function"`)
2. **Choose from predefined templates** for common use cases (minimal, development, etc.)
3. **Maintain backward compatibility** with existing DID creation functions

## Use Cases

### Minimal Privilege DID
For security-conscious applications that only need basic DID operations:
```
0x3::did::*
0x3::payment_channel::*
```

### Application-Specific DID
For DeFi applications that need DID management plus specific contract access:
```
0x3::did::*
0x123::defi_app::swap
0x123::defi_app::add_liquidity
```

### Development DID
For development and testing with full permissions (current behavior):
```
0x3::*::*
{address}::*::*
```

## Proposed Solution Overview

1. **Extend DID creation functions** to accept optional custom scope configurations
2. **Leverage existing session_key infrastructure** that already supports scope string parsing
3. **Provide predefined templates** for common scenarios
4. **Maintain backward compatibility** by keeping existing functions unchanged

## Implementation Requirements

- [ ] Modify `internal_ensure_session_key` to accept custom scope parameters
- [ ] Add scope string parsing helper functions
- [ ] Create new entry functions for custom scope configuration
- [ ] Add predefined scope templates for common use cases
- [ ] Update CADOP creation functions to support custom scopes
- [ ] Write comprehensive tests for all scope configurations
- [ ] Update documentation and provide usage examples

## Acceptance Criteria

- [ ] Users can create DIDs with custom session key scopes via new entry functions
- [ ] Predefined scope templates are available for common use cases
- [ ] Existing DID creation functions continue to work unchanged
- [ ] All scope configurations are properly validated
- [ ] Session keys respect the configured scopes during transaction execution
- [ ] Documentation includes security best practices for scope configuration

## Priority

**Medium** - This is a security and usability improvement that doesn't break existing functionality but provides important capabilities for production use.

## Labels

- `enhancement`
- `security`
- `did`
- `session-key`
- `good-first-issue` (for some subtasks)

## Related Documentation

- Design document: `docs/design/did-session-key-scope-customization.md`
- Session key module: `frameworks/rooch-framework/sources/session_key.move`
- DID module: `frameworks/rooch-framework/sources/did.move`

# Rooch Move Framework v8

** Note: This version contains breaking changes and is not compatible with the previous version.**

## Major changes

1. Bitcoin-related improvements:
   - Implemented Bitcoin consensus encode/decode
   - Added support for TempStateDropEvent for UTXO and Inscription
   - Refactored ordinals inscription updater
   - Fixed inscription bugs and skipped op return
   - Fixed Taproot leaf node serialize bug.

2. Framework enhancements:
   - Introduced oracle functionality and admin capabilities
   - Renamed Rooch GasCoin symbol to RGas and Set gas coin icon
   - Added onchain_config::ConfigUpdateCap
   - Refactored package publishing, Implemented new framework upgrade function and `UpgradeCap`
   - Cleaned up deprecated functions and TODOs
   - Cleaned up transaction sequence info compatible code
   - SignData use bitcoin consensus encode to serialize.
   - Added support for TypeTag and StructTag to canonical string with prefix
   - Init rooch dao multisign account when genesis init.

3. Miscellaneous updates:
   - Migrated wasm library from moveos_std to rooch_nursery
   - Refactored sort function in moveos_std
   - Added unpack_transfer_utxo_event function
# Integration Test Steps:
#
# 1. Initial Setup and Package Deployment
#    1.1. Publish a new coin package (coin admin receives initial caps struct)
#    1.2. Publish minter_manager package (in minter_manager directory)
#    1.3. Publish free_tunnel package
#
# 2. Setup Treasury Cap Manager
#    Coin admin calls minter_manager::setupTreasuryCapManager to obtain 
#    a TREASURY_CAP_MANAGER object ID
#
# 3. Minter Cap Operations
#    3.1. Issue minter cap: minter manager admin calls issueMinterCap
#    3.2. Test mint functionality
#    3.3. Test burn functionality
#    3.4. Revoke minter cap
#
# 4. Add Token
#    Call addToken on free_tunnel contract to register the token
#
# 5. Transfer Minter Cap
#    The address holding MINTER_CAP calls transferMinterCap to transfer
#    the MINTER_CAP to free_tunnel contract
#
# 6. Remove Token
#    Call removeToken on free_tunnel contract to unregister the token
#
# 7. Cleanup
#    Call destroyTreasuryCapManager to clean up resources
#


# 0. Import your addresses here, make sure you have the private key of the addresses
export MINTER_MANAGER="0x0af854fcad035f4134636ff2d7fa22591f8ff2f264f354ac04e53da06e318529"          # address #1
export COIN_ADMIN="0x3cf04c5602fbd9a8cb410174c3e46cf2c60d100431848d8f25375eef4f413480"              # address #2
export MINTER_CAP_HOLDER="0xef9201e82a49895312e4621ce73862700bdc8cc18b469906db712432808a6ae9"       # address #3
export FREE_TUNNEL="0xef9201e82a49895312e4621ce73862700bdc8cc18b469906db712432808a6ae9"             # address #3


# 1. Initial Setup and Package Deployment

# 1.1 Publish a new coin package (skipped, here we use bbusd)
export COIN_INFO="0xb3f1e4561f6ffc0fbb846cccf34e4f3123aa0235a1eeed755fc2ec899615fabb"
export COIN="$COIN_ADMIN::bbusd::BBUSD"

# 1.2 Publish minter_manager package (see minter_manager/test/deploy.sh)

# 1.3 Publish free_tunnel package
cd free_tunnel
rooch account switch --address $FREE_TUNNEL
rooch move publish --named-addresses minter_manager="$MINTER_MANAGER",free_tunnel_rooch="$FREE_TUNNEL"


# 2. Setup Treasury Cap Manager
rooch account switch --address $COIN_ADMIN
rooch move run --function $MINTER_MANAGER::minter_manager::setupTreasuryCapManager \
    --args object_id:$COIN_INFO --type-args $COIN

# return object id of TreasuryCapManager
export TREASURY_CAP_MANAGER="0xd634c0255d5043a9061e213f75b66f81947ac9530419ff0691480154ba8ba6d0"


# 3.  Minter Cap Operations

# 3.1. Issue minter cap
rooch account switch --address $COIN_ADMIN
rooch move run --function $MINTER_MANAGER::minter_manager::issueMinterCap \
    --args object_id:$TREASURY_CAP_MANAGER --args address:$MINTER_CAP_HOLDER --type-args $COIN

# return object id of MinterCap
export MINTER_CAP="0xa7ec7b30cd2ec25333f2c2fb8e12a268139acd4f547fdbb43537794db74492a4"

# 3.2. Test mint functionality
export RECIPIENT="0xb072a8901831f11fb096aa53bbcebc9d5bf7d503d1ac52c911db7a4bcf3c51e2"
rooch account switch --address $MINTER_CAP_HOLDER
rooch move run --function $MINTER_MANAGER::minter_manager::mint \
    --args object_id:$TREASURY_CAP_MANAGER --args object_id:$MINTER_CAP \
    --args 5000u256 --args address:$RECIPIENT \
    --type-args $COIN
rooch account balance --address $RECIPIENT

# 3.3. Test burn functionality
rooch move run --function $MINTER_MANAGER::minter_manager::burnFromSigner \
    --args object_id:$TREASURY_CAP_MANAGER --args object_id:$MINTER_CAP \
    --args 2000u256 --type-args $COIN
rooch account balance --address $MINTER_CAP_HOLDER

# 3.4. Revoke minter cap
rooch account switch --address $COIN_ADMIN
rooch move run --function $MINTER_MANAGER::minter_manager::revokeMinterCap \
    --args object_id:$TREASURY_CAP_MANAGER --args object_id:$MINTER_CAP \
    --type-args $COIN


# 4. Add Token
rooch account switch --address $FREE_TUNNEL     # Admin address of free_tunnel
rooch move run --function $FREE_TUNNEL::atomic_mint::addToken \
    --args 36u8 --args 8u8 --type-args $COIN        # 36u8 is the token index, 8u8 is the decimals


# 5. Transfer Minter Cap
rooch account switch --address $COIN_ADMIN
rooch move run --function $MINTER_MANAGER::minter_manager::issueMinterCap \
    --args object_id:$TREASURY_CAP_MANAGER --args address:$MINTER_CAP_HOLDER --type-args $COIN

# return object id of MinterCap
export MINTER_CAP="0x8b9a8db1ddf9413bc4bea733b8bea324bb43848ae74e449dc9a0066bf0283708"

rooch account switch --address $MINTER_CAP_HOLDER
rooch move run --function $FREE_TUNNEL::atomic_mint::transferMinterCap \
    --args 36u8 --args object_id:$MINTER_CAP --type-args $COIN


# 6. Remove Token
rooch account switch --address $FREE_TUNNEL
rooch move run --function $FREE_TUNNEL::atomic_mint::removeToken --args 36u8 --type-args $COIN


# 7. destroyTreasuryCapManager
rooch account switch --address $COIN_ADMIN
rooch move run --function $MINTER_MANAGER::minter_manager::destroyTreasuryCapManager \
    --args object_id:$TREASURY_CAP_MANAGER --type-args $COIN

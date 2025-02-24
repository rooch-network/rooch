# Export your address here
export MINTER_MANAGER="0x0af854fcad035f4134636ff2d7fa22591f8ff2f264f354ac04e53da06e318529"          # address #1

# Deploy minter_manager package
cd minter_manager
rooch move publish --named-addresses minter_manager="$MINTER_MANAGER"
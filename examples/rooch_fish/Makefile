# Define variables
PACKAGE_NAME = rooch_fish
GAS_BUDGET = 1000
PACKAGE_PATH = .
BUILD_DIR = $(PACKAGE_PATH)/build
SOURCES_DIR = $(PACKAGE_PATH)/sources
TESTS_DIR = $(PACKAGE_PATH)/tests

# Default target
all: build

# Build contract
build:
	@echo "Building the Move package..."
	rooch move build --path $(PACKAGE_PATH) --named-addresses rooch_fish=default --json

# Publish contract
publish:
	@echo "Publishing the Move package..."
	rooch move publish --path $(PACKAGE_PATH) --named-addresses rooch_fish=default

# Initialize game world
init-world:
	@echo "Init RoochFish world..."
	rooch move run --function  0xb38a327121ab8e9091a04377ec1e9af9ab4b801dbfb368f20fb0c080c763f7e8::rooch_fish::init_world --json

# View game world
view-world:
	rooch state --access-path /object/0xb38a327121ab8e9091a04377ec1e9af9ab4b801dbfb368f20fb0c080c763f7e8::rooch_fish::GameState

# View game world
view-ponds:
	rooch rpc request --method rooch_listFieldStates --params '["0xae308aa50bded0b341120431a9049ad1f151b345115a17bcefcad943505fa3b6", null, "8", {"decode": true, "showDisplay": true}]' --json

# Purchase fish
purchase_fish:
	rooch move run --function  0xb38a327121ab8e9091a04377ec1e9af9ab4b801dbfb368f20fb0c080c763f7e8::rooch_fish::purchase_fish --args object_id:0x5e89df84a672ea3697916f3a2a2ada4c63586db573b2e8af666da7d2b1084fd6 --args u64:0 --json

# View purchased fish IDs
view_fish_ids:
	rooch move view --function 0xb38a327121ab8e9091a04377ec1e9af9ab4b801dbfb368f20fb0c080c763f7e8::rooch_fish::get_pond_player_fish_ids --args object_id:0x5e89df84a672ea3697916f3a2a2ada4c63586db573b2e8af666da7d2b1084fd6 --args 0u64 --args 'address:default'

# Move fish
move_fish:
	rooch move run --function  0xb38a327121ab8e9091a04377ec1e9af9ab4b801dbfb368f20fb0c080c763f7e8::rooch_fish::move_fish --args object_id:0x5e89df84a672ea3697916f3a2a2ada4c63586db573b2e8af666da7d2b1084fd6 --args u64:0 --args u64:5 --args u8:0 --json --gas-profile

# Debug contract
debug:
	@echo "Running tests..."
	rooch move test --path $(PACKAGE_PATH) --skip-fetch-latest-git-deps --ignore_compile_warnings --named-addresses rooch_fish=default pond

# Test contract
test:
	@echo "Running tests..."
	rooch move test --path $(PACKAGE_PATH) --skip-fetch-latest-git-deps --ignore_compile_warnings --named-addresses rooch_fish=default

# Clean build files
clean:
	@echo "Cleaning build directory..."
	rm -rf $(BUILD_DIR)

# Help information
help:
	@echo "Makefile for Sui Move project"
	@echo ""
	@echo "Usage:"
	@echo "  make build	 - Build the Move package"
	@echo "  make publish   - Publish the Move package"
	@echo "  make test	  - Run tests"
	@echo "  make clean	 - Clean build directory"
	@echo "  make help	  - Show this help message"

.PHONY: all build publish test clean help

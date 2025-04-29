#!/bin/bash
# Copyright (c) RoochNetwork
# SPDX-License-Identifier: Apache-2.0

# Check if the input file is provided
if [ "$#" -lt 1 ] || [ "$#" -gt 2 ]; then
    echo "Usage: $0 <path_to_coin_stores_file> [parallel_jobs]"
    echo "Example: $0 ./scripts/coin_stores_testnet.txt 10"
    echo "         (default parallel_jobs is 5 if not specified)"
    exit 1
fi

INPUT_FILE="$1"
PARALLEL_JOBS=${2:-2}  # Default to 2 parallel jobs if not specified

# Check if the file exists
if [ ! -f "$INPUT_FILE" ]; then
    echo "Error: File $INPUT_FILE does not exist."
    echo "Please provide the full path to the coin stores file."
    exit 1
fi

# Check if GNU Parallel is installed
if ! command -v parallel &> /dev/null; then
    echo "Error: GNU Parallel is not installed. Please install it first."
    echo "For macOS: brew install parallel"
    echo "For Ubuntu/Debian: apt-get install parallel"
    exit 1
fi

# Create a temporary file for commands
TEMP_COMMANDS_FILE=$(mktemp)

# Counter for tracking progress
total_records=$(wc -l < "$INPUT_FILE")
current=0
valid=0
skipped=0

echo "Processing migration commands from $INPUT_FILE"
echo "Total records to process: $total_records"
echo "Will execute in parallel with $PARALLEL_JOBS jobs"
echo "-------------------------------------------"

# Process each line of the file
while IFS=$'\t' read -r id owner object_type || [[ -n "$id" ]]; do
    # Skip header or empty lines
    if [[ "$id" =~ ^[[:space:]]*$ || "$id" == "id" ]]; then
        continue
    fi

    # Increment counter
    ((current++))

    # Extract address and coin type
    address="$owner"

    # Extract the actual coin type from CoinStore<TYPE> format
    if [[ "$object_type" =~ CoinStore\<(.*)\> ]]; then
        coin_type="${BASH_REMATCH[1]}"
    else
        echo "[$current/$total_records] Warning: Could not parse coin type from $object_type, skipping..."
        ((skipped++))
        continue
    fi

    # Remove any potential whitespace
    address=$(echo "$address" | tr -d ' ')
    coin_type=$(echo "$coin_type" | tr -d ' ')

#    # Skip migration for address 0x0 (it's often a placeholder/dummy address)
#    if [[ "$address" == "0x0" ]]; then
#        echo "[$current/$total_records] Skipping address 0x0..."
#        ((skipped++))
#        continue
#    fi

    # Generate the migration command 
    # Using printf to ensure proper escaping of special characters
    printf "echo \"Migrating %s for address %s...\"; rooch move run --function 0x3::coin_migration::migrate_account_entry --type-args \"%s\" --args address:%s --sender default; echo \"-------------------------------------------\"\n" "$coin_type" "$address" "$coin_type" "$address" >> "$TEMP_COMMANDS_FILE"
    
    echo "[$current/$total_records] Generated command for $coin_type at address $address"
    ((valid++))
done < "$INPUT_FILE"

# Print summary before execution
echo "-------------------------------------------"
echo "Command generation completed"
echo "Total records processed: $current"
echo "Valid migration commands: $valid"
echo "Skipped records: $skipped"
echo "-------------------------------------------"
echo "Starting parallel execution with $PARALLEL_JOBS jobs..."
echo "-------------------------------------------"

# Execute all commands in parallel
# Using --bar to show progress bar
parallel --bar -j "$PARALLEL_JOBS" < "$TEMP_COMMANDS_FILE"

# Clean up temporary file
#echo "$TEMP_COMMANDS_FILE"
rm "$TEMP_COMMANDS_FILE"

# Print completion message
echo "-------------------------------------------"
echo "Parallel execution completed at $(date)"
echo "-------------------------------------------" 
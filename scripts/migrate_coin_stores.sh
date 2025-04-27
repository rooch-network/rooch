#!/bin/bash
# Copyright (c) RoochNetwork
# SPDX-License-Identifier: Apache-2.0

# Check if the input file is provided
if [ "$#" -ne 1 ]; then
    echo "Usage: $0 <path_to_coin_stores_file>"
    echo "Example: $0 ./scripts/coin_stores_testnet.txt"
    exit 1
fi

INPUT_FILE="$1"

# Check if the file exists
if [ ! -f "$INPUT_FILE" ]; then
    echo "Error: File $INPUT_FILE does not exist."
    echo "Please provide the full path to the coin stores file."
    exit 1
fi

# Counter for tracking progress
total_records=$(wc -l < "$INPUT_FILE")
current=0
success=0
failed=0

echo "Processing coin store migrations from $INPUT_FILE"
echo "Total records to process: $total_records"
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
        echo "Warning: Could not parse coin type from $object_type, skipping..."
        ((failed++))
        continue
    fi

    # Remove any potential whitespace
    address=$(echo "$address" | tr -d ' ')
    coin_type=$(echo "$coin_type" | tr -d ' ')

#    # Skip migration for address 0x0 (it's often a placeholder/dummy address)
#    if [[ "$address" == "0x0" ]]; then
#        echo "[$current/$total_records] Skipping address 0x0..."
#        continue
#    fi

    # Log the command
    echo "[$current/$total_records] Migrating $coin_type for address $address..."

    # Prepare and execute the migration command
    migration_cmd="rooch move run --function 0x3::coin_migration::migrate_account_entry --type-args $coin_type --args address:$address --sender default"
    echo "Executing: $migration_cmd"

    # Execute the command and capture output
    if output=$(eval "$migration_cmd" 2>&1); then
        echo "  Success"
        ((success++))
    else
        echo "  Failed: $output"
        ((failed++))
    fi

    # Add a separator for readability
    echo "-------------------------------------------"

    # Optional: add a small delay to prevent overwhelming the system
    sleep 0.1
done < "$INPUT_FILE"

# Print summary
echo "-------------------------------------------"
echo "Migration completed at $(date)"
echo "Total records processed: $current"
echo "Successful migrations: $success"
echo "Failed migrations: $failed"
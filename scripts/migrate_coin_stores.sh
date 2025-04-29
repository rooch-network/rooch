#!/bin/bash
# Copyright (c) RoochNetwork
# SPDX-License-Identifier: Apache-2.0

# Check if the input file is provided
if [ "$#" -lt 1 ] || [ "$#" -gt 2 ]; then
    echo "Usage: $0 <path_to_coin_stores_file> [--batch=batch_size]"
    echo "Example: $0 ./scripts/coin_stores_testnet.txt"
    echo "Example with batch: $0 ./scripts/coin_stores_testnet.txt --batch=30"
    echo "         (default batch size is 30)"
    exit 1
fi

INPUT_FILE="$1"
BATCH_SIZE=30

# Check if batch parameter is provided
if [ "$#" -eq 2 ]; then
    if [[ "$2" =~ ^--batch=([0-9]+)$ ]]; then
        BATCH_SIZE="${BASH_REMATCH[1]}"
        echo "Running with batch size: $BATCH_SIZE"
    else
        echo "Error: Invalid batch parameter format. Use --batch=N where N is a number."
        exit 1
    fi
fi

# Check if the file exists
if [ ! -f "$INPUT_FILE" ]; then
    echo "Error: File $INPUT_FILE does not exist."
    echo "Please provide the full path to the coin stores file."
    exit 1
fi

# Create a temporary file for commands
TEMP_COMMANDS_FILE=$(mktemp)

# Create a temporary file for addresses
TEMP_ADDRESSES_FILE=$(mktemp)

# Counter for tracking progress
total_records=$(wc -l < "$INPUT_FILE")
current=0
valid=0
skipped=0

echo "Processing migration commands from $INPUT_FILE"
echo "Total records to process: $total_records"
echo "Will process in batches of $BATCH_SIZE addresses"
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
#    coin_type=$(echo "$coin_type" | tr -d ' ')

    # Add address and coin type to temp file
    echo -e "$address\t$coin_type" >> "$TEMP_ADDRESSES_FILE"
    ((valid++))
    
    echo "[$current/$total_records] Added address $address for coin type $coin_type"
done < "$INPUT_FILE"

# Print summary before processing batches
echo "-------------------------------------------"
echo "Address processing completed"
echo "Total records processed: $current"
echo "Valid addresses: $valid"
echo "Skipped records: $skipped"
echo "-------------------------------------------"

# Function to finalize a batch command
finalize_batch() {
    batch_cmd+="\" --sender default; echo \"-------------------------------------------\""
    echo "$batch_cmd" >> "$TEMP_COMMANDS_FILE"
}

# Process addresses in batches
current_coin_type=""
addresses_in_coin_type=0
last_finalized_batch=0

while IFS=$'\t' read -r address coin_type; do
    # Check if we're starting a new coin type
    if [[ "$coin_type" != "$current_coin_type" ]]; then
        # Finalize previous batch if needed
        if [[ -n "$batch_cmd" && last_finalized_batch -lt addresses_in_coin_type ]]; then
            finalize_batch
        fi
        
        # Reset counters for new coin type
        addresses_in_coin_type=0
        last_finalized_batch=0
        current_coin_type="$coin_type"
        
        # Count total addresses for this coin type
        total_addresses=$(grep -c "$coin_type" "$TEMP_ADDRESSES_FILE")
        total_batches=$(( (total_addresses + BATCH_SIZE - 1) / BATCH_SIZE ))
        
        echo "Processing $total_addresses addresses for coin type $coin_type in $total_batches batches"
        echo "-------------------------------------------"
        
        # Start a new batch
        current_batch=1
        batch_cmd="echo \"Processing batch $current_batch/$total_batches for coin type $coin_type...\"; rooch move run --function 0x3::coin_migration::migrate_accounts_batch_entry --type-args \"$coin_type\" --args \"vector<address>:"
        first_in_batch=true
    fi
    
    # Add separator if not the first in batch
    if ! $first_in_batch; then
        batch_cmd+=","
    else
        first_in_batch=false
    fi
    
    # Add address to batch command
    batch_cmd+="$address"
    ((addresses_in_coin_type++))
    
    # Finalize batch if it's full
    if (( addresses_in_coin_type % BATCH_SIZE == 0 )); then
        finalize_batch
        last_finalized_batch=addresses_in_coin_type
        
        # Start a new batch if there are more addresses
        if (( addresses_in_coin_type < total_addresses )); then
            current_batch=$(( addresses_in_coin_type / BATCH_SIZE + 1 ))
            batch_cmd="echo \"Processing batch $current_batch/$total_batches for coin type $coin_type...\"; rooch move run --function 0x3::coin_migration::migrate_accounts_batch_entry --type-args \"$coin_type\" --args \"vector<address>:"
            first_in_batch=true
        else
            batch_cmd=""
        fi
    fi
done < "$TEMP_ADDRESSES_FILE"

# Finalize the last batch if needed
if [[ -n "$batch_cmd" && last_finalized_batch -lt addresses_in_coin_type ]]; then
    finalize_batch
fi

# Execute commands sequentially
echo "Starting batch execution..."
echo "-------------------------------------------"
bash "$TEMP_COMMANDS_FILE"

# Clean up temporary files
rm "$TEMP_ADDRESSES_FILE"
rm "$TEMP_COMMANDS_FILE"

# Print completion message
echo "-------------------------------------------"
echo "Execution completed at $(date)"
echo "-------------------------------------------" 
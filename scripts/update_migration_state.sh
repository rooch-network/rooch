#!/bin/bash
# Copyright (c) RoochNetwork
# SPDX-License-Identifier: Apache-2.0

# Check if the input file is provided
if [ "$#" -lt 1 ] || [ "$#" -gt 2 ]; then
    echo "Usage: $0 <path_to_users_file> [--batch=batch_size]"
    echo "Example: $0 ./scripts/users_test.txt"
    echo "Example with batch: $0 ./scripts/users_test.txt --batch=10"
    echo "         (default batch size is 10)"
    exit 1
fi

INPUT_FILE="$1"
BATCH_SIZE=10

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
    echo "Please provide the full path to the users file."
    exit 1
fi

# Create a temporary file for commands
TEMP_COMMANDS_FILE=$(mktemp)

# Counter for tracking progress
total_records=$(wc -l < "$INPUT_FILE")
current=0
valid=0
skipped=0

echo "Processing migration state updates from $INPUT_FILE"
echo "Total records to process: $total_records"
echo "Will process in batches of $BATCH_SIZE addresses"

# Create a temporary addresses file
TEMP_ADDRESSES_FILE=$(mktemp)

# Process each line of the file for batch mode
while IFS=$'\t' read -r address rest_of_line || [[ -n "$address" ]]; do
    # Skip header or empty lines
    if [[ "$address" =~ ^[[:space:]]*$ || "$address" == "address" || "$address" == "owner" ]]; then
        continue
    fi

    # Increment counter
    ((current++))

    # Remove any potential whitespace
    address=$(echo "$address" | tr -d ' ')

    # Skip address 0x0
    if [[ "$address" == "0x0" ]]; then
        echo "[$current/$total_records] Skipping address 0x0..."
        ((skipped++))
        continue
    fi

    # Add address to temp file
    echo "$address" >> "$TEMP_ADDRESSES_FILE"
    ((valid++))
done < "$INPUT_FILE"

# Print summary before processing batches
echo "-------------------------------------------"
echo "Address processing completed"
echo "Total records processed: $current"
echo "Valid addresses: $valid"
echo "Skipped addresses: $skipped"
echo "-------------------------------------------"

# Count total batches
total_lines=$(wc -l < "$TEMP_ADDRESSES_FILE")
total_batches=$(( (total_lines + BATCH_SIZE - 1) / BATCH_SIZE ))
echo "Will create $total_batches batches"
echo "-------------------------------------------"

# Process in batches
batch_num=0
while read -r line; do
    # Start a new batch command
    if (( batch_num % BATCH_SIZE == 0 )); then
        if [[ -n "$batch_cmd" ]]; then
            # Finalize previous batch command
            batch_cmd+="]"
            echo "$batch_cmd" >> "$TEMP_COMMANDS_FILE"
        fi

        # Start a new batch
        current_batch=$(( batch_num / BATCH_SIZE + 1 ))
        batch_cmd="echo \"Processing batch $current_batch/$total_batches...\"; rooch move run --function 0x3::coin_migration::update_migration_states_batch_entry --args \"vector<address>:"
        first_in_batch=true
    fi

    # Add separator if not the first in batch
    if ! $first_in_batch; then
        batch_cmd+=","
    else
        first_in_batch=false
    fi

    # Add address to batch command
    batch_cmd+="$line"
    ((batch_num++))

    # Finalize the last batch if needed
    if (( batch_num == total_lines )); then
        batch_cmd+="\" --sender default; echo \"-------------------------------------------\""
        echo "$batch_cmd" >> "$TEMP_COMMANDS_FILE"
    elif (( batch_num % BATCH_SIZE == 0 )); then
        batch_cmd+="\" --sender default; echo \"-------------------------------------------\""
    fi
done < "$TEMP_ADDRESSES_FILE"

# Execute commands sequentially
echo "Starting batch execution..."
echo "-------------------------------------------"
bash "$TEMP_COMMANDS_FILE"

# Clean up
rm "$TEMP_ADDRESSES_FILE"

# Clean up temporary file
echo  "$TEMP_COMMANDS_FILE"
rm "$TEMP_COMMANDS_FILE"

# Print completion message
echo "-------------------------------------------"
echo "Execution completed at $(date)"
echo "-------------------------------------------" 
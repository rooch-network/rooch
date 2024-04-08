#!/bin/bash
# Copyright (c) RoochNetwork
# SPDX-License-Identifier: Apache-2.0

# Function to search for constant errors in .move files with "E" prefix
check_constant_errors_for_e() {
    local dir="$1"

    # Find .move files and check constant errors within them
    find "$dir" -type f -name "*.move" -print0 | while IFS= read -r -d '' file; do
        constants=($(grep -E "const [A-Z0-9_].*;" "$file" | cut -d':' -f1))
        
        for constant in "${constants[@]}"; do
            if [[ "$constant" =~ ^E.*$ ]]; then
                if [[ ! "$constant" =~ ^E[A-Z0-9_]*$ ]]; then
                    echo "Integrity violated: $constant does not match expected format in $file"
                    exit 1
                else
                    echo "$file: $constant -> E"
                fi
            fi
        done
    done
}

# Function to search for constant errors in .move files with "Error" prefix
check_constant_errors_for_error() {
    local dir="$1"

    # Find .move files and check constant errors within them
    find "$dir" -type f -name "*.move" -print0 | while IFS= read -r -d '' file; do
        constants=($(grep -E "const [A-Za-z0-9].*;" "$file" | cut -d':' -f1))
        
        for constant in "${constants[@]}"; do
            if [[ "$constant" =~ ^(E|Error)[A-Za-z0-9]*$ ]]; then
                if [[ "$constant" == "E"* && "$constant" != "Error"* ]]; then
                    echo "Integrity violated: $constant does not match expected prefix Error in $file"
                    exit 1
                fi
                echo "$file: $constant -> Error"
            fi
        done
    done
}

# Check constant errors for E* prefix in move-stdlib
check_constant_errors_for_e "frameworks/move-stdlib/sources" || exit 1

# Check constant errors for Error* prefix in moveos-stdlib, rooch-framework, and examples
for dir in "frameworks/moveos-stdlib/sources" \
           "frameworks/rooch-framework/sources" \
           "frameworks/bitcoin-move/sources" \
           "examples"; do
    check_constant_errors_for_error "$dir" || exit 1
done
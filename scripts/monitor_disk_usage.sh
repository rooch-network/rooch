#!/bin/bash
# Copyright (c) RoochNetwork
# SPDX-License-Identifier: Apache-2.0
#
# Disk usage monitoring script for pruner E2E tests
# Monitors disk usage of TestBox data directories and logs to a CSV file
#
# Usage: monitor_disk_usage.sh <base_dir> <output_file>
#   base_dir: Base directory where TestBox creates testbox-* subdirectories
#   output_file: CSV file to write monitoring data (format: timestamp,current_size,peak_size)

set -euo pipefail

BASE_DIR="${1:-}"
OUTPUT_FILE="${2:-}"

if [ -z "$BASE_DIR" ] || [ -z "$OUTPUT_FILE" ]; then
  echo "Usage: $0 <base_dir> <output_file>" >&2
  exit 1
fi

PEAK_SIZE=0
LAST_LOG_TIME=0

# Function to calculate total size of all testbox directories (cross-platform)
calculate_size() {
  local total=0

  if [ -d "$BASE_DIR" ]; then
    while IFS= read -r dir; do
      # Skip .rooch_test if .rooch_test/data exists (avoid double counting)
      if [[ "$dir" == */.rooch_test ]] && [ -d "${dir}/data" ]; then
        continue
      fi

      if [ -d "$dir" ]; then
        local size
        # Try GNU du first (Linux CI runners), then fall back to find+stat (macOS/BSD)
        if size=$(du -sb "$dir" 2>/dev/null | awk '{print $1}'); then
          # GNU du worked
          if [ -n "$size" ] && [ "$size" -ge 0 ] 2>/dev/null; then
            total=$((total + size))
          fi
        else
          # Fallback for BSD/macOS: use find with stat
          size=$(find "$dir" -type f -exec stat -f %z {} + 2>/dev/null | awk '{sum += $1} END {print sum}')
          if [ -n "$size" ] && [ "$size" -ge 0 ] 2>/dev/null; then
            total=$((total + size))
          fi
        fi
      fi
    done < <(find "$BASE_DIR" -type d \( -path "*/testbox-*/.rooch_test/data" -o -path "*/testbox-*/.rooch_test" \) 2>/dev/null || true)
  fi
  echo "$total"
}

# Wait a bit for directories to be created
echo "Waiting 10 seconds for test directories to be created..."
sleep 10

echo "Starting disk usage monitoring for: $BASE_DIR"
echo "Output file: $OUTPUT_FILE"
echo "Format: timestamp,current_size,peak_size"

# Write header to output file
echo "timestamp,current_size,peak_size" > "$OUTPUT_FILE"

# Debug: Check if base directory exists and list found directories
if [ -d "$BASE_DIR" ]; then
  echo "Base directory exists: $BASE_DIR"
  FOUND_DIRS=$(find "$BASE_DIR" -type d \( -path "*/testbox-*/.rooch_test/data" -o -path "*/testbox-*/.rooch_test" \) 2>/dev/null | wc -l)
  echo "Found $FOUND_DIRS matching directories"
  if [ "$FOUND_DIRS" -gt 0 ]; then
    echo "Directories found:"
    find "$BASE_DIR" -type d \( -path "*/testbox-*/.rooch_test/data" -o -path "*/testbox-*/.rooch_test" \) 2>/dev/null | head -5
  fi
else
  echo "WARNING: Base directory does not exist: $BASE_DIR"
fi

ITERATION=0
while true; do
  ITERATION=$((ITERATION + 1))
  CURRENT_SIZE=$(calculate_size)
  CURRENT_TIME=$(date +%s)
  
  # Always log at least once after waiting period, even if size is 0
  if [ $ITERATION -eq 1 ]; then
    PEAK_SIZE=$CURRENT_SIZE
    echo "$CURRENT_TIME,$CURRENT_SIZE,$PEAK_SIZE" >> "$OUTPUT_FILE"
    LAST_LOG_TIME=$CURRENT_TIME
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] Initial measurement: $CURRENT_SIZE bytes"
  # Log every time size increases
  elif [ -n "$CURRENT_SIZE" ] && [ "$CURRENT_SIZE" -gt $PEAK_SIZE ] 2>/dev/null; then
    PEAK_SIZE=$CURRENT_SIZE
    echo "$CURRENT_TIME,$CURRENT_SIZE,$PEAK_SIZE" >> "$OUTPUT_FILE"
    LAST_LOG_TIME=$CURRENT_TIME
    # Format size in human-readable format (MB)
    SIZE_MB=$(awk "BEGIN {printf \"%.2f\", $CURRENT_SIZE / 1024 / 1024}")
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] Peak updated: $CURRENT_SIZE bytes (${SIZE_MB} MB)"
  # Log current size periodically (every 30 seconds) even if not peak
  elif [ $((CURRENT_TIME - LAST_LOG_TIME)) -ge 30 ]; then
    echo "$CURRENT_TIME,$CURRENT_SIZE,$PEAK_SIZE" >> "$OUTPUT_FILE"
    LAST_LOG_TIME=$CURRENT_TIME
    # Format sizes in human-readable format (MB)
    if [ "$CURRENT_SIZE" -gt 0 ] 2>/dev/null; then
      SIZE_MB=$(awk "BEGIN {printf \"%.2f\", $CURRENT_SIZE / 1024 / 1024}")
      PEAK_MB=$(awk "BEGIN {printf \"%.2f\", $PEAK_SIZE / 1024 / 1024}")
      echo "[$(date '+%Y-%m-%d %H:%M:%S')] Current: $CURRENT_SIZE bytes (${SIZE_MB} MB), Peak: $PEAK_SIZE bytes (${PEAK_MB} MB)"
    else
      echo "[$(date '+%Y-%m-%d %H:%M:%S')] Current: $CURRENT_SIZE bytes, Peak: $PEAK_SIZE bytes (no data found yet)"
    fi
  fi
  sleep 5
done


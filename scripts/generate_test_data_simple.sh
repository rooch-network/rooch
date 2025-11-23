#!/bin/bash
# Simplified test data generator for Pruning verification
# Uses batch account creation to generate realistic state data

set -e

TARGET_SIZE_GB="${1:-3}"
ROOCH_DATA_DIR="${ROOCH_DATA_DIR:-$HOME/.rooch/local}"
ROOCH_DB="$ROOCH_DATA_DIR/roochdb/store"

echo "╔═══════════════════════════════════════════════════════════════════════╗"
echo "║     Generate Test Data for Pruning Verification (Simple Mode)        ║"
echo "╚═══════════════════════════════════════════════════════════════════════╝"
echo ""
echo "Target: ${TARGET_SIZE_GB} GB of test data"
echo "Method: Batch account creation and state updates"
echo ""

# ============================================================================
# Check environment
# ============================================================================

if ! command -v rooch &> /dev/null; then
    echo "❌ Error: 'rooch' not found in PATH"
    exit 1
fi

if [ ! -d "$ROOCH_DB" ]; then
    echo "⚠️  Database not found at: $ROOCH_DB"
    echo "   Initializing new database..."
    rooch server start -d "$ROOCH_DATA_DIR" &
    SERVER_PID=$!
    sleep 5
else
    echo "✅ Database found: $ROOCH_DB"
fi

# Check if server is running
if ! pgrep -f "rooch.*server" > /dev/null; then
    echo "Starting rooch server..."
    rooch server start -d "$ROOCH_DATA_DIR" &
    SERVER_PID=$!
    sleep 5
fi

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "📸 Taking baseline snapshot"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

INITIAL_SIZE=$(du -sb "$ROOCH_DB" 2>/dev/null | awk '{print $1}' || echo "0")
INITIAL_SIZE_GB=$((INITIAL_SIZE / 1024 / 1024 / 1024))

echo "Initial database size: ${INITIAL_SIZE_GB} GB"
./rooch/scripts/measure_pruning_effect.sh test-baseline 2>/dev/null || true

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🔨 Generating test data"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "📋 Strategy (True 50% Object Update for Pruning Test):"
echo "   • Step 1: Deploy custom Move module (test_data_generator)"
echo "   • Step 2: Create 100 test objects as fixed object pool"
echo "   • 50% operations: Create NEW objects (new data, will be kept)"
echo "   • 50% operations: Update existing objects in pool (generates old versions)"
echo "   • Object updates overwrite data → old versions become prunable"
echo "   • Sequential execution with 100 ops/batch, 0.01s delay"
echo ""
echo "📊 Node Size: 250-800 bytes typical, ~600B-1.5KB per op after sharing"
echo "🎯 Target: ${TARGET_SIZE_GB} GB = $((TARGET_SIZE_GB * 1024)) MB"
echo "   • Estimated ops:   $((TARGET_SIZE_GB * 1024 * 1024)) operations"
echo "   • Expected rate:   50-100 ops/sec (stable)"
echo ""

TARGET_SIZE_BYTES=$((TARGET_SIZE_GB * 1024 * 1024 * 1024 + INITIAL_SIZE))

echo "Progress will be displayed every 30 seconds..."
echo "Press Ctrl+C to stop early"
echo ""

# Object pool for repeated updates
OBJECT_POOL=()
OBJECT_INDEX=0
MODULE_ADDRESS=""

# Deploy the test module
deploy_test_module() {
    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "📦 Deploying test_data_generator module..."
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""
    
    # Get default account address first
    MODULE_ADDRESS=$(rooch account list --json 2>/dev/null | jq -r '.[0].hex_address // empty' 2>/dev/null)
    
    if [ -z "$MODULE_ADDRESS" ]; then
        # Fallback to grep if jq is not available
        MODULE_ADDRESS=$(rooch account list --json 2>/dev/null | grep -oE '"hex_address":"0x[^"]+' | head -1 | cut -d'"' -f4)
    fi
    
    if [ -z "$MODULE_ADDRESS" ]; then
        echo "❌ Error: Could not find default account address"
        exit 1
    fi
    
    echo "📝 Using account address: $MODULE_ADDRESS"
    echo ""
    
    cd "$(dirname "$0")/../examples/test_data_generator" || exit 1
    
    # Skip checking if module exists (rooch move list can hang)
    # Just try to publish - if module already exists, publish will handle it
    echo "Preparing to publish module..."
    
    # Publish the module directly (publish will build automatically)
    # Add timeout and progress indicator
    echo "Publishing module (this may take 30-60 seconds for first time)..."
    echo "   Note: First publish may download dependencies from GitHub"
    echo ""
    
    # Start progress indicator in background
    (
        while true; do
            echo "   ⏳ Still working... (this is normal, please wait)"
            sleep 10
        done
    ) &
    PROGRESS_PID=$!
    
    # Use timeout to prevent hanging (5 minutes should be enough)
    # Try without skip-fetch first, if it's too slow user can add the flag
    PUBLISH_SUCCESS=false
    if timeout 300 rooch move publish \
        --named-addresses test_data_generator=default \
        2>&1 | tee /tmp/publish_output.txt; then
        PUBLISH_SUCCESS=true
    fi
    
    # Kill progress indicator
    kill $PROGRESS_PID 2>/dev/null || true
    wait $PROGRESS_PID 2>/dev/null || true
    
    if [ "$PUBLISH_SUCCESS" = true ]; then
        echo ""
        echo "✅ Publish completed"
    else
        PUBLISH_EXIT_CODE=$?
        if [ $PUBLISH_EXIT_CODE -eq 124 ]; then
            echo ""
            echo "❌ Error: Publish timed out after 5 minutes"
            echo "   This might be due to network issues or large dependencies"
            cd - > /dev/null || exit 1
            exit 1
        else
            # Check if module already exists (this is OK)
            if grep -qi "already exists\|already published\|duplicate" /tmp/publish_output.txt 2>/dev/null; then
                echo ""
                echo "✅ Module already exists (this is OK)"
            else
                echo ""
                echo "❌ Error: Failed to publish module (exit code: $PUBLISH_EXIT_CODE)"
                echo "Last 30 lines of output:"
                cat /tmp/publish_output.txt | tail -30
                cd - > /dev/null || exit 1
                exit 1
            fi
        fi
    fi
    
    cd - > /dev/null || exit 1
    
    # Wait a moment for state to sync
    echo "Waiting for state sync..."
    sleep 2
    
    echo ""
    echo "✅ Module published successfully"
    echo "📝 Module address: $MODULE_ADDRESS"
    echo "📝 Module name: test_object"
    echo "📝 Full module path: ${MODULE_ADDRESS}::test_object"
    echo ""
    
    # Test if we can call a module function (will fail if module doesn't exist, but object might not exist yet)
    echo "Verifying module accessibility..."
    if rooch move view --function ${MODULE_ADDRESS}::test_object::get_object_value --args "u64:0" 2>&1 | grep -q "error\|Error"; then
        echo "⚠️  Warning: Module view call failed (this is OK if objects don't exist yet)"
    else
        echo "✅ Module is accessible"
    fi
    echo ""
}

# Initialize object pool (create 100 objects with fixed IDs 0-99)
init_object_pool() {
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "🔨 Creating object pool (100 objects with IDs 0-99)..."
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""
    
    local created=0
    for i in {0..99}; do
        # Create object with custom ID (i will be the object ID)
        rooch move run \
            --function ${MODULE_ADDRESS}::test_object::create_object \
            --args "u64:$i" \
            > /dev/null 2>&1 || true
        
        # Store object ID value for later updates
        OBJECT_POOL+=("$i")
        created=$((created + 1))
        
        if [ $((created % 20)) -eq 0 ]; then
            echo "  Created $created objects..."
        fi
        
        sleep 0.02
    done
    
    echo "✅ Created ${#OBJECT_POOL[@]} objects in pool (IDs: 0-99)"
    echo ""
}

# Function to create objects in batch
generate_batch() {
    local batch_num=$1
    
    # 数据生成策略 (真正的 50% object 重复更新):
    # - 50% 创建新 object（新数据）
    # - 50% 更新 object pool 中的 object（覆盖旧数据，产生可清理的旧版本）
    for i in {1..100}; do
        local rand=$((RANDOM % 100))
        
        if [ $rand -lt 50 ]; then
            # 50%: 创建新 object（新数据，保留）
            # Use large ID values (1000+) to avoid conflict with pool IDs (0-99)
            local new_id=$((1000 + RANDOM))
            rooch move run \
                --function ${MODULE_ADDRESS}::test_object::create_object \
                --args "u64:$new_id" \
                > /dev/null 2>&1 || true
        else
            # 50%: 更新已存在的 object（覆盖旧数据，产生旧版本）
            if [ ${#OBJECT_POOL[@]} -gt 0 ]; then
                local obj_id_value="${OBJECT_POOL[$OBJECT_INDEX]}"
                rooch move run \
                    --function ${MODULE_ADDRESS}::test_object::update_object_by_id \
                    --args "u64:$obj_id_value" "u64:$RANDOM" \
                    > /dev/null 2>&1 || true
                
                # 轮询使用 object pool
                OBJECT_INDEX=$(( (OBJECT_INDEX + 1) % ${#OBJECT_POOL[@]} ))
            fi
        fi
        
        # 添加小延迟避免过载
        sleep 0.01
    done
}

# Deploy module and initialize
deploy_test_module
init_object_pool

START_TIME=$(date +%s)
ITERATION=0
LAST_REPORT_TIME=$START_TIME

echo "Starting data generation loop..."
echo ""

while true; do
    # Generate a batch of operations
    generate_batch $ITERATION
    
    ITERATION=$((ITERATION + 1))
    
    # Check current size (with default values to prevent unary operator errors)
    CURRENT_SIZE=$(du -sb "$ROOCH_DB" 2>/dev/null | awk '{print $1}')
    CURRENT_SIZE=${CURRENT_SIZE:-$INITIAL_SIZE}
    CURRENT_SIZE=${CURRENT_SIZE:-0}
    CURRENT_SIZE_GB=$((CURRENT_SIZE / 1024 / 1024 / 1024))
    ADDED_SIZE_GB=$((CURRENT_SIZE_GB - INITIAL_SIZE_GB))
    
    # Report progress every 30 seconds
    CURRENT_TIME=$(date +%s)
    if [ $((CURRENT_TIME - LAST_REPORT_TIME)) -ge 30 ]; then
        ELAPSED=$((CURRENT_TIME - START_TIME))
        # Prevent division by zero
        if [ "$TARGET_SIZE_GB" -gt 0 ]; then
            PROGRESS=$((ADDED_SIZE_GB * 100 / TARGET_SIZE_GB))
        else
            PROGRESS=0
        fi
        if [ "$PROGRESS" -gt 100 ]; then PROGRESS=100; fi
        
        # Calculate detailed metrics (100 ops per iteration)
        OPS_DONE=$((ITERATION * 100))
        ADDED_SIZE_KB=$((ADDED_SIZE_GB * 1024 * 1024))
        if [ "$OPS_DONE" -gt 0 ] && [ "$ADDED_SIZE_KB" -gt 0 ] && [ "$ELAPSED" -gt 0 ]; then
            AVG_SIZE_PER_OP=$((ADDED_SIZE_KB * 1024 / OPS_DONE))
            OPS_RATE=$((OPS_DONE / ELAPSED))
            echo "$(date '+%H:%M:%S') | ${PROGRESS}% | +${ADDED_SIZE_GB}/${TARGET_SIZE_GB} GB | Ops: ${OPS_DONE} @ ${OPS_RATE}/s | Avg: ${AVG_SIZE_PER_OP}B/op | ${ELAPSED}s"
        else
            echo "$(date '+%H:%M:%S') | ${PROGRESS}% | Added: ${ADDED_SIZE_GB}/${TARGET_SIZE_GB} GB | Total: ${CURRENT_SIZE_GB} GB | Iterations: $ITERATION"
        fi
        LAST_REPORT_TIME=$CURRENT_TIME
    fi
    
    # Check if target reached (with safe comparison)
    if [ "${CURRENT_SIZE:-0}" -ge "${TARGET_SIZE_BYTES:-0}" ]; then
        echo ""
        echo "✅ Target size reached!"
        echo "   Initial: ${INITIAL_SIZE_GB} GB"
        echo "   Added:   ${ADDED_SIZE_GB} GB"
        echo "   Final:   ${CURRENT_SIZE_GB} GB"
        break
    fi
    
    # Safety check - stop if over 2x target (something might be wrong)
    SAFETY_LIMIT=$((TARGET_SIZE_GB * 2 + INITIAL_SIZE_GB))
    if [ "${CURRENT_SIZE_GB:-0}" -gt "${SAFETY_LIMIT:-999999}" ]; then
        echo ""
        echo "⚠️  Size exceeded 2x target, stopping for safety"
        break
    fi
done

END_TIME=$(date +%s)
TOTAL_TIME=$((END_TIME - START_TIME))

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "✅ Data Generation Complete"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "Summary:"
echo "  Initial size:     ${INITIAL_SIZE_GB} GB"
echo "  Data added:       ${ADDED_SIZE_GB} GB"
echo "  Final size:       ${CURRENT_SIZE_GB} GB"
echo "  Time taken:       ${TOTAL_TIME} seconds ($((TOTAL_TIME / 60)) min)"
echo ""
echo "Operations:"
TOTAL_OPS=$((ITERATION * 100))
# Safe division with default values
if [ "${TOTAL_TIME:-0}" -gt 0 ]; then
    OPS_RATE=$((TOTAL_OPS / TOTAL_TIME))
else
    OPS_RATE=0
fi
ADDED_SIZE_BYTES=$((ADDED_SIZE_GB * 1024 * 1024 * 1024))
if [ "${TOTAL_OPS:-0}" -gt 0 ]; then
    AVG_BYTES_PER_OP=$((ADDED_SIZE_BYTES / TOTAL_OPS))
else
    AVG_BYTES_PER_OP=0
fi
echo "  Total operations: $TOTAL_OPS"
echo "  Operations/sec:   $OPS_RATE"
echo "  Avg size/op:      ${AVG_BYTES_PER_OP} bytes"
echo "  Iterations:       $ITERATION (100 ops/iteration, sequential)"
echo ""
echo "📊 Analysis:"
if [ "${AVG_BYTES_PER_OP:-0}" -ge 600 ] && [ "${AVG_BYTES_PER_OP:-0}" -le 1500 ]; then
    echo "  ✅ Average size/op is within expected range (600-1500 bytes)"
elif [ "${AVG_BYTES_PER_OP:-0}" -lt 600 ] && [ "${AVG_BYTES_PER_OP:-0}" -gt 0 ]; then
    echo "  ⚠️  Average size/op is lower than expected (high node sharing)"
elif [ "${AVG_BYTES_PER_OP:-0}" -gt 1500 ]; then
    echo "  ⚠️  Average size/op is higher than expected (large values?)"
else
    echo "  ⚠️  Not enough data to analyze (too few operations)"
fi
echo ""

echo "📊 Taking final snapshot..."
./rooch/scripts/analyze_node_sharing.sh

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "📝 Next Steps for Pruning Verification"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "Now you can test the Pruner's effectiveness:"
echo ""
echo "1. Take a 'before' snapshot:"
echo "   ./rooch/scripts/measure_pruning_effect.sh before-prune"
echo ""
echo "2. Wait for Pruner to run (check logs):"
echo "   tail -f $ROOCH_DATA_DIR/rooch.log | grep -i pruner"
echo ""
echo "3. Compare results:"
echo "   ./rooch/scripts/measure_pruning_effect.sh before-prune"
echo "   (Choose option 1 to compare)"
echo ""
echo "4. If Pruner doesn't auto-run, you can:"
echo "   - Restart the server to trigger BuildReach phase"
echo "   - Or wait for the configured interval"
echo ""
echo "5. Force compaction after Pruner completes:"
echo "   rooch db rocksdb-gc --db-path $ROOCH_DB"
echo ""
echo "6. Final comparison:"
echo "   ./rooch/scripts/measure_pruning_effect.sh before-prune"
echo ""
echo "Expected result: 5-15% space reclamation due to node sharing"
echo "(This is NORMAL behavior, not a bug!)"
echo ""

# Cleanup
if [ -n "$SERVER_PID" ]; then
    echo "Note: Server was started by this script (PID: $SERVER_PID)"
    echo "      You may want to stop it after testing"
fi


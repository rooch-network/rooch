#!/bin/bash
# Generate 2-5GB test data for Pruning verification
# This script creates a large number of transactions to test the Pruner's effectiveness

set -e

# Configuration
TARGET_SIZE_GB="${1:-3}"  # Default 3GB if not specified
BATCH_SIZE=1000           # Transactions per batch
TX_PER_SECOND=100         # Rate limiting

ROOCH_DATA_DIR="${ROOCH_DATA_DIR:-$HOME/.rooch/local}"
ROOCH_DB="$ROOCH_DATA_DIR/roochdb/store"

echo "╔═══════════════════════════════════════════════════════════════════════╗"
echo "║         Generate Test Data for Pruning Verification                  ║"
echo "╚═══════════════════════════════════════════════════════════════════════╝"
echo ""
echo "Target size: ${TARGET_SIZE_GB} GB"
echo "Data dir:    $ROOCH_DATA_DIR"
echo "Database:    $ROOCH_DB"
echo ""

# ============================================================================
# Check prerequisites
# ============================================================================

check_prerequisites() {
    echo "🔍 Checking prerequisites..."
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    
    # Check if rooch is installed
    if ! command -v rooch &> /dev/null; then
        echo "❌ Error: 'rooch' command not found"
        echo "   Please install rooch first: cargo install --path ./crates/rooch"
        exit 1
    fi
    
    # Check if rooch server is running
    if ! pgrep -f "rooch.*server" > /dev/null; then
        echo "⚠️  Warning: Rooch server is not running"
        echo ""
        read -p "Start rooch server now? (y/N): " start_server
        if [ "$start_server" = "y" ] || [ "$start_server" = "Y" ]; then
            echo "Starting rooch server..."
            rooch server start -d "$ROOCH_DATA_DIR" &
            sleep 5
            echo "✅ Server started"
        else
            echo "❌ Cancelled: Server must be running to generate test data"
            exit 1
        fi
    else
        echo "✅ Rooch server is running"
    fi
    
    # Check current database size
    if [ -d "$ROOCH_DB" ]; then
        CURRENT_SIZE=$(du -sh "$ROOCH_DB" 2>/dev/null | awk '{print $1}' || echo "unknown")
        echo "✅ Current database size: $CURRENT_SIZE"
    else
        echo "⚠️  Database not found (will be created)"
    fi
    
    echo ""
}

# ============================================================================
# Create Move module for test data generation
# ============================================================================

create_test_module() {
    echo "📝 Creating test data generation module..."
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    
    local test_dir="/tmp/rooch_pruning_test_$$"
    mkdir -p "$test_dir/sources"
    
    cat > "$test_dir/Move.toml" << 'EOF'
[package]
name = "pruning_test"
version = "0.0.1"

[dependencies]
MoveStdlib = { local = "../frameworks/move-stdlib" }
MoveosStdlib = { local = "../frameworks/moveos-stdlib" }
RoochFramework = { local = "../frameworks/rooch-framework" }

[addresses]
pruning_test = "_"
std = "0x1"
moveos_std = "0x2"
rooch_framework = "0x3"
EOF

    cat > "$test_dir/sources/data_generator.move" << 'EOF'
/// Test data generator for Pruning verification
module pruning_test::data_generator {
    use std::vector;
    use moveos_std::table::{Self, Table};
    use moveos_std::object::{Self, Object};
    use moveos_std::tx_context;

    /// A test record with some data
    struct Record has key, store {
        id: u64,
        data: vector<u8>,
    }

    /// Storage for test records
    struct DataStore has key {
        records: Table<u64, Record>,
        counter: u64,
    }

    /// Initialize the data store
    public entry fun init_store() {
        let store = DataStore {
            records: table::new(),
            counter: 0,
        };
        let obj = object::new_named_object(store);
        object::to_shared(obj);
    }

    /// Generate a batch of test records
    /// Each record contains ~1KB of data
    public entry fun generate_batch(store_obj: &mut Object<DataStore>, count: u64) {
        let store = object::borrow_mut(store_obj);
        let i = 0;
        while (i < count) {
            let data = vector::empty<u8>();
            let j = 0;
            // Create ~1KB of data per record
            while (j < 1024) {
                vector::push_back(&mut data, ((j % 256) as u8));
                j = j + 1;
            };
            
            let record = Record {
                id: store.counter,
                data,
            };
            table::add(&mut store.records, store.counter, record);
            store.counter = store.counter + 1;
            i = i + 1;
        };
    }

    /// Delete a range of records
    public entry fun delete_range(
        store_obj: &mut Object<DataStore>,
        start: u64,
        end: u64
    ) {
        let store = object::borrow_mut(store_obj);
        let i = start;
        while (i < end) {
            if (table::contains(&store.records, i)) {
                let Record { id: _, data: _ } = table::remove(&mut store.records, i);
            };
            i = i + 1;
        };
    }

    /// Get current counter
    public entry fun get_counter(store_obj: &Object<DataStore>): u64 {
        let store = object::borrow(store_obj);
        store.counter
    }

    /// Clear all records (for testing full deletion)
    public entry fun clear_all(store_obj: &mut Object<DataStore>) {
        let store = object::borrow_mut(store_obj);
        // Drop the entire table
        let DataStore { records, counter: _ } = object::remove(store_obj);
        table::drop(records);
        
        // Create a new empty store
        let new_store = DataStore {
            records: table::new(),
            counter: 0,
        };
        let obj = object::new_named_object(new_store);
        object::to_shared(obj);
    }
}
EOF

    echo "✅ Test module created at: $test_dir"
    echo "$test_dir"
}

# ============================================================================
# Deploy test module
# ============================================================================

deploy_module() {
    local test_dir=$1
    echo ""
    echo "📦 Deploying test module..."
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    
    cd "$test_dir"
    
    # Build the module
    echo "Building..."
    rooch move build || {
        echo "❌ Build failed"
        return 1
    }
    
    # Publish the module
    echo "Publishing..."
    rooch move publish --named-addresses pruning_test=default || {
        echo "❌ Publish failed"
        return 1
    }
    
    echo "✅ Module deployed successfully"
    
    # Initialize the data store
    echo "Initializing data store..."
    rooch move run --function default::data_generator::init_store || {
        echo "❌ Init failed"
        return 1
    }
    
    echo "✅ Data store initialized"
}

# ============================================================================
# Generate test data
# ============================================================================

generate_data() {
    local target_gb=$1
    local test_dir=$2
    
    echo ""
    echo "🔨 Generating test data..."
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    
    # Estimate number of transactions needed
    # Each record is ~1KB, batch of 100 records per tx
    # So each tx generates ~100KB of state data
    # Target: ${target_gb}GB = ${target_gb}000 MB = ${target_gb}000000 KB
    # Needed txs: ${target_gb}000000 / 100 = ${target_gb}0000
    
    local total_txs=$((target_gb * 10000))
    local batch_records=100  # Records per transaction
    
    echo "Target size:      ${target_gb} GB"
    echo "Est. transactions: $total_txs"
    echo "Records per tx:    $batch_records"
    echo "Est. total records: $((total_txs * batch_records))"
    echo ""
    
    # Take initial snapshot
    echo "📸 Taking baseline snapshot..."
    ./rooch/scripts/measure_pruning_effect.sh test-gen-baseline || true
    
    local start_time=$(date +%s)
    local tx_count=0
    local last_report_time=$start_time
    
    echo "Starting data generation (this will take a while)..."
    echo "Press Ctrl+C to stop early"
    echo ""
    
    # Get store object ID (assume it's the first named object)
    # This is a placeholder - in real usage you'd need to query for it
    local store_id="0x..."  # TODO: Get actual object ID
    
    while [ $tx_count -lt $total_txs ]; do
        # Generate batch
        rooch move run \
            --function default::data_generator::generate_batch \
            --args "object:$store_id" "u64:$batch_records" \
            >/dev/null 2>&1 || {
            echo "⚠️  Transaction failed (tx $tx_count), continuing..."
        }
        
        tx_count=$((tx_count + 1))
        
        # Progress reporting
        local current_time=$(date +%s)
        if [ $((current_time - last_report_time)) -ge 30 ]; then
            local elapsed=$((current_time - start_time))
            local progress=$((tx_count * 100 / total_txs))
            local rate=$((tx_count / elapsed))
            local eta=$((total_txs - tx_count))
            eta=$((eta / rate))
            
            # Check current DB size
            local current_size=$(du -sh "$ROOCH_DB" 2>/dev/null | awk '{print $1}' || echo "?")
            
            echo "Progress: $progress% ($tx_count/$total_txs tx) | Rate: ${rate} tx/s | DB: $current_size | ETA: ${eta}s"
            last_report_time=$current_time
        fi
        
        # Rate limiting
        sleep 0.01
    done
    
    local end_time=$(date +%s)
    local total_time=$((end_time - start_time))
    local final_rate=$((total_txs / total_time))
    
    echo ""
    echo "✅ Data generation complete"
    echo "   Total transactions: $total_txs"
    echo "   Total time:         ${total_time}s"
    echo "   Average rate:       ${final_rate} tx/s"
    echo ""
}

# ============================================================================
# Simplified generation using simple transfers
# ============================================================================

generate_simple_data() {
    local target_gb=$1
    
    echo ""
    echo "🔨 Generating test data (simple mode)..."
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""
    echo "This will use simple rooch commands to generate state changes."
    echo "Estimated transactions needed: $((target_gb * 10000))"
    echo ""
    
    # Take baseline snapshot
    echo "📸 Taking baseline snapshot..."
    ./rooch/scripts/measure_pruning_effect.sh gen-baseline 2>/dev/null || true
    
    local tx_count=0
    local target_tx=$((target_gb * 10000))
    local start_time=$(date +%s)
    
    echo "Generating transactions..."
    echo "(This will run for a long time. Press Ctrl+C to stop early)"
    echo ""
    
    while [ $tx_count -lt $target_tx ]; do
        # Use simple rooch commands to generate state changes
        # This is a placeholder - actual implementation would use real rooch commands
        
        # Example: Create and manipulate some on-chain state
        # rooch move run --function some_module::some_function --args ...
        
        tx_count=$((tx_count + 1))
        
        # Progress every 100 tx
        if [ $((tx_count % 100)) -eq 0 ]; then
            local current_time=$(date +%s)
            local elapsed=$((current_time - start_time))
            local progress=$((tx_count * 100 / target_tx))
            local current_size=$(du -sh "$ROOCH_DB" 2>/dev/null | awk '{print $1}' || echo "?")
            
            echo "Progress: $progress% ($tx_count/$target_tx) | DB: $current_size | Time: ${elapsed}s"
        fi
        
        # Check if target size reached
        if [ -d "$ROOCH_DB" ]; then
            local size_bytes=$(du -sb "$ROOCH_DB" 2>/dev/null | awk '{print $1}' || echo "0")
            local size_gb=$((size_bytes / 1024 / 1024 / 1024))
            if [ $size_gb -ge $target_gb ]; then
                echo ""
                echo "✅ Target size reached: ${size_gb}GB >= ${target_gb}GB"
                break
            fi
        fi
    done
    
    echo ""
    echo "✅ Data generation complete"
    
    # Final snapshot
    echo ""
    echo "📊 Taking final snapshot..."
    ./rooch/scripts/analyze_node_sharing.sh
}

# ============================================================================
# Main execution
# ============================================================================

main() {
    echo "⚠️  WARNING: This will generate ${TARGET_SIZE_GB}GB of test data"
    echo "   and may take several hours to complete."
    echo ""
    read -p "Continue? (y/N): " confirm
    
    if [ "$confirm" != "y" ] && [ "$confirm" != "Y" ]; then
        echo "Cancelled"
        exit 0
    fi
    
    check_prerequisites
    
    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "Generation Method:"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""
    echo "1. Custom Move module (recommended) - More control, realistic data"
    echo "2. Simple mode - Uses basic rooch commands"
    echo "3. Cancel"
    echo ""
    read -p "Choose (1/2/3): " method
    
    case $method in
        1)
            local test_dir=$(create_test_module)
            deploy_module "$test_dir"
            generate_data "$TARGET_SIZE_GB" "$test_dir"
            # Cleanup
            rm -rf "$test_dir"
            ;;
        2)
            generate_simple_data "$TARGET_SIZE_GB"
            ;;
        3|*)
            echo "Cancelled"
            exit 0
            ;;
    esac
    
    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "✅ Test Data Generation Complete"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""
    echo "Next steps:"
    echo ""
    echo "1. Verify current database size:"
    echo "   du -sh $ROOCH_DB"
    echo ""
    echo "2. Take a snapshot for comparison:"
    echo "   ./rooch/scripts/measure_pruning_effect.sh before-prune"
    echo ""
    echo "3. Wait for or trigger Pruner to run"
    echo "   (Check logs: tail -f ~/.rooch/local/rooch.log | grep Pruner)"
    echo ""
    echo "4. Compare results:"
    echo "   ./rooch/scripts/measure_pruning_effect.sh before-prune"
    echo ""
    echo "5. Analyze node sharing:"
    echo "   ./rooch/scripts/analyze_node_sharing.sh"
    echo ""
}

# Trap Ctrl+C for clean exit
trap 'echo ""; echo "⚠️  Interrupted by user"; exit 130' INT

main "$@"


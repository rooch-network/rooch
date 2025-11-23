#!/bin/bash
# 实际测量 Pruning 效果
# 在 pruner 运行前后对比,量化清理效果

set -e

ROOCH_DB="${ROOCH_DB:-$HOME/.rooch/local/roochdb/store}"
SNAPSHOT_NAME="${1:-pruning-test}"

echo "╔═══════════════════════════════════════════════════════════════════════╗"
echo "║              Measure Actual Pruning Effectiveness                    ║"
echo "╚═══════════════════════════════════════════════════════════════════════╝"
echo ""

# ============================================================================
# 函数定义
# ============================================================================

take_snapshot() {
    local name=$1
    echo "📸 Taking snapshot: $name"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    
    rooch db delete-benchmark snapshot --name "$name" --db-path "$ROOCH_DB"
    
    echo ""
    echo "✅ Snapshot saved"
    echo ""
}

compare_snapshots() {
    local name=$1
    echo "📊 Comparing with snapshot: $name"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    
    rooch db delete-benchmark compare --name "$name" --db-path "$ROOCH_DB"
    
    echo ""
}

get_pruner_status() {
    echo "🔍 Checking Pruner Status"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    
    # 检查 pruner 进程
    if pgrep -f "rooch.*server" > /dev/null; then
        echo "✅ Rooch server is running"
        
        # 尝试从日志找最近的 pruner 活动
        LOG_FILE="${ROOCH_LOG:-$HOME/.rooch/local/rooch.log}"
        if [ -f "$LOG_FILE" ]; then
            echo ""
            echo "Recent Pruner activity (last 10 lines):"
            echo "─────────────────────────────────────────"
            grep -i "pruner" "$LOG_FILE" | tail -10 || echo "  (No recent pruner logs found)"
        fi
    else
        echo "⚠️  Rooch server is not running"
        echo "   Start with: rooch server start"
    fi
    echo ""
}

estimate_sharing_rate() {
    echo "🧮 Estimating Node Sharing Rate"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""
    
    # 从 RocksDB 属性获取节点总数估计
    rooch db rocksdb-stats --db-path "$ROOCH_DB" > /tmp/stats_measure.txt 2>&1 || {
        echo "❌ Failed to get stats"
        return
    }
    
    TOTAL_NODES=$(grep -A 15 "state_node" /tmp/stats_measure.txt | grep "Est. live data" | awk '{print $4}')
    TOTAL_SIZE=$(grep -A 15 "state_node" /tmp/stats_measure.txt | grep "Total SST" | awk '{print $3}')
    
    echo "state_node CF Statistics:"
    echo "  Total SST size:       ${TOTAL_SIZE:-N/A} GB"
    echo "  Est. live data size:  ${TOTAL_NODES:-N/A} GB"
    echo ""
    echo "💡 Interpretation:"
    echo "   - If Pruner deleted nodes but size didn't decrease much:"
    echo "     → High node sharing rate (most nodes are still referenced)"
    echo "     → This is EXPECTED behavior, not a bug"
    echo ""
    echo "   - If size decreased significantly:"
    echo "     → Low sharing rate (many orphaned nodes were cleaned)"
    echo "     → Pruning was very effective"
    echo ""
    
    rm -f /tmp/stats_measure.txt
}

# ============================================================================
# 主流程
# ============================================================================

if [ ! -d "$ROOCH_DB" ]; then
    echo "❌ Error: Database not found at $ROOCH_DB"
    echo ""
    echo "Usage: $0 [snapshot-name]"
    echo ""
    echo "Set ROOCH_DB environment variable to specify custom path:"
    echo "  export ROOCH_DB=/path/to/roochdb"
    exit 1
fi

echo "Database: $ROOCH_DB"
echo "Snapshot: $SNAPSHOT_NAME"
echo ""

# 检查是否已有快照
SNAPSHOT_FILE="$ROOCH_DB/snapshot_${SNAPSHOT_NAME}.json"

if [ -f "$SNAPSHOT_FILE" ]; then
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "📋 Found existing snapshot: $SNAPSHOT_NAME"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""
    echo "Options:"
    echo "  1. Compare with existing snapshot (show pruning effect)"
    echo "  2. Take a new snapshot (replace existing)"
    echo "  3. Exit"
    echo ""
    read -p "Choose (1/2/3): " choice
    
    case $choice in
        1)
            compare_snapshots "$SNAPSHOT_NAME"
            ;;
        2)
            echo ""
            read -p "⚠️  This will replace existing snapshot. Continue? (y/N): " confirm
            if [ "$confirm" = "y" ] || [ "$confirm" = "Y" ]; then
                take_snapshot "$SNAPSHOT_NAME"
            else
                echo "Cancelled"
                exit 0
            fi
            ;;
        3|*)
            echo "Exiting"
            exit 0
            ;;
    esac
else
    # 没有快照,创建第一个
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "📋 No existing snapshot found"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""
    echo "This is the first snapshot. It will be used as baseline for comparison."
    echo ""
    
    take_snapshot "$SNAPSHOT_NAME"
    
    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "📝 Next Steps:"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""
    echo "1. Wait for Pruner to complete at least one cycle"
    echo "   (Check pruner logs or wait for Incremental phase)"
    echo ""
    echo "2. OR manually trigger cleanup:"
    echo "   rooch prune run  # if available"
    echo ""
    echo "3. Then run this script again to see the effect:"
    echo "   $0 $SNAPSHOT_NAME"
    echo ""
    
    get_pruner_status
fi

echo ""
estimate_sharing_rate

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "✅ Measurement Complete"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "💡 Key Takeaways:"
echo ""
echo "• If reclaimed < 5%:  Normal! High node sharing in SMT Copy-on-Write"
echo "• If reclaimed 5-15%: Expected for moderate workloads"
echo "• If reclaimed > 20%: Excellent! Many orphaned nodes were cleaned"
echo ""
echo "• Current Pruner keeps only 1 live root (latest state)"
echo "• Scans last 30,000 historical transactions for cleanup"
echo "• Node sharing rate typically 95%+ for active system tables"
echo ""


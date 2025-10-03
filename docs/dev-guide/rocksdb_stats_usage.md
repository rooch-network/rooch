# RocksDB 统计工具使用指南

## 简介

`rooch db rocksdb-stats` 是一个用 Rust 编写的 RocksDB 诊断工具，可以分析 Rooch 数据库的磁盘占用、列族统计等信息。

## 安装

工具已集成在 Rooch CLI 中，编译后即可使用：

```bash
cd rooch
cargo build --release -p rooch
```

## 基本用法

### 1. 查看默认数据库统计

```bash
# 停止 rooch 服务（避免数据库锁定）
pkill rooch

# 运行诊断（默认路径 ~/.rooch/local/roochdb/store）
./target/release/rooch db rocksdb-stats
```

### 2. 指定数据库路径

```bash
./target/release/rooch db rocksdb-stats --db-path ~/.rooch/local/roochdb/store
```

### 3. 列出所有列族

```bash
./target/release/rooch db rocksdb-stats --list-cf
```

### 4. 显示详细统计

```bash
./target/release/rooch db rocksdb-stats --detailed
```

## 输出说明

### 基本统计

```
=== RocksDB Statistics ===
Path: /Users/xxx/.rooch/local/roochdb/store

Total directory size: 10.23 GB (10234567890 bytes)

Column Families: 5

--- Column Family: default ---
  Total SST size: 0.12 GB (120000000 bytes)
  Live SST size: 0.10 GB (100000000 bytes)
  Estimated live data: 0.09 GB (90000000 bytes)
  L0 files: 2
  Active snapshots: 0
  Pending compaction: 0.00 GB (0 bytes)

--- Column Family: state_node ---
  Total SST size: 9.50 GB (9500000000 bytes)
  Live SST size: 9.20 GB (9200000000 bytes)
  Estimated live data: 8.50 GB (8500000000 bytes)
  L0 files: 5
  Active snapshots: 0
  Pending compaction: 0.30 GB (300000000 bytes)
```

### 关键指标解释

| 指标 | 说明 | 判断标准 |
|------|------|----------|
| **Total SST size** | 实际磁盘占用（包含已删除数据） | - |
| **Live SST size** | 有效 SST 文件大小 | 应接近 Total |
| **Estimated live data** | 估算的活跃数据 | 实际有用数据量 |
| **L0 files** | Level 0 文件数 | <10 正常，>50 需要 compact |
| **Active snapshots** | 活跃快照数 | 应为 0（服务停止后） |
| **Pending compaction** | 待压缩数据 | >1GB 说明需要 compact |

### 诊断决策树

```
Total ≈ Live ≈ Estimated?
  ├─ 是 → 数据库健康，磁盘占用合理
  │
  └─ Total >> Live
      ├─ Active snapshots > 0?
      │   └─ 是 → 有泄露的 Snapshot，需修复代码
      │
      ├─ Pending compaction > 1GB?
      │   └─ 是 → 需要运行 aggressive_compact()
      │
      └─ L0 files > 20?
          └─ 是 → 需要 major compaction
```

## 常见问题

### Q1: 提示数据库被锁定

```
Error: IO error: lock ...
```

**解决：** 停止 rooch 服务
```bash
pkill rooch
# 或
ps aux | grep rooch
kill -9 <PID>
```

### Q2: Total 远大于 Live

说明有大量已删除数据未被物理清除。

**解决：** 运行 aggressive compact
```bash
# 方法 1：使用 Rooch 内置 pruner
rooch db compact --aggressive

# 方法 2：重启服务触发自动清理
rooch server start -n local ...
```

### Q3: Pending compaction 很大

说明有大量数据等待压缩。

**解决：** 
```bash
# 手动触发（未来会集成此命令）
# 目前需要通过代码调用 aggressive_compact()
```

## 与 ldb 对比

| 功能 | ldb | rooch db rocksdb-stats |
|------|-----|------------------------|
| 安装难度 | 需编译 | 内置 |
| 路径展开 | 不支持 ~ | 支持 |
| 输出格式 | 文本 | 结构化 + 人类可读 |
| 列族统计 | 需多次调用 | 一次显示全部 |
| 集成度 | 独立工具 | Rooch CLI 子命令 |

## 高级用法

### 监控磁盘变化

```bash
#!/bin/bash
# watch_db.sh
while true; do
    clear
    echo "=== $(date) ==="
    rooch db rocksdb-stats | grep -E "Total|Live|Estimated"
    sleep 10
done
```

### 导出统计到文件

```bash
rooch db rocksdb-stats --detailed > db_stats_$(date +%Y%m%d).txt
```

### 对比 pruner 前后

```bash
# 运行前
rooch db rocksdb-stats > before.txt

# 运行 pruner
# ...

# 运行后
rooch db rocksdb-stats > after.txt

# 对比
diff before.txt after.txt
```

## 开发者信息

源码位置：`rooch/crates/rooch/src/commands/db/commands/rocksdb_stats.rs`

主要依赖：
- `raw_store::rocks::RocksDB` - RocksDB 封装
- `shellexpand` - 路径展开
- `anyhow` - 错误处理

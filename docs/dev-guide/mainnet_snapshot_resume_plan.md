# Mainnet Snapshot 迁移执行计划（评审草案）

> 状态: Draft（仅评审，不执行）
> 日期: 2026-02-27

## 1. 背景与目标

- 当前 snapshot 运行在 `rooch-prune-snapshot`（数据源 `/data-prune/.rooch`）。
- 成本目标: 停用独立磁盘方案，改为从主节点数据目录 `/data/.rooch` 继续。
- 安全目标:
  - 不在 `rooch-mainnet` 容器内直接跑 snapshot。
  - 避免两个进程同时写同一 `--output`。
  - 保留断点续跑能力，避免从零开始。

截至 `2026-02-27 09:39:48 UTC` 的现场进度（用于评审上下文）:
- `nodes_visited=761,699,120`
- `nodes_written=761,650,000`
- 最新日志已到 `761,750,000` 写入。

## 2. 关键决策

1. 停掉 `rooch-prune-snapshot` 容器（先停不删），杜绝双写。
2. 新起专用容器（建议名 `rooch-snapshot-main`）执行 snapshot。
3. 新容器只读挂载主数据目录，只给 snapshot 输出目录写权限。
4. 续跑保持原参数一致:
   - `--chain-id main`
   - `--tx-order 240930877`
   - `--output /root/snapshot-work/snapshots/snapshot-optimized`
   - `--batch-size 50000`
   - `--skip-dedup`
   - `--skip-final-compact`
   - 不加 `--force-restart`
   - 不加 `--no-resume`

## 3. 目录级备份策略

- 本次执行改为**目录级全量备份**（先停 snapshot 再拷贝）。
- 当前 `snapshot-optimized` 目录约 `369G`，同盘复制预计 `20~60` 分钟，按 `1` 小时窗口安排。
- 全量备份的目的:
  - 保留完整 `snapshot.db`，避免回滚时依赖重算。
  - 降低误操作导致断点和数据同时损坏的风险。

## 4. 执行步骤（Runbook）

### Step 0: 变量

```bash
SNAP_DIR=/data/snapshot-work/snapshots/snapshot-optimized
LOG_DIR=/data/snapshot-work/logs
NEW_LOG=$LOG_DIR/snapshot-mainnet-resume.log
BK_DIR=/data/snapshot-work/backup/snapshot-optimized-$(date -u +%Y%m%dT%H%M%SZ)
BK_FULL_DIR=/data/snapshot-work/backup/snapshot-optimized-full-$(date -u +%Y%m%dT%H%M%SZ)
NEW_CONTAINER=rooch-snapshot-main
IMAGE=ghcr.io/rooch-network/rooch:v0.13.3
```

### Step 1: 停旧容器（先停不删）

```bash
docker stop rooch-prune-snapshot
docker ps --format 'table {{.Names}}\t{{.Status}}' | grep rooch-prune-snapshot || true
```

### Step 2: 目录级全量备份（停机后执行）

```bash
mkdir -p /data/snapshot-work/backup

# 全量复制 snapshot 目录（建议使用 rsync）
rsync -aH --numeric-ids --info=progress2 \
  "$SNAP_DIR/" \
  "$BK_FULL_DIR/"

# 备份完成后校验大小和文件数
du -sh "$SNAP_DIR" "$BK_FULL_DIR"
find "$SNAP_DIR" -type f | wc -l
find "$BK_FULL_DIR" -type f | wc -l
```

### Step 3: 备份关键元文件（轻量，便于快速查看/回滚）

```bash
mkdir -p "$BK_DIR"
for f in \
  snapshot_progress.json \
  snapshot_progress.backup \
  operation_meta_initial.json \
  operation_meta.json \
  snapshot_meta.json
do
  [ -f "$SNAP_DIR/$f" ] && cp -a "$SNAP_DIR/$f" "$BK_DIR/"
done

ls -lah "$BK_DIR"
```

### Step 4: 创建专用 snapshot 容器

```bash
docker rm -f "$NEW_CONTAINER" 2>/dev/null || true

docker run -d \
  --name "$NEW_CONTAINER" \
  --restart unless-stopped \
  --entrypoint sleep \
  --mount type=bind,source=/data/.rooch,destination=/root/.rooch,readonly \
  --mount type=bind,source=/data/snapshot-work,destination=/root/snapshot-work \
  --cpus=4 \
  --memory-swap=-1 \
  "$IMAGE" \
  infinity
```

### Step 5: 启动续跑任务

```bash
docker exec -d "$NEW_CONTAINER" sh -lc '
/rooch/rooch db state-prune snapshot \
  --chain-id main \
  --tx-order 240930877 \
  --output /root/snapshot-work/snapshots/snapshot-optimized \
  --batch-size 50000 \
  --skip-confirm \
  --skip-dedup \
  --skip-final-compact \
  --data-dir /root/.rooch \
  > /root/snapshot-work/logs/snapshot-mainnet-resume.log 2>&1
'
```

### Step 6: 验证是“续跑”而不是“重跑”

```bash
tail -n 120 "$NEW_LOG" | egrep \
  "Loaded valid progress|Found resumable progress|Resuming traversal|No resumable state|Failed to build snapshot"
```

期望关键字:
- `Loaded valid progress`
- `Found resumable progress`
- `Resuming traversal`

如果出现 `No resumable state`，立即停止并回滚排查。

### Step 7: 日常监控

```bash
tail -f "$NEW_LOG"
```

```bash
docker exec "$NEW_CONTAINER" sh -lc "pgrep -af 'rooch db state-prune snapshot'"
```

```bash
du -sh "$SNAP_DIR"
```

## 5. 回滚方案

适用场景:
- 未命中 resume（误从头开始）
- 启动后报错
- 主节点读取性能不可接受

回滚步骤:

```bash
# 1) 停新任务
docker rm -f rooch-snapshot-main

# 2) 优先恢复目录级全量备份（完整恢复）
mkdir -p "$SNAP_DIR"
rsync -aH --delete "$BK_FULL_DIR/" "$SNAP_DIR/"

# 3) 恢复元文件（只恢复存在的，可选）
for f in snapshot_progress.json snapshot_progress.backup operation_meta_initial.json operation_meta.json snapshot_meta.json; do
  [ -f "$BK_DIR/$f" ] && cp -a "$BK_DIR/$f" "$SNAP_DIR/$f"
done

# 4) 重新启原容器（若需要）
docker start rooch-prune-snapshot
```

## 6. 评审待确认项

1. 新容器镜像默认使用 `v0.13.3`，如需严格保持二进制一致性再切回 `pr-3947-012f04f`。
2. `--cpus` 是否维持 `4`，还是调高（例如 `6` 或 `8`）。
3. 回滚时是否默认“目录级恢复”还是先尝试“仅元文件恢复”。

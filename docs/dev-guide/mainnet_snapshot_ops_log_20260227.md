# Mainnet Snapshot 运维操作日志（2026-02-27）

## 1. 操作目标

- 按 `docs/dev-guide/mainnet_snapshot_resume_plan.md` 执行迁移：
  - 停止旧 snapshot（`/data-prune` 数据源）
  - 全量备份现有 `snapshot-optimized`
  - 在新容器中从主节点数据目录 `/data/.rooch` 续跑 snapshot
- 回收旧的 prune 专用磁盘，降低云盘成本。

## 2. 环境信息

- 主机：`jolestar@34.146.31.215`（`rooch-mainnet-sha`）
- 新 snapshot 容器：`rooch-snapshot-main`
- 主节点容器：`rooch-mainnet`
- 旧 snapshot 容器：`rooch-prune-snapshot`
- snapshot 镜像：`ghcr.io/rooch-network/rooch:v0.13.3`

## 3. 操作时间线（UTC）

1. `~10:11`
- 停止旧容器：`rooch-prune-snapshot`（状态 `Exited (137)`）。

2. `10:11 - 10:31`
- 执行目录级全量备份：
  - 源：`/data/snapshot-work/snapshots/snapshot-optimized`
  - 目标：`/data/snapshot-work/backup/snapshot-optimized-full-20260227T101150Z`
- 备份完成，watchdog 检测到 `done=yes`。
- 备份结果体积约 `369G`。

3. `~10:41`
- 创建新容器 `rooch-snapshot-main`，挂载：
  - `/data/.rooch -> /root/.rooch`（readonly）
  - `/data/snapshot-work -> /root/snapshot-work`（readwrite）
- 启动 snapshot 续跑命令（保持参数一致，未启用 `--force-restart`）。

4. `10:41+`
- 验证命中 resume（关键日志）：
  - `Loaded valid progress`
  - `Found resumable progress`
  - `Resuming from previous snapshot operation`
  - `Resuming traversal`

5. `10:43+`
- 部署并迭代 snapshot watchdog：
  - 文件：`/data/snapshot-work/scripts/snapshot_mainnet_watchdog.sh`
  - 状态输出：`/data/snapshot-work/logs/snapshot-mainnet-status.txt`
  - 日志：`/data/snapshot-work/logs/snapshot-mainnet-watchdog.log`
- 监控字段包含：`nodes_written`、增量速度、`worklist_len/position/remaining`、目录增量。

6. `14:29`
- 为回收 `/data-prune` 执行留档：
  - 目录：`/data/snapshot-work/backup/prune-disk-decom-20260227T142911Z`
  - 含 `docker inspect` / `mount` / `lsblk` / `df` / `du` 等证据文件。

7. `14:29+`
- 关闭旧容器重启策略并确认停机：
  - `docker update --restart=no rooch-prune-snapshot`
  - `docker stop rooch-prune-snapshot`

8. `14:30`
- 卸载 prune 盘挂载点：
  - `umount /data-prune`
  - 验证 `mount` 中不再存在 `/data-prune`。

9. `14:30+`
- 使用本地有权限的 `gcloud` 执行云侧回收：
  - Detach：`rooch-mainnet-db-for-prune` from `rooch-mainnet-sha`
  - Delete：`rooch-mainnet-db-for-prune`（14TB, pd-ssd）
- 删除后校验：
  - 实例磁盘列表不含 prune 盘
  - 磁盘 `users` 为空后已删除成功。

## 4. 当前状态（as-of 2026-02-27T14:30:13Z）

- `rooch-mainnet`：运行中
- `rooch-snapshot-main`：运行中
- snapshot 进度（watchdog）：
  - `nodes_written=768450000`
  - `nodes_visited=768381468`
  - `worklist_len=100`
  - `worklist_remaining=100`
  - `nodes_per_hour=1736334`
  - `done=no`, `failed=no`
- `/data-prune`：已卸载，且对应云盘已删除（成本回收完成）。

## 5. 关键路径清单

- 迁移方案文档：`docs/dev-guide/mainnet_snapshot_resume_plan.md`
- 操作日志（本文件）：`docs/dev-guide/mainnet_snapshot_ops_log_20260227.md`
- Snapshot 运行日志：`/data/snapshot-work/logs/snapshot-mainnet-resume.log`
- Snapshot 状态文件：`/data/snapshot-work/logs/snapshot-mainnet-status.txt`
- Snapshot watchdog 日志：`/data/snapshot-work/logs/snapshot-mainnet-watchdog.log`
- 全量备份目录：`/data/snapshot-work/backup/snapshot-optimized-full-20260227T101150Z`
- 磁盘回收留档：`/data/snapshot-work/backup/prune-disk-decom-20260227T142911Z`


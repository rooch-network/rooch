# Rooch Testnet Data Maintenance Guide

## 概述

当 Rooch testnet 节点出现故障需要数据维护时（如回滚交易、修复数据库、排查问题等），由于数据库文件被锁定，无法在运行中的节点上直接执行维护操作。本指南提供了安全的数据维护流程。

## 文件说明

- `rooch-node/rooch-data-toolbox-pod.yaml`: 数据维护工具箱容器的 Kubernetes 配置
- `data-maintenance-script.sh`: 自动化数据维护流程的脚本
- `DATA_MAINTENANCE_README.md`: 本说明文档

## 使用方法

### 方法 1: 使用自动化脚本（推荐）

1. **开始数据维护流程**:
```bash
cd kube/testnet
./data-maintenance-script.sh
```

2. **执行维护命令**:
```bash
# 进入数据工具箱容器
kubectl exec -it rooch-data-toolbox -n testnet -- /bin/sh

# 在容器内执行各种维护命令（根据需要选择）:
# 回滚交易
/rooch/rooch db rollback --tx-order <YOUR_TX_ORDER> -d /root/.rooch -n test

# 撤销交易
/rooch/rooch db revert-tx --tx-order <YOUR_TX_ORDER> -d /root/.rooch -n test

# 修复数据库
/rooch/rooch db repair -d /root/.rooch -n test

# 查看交易信息
/rooch/rooch db get-tx-by-order --tx-order <ORDER> -d /root/.rooch -n test

# 列出异常
/rooch/rooch db list-anomaly -d /root/.rooch -n test
```

3. **清理并重启节点**:
```bash
./data-maintenance-script.sh cleanup
```

### 方法 2: 手动操作

1. **停止 StatefulSet**:
```bash
kubectl scale statefulset rooch-testnet --replicas=0 -n testnet
```

2. **等待 Pod 完全停止**:
```bash
kubectl get pods -n testnet -w
```

3. **创建数据工具箱 Pod**:
```bash
kubectl apply -f rooch-node/rooch-data-toolbox-pod.yaml
```

4. **等待 Pod 就绪**:
```bash
kubectl wait --for=condition=Ready pod/rooch-data-toolbox -n testnet --timeout=300s
```

5. **执行维护操作**:
```bash
kubectl exec -it rooch-data-toolbox -n testnet -- /bin/sh
# 在容器内执行所需的维护命令
```

6. **清理并重启**:
```bash
kubectl delete pod rooch-data-toolbox -n testnet
kubectl scale statefulset rooch-testnet --replicas=1 -n testnet
```

## 重要注意事项

### 执行前准备

1. **确认 tx_order**: 确保你知道要回滚到的正确交易序号
2. **数据备份**: 建议在执行 rollback 前备份数据
3. **通知用户**: rollback 会导致节点暂时不可用

### 数据备份命令

```bash
# 备份当前数据
kubectl exec -it rooch-testnet-0 -n testnet -- tar -czf /tmp/rooch-backup.tar.gz /root/.rooch
kubectl cp testnet/rooch-testnet-0:/tmp/rooch-backup.tar.gz ./rooch-backup-$(date +%Y%m%d-%H%M%S).tar.gz
```

### 查看交易信息

```bash
# 查看特定交易
kubectl exec -it rooch-data-toolbox -n testnet -- /rooch/rooch db get-tx-by-order --tx-order <ORDER> -d /root/.rooch -n test

# 查看最新的 sequencer 信息
kubectl exec -it rooch-data-toolbox -n testnet -- /rooch/rooch db list-anomaly -d /root/.rooch -n test
```

### 故障排除

1. **Pod 无法启动**: 检查 PVC 是否正确挂载
2. **Rollback 失败**: 检查 tx_order 是否正确，确保小于当前 last_order
3. **节点重启失败**: 检查 rollback 是否成功完成

### 监控和验证

```bash
# 查看节点日志
kubectl logs -f rooch-testnet-0 -n testnet

# 检查节点状态
kubectl get pods -n testnet

# 检查 StatefulSet 状态
kubectl get statefulset -n testnet
```

## 安全建议

1. 在生产环境中执行 rollback 前，务必在测试环境验证
2. 保留完整的操作日志
3. 确保有回滚计划（如果需要回滚 rollback 操作）
4. 通知相关团队维护窗口时间

## 联系支持

如果遇到问题，请提供以下信息：
- 错误日志
- 执行的命令
- 节点状态信息
- 备份文件（如果可能）

# Rooch Testnet Kubernetes Deployment Guide

## 概述

本文档描述了 Rooch 测试网在 Kubernetes 集群中的部署流程。测试网已经从传统的 VM + Docker 部署方式迁移到 Kubernetes 集群，提供了更好的可扩展性、监控和管理能力。

## 部署架构

### 核心组件

- **Rooch Node**: 主要的区块链节点，运行在 StatefulSet 中
- **Bitcoin Node**: 比特币测试网节点，为 Rooch 提供比特币数据
- **Oracle Services**: 价格预言机服务（Binance、OKX、Pyth）
- **Faucet**: 测试代币水龙头服务
- **TBTC Faucet**: 比特币测试网代币水龙头

### 网络配置

- **Ingress**: 通过 GKE Ingress 提供外部访问
- **Services**: 内部服务发现和负载均衡
- **Certificates**: 使用 Google Managed Certificates 提供 HTTPS

## 部署流程

### 1. 自动部署（推荐）

通过 GitHub Actions 工作流自动部署：

```yaml
# 触发方式
workflow_dispatch:
  inputs:
    ref:
      description: 'Tag or branch to deploy'
      default: 'main'
      required: true
```

**所需 GitHub Secrets:**
- `GCP_TESTNET_SA_KEY`: GCP 服务账号密钥（base64 编码）
- `GCP_TESTNET_PROJECT_ID`: GCP 项目 ID
- `GCP_TESTNET_CLUSTER_NAME`: GKE 集群名称
- `GCP_TESTNET_CLUSTER_ZONE`: GKE 集群区域
- `BTC_TEST_RPC_URL`: 比特币测试网 RPC URL
- `BTC_TEST_RPC_PWD`: 比特币测试网 RPC 密码
- `OPENDA_GCP_TESTNET_BUCKET`: OpenDA GCP 存储桶
- `OPENDA_GCP_TESTNET_CREDENTIAL`: OpenDA GCP 凭证
- `TURBO_DA_TURING_ENDPOINT`: Turbo DA Turing 端点
- `TURBO_DA_TURING_API_KEY`: Turbo DA Turing API 密钥

### 2. 手动部署

如果需要手动部署，可以使用以下脚本：

```bash
# 设置 kubectl 访问 GKE 集群
gcloud container clusters get-credentials <cluster-name> --zone <zone>

# 运行部署脚本
bash scripts/deploy_rooch_testnet_k8s.sh \
  v0.10.0 \
  <btc_rpc_url> \
  <btc_rpc_password> \
  <openda_bucket> \
  <openda_credential> \
  <turbo_endpoint> \
  <turbo_api_key>
```

## 部署脚本说明

### `scripts/deploy_rooch_testnet_k8s.sh`

新的 Kubernetes 部署脚本，替代了原来的 Docker 部署脚本。

**功能特性:**
- 镜像引用验证（只允许 `ghcr.io/rooch-network/rooch` 镜像）
- 支持多种镜像格式（标签、完整引用、摘要）
- 自动更新 StatefulSet 和 init 容器镜像
- 滚动更新和状态检查
- 详细的部署日志和错误处理

**支持的镜像格式:**
- 标签: `v0.10.0` → `ghcr.io/rooch-network/rooch:v0.10.0`
- 完整引用: `ghcr.io/rooch-network/rooch:v0.10.0`
- 摘要: `sha256:abc123...` → `ghcr.io/rooch-network/rooch@sha256:abc123...`

### 测试脚本

使用 `scripts/test_deploy_rooch_testnet_k8s.sh` 可以测试部署脚本的语法和验证逻辑：

```bash
bash scripts/test_deploy_rooch_testnet_k8s.sh
```

## 监控和维护

### 查看部署状态

```bash
# 查看 Pod 状态
kubectl get pods -n testnet

# 查看 StatefulSet 状态
kubectl get statefulset -n testnet

# 查看服务状态
kubectl get svc -n testnet

# 查看 Ingress 状态
kubectl get ingress -n testnet
```

### 查看日志

```bash
# 查看 Rooch 节点日志
kubectl logs -f rooch-testnet-0 -n testnet

# 查看 Faucet 日志
kubectl logs -f deployment/testnet-faucet -n testnet

# 查看 Oracle 日志
kubectl logs -f deployment/testnet-oracle-binance -n testnet
```

### 数据维护

当需要进行数据维护时（如回滚交易、修复数据库），可以使用数据维护脚本：

```bash
cd kube/testnet
./data-maintenance-script.sh
```

详细说明请参考 `kube/testnet/DATA_MAINTENANCE_README.md`。

## 故障排除

### 常见问题

1. **Pod 启动失败**
   - 检查镜像是否存在和可访问
   - 检查资源限制和请求
   - 查看 Pod 事件和日志

2. **服务不可访问**
   - 检查 Service 配置
   - 检查 Ingress 配置
   - 检查防火墙规则

3. **证书问题**
   - 检查 Google Managed Certificate 状态
   - 验证域名 DNS 配置

### 回滚操作

如果需要回滚到之前的版本：

```bash
# 查看部署历史
kubectl rollout history statefulset/rooch-testnet -n testnet

# 回滚到上一个版本
kubectl rollout undo statefulset/rooch-testnet -n testnet

# 回滚到特定版本
kubectl rollout undo statefulset/rooch-testnet -n testnet --to-revision=<revision>
```

## 安全考虑

- 所有敏感信息都通过 Kubernetes Secrets 管理
- 镜像验证确保只部署可信的 Rooch 镜像
- 使用 Google Managed Certificates 提供安全的 HTTPS 连接
- 网络策略限制不必要的网络访问

## 性能优化

- 使用 StatefulSet 确保数据持久性
- 配置适当的资源请求和限制
- 使用持久卷存储区块链数据
- 配置健康检查和就绪探针

## 更新日志

- **v1.0.0**: 初始 Kubernetes 部署支持
- 从 VM + Docker 部署迁移到 Kubernetes
- 添加自动部署脚本和测试
- 集成 GitHub Actions 工作流

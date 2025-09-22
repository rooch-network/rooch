# Rooch Testnet 迁移到 Kubernetes 部署修复总结

## 问题描述

原来的 GitHub CI 工作流 `deploy_testnet.yml` 通过脚本直接操作 VM 上的 Docker 容器来更新测试网版本。当测试网迁移到 Kubernetes 后，这个脚本失效了。

## 解决方案

### 1. 创建新的 Kubernetes 部署脚本

**文件**: `scripts/deploy_rooch_testnet_k8s.sh`

**功能特性**:
- 替代原来的 Docker 部署脚本
- 支持多种镜像格式（标签、完整引用、摘要）
- 镜像引用验证（只允许 `ghcr.io/rooch-network/rooch` 镜像）
- 自动更新 StatefulSet 和 init 容器镜像
- 滚动更新和状态检查
- 详细的部署日志和错误处理

**支持的镜像格式**:
- 标签: `v0.10.0` → `ghcr.io/rooch-network/rooch:v0.10.0`
- 完整引用: `ghcr.io/rooch-network/rooch:v0.10.0`
- 摘要: `sha256:abc123...` → `ghcr.io/rooch-network/rooch@sha256:abc123...`

### 2. 更新 GitHub Actions 工作流

**文件**: `.github/workflows/deploy_testnet.yml`

**主要变更**:
- 移除了 SSH 到 VM 的步骤
- 添加了 kubectl 和 GKE 配置
- 使用新的 K8s 部署脚本

**GCE 优化版本**: `.github/workflows/deploy_testnet_gce.yml`
- 专门针对在 GCE 上运行的 self-hosted runner 优化
- 使用默认服务账号，无需管理服务账号密钥
- 简化的认证流程

### 3. 测试和验证

**文件**: `scripts/test_deploy_rooch_testnet_k8s.sh`

**功能**:
- 测试部署脚本的语法和验证逻辑
- 验证镜像引用格式
- 确保脚本可以正确拒绝无效镜像

### 4. 文档和指南

**文件**:
- `docs/deployment/testnet-k8s-deployment.md`: 完整的 K8s 部署指南
- `docs/deployment/gce-runner-setup.md`: GCE self-hosted runner 设置指南
- `docs/deployment/testnet-migration-summary.md`: 本总结文档

## 配置要求

### GitHub Secrets

**必需的 Secrets**:
- `GCP_TESTNET_CLUSTER_NAME`: GKE 集群名称
- `GCP_TESTNET_CLUSTER_ZONE`: GKE 集群区域
- `BTC_TEST_RPC_URL`: 比特币测试网 RPC URL
- `BTC_TEST_RPC_PWD`: 比特币测试网 RPC 密码
- `OPENDA_GCP_TESTNET_BUCKET`: OpenDA GCP 存储桶
- `OPENDA_GCP_TESTNET_CREDENTIAL`: OpenDA GCP 凭证
- `TURBO_DA_TURING_ENDPOINT`: Turbo DA Turing 端点
- `TURBO_DA_TURING_API_KEY`: Turbo DA Turing API 密钥

**GCE Self-Hosted Runner 优化**:
- 不再需要 `GCP_TESTNET_SA_KEY` 和 `GCP_TESTNET_PROJECT_ID`
- 使用 GCE 实例的默认服务账号

### GCE Self-Hosted Runner 设置

如果使用 GCE self-hosted runner，需要：

1. **创建 GCE 实例**:
   ```bash
   gcloud compute instances create github-runner \
     --zone=us-central1-a \
     --machine-type=e2-standard-2 \
     --image-family=ubuntu-2004-lts \
     --scopes=https://www.googleapis.com/auth/cloud-platform
   ```

2. **配置 IAM 权限**:
   ```bash
   gcloud projects add-iam-policy-binding $PROJECT_ID \
     --member="serviceAccount:$SERVICE_ACCOUNT" \
     --role="roles/container.developer"
   ```

3. **安装必要工具**: Docker, kubectl, Google Cloud SDK

## 使用方法

### 自动部署

通过 GitHub Actions 工作流触发：

1. 进入 GitHub 仓库的 Actions 页面
2. 选择 "Deploy TESTNET Seed" 工作流
3. 点击 "Run workflow"
4. 输入要部署的版本标签
5. 点击 "Run workflow"

### 手动部署

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

## 优势

1. **更好的可扩展性**: Kubernetes 提供更好的资源管理和扩展能力
2. **简化的认证**: GCE self-hosted runner 可以使用默认服务账号
3. **安全性**: 镜像验证确保只部署可信的 Rooch 镜像
4. **监控**: 更好的日志和状态监控
5. **维护性**: 标准化的 Kubernetes 部署流程

## 测试状态

- ✅ 部署脚本语法验证通过
- ✅ 镜像引用验证测试通过
- ✅ 无效镜像拒绝测试通过
- ✅ GitHub Actions 工作流配置完成
- ✅ 文档和指南创建完成

## 下一步

1. 在测试环境中验证完整的部署流程
2. 配置 GCE self-hosted runner（如果选择此方案）
3. 更新相关的监控和告警配置
4. 培训团队使用新的部署流程

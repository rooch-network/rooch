# GCE Self-Hosted Runner Setup Guide

## 概述

本指南说明如何在 Google Cloud Engine (GCE) 上设置 GitHub Actions self-hosted runner，用于部署 Rooch 测试网到 Kubernetes 集群。

## 优势

使用 GCE self-hosted runner 相比 GitHub-hosted runner 的优势：

1. **简化的认证**: 可以直接使用 GCE 实例的默认服务账号，无需管理服务账号密钥
2. **更好的性能**: 在同一个 GCP 项目内，网络延迟更低
3. **成本效益**: 对于频繁的部署，可能比 GitHub-hosted runner 更经济
4. **安全性**: 服务账号密钥不需要存储在 GitHub Secrets 中

## 设置步骤

### 1. 创建 GCE 实例

```bash
# 创建用于 GitHub Actions runner 的 GCE 实例
gcloud compute instances create github-runner \
  --zone=us-central1-a \
  --machine-type=e2-standard-2 \
  --image-family=ubuntu-2004-lts \
  --image-project=ubuntu-os-cloud \
  --boot-disk-size=50GB \
  --boot-disk-type=pd-standard \
  --scopes=https://www.googleapis.com/auth/cloud-platform \
  --tags=github-runner
```

### 2. 配置服务账号权限

为 GCE 实例的默认服务账号或指定的服务账号添加以下 IAM 角色：

```bash
# 获取项目 ID
PROJECT_ID=$(gcloud config get-value project)

# 获取服务账号邮箱（使用默认计算服务账号）
SERVICE_ACCOUNT="${PROJECT_ID}-compute@developer.gserviceaccount.com"

# 添加必要的 IAM 角色
gcloud projects add-iam-policy-binding $PROJECT_ID \
  --member="serviceAccount:$SERVICE_ACCOUNT" \
  --role="roles/container.developer"

gcloud projects add-iam-policy-binding $PROJECT_ID \
  --member="serviceAccount:$SERVICE_ACCOUNT" \
  --role="roles/storage.admin"

gcloud projects add-iam-policy-binding $PROJECT_ID \
  --member="serviceAccount:$SERVICE_ACCOUNT" \
  --role="roles/container.clusterViewer"
```

### 3. 安装必要的工具

在 GCE 实例上安装必要的工具：

```bash
# 更新系统
sudo apt update && sudo apt upgrade -y

# 安装 Docker
curl -fsSL https://get.docker.com -o get-docker.sh
sudo sh get-docker.sh
sudo usermod -aG docker $USER

# 安装 Google Cloud SDK (如果未安装)
curl https://sdk.cloud.google.com | bash
exec -l $SHELL

# 安装 kubectl
curl -LO "https://dl.k8s.io/release/$(curl -L -s https://dl.k8s.io/release/stable.txt)/bin/linux/amd64/kubectl"
chmod +x kubectl
sudo mv kubectl /usr/local/bin/

# 安装 GitHub Actions runner
mkdir actions-runner && cd actions-runner
curl -o actions-runner-linux-x64-2.311.0.tar.gz -L https://github.com/actions/runner/releases/download/v2.311.0/actions-runner-linux-x64-2.311.0.tar.gz
tar xzf ./actions-runner-linux-x64-2.311.0.tar.gz
```

### 4. 配置 GitHub Actions Runner

```bash
# 在 GitHub 仓库的 Settings > Actions > Runners 中获取配置令牌
# 然后运行配置命令
./config.sh --url https://github.com/rooch-network/rooch --token <YOUR_TOKEN>

# 安装并启动 runner 服务
sudo ./svc.sh install
sudo ./svc.sh start
```

### 5. 配置 GitHub Secrets

在 GitHub 仓库的 Settings > Secrets and variables > Actions 中添加以下 secrets：

**必需的 Secrets:**
- `GCP_TESTNET_CLUSTER_NAME`: GKE 集群名称
- `GCP_TESTNET_CLUSTER_ZONE`: GKE 集群区域
- `BTC_TEST_RPC_URL`: 比特币测试网 RPC URL
- `BTC_TEST_RPC_PWD`: 比特币测试网 RPC 密码
- `OPENDA_GCP_TESTNET_BUCKET`: OpenDA GCP 存储桶
- `OPENDA_GCP_TESTNET_CREDENTIAL`: OpenDA GCP 凭证
- `TURBO_DA_TURING_ENDPOINT`: Turbo DA Turing 端点
- `TURBO_DA_TURING_API_KEY`: Turbo DA Turing API 密钥

**不再需要的 Secrets:**
- ~~`GCP_TESTNET_SA_KEY`~~: 不再需要，使用默认服务账号
- ~~`GCP_TESTNET_PROJECT_ID`~~: 可以从 GCE 实例自动获取

### 6. 测试部署

使用 GitHub Actions 的 workflow_dispatch 触发部署：

1. 进入 GitHub 仓库的 Actions 页面
2. 选择 "Deploy TESTNET Seed (GCE Optimized)" 工作流
3. 点击 "Run workflow"
4. 输入要部署的版本标签（如 `v0.10.0`）
5. 点击 "Run workflow"

## 故障排除

### 常见问题

1. **权限不足**
   ```bash
   # 检查当前服务账号
   gcloud auth list
   
   # 检查 IAM 权限
   gcloud projects get-iam-policy $PROJECT_ID
   ```

2. **kubectl 无法连接集群**
   ```bash
   # 重新配置 kubectl
   gcloud container clusters get-credentials <cluster-name> --zone <zone>
   
   # 测试连接
   kubectl get nodes
   ```

3. **Runner 离线**
   ```bash
   # 检查 runner 服务状态
   sudo systemctl status actions.runner.rooch-network-rooch.runner-*
   
   # 重启服务
   sudo ./svc.sh restart
   ```

### 监控和日志

```bash
# 查看 runner 日志
sudo journalctl -u actions.runner.rooch-network-rooch.runner-* -f

# 查看部署日志
kubectl logs -f rooch-testnet-0 -n testnet
```

## 安全最佳实践

1. **网络安全**
   - 使用防火墙规则限制实例的入站连接
   - 只允许必要的端口（SSH, HTTPS）

2. **访问控制**
   - 定期轮换服务账号密钥
   - 使用最小权限原则配置 IAM 角色

3. **监控**
   - 启用 Cloud Logging 和 Monitoring
   - 设置告警监控异常活动

## 成本优化

1. **实例类型**
   - 根据实际需求选择合适的机器类型
   - 考虑使用抢占式实例降低成本

2. **自动启停**
   - 配置自动启停脚本，在非工作时间停止实例
   - 使用 Cloud Scheduler 管理实例生命周期

3. **存储优化**
   - 使用标准持久磁盘而非 SSD
   - 定期清理不需要的 Docker 镜像和容器

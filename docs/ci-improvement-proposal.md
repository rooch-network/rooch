# CI 工作流改进方案

## 问题分析

当前 `check_build_test.yml` 工作流存在以下问题：

### 1. 性能问题
- **所有测试顺序执行**：单个巨大的 job，总时间 = 所有测试时间之和（60-90 分钟）
- **并行度不足**：Rust 测试只用 `-j 8`，在 16 核 larger runner 上没有充分利用资源
- **没有 job 级别并行化**：GitHub Actions 支持多个 jobs 并行运行，但当前只有一个 job

### 2. 可靠性问题
- **没有超时控制**：测试卡住时会无限期等待，导致 CI 永远不结束
- **缺少失败快速停止**：某个测试失败后，其他测试继续运行浪费时间
- **难以定位问题**：所有测试在一个 job 中，无法快速定位哪个测试失败

### 3. 维护问题
- **重试成本高**：一个测试失败，需要重新运行整个 job（60-90 分钟）
- **调试困难**：无法单独运行某个测试步骤进行调试

## 改进方案

### 核心策略：将单一 job 拆分为多个并行 jobs

```
当前结构：
┌─────────────────────────────────────────┐
│  check_build_test (60-90 分钟)          │
│  ├─ Build (20-30 分钟)                  │
│  ├─ Lint (10-15 分钟)                   │
│  ├─ Test Rust Unit (15-20 分钟)         │
│  ├─ Test Rust Integration (15-20 分钟)  │
│  ├─ Test Move Frameworks (10-15 分钟)   │
│  ├─ Test Move Examples (5-10 分钟)      │
│  ├─ Generate Genesis (5-10 分钟)        │
│  └─ Test SDK/Web (15-20 分钟)           │
└─────────────────────────────────────────┘
总时间：~90 分钟

改进后结构：
┌──────────────┐     ┌─────────────┐     ┌──────────────────┐
│ check_changes│ ──> │   build     │ ──> │  test-rust-unit  │ (并行)
│  (1 分钟)    │     │ (30 分钟)   │     │  test-rust-int   │ (并行)
└──────────────┘     └─────────────┘     │  lint            │ (并行)
                                           │  test-move-fw    │ (并行)
                                           │  test-move-ex    │ (并行)
                                           │  test-sdk-web    │ (并行)
                                           │  generate-gen    │ (并行)
                                           └──────────────────┘
估算时间：1 + 30 + max(20, 20, 15, 15, 10, 20, 10) = 51 分钟
```

**时间节省：约 40% (90 分钟 → 51 分钟)**

## 详细改进内容

### 1. 添加超时控制
为每个 job 添加 `timeout-minutes`：

```yaml
build:
  timeout-minutes: 60  # 构建最多 60 分钟

test-rust-unit:
  timeout-minutes: 90  # 单元测试最多 90 分钟

test-rust-integration:
  timeout-minutes: 90  # 集成测试最多 90 分钟
```

**好处**：防止测试卡住导致 CI 永远不结束

### 2. 增加并行度
充分利用 16 核 larger runner：

```yaml
# 构建阶段
cargo build --profile optci --workspace --bins -j 16

# 测试阶段
env:
  RUST_TEST_THREADS: 16
```

**好处**：加速构建和测试执行

### 3. Job 拆分和并行化
将单一 job 拆分为多个独立 jobs：

#### Phase 1: 准备阶段
- `check_changes`: 检查文件变更（1 分钟）

#### Phase 2: 构建阶段
- `build`: 构建 Rust 二进制文件（30 分钟）

#### Phase 3: 测试阶段（并行）
- `test-rust-unit`: Rust 单元测试（20 分钟）
- `test-rust-integration`: Rust 集成测试（20 分钟）
- `lint`: Rust 代码检查（15 分钟）
- `test-move-frameworks`: Move 框架测试（15 分钟）
- `test-move-examples`: Move 示例测试（10 分钟）
- `test-sdk-web`: SDK 和 Web 测试（20 分钟）
- `generate-genesis`: 生成创世文件（10 分钟）

#### Phase 4: 验证阶段
- `check-git-status`: 检查 git 状态（1 分钟）
- `validate-dockerfile-debug`: 验证 debug Dockerfile（按需）
- `validate-dockerfile`: 验证 Dockerfile（按需）
- `validate-homebrew`: 验证 Homebrew formula（按需）
- `shellcheck`: Shell 脚本检查（按需）

**好处**：
- 最慢的测试决定总时间，而不是所有测试时间之和
- 一个失败不影响其他测试
- 可以独立重试失败的测试

### 4. 失败快速停止
使用 `fail-fast` 策略（默认启用）：

```yaml
# 在矩阵或依赖关系中，一旦某个 job 失败，立即停止后续 jobs
```

**好处**：节省时间，快速反馈

### 5. 条件执行优化
使用更智能的条件判断：

```yaml
# 只在相关文件变更时运行
if: ${{ needs.check_changes.outputs.core == 'true' }}

# 只在 PR 中包含相关文件时运行
if: ${{ contains(github.event.pull_request.changed_files.*.filename, 'docker/Dockerfile') }}
```

**好处**：避免不必要的测试执行

### 6. 缓存优化
使用独立的缓存 key：

```yaml
shared-key: 'ci-build-improved'
```

**好处**：避免与旧工作流冲突，提高缓存命中率

## 实施建议

### 选项 1: 直接替换（推荐）
1. 备份当前工作流
2. 用新工作流替换 `check_build_test.yml`
3. 在 PR 中测试验证
4. 监控一周性能指标

### 选项 2: 并行运行（更安全）
1. 保留 `check_build_test.yml`，改名为 `check_build_test_legacy.yml`
2. 新工作流命名为 `check_build_test_new.yml`
3. 两个工作流并行运行一周，对比性能
4. 确认无问题后删除 legacy 版本

### 选项 3: 逐步迁移（最保守）
1. 先只拆分测试 jobs，保留单一 job 结构
2. 验证测试并行化效果
3. 再拆分构建和其他 jobs
4. 最后优化缓存和超时设置

## 成本分析

### 当前工作流
- 运行时间：~90 分钟
- 使用 runner：larger-runner (16-core)
- 成本：$0.64/分钟 × 90 分钟 = $57.60 每次
- 每天 15 次：$864/天

### 改进后工作流
- 运行时间：~51 分钟（节省 43%）
- 使用 runner：
  - build: larger-runner (30 分钟) = $19.20
  - 7 个并行 jobs (21 分钟) = $13.44 每个
  - 但实际只有 51 分钟的总时间
- 成本：$0.64/分钟 × 51 分钟 = $32.64 每次
- 每天 15 次：$489.60/天

**成本节省：约 43% ($374.40/天)**

## 风险评估

### 低风险
- ✅ 拆分 jobs：GitHub Actions 原生支持，技术成熟
- ✅ 添加超时：标准实践，不会破坏现有功能
- ✅ 增加并行度：在更大 runner 上更安全

### 中等风险
- ⚠️ 依赖关系：需要仔细管理 jobs 之间的依赖
- ⚠️ 缓存失效：新的缓存 key 可能导致首次构建较慢
- ⚠️ 条件执行：需要确保所有必要场景都被覆盖

### 缓解措施
1. 在 feature branch 上充分测试
2. 保留旧工作流作为备份（并行运行一周）
3. 监控首次运行的所有日志
4. 准备快速回滚方案

## 监控指标

迁移后需要监控的指标：

1. **性能指标**
   - 总运行时间
   - 各个 job 的运行时间
   - 缓存命中率

2. **可靠性指标**
   - 成功率
   - 超时发生次数
   - Flaky test 率

3. **成本指标**
   - 每次 CI 运行成本
   - 每日/每周总成本
   - 成本节省百分比

## 后续优化方向

1. **智能调度**
   - 根据文件变更动态调整并行度
   - 只运行受影响的测试

2. **测试分组**
   - 将慢速测试拆分到更多 jobs
   - 使用矩阵策略进一步并行化

3. **增量构建**
   - 优化构建缓存策略
   - 使用更激进的增量编译

4. **资源优化**
   - 评估是否需要所有测试都使用 larger-runner
   - 部分 jobs 可以使用标准 runner 降低成本

## 总结

这个改进方案通过**并行化**和**超时控制**，可以：
- ⏱️ **节省 43% 的运行时间**（90 分钟 → 51 分钟）
- 💰 **节省 43% 的成本**（$57.60 → $32.64 每次）
- 🛡️ **提高可靠性**（超时保护，失败快速停止）
- 🔧 **改善维护性**（独立 jobs，易于调试和重试）

建议采用**选项 2（并行运行）**进行迁移，在验证无问题后再完全切换到新工作流。

---

**文档版本**: 1.0
**创建日期**: 2026-01-07
**作者**: CI 改进提案

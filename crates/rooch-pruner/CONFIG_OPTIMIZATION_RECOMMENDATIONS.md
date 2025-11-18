# Rooch Pruner 配置参数优化建议

## 当前默认配置分析

### 现有配置参数
```rust
pub struct PruneConfig {
    pub enable: bool,                          // true
    pub boot_cleanup_done: bool,               // false
    pub scan_batch: usize,                    // 10,000
    pub delete_batch: usize,                   // 5,000
    pub interval_s: u64,                      // 60s
    pub bloom_bits: usize,                    // 2^33 = 8,589,934,592 bits ≈ 1GB
    pub enable_reach_seen_cf: bool,           // false
    pub window_days: u64,                     // 30 days
}
```

### 问题识别

#### 1. Bloom过滤器过大 🔴 严重问题
- **当前配置**：8.6GB (2^33 bits)
- **问题**：内存占用过高，可能导致OOM
- **影响**：小部署环境无法使用，启动时间长
- **建议**：根据实际节点数量动态调整

#### 2. 批处理大小不合理 🟡 中等问题
- **scan_batch**：10,000可能导致内存压力
- **delete_batch**：5,000可能过于保守
- **interval_s**：60s间隔可能过长

#### 3. 缺乏自适应机制 🟡 中等问题
- 配置是静态的，无法根据网络状态调整
- 没有考虑硬件配置差异

## 优化建议

### 方案1：基于部署规模的分层配置

#### 小型部署 (< 100万节点)
```rust
pub struct SmallDeploymentConfig {
    pub enable: true,
    pub boot_cleanup_done: false,
    pub scan_batch: 1_000,              // 减少内存压力
    pub delete_batch: 500,               // 更保守的删除
    pub interval_s: 120,                 // 降低频率减少影响
    pub bloom_bits: 8_000_000,           // 8M bits = 1MB
    pub enable_reach_seen_cf: false,
    pub window_days: 30,
}
```

#### 中型部署 (100万 - 1000万节点)
```rust
pub struct MediumDeploymentConfig {
    pub enable: true,
    pub boot_cleanup_done: false,
    pub scan_batch: 5_000,               // 平衡性能和内存
    pub delete_batch: 2_000,
    pub interval_s: 60,
    pub bloom_bits: 80_000_000,          // 80M bits = 10MB
    pub enable_reach_seen_cf: true,       // 启用cold hash spill
    pub window_days: 30,
}
```

#### 大型部署 (> 1000万节点)
```rust
pub struct LargeDeploymentConfig {
    pub enable: true,
    pub boot_cleanup_done: false,
    pub scan_batch: 10_000,              // 当前值适合大型部署
    pub delete_batch: 5_000,
    pub interval_s: 30,                  // 更频繁处理
    pub bloom_bits: 800_000_000,         // 800M bits = 100MB
    pub enable_reach_seen_cf: true,
    pub window_days: 30,
}
```

### 方案2：动态自适应配置

#### 自适应Bloom过滤器大小
```rust
fn calculate_optimal_bloom_bits(estimated_nodes: usize, available_memory_mb: usize) -> usize {
    // 基于节点数量估算所需bits
    let theoretical_bits = estimated_nodes * 8;

    // 基于可用内存限制
    let memory_limited_bits = (available_memory_mb * 1024 * 1024 * 8) / 2; // 50%内存用于bloom

    // 取较小值，但最小8M bits
    std::cmp::max(
        8_000_000,
        std::cmp::min(theoretical_bits, memory_limited_bits)
    )
}
```

#### 自适应批处理大小
```rust
fn calculate_optimal_batch_size(available_memory_mb: usize, cpu_cores: usize) -> (usize, usize) {
    let base_scan_batch = std::cmp::max(1_000, cpu_cores * 1_000);
    let memory_limited_scan = (available_memory_mb * 1024 * 1024) / 1_024; // 1KB per node

    let scan_batch = std::cmp::min(base_scan_batch, memory_limited_scan);
    let delete_batch = scan_batch / 2;

    (scan_batch, delete_batch)
}
```

#### 自适应处理间隔
```rust
fn calculate_optimal_interval(node_count: usize, system_load: f64) -> u64 {
    let base_interval = if node_count < 1_000_000 {
        120  // 小型部署：2分钟
    } else if node_count < 10_000_000 {
        60   // 中型部署：1分钟
    } else {
        30   // 大型部署：30秒
    };

    // 根据系统负载调整
    if system_load > 0.8 {
        base_interval * 2  // 高负载时降低频率
    } else if system_load < 0.3 {
        base_interval / 2  // 低负载时提高频率
    } else {
        base_interval
    }
}
```

### 方案3：配置文件模板

#### 创建配置文件模板 `pruner-config.toml`
```toml
# Pruner Configuration Template

[pruner]
enable = true
boot_cleanup_done = false
window_days = 30

# Deployment size: small, medium, large, auto
deployment_size = "auto"

# Memory configuration (in MB)
max_memory_mb = 1024

# Performance tuning
cpu_cores = 0  # 0 = auto-detect

# Advanced options
enable_reach_seen_cf = false
adaptive_config = true
monitoring_enabled = true

[pruner.limits]
max_scan_batch = 20000
min_scan_batch = 1000
max_delete_batch = 10000
min_delete_batch = 500
min_interval_s = 10
max_interval_s = 300
```

## 实现建议

### 1. 立即优化（高优先级）

#### 修复Bloom过滤器默认大小
```rust
impl Default for PruneConfig {
    fn default() -> Self {
        Self {
            enable: true,
            boot_cleanup_done: false,
            scan_batch: 1_000,        // 减少默认值
            delete_batch: 500,         // 减少默认值
            interval_s: 120,           // 增加间隔减少影响
            bloom_bits: 8_000_000,     // 1MB而不是1GB
            enable_reach_seen_cf: false,
            window_days: 30,
        }
    }
}
```

#### 添加配置验证
```rust
impl PruneConfig {
    pub fn validate(&self) -> Result<(), PrunerConfigError> {
        if self.bloom_bits < 8_000_000 {
            return Err(PrunerConfigError::BloomFilterTooSmall);
        }

        if self.bloom_bits > 1_600_000_000 {  // 200MB max
            return Err(PrunerConfigError::BloomFilterTooLarge);
        }

        if self.scan_batch < self.delete_batch {
            return Err(PrunerConfigError::InvalidBatchSize);
        }

        Ok(())
    }
}
```

### 2. 中期优化（中优先级）

#### 实现配置预设
```rust
pub enum DeploymentSize {
    Small,
    Medium,
    Large,
    Auto,
}

impl PruneConfig {
    pub fn for_deployment_size(size: DeploymentSize,
                              available_memory_mb: Option<usize>,
                              cpu_cores: Option<usize>) -> Self {
        match size {
            DeploymentSize::Small => Self::small_config(),
            DeploymentSize::Medium => Self::medium_config(),
            DeploymentSize::Large => Self::large_config(),
            DeploymentSize::Auto => Self::auto_config(available_memory_mb, cpu_cores),
        }
    }
}
```

#### 添加运行时调整能力
```rust
pub struct AdaptivePrunerConfig {
    base_config: PruneConfig,
    current_config: PruneConfig,
    metrics: PrunerMetrics,
    adjustment_history: Vec<ConfigAdjustment>,
}

impl AdaptivePrunerConfig {
    pub fn adjust_based_on_performance(&mut self, metrics: &PrunerMetrics) {
        // 基于性能指标动态调整配置
        if metrics.average_scan_time() > Duration::from_secs(30) {
            // 扫描太慢，减少批处理大小
            self.current_config.scan_batch =
                (self.current_config.scan_batch * 8) / 10;
        }

        if metrics.memory_usage() > 0.8 {
            // 内存使用过高，增加间隔
            self.current_config.interval_s =
                self.current_config.interval_s * 2;
        }
    }
}
```

### 3. 长期优化（低优先级）

#### 智能配置学习
- 基于历史性能数据学习最优配置
- 机器学习模型预测最佳参数
- A/B测试验证配置效果

#### 配置热更新
- 无需重启即可调整配置
- 渐进式配置变更
- 自动回滚机制

## 测试验证计划

### 1. 单元测试
- 各种配置参数的边界测试
- 内存使用限制验证
- 性能回归测试

### 2. 集成测试
- 不同规模部署的端到端测试
- 长期稳定性测试
- 异常场景测试

### 3. 性能基准测试
- 不同配置下的性能对比
- 内存使用效率测试
- CPU利用率测试

## 风险评估

### 配置变更风险
- **数据丢失风险**：低（配置变更不会直接影响数据）
- **性能回归风险**：中（需要充分测试）
- **兼容性风险**：低（向后兼容）

### 建议的部署策略
1. **测试环境验证**：先在测试环境充分验证
2. **灰度部署**：逐步在生产环境推广
3. **监控告警**：配置变更时的监控指标
4. **快速回滚**：必要时快速回滚到原配置

## 总结

通过分层配置、自适应机制和完善的测试验证，可以显著改善当前pruner的配置问题。重点是解决Bloom过滤器过大的问题，并提供更灵活的配置选项以适应不同规模的部署需求。

这些优化将使pruner更加：
- **内存友好**：合理的内存使用
- **性能高效**：优化的批处理大小
- **部署灵活**：适应不同规模需求
- **运维友好**：自动配置调整
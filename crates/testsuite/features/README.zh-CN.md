# Rooch 集成测试 (BDD/Cucumber) 指南

本文档面向 **Rust / Move** 开发者及 AI 伙伴，介绍 `crates/testsuite` 目录下基于 [Cucumber](https://cucumber.io/) 的集成测试框架、语法规则以及最佳实践。

> 所有示例命令均假定在仓库根目录执行，且已安装 `cargo`, `make` 等开发依赖。

---

## 1. 目录结构

```
crates/
  testsuite/
    features/        # *.feature 文件（Gherkin 语法）
      payment_channel.feature   # 支付通道相关测试
      cmd.feature               # CLI 综合测试（528 行）
      ...
    tests/
      integration.rs            # Step 定义 & 测试运行入口
```

- `*.feature`  使用 **Gherkin** 语言描述测试场景。
- `integration.rs` 借助 [`cucumber-rs`](https://github.com/cucumber-rs/cucumber) 提供的宏，将 **Given / Then** 步骤映射到 Rust 代码。

## 2. 运行测试

1. **全部集成测试**

   ```bash
   # 单线程执行，避免端口/资源冲突
   cargo test --test integration -- --test-threads=1
   ```

2. **使用 Makefile（推荐）** [[repo rules §3.1]]

   ```bash
   make test-integration
   ```

---

## 3. Gherkin 基础

```gherkin
Feature: <模块说明>
  @serial                    # 标记需要串行运行的场景
  Scenario: <场景名>
    Given a server for <id>
    Then cmd: "account create"
    Then assert: "{{$.account[-1]}} != null"
    Then stop the server
```

- **Feature / Scenario**：标准 Gherkin 概念。
- **Tag** `@serial`：告诉 Cucumber **串行** 执行该场景，防止同一时间启动多个服务冲突。
- **Given / Then**：所有步骤均由 `integration.rs` 中的宏匹配实现。

---

## 4. 步骤一览

| 类型 | 语法示例 | 说明 |
|------|----------|------|
| Given | `Given a server for payment_channel` | 在本机随机端口启动 Rooch 全节点服务。 |
| Given | `Given a bitcoind server for btc` | 通过 [testcontainers](https://github.com/testcontainers/testcontainers-rs) 启动 regtest 比特币节点。 |
| Given | `Given a ord server for inscriptions` | 启动 ord 服务（依赖 bitcoind）。 |
| Then  | `Then stop the server` | 关闭 Rooch 服务。 |
| Then  | `Then stop the bitcoind server` | 关闭 bitcoind 容器。 |
| Then  | `Then stop the ord server` | 关闭 ord 容器。 |
| Then  | `Then sleep: "5"` | 休眠 N 秒（整数）。 |
| Then  | `Then cmd: "<cli args>"` | 执行 `rooch <cli args>`，自动附带 `--config-dir`。 |
| Then  | `Then cmd bitcoin-cli: "<args>"` | 在 bitcoind 容器内执行 `bitcoin-cli -regtest <args>`。 |
| Then  | `Then cmd ord: "<args>"` | 在 ord 容器内执行 `ord --regtest ... <args>`。 |
| Then  | `Then cmd ord bash: "<bash>"` | 在 ord 容器内直接运行 bash 命令。 |
| Then  | `Then cmd bitseed: "<args>"` | 运行 Bitseed CLI（内部会启动一次性容器）。 |
| Then  | `Then assert: "<expr>"` | 断言表达式，支持多个子表达式（见 §6）。 |

> **提示**：所有 `cmd:` 步骤会尝试将 stdout 解析为 JSON，并写入 [`TemplateContext`](https://docs.rs/jpst)。键名为命令首单词，例如 `account`, `rpc`, `payment-channel` 等。

---

## 5. TemplateContext & 占位符

`integration.rs` 通过 [`jpst::TemplateContext`](https://docs.rs/jpst) 在 **每一步** 测试执行后把命令结果写入上下文，供后续步骤引用。占位符使用 `{{ ... }}` 包裹，在 **步骤解析前** 完成字符串替换。

### 5.1 占位符基本语法

| 语法 | 说明 |
|------|------|
| `{{$.<key>[<idx>]}}` | 读取名为 `<key>` 的结果数组第 `<idx>` 项；`<idx>` 可为正数或负数（负数自尾部向前计数，`-1` 表示最后一次结果）。 |
| `{{$.<key>[<idx>].<field>}}` | 继续使用“点”语法深入 **JSON 字段**。支持任意层级。 |
| `{{$.<key>[<idx>]["<field with space>"]}}` | 字段名中有空格或特殊字符时，用 `"..."` 包裹。 |
| `{{$.address_mapping.<alias>}}` | 特殊键：初始化时写入的钱包地址映射。 |

> **注意**：`$` 固定为根对象；第一层索引是 **命令名称**（去掉前缀），自动按调用顺序形成数组。

### 5.2 数组索引规则

- **正整数**：从头开始计数，`0` 是第一次调用结果。
- **负整数**：从尾部开始计数，`-1` 是最后一次，`-2` 倒数第二次。

```gherkin
{{$.rpc[0]}}       # 第一次 rpc 命令结果
{{$.rpc[-1]}}      # 最近一次 rpc 命令结果
{{$.account[-3]}}  # 倒数第三次 account 调用结果
```

### 5.3 深入 JSON

- **嵌套字段**：`{{$.payment-channel[-1].execution_info.status.type}}`
- **数组字段**：`{{$.payment-channel[-1].balances[0].amount}}`
- **动态键名**：
  ```gherkin
  # object_type 含 '<' '>' 等特殊符号，直接当作字符串键
  {{$.rpc[-1]["object_type"]}}
  ```

### 5.4 占位符转义与空格

- 占位符内 **不能** 出现未配对的括号或引号。
- 如果需要在占位符外部使用空格，应放在占位符之外：
  ```gherkin
  Then assert: "'{{$.rpc[-1].balance}}' != '0'"
  ```

### 5.5 组合示例

```gherkin
# 取最新一次 account create 返回地址作为 sender
--sender {{$.account[-1].account0.address}}

# 依赖前一步生成的 channel_id
auto_var={{$.payment-channel[1].channel_id}}

# 比较两个字段是否相等
Then assert: "{{$.rpc[-1][0].value}} == {{$.rpc[-2][0].value}}"
```

> 若占位符解析失败（键不存在 / 索引越界），`jpst` 会 panic，测试立即失败。请确保正确顺序。

---

## 6. 断言表达式

断言步骤示例：

```gherkin
Then assert: "'{{$.rpc[-1].balance}}' != '0'"
```

### 6.1 支持的操作符

| 操作符 | 说明 | 示例 |
|--------|------|------|
| `==` | 相等比较 | `"{{$.balance}} == 1000"` |
| `!=` | 不等比较 | `"{{$.status}} != error"` |
| `contains` | 字符串包含（区分大小写） | `"{{$.message}} contains success"` |
| `not_contains` | 字符串不包含（不区分大小写回退） | `"{{$.error}} not_contains timeout"` |
| `>` | 大于（数值） | `"{{$.balance}} > 0"` |
| `<` | 小于（数值） | `"{{$.gas_used}} < 1000000"` |
| `>=` | 大于等于（数值） | `"{{$.amount}} >= 100"` |
| `<=` | 小于等于（数值） | `"{{$.fee}} <= 50000"` |

### 6.2 数值比较精度

框架支持 **高精度数值比较**，无精度丢失：

- **u128/i128**：区块链金额（最大 128 位整数）直接比较，不转换为浮点数
- **f64**：支持浮点数比较，正确处理 NaN 情况
- **自动检测**：系统自动检测合适的数值类型

```gherkin
# 大数值区块链金额（无精度丢失）
Then assert: "{{$.hub_balance}} > {{$.account_balance}}"

# 浮点数比较
Then assert: "{{$.exchange_rate}} >= 1.5"
```

### 6.3 多重断言

可以在 **同一行** 写多个断言，按空格分隔，每 3 个 token 组成一条规则：

```gherkin
Then assert: "{{$.a}} == 1 {{$.b}} != 2"
```

为防止空格拆分，可使用单/双引号包裹操作数。

---

## 7. 编写新的 Feature

1. 新建 `crates/testsuite/features/<name>.feature`。
2. 标记 `@serial`（如需独占服务）。
3. **启动服务**：
   ```gherkin
   Given a server for <id>
   ```
4. **可选**：比特币/Ord 依赖。
5. **执行 CLI / RPC 命令**。推荐：
   - 为账户申请 Gas：
     ```gherkin
     Then cmd: "move run --function rooch_framework::gas_coin::faucet_entry --args u256:10000000000 --json"
     Then assert: "{{$.move[-1].execution_info.status.type}} == executed"
     ```
6. **断言业务逻辑**。善用模板变量，比对状态变化。
7. **关闭服务**：
   ```gherkin
   Then stop the server
   ```

---

## 8. payment_channel.feature 快速解读

```gherkin
Scenario: payment_channel_operations
  # 1. 启动服务
  Given a server for payment_channel_operations

  # 2. 创建账户 & 领取 Gas
  Then cmd: "account create"
  ...

  # 3. 初始化 Hub
  Then cmd: "payment-channel init --owner {{$.did[0].did}} --amount 1000000000"

  # 4. 开启子通道并查询
  Then cmd: "payment-channel open --sender ..."
  Then cmd: "payment-channel query channel --channel-id {{$.payment-channel[1].channel_id}} --list-sub-channels"

  # 5. 生成并兑现 RAV
  Then cmd: "payment-channel create-rav ..."
  Then cmd: "payment-channel claim --rav {{$.payment-channel[-1].encoded}}"

  # 6. 取消 & 完成取消流程
  Then cmd: "payment-channel cancel --channel-id ..."
  Then cmd: "move run --function 0x3::timestamp::fast_forward_seconds_for_local --args u64:86401"
  Then cmd: "payment-channel finalize-cancellation --channel-id ..."

  Then stop the server
```

- 使用 `TemplateContext` 在多步骤间传递 `channel_id` / `signed_rav` 等动态值。
- 演示了 **时间快进**（timestamp 模块）与支付通道生命周期完整闭环。

---

## 9. 高级测试模式与最佳实践

### 9.1 命令历史索引

理解 `move[-N]` 索引模式对复杂测试场景至关重要：

```gherkin
# 记录交易前余额
Then cmd: "move view --function rooch_framework::gas_coin::balance --args address:{{$.account[0].default.address}}"
Then cmd: "move view --function rooch_framework::payment_channel::get_balance_in_hub --type-args 0x3::gas_coin::RGas --args address:{{$.account[0].default.address}}"

# 执行交易
Then cmd: "move run --function rooch_framework::empty::empty --sender {{$.account[0].default.address}} --json"

# 检查交易后余额
Then cmd: "move view --function rooch_framework::gas_coin::balance --args address:{{$.account[0].default.address}}"
Then cmd: "move view --function rooch_framework::payment_channel::get_balance_in_hub --type-args 0x3::gas_coin::RGas --args address:{{$.account[0].default.address}}"

# 使用正确索引比较前后值
Then assert: "{{$.move[-4].return_values[0].decoded_value}} > {{$.move[-1].return_values[0].decoded_value}}"  # hub 余额减少
Then assert: "{{$.move[-5].return_values[0].decoded_value}} == {{$.move[-2].return_values[0].decoded_value}}"  # 账户余额不变
```

**关键理解**：`move[-1]` 指按时间顺序的**最后一个** move 命令，`move[-2]` 指倒数第二个，以此类推。

### 9.2 测试 Gas 支付来源

测试 gas 支付机制时，需考虑本地/开发环境的**自动 gas 分配**：

```gherkin
# 在本地/开发环境，用户余额为 0 时会自动获得 1000000000000 RGAS
# 所以 faucet_entry(5000000000000) 会产生约 6000000000000 总额（减去 gas 消耗）
Then cmd: "move run --function rooch_framework::gas_coin::faucet_entry --args u256:5000000000000 --sender {{$.account[0].default.address}} --json"
Then cmd: "move view --function rooch_framework::transaction_gas::total_available_gas_balance --args address:{{$.account[0].default.address}}"
Then assert: "{{$.move[-1].return_values[0].decoded_value}} > 5900000000000"  # 考虑自动分配 + gas 消耗
```

### 9.3 精确余额验证策略

要验证 gas 支付来源（hub vs 账户存储），使用此模式：

1. **记录前状态**：捕获账户和 hub 余额
2. **执行交易**：执行消耗 gas 的操作
3. **记录后状态**：再次捕获余额
4. **断言变化**：使用数值比较验证预期来源被使用

```gherkin
# 场景：验证 gas 来自 payment hub，而非账户存储
Then cmd: "move view --function rooch_framework::gas_coin::balance --args address:{{$.account[0].default.address}}"
Then cmd: "move view --function rooch_framework::payment_channel::get_balance_in_hub --type-args 0x3::gas_coin::RGas --args address:{{$.account[0].default.address}}"
Then cmd: "move run --function rooch_framework::empty::empty --sender {{$.account[0].default.address}} --json"
Then cmd: "move view --function rooch_framework::gas_coin::balance --args address:{{$.account[0].default.address}}"
Then cmd: "move view --function rooch_framework::payment_channel::get_balance_in_hub --type-args 0x3::gas_coin::RGas --args address:{{$.account[0].default.address}}"

# 验证 hub 余额减少（gas 从 hub 扣除）
Then assert: "{{$.move[-4].return_values[0].decoded_value}} > {{$.move[-1].return_values[0].decoded_value}}"
# 验证账户余额不变（gas 未从账户扣除）
Then assert: "{{$.move[-5].return_values[0].decoded_value}} == {{$.move[-2].return_values[0].decoded_value}}"
```

### 9.4 CLI 命令中的类型参数

调用泛型 Move 函数时，使用正确的类型参数语法：

```gherkin
# 正确：对泛型类型参数使用 --type-args
Then cmd: "move run --function rooch_framework::payment_channel::deposit_to_hub_entry --type-args 0x3::gas_coin::RGas --args address:{{$.account[0].default.address}} --args u256:6000000000 --sender {{$.account[0].default.address}} --json"

# 正确：view 函数也一样
Then cmd: "move view --function rooch_framework::payment_channel::get_balance_in_hub --type-args 0x3::gas_coin::RGas --args address:{{$.account[0].default.address}}"
```

### 9.5 测试混合支付场景

对于混合 gas 支付（hub + 账户存储）等复杂场景，创建专门的测试用例：

```gherkin
Scenario: gas_payment_mixed_behavior
  # 设置：创建余额不足以支付全部 gas 的 payment hub
  Then cmd: "move run --function rooch_framework::payment_channel::deposit_to_hub_entry --type-args 0x3::gas_coin::RGas --args address:{{$.account[0].default.address}} --args u256:100000 --sender {{$.account[0].default.address}} --json"
  
  # 执行需要超过 hub 余额的 gas 的交易
  # 这应该触发：hub 余额 → 0，剩余 gas 从账户存储扣除
  Then cmd: "move run --function rooch_framework::empty::empty --sender {{$.account[0].default.address}} --json"
  
  # 验证两个来源都被使用
  Then assert: "{{$.move[-4].return_values[0].decoded_value}} > {{$.move[-1].return_values[0].decoded_value}}"  # hub 减少
  Then assert: "{{$.move[-5].return_values[0].decoded_value}} > {{$.move[-2].return_values[0].decoded_value}}"  # 账户也减少
```

---

## 10. 调试技巧

1. **查看当前上下文**：在测试失败时框架会打印 `TemplateContext`，便于定位变量。
2. **定位端口占用**：`integration.rs` 会等待端口可用，若 60s 未释放会 panic。
3. **JSON 解析失败**：确认 CLI 命令是否加上 `--json`，否则输出将被当作字符串处理。
4. **索引计算错误**：当断言失败且值不符合预期时，请仔细检查 `move[-N]` 索引，按时间顺序计算命令。
5. **精度问题**：如果数值比较意外失败，请验证大整数是否被截断。框架现在支持完整的 u128 精度。
6. **模板解析失败**：使用描述性注释来记录每个 `{{$.move[-N]}}` 引用应该包含什么内容。
7. **Gas 消耗变化**：在本地/开发环境中，测试初始余额时要考虑自动 RGAS 分配（1000000000000）。

### 10.1 常见断言模式

```gherkin
# 验证交易成功
Then assert: "{{$.move[-1].execution_info.status.type}} == executed"

# 检查余额减少（gas 被消耗）
Then assert: "{{$.move[-2].return_values[0].decoded_value}} > {{$.move[-1].return_values[0].decoded_value}}"

# 验证余额不变
Then assert: "{{$.move[-2].return_values[0].decoded_value}} == {{$.move[-1].return_values[0].decoded_value}}"

# 检查非零余额
Then assert: "{{$.move[-1].return_values[0].decoded_value}} > 0"
```

---

## 11. 参考链接

- Cucumber for Rust: <https://github.com/cucumber-rs/cucumber>
- jpst 模板引擎: <https://crates.io/crates/jpst>
- Rooch 开发者文档: `docs/dev-guide/`

---

祝编写测试愉快！如果有任何改进建议，欢迎提 PR 😄

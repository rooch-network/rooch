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

- 支持的操作符： `==`, `!=`, `contains`, `not_contains`。
- 可以在 **同一行** 写多个断言，按空格分隔，每 3 个 token 组成一条规则：

```gherkin
Then assert: "{{$.a}} == 1 {{$.b}} != 2"
```

- 为防止空格拆分，可使用单/双引号包裹操作数。

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

## 9. 调试技巧

1. **查看当前上下文**：在测试失败时框架会打印 `TemplateContext`，便于定位变量。
2. **定位端口占用**：`integration.rs` 会等待端口可用，若 60s 未释放会 panic。
3. **JSON 解析失败**：确认 CLI 命令是否加上 `--json`，否则输出将被当作字符串处理。

---

## 10. 参考链接

- Cucumber for Rust: <https://github.com/cucumber-rs/cucumber>
- jpst 模板引擎: <https://crates.io/crates/jpst>
- Rooch 开发者文档: `docs/dev-guide/`

---

祝编写测试愉快！如果有任何改进建议，欢迎提 PR 😄

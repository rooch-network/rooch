## Rooch PaymentChannel 测试用例总览

> 本文档列出了 `payment_channel.move` 关键逻辑应覆盖的单元 / 集成测试场景，供编写 Move 测试脚本时参考。每个用例均应在 **rooch_framework::genesis::init_for_test()** 环境下执行，确保链上初始状态一致。

### 约定
* **Alice** —— Sender（通道发起方，需先完成 DID 注册）
* **Bob** —— Receiver（通道接收方）
* **RGas** —— 测试币种（内置 Gas 代币）
* 省略参数均使用合理缺省值；如无特别说明，所有断言均应使用 `assert!` 语义与模块中对应错误码一致。

---

### 1. PaymentHub 基础功能
| # | 场景 | 断言 |
|---|------|------|
| 1.1 | `create_payment_hub` 创建成功 | PaymentHub 对象存在，`multi_coin_store` 为空 |
| 1.2 | `deposit_to_hub_entry<CoinA>` 存入 100 | `multi_coin_store` 余额为 100 |
| 1.3 | **无活跃通道时** `withdraw_from_hub_entry<CoinA>(50)` | AccountCoinStore +50, Hub 余额剩余 50 |
| 1.4 | **有活跃通道时** 执行同上提款 | **Abort** `ErrorActiveChannelExists` |

---

### 2. Channel / Sub-Channel 生命周期
| # | 流程 | 关键断言 |
|---|------|---------|
| 2.1 | `open_channel_entry<CoinA>(Alice→Bob)` | Channel 状态 = `STATUS_ACTIVE`; PaymentHub `active_channels[CoinA]=1` |
| 2.2 | 再次调用 `open_channel_entry` | **Abort** `ErrorChannelAlreadyExists` |
| 2.3 | `open_sub_channel_entry` 首次授权 VM | SubChannel 记录存在，事件 `SubChannelOpenedEvent` 触发 |
| 2.4 | 重复授权相同 VM | **Abort** `ErrorVerificationMethodAlreadyExists` |

---

### 3. Claim & Close Sub-Channel
| # | 场景 | 金额/Nonce | 预期 |
|---|------|-----------|------|
| 3.1 | 第一次 `claim_from_channel` | (acc=10, nonce=1) | 资金从 Alice Hub → Bob Hub 10 |
| 3.2 | **重复同一笔** claim | (10,1) | idempotent 成功；增量 0；无资金转移 |
| 3.3 | **金额回退** claim | (5,2) | **Abort** `ErrorInvalidAmount` |
| 3.4 | `close_sub_channel` 最终结算 | (acc=15, nonce=3) | 额外 5 转账；SubChannel 删除；`SubChannelClosedEvent` 发出 |

---

### 4. Close Channel
| # | 流程 | 断言 |
|---|------|------|
| 4.1 | Receiver `close_channel` 提交所有 SubCloseProof | Channel 状态变 `STATUS_CLOSED`；PaymentHub `active_channels`-1 |
| 4.2 | 关闭后再次 claim / open_sub_channel | 均 **Abort** `ErrorChannelNotActive` |

---

### 5. Cancellation流程
| # | 场景 | 断言 |
|---|------|------|
| 5.1 | **无 SubChannel** initiate_cancellation | 立即 `STATUS_CLOSED`; `ChannelCancellationFinalizedEvent`；计数-1 |
| 5.2 | **有 SubChannel** initiate_cancellation | `STATUS_CANCELLING`; 保存 `cancellation_info`；事件 `ChannelCancellationInitiatedEvent` |
| 5.3 | Receiver dispute_cancellation 更新金额 ↑ | `pending_amount` 增加；事件 `ChannelDisputeEvent` |
| 5.4 | 未到挑战期 finalize_cancellation | **Abort** `ErrorChallengePeriodNotElapsed` |
| 5.5 | 到期后 finalize_cancellation | Channel 关闭；余额结算；计数-1 |

---

### 6. Channel Reactivation
| # | 流程 | 断言 |
|---|------|------|
| 6.1 | 关闭后再次调用 `open_channel` | 状态回到 `STATUS_ACTIVE`; 计数重新 +1；旧 SubChannel 表仍在 |
| 6.2 | 旧 VM 继续 claim | 功能正常

---

### 7. Withdrawal 安全性验收
1. Alice 开启通道后尝试 `withdraw_from_hub` —— 应失败。
2. 关闭所有通道后再次提款 —— 成功。

---

### 8. 事件验证（可选）
对每个流程检查对应事件是否被正确触发、字段符合预期；建议使用 `event::select` 帮助函数收集事件并断言数量/内容。

---

> **提示**：
> * 尽量在单个 Move 测试脚本中涵盖关联流程，减少重复初始化。
> * 对于需要等待挑战期的用例，可通过 `timestamp::set_time_for_testing()` 或模拟时间推进的辅助函数加速测试。
 
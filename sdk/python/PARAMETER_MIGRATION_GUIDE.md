# Rooch Python SDK 参数序列化重构指南

## 概述

Rooch Python SDK 的参数序列化系统已经重构，以解决以下问题：

1. **参数序列化与交易序列化耦合** - 旧系统将类型标签与参数值混合序列化
2. **类型推断局限性** - 无法精确指定参数类型（如区分 u64 和 u256）
3. **与 Rust/TypeScript 不兼容** - 参数格式与其他 SDK 不一致

新系统借鉴了 TypeScript SDK 和 Rust 实现的最佳实践。

## 架构变更

### 旧系统
```python
# 旧方式：使用 TransactionArgument，包含类型标签
from rooch.transactions.move.move_types import TransactionArgument, TypeTagCode

arg = TransactionArgument(TypeTagCode.U256, 1000)
# 序列化时会包含类型标签：0x0a + 32字节值
```

### 新系统
```python
# 新方式：使用 Args 类，只序列化原始值
from rooch.bcs import Args

arg = Args.u256(1000)
# 序列化时只包含原始值：32字节值（无类型标签）
```

## API 对比

### 基础类型

**旧系统：**
```python
from rooch.transactions.move.move_types import TransactionArgument, TypeTagCode

# 需要手动指定类型标签
u8_arg = TransactionArgument(TypeTagCode.U8, 255)
u256_arg = TransactionArgument(TypeTagCode.U256, 1000)
bool_arg = TransactionArgument(TypeTagCode.BOOL, True)
addr_arg = TransactionArgument(TypeTagCode.ADDRESS, "0x123...")
```

**新系统：**
```python
from rooch.bcs import Args

# 类型安全的静态方法
u8_arg = Args.u8(255)
u256_arg = Args.u256(1000)
bool_arg = Args.bool(True)
addr_arg = Args.address("0x123...")
```

### 向量类型

**旧系统：**
```python
# 复杂且容易出错
from rooch.transactions.move.move_types import TransactionArgument, TypeTagCode

# 需要手动构造向量，类型推断不准确
vector_arg = TransactionArgument(TypeTagCode.VECTOR, [1, 2, 3])  # 不清楚元素类型
```

**新系统：**
```python
from rooch.bcs import Args

# 明确的类型指定
vec_u8 = Args.vec_u8([1, 2, 3, 4, 5])
vec_u256 = Args.vec_u256([1000, 2000, 3000])
vec_bool = Args.vec_bool([True, False, True])
vec_address = Args.vec_address(["0x123...", "0x456..."])
```

### 函数调用构建

**旧系统：**
```python
from rooch.transactions.move.move_types import FunctionArgument, TransactionArgument, TypeTagCode

# 手动构造参数列表
args = [
    TransactionArgument(TypeTagCode.ADDRESS, "0x123..."),
    TransactionArgument(TypeTagCode.U256, 1000)
]

function_call = FunctionArgument(
    function_id="0x3::transfer::transfer_coin",
    ty_args=[],
    args=args
)
```

**新系统：**
```python
from rooch.bcs import MoveFunctionBuilder, Args, transfer_coin

# 方法 1：构建器模式（推荐）
function_call = (MoveFunctionBuilder("0x3::transfer::transfer_coin")
                 .add_arg(Args.address("0x123..."))
                 .add_arg(Args.u256(1000))
                 .build())

# 方法 2：便利函数
function_call = transfer_coin("0x123...", 1000)
```

## 迁移步骤

### 步骤 1：更新导入

**旧导入：**
```python
from rooch.transactions.move.move_types import TransactionArgument, TypeTagCode, FunctionArgument
```

**新导入：**
```python
from rooch.bcs import Args, MoveFunctionBuilder
# 如果需要便利函数
from rooch.bcs import transfer_coin, faucet_claim
```

### 步骤 2：重写参数创建

**旧代码：**
```python
def create_transfer_args(to_address: str, amount: int):
    return [
        TransactionArgument(TypeTagCode.ADDRESS, to_address),
        TransactionArgument(TypeTagCode.U256, amount)
    ]
```

**新代码：**
```python
def create_transfer_args(to_address: str, amount: int):
    return [
        Args.address(to_address),
        Args.u256(amount)
    ]
```

### 步骤 3：重写函数调用

**旧代码：**
```python
def build_transfer_call(to_address: str, amount: int):
    args = [
        TransactionArgument(TypeTagCode.ADDRESS, to_address),
        TransactionArgument(TypeTagCode.U256, amount)
    ]
    
    return FunctionArgument(
        function_id="0x3::transfer::transfer_coin",
        ty_args=[],
        args=args
    )
```

**新代码（推荐）：**
```python
def build_transfer_call(to_address: str, amount: int):
    return (MoveFunctionBuilder("0x3::transfer::transfer_coin")
            .add_arg(Args.address(to_address))
            .add_arg(Args.u256(amount))
            .build())

# 或者使用便利函数
def build_transfer_call(to_address: str, amount: int):
    return transfer_coin(to_address, amount)
```

## 类型安全改进

### 明确的整数类型

**问题：**
旧系统中，所有整数都被推断为 u256，无法传递其他类型。

**解决方案：**
```python
# 现在可以精确指定类型
u8_val = Args.u8(255)          # 1 字节
u16_val = Args.u16(65535)      # 2 字节
u32_val = Args.u32(4294967295) # 4 字节
u64_val = Args.u64(amount)     # 8 字节
u128_val = Args.u128(big_num)  # 16 字节
u256_val = Args.u256(huge_num) # 32 字节
```

### 运行时验证

新系统在编码时验证值的范围：

```python
try:
    Args.u8(256)  # 超出 u8 范围
except ValueError as e:
    print(f"错误：{e}")  # "u8 value must be in range 0-255, got 256"
```

### 向量类型安全

```python
# 每种向量类型都有特定方法
Args.vec_u8([1, 2, 3])        # vector<u8>
Args.vec_u64([1000, 2000])    # vector<u64>
Args.vec_u256([big1, big2])   # vector<u256>
```

## 性能改进

### 序列化输出对比

**旧系统输出（包含类型标签）：**
```
地址参数：0x04 + 32字节地址 = 33字节
u256参数：0x0a + 32字节值 = 33字节
```

**新系统输出（原始值）：**
```
地址参数：32字节地址 = 32字节
u256参数：32字节值 = 32字节
```

这与 Rust `FunctionCall.args: Vec<Vec<u8>>` 格式完全匹配。

## 向后兼容性

### 自动迁移选项

如果需要快速迁移，可以使用类型推断：

```python
from rooch.bcs import infer_and_encode, create_mixed_args

# 自动推断类型（谨慎使用）
args = create_mixed_args("0x123...", 1000, True)

# 或单个值
addr_arg = infer_and_encode("0x123...")  # 推断为地址
amount_arg = infer_and_encode(1000)      # 推断为 u256
```

**警告：** 类型推断可能不准确，建议明确指定类型。

### 渐进式迁移

可以在同一项目中混合使用新旧系统：

```python
# 新系统参数
new_args = [Args.address("0x123"), Args.u256(1000)]

# 转换为旧系统格式（如果需要）
old_args = [arg.encode() for arg in new_args]  # 获取原始字节

# 在 FunctionArgument 中使用原始字节
function_call = FunctionArgument(
    function_id="0x3::transfer::transfer_coin",
    ty_args=[],
    args=old_args  # 传递原始字节
)
```

## 最佳实践

### 1. 优先使用构建器模式

```python
# 推荐
call = (MoveFunctionBuilder("0x1::module::function")
        .add_arg(Args.u256(amount))
        .add_arg(Args.address(recipient))
        .build())
```

### 2. 为常用操作创建便利函数

```python
def swap_tokens(token_in: str, token_out: str, amount_in: int, min_amount_out: int):
    return (MoveFunctionBuilder("0x1::dex::swap")
            .add_arg(Args.address(token_in))
            .add_arg(Args.address(token_out))
            .add_arg(Args.u256(amount_in))
            .add_arg(Args.u256(min_amount_out))
            .build())
```

### 3. 明确指定类型而非依赖推断

```python
# 好
Args.u64(timestamp)

# 避免（除非确定推断正确）
infer_and_encode(timestamp)
```

### 4. 使用类型安全的向量方法

```python
# 好
Args.vec_u256([1000, 2000, 3000])

# 避免
Args.vec_u256([1000, "2000", 3000])  # 会导致类型错误
```

## 示例：完整的迁移

### 旧代码
```python
from rooch.transactions.move.move_types import TransactionArgument, TypeTagCode, FunctionArgument

def execute_transfer(client, sender, to_address: str, amount: int):
    args = [
        TransactionArgument(TypeTagCode.ADDRESS, to_address),
        TransactionArgument(TypeTagCode.U256, amount)
    ]
    
    function_call = FunctionArgument(
        function_id="0x3::transfer::transfer_coin",
        ty_args=[],
        args=args
    )
    
    # 执行交易...
```

### 新代码
```python
from rooch.bcs import MoveFunctionBuilder, Args, transfer_coin

def execute_transfer(client, sender, to_address: str, amount: int):
    # 方法 1：构建器模式
    function_call = (MoveFunctionBuilder("0x3::transfer::transfer_coin")
                     .add_arg(Args.address(to_address))
                     .add_arg(Args.u256(amount))
                     .build())
    
    # 方法 2：便利函数（推荐）
    function_call = transfer_coin(to_address, amount)
    
    # 执行交易...
```

## 常见问题

### Q: 为什么要移除类型标签？
A: 新系统匹配 Rust `FunctionCall.args: Vec<Vec<u8>>` 格式，确保跨语言兼容性。Move VM 期望原始参数字节，而非带类型标签的数据。

### Q: 如何处理复杂类型？
A: 使用 `Args.raw_bytes()` 或 `Args.from_hex()` 处理预序列化的复杂结构：

```python
complex_data = some_complex_serialization()
Args.raw_bytes(complex_data)
```

### Q: 性能影响如何？
A: 新系统更高效，因为减少了序列化开销并避免了类型标签。

### Q: 是否需要立即迁移？
A: 建议渐进式迁移。新代码使用新系统，旧代码可以逐步更新。

## 总结

重构后的参数系统提供了：

- ✅ **类型安全** - 编译时和运行时类型检查
- ✅ **明确性** - 显式类型指定，无歧义
- ✅ **兼容性** - 与 Rust/TypeScript SDK 一致
- ✅ **性能** - 更小的序列化输出
- ✅ **易用性** - 构建器模式和便利函数
- ✅ **可扩展性** - 支持新类型和复杂参数

建议在新项目中使用新系统，并逐步迁移现有代码。

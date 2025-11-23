# Test Data Generator Module

自定义 Move module，用于生成真正的 50% 数据重复，测试 Rooch Pruner 效果。

## 功能

### 1. TestObject 结构
```move
struct TestObject has key, store {
    value: u64,
    update_count: u64,
    last_update_time: u64,
    data: vector<u8>,
}
```

### 2. 主要函数

- `create_object(initial_value: u64)` - 创建新对象
- `update_object_by_id(object_id: address, new_value: u64)` - 更新已存在的对象
- `batch_create_objects(count: u64)` - 批量创建对象

## 数据重复机制

```
初始化阶段:
  创建 100 个 TestObject 作为 object pool

数据生成阶段:
  50% 操作: create_object()
    → 创建新对象（新数据，保留）
  
  50% 操作: update_object_by_id()
    → 更新 pool 中的对象
    → 同一个 object 被反复更新
    → SMT 中同一位置产生新版本
    → 旧版本成为孤儿节点（可被 pruner 清理）
```

## 预期效果

- ✅ **真正的 50% 数据重复**
- ✅ **Object 级别的覆盖更新**
- ✅ **产生可清理的旧版本节点**
- ✅ **Pruner 清理率**: 预计 15-25%（比系统表更新高）

## 使用方法

脚本会自动：
1. 部署此 module
2. 创建 100 个对象池
3. 开始 50/50 创建/更新循环

手动测试：
```bash
# 发布 module
rooch move publish --named-addresses test_data_generator=default

# 创建对象
rooch move run --function <addr>::test_object::create_object --args u64:123

# 更新对象
rooch move run --function <addr>::test_object::update_object_by_id --args address:<obj_id> u64:456
```


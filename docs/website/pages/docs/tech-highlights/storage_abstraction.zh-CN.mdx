# 存储抽象

## 动机

智能合约编程语言和传统编程语言最大的区别是智能合约编程语言需要在编程语言内部提供标准化的状态存储接口，屏蔽状态存储的底层实现，智能合约应用主要关心自己的业务逻辑。

**而“存储抽象（Storage Abstraction）”的目标是让开发者可以在智能合约中更灵活地定义自己的状态存储结构，而不局限于平台提供的标准化方案**。

我们先回顾一下当前的智能合约平台提供的方案，然后提出 Rooch 的 `Storage Abstraction` 方案。

### EVM 的方案

EVM 中，合约的状态是通过合约的字段变量来存储，虚拟机直接将合约的字段映射到存储槽（Slot）中持久化。这种方案的优点是对开发者几乎透明，开发者可以把存储当内存使用，但也有一些缺点：

1. 设计复杂的状态存储时，会遇到扩展性难题，开发者依然需要理解合约变量是如何映射到底层存储的。
2. 存储槽对外部系统访问不友好，外部系统很难直接解析存储槽中的数据。
3. 状态锁定在合约内部，链无法区分用户的状态和合约内的共享状态。这给[状态计费](https://ethresear.ch/t/state-fees-formerly-state-rent-pre-eip-proposal-version-3/4996/4)，以及在链的层面提供状态的安全保证都带来了难题。
4. 同时，也因为状态锁定在合约内部，合约之间只能进行“信息”互通，无法进行“状态”互通。
  
### Move 的方案

Move 对智能合约状态存储做了改进，应用需要通过全局存储指令进行显式操作，它主要提供以下指令：

1. `move_to<T:key>(signer)`：将 `T` 类型的资源存储在 `signer` 的用户状态空间内，这个只能通过用户自己发起的交易执行。
2. `move_from<T:key>(address):T`：将 `T` 类型的资源从用户状态空间中取出来。
3. `borrow_global<T:key>(address):&T`：从用户空间中读取 `T` 类型的的不可变引用。
4. `borrow_global_mut<T:key>(address):&mut T`：从用户空间中读取 `T` 类型的的可变引用。

以上指令都包含两个安全层面的约束：

1. `T` 类型必须是拥有 `key` 能力（Ability）的结构体（Struct）。而结构体中的字段都必须是拥有 `store` 能力的数据类型。
2. `T` 类型必须定义在当前的模块中。

第一个约束确保只有开发者明确声明具有 `store` 能力的数据类型才能写入存储层。第二个约束确保合约之间的状态安全。当前模块中数据结构的存储操作只能通过当前合约提供的方法进行，开发者可以在方法中封装访问校验逻辑，保证安全性。

这种方式带来了几个好处：

1. 明确了状态的所有权，方便底层的系统设计状态计费以及提供安全措施。
2. 类型系统是全局的，合约之间可以进行状态互通。当然，这点也依赖 Move 提供的类型安全机制。

同时，Move 提供了 `Table<K,V>` 扩展，开发者可以通过 `Table<K,V>` 自定义 Key-Value 存储。

### Sui Move 的 Object 改进

Sui Move 中废弃了上面的 Move 的全局存储指令，提供了一种对象（Object）模型。Object 是一种特殊的结构体，它需要拥有 `key` 能力，同时第一个字段必须是 `UID`。Sui Move 设计了一套所有权机制，来定义 Object 的所有权。Sui Move 这样设计主要是为了通过类似 UTXO 的模式来实现交易的并行，所以它需要客户端在交易参数中指定合约中需要操作的 Object，这样可以快速判断交易之间是否有冲突。同时，它还提供了 Object 的父子关系机制，方便开发者设计复杂的状态结构。

Sui Move 的 Object 安全约束：

1. Object 有明确的拥有者（Owner），或者是共享的（Shared）Object，虚拟机加载参数的时候需要校验 Object 的所有权。
2. 合约内的 Object 有两种方式转让所有权。一种是 `transfer<T:key>(T,address)`，这个方法只能是在定义 `T` 的当前模块中调用，需要开发者再次封装给用户使用，开发者可以自定义转让的验证逻辑。另外一种是 `public_transfer<T:key,store>(T,address)`，对象拥有者可以直接调用该方法转让 Object，但对 `T` 有额外的能力约束，必须拥有 `store` 能力。

Move 的全局存储指令都是基于用户的，获取一个资源必须先知道它存在哪个用户的空间下，然后基于该资源的类型来获取。Object 模型给 Move 的状态存储引入一种基于 ID 的存储。同时，它通过 `public_transfer` 提供了一种用户可以跳过开发者直接操作状态的能力。

### Rooch Storage Abstraction 设计原则

通过以上方案的分析，我们发现合约的存储模型主要是定义合约的执行平台、合约开发者和用户三者之间的关系。所以在 Rooch 中，借鉴以上方案的优点，提出了 `Storage Abstraction` 的概念，有以下设计原则：

1. 简单抽象原则。底层的存储接口尽量抽象简单。
2. 合约优先原则。更丰富的状态存储接口应该尽量通过合约来实现，而不依赖底层的智能合约平台的实现。
3. 自我感知（self-aware）原则。合约内部对合约的存储数据结构有感知。
4. 互操作友好原则。外部系统可以方便地访问到合约定义的数据结构，同时也可以方便地获取到存储证明。
5. 所有权明确原则。所有的存储数据所有权明确，合约的执行平台、合约开发者和用户三者之间的关系明确。

## 设计方案

![Storage Abstraction](../static/design/rooch-design-storage-abstraction.svg)

1. `RawTable` 提供最底层的 Key-Value 存储接口，合约状态变更最终都统一为 `RawTable` 的 Key-Value 变更集。
2. 基于 `RawTable` 实现 `Table<K,V>`，对 Key 和 Value 的类型进行约束。
3. 基于 `RawTable` 实现 `TypeTable`，以 Value 的类型为 Key 的存储结构。
4. 基于 `RawTable` 实现 `ObjectStorage`，提供 Object 的存储能力。
5. 基于 `TypeTable` 和 `Table` 实现 `ResourceTable` 和 `ModuleTable`，封装为 `AccountStorage`，提供 Move 的用户空间的存储接口，用于替代 Move 的全局存储指令。同时，`AccountStorage` 也提供了在合约中操作模块（Module）的接口，方便未来在合约中定义合约的升级逻辑。
6. 开发者可以基于以上存储结构，封装自己应用专用的存储接口。

### 状态树的设计

我们认为通过状态树提供状态证明，是 Web3 系统和外部系统实现互操作的重要特性，所以 Rooch 的状态存储都围绕状态树来设计。

Rooch 的状态树使用稀疏默克尔树（[Sparse Merkle Tree](https://github.com/rooch-network/smt)）实现。SMT 的两个特性对状态树非常有帮助：

1. 通过压缩优化 SMT 的中间节点，性能良好。
2. 可以同时提供包含证明和不包含证明，这点在 Rollup 场景尤其有用。

Rooch 的 StateDB 状态树整体架构如下图所示：

![statedb](../static/design/rooch-design-statedb.svg)

状态树的第一层是一个以 `ObjectID` 为键的 SMT，值是 Object 的序列化二进制数据，可以理解为一个全局的 `ObjectStore`。其中有三个特殊的 Object：

1. 以用户的 `Address` 为 Key，保存了 `AccountStorage` Object，后面会详细介绍。
2. 以 `Hash(Address + 0)` 为 Key，保存了用户的 `ResourceTable` Object。Resource Table 的 Key 为 Resource 类型，Value 为 Resource。
3. 以 `Hash(Address + 1)` 为 Key，保存了用户的 `ModuleTable` Object。Module Table 的 Key 为 Module Name，Value 为 Module 字节码。

在 Rooch 中，Move 中的每个 Table 对应一个 SMT，Table Object 主要存储了 SMT 的 Root。加载 Table 的数据的时候，首先通过 Table Object ID 获取到 SMT 的 Root，然后初始化 SMT，通过 Key 获取 Value。而 Table 也可以成为 Struct 的字段，Struct 中序列化的是 Table 的 Object ID（TableHandle）。这样，Rooch 的状态树就是一个层层嵌套的 SMT。

### StorageContext

StorageContext 包含两个字段，一个是 `TxContext` 包含当前交易相关的信息，另外一个是 `ObjectStore`，对应第一层的状态树。

```move
module moveos_std::storage_context{
    struct StorageContext {
        tx_context: TxContext,
        object_storage: ObjectStorage,
    }
}
```

开发者可以在 `entry` 方法中定义 StorageContext 参数，MoveVM 会自动填充该参数。

```move
module example::my_module{
    public entry fun my_entry_fun(ctx: &mut StorageContext){
        //function logic
    }
}
```

### RawTable

Rooch 中，给 Move 实现了一种叫做 RawTable 的扩展，它基于原来的 Table 扩展改造，取消了对 Key 和 Value 的约束。

以下是 RawTable 在 Move 中提供的几个关键接口：

```move
module moveos_std::raw_table{
    public(friend) fun add<K: copy + drop, V>(table_handle: &ObjectID, key: K, val: V);

    public(friend) fun borrow<K: copy + drop, V>(table_handle: &ObjectID, key: K);

    public(friend) fun borrow_mut<K: copy + drop, V>(table_handle: &ObjectID, key: K): &mut V;

    public(friend) fun remove<K: copy + drop, V>(table_handle: &ObjectID, key: K): V;

    public(friend) fun contains<K: copy + drop, V>(table_handle: &ObjectID, key: K): bool;
}
```

TableHandle 即 TableObject 的 ObjectID，RawTable 的上层数据结构拿到 TableHandle 后，可以直接操作任意类型的 K 和 V，相当于一个最底层的 KV 存储接口。每个 RawTable 对应 StateDB 中的一颗 SMT，Global ObjectStore 的 ObjectID 是所有字节都为`0`的一个特殊 ObjectID。

当前 RawTable 中的方法都是 `friend`，开发者无法直接操作，需要通过 RawTable 上层封装的数据结构进行操作。比如 `Table<K,V>`，`TypeTable`，`ObjectStorage` 。

### Rooch Object

Rooch 中的 Object 采用一种箱子（Box）模式的 Object。它相当于一个带有全局唯一 ID 的箱子，可以将类型 `T` 封装为一个 Object。

```move
module moveos_std::object{
    struct Object<T: key>{
        id: ObjectID,
        owner: address,
        value: T,
    }
}
```

Rooch Object 也是 Move 中的 [Hot Potato](https://examples.sui.io/patterns/hot-potato.html) 模式的一个使用案例。Object 不具有任何 `ability`，所以它不可以被 `drop`，`copy`，`store`，创建之后只能被 ObjectStorage 的接口处理。

这样设计的好处是不需要对 Move 的 Struct 定义做额外的要求，也不需要额外的 Native 方法支持，Object 可以直接用 Move 来定义。ObjectStorage 是在用户的存储空间之外的一个以全局唯一 ID 为 Key 的存储空间，解决应用中需要全局存储的场景。

### ObjectStorage

ObjectStorage 提供了 Object 全局存储相关的接口，它基于 RawTable 实现，定义如下：

```move
module moveos_std::object_storage{
    #[private_generics(T)]
    /// Borrow Object from object store with object_id
    public fun borrow<T: key>(this: &ObjectStorage, object_id: ObjectID): &Object<T>;

    #[private_generics(T)]
    /// Borrow mut Object from object store with object_id
    public fun borrow_mut<T: key>(this: &mut ObjectStorage, object_id: ObjectID): &mut Object<T>;

    #[private_generics(T)]
    /// Remove object from object store
    public fun remove<T: key>(this: &mut ObjectStorage, object_id: ObjectID): Object<T>;

    #[private_generics(T)]
    /// Add object to object store
    public fun add<T: key>(this: &mut ObjectStorage, obj: Object<T>);

    #[private_generics(T)]
    public fun contains<T: key>(this: &ObjectStorage, object_id: ObjectID): bool;
}
```

以上方法都通过 [private_generics](https://github.com/rooch-network/rooch/issues/64) 来保证只有 `T` 所在的 Module 才能直接调用以上方法，安全模型上遵循 Move 全局存储指令相关的约束。

>TBD: 是否提供一种允许 Owner 直接操作 Object 的方法，类似于 Sui 的 `public_transfer`，需要进一步研究。

### TypeTable

TypeTable 是一种特殊的 Table，它模拟 Move 全局存储指令，以类型为 Key，类型的实例为 Value。基于 RawTable 实现，提供以下 API：

```move
module moveos_std::type_table {

    #[private_generics(V)]
    public fun add<V: key>(table: &mut TypeTable, val: V);

    #[private_generics(V)]
    public fun borrow<V: key>(table: &TypeTable): &V;

    #[private_generics(V)]
    public fun borrow_mut<V: key>(table: &mut TypeTable): &mut V;

    #[private_generics(V)]
    public fun remove<V: key>(table: &mut TypeTable): V;

    #[private_generics(V)]
    public fun contains<V: key>(table: &TypeTable): bool;
}
```

TypeTable 的方法需要依赖 `private_generics` 的类型安全保证，和 Move 全局存储指令一样的约束。

### AccountStorage

AccountStorage 是用户存储空间在 Move 中的抽象，它包含两个 Table，Resource Table 和 Module Table，这样可以在 Move 中直接操作 Resource 和 Module，而不需要通过全局存储指令。

它主要提供以下 API：

```move
module moveos_std::account_storage{

    struct AccountStorage has key {
        resources: TypeTable,
        modules: Table<String, MoveModule>,
    }

    #[private_generics(T)]
    /// Borrow a resource from the account's storage
    /// This function equates to `borrow_global<T>(address)` instruction in Move
    public fun global_borrow<T: key>(ctx: &StorageContext, account: address): &T;

    #[private_generics(T)]
    /// Borrow a mut resource from the account's storage
    /// This function equates to `borrow_global_mut<T>(address)` instruction in Move
    public fun global_borrow_mut<T: key>(ctx: &mut StorageContext, account: address): &mut T;

    #[private_generics(T)]
    /// Move a resource to the account's storage
    /// This function equates to `move_to<T>(&signer, resource)` instruction in Move
    public fun global_move_to<T: key>(ctx: &mut StorageContext, account: &signer, resource: T);

    #[private_generics(T)]
    /// Move a resource from the account's storage
    /// This function equates to `move_from<T>(address)` instruction in Move
    public fun global_move_from<T: key>(ctx: &mut StorageContext, account: address): T;

    #[private_generics(T)]
    /// Check if the account has a resource of the given type
    /// This function equates to `exists<T>(address)` instruction in Move
    public fun global_exists<T: key>(ctx: &StorageContext, account: address) : bool;

    /// Check if the account has a module with the given name
    public fun exists_module(ctx: &StorageContext, account: address, name: String): bool;

    /// Publish modules to the account's storage
    public fun publish_modules(ctx: &mut StorageContext, account: &signer, modules: vector<MoveModule>);
}
```

以上方法可以替代 Move 的全局存储指令，让 Move 的存储相关的操作，最终都统一为对 Table 的操作。

同时，这种方式还解决了一个 Move 的引用难题，开发者无法在自己的方法中返回通过 `borrow_global<T>` 指令借用的引用，因为这个引用是凭空创造的，不能作为方法的返回值。而如果通过 `StorageContext` 借用，则可以达到这个目标，该引用的生命周期和 `StorageContext` 的生命周期绑定。

另外，AccountStorage 也提供了 module 相关的方法，这样可以很容易实现在 Move 中部署 Move 合约，方便开发者通过合约来定义合约部署规则，比如通过 DAO 治理升级合约。

> TBD: 是否需要完全废弃 Move 的全局存储指令，还是同时提供两种方式，需要进一步的研究来决定。

### 统一的状态访问路径接口

Rooch 的 StateDB 是一个层层嵌套的 SMT，所以我们可以提供一种统一的[访问路径接口](https://github.com/rooch-network/rooch/issues/58)。

`/table/$table_handle/$key1,$key2`: 访问 `$table_handle` 的 Table 下的 `$key1,$key2` 的数据。如果 $table_handle 是 `0x0`，则表明访问的是第一层的 SMT 的数据。

同时还提供几个别名访问方式：

* `/object/$object_id`：访问第一层的 SMT 的快捷方法，等价于 `/table/0x0/$object_id`。
* `/module/$address/$module_name`：访问某个用户的 AccountStorage 里的某个 Module，等价于 `/table/NamedTable($address,resource)/hex($module_name)`。
* `/resource/$address/$resource_struct_tag`：访问某个用户的 AccountStorage 里的某个 Resource，等价于 `/table/NamedTable($address,module)/hex($resource_struct_tag)`。

由于 SMT 存储的是 Move 的 Struct 序列化值，外部系统可以直接反序列化成 JSON 或者其他编程语言中的数据结构，对开发者友好。

## 总结

Rooch 将应用对存储的需求抽象为 KV 存储接口，统一了应用状态的处理方式，简化了虚拟机层的状态的存储逻辑，也为扩展新的存储空间提供了扩展能力。然后，基于 KV 接口，在合约中封装出更丰富安全的存储空间接口，满足应用的不同场景的存储需求。

而 Rooch 的状态树以及统一访问接口，方便外部系统和 Rooch 的组合，保证了互操作性。

最后，关于执行平台、合约开发者和用户三者之间的关系，Rooch 延续了 Move 的安全约束，状态所有权明确，但对状态的操作需要通过开发者定义的接口约束，同时也在探索用户直接操作自己的状态的方案。

`Storage Abstraction` 是一种新的理念，随着应用场景的丰富，对存储方式也会有新的需求。未来有以下探索方向：

1. 提供更丰富的面向具体应用场景的存储解决方案。
2. 扩展 SMT 的抽象，理论上任何能提供状态证明的存储结构，都可以映射到 Move 中作为一种 Table，比如各种链的状态树，Offchain 的 Git。

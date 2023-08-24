---
title: 一文读懂私有泛型函数
description: ""
author: geometryolife
category: Technology
date: 2023/08/23
---

import PostHeader from "/components/blog/postHeader";

<PostHeader />

## 什么是私有泛型函数

私有泛型函数是专门为 Rooch 设计的一种泛型函数，它相比于一般的泛型函数，拥有更严格的约束。

简单来说，私有泛型函数就是拥有 `#[private_generics(T1 [, T2, ... Tn])]` 属性标注的泛型函数。

## 作用

私有泛型函数可以判断泛型类型是否是**调用者所在的模块内**定义的，如果不是则无法使用。对于一般的泛型函数，其他模块定义的类型，通过 `use` 语句导入到当前模块后，仍然能够正常使用。

这意味着，私有泛型函数只适用于**自定义类型**，对于其他模块定义的类型和内置类型均无法使用。

## 例子

### 私有泛型函数不接受内置类型

我们先来看一个不使用私有泛型函数的例子。

```move
module rooch_examples::module1 {
    struct Box<T> has drop {
        v: T
    }

    public fun new_box<T1, T2, T3>(value: T1): Box<T1> {
        Box { v: value }
    }

    public fun get_box_value<T: copy>(box: &Box<T>): T {
        box.v
    }

    #[test]
    fun test1() {
        let box = new_box<u32, u64, u128>(123);
        assert!(get_box_value(&box) == 123, 1000);
    }
}
```

首先定义一个 `Box<T>` 类型，它是一个泛型结构体，包含了一个类型为 `T` 的字段 `v`。

接着定义两个泛型函数 `new_box` 和 `get_box_value`。函数 `new_box` 用来创建 `Box<T>` 类型的值，函数 `get_box_value` 用来获取 `Box<T>` 类型中的字段值 `v: T`。

我们简单地写一个单元测试来验证我们的代码逻辑。我们给 `new_box` 传递一个整数字面量 `123`，并创建一个包裹了 `u32` 类型的 `Box<u32>` 值 `box`。在断言表达式中，整数字面量 `123` 会隐式推断为 `123u32`，`get_box_value` 从 `box` 中获取到 `123u32`，两者相等，能够顺利通过测试。

运行单元测试：

```shell
$ rooch move test
INCLUDING DEPENDENCY MoveStdlib
INCLUDING DEPENDENCY MoveosStdlib
BUILDING pri_generic
Running Move unit tests
[ PASS    ] 0x42::module1::test1
Test result: OK. Total tests: 1; passed: 1; failed: 0
Success
```

接下来我们对泛型函数 `new_box` 进行**私有泛型约束**，在函数的上一行添加 `#[private_generics(T1, T2)]` 属性标注，其他地方保持不变：

```shell
#[private_generics(T1, T2)]
public fun new_box<T1, T2, T3>(value: T1): Box<T1> {
    Box { v: value }
}
```

在添加了私有泛型约束后，在调用函数时，类型 `T1` 和 `T2` 必须在当前模块内定义。显然**内置类型** `u32` 和 `u64` 并不是在当前模块定义的，所以此时再运行代码，就会报错，如下：

```shell
$ rooch move test
error: resource type "U32" in function "0x42::module1::new_box" not defined in current module or not allowed
   ┌─ ./sources/module1.move:17:19
   │
17 │         let box = new_box<u32, u64, u128>(123);
   │                   ^^^^^^^^^^^^^^^^^^^^^^^^^^^^

error: resource type "U64" in function "0x42::module1::new_box" not defined in current module or not allowed
   ┌─ ./sources/module1.move:17:19
   │
17 │         let box = new_box<u32, u64, u128>(123);
   │                   ^^^^^^^^^^^^^^^^^^^^^^^^^^^^


Error: extended checks failed
```

### 私有泛型函数使用当前模块定义的自定义类型

基于上面的代码，我们稍作修改：

```move
module rooch_examples::module1 {
    struct Data has copy, drop {
        v: u64
    }

    struct Box<T> has drop {
        v: T
    }

    #[private_generics(T1, T2)]
    public fun new_box<T1, T2, T3>(value: T1): Box<T1> {
        Box { v: value }
    }

    public fun get_box_value<T: copy>(box: &Box<T>): T {
        box.v
    }

    #[test]
    fun test1() {
        let data = Data { v: 123 };
        let box = new_box<Data, Data, u128>(data);
        assert!(get_box_value(&box).v == 123, 1000);
    }
}
```

首先，新增一个自定义类型 `Data`，它是一个常见的结构体，包含一个 `u64` 类型的字段 `v`。

接着修改一下单元测试的函数 `test1`，构造一个 `Data` 类型的数据 `data`，并将私有泛型函数 `new_box` 的泛型类型参数 `u32` 和 `u64` 修改为 `Data`，断言中，使用成员运算符 `.` 获取 `data` 内的 `u64` 整数值进行比较。

此时再次运行测试，测试例子又能顺利通过测试了：

```shell
$ rooch move test
INCLUDING DEPENDENCY MoveStdlib
INCLUDING DEPENDENCY MoveosStdlib
BUILDING pri_generic
Running Move unit tests
[ PASS    ] 0x42::module1::test1
Test result: OK. Total tests: 1; passed: 1; failed: 0
Success
```

私有泛型函数是在 `module1` 调用的，自定义类型 `Data` 也是在**当前**模块内定义的，因此 `Data` 能通过函数的私有泛型约束。

接下来我们将演示在另一个模块定义一个自定义类型，并通过 `use` 语句将这个自定义类型导入到当前模块，并将其传递给私有泛型函数。猜猜看它是否能同正常使用？

### 私有泛型函数不接受其他模块定义的自定义类型

我们定义一个名为 `module2` 的新模块：

```move
module rooch_examples::module2 {
    struct Data2 has copy, drop {
        v: u64
    }

    public fun new_data(value: u64): Data2 {
        Data2 {
            v: value
        }
    }
}
```

在这个模块里，我们定义一个新的类型 `Data2`，并定义一个创建这个类型的函数 `new_data`。因为在 Move 中，所有的结构体只能在声明它们的模块中构造，字段只能在结构体的模块内部访问，所以要想被外部模块构造，必须要有相应的函数。

接下来，我们继续修改 `module1`：

```move
module rooch_examples::module1 {
    #[test_only]
    use rooch_examples::module2::{new_data, Data2};

    struct Data has copy, drop {
        v: u64
    }

    struct Box<T> has drop {
        v: T
    }

    #[private_generics(T1, T2)]
    public fun new_box<T1, T2, T3>(value: T1): Box<T1> {
        Box { v: value }
    }

    public fun get_box_value<T: copy>(box: &Box<T>): T {
        box.v
    }

    #[test]
    fun test1() {
        let data = Data { v: 123 };
        let box = new_box<Data, Data, u128>(data);
        assert!(get_box_value(&box).v == 123, 1000);
    }

    #[test]
    fun test2() {
        let data2 = new_data(789);
        let box2 = new_box<Data2, Data2, Data2>(data2);
        assert!(get_box_value(&box2) == new_data(789), 2000)
    }
}
```

我们导入 `module2` 定义的类型和函数，然后创建单元测试 `test2`。

测试中我们调用 `new_data` 函数创建 `module2::Data2` 类型的值 `data2`，并在将私有泛型函数的类型参数修改为 `Data2`。

注意，这个时候调用私有泛型函数所传递的类型参数 `Data2` 是在外部模块定义的，显然无法通过私有泛型约束！

此时，运行单元测试：

```shell
$ rooch move test
error: resource type "Data2" in function "0x42::module1::new_box" not defined in current module or not allowed
   ┌─ ./sources/module1.move:32:20
   │
32 │         let box2 = new_box<Data2, Data2, Data2>(data2);
   │                    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^


Error: extended checks failed
```


## 泛型函数

在了解如何使用私有泛型函数之前，我们先来回顾一下如何使用泛型函数。

```move
module rooch_examples::test {
    struct Data has copy, drop {
        v: u64
    }

    // Define a Box type
    struct Box<T> has drop {
        v: T
    }

    // Create an instance of type Data
    fun new_data(value: u64): Data {
        Data { v: value }
    }

    // Create an instance of type Box<T>
    public fun new_box<T>(value: T): Box<T> {
        Box { v: value }
    }

    // Get the value inside the Box<T>
    public fun get_box_value<T: copy>(box: &Box<T>): T {
        box.v
    }

    // Unit Test:
    #[test]
    fun test() {
        let data = new_data(123);
        let box = new_box<Data>(data);
        assert!(get_box_value(&box).v == 123, 1000);
    }
}
```

首先我们定义两个自定义类型 `Data` 和 `Box<T>`。

`Data` 是常见的结构体，包含一个 `u64` 类型的字段 `v`。`Box<T>` 是一个泛型结构体，包含了一个类型为 `T` 的字段 `v`。

接着我们定义函数 `new_data` 用来创建 `Data` 类型的值，定义泛型函数 `new_box` 用来创建 `Box<T>` 类型的值，再定义泛型函数 `get_box_value` 用来获取 `Box<T>` 类型中的字段值 `v: T`。

最后我们编写一个简单的单元测试，测试上面定义的代码是否工作正常。这是一个很简单的流程，将 `123` 放进 `Data`，然后使用 `Box<T>` 包装起来，在断言表达式中判是否能一层层将 `123` 取出来，如果获取失败，则以中断码 `1000`，返回错误结果。

终端运行 `rooch move test` 命令执行单元测试：

```shell
rooch move test

INCLUDING DEPENDENCY MoveStdlib
INCLUDING DEPENDENCY MoveosStdlib
BUILDING pri_generic
Running Move unit tests
[ PASS    ] 0x42::test::test
Test result: OK. Total tests: 1; passed: 1; failed: 0
Success
```

## 私有泛型函数的使用

接下来我们开始修改我们的例子，通过私有泛型标注将泛型函数 `new_box` 变成一个私有泛型函数。

```shell
```

<++>

## 使用泛型函数

## 总结



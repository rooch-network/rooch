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

<++>

## 私有泛型函数的应用

## 使用泛型函数


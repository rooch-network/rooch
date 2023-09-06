---
title: 一文读懂 Rooch 对象
description: ""
author: geometryolife
category: Technology
date: 2023/09/05
---

import PostHeader from "/components/blog/postHeader";

<PostHeader />

## 情景

我们假设一个学生对象拥有一些学生所具备的一些基本属性，比如姓名、学号、年龄。

学生除了一些基本属性外，还有一些与之关联的一些属性信息，比如成绩。

我们接下来会以此为基础，来编写一个 Move 合约来模拟对学生实体、学生属性和学生关联信息相关的一些操作。

## 目标

帮助开发人员了解何时使用 Object 和 ObjectStorage，以及何时使用 AccountStorage。帮助开发人员学习 Object 和 ObjectStorage 功能。

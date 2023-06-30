# Getting started with Rooch

## 1. 什么是 Rooch

[Rooch](https://rooch.network) 是一个快速、模块化、安全、开发人员友好的基础架构解决方案，用于构建 Web3 原生应用程序。

Rooch 于2023年06月28日，发布了第一个版本，版本名为 Sprout，版本号为 v0.1。

## 2. 创建新的 Rooch 项目

这部分将引导你如何安装 Rooch，以及创建 Hello World 程序。

### 2.1 安装 Rooch

#### 2.1.1 下载

```shell
wget https://github.com/rooch-network/rooch/releases/download/v0.1-preview/rooch-ubuntu-latest.zip
```

#### 2.1.2 解压

```shell
unzip rooch-ubuntu-latest.zip
```

解压文件存放在 `rooch-artifacts` 目录里，`rooch` 是我们预编译好的二进制程序。

```shell
rooch-artifacts
├── README.md
└── rooch
```

#### 2.1.3 运行

```shell
./rooch
```

你将看到下面的输出内容，说明程序一切正常。

```shell
rooch 0.1.0
Rooch Contributors <opensource@rooch.network>

USAGE:
    rooch <SUBCOMMAND>

OPTIONS:
    -h, --help       Print help information
    -V, --version    Print version information

SUBCOMMANDS:
    account
    event
    help           Print this message or the help of the given subcommand(s)
    init
    move           CLI frontend for the Move compiler and VM
    object
    resource
    server
    state          Get States by AccessPath
    transaction
```

#### 2.1.4 加入 PATH

为了方便后续使用，建议将 `rooch` 放入能被系统环境变量 `PATH` 检索的路径，或者将当前的解压目录通过 `export` 导出到 `PATH` 中。


### 2.2 Hello Rooch

#### 2.2.1 初始化 Rooch 配置

```shell
rooch init
```

#### 2.2.2 创建一个 Move 项目

使用 `rooch` 封装的 `move new` 命令来创建一个名为 `rooch_counter` 的计数器应用。

```shell
rooch move new rooch_counter
```

生成的 Move 项目里包含一个配置文件 `Move.toml` 和一个用于存放 Move 源代码的 `sources` 目录。

```shell
rooch_counter
├── Move.toml
└── sources
```

我们可以简单看一下 `Move.toml` 文件包含了哪些内容。

```toml
[package]
name = "rooch_counter"
version = "0.0.1"

[dependencies]
MoveStdlib = { git = "https://github.com/rooch-network/rooch.git", subdir = "moveos/moveos-stdlib/move-stdlib", rev = "main" }
MoveosStdlib = { git = "https://github.com/rooch-network/rooch.git", subdir = "moveos/moveos-stdlib/moveos-stdlib", rev = "main" }
RoochFramework = { git = "https://github.com/rooch-network/rooch.git", subdir = "crates/rooch-framework", rev = "main" }

[addresses]
rooch_counter =  "0x1b13ecd47456f506874dbf60a9c4856f7a38782463f28f2506b8006a8f3f8064"
std =  "0x1"
moveos_std =  "0x2"
rooch_framework =  "0x3"
```

- 在 TOML 文件中包含三个表：`package`、`dependencies` 和 `addresses`，存放项目所需的一些元信息。
- `package` 表用来存放项目的一些描述信息，这里包含两个键值对 `name` 和 `version` 来描述项目名和项目的版本号。
- `dependencies` 表用来存放项目所需依赖的元数据。
- `addresses` 表用来存放项目地址以及模块地址，第一个地址是初始化 Rooch 配置时，生成在 `$HOME/.rooch/rooch_config/rooch.yaml` 中的地址。

#### 2.2.3 构建合约代码

在 `sources` 目录里创建一个名为 `rooch_counter.move` 的合约文件，就

```move
module rooch_counter::counter {
   use moveos_std::account_storage;
   use moveos_std::storage_context::{StorageContext};

   struct Counter has key, store {
      value:u64,
   }

   fun init(ctx: &mut StorageContext, account: &signer){
      account_storage::global_move_to(ctx, account, Counter{value:0});
   }

   public fun increase_(ctx: &mut StorageContext) {
      let counter = account_storage::global_borrow_mut<Counter>(ctx, @rooch_counter);
      counter.value = counter.value + 1;
   }

   public entry fun increase(ctx: &mut StorageContext) {
      Self::increase_(ctx)
   }

   public fun value(ctx: &StorageContext): u64 {
      let counter = account_storage::global_borrow<Counter>(ctx, @rooch_counter);
      counter.value
   }
}
```

<++>

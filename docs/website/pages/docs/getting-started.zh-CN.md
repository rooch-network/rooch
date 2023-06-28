# Getting started with Rooch

## 什么是 Rooch


## 创建新的 Rooch 项目

这部分将引导你如何安装 Rooch，以及创建 Hello World 程序。

### 1. 安装 Rooch

- 下载

```shell
wget https://github.com/rooch-network/rooch/releases/download/v0.1-preview/rooch-ubuntu-latest.zip
```

- 解压

```shell
unzip rooch-ubuntu-latest.zip
```

解压文件存放在 `rooch-artifacts` 目录里，`rooch` 是我们预编译好的二进制程序。

```shell
rooch-artifacts
├── README.md
└── rooch
```

- 运行

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

- 加入 PATH

为了方便后续使用，建议将 `rooch` 放入能被系统环境变量 `PATH` 检索的路径，或者将当前的解压目录通过 `export` 导出到 `PATH` 中。


### 2. Hello Rooch

- 初始化 Rooch 配置

```shell
rooch init
```

- 创建一个 Move 项目

```shell
move move new hello_rooch
```

生成的 Move 项目里包含一个配置文件 `Move.toml` 和一个用于存放 Move 源代码的 `sources` 目录。

```shell
hello_rooch
├── Move.toml
└── sources
```

我们可以简单看一下 `Move.toml` 文件包含了哪些内容。

```toml
[package]
name = "hello_rooch"
version = "0.0.1"

[dependencies]
MoveStdlib = { git = "https://github.com/rooch-network/rooch.git", subdir = "moveos/moveos-stdlib/move-stdlib", rev = "main" }
MoveosStdlib = { git = "https://github.com/rooch-network/rooch.git", subdir = "moveos/moveos-stdlib/moveos-stdlib", rev = "main" }
RoochFramework = { git = "https://github.com/rooch-network/rooch.git", subdir = "crates/rooch-framework", rev = "main" }

[addresses]
hello_rooch =  "0x1b13ecd47456f506874dbf60a9c4856f7a38782463f28f2506b8006a8f3f8064"
std =  "0x1"
moveos_std =  "0x2"
rooch_framework =  "0x3"
```

  - 分

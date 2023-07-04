# Rooch 新手入门

## 1. 什么是 Rooch

[Rooch](https://rooch.network) 是一个快速、模块化、安全、开发人员友好的基础架构解决方案，用于构建 Web3 原生应用程序。

Rooch 于2023年06月28日，发布了第一个版本，版本名为 Sprout，版本号为 v0.1。

## 2. 创建新的 Rooch 项目

这部分将引导你安装 Rooch，以及创建一个博客合约，在 Rooch 里体验基本的**增查改删**功能。

### 2.1 安装 Rooch

#### 2.1.1 下载

在 [Rooch 的 GitHub 发布页面](https://github.com/rooch-network/rooch/releases)可以找到预构建的二进制文件压缩包和相应版本的源码压缩包。如果想要体验 Git 版本，可以参考这个页面来[编译安装 Rooch](https://github.com/rooch-network/rooch#getting-started)，下面引导你安装 Rooch 的 Release 版本。

```shell
wget https://github.com/rooch-network/rooch/releases/download/v0.1/rooch-ubuntu-latest.zip
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

进入解压文件夹 `rooch-artifacts` 并测试程序是否正常。

```shell
cd rooch-artifacts
./rooch
```

如果你能看到下面的输出内容，说明程序一切正常。

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

使用下面这段小脚本将 `rooch` 添加到 Bash shell 的 `PATH`。

```shell
echo "export PATH=\$PATH:$PWD" >> ~/.bashrc
source ~/.bashrc
```

### 2.2 初始化 Rooch 配置

```shell
rooch init
```

运行初始化配置的命令后，会在用户的主目录（`$HOME`）创建一个 `.rooch` 目录，并生成 Rooch 账户的相关配置信息。

### 2.3 创建博客合约应用

#### 2.3.1 创建 Move 项目

使用 `rooch` 封装的 `move new` 命令来创建一个名为 `rooch_blog` 的博客应用。

```shell
rooch move new rooch_blog
```

生成的 Move 项目里包含一个配置文件 `Move.toml` 和一个用于存放 Move 源代码的 `sources` 目录。

```shell
rooch_blog
├── Move.toml
└── sources
```

我们可以简单看一下 `Move.toml` 文件包含了哪些内容。

```toml
[package]
name = "rooch_blog"
version = "0.0.1"

[dependencies]
MoveStdlib = { git = "https://github.com/rooch-network/rooch.git", subdir = "moveos/moveos-stdlib/move-stdlib", rev = "main" }
MoveosStdlib = { git = "https://github.com/rooch-network/rooch.git", subdir = "moveos/moveos-stdlib/moveos-stdlib", rev = "main" }
RoochFramework = { git = "https://github.com/rooch-network/rooch.git", subdir = "crates/rooch-framework", rev = "main" }

[addresses]
rooch_blog =  "0x36a1c5014cb1771fb0689e041875c83a31675693301a9ba233932abc0b7e68dc"
std =  "0x1"
moveos_std =  "0x2"
rooch_framework =  "0x3"
```

- 在 TOML 文件中包含三个表：`package`、`dependencies` 和 `addresses`，存放项目所需的一些元信息。
- `package` 表用来存放项目的一些描述信息，这里包含两个键值对 `name` 和 `version` 来描述项目名和项目的版本号。
- `dependencies` 表用来存放项目所需依赖的元数据。
- `addresses` 表用来存放项目地址以及模块地址，第一个地址是初始化 Rooch 配置时，生成在 `$HOME/.rooch/rooch_config/rooch.yaml` 中的地址。

#### 2.3.2 编写合约

[下载博客源码](https://github.com/rooch-network/rooch/archive/refs/heads/main.zip)，解压，并切换到博客合约项目的根目录。

```shell
wget https://github.com/rooch-network/rooch/archive/refs/heads/main.zip
unzip main.zip
cd rooch-main/docs/website/public/codes/rooch_blog
```

#### 2.3.3 编译合约

```shell
rooch move build
```

编译结束后，如果没有错误，会在最后看到 `Success` 的提示信息。

```shell
UPDATING GIT DEPENDENCY https://github.com/rooch-network/rooch.git
UPDATING GIT DEPENDENCY https://github.com/rooch-network/rooch.git
UPDATING GIT DEPENDENCY https://github.com/rooch-network/rooch.git
UPDATING GIT DEPENDENCY https://github.com/rooch-network/rooch.git
UPDATING GIT DEPENDENCY https://github.com/rooch-network/rooch.git
UPDATING GIT DEPENDENCY https://github.com/rooch-network/rooch.git
INCLUDING DEPENDENCY MoveStdlib
INCLUDING DEPENDENCY MoveosStdlib
INCLUDING DEPENDENCY RoochFramework
BUILDING rooch_blog
Success
```

此时，项目文件夹会多出一个 `build` 目录，里面存放的就是 Move 编译器生成的合约字节码文件以及合约的完整代码。

#### 2.3.4 运行 Rooch 服务器

我们再打开另外一个终端，运行下面这条命令，Rooch 服务器会在本地启动 Rooch 服务，用于处理并响应合约的相关行为。

```shell
rooch server start
```

当启动 Rooch 服务后，会在最后看到这两条信息，表明 Rooch 的服务已经正常启动。

```shell
2023-07-03T15:44:33.315228Z  INFO rooch_rpc_server: JSON-RPC HTTP Server start listening 0.0.0.0:50051
2023-07-03T15:44:33.315256Z  INFO rooch_rpc_server: Available JSON-RPC methods : ["wallet_accounts", "eth_blockNumber", "eth_getBalance", "eth_gasPrice", "net_version", "eth_getTransactionCount", "eth_sendTransaction", "rooch_sendRawTransaction", "rooch_getAnnotatedStates", "eth_sendRawTransaction", "rooch_getTransactionByIndex", "rooch_executeRawTransaction", "rooch_getEventsByEventHandle", "rooch_getTransactionByHash", "rooch_executeViewFunction", "eth_getBlockByNumber", "rooch_getEvents", "eth_feeHistory", "eth_getTransactionByHash", "eth_getBlockByHash", "eth_getTransactionReceipt", "rooch_getTransactionInfosByTxOrder", "eth_estimateGas", "eth_chainId", "rooch_getTransactionInfosByTxHash", "wallet_sign", "rooch_getStates"]
```

> 提示：我们在前一个终端窗口操作合约相关的逻辑（功能）时，可以观察这个终端窗口的输出。

#### 2.3.5 发布 Move 合约

```shell
rooch move publish
```

当你看到类似这样的输出（`status` 为 `executed`），就可以确认发布操作已经成功执行了：

```shell
{
  //...
  "execution_info": {
    //...
    "status": {
      "type": "executed"
    }
  },
  //...
}
```

在运行 Rooch 服务的终端也可以看到响应的处理信息：

```shell
2023-07-03T16:02:11.691028Z  INFO connection{remote_addr=127.0.0.1:58770 conn_id=0}: jsonrpsee_server::server: Accepting new connection 1/100
2023-07-03T16:02:13.690733Z  INFO rooch_proposer::actor::proposer: [ProposeBlock] block_number: 0, batch_size: 1
```

### 2.4 与博客合约交互

我们接下来将会使用 Rooch CLI 以及其他命令行工具（`curl`、`jq`）来测试已发布的合约。

使用 `rooch move run` 命令提交一个交易，初始化合约（请注意替换占位符 `{ACCOUNT_ADDRESS}` 为你拥有账户的地址）：

```shell
rooch move run --function {ACCOUNT_ADDRESS}::rooch_blog_demo_init::initialize --sender-account {ACCOUNT_ADDRESS}
```

我们可以查看 `$HOME/.rooch/rooch_config/rooch.yaml` 文件中的 `active_address` 这个键对应的值，即操作合约的默认账户地址。

我的地址为 `0x36a1c5014cb1771fb0689e041875c83a31675693301a9ba233932abc0b7e68dc`，后续将继续使用这个地址来演示相关操作。

```shell
rooch move run --function 0x36a1c5014cb1771fb0689e041875c83a31675693301a9ba233932abc0b7e68dc::rooch_blog_demo_init::initialize --sender-account 0x36a1c5014cb1771fb0689e041875c83a31675693301a9ba233932abc0b7e68dc
```

#### 2.4.1 创建文章

可以像下面这样，使用 Rooch CLI 提交一个交易，创建一篇测试文章：

```shell
rooch move run --function 0x36a1c5014cb1771fb0689e041875c83a31675693301a9ba233932abc0b7e68dc::article_aggregate::create --sender-account 0x36a1c5014cb1771fb0689e041875c83a31675693301a9ba233932abc0b7e68dc --args 'string:Hello Rooch' "string:Accelerating World's Transition to Decentralization"
```

`--function` 指定执行发布在 `0x36a1c5014cb1771fb0689e041875c83a31675693301a9ba233932abc0b7e68dc` 地址上的 `article_aggregate` 模块中的 `create` 函数，即新建一篇博客文章。`--sender-account` 指定谁来提交这个交易。这个函数要求我们必须给它传递两个参数，通过 `--args` 来指定，第一个是文章的标题，我取名为 `Hello Rooch`；第二个是文章的内容，我写上 Rooch 的标语（slogan）：`Accelerating World's Transition to Decentralization`。

参数传递的是字符串，需要使用引号将内容包裹起来，并且通过 `string:` 来显示指定，在第二个参数的内容中有单引号，所以使用双引号包裹，否则必须使用转义符（`\`）。

你可以随意更换 `--args` 后面的第一个参数（`title`）和第二个参数（`body`）的内容，多创建几篇文章。

#### 2.4.2 查询文章

现在，你可以通过查询事件，得到已创建好的文章的 `ObjectID`：

```shell
curl --location --request POST 'http://localhost:50051' \
--header 'Content-Type: application/json' \
--data-raw '{
 "id":101,
 "jsonrpc":"2.0",
 "method":"rooch_getEventsByEventHandle",
 "params":["0x36a1c5014cb1771fb0689e041875c83a31675693301a9ba233932abc0b7e68dc::article::ArticleCreated", null, 1000]
}'
```

返回的响应内容：

```shell
{"jsonrpc":"2.0","result":{"data":[{"event":{"event_id":{"event_handle_id":"0xf73d11468373bfcb25c0f6cc283f127a8dc8074da8bd9ba1ffe1c6f59c835404","event_seq":0},"type_tag":"0x36a1c5014cb1771fb0689e041875c83a31675693301a9ba233932abc0b7e68dc::article::ArticleCreated","event_data":"0x0190ba9f94b397111c779ab18647d5305c0c42843c33622f029da9093254b4f84b0b48656c6c6f20526f6f636833416363656c65726174696e6720576f726c642773205472616e736974696f6e20746f20446563656e7472616c697a6174696f6e","event_index":0},"sender":"0x0000000000000000000000000000000000000000000000000000000000000000","tx_hash":null,"timestamp_ms":null,"parsed_event_data":{"abilities":8,"type":"0x36a1c5014cb1771fb0689e041875c83a31675693301a9ba233932abc0b7e68dc::article::ArticleCreated","value":{"body":"Accelerating World's Transition to Decentralization","id":{"abilities":7,"type":"0x1::option::Option<0x2::object_id::ObjectID>","value":{"vec":["0x90ba9f94b397111c779ab18647d5305c0c42843c33622f029da9093254b4f84b"]}},"title":"Hello Rooch"}}}],"next_cursor":0,"has_next_page":false},"id":101}
```

由于输出的内容比较多，可以在上面的命令最尾添加一个管道操作（` | jq '.result.data[0].parsed_event_data.value.id.value.vec[0]'`），来快速筛选出第一篇文章的 `ObjectID`。 

> 提示：在使用 `jp` 命令（jq - commandline JSON processor）之前，你可能需要先安装它。

添加 `jp` 处理后的命令像下面这样：

```shell
curl --location --request POST 'http://localhost:50051' \
--header 'Content-Type: application/json' \
--data-raw '{
 "id":101,
 "jsonrpc":"2.0",
 "method":"rooch_getEventsByEventHandle",
 "params":["0x36a1c5014cb1771fb0689e041875c83a31675693301a9ba233932abc0b7e68dc::article::ArticleCreated", null, 1000]
}' | jq '.result.data[0].parsed_event_data.value.id.value.vec[0]'
```

通过 `jp` 来筛选出的博客的对象 ID 为：

```shell
"0x90ba9f94b397111c779ab18647d5305c0c42843c33622f029da9093254b4f84b"
```

然后，你可以使用 Rooch CLI 来查询对象的状态，通过 `--id` 来指定文章对象的 ID（注意替换为你的文章的 ObjectID）：

```shell
rooch object --id 0x90ba9f94b397111c779ab18647d5305c0c42843c33622f029da9093254b4f84b
```

```shell
[joe@mx rooch_blog]$ rooch object --id 0x90ba9f94b397111c779ab18647d5305c0c42843c33622f029da9093254b4f84b
{
  "state": {
    "value": "0x90ba9f94b397111c779ab18647d5305c0c42843c33622f029da9093254b4f84b36a1c5014cb1771fb0689e041875c83a31675693301a9ba233932abc0b7e68dc00000000000000000b48656c6c6f20526f6f636833416363656c65726174696e6720576f726c642773205472616e736974696f6e20746f20446563656e7472616c697a6174696f6efd1a25121453bfa91136bb7c089142f6a1aeb5d6ea13f23c238eade83f3ad31d",
    "value_type": "0x2::object::Object<0x36a1c5014cb1771fb0689e041875c83a31675693301a9ba233932abc0b7e68dc::article::Article>"
  },
  "move_value": {
    "abilities": 0,
    "type": "0x2::object::Object<0x36a1c5014cb1771fb0689e041875c83a31675693301a9ba233932abc0b7e68dc::article::Article>",
    "value": {
      "id": "0x90ba9f94b397111c779ab18647d5305c0c42843c33622f029da9093254b4f84b",
      "owner": "0x36a1c5014cb1771fb0689e041875c83a31675693301a9ba233932abc0b7e68dc",
      "value": {
        "abilities": 8,
        "type": "0x36a1c5014cb1771fb0689e041875c83a31675693301a9ba233932abc0b7e68dc::article::Article",
        "value": {
          "body": "Accelerating World's Transition to Decentralization",
          "comments": {
            "abilities": 4,
            "type": "0x2::table::Table<u64, 0x36a1c5014cb1771fb0689e041875c83a31675693301a9ba233932abc0b7e68dc::comment::Comment>",
            "value": {
              "handle": "0xfd1a25121453bfa91136bb7c089142f6a1aeb5d6ea13f23c238eade83f3ad31d"
            }
          },
          "title": "Hello Rooch",
          "version": "0"
        }
      }
    }
  }
}
```

注意观察输出中，`title` 和 `body` 这两个键值对，能看到这个对象确实“存储着”刚刚写的那篇博客文章。

#### 2.4.3 更新文章

`--function` 指定执行发布在 `0x36a1c5014cb1771fb0689e041875c83a31675693301a9ba233932abc0b7e68dc` 地址上的 `article_aggregate` 模块中的 `update` 函数，即更新一篇博客文章。同样也需要使用 `--sender-account` 来指定发送这个更新文章交易的账户。这个函数要求我们必须给它传递三个参数，通过 `--args` 来指定，第一个参数是要修改文章的对象 ID，后面的两个参数分别对应文章的标题和正文。

将文章的标题修改为 `Foo`，文章正文修改为 `Bar`：

```shell
rooch move run --function 0x36a1c5014cb1771fb0689e041875c83a31675693301a9ba233932abc0b7e68dc::article_aggregate::update --sender-account 0x36a1c5014cb1771fb0689e041875c83a31675693301a9ba233932abc0b7e68dc --args 'object_id:0x90ba9f94b397111c779ab18647d5305c0c42843c33622f029da9093254b4f84b' 'string:Foo' 'string:Bar'
```

除了使用 Rooch CLI，你还可以通过调用 JSON RPC 来查询对象的状态：

```shell
curl --location --request POST 'http://127.0.0.1:50051/' \
--header 'Content-Type: application/json' \
--data-raw '{
 "id":101,
 "jsonrpc":"2.0",
 "method":"rooch_getAnnotatedStates",
 "params":["/object/0x90ba9f94b397111c779ab18647d5305c0c42843c33622f029da9093254b4f84b"]
}'
```

在输出中，可以观察到文章的标题和正文已成功修改：

```shell
{"jsonrpc":"2.0","result":[{"state":{"value":"0x90ba9f94b397111c779ab18647d5305c0c42843c33622f029da9093254b4f84b36a1c5014cb1771fb0689e041875c83a31675693301a9ba233932abc0b7e68dc010000000000000003466f6f03426172fd1a25121453bfa91136bb7c089142f6a1aeb5d6ea13f23c238eade83f3ad31d","value_type":"0x2::object::Object<0x36a1c5014cb1771fb0689e041875c83a31675693301a9ba233932abc0b7e68dc::article::Article>"},"move_value":{"abilities":0,"type":"0x2::object::Object<0x36a1c5014cb1771fb0689e041875c83a31675693301a9ba233932abc0b7e68dc::article::Article>","value":{"id":"0x90ba9f94b397111c779ab18647d5305c0c42843c33622f029da9093254b4f84b","owner":"0x36a1c5014cb1771fb0689e041875c83a31675693301a9ba233932abc0b7e68dc","value":{"abilities":8,"type":"0x36a1c5014cb1771fb0689e041875c83a31675693301a9ba233932abc0b7e68dc::article::Article","value":{"body":"Bar","comments":{"abilities":4,"type":"0x2::table::Table<u64, 0x36a1c5014cb1771fb0689e041875c83a31675693301a9ba233932abc0b7e68dc::comment::Comment>","value":{"handle":"0xfd1a25121453bfa91136bb7c089142f6a1aeb5d6ea13f23c238eade83f3ad31d"}},"title":"Foo","version":"1"}}}}}],"id":101}[joe@mx rooch_blog]$
```

#### 2.4.4 删除文章

可以这样提交一个交易，删除文章：

```shell
rooch move run --function 0x36a1c5014cb1771fb0689e041875c83a31675693301a9ba233932abc0b7e68dc::article_aggregate::delete --sender-account 0x36a1c5014cb1771fb0689e041875c83a31675693301a9ba233932abc0b7e68dc --args 'object_id:0x90ba9f94b397111c779ab18647d5305c0c42843c33622f029da9093254b4f84b'
```

`--function` 指定执行发布在 `0x36a1c5014cb1771fb0689e041875c83a31675693301a9ba233932abc0b7e68dc` 地址上的 `article_aggregate` 模块中的 `delete` 函数，即删除一篇博客文章。同样也需要使用 `--sender-account` 来指定发送这个删除文章交易的账户。这个函数只需给它传递一个参数，即文章对应的对象 ID，通过 `--args` 来指定。

#### 2.4.5 检查文章是否正常删除

```shell
rooch object --id 0x90ba9f94b397111c779ab18647d5305c0c42843c33622f029da9093254b4f84b

null
```

可以看到，查询文章的对象 ID 时，结果反回 `null`。说明文章已经被删除了。

## 3. 总结

到这里，相信你已经对 Rooch v1.0 如何运行，如何发布合约，以及如何跟合约交互有了基本的了解。想要在 Rooch 上体验更多的合约例子，请参见 [`rooch/examples`](https://github.com/rooch-network/rooch/tree/main/examples)。

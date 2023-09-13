# Rooch 新手入门

## 1. 什么是 Rooch

[Rooch](https://rooch.network) 是一个快速、模块化、安全、开发人员友好的基础架构解决方案，用于构建 Web3 原生应用程序。

Rooch 于2023年06月28日，发布了第一个版本，版本名为 **萌芽（Sprouting）**，版本号为 `v0.1`。

## 2. 安装 Rooch

### 2.1 下载

在 [Rooch 的 GitHub 发布页面](https://github.com/rooch-network/rooch/releases)可以找到预构建的二进制文件压缩包和相应版本的源码压缩包。如果想要体验 Git 版本，可以参考这个页面来[编译安装 Rooch](https://github.com/rooch-network/rooch#getting-started)，下面引导你安装 Rooch 的 Release 版本。

```shell
wget https://github.com/rooch-network/rooch/releases/download/v0.1/rooch-ubuntu-latest.zip
```

> 注意：请选择对应自己系统的版本，我这里使用 Linux 的版本来演示。

### 2.2 解压

```shell
unzip rooch-ubuntu-latest.zip
```

解压文件存放在 `rooch-artifacts` 目录里，`rooch` 是我们预编译好的二进制程序。

```shell
rooch-artifacts
├── README.md
└── rooch
```

### 2.3 运行

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
    account        Tool for interacting with accounts
    event          Tool for interacting with event
    help           Print this message or the help of the given subcommand(s)
    init           Tool for init with rooch
    move           CLI frontend for the Move compiler and VM
    object         Get object by object id
    resource       Get account resource by tag
    server         Start Rooch network
    state          Get states by accessPath
    transaction    Tool for interacting with transaction
```

### 2.4 加入 PATH

为了方便后续使用，建议将 `rooch` 放入能被系统环境变量 `PATH` 检索的路径，或者将当前的解压目录通过 `export` 导出到 `PATH` 中。

- 方法一，复制 `rooch` 这个程序复制到 `/usr/local/bin` 目录中（**推荐**）

```shell
sudo cp rooch /usr/local/bin
```

- 方法二，导出路径（不推荐）

使用下面这段小脚本将 `rooch` 添加到 Bash shell 的 `PATH`。

```shell
echo "export PATH=\$PATH:$PWD" >> ~/.bashrc
source ~/.bashrc
```

## 3. 初始化 Rooch 配置

```shell
rooch init
```

运行初始化配置的命令后，会在用户的主目录（`$HOME`）创建一个 `.rooch` 目录，并生成 Rooch 账户的相关配置信息。

## 4. 创建新的 Rooch 项目

这部分将引导你如何在 Rooch 上创建一个博客的合约应用，并实现基本的**增查改删（CRUD）**功能。

### 4.1 创建 Move 项目

使用 `rooch` 集成的 `move new` 命令来创建一个名为 `simple_blog` 的博客应用。

```shell
rooch move new simple_blog
```

生成的 Move 项目里包含一个配置文件 `Move.toml` 和一个用于存放 Move 源代码的 `sources` 目录。

```shell
simple_blog
├── Move.toml
└── sources
```

我们可以简单看一下 `Move.toml` 文件包含了哪些内容。

```toml
[package]
name = "simple_blog"
version = "0.0.1"

[dependencies]
MoveStdlib = { git = "https://github.com/rooch-network/rooch.git", subdir = "moveos/moveos-stdlib/move-stdlib", rev = "main" }
MoveosStdlib = { git = "https://github.com/rooch-network/rooch.git", subdir = "moveos/moveos-stdlib/moveos-stdlib", rev = "main" }
RoochFramework = { git = "https://github.com/rooch-network/rooch.git", subdir = "crates/rooch-framework", rev = "main" }

[addresses]
simple_blog =  "_"
std =  "0x1"
moveos_std =  "0x2"
rooch_framework =  "0x3"
```

- 在 TOML 文件中包含三个表：`package`、`dependencies` 和 `addresses`，存放项目所需的一些元信息。
- `package` 表用来存放项目的一些描述信息，这里包含两个键值对 `name` 和 `version` 来描述项目名和项目的版本号。
- `dependencies` 表用来存放项目所需依赖的元数据。
- `addresses` 表用来存放项目地址以及项目所依赖模块的地址，第一个地址是初始化 Rooch 配置时，生成在 `$HOME/.rooch/rooch_config/rooch.yaml` 中的地址。

为了方便其他开发者部署，我们把 `simple_blog` 的地址用 `_` 替代，然后部署的时候通过 `--named--addresses` 来指定。

### 4.2 快速体验

这小节里，将引导你编写一个博客的初始化函数，并在 Rooch 中运行起来，体验`编写 -> 编译 -> 发布 -> 调用`合约这样一个基本流程。

我们在 `sources` 目录里新建一个 `blog.move` 文件，并开始编写我们的博客合约。

#### 4.2.1 定义博客的结构

我们的博客系统允许每个人创建自己的博客，并保存自己的博客列表。首先，我们定义一个博客结构体：

```move
struct MyBlog has key {
    name: String,
    articles: vector<ObjectID>,
}
```

这个结构体包含两个字段，一个是博客的名字，另一个是博客的文章列表。文章列表我们只保存文章 Object 的 ID。

然后定义一个创建博客的函数：

```move
public fun create_blog(ctx: &mut StorageContext, owner: &signer) {
    let articles = vector::empty();
    let myblog = MyBlog{
        name: string::utf8(b"MyBlog"),
        articles,
    };
    account_storage::global_move_to(ctx, owner, myblog);
}

public entry fun set_blog_name(ctx: &mut StorageContext, owner: &signer, blog_name: String) {
    assert!(std::string::length(&blog_name) <= 200, error::invalid_argument(EDATA_TOO_LONG));
    let owner_address = signer::address_of(owner);
    // if blog not exist, create it
    if(!account_storage::global_exists<MyBlog>(ctx, owner_address)){
        create_blog(ctx, owner);
    };
    let myblog = account_storage::global_borrow_mut<MyBlog>(ctx, owner_address);
    myblog.name = blog_name;
}
```

创建博客就是初始化 `MyBlog` 数据结构，并把 `MyBlog` 保存在用户的存储空间内。同时提供了一个设置博客名称的入口函数，如果博客不存在，则先创建博客，然后设置博客名称。

然后再提供一个合约初始化函数，合约发布的时候会自动执行这个初始化函数，给发布合约的用户先自动初始化博客。

```move
/// This init function is called when the module is published
/// The owner is the address of the account that publishes the module
fun init(storage_ctx: &mut StorageContext, owner: &signer) {
    // auto create blog for module publisher 
    create_blog(storage_ctx, owner);
}
```

然后，再提供一个查询博客列表的函数和添加删除文章的函数，全部代码如下：

```move
module simple_blog::blog {
    use std::error;
    use std::signer;
    use std::string::{Self,String};
    use std::vector;
    use moveos_std::object_id::ObjectID;
    use moveos_std::storage_context::StorageContext;
    use moveos_std::account_storage;

    const EDATA_TOO_LONG: u64 = 1;
    const ENOT_FOUND: u64 = 2;

    struct MyBlog has key {
        name: String,
        articles: vector<ObjectID>,
    }

    /// This init function is called when the module is published
    /// The owner is the address of the account that publishes the module
    fun init(storage_ctx: &mut StorageContext, owner: &signer) {
        // auto create blog for module publisher 
        create_blog(storage_ctx, owner);
    }

    public fun create_blog(ctx: &mut StorageContext, owner: &signer) {
        let articles = vector::empty();
        let myblog = MyBlog{
            name: string::utf8(b"MyBlog"),
            articles,
        };
        account_storage::global_move_to(ctx, owner, myblog);
    }

    public entry fun set_blog_name(ctx: &mut StorageContext, owner: &signer, blog_name: String) {
        assert!(std::string::length(&blog_name) <= 200, error::invalid_argument(EDATA_TOO_LONG));
        let owner_address = signer::address_of(owner);
        // if blog not exist, create it
        if(!account_storage::global_exists<MyBlog>(ctx, owner_address)){
            create_blog(ctx, owner);
        };
        let myblog = account_storage::global_borrow_mut<MyBlog>(ctx, owner_address);
        myblog.name = blog_name;
    }

    fun add_article_to_myblog(ctx: &mut StorageContext, owner: &signer, article_id: ObjectID) {
        let owner_address = signer::address_of(owner);
        // if blog not exist, create it
        if(!account_storage::global_exists<MyBlog>(ctx, owner_address)){
            create_blog(ctx, owner);
        };
        let myblog = account_storage::global_borrow_mut<MyBlog>(ctx, owner_address);
        vector::push_back(&mut myblog.articles, article_id);
    }

    fun delete_article_from_myblog(ctx: &mut StorageContext, owner: &signer, article_id: ObjectID) {
        let owner_address = signer::address_of(owner);
        let myblog = account_storage::global_borrow_mut<MyBlog>(ctx, owner_address);
        let (contains, index) = vector::index_of(&myblog.articles, &article_id);
        assert!(contains, error::not_found(ENOT_FOUND));
        vector::remove(&mut myblog.articles, index); 
    }

    /// Get owner's blog's articles
    public fun get_blog_articles(ctx: &StorageContext, owner_address: address): vector<ObjectID> {
        if(!account_storage::global_exists<MyBlog>(ctx, owner_address)){
            vector::empty()
        }else{
            let myblog = account_storage::global_borrow<MyBlog>(ctx, owner_address);
            myblog.articles
        }
    }
}
```

- `module simple_blog::blog` 用来声明我们的合约属于哪个模块，它的语法是 `module 地址::模块名`，花括号 `{}` 里编写的就是合约的逻辑（功能）。
- `use` 语句导入我们编写合约时需要依赖的库。
- `const` 定义合约中使用的常量，通常用来定义一些错误代码。
- `fun` 是用来定义函数的关键字，通常在这里定义函数的功能。为了安全，这类函数禁止直接在命令行中调用，需要在入口函数中封装调用逻辑。
- `entry fun` 用来定义入口函数，`entry` 修饰符修饰的函数可以像脚本一样直接在命令行中调用。

#### 4.2.2 编译 Move 合约

在发布合约前，需要编译我们的合约。这里通过 `--named-addresses` 来指定 `simple_blog` 模块的地址为当前设备上的 `default` 地址。  

```shell
rooch move build --named-addresses simple_blog=default
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
BUILDING simple_blog
Success
```

此时，项目文件夹会多出一个 `build` 目录，里面存放的就是 Move 编译器生成的合约字节码文件以及合约**完整的**源代码。

#### 4.2.3 运行 Rooch 服务器

我们再打开另外一个终端，运行下面这条命令，Rooch 服务器会在本地启动 Rooch 容器服务，用于处理并响应合约的相关行为。

```shell
rooch server start
```

当启动 Rooch 服务后，会在最后看到这两条信息，表明 Rooch 的服务已经正常启动。

```shell
2023-07-03T15:44:33.315228Z  INFO rooch_rpc_server: JSON-RPC HTTP Server start listening 0.0.0.0:50051
2023-07-03T15:44:33.315256Z  INFO rooch_rpc_server: Available JSON-RPC methods : ["wallet_accounts", "eth_blockNumber", "eth_getBalance", "eth_gasPrice", "net_version", "eth_getTransactionCount", "eth_sendTransaction", "rooch_sendRawTransaction", "rooch_getAnnotatedStates", "eth_sendRawTransaction", "rooch_getTransactions", "rooch_executeRawTransaction", "rooch_getEventsByEventHandle", "rooch_getTransactionByHash", "rooch_executeViewFunction", "eth_getBlockByNumber", "rooch_getEvents", "eth_feeHistory", "eth_getTransactionByHash", "eth_getBlockByHash", "eth_getTransactionReceipt", "rooch_getTransactionInfosByTxOrder", "eth_estimateGas", "eth_chainId", "rooch_getTransactionInfosByTxHash", "wallet_sign", "rooch_getStates"]
```

> 提示：我们在前一个终端窗口操作合约相关的逻辑（功能）时，可以观察这个终端窗口的输出。

#### 4.2.4 发布 Move 合约

```shell
rooch move publish --named-addresses simple_blog=default
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

#### 4.2.5 调用 Move 合约

此时，我们的博客合约已经发布到链上了，并且默认账户下已经初始化了博客。我们可以用状态查询命令来查看该账户下的博客 Resource：

```shell
rooch state --access-path /resource/{ACCOUNT_ADDRESS}/{RESOURCE_TYPE}
```

其中，`{ACCOUNT_ADDRESS}` 是账户地址，`{RESOURCE_TYPE}` 是资源类型，这里是 `{MODULE_ADDRESS}::blog::MyBlog`。这里 `{ACCOUNT_ADDRESS}` 和 `{MODULE_ADDRESS}` 都是我本机的默认账户地址。

我们可以查看 `$HOME/.rooch/rooch_config/rooch.yaml` 文件中的 `active_address` 这个键对应的值，即操作合约的默认账户地址。

我的地址为 `0xbbfc33692c7d57839fde9643681fb64c83b377e4c70b1e4b76aa35ff1e410d01`，后续将继续使用这个地址来演示相关操作。

所以我这里实际执行的命令应该是：

```shell
rooch state --access-path /resource/0xbbfc33692c7d57839fde9643681fb64c83b377e4c70b1e4b76aa35ff1e410d01/0xbbfc33692c7d57839fde9643681fb64c83b377e4c70b1e4b76aa35ff1e410d01::blog::MyBlog
```

返回结果：

```json
[
  {
    "state": {
      "value": "0x064d79426c6f6700",
      "value_type": "0xbbfc33692c7d57839fde9643681fb64c83b377e4c70b1e4b76aa35ff1e410d01::blog::MyBlog"
    },
    "move_value": {
      "abilities": 8,
      "type": "0xbbfc33692c7d57839fde9643681fb64c83b377e4c70b1e4b76aa35ff1e410d01::blog::MyBlog",
      "value": {
        "articles": [],
        "name": "MyBlog"
      }
    }
  }
]
```

可以看到，`MyBlog` Resource 已经存在，名称是默认的 `MyBlog`，文章列表为空。

然后我们通过 `set_blog_name` 函数来设置博客名。调用合约入口函数的语法是：

```shell
rooch move run --function {ACCOUNT_ADDRESS}::{MODULE_NAME}::{FUNCTION_NAME} --sender-account {ACCOUNT_ADDRESS}
```

使用 `rooch move run` 命令运行一个函数。`--function` 指定函数名，需要传递一个完整的函数名，即`发布合约的地址::模块名::函数名`，才能够准确识别需要调用的函数。`--sender-account` 指定调用这个函数的账户地址，即使用哪个账户来调用这个函数，任何人都可以调用链上的合约。

```shell
rooch move run --function 0xbbfc33692c7d57839fde9643681fb64c83b377e4c70b1e4b76aa35ff1e410d01::blog::set_blog_name --sender-account default --args 'string:Rooch blog'
```

这条命令执行时，会向链发送一笔交易，交易的内容就是就是调用博客合约中的 `set_blog_name` 函数。

执行成功后，再次运行前面的状态查询命令，查看博客 Resource：

```json
[
  {
    "state": {
      "value": "0x0a526f6f636820626c6f6700",
      "value_type": "0xbbfc33692c7d57839fde9643681fb64c83b377e4c70b1e4b76aa35ff1e410d01::blog::MyBlog"
    },
    "move_value": {
      "abilities": 8,
      "type": "0xbbfc33692c7d57839fde9643681fb64c83b377e4c70b1e4b76aa35ff1e410d01::blog::MyBlog",
      "value": {
        "articles": [],
        "name": "Rooch blog"
      }
    }
  }
]
```

可以看到，博客名已经修改成功。至此，我们体验了在 Rooch 中从零到一地安装，初始化配置，创建项目，编写合约，编译合约，发布合约，调用合约。

### 4.3 完善博客合约

接下来我们继续完善博客合约，增加博客文章的**增查改删**功能。

#### 4.3.1 创建博客文章合约

我们在 `sources` 目录再创建一个 `article.move` 文件，这个文件中存放文章的数据类型以及对文章进行 CRUD 操作的相关事件的定义。

定义文章数据类型，以及文章事件类型：

```move
struct Article has key {
    version: u64,
    title: String,
    body: String,
}

struct ArticleCreatedEvent has copy,store {
    id: ObjectID,
}

struct ArticleUpdatedEvent has copy,store {
    id: ObjectID,
    version: u64,
}

struct ArticleDeletedEvent has copy,store {
    id: ObjectID,
    version: u64,
}
```

文章数据结构包含三个字段，`version` 用来记录文章的版本号，`title` 用来记录文章标题，`body` 用来记录文章内容。

定义创建文章的函数：

```move
/// Create article
public fun create_article(
    ctx: &mut StorageContext,
    owner: &signer,
    title: String,
    body: String,
): ObjectID {
    assert!(std::string::length(&title) <= 200, error::invalid_argument(EDATA_TOO_LONG));
    assert!(std::string::length(&body) <= 2000, error::invalid_argument(EDATA_TOO_LONG));

    let tx_ctx = storage_context::tx_context_mut(ctx);
    let article = Article {
        version: 0,
        title,
        body,
    };
    let owner_address = signer::address_of(owner);
    let article_obj = object::new(
        tx_ctx,
        owner_address,
        article,
    );
    let id = object::id(&article_obj);
    let object_storage = storage_context::object_storage_mut(ctx);
    object_storage::add(object_storage, article_obj);

    let article_created_event = ArticleCreatedEvent {
        id,
    };
    event::emit_event(ctx, article_created_event);
    id
}
```

这个函数中，先检查文章标题和内容的长度是否超过限制。然后创建文章对象，将文章对象添加到对象存储中，最后发送文章创建事件，返回文章的 ID。

然后定义修改函数：

```move
public fun update_article(
        ctx: &mut StorageContext,
        owner: &signer,
        id: ObjectID,
        new_title: String,
        new_body: String,
) {
    assert!(std::string::length(&new_title) <= 200, error::invalid_argument(EDATA_TOO_LONG));
    assert!(std::string::length(&new_body) <= 2000, error::invalid_argument(EDATA_TOO_LONG));

    let object_storage = storage_context::object_storage_mut(ctx);
    let article_obj = object_storage::borrow_mut<Article>(object_storage, id);
    let owner_address = signer::address_of(owner);
    
    // only article owner can update the article 
    assert!(object::owner(article_obj) == owner_address, error::permission_denied(ENOT_OWNER_ACCOUNT));

    let article = object::borrow_mut(article_obj);
    article.version = article.version + 1;
    article.title = new_title;
    article.body = new_body;

    let article_update_event = ArticleUpdatedEvent {
        id,
        version: article.version,
    };
    event::emit_event(ctx, article_update_event);
}
```

这个函数中，先检查新的文章标题和内容的长度是否超过限制。然后从对象存储中获取文章对象，检查调用者是否是文章的所有者，如果不是，则抛出异常。最后更新文章对象的版本号，标题和内容，发送文章更新事件。

然后再定义删除函数：

```move
 /// Delete article
public fun delete_article(
    ctx: &mut StorageContext,
    owner: &signer,
    id: ObjectID,
) {
    let object_storage = storage_context::object_storage_mut(ctx);
    let article_obj = object_storage::remove<Article>(object_storage, id);
    let owner_address = signer::address_of(owner);
    
    // only article owner can delete the article 
    assert!(object::owner(&article_obj) == owner_address, error::permission_denied(ENOT_OWNER_ACCOUNT));

    let article_deleted_event = ArticleDeletedEvent {
        id,
        version: object::borrow(&article_obj).version,
    };
    event::emit_event(ctx, article_deleted_event);
    drop_article(article_obj);
}
```

这个函数中，先从对象存储中删除文章对象，检查调用者是否是文章的所有者，如果不是，则抛出异常。最后发送文章删除事件并销毁文章对象。

最后，我们还需要提供一个根据 ID 查询文章的函数，供其他合约使用：

```move
/// get article object by id
public fun get_article(ctx: &StorageContext, article_id: ObjectID): &Object<Article> {
    let obj_store = storage_context::object_storage(ctx);
    object_storage::borrow(obj_store, article_id)
}
```

完整的合约代码如下：

```move
module simple_blog::article {

    use std::error;
    use std::signer;
    use std::string::String; 
    use moveos_std::event;
    use moveos_std::object::{Self, Object};
    use moveos_std::object_id::ObjectID;
    use moveos_std::object_storage;
    use moveos_std::storage_context::{Self, StorageContext};

    const EDATA_TOO_LONG: u64 = 1;
    const ENOT_OWNER_ACCOUNT: u64 = 2;

    struct Article has key {
        version: u64,
        title: String,
        body: String,
    }

    struct ArticleCreatedEvent has key,copy,store {
        id: ObjectID,
    }

    struct ArticleUpdatedEvent has key,copy,store {
        id: ObjectID,
        version: u64,
    }

    struct ArticleDeletedEvent has key,copy,store {
        id: ObjectID,
        version: u64,
    }


    /// Create article
    public fun create_article(
        ctx: &mut StorageContext,
        owner: &signer,
        title: String,
        body: String,
    ): ObjectID {
        assert!(std::string::length(&title) <= 200, error::invalid_argument(EDATA_TOO_LONG));
        assert!(std::string::length(&body) <= 2000, error::invalid_argument(EDATA_TOO_LONG));

        let tx_ctx = storage_context::tx_context_mut(ctx);
        let article = Article {
            version: 0,
            title,
            body,
        };
        let owner_address = signer::address_of(owner);
        let article_obj = object::new(
            tx_ctx,
            owner_address,
            article,
        );
        let id = object::id(&article_obj);
        let object_storage = storage_context::object_storage_mut(ctx);
        object_storage::add(object_storage, article_obj);

        let article_created_event = ArticleCreatedEvent {
            id,
        };
        event::emit_event(ctx, article_created_event);
        id
    }

    /// Update article
    public fun update_article(
        ctx: &mut StorageContext,
        owner: &signer,
        id: ObjectID,
        new_title: String,
        new_body: String,
    ) {
        assert!(std::string::length(&new_title) <= 200, error::invalid_argument(EDATA_TOO_LONG));
        assert!(std::string::length(&new_body) <= 2000, error::invalid_argument(EDATA_TOO_LONG));

        let object_storage = storage_context::object_storage_mut(ctx);
        let article_obj = object_storage::borrow_mut<Article>(object_storage, id);
        let owner_address = signer::address_of(owner);
        
        // only article owner can update the article 
        assert!(object::owner(article_obj) == owner_address, error::permission_denied(ENOT_OWNER_ACCOUNT));

        let article = object::borrow_mut(article_obj);
        article.version = article.version + 1;
        article.title = new_title;
        article.body = new_body;

        let article_update_event = ArticleUpdatedEvent {
            id,
            version: article.version,
        };
        event::emit_event(ctx, article_update_event);
    }

    /// Delete article
    public fun delete_article(
        ctx: &mut StorageContext,
        owner: &signer,
        id: ObjectID,
    ) {
        let object_storage = storage_context::object_storage_mut(ctx);
        let article_obj = object_storage::remove<Article>(object_storage, id);
        let owner_address = signer::address_of(owner);
        
        // only article owner can delete the article 
        assert!(object::owner(&article_obj) == owner_address, error::permission_denied(ENOT_OWNER_ACCOUNT));

        let article_deleted_event = ArticleDeletedEvent {
            id,
            version: object::borrow(&article_obj).version,
        };
        event::emit_event(ctx, article_deleted_event);
        drop_article(article_obj);
    }

    fun drop_article(article_obj: Object<Article>) {
        let (_id, _owner, article) =  object::unpack(article_obj);
        let Article {
            version: _version,
            title: _title,
            body: _body,
        } = article;
    }

    /// Read function of article

    /// get article object by id
    public fun get_article(ctx: &StorageContext, article_id: ObjectID): &Object<Article> {
        let obj_store = storage_context::object_storage(ctx);
        object_storage::borrow(obj_store, article_id)
    }

    /// get article id
    public fun id(article_obj: &Object<Article>): ObjectID {
        object::id(article_obj)
    }

    /// get article version
    public fun version(article_obj: &Object<Article>): u64 {
        object::borrow(article_obj).version
    }

    /// get article title
    public fun title(article_obj: &Object<Article>): String {
        object::borrow(article_obj).title
    }

    /// get article body
    public fun body(article_obj: &Object<Article>): String {
        object::borrow(article_obj).body
    }
}
```

#### 4.3.2 博客合约集成文章合约

接下来，我们在 `blog.move` 中集成文章合约，并提供入口函数：

```move
    public entry fun create_article(
        ctx: &mut StorageContext,
        owner: signer,
        title: String,
        body: String,
    ) {
        let article_id = article::create_article(ctx, &owner, title, body);
        add_article_to_myblog(ctx, &owner, article_id);
    }

    public entry fun update_article(
        ctx: &mut StorageContext,
        owner: signer,
        id: ObjectID,
        new_title: String,
        new_body: String,
    ) {
        article::update_article(ctx, &owner, id, new_title, new_body);
    }

    public entry fun delete_article(
        ctx: &mut StorageContext,
        owner: signer,
        id: ObjectID,
    ) {
        article::delete_article(ctx, &owner, id);
        delete_article_from_myblog(ctx, &owner, id);
    }
```

创建和删除文章的时候，同时更新博客中的文章列表。

#### 4.3.3 创建博客文章

可以像下面这样，使用 Rooch CLI 提交一个交易，创建一篇测试文章：

```shell
rooch move run --function 0xbbfc33692c7d57839fde9643681fb64c83b377e4c70b1e4b76aa35ff1e410d01::blog::create_article --sender-account default --args 'string:Hello Rooch' "string:Accelerating World's Transition to Decentralization"
```

`--function` 指定执行发布在 `0xbbfc33692c7d57839fde9643681fb64c83b377e4c70b1e4b76aa35ff1e410d01` 地址上的 `blog` 模块中的 `create_article` 函数，即新建一篇博客文章。`--sender-account` 指定谁来提交这个交易。这个函数要求我们必须给它传递两个参数，通过 `--args` 来指定，第一个是文章的标题，我取名为 `Hello Rooch`；第二个是文章的内容，我写上 Rooch 的标语（slogan）：`Accelerating World's Transition to Decentralization`。

参数传递的是字符串，需要使用引号将内容包裹起来，并且通过 `string:` 来显示指定，在第二个参数的内容中有单引号，所以使用双引号包裹，否则必须使用转义符（`\`）。

你可以随意更换 `--args` 后面的第一个参数（`title`）和第二个参数（`body`）的内容，多创建几篇文章。

#### 4.3.4 查询文章

现在，你可以通过查询事件，得到已创建好的文章的 `ObjectID`：

```shell
curl --location --request POST 'http://localhost:50051' \
--header 'Content-Type: application/json' \
--data-raw '{
 "id":101,
 "jsonrpc":"2.0",
 "method":"rooch_getEventsByEventHandle",
 "params":["0xbbfc33692c7d57839fde9643681fb64c83b377e4c70b1e4b76aa35ff1e410d01::article::ArticleCreatedEvent", null, 1000]
}'
```

返回的响应内容：

```json
{"jsonrpc":"2.0","result":{"data":[{"event":{"event_id":{"event_handle_id":"0xc48dc675718370db4273a419875967e7c32615f907262d475730d8faf0afca44","event_seq":0},"type_tag":"0xbbfc33692c7d57839fde9643681fb64c83b377e4c70b1e4b76aa35ff1e410d01::article::ArticleCreatedEvent","event_data":"0x1f27bd310f51b09915648d53319e65509dcc7ca42ffc1cf989bfa24073d78a41","event_index":0},"sender":"0x0000000000000000000000000000000000000000000000000000000000000000","tx_hash":null,"timestamp_ms":null,"parsed_event_data":{"abilities":13,"type":"0xbbfc33692c7d57839fde9643681fb64c83b377e4c70b1e4b76aa35ff1e410d01::article::ArticleCreatedEvent","value":{"id":"0x1f27bd310f51b09915648d53319e65509dcc7ca42ffc1cf989bfa24073d78a41"}}}],"next_cursor":0,"has_next_page":false},"id":101}
```

由于输出的内容比较多，可以在上面的命令最尾添加一个管道操作（` | jq '.result.data[0].parsed_event_data.value.id'`），来快速筛选出第一篇文章的 `ObjectID`。 

> 提示：在使用 `jp` 命令（jq - commandline JSON processor）之前，你可能需要先安装它。

添加 `jp` 处理后的命令像下面这样：

```shell
curl --location --request POST 'http://localhost:50051' \
--header 'Content-Type: application/json' \
--data-raw '{
 "id":101,
 "jsonrpc":"2.0",
 "method":"rooch_getEventsByEventHandle",
 "params":["0xbbfc33692c7d57839fde9643681fb64c83b377e4c70b1e4b76aa35ff1e410d01::article::ArticleCreatedEvent", null, 1000]
}' | jq '.result.data[0].parsed_event_data.value.id'
```

通过 `jp` 来筛选出的博客的对象 ID 为：

```shell
"0x1f27bd310f51b09915648d53319e65509dcc7ca42ffc1cf989bfa24073d78a41"
```

然后，你可以使用 Rooch CLI 来查询对象的状态，通过 `--id` 来指定文章对象的 ID（注意替换为你的文章的 ObjectID）：

```shell
rooch state --access-path /object/0x1f27bd310f51b09915648d53319e65509dcc7ca42ffc1cf989bfa24073d78a41
```

```json
[
  {
    "state": {
      "value": "0x1f27bd310f51b09915648d53319e65509dcc7ca42ffc1cf989bfa24073d78a41bbfc33692c7d57839fde9643681fb64c83b377e4c70b1e4b76aa35ff1e410d0100000000000000000b48656c6c6f20526f6f636833416363656c65726174696e6720576f726c642773205472616e736974696f6e20746f20446563656e7472616c697a6174696f6e",
      "value_type": "0x2::object::Object<0xbbfc33692c7d57839fde9643681fb64c83b377e4c70b1e4b76aa35ff1e410d01::article::Article>"
    },
    "move_value": {
      "abilities": 0,
      "type": "0x2::object::Object<0xbbfc33692c7d57839fde9643681fb64c83b377e4c70b1e4b76aa35ff1e410d01::article::Article>",
      "value": {
        "id": "0x1f27bd310f51b09915648d53319e65509dcc7ca42ffc1cf989bfa24073d78a41",
        "owner": "0xbbfc33692c7d57839fde9643681fb64c83b377e4c70b1e4b76aa35ff1e410d01",
        "value": {
          "abilities": 8,
          "type": "0xbbfc33692c7d57839fde9643681fb64c83b377e4c70b1e4b76aa35ff1e410d01::article::Article",
          "value": {
            "body": "Accelerating World's Transition to Decentralization",
            "title": "Hello Rooch",
            "version": "0"
          }
        }
      }
    }
  }
]
```

注意观察输出中，`title` 和 `body` 这两个键值对，能看到这个对象确实“存储着”刚刚写的那篇博客文章。

我们也可以用前面的命令来查询账户下的 `MyBlog` Resource：

```shell
rooch state --access-path /resource/0xbbfc33692c7d57839fde9643681fb64c83b377e4c70b1e4b76aa35ff1e410d01/0xbbfc33692c7d57839fde9643681fb64c83b377e4c70b1e4b76aa35ff1e410d01::blog::MyBlog        
```
```json
[
  {
    "state": {
      "value": "0x0a526f6f636820626c6f67011f27bd310f51b09915648d53319e65509dcc7ca42ffc1cf989bfa24073d78a41",
      "value_type": "0xbbfc33692c7d57839fde9643681fb64c83b377e4c70b1e4b76aa35ff1e410d01::blog::MyBlog"
    },
    "move_value": {
      "abilities": 8,
      "type": "0xbbfc33692c7d57839fde9643681fb64c83b377e4c70b1e4b76aa35ff1e410d01::blog::MyBlog",
      "value": {
        "articles": [
          "0x1f27bd310f51b09915648d53319e65509dcc7ca42ffc1cf989bfa24073d78a41"
        ],
        "name": "Rooch blog"
      }
    }
  }
]
```

可以看到，`MyBlog` 中的 `articles` 字段，存储着我们刚刚创建的那篇文章的对象 ID。

#### 4.3.5 更新文章

我们尝试使用 `update_article` 函数来更新文章的内容。

`--function` 指定执行发布在 `0xbbfc33692c7d57839fde9643681fb64c83b377e4c70b1e4b76aa35ff1e410d01` 地址上的 `blog` 模块中的 `update` 函数，即更新一篇博客文章。同样也需要使用 `--sender-account` 来指定发送这个更新文章交易的账户。这个函数要求我们必须给它传递三个参数，通过 `--args` 来指定，第一个参数是要修改文章的对象 ID，后面的两个参数分别对应文章的标题和正文。

将文章的标题修改为 `Foo`，文章正文修改为 `Bar`：

```shell
rooch move run --function 0xbbfc33692c7d57839fde9643681fb64c83b377e4c70b1e4b76aa35ff1e410d01::blog::update_article --sender-account default --args 'object_id:0x1f27bd310f51b09915648d53319e65509dcc7ca42ffc1cf989bfa24073d78a41' 'string:Foo' 'string:Bar'
```

除了使用 Rooch CLI，你还可以通过调用 JSON RPC 来查询对象的状态：

```shell
curl --location --request POST 'http://127.0.0.1:50051/' \
--header 'Content-Type: application/json' \
--data-raw '{
 "id":101,
 "jsonrpc":"2.0",
 "method":"rooch_getAnnotatedStates",
 "params":["/object/0x1f27bd310f51b09915648d53319e65509dcc7ca42ffc1cf989bfa24073d78a41"]
}'
```

在输出中，可以观察到文章的标题和正文已成功修改：

```json
{"jsonrpc":"2.0","result":[{"state":{"value":"0x1f27bd310f51b09915648d53319e65509dcc7ca42ffc1cf989bfa24073d78a41bbfc33692c7d57839fde9643681fb64c83b377e4c70b1e4b76aa35ff1e410d01010000000000000003466f6f03426172","value_type":"0x2::object::Object<0xbbfc33692c7d57839fde9643681fb64c83b377e4c70b1e4b76aa35ff1e410d01::article::Article>"},"move_value":{"abilities":0,"type":"0x2::object::Object<0xbbfc33692c7d57839fde9643681fb64c83b377e4c70b1e4b76aa35ff1e410d01::article::Article>","value":{"id":"0x1f27bd310f51b09915648d53319e65509dcc7ca42ffc1cf989bfa24073d78a41","owner":"0xbbfc33692c7d57839fde9643681fb64c83b377e4c70b1e4b76aa35ff1e410d01","value":{"abilities":8,"type":"0xbbfc33692c7d57839fde9643681fb64c83b377e4c70b1e4b76aa35ff1e410d01::article::Article","value":{"body":"Bar","title":"Foo","version":"1"}}}}}],"id":101}
```

#### 4.3.6 删除文章

可以这样提交一个交易，调用 `blog::delete_article` 删除文章：

```shell
rooch move run --function 0xbbfc33692c7d57839fde9643681fb64c83b377e4c70b1e4b76aa35ff1e410d01::blog::delete_article --sender-account default --args 'object_id:0x1f27bd310f51b09915648d53319e65509dcc7ca42ffc1cf989bfa24073d78a41'
```

`--function` 指定执行发布在 `0xbbfc33692c7d57839fde9643681fb64c83b377e4c70b1e4b76aa35ff1e410d01` 地址上的 `blog` 模块中的 `delete_article` 函数，即删除一篇博客文章。同样也需要使用 `--sender-account` 来指定发送这个删除文章交易的账户。这个函数只需给它传递一个参数，即文章对应的对象 ID，通过 `--args` 来指定。

#### 4.3.7 检查文章是否正常删除

```shell
rooch state --access-path /object/0x1f27bd310f51b09915648d53319e65509dcc7ca42ffc1cf989bfa24073d78a41

[
  null
]
```

可以看到，查询文章的对象 ID 时，结果反回 `null`。说明文章已经被删除了。再查询 `MyBlog` Resource：

```shell
rooch state --access-path /resource/0xbbfc33692c7d57839fde9643681fb64c83b377e4c70b1e4b76aa35ff1e410d01/0xbbfc33692c7d57839fde9643681fb64c83b377e4c70b1e4b76aa35ff1e410d01::blog::MyBlog        
```

```json
[
  {
    "state": {
      "value": "0x0a526f6f636820626c6f6700",
      "value_type": "0xbbfc33692c7d57839fde9643681fb64c83b377e4c70b1e4b76aa35ff1e410d01::blog::MyBlog"
    },
    "move_value": {
      "abilities": 8,
      "type": "0xbbfc33692c7d57839fde9643681fb64c83b377e4c70b1e4b76aa35ff1e410d01::blog::MyBlog",
      "value": {
        "articles": [],
        "name": "Rooch blog"
      }
    }
  }
]
```

可以看到，`articles` 数组为空，说明文章列表也已经更新。

## 5. 总结

到这里，相信你已经对 Rooch v0.1 如何运行，如何发布合约，以及如何跟合约交互有了基本的了解。想要在 Rooch 上体验更多的合约例子，请参见 [`rooch/examples`](https://github.com/rooch-network/rooch/tree/main/examples)。

如果想直接体验这个博客合约的功能，可以直接[下载博客源码](https://github.com/rooch-network/rooch/archive/refs/heads/main.zip)，解压，并切换到博客合约项目的根目录，交互方式请参照上文。

```shell
wget https://github.com/rooch-network/rooch/archive/refs/heads/main.zip
unzip main.zip
cd rooch-main/examples/simple_blog
```

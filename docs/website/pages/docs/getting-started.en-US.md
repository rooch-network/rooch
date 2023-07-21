# Getting started with Rooch

## 1. What is Rooch

[Rooch](https://rooch.network) is a fast, modular, secure, developer-friendly infrastructure solution for building Web3-native applications.

Rooch released the first version on June 28, 2023, the version name is **Sprouting**, and the version number is `v0.1`.

## 2. Install Rooch

### 2.1 Download

Prebuilt binary tarballs and corresponding source tarballs can be found on [Rooch's GitHub releases page](https://github.com/rooch-network/rooch/releases). If you want to experience the Git version, you can refer to this page to [compile and install Rooch](https://github.com/rooch-network/rooch#getting-started). The following guides you to install the Release version of Rooch.

```shell
wget https://github.com/rooch-network/rooch/releases/download/v0.1/rooch-ubuntu-latest.zip
```

> Note: Please choose the version corresponding to your own system. I will use the Linux version to demonstrate here.

### 2.2 Decompress

```shell
unzip rooch-ubuntu-latest.zip
```

The decompressed files are stored in the `rooch-artifacts` directory, and `rooch` is our precompiled binary program.

```shell
rooch-artifacts
├── README.md
└── rooch
```

### 2.3 Run

Go to the unzipped folder `rooch-artifacts` and test if the program works.

```shell
cd rooch-artifacts
./rooch
```

If you can see the output below, it means everything is working fine.

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

#### 2.4 Add to PATH

For the convenience of subsequent use, it is recommended to put `rooch` into a path that can be retrieved by the system environment variable `PATH`, or `export` the current decompressed directory to `PATH` through export.

- Method 1, copy the `rooch` program to the `/usr/local/bin` directory (**recommended**)

```shell
sudo cp rooch /usr/local/bin
```

- Method 2, export path (not recommended)

Use the following small script to add `rooch` to the Bash shell's `PATH`.

```shell
echo "export PATH=\$PATH:$PWD" >> ~/.bashrc
source ~/.bashrc
```

## 3. Initialize Rooch configuration

```shell
rooch init
```

After running the command to initialize the configuration, a `.rooch` directory will be created in the user's home directory (`$HOME`), and the relevant configuration information of the Rooch account will be generated.

## 4. Create a new Rooch project

This part will guide you how to create a blog contract application on Rooch, and realize the basic **CRUD** functions.

### 4.1 Create a Move project

Use the `rooch` integration's `move new` command to create a blog application called simple_blog.

```shell
rooch move new simple_blog
```

The generated Move project contains a configuration file `Move.toml` and a `sources` directory for storing Move source code.

```shell
simple_blog
├── Move.toml
└── sources
```

We can take a quick look at what the `Move.toml` file contains.

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

- There are three tables in the TOML file: `package`, `dependencies` and `addresses`, which store some meta information required by the project.
- The `package` table is used to store some description information of the project, which contains two key-value pairs `name` and `version` to describe the project name and version number of the project.
- The `dependencies` table is used to store the metadata that the project depends on.
- The `addresses` table is used to store the address of the project and the addresses of the modules that the project depends on. The first address is the address generated in `$HOME/.rooch/rooch_config/rooch.yaml` when initializing the Rooch configuration.

In order to facilitate the deployment of other developers, we replace the address of `simple_blog` with `_`, and then specify it through `--named--addresses` when deploying.

### 4.2 Quick experience

In this section, I will guide you to write a blog initialization function and run it in Rooch to experience the basic process of `writing -> compiling -> publishing -> calling` the contract.

We create a new `blog.move` file in the `sources` directory and start writing our blog contract.

#### 4.2.1 Define the structure of the blog

Our blogging system allows everyone to create their own blog and keep their own list of blogs. First, we define a blog structure:

```move
struct MyBlog has key {
    name: String,
    articles: vector<ObjectID>,
}
```

This structure contains two fields, one is the name of the blog, and the other is the list of blog posts. For the article list, we only save the ID of the article Object.

Then define a function to create a blog:

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

Creating a blog is to initialize the `MyBlog` data structure and save `MyBlog` in the user's storage space. At the same time, an entry function for setting the blog name is provided. If the blog does not exist, the blog will be created first, and then the blog name will be set.

Then provide a contract initialization function, which will be automatically executed when the contract is published, and the blog will be automatically initialized for the user who publishes the contract.

```move
/// This init function is called when the module is published
/// The owner is the address of the account that publishes the module
fun init(storage_ctx: &mut StorageContext, owner: &signer) {
    // auto create blog for module publisher 
    create_blog(storage_ctx, owner);
}
```

Then, provide a function to query the blog list and a function to add and delete articles. The whole code is as follows:

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

- `module simple_blog::blog` is used to declare which module our contract belongs to. Its syntax is `module address::module_name`, and the logic (function) of the contract is written in curly braces `{}`.
- The `use` statement imports the libraries we need to depend on when writing contracts.
- `const` defines the constants used in the contract, usually used to define some error codes.
- `fun` is a keyword used to define a function, usually the function of the function is defined here. For safety, such functions are prohibited from being called directly on the command line, and the calling logic needs to be encapsulated in the entry function.
- `entry fun` is used to define the entry function, and the function modified by the `entry` modifier can be called directly on the command line like a script.

#### 4.2.2 Compile the Move contract

Before publishing the contract, we need to compile our contract. Here, `--named-addresses` is used to specify the address of the `simple_blog` module as the `default` address on the current device.

```shell
rooch move build --named-addresses simple_blog=default
```

After compiling, if there are no errors, you will see the message of `Success` at the end.

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

At this time, there will be a `build` directory in the project folder, which stores the contract bytecode file generated by the Move compiler and the **complete** code of the contract.

#### 4.2.3 Running the Rooch server

Let's open another terminal and run the following command, the Rooch server will start the Rooch container service locally to process and respond to the relevant behavior of the contract.

```shell
rooch server start
```

After starting the Rooch service, you will see these two messages at the end, indicating that the Rooch service has been started normally.

```shell
2023-07-03T15:44:33.315228Z  INFO rooch_rpc_server: JSON-RPC HTTP Server start listening 0.0.0.0:50051
2023-07-03T15:44:33.315256Z  INFO rooch_rpc_server: Available JSON-RPC methods : ["wallet_accounts", "eth_blockNumber", "eth_getBalance", "eth_gasPrice", "net_version", "eth_getTransactionCount", "eth_sendTransaction", "rooch_sendRawTransaction", "rooch_getAnnotatedStates", "eth_sendRawTransaction", "rooch_getTransactionByIndex", "rooch_executeRawTransaction", "rooch_getEventsByEventHandle", "rooch_getTransactionByHash", "rooch_executeViewFunction", "eth_getBlockByNumber", "rooch_getEvents", "eth_feeHistory", "eth_getTransactionByHash", "eth_getBlockByHash", "eth_getTransactionReceipt", "rooch_getTransactionInfosByTxOrder", "eth_estimateGas", "eth_chainId", "rooch_getTransactionInfosByTxHash", "wallet_sign", "rooch_getStates"]
```

> Tip: When we operate the contract-related logic (function) in the previous terminal window, we can observe the output of this terminal window.

#### 4.2.4 Publish the Move contract

```shell
rooch move publish --named-addresses simple_blog=default
```

You can confirm that the publish operation was successfully executed when you see output similar to this (`status` is `executed`):

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

You can also see the processing information of the response in the terminal running the Rooch service:

```shell
2023-07-03T16:02:11.691028Z  INFO connection{remote_addr=127.0.0.1:58770 conn_id=0}: jsonrpsee_server::server: Accepting new connection 1/100
2023-07-03T16:02:13.690733Z  INFO rooch_proposer::actor::proposer: [ProposeBlock] block_number: 0, batch_size: 1
```

#### 4.2.5 Call the Move contract

At this point, our blog contract has been released to the chain, and the blog has been initialized under the default account. We can use the status query command to check the blog Resource under the account:

```shell
rooch state --access-path /resource/{ACCOUNT_ADDRESS}/{RESOURCE_TYPE}
```

Among them, `{ACCOUNT_ADDRESS}` is the account address, `{RESOURCE_TYPE}` is the resource type, here is `{MODULE_ADDRESS}::blog::MyBlog`. Here `{ACCOUNT_ADDRESS}` and `{MODULE_ADDRESS}` are the default account addresses of my machine.

We can check the value corresponding to the `active_address` key in the `$HOME/.rooch/rooch_config/rooch.yaml` file, which is the default account address of the operation contract.

My address is `0xbbfc33692c7d57839fde9643681fb64c83b377e4c70b1e4b76aa35ff1e410d01`, and I will continue to use this address to demonstrate related operations.

So the command I actually execute here should be:

```shell
rooch state --access-path /resource/0xbbfc33692c7d57839fde9643681fb64c83b377e4c70b1e4b76aa35ff1e410d01/0xbbfc33692c7d57839fde9643681fb64c83b377e4c70b1e4b76aa35ff1e410d01::blog::MyBlog
```

return result:

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

It can be seen that `MyBlog` Resource already exists, the name is the default `MyBlog`, and the article list is empty.

Then we set the blog name through the `set_blog_name` function. The syntax for calling a contract entry function is:

```shell
rooch move run --function {ACCOUNT_ADDRESS}::{MODULE_NAME}::{FUNCTION_NAME} --sender-account {ACCOUNT_ADDRESS}
```

Run a function with the `rooch move run` command. `--function` Specify the function name, you need to pass a complete function name, that is, `the_address_of_the_published_contract::module_name::function_name`, in order to accurately identify the function that needs to be called. `--sender-account` specifies the address of the account that calls this function, that is, which account is used to call this function, and anyone can call the contract on the chain.

```shell
rooch move run --function 0xbbfc33692c7d57839fde9643681fb64c83b377e4c70b1e4b76aa35ff1e410d01::blog::set_blog_name --sender-account default --args 'string:Rooch blog'
```

When this command is executed, a transaction will be sent to the chain, and the content of the transaction is to call the `set_blog_name` function in the blog contract.

After the execution is successful, run the previous status query command again, and view the blog Resource:

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

As you can see, the blog name has been modified successfully. So far, we have experienced the zero-to-one installation in Rooch, initial configuration, project creation, writing contracts, compiling contracts, publishing contracts, and calling contracts.

### 4.3 Improve the blog contract

Next, we will continue to improve the blog contract and increase the function of **CRUD** blog posts.

#### 4.3.1 Create blog post contract

We create another `article.move` file in the `sources` directory, which stores the data type of the article and the definition of related events for CRUD operations on the article.

Define the article data type, and the article event type:

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

The article data structure contains three fields, `version` is used to record the version number of the article, `title` is used to record the title of the article, and `body` is used to record the content of the article.

Define a function to create an article:

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

In this function, first check whether the length of the article title and content exceeds the limit. Then create the article object, add the article object to the object store, and finally send the article creation event and return the ID of the article.

Then define the modification function:

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

In this function, first check whether the length of the new article title and content exceeds the limit. Then get the article object from the object store, check if the caller is the owner of the article, and throw an exception if not. Finally, update the version number, title and content of the article object, and send an article update event.

Then define the delete function:

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

In this function, first delete the article object from the object store, check whether the caller is the owner of the article, and throw an exception if not. Finally send the article delete event and destroy the article object.

Finally, we also need to provide a function to query articles by ID for use by other contracts:

```move
/// get article object by id
public fun get_article(ctx: &StorageContext, article_id: ObjectID): &Object<Article> {
    let obj_store = storage_context::object_storage(ctx);
    object_storage::borrow(obj_store, article_id)
}
```

The complete contract code is as follows:

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

#### 4.3.2 Blog Contract Integration Article Contract

Next, we integrate the article contract in `blog.move` and provide the entry function:

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

When creating and deleting articles, update the list of articles in the blog at the same time.

#### 4.3.3 Creating blog posts

A test article can be created by submitting a transaction using the Rooch CLI like this:

```shell
rooch move run --function 0xbbfc33692c7d57839fde9643681fb64c83b377e4c70b1e4b76aa35ff1e410d01::blog::create_article --sender-account default --args 'string:Hello Rooch' "string:Accelerating World's Transition to Decentralization"
```

`--function` specifies to execute the `create_article` function in the `blog` module published at address `0xbbfc33692c7d57839fde9643681fb64c83b377e4c70b1e4b76aa35ff1e410d01`, that is, create a new blog post. `--sender-account` specifies who should submit this transaction. This function requires us to pass two parameters to it, specified by `--args`, the first is the title of the article, I named it `Hello Rooch`; the second is the content of the article, I wrote the slogan of Rooch: `Accelerating World's Transition to Decentralization`.

The parameter passed is a string, which needs to be wrapped in quotation marks and specified through `string:`. There are single quotation marks in the content of the second parameter, so use double quotation marks to wrap it, otherwise you must use an escape character (`\`).

You can freely change the content of the first parameter (`title`) and the second parameter (`body`) after `--args` to create more articles.

#### 4.3.4 Query Articles

Now, you can get the `ObjectID` of the created article by querying the event:

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

The returned response content:

```json
{"jsonrpc":"2.0","result":{"data":[{"event":{"event_id":{"event_handle_id":"0xc48dc675718370db4273a419875967e7c32615f907262d475730d8faf0afca44","event_seq":0},"type_tag":"0xbbfc33692c7d57839fde9643681fb64c83b377e4c70b1e4b76aa35ff1e410d01::article::ArticleCreatedEvent","event_data":"0x1f27bd310f51b09915648d53319e65509dcc7ca42ffc1cf989bfa24073d78a41","event_index":0},"sender":"0x0000000000000000000000000000000000000000000000000000000000000000","tx_hash":null,"timestamp_ms":null,"parsed_event_data":{"abilities":13,"type":"0xbbfc33692c7d57839fde9643681fb64c83b377e4c70b1e4b76aa35ff1e410d01::article::ArticleCreatedEvent","value":{"id":"0x1f27bd310f51b09915648d53319e65509dcc7ca42ffc1cf989bfa24073d78a41"}}}],"next_cursor":0,"has_next_page":false},"id":101}
```

Since there are many output contents, you can add a pipeline operation (` | jq '.result.data[0].parsed_event_data.value.id'`) at the end of the above command to quickly filter out the `ObjectID` of the first article.

> Tip: Before using the `jp` command (jq - commandline JSON processor), you may need to install it first.

The command after adding `jp` processing looks like this:

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

The object ID of the blog that is screened through `jp` is:

```shell
"0x1f27bd310f51b09915648d53319e65509dcc7ca42ffc1cf989bfa24073d78a41"
```

Then, you can use the Rooch CLI to query the status of the object, passing `--id` to specify the ID of the article object (replace it with the ObjectID of your article):

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

Pay attention to the two key-value pairs `title` and `body` in the output, and you can see that this object does 'store' the blog post you just wrote.

We can also use the previous command to query `MyBlog` Resource under the account:

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

As you can see, the `articles` field in `MyBlog` stores the object ID of the article we just created.

#### 4.3.5 Updating Articles

We try to update the content of an article using the `update_article` function.

`--function` specifies to execute the `update` function in the `blog` module published at address `0xbbfc33692c7d57839fde9643681fb64c83b377e4c70b1e4b76aa35ff1e410d01`, that is, to update a blog post. Also need to use `--sender-account` to specify the account that will send this update article transaction. This function requires us to pass three parameters to it, specified by `--args`, the first parameter is the object ID of the article to be modified, and the latter two parameters correspond to the title and body of the article respectively.

Change the title of the article to be `Foo` and the body of the article to be `Bar`:

```shell
rooch move run --function 0xbbfc33692c7d57839fde9643681fb64c83b377e4c70b1e4b76aa35ff1e410d01::blog::update_article --sender-account default --args 'object_id:0x1f27bd310f51b09915648d53319e65509dcc7ca42ffc1cf989bfa24073d78a41' 'string:Foo' 'string:Bar'
```

In addition to using the Rooch CLI, you can also query the state of objects by calling JSON RPC:

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

In the output, it can be observed that the title and body of the article are successfully modified:

```json
{"jsonrpc":"2.0","result":[{"state":{"value":"0x1f27bd310f51b09915648d53319e65509dcc7ca42ffc1cf989bfa24073d78a41bbfc33692c7d57839fde9643681fb64c83b377e4c70b1e4b76aa35ff1e410d01010000000000000003466f6f03426172","value_type":"0x2::object::Object<0xbbfc33692c7d57839fde9643681fb64c83b377e4c70b1e4b76aa35ff1e410d01::article::Article>"},"move_value":{"abilities":0,"type":"0x2::object::Object<0xbbfc33692c7d57839fde9643681fb64c83b377e4c70b1e4b76aa35ff1e410d01::article::Article>","value":{"id":"0x1f27bd310f51b09915648d53319e65509dcc7ca42ffc1cf989bfa24073d78a41","owner":"0xbbfc33692c7d57839fde9643681fb64c83b377e4c70b1e4b76aa35ff1e410d01","value":{"abilities":8,"type":"0xbbfc33692c7d57839fde9643681fb64c83b377e4c70b1e4b76aa35ff1e410d01::article::Article","value":{"body":"Bar","title":"Foo","version":"1"}}}}}],"id":101}
```

#### 4.3.6 Delete article

A transaction can be submitted like this, calling `blog::delete_article` to delete an article:

```shell
rooch move run --function 0xbbfc33692c7d57839fde9643681fb64c83b377e4c70b1e4b76aa35ff1e410d01::blog::delete_article --sender-account default --args 'object_id:0x1f27bd310f51b09915648d53319e65509dcc7ca42ffc1cf989bfa24073d78a41'
```

`--function` specifies to execute the `delete_article` function in the `blog` module published at `0xbbfc33692c7d57839fde9643681fb64c83b377e4c70b1e4b76aa35ff1e410d01`, that is, to delete a blog post. Also need to use `--sender-account` to specify the account to send this delete article transaction. This function only needs to pass one parameter to it, which is the object ID corresponding to the article, specified by `--args`.

#### 4.3.7 Check whether the article is deleted normally

```shell
rooch state --access-path /object/0x1f27bd310f51b09915648d53319e65509dcc7ca42ffc1cf989bfa24073d78a41

[
  null
]
```

As you can see, when querying the object ID of the article, the result returns `null`. The description article has been deleted. Then query `MyBlog` Resource:

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

As you can see, the `articles` array is empty, indicating that the article list has also been updated.

## 5. Summary

At this point, I believe you have a basic understanding of how Rooch v0.1 works, how to publish contracts, and how to interact with contracts. To experience more contract examples on Rooch, see [`rooch/examples`](https://github.com/rooch-network/rooch/tree/main/examples).

If you want to directly experience the functions of this blog contract, you can directly [download the blog source code](https://github.com/rooch-network/rooch/archive/refs/heads/main.zip), decompress it, and switch to the root directory of the blog contract project. For the interactive method, please refer to the above.

```shell
wget https://github.com/rooch-network/rooch/archive/refs/heads/main.zip
unzip main.zip
cd rooch-main/examples/simple_blog
```

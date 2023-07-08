# Getting started with Rooch

## 1. What is Rooch

Rooch(opens in a new tab) is a fast, modular, secure, developer-friendly infrastructure solution for building Web3-native applications.

Rooch released the first version on June 28, 2023, the version name is Sprout, and the version number is v0.1.

## 2. Install Rooch

### 2.1 Download

Prebuilt binary tarballs and corresponding source tarballs can be found on [Rooch's GitHub releases page](https://github.com/rooch-network/rooch/releases). If you want to experience the Git version, you can refer to this page to [compile and install Rooch](https://github.com/rooch-network/rooch#getting-started). The following guides you to install the Release version of Rooch.

```shell
wget https://github.com/rooch-network/rooch/releases/download/v0.1/rooch-ubuntu-latest.zip
```

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

#### 2.4 Add to PATH

For the convenience of subsequent use, it is recommended to put `rooch` into a path that can be retrieved by the system environment variable `PATH`, or `export` the current decompressed directory to `PATH` through export.

- Method 1, copy the `rooch` program to the `/usr/local/bin` directory (recommended)

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

Use the `move new` command from the `rooch` package to create a blog application called `rooch_blog`.

```shell
rooch move new rooch_blog
```

The generated Move project contains a configuration file `Move.toml` and a `sources` directory for storing Move source code.

```shell
rooch_blog
├── Move.toml
└── sources
```

We can take a quick look at what the `Move.toml` file contains.

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

- There are three tables in the TOML file: `package`, `dependencies` and `addresses`, which store some meta information required by the project.
- The `package` table is used to store some description information of the project, which contains two key-value pairs `name` and `version` to describe the project name and version number of the project.
- The `dependencies` table is used to store the metadata that the project depends on.
- The `addresses` table is used to store project addresses and module addresses. The first address is the address generated in `$HOME/.rooch/rooch_config/rooch.yaml` when initializing Rooch configuration.

### 4.2 Quick experience

In this section, I will guide you to write a blog initialization function and run it in Rooch to experience the basic process of `writing -> compiling -> publishing -> calling` the contract.

We create a new `blog.move` file in the `sources` directory and start writing our blog contract.

#### 4.2.1 Write the initialization function

After the contract project is deployed (published) on the chain, it needs to be initialized first, that is, the initialization function is called (only needs to be called once).

We write the following code to initialize the contract in the `blog.move` file:

```move
module rooch_blog::rooch_blog {
    use std::error;
    use std::signer;
    use moveos_std::storage_context::StorageContext;

    const EID_DATA_TOO_LONG: u64 = 102;
    const EINAPPROPRIATE_VERSION: u64 = 103;
    const ENOT_GENESIS_ACCOUNT: u64 = 105;

    // Define a function that initialize the blog
    fun init_blog(storage_ctx: &mut StorageContext, account: &signer) {
        assert!(signer::address_of(account) == @rooch_blog, error::invalid_argument(ENOT_GENESIS_ACCOUNT));
        let _ = storage_ctx;
        let _ = account;
    }

    // The entry function that initializes.
    entry fun initialize(storage_ctx: &mut StorageContext, account: &signer) {
        init_blog(storage_ctx, account);
    }
}
```
- `module rooch_blog::rooch_blog` is used to declare which module our contract belongs to. Its syntax is `module address::module_name`, and the logic (function) of the contract is written in curly braces `{}`.
- The `use` statement imports the libraries we need to depend on when writing contracts.
- `const` defines the constants used in the contract, usually used to define some error codes.
- `fun` is a keyword used to define a function, usually the function of the function is defined here. For safety, such functions are prohibited from being called directly on the command line, and the calling logic needs to be encapsulated in the entry function.
- `entry fun` is used to define the entry function, and the function modified by the `entry` modifier can be called directly on the command line like a script.

#### 4.2.2 Compile the Move contract

Before publishing to the chain, we need to compile our contract:

```shell
rooch move build
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
BUILDING rooch_blog
Success
```

At this time, there will be a `build` directory in the project folder, which stores the contract bytecode file generated by the Move compiler and the complete code of the contract.

#### 4.2.3 Running the Rooch server

Let's open another terminal and run the following command. The Rooch server will start the Rooch service locally (simulating the behavior of the chain) to process and respond to the relevant behavior of the contract.

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
rooch move publish
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

At this point, our blog contract has been released to the chain, and we try to call the `initialize` function we just wrote to initialize our contract.

The syntax for calling a contract entry function is:

```shell
rooch move run --function {ACCOUNT_ADDRESS}::{MODULE_NAME}::{FUNCTION_NAME} --sender-account {ACCOUNT_ADDRESS}
```

Run a function with the `rooch move run` command. `--function` specify the function name, you need to pass a complete function name, that is, `the_address_of_the_published_contract::module_name::function_name`, in order to accurately identify the function that needs to be called. `--sender-account` specifies the address of the account that calls this function, that is, which account is used to call this function, and anyone can call the contract on the chain.

We can check the value corresponding to the `active_address` key in the `$HOME/.rooch/rooch_config/rooch.yaml` file, which is the default account address of the operation contract.

My address is `0x36a1c5014cb1771fb0689e041875c83a31675693301a9ba233932abc0b7e68dc`, and I will continue to use this address to demonstrate related operations.

```shell
rooch move run --function 0x36a1c5014cb1771fb0689e041875c83a31675693301a9ba233932abc0b7e68dc::rooch_blog::initialize --sender-account 0x36a1c5014cb1771fb0689e041875c83a31675693301a9ba233932abc0b7e68dc
```

When this command is executed, a transaction will be sent to the chain, and the content of the transaction is to call the `initialize` function in the blog contract.

Of course, if you use Rooch's default account to call the function, the command can be simplified to:

```shell
rooch move run --function 0x36a1c5014cb1771fb0689e041875c83a31675693301a9ba233932abc0b7e68dc::rooch_blog::initialize --sender-account default
```

So far, we have experienced the zero-to-one installation in Rooch, initial configuration, project creation, writing contracts, compiling contracts, publishing contracts, and calling contracts.

### 4.3 Improve the blog contract

Next, we will continue to improve the blog contract and increase the function of **CRUD** blog posts.

#### 4.3.1 Improved initialization function

The Move language provides a standardized initialization function to initialize the contract. If the default standardized function `init` is used instead of the `init_blog` function demonstrated above, the entry function can be omitted. Using a standardized initialization function, the contract will be initialized directly after it is released to the chain, eliminating the need to manually call the initialization function.

```move
module rooch_blog::rooch_blog {
    use std::error;
    use std::signer;
    use moveos_std::storage_context::StorageContext;

    const EID_DATA_TOO_LONG: u64 = 102;
    const EINAPPROPRIATE_VERSION: u64 = 103;
    const ENOT_GENESIS_ACCOUNT: u64 = 105;

    // Define a function that initialize the blog app
    fun init(storage_ctx: &mut StorageContext, account: &signer) {
        assert!(signer::address_of(account) == @rooch_blog, error::invalid_argument(ENOT_GENESIS_ACCOUNT));
        let _ = storage_ctx;
        let _ = account;
    }
}
```

Maybe when you study the Move contract, you will see the initialization function decorated with `entry`, which is usually the behavior of the old version. If you start writing a Move contract, it is recommended that you use the standardized initialization function `init`. I hope this can help you understand the difference between manual initialization and automatic initialization.

#### 4.3.2 Create a blog contract library file

We create another `article.move` file in the `sources` directory, which stores the data type of the article and the definition of related events for CRUD operations on the article.

```move
module rooch_blog::article {
    use moveos_std::event;
    use moveos_std::object::{Self, Object};
    use moveos_std::object_id::ObjectID;
    use moveos_std::object_storage;
    use moveos_std::storage_context::{Self, StorageContext};
    use moveos_std::tx_context;
    use std::error;
    use std::option;
    use std::signer;
    use std::string::String;
    friend rooch_blog::rooch_blog;

    const EID_DATA_TOO_LONG: u64 = 102;
    const EINAPPROPRIATE_VERSION: u64 = 103;
    const ENOT_GENESIS_ACCOUNT: u64 = 105;

    public fun initialize(storage_ctx: &mut StorageContext, account: &signer) {
        assert!(signer::address_of(account) == @rooch_blog, error::invalid_argument(ENOT_GENESIS_ACCOUNT));
        let _ = storage_ctx;
        let _ = account;
    }

    struct Article has key {
        version: u64,
        title: String,
        body: String,
    }

    /// get object id
    public fun id(article_obj: &Object<Article>): ObjectID {
        object::id(article_obj)
    }

    public fun version(article_obj: &Object<Article>): u64 {
        object::borrow(article_obj).version
    }

    public fun title(article_obj: &Object<Article>): String {
        object::borrow(article_obj).title
    }

    public(friend) fun set_title(article_obj: &mut Object<Article>, title: String) {
        object::borrow_mut(article_obj).title = title;
    }

    public fun body(article_obj: &Object<Article>): String {
        object::borrow(article_obj).body
    }

    public(friend) fun set_body(article_obj: &mut Object<Article>, body: String) {
        object::borrow_mut(article_obj).body = body;
    }

    fun new_article(
        _tx_ctx: &mut tx_context::TxContext,
        title: String,
        body: String,
    ): Article {
        assert!(std::string::length(&title) <= 200, EID_DATA_TOO_LONG);
        assert!(std::string::length(&body) <= 2000, EID_DATA_TOO_LONG);
        Article {
            version: 0,
            title,
            body,
        }
    }

    struct ArticleCreated has key {
        id: option::Option<ObjectID>,
        title: String,
        body: String,
    }

    public fun article_created_id(article_created: &ArticleCreated): option::Option<ObjectID> {
        article_created.id
    }

    public(friend) fun set_article_created_id(article_created: &mut ArticleCreated, id: ObjectID) {
        article_created.id = option::some(id);
    }

    public fun article_created_title(article_created: &ArticleCreated): String {
        article_created.title
    }

    public fun article_created_body(article_created: &ArticleCreated): String {
        article_created.body
    }

    public fun new_article_created(
        title: String,
        body: String,
    ): ArticleCreated {
        ArticleCreated {
            id: option::none(),
            title,
            body,
        }
    }

    struct ArticleUpdated has key {
        id: ObjectID,
        version: u64,
        title: String,
        body: String,
    }

    public fun article_updated_id(article_updated: &ArticleUpdated): ObjectID {
        article_updated.id
    }

    public fun article_updated_title(article_updated: &ArticleUpdated): String {
        article_updated.title
    }

    public fun article_updated_body(article_updated: &ArticleUpdated): String {
        article_updated.body
    }

    public(friend) fun new_article_updated(
        article_obj: &Object<Article>,
        title: String,
        body: String,
    ): ArticleUpdated {
        ArticleUpdated {
            id: id(article_obj),
            version: version(article_obj),
            title,
            body,
        }
    }

    struct ArticleDeleted has key {
        id: ObjectID,
        version: u64,
    }

    public fun article_deleted_id(article_deleted: &ArticleDeleted): ObjectID {
        article_deleted.id
    }

    public fun new_article_deleted(
        article_obj: &Object<Article>,
    ): ArticleDeleted {
        ArticleDeleted {
            id: id(article_obj),
            version: version(article_obj),
        }
    }

    public fun create_article(
        storage_ctx: &mut StorageContext,
        title: String,
        body: String,
    ): Object<Article> {
        let tx_ctx = storage_context::tx_context_mut(storage_ctx);
        let article = new_article(
            tx_ctx,
            title,
            body,
        );
        let obj_owner = tx_context::sender(tx_ctx);
        let article_obj = object::new(
            tx_ctx,
            obj_owner,
            article,
        );
        article_obj
    }

    public(friend) fun update_version_and_add(storage_ctx: &mut StorageContext, article_obj: Object<Article>) {
        object::borrow_mut(&mut article_obj).version = object::borrow( &mut article_obj).version + 1;
        //assert!(object::borrow(&article_obj).version != 0, EINAPPROPRIATE_VERSION);
        private_add_article(storage_ctx, article_obj);
    }

    public(friend) fun remove_article(storage_ctx: &mut StorageContext, obj_id: ObjectID): Object<Article> {
        let obj_store = storage_context::object_storage_mut(storage_ctx);
        object_storage::remove<Article>(obj_store, obj_id)
    }

    public(friend) fun add_article(storage_ctx: &mut StorageContext, article_obj: Object<Article>) {
        assert!(object::borrow(&article_obj).version == 0, EINAPPROPRIATE_VERSION);
        private_add_article(storage_ctx, article_obj);
    }

    fun private_add_article(storage_ctx: &mut StorageContext, article_obj: Object<Article>) {
        assert!(std::string::length(&object::borrow(&article_obj).title) <= 200, EID_DATA_TOO_LONG);
        assert!(std::string::length(&object::borrow(&article_obj).body) <= 2000, EID_DATA_TOO_LONG);
        let obj_store = storage_context::object_storage_mut(storage_ctx);
        object_storage::add(obj_store, article_obj);
    }

    public fun get_article(storage_ctx: &mut StorageContext, obj_id: ObjectID): Object<Article> {
        remove_article(storage_ctx, obj_id)
    }

    public fun return_article(storage_ctx: &mut StorageContext, article_obj: Object<Article>) {
        private_add_article(storage_ctx, article_obj);
    }

    public(friend) fun drop_article(article_obj: Object<Article>) {
        let (_id, _owner, article) =  object::unpack(article_obj);
        let Article {
            version: _version,
            title: _title,
            body: _body,
        } = article;
    }

    public(friend) fun emit_article_created(storage_ctx: &mut StorageContext, article_created: ArticleCreated) {
        event::emit_event(storage_ctx, article_created);
    }

    public(friend) fun emit_article_updated(storage_ctx: &mut StorageContext, article_updated: ArticleUpdated) {
        event::emit_event(storage_ctx, article_updated);
    }

    public(friend) fun emit_article_deleted(storage_ctx: &mut StorageContext, article_deleted: ArticleDeleted) {
        event::emit_event(storage_ctx, article_deleted);
    }
}
```

#### 4.3.3 Creating Articles

Next, we write the function to create a post in `blog.move`:

```move
    // === Create ===

    fun create_verify(
        storage_ctx: &mut StorageContext,
        account: &signer,
        title: String,
        body: String,
    ): article::ArticleCreated {
        let _ = storage_ctx;
        let _ = account;
        article::new_article_created(
            title,
            body,
        )
    }

    fun create_mutate(
        storage_ctx: &mut StorageContext,
        article_created: &article::ArticleCreated,
    ): Object<article::Article> {
        let title = article::article_created_title(article_created);
        let body = article::article_created_body(article_created);
        article::create_article(
            storage_ctx,
            title,
            body,
        )
    }

    public entry fun create(
        storage_ctx: &mut StorageContext,
        account: &signer,
        title: String,
        body: String,
    ) {
        let article_created = create_verify(
            storage_ctx,
            account,
            title,
            body,
        );
        let article_obj = create_mutate(
            storage_ctx,
            &article_created,
        );
        article::set_article_created_id(&mut article_created, article::id(&article_obj));
        article::add_article(storage_ctx, article_obj);
        article::emit_article_created(storage_ctx, article_created);
    }
```

A test article can be created by submitting a transaction using the Rooch CLI like this:

```shell
rooch move run --function 0x36a1c5014cb1771fb0689e041875c83a31675693301a9ba233932abc0b7e68dc::rooch_blog::create --sender-account 0x36a1c5014cb1771fb0689e041875c83a31675693301a9ba233932abc0b7e68dc --args 'string:Hello Rooch' "string:Accelerating World's Transition to Decentralization"
```

`--function` specifies to execute the `create` function in the `rooch_blog` module published at the address `0x36a1c5014cb1771fb0689e041875c83a31675693301a9ba233932abc0b7e68dc`, that is, create a new blog post. `--sender-account` specifies who should submit this transaction. This function requires us to pass two parameters to it, specified by `--args`, the first is the title of the article, I named it `Hello Rooch`; the second is the content of the article, I wrote the slogan of Rooch `Accelerating World's Transition to Decentralization`.

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
 "params":["0x36a1c5014cb1771fb0689e041875c83a31675693301a9ba233932abc0b7e68dc::article::ArticleCreated", null, 1000]
}'
```

The returned response content:

```shell
{"jsonrpc":"2.0","result":{"data":[{"event":{"event_id":{"event_handle_id":"0xf73d11468373bfcb25c0f6cc283f127a8dc8074da8bd9ba1ffe1c6f59c835404","event_seq":0},"type_tag":"0x36a1c5014cb1771fb0689e041875c83a31675693301a9ba233932abc0b7e68dc::article::ArticleCreated","event_data":"0x0190ba9f94b397111c779ab18647d5305c0c42843c33622f029da9093254b4f84b0b48656c6c6f20526f6f636833416363656c65726174696e6720576f726c642773205472616e736974696f6e20746f20446563656e7472616c697a6174696f6e","event_index":0},"sender":"0x0000000000000000000000000000000000000000000000000000000000000000","tx_hash":null,"timestamp_ms":null,"parsed_event_data":{"abilities":8,"type":"0x36a1c5014cb1771fb0689e041875c83a31675693301a9ba233932abc0b7e68dc::article::ArticleCreated","value":{"body":"Accelerating World's Transition to Decentralization","id":{"abilities":7,"type":"0x1::option::Option<0x2::object_id::ObjectID>","value":{"vec":["0x90ba9f94b397111c779ab18647d5305c0c42843c33622f029da9093254b4f84b"]}},"title":"Hello Rooch"}}}],"next_cursor":0,"has_next_page":false},"id":101}
```

Since there are many output contents, you can add a pipeline operation (` | jq '.result.data[0].parsed_event_data.value.id.value.vec[0]'`) at the end of the above command to quickly filter out the first `ObjectID` of an article.

> Tip: Before using the `jp` command (jq - commandline JSON processor), you may need to install it first.

The command after adding `jp` processing looks like this:

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

The object IDs of the blogs filtered by `jp` are:

```shell
"0x90ba9f94b397111c779ab18647d5305c0c42843c33622f029da9093254b4f84b"
```

Then, you can use the Rooch CLI to query the status of the object, passing `--id` to specify the ID of the article object (replace it with the ObjectID of your article):

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

Pay attention to the two key-value pairs `title` and `body` in the output, and you can see that this object does "store" the blog post you just wrote.

#### 4.3.5 Updating Articles

We continue to write the `blog.move` file and add the function of updating articles:

```move
    // === Update ===

    fun update_verify(
        storage_ctx: &mut StorageContext,
        account: &signer,
        title: String,
        body: String,
        article_obj: &Object<article::Article>,
    ): article::ArticleUpdated {
        let _ = storage_ctx;
        let _ = account;
        article::new_article_updated(
            article_obj,
            title,
            body,
        )
    }

    fun update_mutate(
        storage_ctx: &mut StorageContext,
        article_updated: &article::ArticleUpdated,
        article_obj: Object<article::Article>,
    ): Object<article::Article> {
        let title = article::article_updated_title(article_updated);
        let body = article::article_updated_body(article_updated);
        let id = article::article_updated_id(article_updated);
        let _ = storage_ctx;
        let _ = id;
        article::set_title(&mut article_obj, title);
        article::set_body(&mut article_obj, body);
        article_obj
    }

    public entry fun update(
        storage_ctx: &mut StorageContext,
        account: &signer,
        id: ObjectID,
        title: String,
        body: String,
    ) {
        let article_obj = article::remove_article(storage_ctx, id);
        let article_updated = update_verify(
            storage_ctx,
            account,
            title,
            body,
            &article_obj,
        );
        let updated_article_obj = update_mutate(
            storage_ctx,
            &article_updated,
            article_obj,
        );
        article::update_version_and_add(storage_ctx, updated_article_obj);
        article::emit_article_updated(storage_ctx, article_updated);
    }
```

Re-run `rooch server start`, compile the contract, and publish the contract to the chain again. After creating an article according to the steps of creating an article above, we try to use the `update` function to update the content of the article.

`--function` specifies to execute the `update` function in the `rooch_blog` module published at the address `0x36a1c5014cb1771fb0689e041875c83a31675693301a9ba233932abc0b7e68dc`, that is, to update a blog post. Also need to use `--sender-account` to specify the account that will send this update article transaction. This function requires us to pass three parameters to it, specified by `--args`, the first parameter is the object ID of the article to be modified, and the latter two parameters correspond to the title and body of the article respectively.

Change the title of the article to be `Foo` and the body of the article to be `Bar`:

```shell
rooch move run --function 0x36a1c5014cb1771fb0689e041875c83a31675693301a9ba233932abc0b7e68dc::rooch_blog::update --sender-account 0x36a1c5014cb1771fb0689e041875c83a31675693301a9ba233932abc0b7e68dc --args 'object_id:0x90ba9f94b397111c779ab18647d5305c0c42843c33622f029da9093254b4f84b' 'string:Foo' 'string:Bar'
```

In addition to using the Rooch CLI, you can also query the state of objects by calling JSON RPC:

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

In the output, it can be observed that the title and body of the article are successfully modified:

```shell
{"jsonrpc":"2.0","result":[{"state":{"value":"0x90ba9f94b397111c779ab18647d5305c0c42843c33622f029da9093254b4f84b36a1c5014cb1771fb0689e041875c83a31675693301a9ba233932abc0b7e68dc010000000000000003466f6f03426172fd1a25121453bfa91136bb7c089142f6a1aeb5d6ea13f23c238eade83f3ad31d","value_type":"0x2::object::Object<0x36a1c5014cb1771fb0689e041875c83a31675693301a9ba233932abc0b7e68dc::article::Article>"},"move_value":{"abilities":0,"type":"0x2::object::Object<0x36a1c5014cb1771fb0689e041875c83a31675693301a9ba233932abc0b7e68dc::article::Article>","value":{"id":"0x90ba9f94b397111c779ab18647d5305c0c42843c33622f029da9093254b4f84b","owner":"0x36a1c5014cb1771fb0689e041875c83a31675693301a9ba233932abc0b7e68dc","value":{"abilities":8,"type":"0x36a1c5014cb1771fb0689e041875c83a31675693301a9ba233932abc0b7e68dc::article::Article","value":{"body":"Bar","comments":{"abilities":4,"type":"0x2::table::Table<u64, 0x36a1c5014cb1771fb0689e041875c83a31675693301a9ba233932abc0b7e68dc::comment::Comment>","value":{"handle":"0xfd1a25121453bfa91136bb7c089142f6a1aeb5d6ea13f23c238eade83f3ad31d"}},"title":"Foo","version":"1"}}}}}],"id":101}[joe@mx rooch_blog]$
```

#### 4.3.6 Delete Article

We continue to write the blog.move file and add the function of deleting articles:

```move
    // === Delete ===

    fun delete_verify(
        storage_ctx: &mut StorageContext,
        account: &signer,
        article_obj: &Object<article::Article>,
    ): article::ArticleDeleted {
        let _ = storage_ctx;
        let _ = account;
        article::new_article_deleted(
            article_obj,
        )
    }

    fun delete_mutate(
        storage_ctx: &mut StorageContext,
        article_deleted: &article::ArticleDeleted,
        article_obj: Object<article::Article>,
    ): Object<article::Article> {
        let id = article::id(&article_obj);
        let _ = storage_ctx;
        let _ = id;
        let _ = article_deleted;
        article_obj
    }

    public entry fun delete(
        storage_ctx: &mut StorageContext,
        account: &signer,
        id: ObjectID,
    ) {
        let article_obj = article::remove_article(storage_ctx, id);
        let article_deleted = delete_verify(
            storage_ctx,
            account,
            &article_obj,
        );
        let updated_article_obj = delete_mutate(
            storage_ctx,
            &article_deleted,
            article_obj,
        );
        article::drop_article(updated_article_obj);
        article::emit_article_deleted(storage_ctx, article_deleted);
    }
```

A transaction can be submitted like this to delete articles:

```shell
rooch move run --function 0x36a1c5014cb1771fb0689e041875c83a31675693301a9ba233932abc0b7e68dc::rooch_blog::delete --sender-account 0x36a1c5014cb1771fb0689e041875c83a31675693301a9ba233932abc0b7e68dc --args 'object_id:0x90ba9f94b397111c779ab18647d5305c0c42843c33622f029da9093254b4f84b'
```

`--function` specifies to execute the `delete` function in the `rooch_blog` module published at the address `0x36a1c5014cb1771fb0689e041875c83a31675693301a9ba233932abc0b7e68dc`, that is, to delete a blog post. Also need to use `--sender-account` to specify the account to send this delete article transaction. This function only needs to pass one parameter to it, which is the object ID corresponding to the article, specified by `--args`.

#### 4.3.7 Check whether the article is deleted normally

```shell
rooch object --id 0x90ba9f94b397111c779ab18647d5305c0c42843c33622f029da9093254b4f84b

null
```

As you can see, when querying the object ID of the article, the result returns `null`. The description article has been deleted.

## 5. Summary

If you want to directly experience the functions of this blog contract, you can directly [download the blog source code](https://github.com/rooch-network/rooch/archive/refs/heads/main.zip), decompress it, and switch to the root directory of the blog contract project. For the interactive method, please refer to the above.

```shell
wget https://github.com/rooch-network/rooch/archive/refs/heads/main.zip
unzip main.zip
cd rooch-main/docs/website/public/codes/rooch_blog
```

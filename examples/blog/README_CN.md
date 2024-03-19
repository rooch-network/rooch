# README

[English](README.md) | 中文

本文主要介绍如何使用低代码工具来开发一个博客示例应用。

## 前提条件

目前 [dddappp](https://www.dddappp.org) 低代码工具以 Docker 镜像的方式发布，供开发者体验。

该工具所生成应用的链下服务使用 Java 语言编写，默认使用了 MySQL 数据库。但是本文不打算详细讲解链下服务的部署和测试，而是主要介绍如何使用 Rooch CLI 以及 jq 等命令行工具进行链上状态的查询以及对合约进行测试。

所以在开始体验前，你需要先：

* 安装 [Rooch CLI](https://github.com/rooch-network/rooch)。

* 安装 [Docker](https://docs.docker.com/engine/install/)。

* 安装 curl 以及 jp 命令（jp - commandline JSON processor）。我们在测试合约的时候可以使用 jp 来处理 JSON RPC 返回的 JSON 结果。

* （可选）安装 MySQL 数据库。可用于部署和测试链下服务。

* （可选）安装 JDK 和 Maven。用于构建和测试链下服务。

## 编码

你可以按照下面的介绍重现本示例应用的“编码”过程。你会发现要开发一个完整的应用，你只需要编写很少的代码。如果你的应用的业务逻辑只是对一些实体进行简单的 CRUD 操作，那么你甚至可能不需要编写除了“模型”之外的任何代码。

### 编写 DDDML 模型文件

我们介绍的低代码 dddappp 工具依赖 DDDML（领域驱动设计建模语言）描述的领域模型来生成应用的各部分代码。

> **提示**
>
> 关于 DDDML，这里有一篇入门的介绍文章：[《DDDML 简介：开启去中心化应用低代码开发的钥匙》](https://github.com/wubuku/Dapp-LCDP-Demo/blob/main/IntroducingDDDML_CN.md)。这篇文章包含了一些更复杂的 DDDML 示例模型文件的详细讲解。

你可以创建一个目录，比如叫做 `test`，来放置应用的所有代码，然后在该目录下面创建一个子目录 `dddml`。我们一般在这个目录下放置按照 DDDML 的规范编写的模型文件。

在 `dddml` 目录下创建一个纯文本文件，命名为 `blog.yaml`，文件内容如下：

```yaml
aggregates:
  Article:
    metadata:
      Preprocessors: [ "MOVE_CRUD_IT" ]
    id:
      name: Id
      type: ObjectID

    properties:
      Title:
        type: String
        length: 200
      Body:
        type: String
        length: 2000
      Comments:
        itemType: Comment
    entities:
      Comment:
        metadata:
          Preprocessors: [ "MOVE_CRUD_IT" ]
        id:
          name: CommentSeqId
          type: u64
        properties:
          Commenter:
            type: String
            length: 100
          Body:
            type: String
            length: 500
          Owner:
            type: address
```

上面的 DDDML 模型对于开发者而言，含义应该十分浅白，但是我们下面还是会略作解释。

这些代码定义了一个名为 `Article` 的聚合及同名聚合根实体，以及一个名为 `Comment` 的聚合内部实体。

相信稍有经验的开发者都已经了解“实体”这个概念。

如果你不太了解“聚合”、“聚合根”这些 DDD 的概念，也不要紧。你可以先把聚合大致理解为“具有紧密联系的聚合根实体以及聚合内部实体的集合”，以及认为“聚合根和聚合内部实体之间是‘我拥有你’的关系”，如此即可。

#### “文章”聚合

在 `/aggregates/Article/metadata` 这个键结点下，我们定义了一些元数据，用来指示生成代码时应用的一些预处理器。这里我们使用了 `MOVE_CRUD_IT` 这个预处理器，它的作用是自动实现实体的 CRUD 操作逻辑。

在 `/aggregates/Article/id` 这个键结点下，我们定义了文章聚合根的 ID。文章的 ID 的名字为 `Id`，类型为 `ObjectID`。这里的 `ObjectID` 是一个平台特定的类型，我们假设现在正在开发一个基于 Rooch 的去中心化应用。

在 `/aggregates/Article/properties` 这个键结点下，我们定义了文章的属性分别表示文章的标题、正文和评论。

文章的标题（Title）属性是一个类型为 String 的属性，长度限制为 200 个字符。

文章的正文（Body）属性是一个类型为 String 的属性，长度限制为 2000 个字符。

文章的评论（Comments）属性是一个由类型是 `Comment` 的元素所组成的集合（`itemType: Comment`）。这里的 `Comment` 是一个聚合内部实体。

#### “评论”实体

在 `/aggregates/Article/entities/Comment` 这个键结点下，我们定义了“评论”这个聚合内部实体。

在这里定义的评论（聚合内部实体）的 `id` 是个局部 ID（local ID），同样只要保证在同一篇文章内不同的评论之间这个 ID 的值具备唯一性就可以了。

我们将评论的 ID 命名为 `CommentSeqId`，声明其类型为 u64。

在 `/aggregates/Article/entities/Comment/metadata` 结点下我们也定义了一些元数据，同样使用了 `MOVE_CRUD_IT` 这个预处理器，让评论实体拥有自己的 CRUD 操作。

在 `/aggregates/Article/entities/Comment/properties` 结点下我们定义了评论的属性，分别表示评论者和评论内容。

评论者（Commenter）属性是一个类型为 `String` 的属性，长度限制为 100 个字符。

评论内容（Body）属性是一个类型为 `String` 的属性，长度限制为 500 个字符。

### 运行 dddappp 项目创建工具

使用 Docker 运行项目创建工具：

```shell
docker run \
-v /PATH/TO/test:/myapp \
wubuku/dddappp-rooch:0.0.1 \
--dddmlDirectoryPath /myapp/dddml \
--boundedContextName Test.RoochBlogDemo \
--roochMoveProjectDirectoryPath /myapp/move \
--boundedContextRoochPackageName RoochBlogDemo \
--boundedContextRoochNamedAddress rooch_examples \
--boundedContextJavaPackageName org.test.roochblogdemo \
--javaProjectsDirectoryPath /myapp/rooch-java-service \
--javaProjectNamePrefix roochblogdemo \
--pomGroupId test.roochblogdemo
```

上面的命令参数很直白：

* 注意将 `/PATH/TO/test` 替换为你实际放置应用代码的本机目录的路径。这一行表示将该本机目录挂载到容器内的 `/myapp` 目录。
* `dddmlDirectoryPath` 是 DDDML 模型文件所在的目录。它应该是容器内可以读取的目录路径。
* 把参数 `boundedContextName` 的值理解为你要开发的应用的名称即可。名称有多个部分时请使用点号分隔，每个部分使用 PascalCase 命名风格。Bounded-context 是领域驱动设计（DDD）中的一个术语，指的是一个特定的问题域范围，包含了特定的业务边界、约束和语言，这个概念你暂时不能理解也没有太大的关系。
* `roochMoveProjectDirectoryPath` 是放置链上 Rooch 合约代码的目录路径。它应该使用容器内可以读写的目录路径。
* `boundedContextRoochPackageName` 是链上 Rooch 合约的包名。建议采用 PascalCase 命名风格。
* `boundedContextRoochNamedAddress` 是链上 Rooch 合约默认的命名地址。建议采用 snake_case 命名风格。
* `boundedContextJavaPackageName` 是链下服务的 Java 包名。按照 Java 的命名规范，它应该全小写、各部分以点号分隔。
* `javaProjectsDirectoryPath` 是放置链下服务代码的目录路径。链下服务由多个模块（项目）组成。它应该使用容器内的可以读写的目录路径。
* `javaProjectNamePrefix` 是组成链下服务的各模块的名称前缀。建议使用一个全小写的名称。
* `pomGroupId` 链下服务的 GroupId，我们使用 Maven 作为链下服务的项目管理工具。它应该全小写、各部分以点号分隔。

上面的命令执行成功后，在本地目录 `/PATH/TO/test` 下应该会增加两个目录 `move` 以及 `rooch-java-service`。

### 项目源代码结构

进入 `move` 目录，这里放置的是从模型生成的 Move 合约项目。执行 Move 编译命令：

```shell
rooch move build --named-addresses rooch_examples={ACCOUNT_ADDRESS}
```

如果没有意外，合约项目可以构建成功（输出的最后一行应该显示 `Success`），但是此时应该存在一些编译警告。那是因为一些以 `_logic.move` 结尾的 Move 源代码中引入（`use`）了一些没有用到的模块。

此时你还可以尝试编译链下服务。进入目录 `rooch-java-service`，执行：`mvn compile`。如果没有意外，编译应该可以成功。

#### 合约项目源代码结构

在 `move/sources` 目录中，包含了链上合约项目的所有 Move 源代码。我们先忽略以 `_logic.move` 结尾的文件，介绍一下其他文件。

* `rooch_blog_demo_init.move`。它包含了链上合约的初始化（`initialize`）函数。一般来说，在合约项目部署到链上后，需要首先调用它（只需要调用一次）。不过，因为我们的示例项目比较简单，所以目前工具生成的 `initialize` 函数内没有包含什么有意义的初始化逻辑，我们可以先忽略它。
* `article_aggregate.move`。这是入口函数（entry functions）所在的地方。现在它包含的对文章和评论进行创建、更新、删除操作的函数。你可以看到，创建评论这个聚合内实体的函数被命名为 `add_comment` 而不是 `create_comment`，删除评论的函数被命名为 `remove_comment` 而不是 `delete_comment`，这其实是为了更容易在阅读时分辨出这些函数是对聚合内部实体的操作，而不是对聚合本身的操作。
* `article.move`。这个文件包含了“文章”这个聚合根实体的“数据模型”的定义，以及“文章”聚合相关的事件的定义。
* `comment.move`。这个文件包含了“评论”这个聚合内部实体的“数据模型”的定义。
* 下面列出的几个 Move 文件没有什么复杂的逻辑，只是提供了一些让你可以更便捷地获取事件属性（字段）值的函数。
  * `article_created.move`
  * `article_deleted.move`
  * `article_updated.move`
  * `comment_added.move`
  * `comment_removed.move`
  * `comment_updated.move`

#### 业务逻辑代码

以 `_logic.move` 结尾的 Move 源文件是“业务逻辑”实现代码所在之处。

如果你在 DDDML 文件中为聚合定义了一个方法（method），那么 dddappp 工具就会为你生成对应的一个名为 `{聚合名_方法名}_logic.move` 的 Move 代码文件，然后你需要在这个文件中填充“业务逻辑”的实现代码。

不过，上面我们使用的 `MOVE_CRUD_IT` 预处理器更进一步，直接为我们生成简单的 CRUD 方法的默认实现。当然，我们可以检查一下这些“填充好的默认逻辑”，视自己的需要修改它们。

使用上面的模型生成项目后，已经存在的“业务逻辑”代码文件是（可执行命令 `ls sources/*_logic.move` 列出）：

* `article_create_logic.move`
* `article_delete_logic.move`
* `article_update_logic.move`
* `article_add_comment_logic.move`
* `article_remove_comment_logic.move`
* `article_update_comment_logic.move`

现在就打开它们，移除那些多余的 `use` 语句。如果你的 IDE 安装了一些 Move 语言的插件，可能你只需要使用“格式化”功能对这几个源文件重新格式化一下即可。

然后使用 `rooch move build` 命令重新编译 Move 项目，现在应该没有警告信息了。

现在，你可以开始测试这个应用了。

## 测试应用

### 运行 Rooch Server 以及发布合约

首先，运行一个本地 Rooch 服务器：

```shell
rooch server start
```

发布 Move 合约：

```shell
rooch move publish --named-addresses rooch_examples={ACCOUNT_ADDRESS}
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

### 使用 CLI 工具测试合约

我们下面将会使用 Rooch CLI 以及其他命令行工具（`curl`、`jq`）来测试已发布的合约。

使用 `rooch move run` 命令提交一个交易，初始化合约（请注意替换占位符 `{ACCOUNT_ADDRESS}` 为你拥有账户的地址）：

```shell
rooch move run --function {ACCOUNT_ADDRESS}::rooch_blog_demo_init::initialize --sender-account {ACCOUNT_ADDRESS}
```

#### CRUD 文章

##### 创建文章

可以像下面这样，使用 Rooch CLI 提交一个交易，创建一篇测试文章：

```shell
rooch move run --function {ACCOUNT_ADDRESS}::article_aggregate::create --sender-account {ACCOUNT_ADDRESS} --args 'string:Hello' 'string:World!'
```

然后你可以更换一下 `--args` 后面的第一个参数（`title`）和第二个参数（`body`）的内容，多创建几篇文章。

##### 查询文章

现在，你可以通过查询事件，得到已创建好的文章的 `ObjectID`：

```shell
curl --location --request POST 'http://localhost:50051' \
--header 'Content-Type: application/json' \
--data-raw '{
 "id":101,
 "jsonrpc":"2.0",
 "method":"rooch_getEventsByEventHandle",
 "params":["{ACCOUNT_ADDRESS}::article::ArticleCreated", null, 1000]
}'
```

你可以在上面的命令最尾添加一个管道操作（` | jq '.result.data[0].parsed_event_data.value.id.value.vec[0]'`），来快速筛选出第一篇文章的 ObjectID。 

> **提示**
>
> 在使用 `jp` 命令（jq - commandline JSON processor）之前，你可能需要在本机上先安装它。

添加 `jp` 处理后的命令像下面这样：

```shell
curl --location --request POST 'http://localhost:50051' \
--header 'Content-Type: application/json' \
--data-raw '{
 "id":101,
 "jsonrpc":"2.0",
 "method":"rooch_getEventsByEventHandle",
 "params":["{ACCOUNT_ADDRESS}::article::ArticleCreated", null, 1000]
}' | jq '.result.data[0].parsed_event_data.value.id.value.vec[0]'
```

然后，你可以使用 Rooch CLI 来查询对象的状态（注意将占位符 `{ARTICLE_OBJECT_ID}` 替换为上面命令得到的文章的 ObjectID）：

```shell
rooch object --id {ARTICLE_OBJECT_ID}
```

##### 更新文章

可以这样提交一个交易，更新文章：

```shell
rooch move run --function {ACCOUNT_ADDRESS}::article_aggregate::update --sender-account {ACCOUNT_ADDRESS} --args 'object_id:{ARTICLE_OBJECT_ID}' 'string:Foo' 'string:Bar'
```

除了使用 Rooch CLI，你还可以通过调用 JSON RPC 来查询对象的状态：

```shell
curl --location --request POST 'http://127.0.0.1:50051/' \
--header 'Content-Type: application/json' \
--data-raw '{
 "id":101,
 "jsonrpc":"2.0",
 "method":"rooch_getAnnotatedStates",
 "params":["/object/{ARTICLE_OBJECT_ID}"]
}'
```

##### 删除文章

可以这样提交一个交易，删除文章：

```shell
rooch move run --function {ACCOUNT_ADDRESS}::article_aggregate::delete --sender-account {ACCOUNT_ADDRESS} --args 'object_id:{ARTICLE_OBJECT_ID}'
```

#### CRUD 评论

##### 添加评论

让我们再获取另一篇文章的 ObjectID（注意下面 `jq` 命令的路径参数 `.result.data[1]`，我们打算获取的是“第二个” `ArticleCreated` 事件的信息）：

```shell
curl --location --request POST 'http://localhost:50051' \
--header 'Content-Type: application/json' \
--data-raw '{
 "id":101,
 "jsonrpc":"2.0",
 "method":"rooch_getEventsByEventHandle",
 "params":["{ACCOUNT_ADDRESS}::article::ArticleCreated", null, 1000]
}' | jq '.result.data[1].parsed_event_data.value.id.value.vec[0]'
```

然后，我们可以使用这个文章的 ID，给它添加一个评论（注意替换占位符 `{ARTICLE_OBJECT_ID}` 为上面获取到的“第二篇”文章的 ObjectID）：

```shell
rooch move run --function {ACCOUNT_ADDRESS}::article_aggregate::add_comment --sender-account {ACCOUNT_ADDRESS} --args 'object_id:{ARTICLE_OBJECT_ID}' 'u64:1' 'string:Anonymous' 'string:"A test comment"' 'address:{ACCOUNT_ADDRESS}'
```

我们可以给这篇文章多添加几条评论，像下面这样执行命令（需要注意修改 `--args` 后面的第二个参数，该参数是评论的序号）：

```shell
rooch move run --function {ACCOUNT_ADDRESS}::article_aggregate::add_comment --sender-account {ACCOUNT_ADDRESS} --args 'object_id:{ARTICLE_OBJECT_ID}' 'u64:2' 'string:Anonymous2' 'string:"A test comment2"' 'address:{ACCOUNT_ADDRESS}'
```

##### 查询评论

在我们的合约代码中，当为一篇文章添加评论时，会 emit 一个 `CommentTableItemAdded` 事件，事件属性包含了当前文章的 ObjectID 以及添加到它的评论表的 key（即 `comment_seq_id`）。

所以，通过查询事件，我们知道一篇文章有那些评论：

```shell
curl --location --request POST 'http://localhost:50051' \
--header 'Content-Type: application/json' \
--data-raw '{
 "id":101,
 "jsonrpc":"2.0",
 "method":"rooch_getEventsByEventHandle",
 "params":["{ACCOUNT_ADDRESS}::article::CommentTableItemAdded", null, 10000]
}' | jq '.result.data[] | select(.parsed_event_data.value.article_id == "{ARTICLE_OBJECT_ID}")'
```

在我们的 Move 合约中，一篇文章的所有评论，是保存在嵌入在该文章对象的一个类型为 `Table<u64, Comment>` 的字段中的。

我们可以通过 JSON RPC 来查询评论的具体信息。获取评论表（comment table）中的项目（item）需要提供两个参数的值：table handle 以及 item key。

首先，我们要取得一篇文章的评论表的 handle：

```shell
curl --location --request POST 'http://127.0.0.1:50051/' \
--header 'Content-Type: application/json' \
--data-raw '{
 "id":101,
 "jsonrpc":"2.0",
 "method":"rooch_getAnnotatedStates",
 "params":["/object/{ARTICLE_OBJECT_ID}"]
}' | jq '.result[0].move_value.value.value.value.comments.value.handle'
```

我们已经知道上面已创建的一条评论的 `comment_seq_id`（即 table 的 item key）是类型为 u64 的整数值 `1`。 

那么，我们可以通过下面的方式获取的评论的具体信息（注意替换下面的占位符 `{COMMENT_TABLE_HANDLE}` 为上面获取到的“评论表”的 handle）：

```shell
curl --location --request POST 'http://127.0.0.1:50051/' \
--header 'Content-Type: application/json' \
--data-raw '{
 "id":101,
 "jsonrpc":"2.0",
 "method":"rooch_getAnnotatedStates",
 "params":["/table/{COMMENT_TABLE_HANDLE}/0x0100000000000000"]
}'
```

注意上面的命令，路径参数中的 table key（在 `{COMMENT_TABLE_HANDLE}/` 之后的那部分），是以十六进制字符串表示的 key 值的 BCS 序列化的结果。

比如，类型为 `u64` 的整数值 `1` 的 BCS 序列化结果，以十六进制字符串表示为 `0x0100000000000000`。

##### 更新评论

我们可以这样提交一个交易，更新评论：

```shell
rooch move run --function {ACCOUNT_ADDRESS}::article_aggregate::update_comment --sender-account {ACCOUNT_ADDRESS} --args 'object_id:{ARTICLE_OBJECT_ID}' 'u64:1' 'string:Anonymous' 'string:"Updated test comment"' 'address:{ACCOUNT_ADDRESS}'
```

然后我们可以再次查询评论的状态，看看评论内容是否已经更新：

```shell
curl --location --request POST 'http://127.0.0.1:50051/' \
--header 'Content-Type: application/json' \
--data-raw '{
 "id":101,
 "jsonrpc":"2.0",
 "method":"rooch_getAnnotatedStates",
 "params":["/table/{COMMENT_TABLE_HANDLE}/0x0100000000000000"]
}'
```

##### 移除评论

提及一个交易，移除评论：

```shell
rooch move run --function {ACCOUNT_ADDRESS}::article_aggregate::remove_comment --sender-account {ACCOUNT_ADDRESS} --args 'object_id:{ARTICLE_OBJECT_ID}' 'u64:1'
```

再次执行上面的 curl 命令查询评论，这次会返回类似这样的信息：

```json
{"jsonrpc":"2.0","result":[null],"id":101}
```

~~因为我们后面这篇文章还有未被删除的评论，所以如果现在想要删除它，应该不会成功。尝试执行~~：

```shell
rooch move run --function {ACCOUNT_ADDRESS}::article_aggregate::delete --sender-account {ACCOUNT_ADDRESS} --args 'object_id:{ARTICLE_OBJECT_ID}'
```

返回的交易执行状态应该是失败的：

```json
//[TBD]
```

## 使用链下服务

通过查询链下服务的 RESTful API，你可以更容易地查询到文章和评论的具体信息，而不需要使用上面介绍的 curl 和 jp 命令。

> **提示**
>
> 由于链下服务的代码可以通过 dddml 工具快速重新生成，所以在这个代码库里，我们并没有包含链下服务的代码。

### 配置和启动链下服务

#### 修改配置文件

打开位于目录 `rooch-java-service/roochblogdemo-service-rest/src/main/resources` 下的 `application-test.yml` 文件，找到类似下面的几行，将占位符 `{ACCOUNT_ADDRESS}` 替换为你的账户地址：

```yaml
rooch:
  contract:
    address: "{ACCOUNT_ADDRESS}"
    jsonrpc:
      url: "http://127.0.0.1:50051"
```

这是链下服务唯一必需配置的地方，就是这么简单。


#### 创建链下服务的数据库

如果你已经安装了 Docker，可以使用 Docker 来运行一个 MySQL 数据库服务。比如：

```shell
sudo docker run -p 3306:3306 --name mysql \
-v ~/docker/mysql/conf:/etc/mysql \
-v ~/docker/mysql/logs:/var/log/mysql \
-v ~/docker/mysql/data:/var/lib/mysql \
-e MYSQL_ROOT_PASSWORD=123456 \
-d mysql:5.7
```

注意，上面的命令中我们将数据库 `root` 账号的密码为 `123456`。下面示例的 shell 命令和 Off-chain 服务的配置中我们直接使用这个 root 账号/密码。你可以视你的运行环境修改它们。

使用 MySQL 客户端连接本地的 MySQL 服务器，执行以下脚本创建一个空的数据库（假设名称为 `test2`）：

```sql
CREATE SCHEMA `test2` DEFAULT CHARACTER SET utf8mb4 COLLATE utf8mb4_bin;
```

进入 `rooch-java-service` 目录，打包 Java 项目：

```shell
mvn package
```

然后运行一个命令行工具，初始化数据库：

```shell
java -jar ./roochblogdemo-service-cli/target/roochblogdemo-service-cli-0.0.1-SNAPSHOT.jar ddl -d "./scripts" -c "jdbc:mysql://127.0.0.1:3306/test2?enabledTLSProtocols=TLSv1.2&characterEncoding=utf8&serverTimezone=GMT%2b0&useLegacyDatetimeCode=false" -u root -p 123456
```

#### 启动链下服务

在 `rooch-java-service` 目录下，执行以下命令启动链下服务：

```shell
mvn -pl roochblogdemo-service-rest -am spring-boot:run
```

> **提示**
>
> 你可以使用这个 Rooch Move CLI 速查表！
> 
> 在链下服务启动后，你可以访问这个网址，得到一个如何使用 Rooch Move CLI 调用链上合约的速查表（Cheatsheet）：http://localhost:1023/api/rooch.contract/RoochMoveCLICheatsheet.md
>
> 在 Cheatsheet 中已经把刚才发布的 Move 合约的地址帮你填好了。你需要填写的参数是一些“有名字”的占位符。 你可以拷贝这些命令，视你的需要稍作修改，然后在命令行终端中直接执行。

### 链下服务的 RESTful API

现在，你可以访问这个 RESTful API 获取已创建的文章的列表：

```shell
curl http://localhost:1023/api/Articles
```

可以使用参数过滤文章列表，比如：

```shell
curl 'http://localhost:1023/api/Articles?title=Hello'
```

你可以这样访问单篇文章的信息：

```shell
curl 'http://localhost:1023/api/Articles/{ARTICLE_OBJECT_ID}'
```

## 改进应用

在上面的过程中，`MOVE_CRUD_IT` 预处理器已经为我们生成了完整的 CRUD 方法。如果 CRUD 是你需要的所有业务逻辑，那么你不需要再写一行代码。

当然，在开发一个真实的应用时，事情往往不会这么简单。下面我们接着探讨一下，如何从几个方面改进上面的例子，使它更接近“实际的业务需求”。

### 修改添加评论的方法

有可能你觉得默认生成的 CRUD 逻辑不符合你的需求，比如，你可能想要添加评论时不需要传递 `Owner` 参数给 `entry fun add_comment`，而是直接使用发送者的账户地址作为 Owner，那么这个需求目前可以这样满足：

首先，在模型文件中像这样自定义一个方法：

```yaml
aggregates:
  Article:
    # ...
    methods:
      AddComment:
        event:
          name: CommentAdded
          properties:
            Owner:
              type: address
        parameters:
          CommentSeqId:
            type: u64
          Commenter:
            type: String
          Body:
            type: String
```

注意，上面的方法参数列表中已经没有 `Owner` 参数。

然后，删除 `article_add_comment_logic.move` 文件，再次运行 dddappp 工具。（注意，因为工具默认不会覆盖已经存在的 `*_logic.move` 文件，所以你需要手动删除它。）

打开重新生成的 `article_add_comment_logic.move` 文件中，找到 `verify` 函数，在函数体中填充你想要的业务逻辑代码。事实上你要做的可能只是在 `verify` 函数的最后添加这样一行代码：

```
    public(friend) fun verify(
        // ...
    ): article::CommentAdded {
        // ...
            body,
            // 添加下面这行代码
            std::signer::address_of(account),
        )
    }
```

### 增加一个单例对象 Blog

我们打算增加一个（只有一个实例的）单例对象 Blog，它有一个属性 `Name`，和一个属性 `Articles`，`Articles` 是一个 `ObjectID` 的数组，表示博客中包含的文章。

在 `dddml/blog.yaml` 文件中，增加一个单例对象的定义：

```yaml
singletonObjects:
  Blog:
    metadata:
      Preprocessors: [ "MOVE_CRUD_IT" ]
    properties:
      Name:
        type: String
        length: 200
      Articles:
        itemType: ObjectID
    methods:
      AddArticle:
        event:
          name: ArticleAddedToBlog
        parameters:
          ArticleId:
            type: ObjectID
      RemoveArticle:
        event:
          name: ArticleRemovedFromBlog
        parameters:
          ArticleId:
            type: ObjectID
```

再次运行 dddappp 工具。

打开生成的 `blog_add_article_logic.move` 文件，填充业务逻辑代码：

```
public(friend) fun verify(
    _account: &signer,
    article_id: ObjectID, blog: &blog::Blog,
): blog::ArticleAddedToBlog {
    blog::new_article_added_to_blog(
        blog, article_id,
    )
}

public(friend) fun mutate(
    _account: &signer,
    article_added_to_blog: &blog::ArticleAddedToBlog,
    blog: blog::Blog,
): blog::Blog {
    let article_id = article_added_to_blog::article_id(article_added_to_blog);
    let articles = blog::articles(&blog);
    if (!vector::contains(&articles, &article_id)) {
        vector::push_back(&mut articles, article_id);
        blog::set_articles(&mut blog, articles);
    };
    blog
}
```

打开生成的 `blog_remove_article_logic.move` 文件，填充业务逻辑代码：

```
    public(friend) fun verify(
        _account: &signer,
        article_id: ObjectID, blog: &blog::Blog,
    ): blog::ArticleRemovedFromBlog {
        blog::new_article_removed_from_blog(
            blog, article_id,
        )
    }

    public(friend) fun mutate(
        _account: &signer,
        article_removed_from_blog: &blog::ArticleRemovedFromBlog,
        blog: blog::Blog,
    ): blog::Blog {
        let article_id = article_removed_from_blog::article_id(article_removed_from_blog);
        let articles = blog::articles(&blog);
        let (found, idx) = vector::index_of(&articles, &article_id);
        if (found) {
            vector::remove(&mut articles, idx);
            blog::set_articles(&mut blog, articles);
        };
        blog
    }
```

### 修改创建文章的逻辑

打开文件 `article_create_logic.move`，找到 `mutate` 函数，修改它的实现，使它能够把新创建的文章添加到 Blog 对象的 `Articles` 属性中。

```
    public(friend) fun mutate(
        //...
    ): Object<article::Article> {
        let title = article_created::title(article_created);
        let body = article_created::body(article_created);
        let article_obj = article::create_article(
            
            title,
            body,
        );
        blog_aggregate::add_article(_account, article::id(&article_obj));
        article_obj
    }
```

### 修改删除文章的逻辑

打开文件 `article_delete_logic.move`，找到 `mutate` 函数，修改它的实现，使它能够把被删除的文章从 Blog 对象的 `Articles` 属性中移除。

```
    public(friend) fun mutate(
        //...
    ): Object<article::Article> {
        let _ = article_deleted;
        blog_aggregate::remove_article(_account, article::id(&article_obj));
        article_obj
    }
```

### 修改更新文章的逻辑

打开文件 `article_update_logic.move`，找到 `verify` 函数，修改它的实现，检查调用者是否是文章的所有者。

```
    const ENOT_OWNER_ACCOUNT: u64 = 113;

    public(friend) fun verify(
        //...
    ): article::ArticleUpdated {
        
        assert!(signer::address_of(account) == object::owner(article_obj), ENOT_OWNER_ACCOUNT);
        article::new_article_updated(
            article_obj,
            title,
            body,
        )
    }
```

### 修改移除评论和更新评论的逻辑

打开文件 `article_update_comment_logic.move`，找到 `verify` 函数，修改它的实现，检查调用者是否是评论的所有者。

```
    const ENOT_OWNER_ACCOUNT: u64 = 113;

    public(friend) fun verify(
        //...
    ): article::CommentUpdated {
        
        let comment = article::borrow_comment(article_obj, comment_seq_id);
        assert!(std::signer::address_of(account) == comment::owner(comment), ENOT_OWNER_ACCOUNT);
        article::new_comment_updated(
            //...
        )
    }
```

打开文件 `article_remove_comment_logic.move`，找到 `verify` 函数，修改它的实现，检查调用者是否是评论的所有者。

```
    const ENOT_OWNER_ACCOUNT: u64 = 113;

    public(friend) fun verify(
        //...
    ): article::CommentRemoved {
        
        let comment = article::borrow_comment(article_obj, comment_seq_id);
        assert!(std::signer::address_of(account) == comment::owner(comment), 111);
        article::new_comment_removed(
            //...
        )
    }
```

### 测试改进后的应用

在增加了 Blog 这个单例对象后，在添加文章之前，需要先将它初始化：

```shell
rooch move run --function {ACCOUNT_ADDRESS}::blog_aggregate::create --sender-account {ACCOUNT_ADDRESS} --args 'string:My Blog' 'vector<object_id>:'
```

另外，添加评论时不再需要传入 `Owner` 参数：

```shell
rooch move run --function {ACCOUNT_ADDRESS}::article_aggregate::add_comment --sender-account {ACCOUNT_ADDRESS} --args 'object_id:{ARTICLE_OBJECT_ID}' 'u64:1' 'string:Anonymous' 'string:"A test comment"'
```

现在，添加文章之后，你可以这样查询 Blog 的状态：

```shell
rooch state --access-path /resource/{ACCOUNT_ADDRESS}/{ACCOUNT_ADDRESS}::blog::Blog
```

在返回的结果中，应该可以看到博客文章的 `ObjectID` 的列表。

## 再次改进应用

你可能已经发现，这个应用目前还有一些地方不如人意：

* 我们在 `Blog` 对象内定义 `AddArticle` 和 `RemoveArticle` 只是打算供内部使用，没有必要在 `blog_aggregate.move` 文件中把它们对应的 `add_article` 和 `remove_article` 函数声明为 `public entry fun`。
* 现在我们无法使用另外一个账户来创建博客文章。我们检视一下 `add_article` 和 `remove_article` 函数的实现，发现生成的代码默认使用了类型为 `&signer` 的参数来操作签名者的账户资源中的 `Blog` 对象；然而，想要实现这个两个方法的业务逻辑（在已有的 `Blog` 对象中添加和删除文章的 `ObjectID`），代码完全可以采用另外一种写法。
* 在添加评论的时候，需要传入 `CommentSeqId` 参数作为评论的局部 ID，这个参数的值是由调用者自己指定的。要是评论的 ID 能够自动生成就更好了。

现在让我们来解决这些问题。

### 修改模型文件

#### 修改单例对象 Blog

打开 `blog.yaml` 模型文件，按照下面注释的提示，修改 `Blog` 这个单例对象的定义：

```yaml
singletonObjects:
  Blog:
    # 下面这行代码是新增的
    friends: [ "Article.Create", "Article.Delete" ]
    metadata:
      # ...
    methods:
      AddArticle:
        # 下面三行代码是新增的
        isInternal: true
        metadata:
          NoSigner: true
        event:
            # ...
      RemoveArticle:
        # 下面三行代码是新增的
        isInternal: true
        metadata:
          NoSigner: true
        event:
          # ...
```

#### 修改评论实体

找到模型文件中的 `Comment` 实体的定义，按照下面注释的提示，添加几行代码：

```yaml
    entities:
      Comment:
        # ...
        id:
          name: CommentSeqId
          type: u64
          # 下面这三行代码是新增的
          generator:
            class: sequence
            structName: CommentSeqIdGenerator
```

然后，按照下面注释的提示，移除或者注释掉 `AddComment` 方法中的 `CommentSeqId` 参数的定义：

```yaml
      AddComment:
        # ...
        parameters:
          # 移除或者注释掉下面这两行代码
          # CommentSeqId:
          #   type: u64
          Commenter:
            type: String
```

### 重新生成代码、填充业务逻辑

删除 `blog_add_article_logic.move`，`blog_remove_article_logic.move` 和 `article_add_comment_logic.move` 这三个文件。重新运行 dddappp 工具重新生成代码。

打开 `blog_add_article_logic.move` 文件，填充业务逻辑代码：

```
    public(friend) fun verify(
        article_id: ObjectID,
        blog: &blog::Blog,
    ): blog::ArticleAddedToBlog {
        blog::new_article_added_to_blog(
            blog,
            article_id,
        )
    }

    public(friend) fun mutate(
        article_added_to_blog: &blog::ArticleAddedToBlog,
        blog: &mut blog::Blog,
    ) {
        let article_id = article_added_to_blog::article_id(article_added_to_blog);
        let articles = blog::articles(blog);
        if (!vector::contains(&articles, &article_id)) {
            vector::push_back(&mut articles, article_id);
            blog::set_articles(blog, articles);
        };
    }
```

打开 `blog_remove_article_logic.move` 文件，填充业务逻辑代码：

```
    public(friend) fun verify(
        article_id: ObjectID,
        blog: &blog::Blog,
    ): blog::ArticleRemovedFromBlog {
        blog::new_article_removed_from_blog(
            blog,
            article_id,
        )
    }

    public(friend) fun mutate(
        article_removed_from_blog: &blog::ArticleRemovedFromBlog,
        blog: &mut blog::Blog,
    ) {
        let article_id = article_removed_from_blog::article_id(article_removed_from_blog);
        let articles = blog::articles(blog);
        let (found, idx) = vector::index_of(&articles, &article_id);
        if (found) {
            vector::remove(&mut articles, idx);
            blog::set_articles(blog, articles);
        };
    }
```


打开重新生成的 `article_add_comment_logic.move` 文件中，找到 `verify` 函数，在函数体中填充你想要的业务逻辑代码。事实上你要做的可能只是在 `verify` 函数的最后添加这样一行代码：

```
    public(friend) fun verify(
        // ...
    ): article::CommentAdded {
        // ...
            body,
            // 添加下面这行代码
            std::signer::address_of(account),
        )
    }
```

### 修改创建文章的逻辑

打开文件 `article_create_logic.move`，将下面这行代码：

```
        blog_aggregate::add_article(_account, article::id(&article_obj));
```

修改为：

```
        blog_aggregate::add_article(article::id(&article_obj));
```

### 修改删除文章的逻辑

打开文件 `article_delete_logic.move`，将下面这行代码：

```
        blog_aggregate::remove_article(_account, article::id(&article_obj));
```

修改为：

```
        blog_aggregate::remove_article(article::id(&article_obj));
```

### 测试再次改进后的应用

重启 Rooch Server，重新发布应用。

现在，你可以使用另外一个账户来创建博客文章了（将下面的占位符 `{ANOTHER_ACCOUNT_ADDRESS}` 替换为你的另一个账户的地址）：

```shell
rooch move run --function {ACCOUNT_ADDRESS}::article_aggregate::create --sender-account {ANOTHER_ACCOUNT_ADDRESS} --args 'string:Hello' 'string:World2!'
```

---

最终修改后的模型文件和 Move 合约代码，已经上传到了本代码库。模型文件见 [dddml/blog.yaml](./dddml/blog.yaml)，代码见 [sources/](./sources/) 目录。

## 其他

### 更复杂的 Rooch Demo

如果你有兴趣，可以在这里找到一个更复杂的 Rooch Demo：["A Rooch Demo"](https://github.com/dddappp/A-Rooch-Demo#configure-off-chain-service)。


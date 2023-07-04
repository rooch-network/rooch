# README

[English](README.md) | 中文

本文主要介绍使用如何使用低代码工具来开发一个博客示例应用。

## 前提条件

目前 dddappp 低代码工具以 Docker 镜像的方式发布，供开发者体验。

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
      Preprocessors: ["MOVE_CRUD_IT"]
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
          Preprocessors: ["MOVE_CRUD_IT"]
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
rooch move run --function {ACCOUNT_ADDRESS}::article_aggregate::add_comment --sender-account {ACCOUNT_ADDRESS} --args 'object_id:{ARTICLE_OBJECT_ID}' 'u64:1' 'string:Anonymous' 'string:"A test comment"'
```

我们可以给这篇文章多添加几条评论，像下面这样执行命令（需要注意修改 `--args` 后面的第二个参数，该参数是评论的序号）：

```shell
rooch move run --function {ACCOUNT_ADDRESS}::article_aggregate::add_comment --sender-account {ACCOUNT_ADDRESS} --args 'object_id:{ARTICLE_OBJECT_ID}' 'u64:2' 'string:Anonymous2' 'string:"A test comment2"'
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
rooch move run --function {ACCOUNT_ADDRESS}::article_aggregate::update_comment --sender-account {ACCOUNT_ADDRESS} --args 'object_id:{ARTICLE_OBJECT_ID}' 'u64:1' 'string:Anonymous' 'string:"Updated test comment"'
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

### One more thing

如果你有兴趣，可以参考 ["A Rooch Demo"](https://github.com/dddappp/A-Rooch-Demo#configure-off-chain-service) 的介绍，配置目录 `rooch-java-service` 下的 Java 链下服务，然后将服务运行起来。

通过查询链下服务的 RESTful API，你可以更容易地查询到文章和评论的具体信息，而不需要使用上面介绍的 curl 和 jp 命令。


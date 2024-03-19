# 为 Rooch 发布第一份 PR

Rooch 是一个开源项目，每个人都可以贡献自己的代码，将自己的创意变成现实。本文将帮助新贡献者了解如何在 GitHub 上为 Rooch 创建 PR，并提供代码贡献的实用注意事项。

## 前提条件

Rooch 由 Rust 编写，要从头开始构建 Rooch，您需要安装以下工具：

* Git
* 使用 [rustup](https://rustup.rs/) 安装 Rust

### Pull Request

### 提交 PR

1. Fork `rooch` 仓库并从 `main` 创建你的分支。
2. 打开一个常规的 [issue](https://github.com/rooch-network/rooch/issues/new) 来绑定 PR。
3. 提交一个 [Draft Pull Requests](https://github.blog/2019-02-14-introducing-draft-pull-requests/)，标记你的工作进展。
4. 如果您添加了需要测试的代码，请添加单元测试。
5. 将状态更改为 "Ready for review"。

### PR 标题

格式： `<类型>(<范围>)： <主题>`。

`<范围>`为可选项

```
feat(rooch-da): add lifecycle in put policy
^--^  ^------------^
|     |
|     +-> Summary in present tense.
|
+-------> Type: rfc, feat, fix, refactor, ci, docs, chore
```

类型：

* `rfc`：该 PR 提出了一个新的 RFC
* `feat`：该 PR 为代码库引入了一个新功能
* `fix`: 此 PR 修补了代码库中的一个错误
* `refactor`: 此 PR 会更改代码库，但不会引入新功能或修复错误。
* `ci`：此 PR 会更改构建/ci 步骤
* `docs`：此 PR 会更改文档或网站
* `chore`: 此 PR 仅有无需记录的小改动，如编码样式。

### PR 模板

Rooch 有一个 [Pull Request Template](.github/PULL_REQUEST_TEMPLATE.md):

```
## Summary

Summary about this PR

Fixes #issue
```

您不应更改 PR 模板上下文，但需要完成：

* `Summary` - 描述构成该 Pull Request 的内容，以及您对代码所做的更改。例如，修复了哪个问题。

## 问题

Rooch 使用 [GitHub issues](https://github.com/rooch-network/rooch/issues) 跟踪错误。请包含必要的信息和说明，以便重现您的问题。

## 文档

所有开发者文档均发布在 Rooch 开发者网站 [rooch.network](https://rooch.network/learn/introduction)。

## 行为准则

请参阅 [行为准则](CODE_OF_CONDUCT.md)。

# Your First PR for Rooch

Rooch is an open-source project where everyone can contribute their code and make their creative ideas into a reality. This topic helps new contributors understand how to create a pull request for Rooch on GitHub and provides useful considerations for code contributions.

## Prerequisites

Rooch is written in Rust, to build Rooch from scratch you will need to install the following tools:

* Git
* Rust install with [rustup](https://rustup.rs/)

## Pull Requests

### Submit a PR

1. Fork the `rooch` repo and create your branch from `main`.
2. Open a regular [issue](https://github.com/rooch-network/rooch/issues/new) for binding the pull request.
3. Submit a [Draft Pull Requests](https://github.blog/2019-02-14-introducing-draft-pull-requests/), tag your work in progress.
4. If you have added code that should be tested, add unit tests.
5. Change the status to “Ready for review”.

### PR Title

Format: `<type>(<scope>): <subject>`

`<scope>` is optional

```
feat(rooch-da): add lifecycle in put policy
^--^  ^------------^
|     |
|     +-> Summary in present tense.
|
+-------> Type: rfc, feat, fix, refactor, ci, docs, chore
```

Types:

* `rfc`: this PR proposes a new RFC
* `feat`: this PR introduces a new feature to the codebase
* `fix`: this PR patches a bug in codebase
* `refactor`: this PR changes the code base without new features or bugfix
* `ci`: this PR changes build/ci steps
* `docs`: this PR changes the documents or websites
* `chore`: this PR only has small changes that no need to record, like coding styles.

### PR Template

Rooch has a [Pull Request Template](.github/PULL_REQUEST_TEMPLATE.md):

```
## Summary

Summary about this PR

Fixes #issue
```

You should not change the PR template context, but need to finish:

* `Summary` - Describes what constitutes the Pull Request and what changes you have made to the code. For example, fixes which issue.

## Issues

Rooch uses [GitHub issues](https://github.com/rooch-network/rooch/issues) to track bugs. Please include necessary information and instructions to reproduce your issue.

## Documentations

All developer documentation is published on the Rooch developer site, [rooch.network](https://rooch.network/).

## Code of Conduct

Please refer to the [Code of Conduct](CODE_OF_CONDUCT.md), which describes the expectations for interactions within the community.
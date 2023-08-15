# Contributing to Rooch

Thank you for your interest in contributing to Rooch! There are many ways to contribute and we appreciate all of them.

- Communicating in our [Discord channel](https://discord.gg/kgXEmHGB), share your new ideas, talk technologies
- Learning Rooch
- Reporting bugs
- Requesting new features
- Submitting PRs

## Contributing via GitHub

To contribute to the Rooch source code or documentation, you need a GitHub account.

There are usually two ways to submit a PR:

- One is to edit directly in the repository on the GitHub website, and after submitting the changes, push them directly to the main branch. This method is especially suitable for submitting simple PRs such as correcting typos. It is not recommended to use this method when multiple files are involved or there are many changes.

- Another way is to `fork` the Rooch repository under your own account, and `clone` it locally, and then push it to the upstream `main` branch (referring to the Rooch organization) after the modification is completed. The branch on your own GitHub is called a remote branch.

## Detailed operation process of method 2

### Create a new fork

First, create a fork of the `rooch` repository in your own account so that you can use your own copy.

1. Log in to your Github account.
2. Browse the [Rooch repo](https://github.com/rooch-network/rooch) on GitHub.
3. Select `Fork` in the upper right corner, then select `Create a new fork`.
4. For **Owner**, select your username.
5. For the **Repository name**, we recommend keeping the name `rooch`, but you can use any name.
6. Optional. To contribute, you only need the main branch of the repository. To include all branches, uncheck the checkbox for **Copy the `main` branch only**.
7. Click `Create fork`.

### Clone your fork

Next, clone the fork of the repository to your local repository.

1. Open the repository page of your fork, and click the `Sync fork` button (no operation is usually required for the fork just now, and synchronization is only required if your forked repository commits lags behind the upstream repository).
2. Click on `Code`, then click on `HTTPS` and copy the displayed web URL.
3. Open a terminal session and navigate to the folder you want to use, then run the following command, replacing the URL with the one you copied from the GitHub page:

```shell
git clone https://github.com/<GITHUB-USER-NAME>/rooch.git` 
```

### Create a new branch

After cloning is completed, you can modify any file in the `rooch` directory. By default, the clone is the default branch (can be set in GitHub), usually the `main` branch.

Before making corresponding changes to Rooch's project, you need to perform **the most important step**: creating a new topic branch.

In general, it is not recommended to submit a PR to Rooch's upstream directly using the `main` branch, which is not conducive to the collaboration of Rooch maintainers with you.

## Create a new pull request


When you have completed the modification, submitted and pushed the changes to the remote repository, you can usually see a `Compare & pull request` pop-up button on the GitHub page, just click and fill in the corresponding information.

The content is usually filled in automatically. If you feel that the title or the content description of this PR is not accurate enough, you can continue to modify it.

After the modification is complete, click the `Create pull request` button below, and your PR will appear on Rooch's `Pull requests` page.

## Submitting new issues

Reporting issues and submitting feature requests is usually done by submitting corresponding issue posts on the `Issues` page.

**When you want to report a bug**, please open [Rooch's GitHub Issues page](https://github.com/rooch-network/rooch/issues). In the search bar above, simply search for your question, maybe the question you found may have been submitted by others, so avoid duplicating submissions.If there is no bug post you want to report, please click the `New issue` button on the upper right, enter a brief description of the current bug as the title, and fill in the **detailed description** of the current bug in the content box, including *your operating (system) environment*, *the version of Rooch used*, and *the process of reproducing the problem*, etc.

**When you want to request a new feature**, fill in the brief description of the feature on the title bar of the current Issues page as the title, and add `[Feature Request]` as the identifier before the title.

After filling in the relevant information, click on `Lables` on the right, label it accordingly, and click on the `Submit new issue` button below.

## Code of Conduct

Please refer to the [Code of Conduct](CODE_OF_CONDUCT.md), which describes the expectations for interactions within the community.
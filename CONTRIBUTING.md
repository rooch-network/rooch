
# Contributing

Our goal is to make contributing to Rooch Network easy and transparent.

## Install Rooch to contribute

To contribute to Rooch source code or documentation, you need only a GitHub account. You can commit updates and then submit a PR directly from the Github website, or create a fork of the repo to your local environment and use your favorite tools to make changes. Always submit PRs to the `main` branch.

### Create a fork

First, create a fork of the Rooch Network Rooch repo in your own account so that you can work with your own copy.

**To create a fork using the website**

1. Log in to your Github account.
1. Browse to the [Rooch repo](https://github.com/rooch-network/rooch) on GitHub.
1. Choose **Fork** in the top-right, then choose **Create new fork**.
1. For **Owner**, select your username.
1. For **Repository name**, we suggest keeping the name rooch, but you can use any name. 
1. Optional. To contribute you need only the main branch of the repo. To include all branches, unselect the checkbox for **Copy the `main` branch only**.
1. Click **Create fork**.

### Clone your fork

Next, clone your fork of the repo to your local workspace.

**To clone your fork to your local workspace**
1. Open the GitHub page for your fork of the repo, then click **Sync fork**.
1. Click **Code**, then click **HTTPS** and copy the web URL displayed.
1. Open a terminal session and navigate to the folder to use, then run the following command, replacing the URL with the URL you copied from the Git page:

`git clone https://github.com/github-user-name/rooch.git`

The repo is automatically cloned into the `rooch` folder in your workspace.
Create a branch of your fork with following command (or follow the [GitHub topic on branching](https://docs.github.com/en/pull-requests/collaborating-with-pull-requests/proposing-changes-to-your-work-with-pull-requests/creating-and-deleting-branches-within-your-repository))

`git checkout -b your-branch-name`

Use the following command to set the [remote upstream repo](https://docs.github.com/en/pull-requests/collaborating-with-pull-requests/working-with-forks/configuring-a-remote-repository-for-a-fork):

`git remote add upstream https://github.com/rooch-network/rooch.git`

When you're under the rooch dir, you can locally run `./scripts/dev_setup.sh` to ensure you have all development dependencies required for our workflows.

## Issues

Rooch Network uses [GitHub issues](https://github.com/rooch-network/rooch/issues) to track bugs. Please include necessary information and instructions to reproduce your issue.


## Pull Request Requirements

You now have a fork of the Rooch repo set up in your local workspace. You can make changes to the files in the workspace, add commits, then push your changes to your fork of the repo to then create a Pull Request.

Rooch Network welcomes everyone to participate and contribute, after reading the contribution guidelines, we also invite you to take a look at the requirements for Pull Requests [PR-Guidelines](./docs/pr-requirements.md). 


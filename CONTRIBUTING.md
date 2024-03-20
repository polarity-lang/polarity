# Contributing Guidelines

Pull requests, bug reports and feature requests are highly welcomed and encouraged!

## Contents

- [Pull Request Workflow](#pull-request-workflow)
- [Release Workflow](#release-workflow)

## Pull request workflow

All changes to the codebase should go through pull-requests.
We do not allow commits directly on the `main` branch of the repository.
Furthermore, all pull requests should be associated with at least one specific issue unless the PR fixes a minor problem.
Please check that you observe the following guidelines:

- If your PR changes the observable behaviour of the binary, then you have to add an entry to the `CHANGELOG.md` file with your PR under the `Unreleased` section of the changelog.
- Every PR needs to have at least 1 approval before it can be merged.
- We enforce a linear history on the `main` branch, so every PR must either be rebased or rebased and squashed before it is merged into `main`.

## Release workflow

We use the following workflow for generating a new release for version `x.x.x`:

- Open a branch with the name `prepare-release-x.x.x` and create a corresponding PR.
- Change the versions in all the `Cargo.toml` files to the new version `x.x.x`, build the project, and also commit the generated `Cargo.lock` file.
- Move everything under the section `Unreleased` in the `CHANGELOG.md` into a new section `[x.x.x] YYYY-MM-DD` with the current date.
- Merge the Pull request into `main`.
- In the main branch, use `git tag -a x.x.x -m "Version x.x.x` to create a tag, and `git push origin x.x.x` to publish the tag.

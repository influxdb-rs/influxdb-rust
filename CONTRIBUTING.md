# Contributing to influxdb-rust

Thank you for contributing. It's much apprechiated!

The following is a set of guidelines for contributing to influxdb-rust, which is hosted at [Empty2k12/influxdb-rust](https://github.com/Empty2k12/influxdb-rust) on GitHub. These are mostly guidelines, not rules. Use your best judgment, and feel free to propose changes to this document in a pull request.

#### Table Of Contents

[Code of Conduct](#code-of-conduct)

[How Can I Contribute?](#how-can-i-contribute)

-   [Reporting Bugs](#reporting-bugs)
-   [Suggesting Enhancements](#suggesting-enhancements)
-   [Your First Code Contribution](#your-first-code-contribution)
-   [Pull Requests](#pull-requests)

[Styleguides](#styleguides)

-   [Git Commit Messages](#git-commit-messages)
-   [Rust Styleguide](#rust-styleguide)

[Additional Notes](#additional-notes)

-   [Issue and Pull Request Labels](#issue-and-pull-request-labels)

## Code of Conduct

This project and everyone participating in it is governed by the [Contributor Covenant Code of Conduct](./CODE_OF_CONDUCT.md). By participating, you are expected to uphold this code. Please report unacceptable behavior to [influxdbrust@gerogerke.de](mailto:influxdbrust@gerogerke.de).

## How Can I Contribute?

### Reporting Bugs

Please search the closed issues list before opening a bug report. If you find a **Closed** issue that seems like it is the same thing that you're experiencing, open a new issue and include a link to the original issue in the body of your new one.

#### How Do I Submit A (Good) Bug Report?

Bugs are tracked as [GitHub issues](https://guides.github.com/features/issues/). Please fill out the bug report template when filing a bug.

Explain the problem and include additional details to help maintainers reproduce the problem:

-   **Use a clear and descriptive title** for the issue to identify the problem.
-   **Describe the exact steps which reproduce the problem** in as many details as possible.
-   **Provide specific examples to demonstrate the steps**. Include links to files or GitHub projects, or copy/pasteable snippets, which you use in those examples. If you're providing snippets in the issue, use [Markdown code blocks](https://help.github.com/articles/markdown-basics/#multiple-lines).
-   **Describe the behavior you observed after following the steps** and point out what exactly is the problem with that behavior.
-   **Explain which behavior you expected to see instead and why.**

Provide more context by answering these questions:

-   **Did the problem start happening recently** (e.g. after updating to a new version of influxdb-rust) or was this always a problem?
-   If the problem started happening recently, **can you reproduce the problem in an older version of influxdb-rust?** What's the most recent version in which the problem doesn't happen? You can download older versions of influxdb-rust from [the releases page](https://github.com/Empty2k12/influxdb-rust/releases).
-   **Can you reliably reproduce the issue?** If not, provide details about how often the problem happens and under which conditions it normally happens.

### Suggesting Enhancements

This section guides you through submitting an enhancement suggestion for influxdb-rust, including completely new features and minor improvements to existing functionality. Following these guidelines helps maintainers and the community understand your suggestion and find related suggestions.

When you are creating an enhancement suggestion, please [include as many details as possible](#how-do-i-submit-a-good-enhancement-suggestion). Fill in the enhancement suggestion template, including the steps that you imagine you would take if the feature you're requesting existed.

#### How Do I Submit A (Good) Enhancement Suggestion?

Enhancement suggestions are tracked as [GitHub issues](https://guides.github.com/features/issues/). Create an issue and provide the following information:

-   **Use a clear and descriptive title** for the issue to identify the suggestion.
-   **Provide a step-by-step description of the suggested enhancement** in as many details as possible.
-   **Provide specific examples to demonstrate the steps**. Include copy/pasteable snippets which you use in those examples, as [Markdown code blocks](https://help.github.com/articles/markdown-basics/#multiple-lines).
-   **Describe the current behavior** and **explain which behavior you expected to see instead** and why.
-   **Explain why this enhancement would be useful** to most influxdb-rust users.
-   **Specify the name and version of the OS you're using.**

### Your First Code Contribution

Unsure where to begin contributing to influxdb-rust? You can start by looking through these `good first issue` and `Type of Change: Minor` issues:

-   [good first issue issues](https://github.com/Empty2k12/influxdb-rust/labels/good%20first%20issue) - issues which are suited for new developers. usually just some lines of code or just a test.
-   [Type of Change: Minor issues](https://github.com/Empty2k12/influxdb-rust/labels/Type%20of%20Change%3A%20Minor) - issues which only change a small amount of code in the library.
-   [Hacktoberfest issues](https://github.com/Empty2k12/influxdb-rust/labels/Hacktoberfest) - issues which are suited for someone taking part in Hacktoberfest.

#### Local development

influxdb-rust can be developed locally.

`cargo build` can be used to check if code is compiling. To run the integration tests, first start a InfluxDB docker container which the tests will run against using `docker run -d -p127.0.0.1:8086:8086 influxdb:alpine`. Once the container has started, you can run the integration tests using `cargo test`.

### Pull Requests

The process described here has several goals:

-   Maintain influxdb-rust's quality
-   Fix problems that are important to users
-   Engage the community in working toward the best possible influxdb-rust
-   Enable a sustainable system for influxdb-rust's maintainers to review contributions

Please follow these steps to have your contribution considered by the maintainers:

1. Follow all instructions in [the template](./.github/PULL_REQUEST_TEMPLATE.md)
2. Follow the [styleguides](#styleguides)
3. After you submit your pull request, verify that all [status checks](https://help.github.com/articles/about-status-checks/) are passing <details><summary>What if the status checks are failing?</summary>If a status check is failing, and you believe that the failure is unrelated to your change, please leave a comment on the pull request explaining why you believe the failure is unrelated. A maintainer will re-run the status check for you. If we conclude that the failure was a false positive, then we will open an issue to track that problem with our status check suite.</details>

While the prerequisites above must be satisfied prior to having your pull request reviewed, the reviewer(s) may ask you to complete additional design work, tests, or other changes before your pull request can be ultimately accepted.

## Styleguides

### Git Commit Messages

-   Use the present tense ("Add feature" not "Added feature")
-   Use the imperative mood ("Move cursor to..." not "Moves cursor to...")
-   Limit the first line to 72 characters or less

### Rust Styleguide

Please format your code using `cargo fmt --all` and make sure `cargo clippy` produces no warnings.

## Additional Notes

### Issue and Pull Request Labels

This section lists the labels we use to help us track and manage issues and pull requests.

[GitHub search](https://help.github.com/articles/searching-issues/) makes it easy to use labels for finding groups of issues or pull requests you're interested in.

The labels are loosely grouped by their purpose, but it's not required that every issue have a label from every group or that an issue can't have more than one label from the same group.

Please open an issue if you have suggestions for new labels.

#### Type of Issue or Pull Request

| Issue label                    | List issues                                                                                        | Description                                                                                                                            |
| ------------------------------ | -------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------- |
| `Status: Merge when CI passes` | [search](https://github.com/empty2k12/influxdb-rust/labels/status%3a%20merge%20when%20ci%20passes) | Applied when the pull request has been reviewed and is ready for merge once the CI pipeline passes.                                    |
| `Status: Awaiting Response`    | [search](https://github.com/empty2k12/influxdb-rust/labels/status%3a%20awaiting%20response)        | Applied to issues which have a response from the issue owner pending.                                                                  |
| `Status: Pending Discussion`   | [search](https://github.com/empty2k12/influxdb-rust/labels/status%3a%20pending%20discussion)       | Applied to pull requests where a pull request review has been submitted and the pull request author has not responded to feedback yet. |
| `Status: Pending Updates`      | [search](https://github.com/Empty2k12/influxdb-rust/labels/Status%3A%20Pending%20Updates)          | Applied to pull requests where updates to the changeset is pending.                                                                    |
| `Status: Work in Progress`     | [search](https://github.com/Empty2k12/influxdb-rust/labels/Status%3A%20Work%20in%20Progress)       | Applied to pull requests that are work in progress.                                                                                    |

#### Type of Change of Issue or Pull Request

| Issue label          | List issues                                                                                  | Description                                                                            |
| -------------------- | -------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------- |
| `Type: Bug`          | [search](https://github.com/Empty2k12/influxdb-rust/labels/Status%3A%20Type%20Bug)           | Applied to issues reporting bugs.                                                      |
| `Type: Chore`        | [search](https://github.com/Empty2k12/influxdb-rust/labels/Status%3A%20Type%20Chore)         | Applied to issues and pull requests regarding miscellaneous tasks around the reposity. |
| `Type: Enhancement`  | [search](https://github.com/Empty2k12/influxdb-rust/labels/Status%3A%20Type%20Enhancement)   | Applied to issues and pull requests where an existing feature is improved.             |
| `Type: Governance`   | [search](https://github.com/Empty2k12/influxdb-rust/labels/Status%3A%20Type%20Governance)    | Applied to issues pull requests regarding repository governance.                       |
| `Type: New Feature`  | [search](https://github.com/empty2k12/influxdb-rust/labels/status%3a%20Type%20New%20Feature) | Applied to issues and pull requests requesting or implementing new features.           |  |
| `Type: Optimization` | [search](https://github.com/empty2k12/influxdb-rust/labels/status%3a%20Type%20Optimization)  | Applied to issues and pull requests regarding optimizing existing code.                |  |
| `Type: Security`     | [search](https://github.com/empty2k12/influxdb-rust/labels/status%3a%20Type%20Security)      | Applied to issues and pull requests regarding the security of the library.             |  |

#### Size of Change Labels

| Issue label              | List issues                                                                                | Description                                                   |
| ------------------------ | ------------------------------------------------------------------------------------------ | ------------------------------------------------------------- |
| `Type of Change: Master` | [search](https://github.com/Empty2k12/influxdb-rust/labels/Type%20of%20Change%3A%20Master) | Applied to issues and pull requests which are major changes.  |
| `Type of Change: Medium` | [search](https://github.com/Empty2k12/influxdb-rust/labels/Type%20of%20Change%3A%20Medium) | Applied to issues and pull requests which are medium changes. |
| `Type of Change: Minor`  | [search](https://github.com/Empty2k12/influxdb-rust/labels/Type%20of%20Change%3A%20Minor)  | Applied to issues and pull requests which are small changes.  |

#### Misc Labels

| Issue label      | List issues                                                                      | Description                                                       |
| ---------------- | -------------------------------------------------------------------------------- | ----------------------------------------------------------------- |
| good first issue | [search](https://github.com/Empty2k12/influxdb-rust/labels/good%20first%20issue) | Indicates this issue is suited for new contributors               |
| Hacktoberfest    | [search](https://github.com/Empty2k12/influxdb-rust/labels/Hacktoberfest)        | Issues which are suited for someone taking part in Hacktoberfest. |

This document has been adopted from [Atom contributing guidelines](https://github.com/atom/atom/blob/master/CONTRIBUTING.md)

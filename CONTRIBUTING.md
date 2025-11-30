# Contributing to Gage CLI

The following is a set of guidelines for contributing to Gage CLI.

## Code of Conduct

Take a moment to review our [Code of Conduct][conduct] before engaging
with the Gage CLI community. The purpose of the Code of Conduct is to
support fairness and safety for everyone.

## Source code repository

The main Gage CLI source code repository is hosted on GitHub at:

<https://github.com/gageml/gage-cli>

Contributions should be made by submitting pull requests to this
repository.

## Contributions

Contributions generally fall into one of these categories:

- Bug report
- Feature request
- Fixes
- Help with features

We use these GitHub tools to facilitate contributions:

- [Discussions][discussions]
- [Issues][issues]
- [Pull requests][prs]

If you're not sure where to start, just [ask a question][discussions].

### Report a bug

If you see something that's broken, please report it by [opening a new
issue][issues]. If you happen to see an existing issue that describes
your issue, contribute to it by adding comments up upvotes (thumbs up).

When reporting an issue, please include this information:

- Summary of the problem
- Steps to reproduce

### Request a feature

Use [project discussions][discussions] to present your idea. We'll work
to move your idea forward. If your submission lingers without feedback,
we apologize --- that's an oversight. We encourage you to "bump" a topic
if you feel it's not getting due attention.

### Fix a bug

We encourage you to submit bug fixes.

We value your time and effort as a contributor and want you code to make
into future releases. It's frustrating to spend time fixing a bug only
to have your code rejected for petty reasons. We commit to making this
as easy as possible for both you and us.

Before spending time on a bug, take a moment to review this short
checklist.

- [ ] Is there an open issue that describes the bug?
- [ ] Is the issue classified as a bug or is there otherwise agreement
      with the project maintainers that the issue needs to be fixed now?

Any documented feature in Gage CLI that doesn't work as designed is a
bug and always meets the "needs to be fixed now" standard.

Reasons for rejecting a contribution:

- The fix is not for an agreed upon problem that needs to be fixed now
- We feel you are not working in good faith to address concerns we have
  about your contribution

Use the [GitHub pull request model][github-prs] to submit fixes.

Your pull request may need revision for any of these reasons:

- Incorrect implementation (code doesn't work as it should)
- Code that we feel would require controversial changes (i.e. changes
  that you might not agree with)
- Lack of acceptable tests

We will NOT delay merging your PR for these reasons:

- Code formatting
- Code style
- Easily correct errors (generally defined as errors that can be fixed
  without controversy)
- Easily added tests

Rather than present you with a series of change requests, we may move
your PR to a project branch and make the changes ourselves. For minor
changes or changes we feel are not controversial, we may merge our
changes directly without your input. For cases that we feel require your
input, we will defer merging until you've had a chance to review our
changes.

We will explain any changes we make so you know why we made them.

### Help with a feature

If you would like to contribute to a feature with the expectation that
it be included in future releases, review this checklist:

- [ ] Are we aware that you are working on a feature that you would like
      merged at some point?
- [ ] Is there some level of agreement with us that the feature is in
      scope and desirable for the project?
- [ ] Have you established a positive working relationship with the
      project maintainers, e.g. by submitting accepted bug fixes, etc.

We recognize that you may need to solve problems without taking the time
and effort to work with us. In this case, fork the project and make your
changes. This can be a path to merge later. It allows you to resolve any
issue to your satisfaction and later present us with a working solution.

We may decline to merge downstream changes for any of these reasons:

- The feature is not in scope for the project
- We feel the cost of maintaining the feature is too high (e.g. we lack
  the expertise or skill to fix issues, lack ready access to required
  software or systems, high defect rate, etc.)

## Documentation

Documentation is currently hosted at <https://gage.io/docs/cli>. The
source code for Gage CLI is currently in a private repository. This will
change in the near future. All Gage CLI document will eventually be
located in this repository under `/docs` and will be freely modifiable
via contributions.

For the time being, if you need to change the Gage CLI documentation,
please [open an issue][issues].

## Style

These notes apply to maintainers as well as external contributors.

Code should be free from individual style or idioms.

### Rust conventions

Gage CLI is written in Rust and adheres closely to Rust conventions.

- Source code should be formatted on save using [`rustfmt`][rustfmt] or
  equivalent (e.g. Rust Analyzer for VS Code)

### Git commit messages

Our commit rules are simple but strict.

- First line is the message title
  - Must begin with a capital letter
  - Must not exceed 50 chars
- Additional content
  - Must be separated from the title with a single blank line
  - No line may exceed 72 chars
- No line may contain trailing whitespace (tabs or spaces)

These rules are enforced by a local Git commit hook.

There are otherwise no rules governing commit message content. We are
not concerned with tags, labels, or other patterns that might be used by
automated tools.

## Tests

Tests are maintained under [`/tests`][tests] and within Rust modules.

Tests under `/tests` are run using [Groktest][groktest].

Use the `test` recipe in [`justfile`][justfile] to run tests.

```shell
just test
```

If you provide a fix, please include tests that show the fix works as
you intend.

If you feel the fix doesn't warrant tests, mention that in your pull
request. We may ask you to add tests, add tests ourselves, or agree that
the fix does not warrant tests.

## Changes to this document

Please use [discussions] to submit ideas for improving this document.

<!-- Links -->

[conduct]: ./CONDUCT.md
[discussions]: https://github.com/gageml/gage-cli/discussions
[github-prs]:
  https://docs.github.com/en/pull-requests/collaborating-with-pull-requests/proposing-changes-to-your-work-with-pull-requests/creating-a-pull-request
[issues]: https://github.com/gageml/gage-cli/issues
[justfile]: ./justfile
[groktest]: https://github.com/gar1t/groktest
[prs]: https://github.com/gageml/gage-cli/pulls
[rustfmt]: https://github.com/rust-lang/rustfmt
[tests]: ./tests

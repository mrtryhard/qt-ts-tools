# Contributing

Thank you for your interest in qt-ts-tools. Before contributing, please read these guidelines.

### What is a good contribution for this project ?

Below is a an example list of what is a contribution we're looking for, but is not limited to:

* Bug reporting
* Bug fix patch / pull request
* Feature enhancement
* Security vulnerability reporting
* Other improvements

### What contributions is less appropriate for this project ?

This project aims to manipulate Qt translation files, and aims to do it with the simplest usage possible. 
Here's a list of less appropriate contributions:

* Proposal for different file format i.e. `.po` files since it's out of scope for this project.
* New feature patch/pull request for **non-planified** feature (opening a discussion or issue is fine, once validated a patch is welcome).
  * In other words, be sure your idea / feature is approved before starting any work.  

# Ground Rules
### Code of Conduct

In general following [Rust Code of Conduct](https://www.rust-lang.org/policies/code-of-conduct) is a pretty good starting point.

### Project philosophy

`qt-ts-tools` aim to a predictable and user-friendly tool. This means any contributions to the project should follow these rules of thumb:
* An input file is never modified implicitly; in other words the user should specify via a specific output path command flag.
* The command line interface should remain consistent across functionalities (e.g. `output-path` should be standard across commands).
* It must target solutions for Qt translation files.

# Your First Contribution
You want to help ? Not sure where to start ? Look at the `good first issue` label in the issue tracker:
https://github.com/mrtryhard/qt-ts-tools/issues?q=is%3Aopen+is%3Aissue+label%3A%22good+first+issue%22

Feel free to get in touch if you are not sure how you could contribute.

### Some general documentation if you're a bit lost
Here are a couple of friendly tutorials you can include: http://makeapullrequest.com/ and http://www.firsttimersonly.com/

# How to report a bug
Please open an issue if you encounter any bug or security vulnerability.  
In your issue, please mention the following:

* Operating system
* qt-ts-tools version
* Short description on what failed (command, flags, etc)
* Please provide a sample translation file that triggers the issue, if possible. 

Any other relevant information is welcome. 

# How to suggest a feature or enhancement

Please look at the mini roadmap in the [README.md](README.md), or at the `enhancement` tickets to know what's already taken into account.

### Suggesting a new feature

Please identify what problem this new feature would solve. Make sure it aligns with the project philosophy. 
If you have a detailed idea of the user interactions, or any relevant details, please explain them as well. 
The cleared the request, the easier to make it go through.

# Code review process

For patch submission please go through the pull request mechanism on Github. Makes sure the contribution contains:

* License expectation if different from `Apache 2.0`. This will need a discussion.
* A pull request may not be merged unless the author has approved of it. 
* Contributions must adhere and respect the [Developer Certificate](https://developercertificate.org/).

# Community

No community at the moment, feel free to open a discussion in the repository or an issue, or even email the author directly.

# Commit convention
Nothing enforced for now but please do refer to the ticket number via Github's keywords, and add a description to your commit. 
For example:

* `fixes #123: Merging now take into account "extra-*" fields.`
* `Refactored merging engine, see #45, see #67`

# Any questions ?
Feel free to reach me on my email or by discussion.
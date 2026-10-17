# Security policy

Wren is an early preview. Security fixes target `main` and the latest release,
once releases are available. Older builds are not maintained separately.

## Report a vulnerability

Use [GitHub's private vulnerability reporting form](https://github.com/MaxTheTurtle0/wren/security/advisories/new).
Keep suspected vulnerabilities out of public issues and pull requests.

Include the affected Wren version or commit, operating system, steps to
reproduce, and the impact. Use a test account or synthetic data. Do not include
access or refresh tokens, authorization codes, private keys, real messages,
or screenshots containing account data.

For a normal bug without a security impact, open a GitHub issue. Wren's
[privacy documentation](docs/privacy.md) explains its network access, local
cache, credential storage, and current limits.

## Maintainer setup

GitHub private vulnerability reporting is available for public repositories.
Enable it in the repository's security settings when making Wren public, and
verify that the reporting form works before announcing the project. Until
then, the private repository is accessible only to its collaborators.

# Maintainers

| Role | Who | Responsibilities |
|---|---|---|
| Maintainer | the xylarium-kernel maintainer session, an AI coding agent working for the operator | triage of issues, review and merge of pull requests into `main`, releases, documentation, the cross-project standard |
| Final approver | the project's operator | changes to the model's laws, the network contract, releases and the project's standard |

Contributions arrive as pull requests reviewed by the maintainer
([CONTRIBUTING.md, Development workflow](CONTRIBUTING.md#development-workflow)).
Questions and proposals go in
[issues](https://github.com/SAIOS-SAMN/xylarium/issues); security reports follow
[SECURITY.md](SECURITY.md).

Xylarium is developed alongside two projects, each maintained in a separate
repository: [mycelium](https://github.com/SAIOS-SAMN/mycelium), the network
Xylarium runs on, and [netstratum](https://github.com/SAIOS-SAMN/netstratum). A
change that affects them is opened in each affected repository with the
`cross-project` label; no maintainer edits another project's repository.

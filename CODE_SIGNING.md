# Code signing policy

Free code signing provided by [SignPath.io](https://signpath.io), certificate
by [SignPath Foundation](https://signpath.org).

Windows installers and programs of Prototype are signed only when they are
built by this repository's public GitHub Actions workflow
(`.github/workflows/release.yml`) from the source code in this repository.
Nothing built anywhere else is signed.

## Team roles

| Role | Member |
|---|---|
| Committer and reviewer | [Ryan Johnson (deadlyxreturn)](https://github.com/deadlyxreturn) |
| Approver | [Ryan Johnson (deadlyxreturn)](https://github.com/deadlyxreturn) |

Every signing request is approved by hand by the approver. Changes from
people outside the team are reviewed before they are merged.

## Privacy

This program will not transfer any information to other networked systems
unless specifically requested by the user or the person installing or
operating it, apart from what any Second Life viewer must send to log in.
See the [privacy policy](PRIVACY.md).

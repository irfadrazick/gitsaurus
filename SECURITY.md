# Security policy

## Reporting a vulnerability

Please report security problems privately through GitHub: open the [Security tab](https://github.com/irfadrazick/gitsaurus/security) and click **Report a vulnerability**. Don't open a public issue for them.

Include what you found, the steps to reproduce it and the version you tested (Settings shows it). You'll get a reply within a week. Once a fix is released, the advisory is published with credit to you unless you'd rather stay anonymous.

## Supported versions

Only the latest release gets security fixes. Gitsaurus updates itself, so installed copies pick up fixes once they are released.

## Scope

Gitsaurus runs the `git` CLI on repositories you open, with your own credentials and config. In scope:

- Anything that lets a repository, remote or file content run code or commands you didn't ask for
- The update mechanism: anything that could make an installed copy accept an update that wasn't signed with the project's key
- Leaking credentials, file contents or repository data off your machine

Problems in git itself, in macOS or in a repository's own hooks (which git runs by design) are out of scope; please report those upstream.

# Security Policy

## Threat Model

`hunch` is a **filename parser**. It reads filename strings (and optional
parent-directory path context) and produces structured metadata. It does
**not**:

- Open, read, or write the contents of any media file
- Make network requests
- Execute external programs
- Persist any state to disk (the library is pure; the CLI only writes
  structured output to stdout)

The CLI does perform **directory traversal** (`hunch --batch -r`),
which is explicitly hardened: depth-bounded (`MAX_WALK_DEPTH = 32`)
and symlink-skipping. See the rustdoc on
[`walk_dir`](src/main.rs) for the full threat-model rationale.

## Supported Versions

Security fixes are applied to the **latest minor release** on the
`1.x` line. Older minor releases are not patched — please upgrade.

| Version | Supported          |
| ------- | ------------------ |
| 1.1.x   | :white_check_mark: |
| < 1.1   | :x:                |

## Reporting a Vulnerability

**Please do not report security vulnerabilities through public GitHub
issues.**

Instead, use one of these private channels:

1. **GitHub private vulnerability reporting** (preferred):
   <https://github.com/lijunzh/hunch/security/advisories/new>
2. **Email**: open an issue tagged `security` requesting a private
   contact, and a maintainer will reach out.

Please include:

- A description of the vulnerability and its potential impact
- Steps to reproduce (a minimal filename / directory layout is ideal)
- The version of `hunch` affected (`hunch --version`)
- Your assessment of severity, if you have one

## Response Timeline

As an open-source project maintained by volunteers:

- **Initial acknowledgment**: within 7 days
- **Triage / severity assessment**: within 14 days
- **Fix or mitigation plan**: communicated within 30 days for
  high-severity issues; longer for low-severity / hardening items

We will credit reporters in the changelog unless they prefer to remain
anonymous.

## Scope

In-scope vulnerabilities include (but are not limited to):

- **Denial of service** via crafted filenames or directory layouts
  (panics, stack overflows, unbounded resource consumption, regex
  catastrophic backtracking)
- **Path traversal / sandbox escape** in the CLI's `--batch -r` mode
- **Vulnerabilities in dependencies** that are exploitable through
  `hunch`'s public API

Out-of-scope:

- Vulnerabilities requiring the attacker to already have write access
  to the parsed filenames AND to a directory the user explicitly chose
  to scan (this is a trust boundary, not a vulnerability)
- Issues in `dev-dependencies` not reachable from the published crate
- Style / hardening preferences without a concrete exploit scenario
  (please file these as regular issues)

## Security Hardening (non-CVE)

For non-CVE security hardening (e.g., adding a defense-in-depth check,
upgrading a yanked dev-dep), please open a regular GitHub issue. These
do not need the private reporting channel.

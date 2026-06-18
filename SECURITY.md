# Security Policy

Eterea is a local-first desktop application. Security reports should avoid
sharing private archives, database files, tokens, cookies, or other sensitive
user data.

## Supported versions

| Version | Supported |
| --- | --- |
| Unreleased / `main` | Security fixes accepted before release |
| `0.1.0` production-candidate | Supported for release-candidate fixes |
| Older snapshots | Not supported |

## Reporting a vulnerability

Please report suspected vulnerabilities through GitHub Security Advisories for
this repository if advisories are enabled. If that channel is unavailable, open a
minimal public issue that says a private security contact is needed, but do not
include exploit details, personal data, or secrets in the public issue.

No project-specific security email or secret reporting endpoint is currently
published, so this policy intentionally does not invent one.

## Local data and privacy cautions

- Imported bookmark archives and SQLite databases may contain private reading
  history, handles, URLs, notes, and timestamps.
- The default database lives under the platform app-data path described in
  [release readiness](docs/operations/release-readiness.md).
- Remote media previews stay hidden by default; enabling them may request stored
  HTTPS media URLs during that session.
- Bug reports and evidence should redact local paths, user names, handles,
  archive contents, cookies, API keys, and screenshots containing private data.

## Maintainer response expectations

A maintainer should acknowledge a valid private report, reproduce with sanitized
inputs where possible, fix on the smallest supported surface, and document the
verification evidence without exposing reporter or user data.

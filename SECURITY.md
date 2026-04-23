# Security Policy

## Supported Versions

Only the latest release is supported with security fixes.

| Version | Supported |
| ------- | --------- |
| 0.1.x   | ✅        |
| < 0.1   | ❌        |

## Reporting a Vulnerability

Please report security vulnerabilities privately via [GitHub Private Vulnerability Reporting](https://github.com/adrien-jeser-doctolib/rust-rapport/security/advisories/new).

You can expect an acknowledgement within 7 days. Coordinated disclosure is appreciated — please do not open a public issue or PR for a vulnerability before it has been acknowledged and a fix is available.

This project ships a small CLI with no network access and no credential handling, so the realistic threat surface is narrow (untrusted clippy JSON as input). Reports along those lines are welcome.

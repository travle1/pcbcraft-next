# Security policy

PCBcraft is pre-release software. There is no supported stable release series or
security backport schedule. Fixes target the current development code and are
reviewed for inclusion on `main`; an unreleased fix may not be in your checkout.
Include the exact commit/build in reports. There is no promised response deadline
or paid security-support service.

## Reporting

The maintainer contact is [@travle1](https://github.com/travle1). Do not put exploit
details, secrets, private designs or sensitive attachments in a public issue/PR.

If the repository's [Security page](https://github.com/travle1/pcbcraft-next/security)
offers **Report a vulnerability**, use that private GitHub route. Its availability
has not been verified; this document does not enable it. Otherwise use an existing
private contact with the maintainer, or open a neutral
[contact-request issue](https://github.com/travle1/pcbcraft-next/issues/new) asking
`@travle1` for a private security-reporting route. Give no vulnerability details in
the public request and wait for a private route before sending the report. No
separate reporting email has been published.

Privately include:

- Affected commit, platform and relevant build configuration.
- Minimal reproduction and a small original/sanitized project or input when useful.
- Expected versus observed behavior, attacker prerequisites and practical impact.
- Whether you reproduced it or inferred it from source/tool output; report uncertainty.
- Any suggested fix and your preference about credit/disclosure.

The maintainer will assess the report, coordinate follow-up and agree on an
appropriate disclosure path. Do not publish sensitive details before coordination.
This file makes no claim that an unimplemented control or planned feature is secure.

## Scope and ordinary bugs

Potential security concerns include unintended code execution or file access when
opening a project, escaping intended IO boundaries, exposure of private data, or
bypassing an implemented authorization control. Resource-exhaustion reports should
explain the input and practical impact; a crash is not automatically dismissed.

Ordinary editing, rendering and compatibility problems belong in the
[issue tracker](https://github.com/travle1/pcbcraft-next/issues), with sanitized
reproduction steps. Accidental data loss matters even when it is not a vulnerability.
If unsure, request a private route first. See [Support](SUPPORT.md) and
[current implementation limits](docs/implementation-status.md).

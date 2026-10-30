# Security Policy

## Supported versions

Only the latest commit on `main` is supported. There are no tagged releases yet.

## Reporting a vulnerability

Do not open a public issue. Use GitHub's private reporting:
[Report a vulnerability](https://github.com/alwaysYSL/Yasal/security/advisories/new).

Include the affected version or commit, steps to reproduce, and the impact you observed. Expect an initial reply within 7 days.

## Scope

Yasal runs as the current user and executes user-typed commands by design. Reports are in scope when they show:

- Command or argument injection through input that the user did not intend to execute (for example, crafted file names, window titles, or clipboard content reaching the `>` shell plugin or an app launch).
- Unsafe handling of `config.json`, `frecency.json`, or the icon cache that leads to code execution or privilege escalation.
- Clipboard history persisted to disk or sent over the network.
- Network requests other than the documented `https://api.ipify.org` lookup and user-initiated web searches.

Running a command that the user typed after `>` is intended behavior and not a vulnerability.

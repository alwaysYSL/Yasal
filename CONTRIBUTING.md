# Contributing

Bug reports, fixes, and plugins are welcome.

## Setup

Windows 10 1809+ or Windows 11, Rust, the `x86_64-pc-windows-gnu` target, and MinGW-w64.

```powershell
rustup target add x86_64-pc-windows-gnu
cargo test
cargo run
```

## Before opening a PR

- `cargo test` passes.
- `cargo fmt --check` passes.
- Behavior changes come with a test in `tests/` or an inline unit test, and a README update (`README.md` and `README.id.md`) when user-visible.
- Keep PRs small and focused. One change per PR.

## Layout

- `src/core/`: dispatcher, plugin trait, matcher, frecency.
- `src/plugins/`: one file per plugin. See `docs/DESIGN.md` before adding one.
- `src/platform/`: Win32, hotkey, tray.
- `src/ui/`: egui views.
- `vendor/eframe/`: patched `eframe` 0.29.1. Avoid touching it; if a change is needed, note it in the PR.

## Security

Do not open a public issue for a vulnerability. Report it privately through GitHub Security Advisories on this repository.

## License

By submitting a contribution you agree it is dual licensed under MIT OR Apache-2.0, matching the project, with no additional terms.

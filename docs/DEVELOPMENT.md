# DEVELOPMENT GUIDE

This document explains the development strategy, TDD approach, applied patterns,
antipatterns avoided, and how to contribute to the `autoversion` project.

License: MIT — this repository is licensed under the MIT License (see `LICENSE`).

## Goals
- Maintain the project as a Rust command-line binary with automated tests.
- Follow Test-Driven Development (TDD): write tests that define expected behavior before implementation.
- Make the project easy to contribute to and maintain (clear comments, doc-comments, and tests).

---

## Updater structure (Phase 1 & 2)
Each updater implements the `VersionUpdater` trait located at `src/updaters/traits.rs`.
Minimum contract:
- `get_current_version(project_path) -> Result<String>`
- `update_version(project_path, new_version) -> Result<Vec<String>>` (files updated)
- `validate_project(project_path) -> Result<()>`
- `technology_name() -> &'static str`
- `get_primary_file(project_path) -> Result<PathBuf>`
- `can_handle(project_path) -> bool`
- `preview_changes(project_path, new_version) -> Result<Vec<VersionChange>>`

### Implemented updaters
- `npm` (Phase 1): JSON handling for `package.json` and `package-lock.json`.
- `generic` (Phase 1): `VERSION`, `version.txt`, and similar simple files.
- `cargo` (Phase 2): TOML parsing for `Cargo.toml` using the `toml` crate.
- `maven` (Phase 2): regex-based replacement for the `<version>` tag in `pom.xml`.
- `python` (Phase 2): TOML parsing for `pyproject.toml` and regex-based updates for `setup.py`.

## Tests and TDD
- Each updater includes unit tests in the same module (`#[cfg(test)] mod tests`).
- Tests create temporary projects using `tempfile::TempDir` and write sample files.
- Tests validate:
  - Version detection (`get_current_version`).
  - Version update (`update_version`) — confirms files were modified.
  - Preview (`preview_changes`) — ensures `old_version` and `new_version` are captured.
  - `can_handle` and `validate_project`.

### Why TDD matters here
- Updaters must handle heterogeneous formats (JSON, TOML, XML, Python). Tests act as a contract
  and protect against regressions when multiple parsers and strategies are involved.
- TDD facilitates refactors: implementations can change (for example, switch from regex to an XML parser)
  while tests maintain the intended behavior.

## Applied patterns
- Strategy pattern: each updater implements the `VersionUpdater` trait. `UpdaterFactory` selects
  the appropriate strategy based on the detected technology.
- Fail-fast: functions return `Result` and fail with explicit errors on precondition violations.
- Backups before writing: all updates create backups using `utils::files::backup_file`.
- Isolated tests: `tempfile::TempDir` is used to avoid side effects on the local repository.

## Antipatterns avoided
- Never modify files without first making a backup.
- Avoid ad-hoc parsing when a reliable library exists (we use `toml` for TOML and `serde_json` for JSON).
- Do not rely on side-effects (stdout) to detect versions.

## Known limitations
- The `maven` updater uses a regex to replace the first `<version>` tag inside `<project>`. This is
  sufficient for many projects but can fail on complex POMs (namespaces, inherited versions via `parent`).
- The `setup.py` updater uses regex-based replacement and can fail if the version is computed dynamically
  in Python rather than being a static string.

## Contribution recommendations
1. Add tests first (TDD).
2. Run `cargo test -- --test-threads=1` to avoid environment variable interference in tests.
3. Keep the `VersionUpdater` contract and add tests that cover edge cases you are addressing.
4. Document architectural decisions in `docs/DEVELOPMENT.md`.

## Error handling policy

This project standardizes on `anyhow::Error` / `anyhow::Result` as the canonical error type. The reasons:

- Simplicity: `anyhow` integrates well with the existing codebase and reduces boilerplate for error propagation.
- Interoperability: many crates used (git2, toml, serde_json) already map cleanly into `anyhow`.
- Tests and implementations should return `anyhow::Result<T>` for public functions where error context is useful.

Guidelines:

- Prefer `anyhow::Result<T>` for application-level functions in CLI and updaters.
- Add context with `anyhow::Context` where additional diagnostic information helps.
- Avoid introducing a new, project-wide typed error enum unless you need structured handling at API boundaries.

An automated test verifies that source files do not reference the old typed error helpers.

---

## How to run locally
```bash
# Build
cargo build

# Run tests (single-threaded recommended)
cargo test -- --test-threads=1
```

---

If you like, I can turn this into a README under `docs/` or add example entries for each updater.

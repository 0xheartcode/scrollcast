# Changelog

All notable changes to this project are documented here. The format is based on
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project adheres
to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.1] - 2026-11-16

### Fixed

- **PDF output now contains the actual source.** The previous renderer was a
  hand-rolled PDF writer that truncated every code block to 10 lines, double-
  escaped string metacharacters (producing structurally invalid PDFs), wrote
  fabricated xref offsets, and left most pages blank. It has been rewritten on
  top of `printpdf` with real pagination.
- **Syntax highlighting now appears in PDFs**, driven by `syntect` per token.
- The output format is now **inferred from the output file extension** when
  `--format` is omitted, so `-o report.html` produces HTML instead of silently
  writing a PDF.
- Running non-interactively (pipes, CI, scripts) **no longer fails with
  `not a terminal`**; the confirmation prompt is skipped when stdin is not a TTY.
- The document title now resolves to the real directory name when the input is
  `.` or a trailing-slash path, instead of the generic "Repository".
- List bullets and non-ASCII characters no longer render as mojibake in PDFs.
- The intermediate markdown file now uses a unique per-process path (and is
  cleaned up afterwards), so concurrent runs over the same repository no longer
  race on a shared temp file or leak files into the system temp directory.

### Changed

- **Pure Rust: removed the `git2` and `reqwest` dependencies**, which were unused
  but pulled in `openssl-sys`, `libgit2-sys`, and `libssh2-sys`. Scrollcast now
  builds with no system C/OpenSSL toolchain and links no native libraries.
- Trimmed `tokio` from the `full` feature set to only what is used.
- CI now runs on the `main` branch (previously only an absent `release` branch),
  enforces `cargo fmt`/`clippy -D warnings`, and no longer installs pandoc/TeX.
- Added Spanish documentation (`README.es-ES.md`), contributed by @webbrain-one.

### Added

- Integration test suite exercising the CLI end to end.

## [0.1.0] - 2025-10-29

- Initial crates.io release.

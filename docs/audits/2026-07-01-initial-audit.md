# Audit — 2026-07-01 (initial implementation)

Two independent auditors reviewed the first implementation of `claude-answers`
and its deployment. This record captures their findings, what was resolved, and
what was deliberately deferred, so the open items can be revisited later.

## Scope

- Rust crate audit: the `claude-answers` crate at commit `a7c4146d` (correctness,
  Rust discipline, NOTA schema, edge cases, tests).
- Nix / deploy audit: `flake.nix`, the CriomOS-home wiring (flake input +
  `modules/home/profiles/med/cli-tools.nix`), and the `ouranos` home deploy.

Commits referenced:

- `a7c4146d` — original crate as audited.
- `11187f6a` — the fix for the blocker + witness test + doc corrections.
- CriomOS-home `12b0746` — initial wiring; `5af23213` — pin bumped to `11187f6a`.

## Rust audit

Checks (re-run by the auditor, fresh compile): `cargo fmt --check`, `cargo
clippy --all-targets -- -D warnings`, and `cargo test` (14 tests) all green.
Discipline (methods on data-bearing types, typed `thiserror` boundary, tests
through the public API, naming) came back clean, and the recursive `Grep` /
`filters()` design was judged sound.

### Findings

- **BLOCKER — `ProjectDirectory::locate` mis-encoded the working directory.**
  The code replaced only `/`, but Claude Code replaces *every* non-alphanumeric
  character with `-` (case preserved). Any working directory containing a `.` or
  `_` — including every `ghq` `github.com/...` path and the tool's own repo path
  — silently resolved to the wrong project directory and returned "no
  transcripts". The happy-path smoke tests missed it because `/home/li/primary`
  has no special characters.
  Status: **FIXED** in `11187f6a` (`src/transcript.rs`). Encoding now maps every
  non-ASCII-alphanumeric character to `-`.

- **MAJOR — the `locate` test could not catch the bug.** The original test used
  `/home/li/primary`, which the slash-only code encodes correctly, so it passed
  against the broken implementation (a witness that never fails).
  Status: **FIXED** in `11187f6a` — the test now asserts
  `/git/github.com/LiGoldragon/claude-answers` →
  `-git-github-com-LiGoldragon-claude-answers`, which fails on the old code.

- **MINOR — multiSelect answer values are unverified.** `Answer::new` defensively
  joins a `Value::Array` (multiSelect) with commas, but no array-valued answer
  was found in the live corpus and no fixture/test exercises the branch.
  Single-select (string) values are verified. The branch is harmless if arrays
  occur, but the on-disk multiSelect format is an unconfirmed assumption.
  Status: **DEFERRED**. Revisit by capturing a real multiSelect answer, then
  either add a fixture + test or drop the branch if the real shape differs.

- **MINOR — a missing `(File <path>)` yields a raw I/O error.** A nonexistent
  explicit file surfaces as `Error::Io("i/o: No such file or directory")` from
  `fs::read_to_string`, rather than a dedicated domain variant.
  Status: **DEFERRED**. Low urgency; consider a `FileNotFound(PathBuf)` variant.

- **NIT — `Answer::render` returns `std::io::Result` not the crate `Result`.**
  Composes fine via `#[from]`, but is inconsistent with the typed-error boundary.
  Status: **DEFERRED**.

- **NIT — `Latest` tie-break on equal mtime is arbitrary.** `sort_by_key` on
  mtime alone leaves same-mtime files in `read_dir` order.
  Status: **DEFERRED**. Edge case only.

- **Confirmed correct, no action:** `annotations` entries carrying shapes beyond
  `{notes}` (e.g. `preview`) are intentionally ignored — `preview` is the
  option's own detail, not a user-typed note.

## Nix / deploy audit

Verdict: **PASS**, no high- or medium-severity defects. The auditor re-ran
`nix build .#packages…default` and `nix build .#checks…default` (all 14 tests,
including the fixture readers) and queried the live node.

- The `cleanSource { extraFilters = [ … ".jsonl" … ]; }` change is the correct,
  idiomatic use of rust-build's helper: it keeps the `.jsonl` fixtures in the
  build sandbox without pulling in `target`/`.git`/`.jj` (pruned first).
- CriomOS-home wiring matches the established `substack-cli` pattern; the lock
  pins `claude-answers` with `nixpkgs` following the top level (no duplication);
  `med` is the correct, active tier on `ouranos`.
- Deploy is genuinely live and reproducibly pinned: `lojix-run`'s exact-reference
  rewrite pins the pushed `CriomOS-home` rev, and the node query shows the new
  home generation current with `claude-answers` on PATH.

### Notes (informational, no defect)

- **Inert toolchain hash / channel mismatch.** `flake.nix`'s `sha256` is not
  consumed — rust-build's `fromToolchainFile` returns a cluster-wide fenix
  nightly regardless, so `rust-toolchain.toml`'s `channel = "stable"` differs
  from the nightly the Nix build actually uses. This is rust-build's design, not
  a defect; left as-is to match the repo-template convention. Status: **NOTED**.
- **No `cargoClippy`/`cargoFmt` flake check.** Only `cargoTest` is exposed as a
  flake check; clippy/fmt run in CI/dev but not via `nix flake check`. Optional
  enhancement. Status: **DEFERRED**.
- **Deploy-safety.** home-manager runs standalone on `ouranos` (not via the
  NixOS module), so the running system generation does not pin this home; a
  system rebuild will not clobber the tool, and the generation is GC-rooted and
  survives reboot. Rollback to prior generations remains available.

## Verification evidence

- `cargo test` (14), `cargo clippy -D warnings`, `cargo fmt --check`,
  `nix flake check` — all green after the fix.
- The installed binary reproduces a real prior answer verbatim from the live
  transcript corpus.
- Post-fix end-to-end proof: the deployed binary, run from a dotted working
  directory (`/tmp/ca.verify/work`), correctly resolved the encoded project
  directory `-tmp-ca-verify-work` and returned the answer — the exact case the
  pre-fix binary failed.

## Open items to revisit

The MINOR/NIT findings above (multiSelect verification, typed `File`-not-found
error, `render` return type, `Latest` mtime tie-break) and the optional
clippy/fmt flake check are all deferred. None affects correctness of the
verified paths; each would cost a full rebuild + redeploy cycle, so they were
batched for a later pass rather than shipped piecemeal.

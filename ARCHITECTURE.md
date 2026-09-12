# ARCHITECTURE — claude-answers

A one-shot, read-only CLI that prints your answers to Claude Code's
AskUserQuestion prompts as canonical Datom. Claude Code records each answered
question in the session transcript under
`~/.claude/projects/<encoded-cwd>/<session>.jsonl`, but the compact
`-> (notes only)` line it prints in the terminal hides the option you picked
and the notes you typed. This tool reads those transcripts back and prints
question, option, and notes as a Datom vector of Answer structs.

## Role

The single command-line argument is one typed Datom value actualized as a
`Query`. Everything else — locating transcripts, reading them, filtering,
rendering as Datom — hangs off that value. No daemon, no state, no writes.

## Boundaries

Owns:

- Actualizing a Datom argument as a `Query`, and treating `Grep` as a
  transparent filter over any selection (`src/query.rs`).
- Locating a project's transcript directory from the working directory
  (`ProjectDirectory`, `src/transcript.rs`).
- Reading `*.jsonl` transcripts and extracting `(question, option, notes)`
  answers (`Transcript`, `Answer`).
- Collecting answers into `Vec<Answer>` and textualizing as canonical Datom.

Does not own:

- The transcript format. Claude Code owns it; this tool reads a tolerant
  subset (`toolUseResult.answers` and `toolUseResult.annotations`) and skips
  any line it does not recognise.
- Writing or mutating transcripts. It is strictly read-only.
- Protos text delineation and printing. Protos owns text; Datom owns the
  typed mapping from Datom to Query/Answer.
- The generated type declarations and Datom derives for Query and Answer.
  ethos-zero owns them; the committed `src/generated.rs` is regenerated from
  `claude-answers.ethos` through the ethos-zero library and verified fresh by
  `tests/regeneration.rs`.

## Argument grammar

```text
Latest                                      newest transcript in this project (default)
All                                         every transcript in this project
Session.47318657                            transcripts whose file name holds the id
File./path/to.jsonl                         one explicit transcript file
Grep.{ All Bluetooth }                      any selection, filtered by text
Grep.{ Session.47318657 «two words» }       guillemet-delimited multi-word text
```

`Grep` wraps another query: it narrows which answers print without changing
which transcripts are read, and it composes, so a nested `Grep` applies every
filter. With no argument the tool behaves as `Latest`.

## Output

Canonical Datom: a vector of Answer structs `[ { question option notes } ... ]`.
Faults print as canonical Datom on stderr; the exit code is nonzero.

## Code map

```
src/
  main.rs        — CLI entry: actualize one Datom value (or default Latest),
                   run, textualize output as Datom, faults as Datom on stderr
  lib.rs         — crate root, re-exports
  generated.rs   — ethos-zero generated: Query enum, Grep data, Answer struct,
                   and Datom derives
  query.rs       — parse(), run(), filter, selection
  transcript.rs  — ProjectDirectory, Transcript, Answer (raw): locate, read
  error.rs       — typed Error + Result
tests/
  regeneration.rs — freshness: committed generated.rs matches ethos-zero output
  query.rs        — parse, round-trip, execution against fixture transcripts
  transcript.rs   — locate, read, match against fixture transcripts
```

## Status

**M3.** Port to Protos 0.29.0, Datom 0.25.4, and Ethos Zero 6.1.2. The
generated types bear `Datomizable` and `Composing`; input actualizes from
`Potential`, and output follows the open Datom -> Protos -> text chain.

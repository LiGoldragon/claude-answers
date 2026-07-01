# ARCHITECTURE — claude-answers

A one-shot, read-only CLI that prints your answers to Claude Code's
AskUserQuestion prompts. Claude Code records each answered question in the
session transcript under `~/.claude/projects/<encoded-cwd>/<session>.jsonl`,
but the compact `→ (notes only)` line it prints in the terminal hides the
option you picked and the notes you typed. This tool reads those transcripts
back and prints question, option, and notes so you can copy an earlier answer.

## Role

The single command-line argument is one NOTA record, decoded via the `nota`
crate into a `Query`. Everything else — locating transcripts, reading them,
filtering, rendering — hangs off that decoded value. No daemon, no state, no
writes.

## Boundaries

Owns:

- Decoding the NOTA argument into a `Query`, and treating `Grep` as a
  transparent filter over any selection (`src/query.rs`).
- Locating a project's transcript directory from the working directory
  (`ProjectDirectory`, `src/transcript.rs`).
- Reading `*.jsonl` transcripts and extracting `(question, option, notes)`
  answers (`Transcript`, `Answer`).
- Rendering answers, with an optional case-insensitive text filter.

Does not own:

- The transcript format. Claude Code owns it; this tool reads a tolerant
  subset (`toolUseResult.answers` and `toolUseResult.annotations`) and skips
  any line it does not recognise.
- Writing or mutating transcripts. It is strictly read-only.
- The NOTA grammar and codec. The `nota` crate owns those.

## Argument grammar

```text
Latest                                   newest transcript in this project (default)
All                                      every transcript in this project
(Session 47318657)                       transcripts whose file name holds the id
(File /path/to.jsonl)                    one explicit transcript file
(Grep (All Bluetooth))                   any selection, filtered by text
(Grep ((Session 47318657) [two words]))  bracket-quote multi-word filter text
```

`Grep` wraps another query: it narrows which answers print without changing
which transcripts are read, and it composes, so a nested `Grep` applies every
filter. With no argument the tool behaves as `Latest`.

## Code map

```
src/
├── main.rs        — CLI entry: read one NOTA argument (or default Latest), run, print
├── query.rs       — Query: the NOTA argument surface; selection + filter + run
├── transcript.rs  — ProjectDirectory, Transcript, Answer: locate, read, render
└── error.rs       — typed Error + Result
```

## Status

**M0.** Feature parity with the original throwaway Python extractor: session
selection (latest / all / id fragment / explicit file), a case-insensitive
filter, and question + option + notes output — re-expressed with a NOTA
argument decoded by the `nota` crate.

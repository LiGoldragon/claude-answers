# claude-answers

Recall your answers to Claude Code's questions.

When Claude Code asks you an interactive question, the terminal keeps only a
compact line like `-> (notes only)` — the option you picked and any notes you
typed are hidden. Both are saved in the session transcript. `claude-answers`
reads those transcripts back and prints each question, the option you chose,
and your full notes as canonical Datom, so you can copy an earlier answer
without re-opening the session (and without spending its context).

## Usage

The single argument is one typed Datom query:

```
claude-answers                          # newest transcript in this project
claude-answers All                      # every transcript in this project
claude-answers Session.47318657         # transcripts whose name holds the id
claude-answers File./path/to.jsonl      # one explicit transcript file
claude-answers 'Grep.{ All Bluetooth }' # any selection, filtered by text
```

Multi-word filter text is curly-quote delimited:

```
claude-answers 'Grep.{ Session.47318657 “Bluetooth adapter” }'
```

With no argument it behaves as `Latest`. Filters are case-insensitive and
match the question, the chosen option, or the notes.

## Output

The output is canonical Datom: a vector of `{ question option notes }` answer
structs with spaced delimiters and curly-quoted strings:

```
[ { "What should the repo be named?" claude-answers "" } { "Bluetooth mic drops during active recording. What should happen?" "(notes only)" "active recording journal\ncaptured chunk immediately" } ]
```

Faults print as canonical Datom on stderr. An empty result is `[]`.

## What it reads

Transcripts live under `~/.claude/projects/<encoded-cwd>/<session>.jsonl`,
where `<encoded-cwd>` is the working directory with every non-alphanumeric
character replaced by `-` (case preserved). The tool is strictly read-only.

## Build

```
nix build          # or: cargo build --release
nix flake check    # runs the test suite
```

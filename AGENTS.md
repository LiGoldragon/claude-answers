# Agent instructions — claude-answers

## Repo role

A tiny read-only CLI that recalls your answers to Claude Code's
AskUserQuestion prompts from the on-disk session transcripts. Its single
argument is a NOTA record decoded via the `nota` crate into a `Query`.

## Carve-outs worth knowing

- Read-only: it only reads `~/.claude/projects/<encoded-cwd>/*.jsonl`, never
  writes to a transcript.
- The transcript format belongs to Claude Code. Parse a tolerant subset and
  skip anything unrecognised; do not hard-fail on an unexpected line.
- The argument grammar lives on the `Query` enum in `src/query.rs`. Keep the
  NOTA doc there, in `ARCHITECTURE.md`, and in `README.md` in step.

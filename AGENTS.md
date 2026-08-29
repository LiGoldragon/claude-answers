# Agent instructions — claude-answers

## Repo role

A tiny read-only CLI that recalls your answers to Claude Code's
AskUserQuestion prompts from the on-disk session transcripts. Its single
argument is a typed Datomic value embodied as a `Query`.

## Carve-outs worth knowing

- Read-only: it only reads `~/.claude/projects/<encoded-cwd>/*.jsonl`, never
  writes to a transcript.
- The transcript format belongs to Claude Code. Parse a tolerant subset and
  skip anything unrecognised; do not hard-fail on an unexpected line.
- The argument grammar and checked recursive data anatomy live on `Query` in
  `src/query.rs`. Keep its documentation, `ARCHITECTURE.md`, and `README.md`
  in step with the authored Ethos map.

## Protos estate status

Stack: correct-new destination
Status: active component, current checkout legacy-wired
This checkout is not proof of correct-new adoption.

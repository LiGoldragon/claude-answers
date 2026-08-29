# Upgrades

## 0.3.0 — Datomic query boundary

Version 0.3.0 is a breaking data-boundary release.

- The CLI argument is now a typed Datomic `Query`, authored in
  `claude-answers.ethos` and embodied at the `Text<Query>` edge.
- The former Dotos codec is removed. Do not retain a translation path or a
  Dotos dependency; pass the same query values directly as Datomic text.
- Canonical multi-word query text uses curly quotes, for example
  `Grep.{All “Bluetooth adapter”}`. The earlier parenthesized input remains
  readable and textualizes canonically with curly quotes.
- `Query` payloads are now validated `QueryText` values. Build selectors with
  `Query::session`, `Query::file`, and `Query::grep` so outbound values are
  representable before they reach the command-line edge.

The recursive `Grep` data anatomy is checked D3 in `src/query.rs`: final
`RustEmitter::datomic_library()` output cannot provide `Datomic` for
`Box<Query>`. `tests/ethos_contract.rs` proves the authored Ethos map parses,
the complete grammar round-trips at `Text<Query>`, and wrong shapes are
refused.

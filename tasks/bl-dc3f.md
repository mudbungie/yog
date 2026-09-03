+++
title = "§9.5's typed config settings are derived and answered to nobody: config_edit::form has no carrier"
created = 1788414808
updated = 1788414808
priority = 3
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"
tags = ["boundary"]
+++
DESIGN §9.5 ("The config pane is controls over facts", bl-c225). Found by the
§11-rule sweep (bl-7dca).

§9.5 rules that a config file is not edited blind: **every setting the files
declare is answered as the typed thing it is**, judged at input rather than at
Apply. `src/config_edit/form.rs` and `src/config_edit/form/schema.rs` are that
enumeration — a `Schema` per file (which column-0 block holds its entries, what
fields an entry carries) and a `FieldSpec {name, control, help}` per setting,
reading and writing through `model_pick::grammar` so the typed view and the
§9.4 pick gate cannot disagree.

**Nothing outside those two files and their tests calls either.** No `Query`
answers a schema, a field, a bound or a judgement; `Query::ReadConfig` answers
raw bytes. So the whole of §9.5 — the settings table, the "judges at input"
ruling, the three justified raw-text fallbacks as a *choice between* typed and
raw — reaches no seat, and a seat editing `cadence.yaml` or `models.yaml` today
has exactly the blind `TextEdit` §9.5 was written to end.

Two facts make this worth closing rather than deleting:
- The bounds are shared with the worker (`src/app/cadence.rs` consts), so a
  seat re-deriving them is a second authority on the clock's own limits.
- `grammar::is_unknown_row` is the same function §9.4's pick gate and role
  marks call. A seat re-implementing the provider judgement is the drift §9.5
  explicitly says cannot happen ("they cannot disagree").

Shape questions: does `Query::ReadConfig` gain a `settings` array beside its
`text` (one answer, both views, the file staying the single fact)? Or a
`Query::ConfigSchema { file }` that is pure interface-description like
`Query::Help` — §8.5 already has one higher-order query and its argument
transfers. Either way the write is unchanged: `ApplyConfig` carries whole text,
and a typed edit is a seat composing that text through a grammar the engine
described to it.

Amend §9.5 (the whole section is stated as live behaviour), §9.2's "edited as
controls" amendment and §9.3's "the lineage and the path are CHOSEN" amendment
when it lands.
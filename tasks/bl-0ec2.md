+++
title = "help is a gesture: --help threads through every command at every seat"
created = 1785812918
updated = 1785812918
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"
+++
Operator ask (2026-08-03), verbatim: "fix those gaps. Add a help option to
every command, even at the top level. It's a higher order operation, which
should thread in the same way through all of the interfaces."

The gaps bl-ec8f left: `yog gesture --help` answers "--help needs a value"
(exit 2); help at the terminal exits 2 on stderr with a "yog gesture: " prefix;
there is no per-verb detail; and `yog --help` is clap's GUI-flag help, which
names none of the namespaces, hatches, or gestures.

The reframe: **help is a Query** (§8.5's taxonomy — it populates, it answers
typed data both frontends render, it has a headless spelling), and it is
*higher order*: it is asked ABOUT a command, in one shape at every seat. That
retires bl-ec8f's "no /help verb because it crosses no boundary" — it does
cross, as `Query::Help { verb }`.

Deliverables:
1. `boundary::help` — the verb table (moved from `line::ROSTER`, extended with
   per-verb detail), `rows(verb)`, `known(verb)`, and the one text rendering
   every seat prints. Single source for parse refusals, the roster, and detail.
2. `Query::Help { verb: Option<String> }` + codec spelling (strict decode
   refuses an unknown verb) + `answer` arm + `Reply::Help(Vec<HelpRow>)`.
3. The line's higher-order rule, applied once above the verb match: `/help`,
   `/help <verb>`, `<verb> --help|-h`, and a bare `/` are all the help query.
4. Help is the one query with no world to read, so every seat answers it in
   place: `yog gesture --help|-h|'/help'` prints to stdout and exits 0 — no
   deposit, no consumer, no timeout.
5. Top level: `yog --help|-h|help` states the whole surface — the window, the
   hatches (env/exec), headless, gesture, and the embedded namespaces — from
   the namespace table itself, not a restated string.
6. The composer renders the help reply as help, not raw JSON.
7. DESIGN §8.5 records help's place in the taxonomy and the one rule; README
   and the module map follow.
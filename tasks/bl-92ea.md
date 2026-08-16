+++
title = "a second yog on one box gets no wire, and only stderr says so"
created = 1786844346
updated = 1786844346
priority = 2
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"
+++
The wire`s default address is one constant for every world (`wire::provision::PORT`), so two yogs on one box contend for one port. That is not exotic: a dev instance under its own `XDG_DATA_HOME` is a different world with the same default, and a stale engine process is the same collision.

The loser is silent. `server::Listener::bind` fails `Address already in use`, `wire::listen` prints one line to stderr and answers `None`, `Engine::window_wire` answers `None` for want of a listener, and `main.rs` keeps the window anyway — so the frame has no asker, no poster and no searcher. Since REMOTE §1.2 made the window a client of its own engine, that is a window that paints no answer and fires no act, and the one diagnostic goes to a stream a desktop launch has nowhere to show.

Not bl-4c50, which only stopped the SUITE contending (a fixture world names `127.0.0.1:0`). This is the app.

Direction, not a decision. Either the failure must reach the frame as a visible refusal — REMOTE §8 already rules that a terminal instruction in front of a desktop launch is not an answer, so a refusal has to be paintable — or the local listener falls back to a kernel-chosen port and PUBLISHES what it bound where a local seat reads it, which touches "the address is one fact with one home" (§8) and must not make a remote seat`s known port a moving target.
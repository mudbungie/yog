+++
title = "the §3.4 empty-world bootstrap surface is never rendered by any test, and the one test that names it passes on the start pane's box instead"
created = 1786515533
updated = 1786515779
claimant = "Larkspur"
priority = 2
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"
tags = ["testing"]
+++
Found by Larkspur while sweeping bl-36c3. This is vacuity **shape 5** — *"a test that CONSTRUCTS the precondition the code depends on cannot discover that the code requires it"* — in its purest form: the fixture never reaches the surface at all, and the assertion is satisfied by a different widget than the one it names.

## The surface

`src/shell/bootstrap.rs` — the §3.4 / STORIES S0 empty-world placeholder: the wordmark, the tagline, the greyed §3.3 name prediction, one box and a Start. It is painted when `model.focused_workspace()` is `None`, and it is **the first thing a new operator ever sees**.

## The finding

Nothing in the tree renders it. Grepping every one of its own literals — `"say what you want done"`, `"start a conversation:"`, `theme::TAGLINE`, `"founds your first workspace"` — finds no test outside the file itself.

It is invisible to the coverage floor too: `src/shell/*` is excluded in `tarpaulin.toml`, so a surface with no render test reports exactly as one with five.

## The assertion that looks like coverage and is not

`src/shell/acceptance/focus.rs:44`:

    /// The empty world takes the same path — the bootstrap composer is not a
    /// special case with a focus flag of its own …
    #[test]
    fn the_empty_world_bootstrap_takes_the_launch_request_too() {
        let mut world = world_unfocused();
        let screen = Screen::new();
        assert!(
            screen.idle(&mut world),
            "with no workspace focused the bootstrap box takes the keyboard"
        );
    }

`fixture::world_unfocused()` is `build_world("hello", false)` — it builds a workspace and declines to focus an **agent**. `focused_workspace()` is therefore `Some`, `bootstrap::render` is never called, and what actually takes the keyboard is `shell::start_pane`'s box. Measured, from the frame that fixture paints:

```
"will be named saddlebag", "start a conversation", "New prompt",
"work directory:", "/tmp/…", "select a conversation — or start one below"
```

— the start pane. No wordmark, no tagline, no `Start`. The assertion passes on a box that is not the one its own message names. It is the bl-f16e pattern (*"assert on the IDENTITY of the thing the gesture was about, not on the presence of a shape"*) on the Rust side.

## The ask

1. A fixture whose world has **no workspace at all**, so `bootstrap::render` is reached. That is the missing half of `fixture::world_unfocused`, whose name promises it.
2. Re-aim `the_empty_world_bootstrap_takes_the_launch_request_too` at it, and make it name what it found — if the bootstrap box holds the keyboard, the bootstrap surface must be on screen in the same frame.
3. Then the surface can carry the assertions it has never had: the masthead's three runs whole and stacked, and its own Start. **bl-fb1c is open against exactly this masthead** (the wordmark left-aligned while the two lines below it are centred) and has no regression test it could be proved by — this fixture is its prerequisite.

I wrote and then withdrew a masthead assertion under bl-36c3 for want of this fixture; the withdrawal is the reason this ball exists rather than a note.

Verify all cited paths against HEAD; ball bodies drift.
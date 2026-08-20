+++
title = "run-s5s8 can type into Conversation and start a model after its Config shortcut misses"
created = 1787206330
updated = 1787206330
priority = 1
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"
tags = ["bug", "drive", "testing"]
+++
## Reproduction

Run `scripts/drive/drive.sh ladder run-s5s8` in fresh isolated worlds. In one of two consecutive runs, `Ctrl+Shift+2` did not move the center from Conversation to Config; the three Config evidence screenshots still showed Conversation. The immediate retry landed Config and passed.

The harness waits a fixed two seconds but never asserts the selected tab. Its Config locator then accepted Conversation geometry, typed the configuration marker into the composer, and the later click caused a `lernie prompt` row.

## Contract and risk

The run describes this fixture as making zero model calls. The isolated failing run had no usable wire and stopped before spend; a configured wall could spend and create a conversation during a supposedly no-wire acceptance run.

The documented keymap does bind `Ctrl+Shift+2` to Config even while the composer owns keyboard focus, so either delivery is intermittent or the drive races it. In either case the harness must not act on an unproved frame.

## Required invariant

Wait on a monotone Config-selected predicate before locating or typing. Make each locator reject a semantically wrong center view, so failed navigation cannot be converted into a different destructive gesture. Preserve a regression proving no `lernie prompt` or model call occurs on every failure path.
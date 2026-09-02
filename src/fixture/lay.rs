//! **The writer**: a [`Recipe`] onto a scratch data root, through the same
//! folds the engine reads it back with.
//!
//! What goes where is [`super::places`]; how a byte is written — and why none
//! of it panics — is [`super::disk`]. This file is the middle: which of those
//! files one [`Recipe`] means.
//!
//! **Times are the recipe's, never the clock's.** Each conversation's dispatch
//! commit, messages and step are stamped `origin - age_secs`, and the step's
//! `response.json` mtime is stamped with them — `last_action_unix` is the
//! newest of those three, so a file left at "now" would drown every dated
//! commit and collapse the §11 order into an id tiebreak.

use super::disk::{display, git, mkdir, stamp, write};
use super::places::Places;
use super::recipe::{Conv, Recipe, Step, Wsp};
use crate::registry;
use crate::wire::material::{LEAVES, Role};
use std::path::{Path, PathBuf};

/// litany's seeded marker (§16.6 W3): with it present the engine skips
/// `litany prime`, which is the general path with the seed already there.
const MODELS_YAML: &str = "models:\n  default:\n    provider: anthropic\n";

/// The §4.4 tails each [`Step`] arm writes into `response.json`.
const SETTLED: &str = "{\"type\":\"finish\"}\n{\"type\":\"end\"}\n";
const FAILED: &str = "{\"type\":\"error\",\"message\":\"the model refused\"}\n{\"type\":\"end\"}\n";
const LIMITED: &str = "{\"type\":\"finish\",\"reason\":\"length\"}\n{\"type\":\"end\"}\n";
const STREAMING: &str = "{\"type\":\"content_delta\",\"delta\":{\"text\":\"Reading the \"}}\n\
     {\"type\":\"content_delta\",\"delta\":{\"text\":\"module map\"}}\n";

/// The step every recipe's newest is, zero-padded to the §2.3 width.
const SEQ: &str = "000";

/// The date every `config/default` root commit carries — a **fixed** instant,
/// not the recipe's origin, so the trunk's oid is byte-identical on every lay
/// and a harness can compare a rendered `config_tip` across runs. The agent
/// commits below cannot join it: their dates are what `age_secs` is measured
/// from, and an age is only stable relative to a moving now.
const TRUNK_EPOCH: i64 = 1_767_225_600;

/// Lay `recipe` under `root`, stamped at `origin`. Returns the paths a harness
/// holds open to make a `Streaming` step read as a live model call — see
/// [`super::Laid::hold`] for why that cannot be a file on disk.
pub fn lay(root: &Path, recipe: &Recipe, origin: i64) -> Result<Vec<PathBuf>, String> {
    let places = Places::under(root);
    mkdir(&places.litany)?;
    write(&places.litany.join("models.yaml"), MODELS_YAML)?;
    mkdir(&places.state)?;
    if let Some(body) = recipe.cadence {
        write(&places.state.join(crate::app::cadence::CADENCE_YAML), body)?;
    }
    let mut hold = Vec::new();
    for wsp in recipe.workspaces {
        lay_workspace(&places, wsp, recipe.brazen, origin, &mut hold)?;
    }
    Ok(hold)
}

/// One workspace: its repo, its conversations, its wall, and the registrations
/// that make it visible to a wire client at all (REMOTE §4 — an unregistered
/// client's snapshot holds no workspace, which is indistinguishable from an
/// empty world and is exactly the trap a fixture must not lay).
fn lay_workspace(
    places: &Places,
    wsp: &Wsp,
    brazen: Option<&str>,
    origin: i64,
    hold: &mut Vec<PathBuf>,
) -> Result<(), String> {
    let ws = places.workspace(wsp.name);
    found(&ws)?;
    for conv in wsp.convs {
        lay_conv(&ws, conv, origin, hold)?;
    }
    if let Some(body) = brazen {
        let wall = places.walls.join(wsp.name).join("brazen");
        mkdir(&wall)?;
        write(&wall.join("config.toml"), body)?;
    }
    // Every leaf the mint issues except the server's — a harness may present
    // any of them, and a certificate that is admitted but registered nowhere
    // sees an empty world, which is REMOTE §4's *absent, not forbidden* read as
    // a broken fixture. `filter_map` over `parse`: the names come from
    // [`Role::common_name`] and are valid by construction, so there is no
    // failure of it for an arm here to answer.
    for client in LEAVES
        .iter()
        .filter(|role| **role != Role::Server)
        .filter_map(|role| registry::Client::parse(&role.common_name()).ok())
    {
        registry::register(&places.state, &client, wsp.name)
            .map_err(|e| format!("register {}: {e}", wsp.name))?;
    }
    Ok(())
}

/// The bare `repo.git` and its orphan `config/default` root (ARCH §2.2) — the
/// lineage every conversation branch forks off, and the `repo.git` marker
/// that makes the directory a workspace at all.
fn found(ws: &Path) -> Result<(), String> {
    let repo = ws.join("repo.git");
    mkdir(&repo)?;
    git(
        &repo,
        &["init", "-q", "--bare", "-b", "config/default"],
        None,
    )?;
    let author = ws.join(".author");
    let author_s = display(&author);
    git(
        &repo,
        &[
            "worktree",
            "add",
            "-q",
            "--orphan",
            "-b",
            "config/default",
            &author_s,
        ],
        None,
    )?;
    write(&author.join("version"), "1\n")?;
    git(&author, &["add", "version"], None)?;
    git(
        &author,
        &["commit", "-q", "-m", "config: init"],
        Some(TRUNK_EPOCH),
    )?;
    git(&repo, &["worktree", "remove", &author_s], None)
}

/// One conversation: the `agents/<id>` branch and worktree, its transcript and
/// summaries, its marks, its deposits and its newest step.
fn lay_conv(ws: &Path, conv: &Conv, origin: i64, hold: &mut Vec<PathBuf>) -> Result<(), String> {
    let repo = ws.join("repo.git");
    let at = origin - conv.age_secs;
    let wt = ws.join("agents").join(conv.id);
    let branch = format!("agents/{}", conv.id);
    git(
        &repo,
        &[
            "worktree",
            "add",
            "-q",
            "-b",
            &branch,
            &display(&wt),
            "config/default",
        ],
        None,
    )?;
    write(&wt.join("goal.md"), conv.goal)?;
    git(&wt, &["add", "goal.md"], None)?;
    git(&wt, &["commit", "-q", "-m", "dispatch"], Some(at))?;
    for (name, body) in conv.messages {
        let path = wt.join("messages").join(name);
        write(&path, body)?;
        stamp(&path, at)?;
    }
    for (name, body) in conv.summaries {
        write(&wt.join("summary").join(name), body)?;
    }
    for (name, body) in conv.deposits {
        write(
            &ws.join("inbox").join(conv.id).join(format!("{name}.md")),
            body,
        )?;
    }
    for mark in conv.marks {
        let refname = format!("refs/litany/{mark}/{}", conv.id);
        git(&repo, &["update-ref", &refname, &branch], None)?;
    }
    lay_step(ws, conv, at, hold)
}

/// The newest step, in the one shape [`Step`] names. A `Wound` writes the step
/// directory with **no** `response.json` and **no** `meta.json`, which is the
/// whole of the §7.3 predicate; every other arm writes a settled or unsettled
/// tail and the `meta.json` litany writes once a call returns.
fn lay_step(ws: &Path, conv: &Conv, at: i64, hold: &mut Vec<PathBuf>) -> Result<(), String> {
    if conv.step == Step::Absent {
        return Ok(());
    }
    let steps = ws.join("steps").join(conv.id);
    let step = steps.join(SEQ);
    write(&step.join("request.json"), "{\"model\":\"fixture-1\"}\n")?;
    if !conv.driver_log.is_empty() {
        write(&steps.join("driver.log"), conv.driver_log)?;
    }
    let tail = match conv.step {
        Step::Settled => SETTLED,
        Step::Failed => FAILED,
        Step::OutputLimit => LIMITED,
        Step::Streaming => STREAMING,
        Step::Absent | Step::Wound(_) => "",
    };
    if let Step::Wound(words) = conv.step {
        write(&step.join("stderr.log"), words)?;
        return Ok(());
    }
    let response = step.join("response.json");
    write(&response, tail)?;
    stamp(&response, at)?;
    if conv.step == Step::Streaming {
        hold.push(ws.join("inbox").join(conv.id));
        hold.push(response);
        return mkdir(&ws.join("inbox").join(conv.id));
    }
    write(&step.join("meta.json"), &meta(at))
}

/// The `meta.json` litany writes once a model call returns — present for every
/// settled arm, and its absence is half the wound predicate.
fn meta(at: i64) -> String {
    let stamp = crate::ui_state::iso8601_extended(at);
    format!("{{\"started_at\":\"{stamp}\",\"ended_at\":\"{stamp}\"}}\n")
}

#[cfg(test)]
mod tests;

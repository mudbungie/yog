//! §3.7 — the project-instruction freeze on the fire's one argv: the `--pin`
//! specs a bound rung carries, and the unbound rung that carries none.

use super::prompt::{make_fifo, prepared, workspace};
use super::{World, write_exec};
use crate::cli_outbound::Cli;
use crate::start::execute_prompt;
use crate::test_support::spawn_guard;

/// §3.7: the freeze rides the same one argv as the binding — one `--pin
/// <dest>=<src>` per instruction document the binding's project declares,
/// before the workspace and the goal, so `clip_goal` still trims exactly the
/// payload and the logged row carries the whole provenance.
#[test]
fn prompt_pins_the_bound_projects_instructions_before_the_goal() {
    let _g = spawn_guard();
    let w = World::new();
    let fifo = w.bin.path().join("report");
    make_fifo(&fifo);
    let body = format!(
        "#!/bin/sh\nprintf '%s\\037%s\\037%s\\037%s\\037%s' \"$4\" \"$5\" \"$6\" \"$7\" \"$8\" > '{}'\n",
        fifo.display()
    );
    let lernie = Cli::new(write_exec(w.bin.path(), "lernie", &body));
    let ws = workspace(&w);
    // A bound project declaring instructions at its own authority root.
    let target = w.balls.path().join("work");
    std::fs::create_dir_all(target.join(".git")).unwrap();
    std::fs::write(target.join("AGENTS.md"), "house rules").unwrap();
    execute_prompt(
        &lernie,
        w.state.path(),
        "TS",
        &crate::start::Fire {
            workspace: ws.clone(),
            prepared: prepared("cobalt-gecko", Some(&target)).clone(),
            goal: "do it".to_owned(),
        },
        &[],
        &super::rng(),
    )
    .unwrap();

    let recorded = std::fs::read_to_string(&fifo).unwrap();
    let fields: Vec<&str> = recorded.split('\u{1f}').collect();
    let spec = format!(
        "instructions/00/AGENTS.md={}",
        target.join("AGENTS.md").display()
    );
    assert_eq!(fields[0], "--cwd");
    assert_eq!(fields[1], target.to_string_lossy());
    assert_eq!(fields[2], "--pin");
    assert_eq!(fields[3], spec);
    assert_eq!(
        fields[4],
        ws.to_string_lossy(),
        "the workspace still follows"
    );
    // The logged argv is built from the same list, so the trail IS the record.
    let e = &w.ops()[0];
    assert_eq!(&e.argv[6..8], ["--pin", spec.as_str()]);
    assert_eq!(e.argv[9], "do it", "the goal is still last");
}

/// The bare rung binds nothing, so it discovers nothing: no policy read, no
/// stat, no pin. The general path with empty inputs.
#[test]
fn an_unbound_rung_freezes_no_instructions() {
    let _g = spawn_guard();
    let w = World::new();
    let fifo = w.bin.path().join("report");
    make_fifo(&fifo);
    let body = format!(
        "#!/bin/sh\nprintf '%s\\037%s' \"$4\" \"$5\" > '{}'\n",
        fifo.display()
    );
    let lernie = Cli::new(write_exec(w.bin.path(), "lernie", &body));
    let ws = workspace(&w);
    execute_prompt(
        &lernie,
        w.state.path(),
        "TS",
        &crate::start::Fire {
            workspace: ws.clone(),
            prepared: prepared("cobalt-gecko", None).clone(),
            goal: "do it".to_owned(),
        },
        &[],
        &super::rng(),
    )
    .unwrap();
    let recorded = std::fs::read_to_string(&fifo).unwrap();
    assert_eq!(recorded, format!("{}\u{1f}do it", ws.to_string_lossy()));
}

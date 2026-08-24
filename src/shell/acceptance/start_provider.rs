//! **The start pane's first rung is provider sign-in on a wall that has none**
//! (bl-1fd0), driven through the real window.
//!
//! The operator ruling: on a wall holding no usable provider credential,
//! typing a goal and hitting Enter works *zero percent of the time* — the
//! conversation is born, dies on no-models, and the first goal is spent. It was
//! hit live twice in one evening. The pane was inviting the one act that cannot
//! succeed while hiding the one act that must come first.
//!
//! Three states, one per beat, because any one of them alone passes on a
//! defect: the rung must **appear** where the wall is bare, must be **absent**
//! where it is not (today's flow is untouched on a signed wall), and must
//! **dissolve** when the sign-in it offered lands — with the draft still in the
//! box, which is the thing the ruling was protecting.
//!
//! The fixture wall is brazen's own built-in table with nothing signed in,
//! which is exactly the wall the ruling was filed from: `ollama` and
//! `claude-code` sit in it at `not required` on every wall there can be, so a
//! predicate that counted them would make this whole file vacuous.

use std::path::Path;

use super::fixture::{World, sign_wall_in, world};
use super::screen::Screen;
use crate::cli_outbound::Cli;
use crate::start::Prepared;

/// The draft the operator has typed and must not lose.
const DRAFT: &str = "wire the loom to the gate";

/// A fragment of the rung's own sentence.
const RUNG: &str = "nothing on this wall is signed in";

/// A fragment of the clause that names what the operator can see and must not
/// count on — the keyless rows brazen ships under every config.
const KEYLESS: &str = "keyless rows";

/// Seat what `▶ Start` leaves behind: a goal awaiting Send.
fn draft(world: &mut World) {
    let ws = world.ws.clone();
    world.state.start.pending = Some(Prepared {
        workspace: crate::naming::leaf(&ws),
        binding: None,
        goal: DRAFT.to_owned(),
        origin: crate::opslog::Origin::Balls,
        lineage: None,
    });
}

/// A `bz --login` that **succeeds**: it writes the credential file brazen reads
/// `stored` off, then exits 0 — the shape of a browser sign-in that landed. Not
/// a stub that merely exits 0: the rung dissolves off the re-read of the table,
/// so a run that changed nothing would leave it standing and the beat would be
/// asserting the fold rather than the outcome.
fn bz_that_signs_in(dir: &Path, creds: &Path) -> Cli {
    use std::os::unix::fs::PermissionsExt;
    let path = dir.join("bz-signs-in");
    std::fs::create_dir_all(dir).unwrap();
    std::fs::write(
        &path,
        format!(
            "#!/bin/sh\nmkdir -p '{creds}'\n\
             printf '%s' '{{\"ApiKey\":{{\"key\":\"signed-by-the-fake\"}}}}' > '{creds}/anthropic.json'\n\
             echo 'authorize at the URL' >&2\nexit 0\n",
            creds = creds.display()
        ),
    )
    .unwrap();
    let mut perms = std::fs::metadata(&path).unwrap().permissions();
    perms.set_mode(0o755);
    std::fs::set_permissions(&path, perms).unwrap();
    Cli::new(path)
}

/// The seated wall's credentials dir — where a sign-in's credential lands, read
/// off the holder rather than re-derived (§16.2 has one answer to "which wall",
/// and it is the one the frame seated).
fn creds_dir(world: &World) -> std::path::PathBuf {
    let wall = world
        .state
        .wall
        .login
        .wall
        .iter()
        .find(|(key, _)| key == crate::world::wall::YOG_WALL)
        .map(|(_, value)| std::path::PathBuf::from(value))
        .expect("a frame has painted, so the start's wall is seated");
    crate::config_edit::brazen::BrazenPaths::in_wall(&wall).credentials_dir
}

/// **The rung stands in front of the goal box, and Send does not fire.**
///
/// Both halves, because either alone passes on a defect: a pane that painted
/// the sentence and fired anyway would still spend the goal, and one that
/// refused without saying why would be the dead row the ruling is about.
#[test]
fn a_bare_wall_leads_with_sign_in_and_refuses_the_goal() {
    let mut world = world();
    let screen = Screen::new();
    screen.idle(&mut world);
    draft(&mut world);

    let painted = screen.text(&mut world);
    assert!(
        painted.contains(RUNG) && painted.contains(KEYLESS),
        "the rung says why the goal box is not yet the point:\n{painted}"
    );
    assert!(
        painted.contains(DRAFT),
        "the goal box stays draftable under it — the draft is what is being \
         protected, not withheld:\n{painted}"
    );
    // The remedy is offered where the reason is, not one tab away: the §8.3
    // roster paints under the sentence with its rows and their verb.
    assert!(
        painted.contains("Login (bz browser sign-in)") && painted.contains("openai-chatgpt"),
        "and the sign-in that fixes it is right there — the whole roster, \
         including the one row a default table can browser-sign-in:\n{painted}"
    );
    // The band is a band, not a takeover: the box it gates keeps its own panel
    // and its own verb row. Painted inside the box's panel this was the defect
    // — the roster ate the room and Send was clipped off the bottom.
    assert!(
        painted.contains("Send (detached prompt)"),
        "and the goal box below it is whole:\n{painted}"
    );

    // The §11 Enter binding is the other hand on Send's trigger, so the refusal
    // is asserted through it: a landed fire spends the §3.3 seed and clears the
    // draft (bl-28ba), and a refused one does neither.
    let seed = world.state.start.mint_seed;
    super::super::start_pane::send_pending(&mut world.model, &mut world.state);
    screen.idle(&mut world);
    assert_eq!(
        world.state.start.mint_seed, seed,
        "nothing was minted — the fire never left"
    );
    assert_eq!(
        world.state.start.pending.as_ref().map(|p| p.goal.as_str()),
        Some(DRAFT),
        "and the operator's typed goal is still in the box, not spent on a dead \
         conversation"
    );
}

/// **A signed wall gets today's flow, byte for byte.** Without this the beat
/// above passes on a pane that leads with a sign-in rung forever.
#[test]
fn a_signed_wall_paints_no_rung_at_all() {
    let mut world = world();
    let screen = Screen::new();
    screen.idle(&mut world);
    sign_wall_in(&mut world);
    draft(&mut world);

    let painted = screen.text(&mut world);
    assert!(
        painted.contains(DRAFT) && painted.contains("Send (detached prompt)"),
        "the goal box is the pane:\n{painted}"
    );
    assert!(
        !painted.contains(RUNG) && !painted.contains(KEYLESS),
        "and nothing is said about providers — this wall can run:\n{painted}"
    );
    assert!(
        !painted.contains("Login (bz browser sign-in)"),
        "nor is a roster stacked over the composer:\n{painted}"
    );
}

/// **A sign-in that lands dissolves the rung and keeps the draft.**
///
/// The dissolve is the outcome's, not a click's: the holder folds a clean exit
/// back into the rows the one frame the streamed run settles, so the credit
/// flips inside the same call that painted the rung and the next frame is the
/// signed wall's.
#[test]
fn a_sign_in_completing_mid_pane_dissolves_the_rung_and_keeps_the_draft() {
    let mut world = world();
    let screen = Screen::new();
    screen.idle(&mut world);
    draft(&mut world);
    assert!(
        screen.text(&mut world).contains(RUNG),
        "the rung is up, or there is nothing here to dissolve"
    );

    let bz = bz_that_signs_in(&world.yog_data, &creds_dir(&world));
    let run = crate::login::start(
        &bz,
        "anthropic",
        world.model.state_root(),
        "t0",
        Some(&world.ws),
    )
    .expect("the fake bz spawns");
    world.state.wall.login.begin(run);

    // A streamed child's exit arrives on the drain thread, so the frame that
    // planted the run is not the frame that settles it.
    let mut painted = String::new();
    for _ in 0..200 {
        painted = screen.text(&mut world);
        if !painted.contains(RUNG) {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    assert!(
        !painted.contains(RUNG),
        "the rung dissolves on the sign-in's own outcome, with no second \
         gesture:\n{painted}"
    );
    assert!(
        painted.contains(DRAFT),
        "and the draft the operator typed through it is untouched:\n{painted}"
    );
    assert!(
        world.state.wall.login.credit.credentialed,
        "because the wall really did become one that can run"
    );
}

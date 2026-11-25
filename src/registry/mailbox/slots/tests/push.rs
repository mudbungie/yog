//! **The pushing read** (REMOTE §5.8, bl-f5cd): a job as it is queued, while
//! others run; what a reconnect's `holding` leaves alone and takes back; the
//! heartbeat; the takeover; and an ask nobody waits on, not handed.

use super::super::super::doubt::NOT_HELD;
use super::super::push::Push;
use super::super::read::Taken;
use super::stream::piece;
use super::{Mailbox, NOW, call, poll, quick};

fn open(mail: &Mailbox, holding: &[&str], now: i64) -> Push {
    let held: Vec<String> = holding.iter().map(|id| (*id).to_owned()).collect();
    mail.push("laptop", &held, Box::new(move || now))
}

/// The id of the one job a frame carries.
fn job(frame: Option<Taken>) -> String {
    let work = frame.expect("a frame").work();
    assert_eq!(work.len(), 1, "one job per frame");
    work.into_iter().map(|job| job.id).collect()
}

/// **No limit, no waiting its turn.** A job queued while the host runs
/// another is written to it at once, and the running one is left alone.
#[test]
fn a_job_queued_while_another_runs_is_pushed_at_once() {
    let mail = quick();
    let first = mail.post(NOW, "driver", &call("laptop", "Bash"));
    let mut read = open(&mail, &[], NOW);
    assert_eq!(job(read.next()), first);
    mail.complete(NOW, "laptop", &first, &piece("half", ""))
        .expect("the first is running");

    let second = mail.post(NOW, "driver", &call("laptop", "Bash"));
    assert_eq!(job(read.next()), second, "pushed while the first runs");
    assert_eq!(
        read.next(),
        Some(Taken::Work(Vec::new())),
        "and nothing else"
    );
    assert_eq!(
        poll(&mail, "driver", &first),
        Ok(None),
        "the first still runs"
    );
    assert!(!mail.busy(NOW, "laptop"), "a pushing host is never busy");
}

/// **A quiet read writes an empty frame on the hold's bound** — the heartbeat
/// a host ignores, and the write that finds a dead connection.
#[test]
fn a_quiet_read_beats() {
    let mail = quick();
    let mut read = open(&mail, &[], NOW);
    for _ in 0..3 {
        assert_eq!(read.next(), Some(Taken::Work(Vec::new())));
    }
    assert!(mail.serving("laptop"), "and it stays open");
}

/// **The reconnect.** Listed: left alone. Unlisted with nothing posted: handed
/// again under its id. Unlisted with a post: answered in doubt, naming why.
/// Unknown, or another machine's: ignored. And the read it replaced is told.
#[test]
fn holding_settles_the_lease_on_reconnect() {
    let mail = quick();
    let ids: Vec<String> = (0..3)
        .map(|_| mail.post(NOW, "driver", &call("laptop", "Bash")))
        .collect();
    let phone = mail.post(NOW, "driver", &call("phone", "Bash"));
    let mut before = open(&mail, &[], NOW);
    for id in &ids {
        assert_eq!(&job(before.next()), id);
    }
    mail.complete(NOW, "laptop", &ids[2], &piece("began", ""))
        .expect("posted for");

    let mut after = open(&mail, &[&ids[0], "inv-999", &phone], NOW);
    assert_eq!(
        job(after.next()),
        ids[1],
        "unlisted, unposted: handed again"
    );
    assert_eq!(after.next(), Some(Taken::Work(Vec::new())), "nothing more");
    assert_eq!(
        poll(&mail, "driver", &ids[0]),
        Ok(None),
        "listed: left alone"
    );
    let doubt = poll(&mail, "driver", &ids[2])
        .expect("answered")
        .expect("ended");
    assert_eq!(doubt.exit_code, 1);
    assert!(
        doubt.stdout.starts_with("began") && doubt.stderr.contains(NOT_HELD),
        "{doubt:?}"
    );
    assert_eq!(
        poll(&mail, "driver", &phone),
        Ok(None),
        "another machine's: untouched"
    );
    assert_eq!(
        before.next(),
        Some(Taken::Superseded),
        "the old read is told"
    );
    assert_eq!(before.next(), None, "and ends");
}

/// **One reader**: a pushing read takes the queue from a serial one, and a
/// bare read under the same certificate is still refused beside it.
#[test]
fn a_pushing_read_takes_the_queue_from_a_serial_one() {
    let mail = Mailbox::holding(4000, std::time::Duration::from_millis(5));
    let old = mail.clone();
    let serial = std::thread::spawn(move || old.take("laptop"));
    while !mail.serving("laptop") {
        std::thread::sleep(std::time::Duration::from_millis(1));
    }
    let _read = open(&mail, &[], NOW);
    assert_eq!(
        serial.join().expect("the serial read"),
        Ok(Taken::Superseded)
    );
    assert!(mail.take("laptop").is_err(), "a bare read is still refused");
}

/// **An ask nobody waits on is not run for nobody** (REMOTE §5.7's rule at the
/// hand-off): queued, kept, and pushed once its asker asks again.
#[test]
fn an_unheard_ask_waits_for_its_asker() {
    let mail = quick();
    let id = mail.post(NOW, "driver", &call("laptop", "Bash"));
    let late = NOW + 31;
    let mut read = open(&mail, &[], late);
    assert_eq!(read.next(), Some(Taken::Work(Vec::new())), "not handed");
    mail.collect(late, "driver", &id, None)
        .expect("its asker is back");
    assert_eq!(job(read.next()), id);
}

/// **Ids outlive a restart** (REMOTE §5.8): a booted mailbox mints from its
/// clock, so an id a host still holds is never a new job's.
#[test]
fn a_booted_mailbox_mints_past_every_earlier_life() {
    let id = Mailbox::booted(7).post(NOW, "driver", &call("laptop", "Bash"));
    assert_eq!(id, "inv-7000001");
    let id = Mailbox::booted(-1).post(NOW, "driver", &call("laptop", "Bash"));
    assert_eq!(id, "inv-1");
}

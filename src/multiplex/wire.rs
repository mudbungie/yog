//! The two wire client arms (REMOTE §8, §5; bl-b6fa, bl-024b) — see the row
//! for this module in [`super`].

fn world() -> crate::xdg::Env {
    crate::world::compose(&crate::xdg::Env::from_env())
}

pub(super) fn seat(args: &[String]) -> i32 {
    crate::wire::seat::run(&world(), args)
}

pub(super) fn tool_host(args: &[String]) -> i32 {
    crate::wire::host::run(&world(), args)
}

//! The **stdin-piped** spawn shape (REMOTE §5.2, bl-024b): a child that is
//! handed a document and streams its answer back.
//!
//! Split from [`super`] the way [`exec`](super::exec) is, and for the same
//! reason: [`super`] holds the [`Cli`] handle and the three shapes that were
//! always there, and this is the fourth. It differs from
//! [`run_in`](Cli::run_in) in exactly one respect — the child's stdin is a pipe
//! yog writes and closes, rather than `/dev/null` — so it reuses that spawn
//! whole rather than restating it.
//!
//! **It is lernie's own tool contract** (its ARCH §3.3): the `tool_use.input`
//! JSON on stdin, bytes on stdout, the exit code the verdict. A tool host's
//! executable is therefore the same kind of program a local pool tool is
//! (REMOTE §5.2), which is why the far end needed no vocabulary of its own.

use std::path::Path;

use super::{Cli, CliError, Stream};

impl Cli {
    /// Spawn `<binary> <args…>` with `input` on its stdin, streaming stdout and
    /// stderr. `cwd` sets the child's working directory when given. Dropping
    /// the returned [`Stream`] terminates the child (SIGTERM, then SIGKILL
    /// after a short grace) — which is how a deadline is enforced on one.
    pub fn run_input(
        &self,
        cwd: Option<&Path>,
        input: &[u8],
        args: &[&str],
    ) -> Result<Stream, CliError> {
        self.run_streaming(cwd, &[], args, Some(input))
    }
}

#[cfg(test)]
mod tests;

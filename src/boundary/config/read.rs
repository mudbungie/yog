//! The §9 destination's bytes, read (§8.5, bl-0164): split from [`super`] per
//! §12's line budget, mirroring [`write`](super::write).

use crate::config_edit::{RealFileIo, load_snapshot};
use std::path::Path;

/// Why a lineage destination refuses a read: browsing which files a config
/// commit holds is the config pane's own gesture (bl-ee0a), over every file
/// in the lineage at once — a fact one destination cannot carry.
pub(super) const BRANCH_REFUSAL: &str =
    "a lineage's files are the config pane's own browse (git show), not a boundary read";

/// One file's current bytes as text, or empty for a file that is not there
/// yet — the same [`load_snapshot`] every §9 editor loads through, minus the
/// hash it also returns: a read answers once and holds no draft to guard
/// later.
pub(super) fn text_at(path: &Path) -> Result<String, String> {
    load_snapshot(&RealFileIo, path)
        .map(|(text, _)| text)
        .map_err(|e| e.to_string())
}

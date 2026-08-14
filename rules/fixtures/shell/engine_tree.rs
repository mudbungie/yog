// Deliberate violation fixture for rules/no-engine-tree-in-paint.yml (a
// shell-path file). The rules-audit's negative check scans rules/fixtures and
// requires a non-zero exit — these three names are the rule's bite: the
// engine's tree derivation reached from paint code, which a thin seat could
// never hold (REMOTE §9.4).
use crate::git_tree::Agent;

fn glue(agents: &[Agent], tree: &crate::git_tree::GitTree) -> Option<crate::git_tree::CommitNode> {
    let _ = agents;
    tree.commits.last().cloned()
}

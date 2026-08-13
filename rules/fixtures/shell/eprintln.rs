// Deliberate violation fixture for rules/no-eprintln-in-shell.yml (a shell-path
// file). The rules-audit's negative check scans rules/fixtures and requires a
// non-zero exit — this eprintln! is the shell-scoped rule's bite.
fn glue() {
    eprintln!("yog: this must be flagged");
}

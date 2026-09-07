//! The shipped default table.

use super::*;

#[test]
fn the_shipped_table_passes_everything_but_loss_and_credentials() {
    // The four classes that are the job pass, open-world among them; only
    // irreversible loss and credential access decline in band.
    assert_eq!(Table::ruling(Effect::Read), Ruling::Pass);
    assert_eq!(Table::ruling(Effect::TargetWrite), Ruling::Pass);
    assert_eq!(Table::ruling(Effect::Process), Ruling::Pass);
    assert_eq!(Table::ruling(Effect::OpenWorld), Ruling::Pass);
    assert_eq!(Table::ruling(Effect::Destructive), Ruling::Refuse);
    assert_eq!(Table::ruling(Effect::Secret), Ruling::Refuse);
    assert_eq!(Table::ruling(Effect::Opaque), Ruling::Hold);
}

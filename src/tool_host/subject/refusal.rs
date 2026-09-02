//! **The worktree lane's sentences** — what the model reads when no machine
//! can be routed to (REMOTE §5.4). Split out of the lane itself (bl-68e1)
//! because they answer a different question: [`super::verdict`] decides which
//! rung a name lands on, and this file decides what a landing on a refusing
//! rung *says*. The lane's own arms are provable without an engine and so are
//! these, which is why the two halves never needed to sit together.

/// **The clause both zero-consent refusals close on** (bl-68e1): the loaded
/// lane is not a remedy for a workspace-subject name, and the refusal says so
/// rather than offering it.
///
/// A load binds a host-qualified instance, and a loaded invocation carries no
/// directory at all — that is REMOTE §5's definition, locality rides in the
/// name — so the far machine runs the argv in whatever directory its own
/// process inherited. For a name whose subject is *this* conversation's
/// working tree that is a different place, and it is the one place thrall's
/// own DESIGN §3.4 refuses to resolve against: *"a place nobody wrote down,
/// which changes when the unit file does, and which nothing in the running
/// system reports."*
///
/// The old sentences named it first, as the remedy the model could take
/// itself — *"load what one advertises"*, *"load it … to run it in that
/// machine's own directory"* — and a drive took it: every write, every test
/// run and every `ls` the model made to check itself happened in the foot's
/// inherited directory, so the check could not fail, the conversation
/// reported success, and the bound directory was empty. Nothing at the
/// boundary could tell the two runs apart, because `/files` and `/work-diff`
/// read the conversation's own tree. **The operator's config edit is the only
/// remedy that puts the work where its subject is**, so it is the only one
/// offered; the lane the model can reach unaided is named as what it is not.
const NOT_A_REMEDY: &str = "loading a machine's tool with the clients tool is not a way to do this \
     work: a loaded instance runs in that machine's own inherited working \
     directory, never this conversation's, and nothing this conversation can \
     read would show what it wrote there";

/// The zero-consent refusal, in the two shapes it honestly has: machines
/// advertise the name but none consents, or nothing advertises it at all.
/// Both name the operator's remedy — the reader is a model and the fixer is
/// an operator — and both close on [`NOT_A_REMEDY`].
pub fn unconsented(name: &str, advertisers: &[String]) -> String {
    if advertisers.is_empty() {
        return format!(
            "no tool of that name is loaded in this conversation, this engine \
             does not implement {name}, and no machine of this workspace \
             advertises it; the operator enrolls a thrall on the box that holds \
             this server's worktrees, with \"subject_cwd\": true on a {name} \
             entry in its tools.json — {NOT_A_REMEDY}"
        );
    }
    format!(
        "{} advertises {name}, but no machine of this workspace consents to run \
         it in this conversation's working directory; the operator adds \
         \"subject_cwd\": true to the {name} entry in tools.json on the box \
         that holds this server's worktrees — {NOT_A_REMEDY}",
        names(advertisers),
    )
}

/// The config ambiguity: more than one machine consents, and one adjudication
/// must stand for exactly one execution on one machine (REMOTE §5, no
/// broadcast). Every claimant is named, because the fixer is the operator who
/// authored both entries.
pub fn ambiguous(name: &str, consenting: &[String]) -> String {
    format!(
        "{} machines consent to run {name} in this conversation's working \
         directory ({}), and one execution needs one machine: the operator \
         must leave \"subject_cwd\": true on exactly one entry",
        consenting.len(),
        names(consenting),
    )
}

/// A comma-joined machine list for a sentence.
fn names(clients: &[String]) -> String {
    clients.join(", ")
}

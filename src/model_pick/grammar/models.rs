//! The `models:` half of the §9.4 block grammar — **yog's own table, in
//! lernie's file** (bl-d9cb): the `models.yaml` block the §9.2 Declare control
//! authors and the §9.5 typed rows edit, plus the readers over it.
//!
//! **It used to be lernie's, and the picker used to write it.** The whole
//! justification was a cross-check — a role naming a model `models.yaml` did
//! not declare was a hard load error, so a pick wrote this half FIRST and the
//! assignment second. That check is gone at the pin: lernie's own
//! `config/cross/mod.rs` says *"There is no roles-against-models check any more
//! (bl-35e2): the global `models.yaml` carries no `models:` table, a role's
//! `providers.yaml` assignment is the single home of its (provider row, model
//! id) pointer"*, and its `config/models.rs` deserializes the file to one
//! optional `adapter:` field, documenting that *"A leftover `models:` block in
//! an operator's file is ignored on parse"*. So the picker writes ONE file now
//! (§9.4) and this block survives for the one fact still read out of it:
//! `context_window`, the §5.1 #35 fullness denominator ([`context_windows`]).
//!
//! The reader is the same anchored grammar as [`roles`](super::roles), applied
//! to the other file.

use super::{BlockKey, GrammarError, MODELS, MODELS_YAML, block_key, entries, field, join};

/// The context window a hand-declared entry starts at (§9.2's Declare control).
/// Deliberately conservative, because under-stating a window degrades to early
/// compaction where over-stating it overflows the request.
///
/// **It is a declared default, and since bl-d9cb it is the only kind there is.**
/// Its one reader is §5.1 #35's fullness figure ([`context_windows`]), so a
/// wrong number shows up as a wrong percentage — which is why bl-848f had the
/// picker seed the entry from `Model.context_window` wherever brazen's roster
/// carried one, so a fabricated 200 000 could not sit beside a served number
/// looking identical. That seed is gone with the picker's write: the one seat
/// left that authors an entry is a hand-typed id with no roster behind it, so
/// every generated number is a declared default under a comment that says so,
/// and the indistinguishability bl-848f found cannot arise. Reading brazen's
/// served window is a *query*, not a field to seed (see [`context_windows`]).
pub const DEFAULT_CONTEXT_WINDOW: u32 = 200_000;

/// The comment yog writes above a generated entry, so its two fabricated
/// fields are never mistaken for facts anybody published, and so the operator
/// reading the file knows what the number is *for* (bl-d9cb — naming the one
/// consumer is what makes editing it an obvious move rather than a mystery).
const DECLARED_NOTE: &str = "  # added by yog's Declare control in the models.yaml editor.\n  \
     # nothing published these two lines — they are declared defaults. \
     context_window\n  \
     # is the denominator of yog's context-fullness figure; edit either here.";

/// The `models.yaml` entry for a hand-declared model, at two-space indent.
fn model_entry(model: &str, provider: &str) -> String {
    format!(
        "{DECLARED_NOTE}\n  {model}:\n    provider: {provider}\n    model_id: {model}\n    \
         capabilities: []\n    context_window: {DEFAULT_CONTEXT_WINDOW}"
    )
}

/// Declare `model` on `provider` under the global `models:` block.
/// `Ok(None)` means nothing to write — the id is already declared **on this
/// same row**, and the operator's own capabilities/context window stand
/// untouched.
///
/// An id already declared on a *different* row has only its one `provider:`
/// line rewritten (bl-bd89) — re-declaring an id is a statement about which row
/// it belongs to, and two entries for one id would leave the reader to guess.
/// Everything else in the entry is the operator's and is preserved.
///
/// The entry is inserted **directly after the `models:` line**, not at EOF, so
/// a file that carries a later top-level key (`adapter:` — the one field lernie
/// still reads out of this file) stays valid. A file with no `models:` key at
/// all gets one appended; an inline `models: {}` is refused rather than
/// transformed.
///
/// **Its one caller is the §9.2 Declare control** since bl-d9cb. The §9.4
/// picker used to call it first and `set_role_model` second; lernie reads no
/// `models:` table any more, so the pick is one write and this is a hand
/// gesture over yog's own table.
pub fn declare_model(
    models_yaml: &str,
    model: &str,
    provider: &str,
) -> Result<Option<String>, GrammarError> {
    let lines: Vec<&str> = models_yaml.lines().collect();
    let mut out: Vec<String> = lines.iter().map(|l| (*l).to_string()).collect();
    match block_key(&lines, MODELS) {
        BlockKey::Inline => Err(GrammarError::Inline {
            file: MODELS_YAML,
            key: MODELS.to_string(),
        }),
        BlockKey::Absent => {
            out.push(format!("{MODELS}:"));
            out.push(model_entry(model, provider));
            Ok(Some(join(&out)))
        }
        BlockKey::At(at) => {
            let declared = entries(&lines, at)
                .into_iter()
                .find(|(name, _)| name == model)
                .map(|(_, i)| i);
            let Some(entry) = declared else {
                out.insert(at + 1, model_entry(model, provider));
                return Ok(Some(join(&out)));
            };
            let (row, at_line) =
                field(&lines, entry, "provider").ok_or_else(|| GrammarError::NoField {
                    file: MODELS_YAML,
                    entry: model.to_string(),
                    field: "provider",
                })?;
            if row == provider {
                return Ok(None);
            }
            // The index came from this very line vector, so the assignment lands.
            if let Some(slot) = out.get_mut(at_line) {
                *slot = format!("    provider: {provider}");
            }
            Ok(Some(join(&out)))
        }
    }
}

/// One `models:` entry as the file declares it: the model id and the brazen
/// provider-row name it points at (§4.1 of lernie's own header — "`provider:`
/// on each model is a brazen provider-row NAME").
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeclaredModel {
    pub model: String,
    pub provider: String,
}

impl std::fmt::Display for DeclaredModel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} → {}", self.model, self.provider)
    }
}

/// Every model the file declares, in file order — the mirror of
/// [`roles`](super::roles) over the other file. An entry with no `provider:`
/// line is omitted: it names no row, so it can name no *wrong* row.
///
/// Its one consumer is [`unknown_rows`], the §9.2 Apply gate.
pub fn declared(models_yaml: &str) -> Vec<DeclaredModel> {
    let lines: Vec<&str> = models_yaml.lines().collect();
    let BlockKey::At(at) = block_key(&lines, MODELS) else {
        return Vec::new();
    };
    entries(&lines, at)
        .into_iter()
        .filter_map(|(model, i)| {
            Some(DeclaredModel {
                model,
                provider: field(&lines, i, "provider")?.0,
            })
        })
        .collect()
}

/// Every declared model's **wire id** paired with the `context_window` its
/// entry declares (§9.2's own field, §5.1 #35) — the denominator of the
/// context-fullness figure, and the one home for it.
///
/// Keyed on `model_id`, falling back to the entry key when the entry declares
/// none, because the id a step's `request.json` names is the wire id lernie
/// sent — not the alias the entry is filed under. An entry with no
/// `context_window:` line, an unparseable one, or a zero is **absent from the
/// map**: a window nobody declared is unknown, and a percentage against a
/// fabricated denominator is exactly the capability theater the figure exists
/// to avoid (brazen's Usage zero-vs-unknown principle, applied one field over).
///
/// **This is the fact's one home, and since bl-d9cb it is the ONLY reader of
/// the `models:` block that reads a number.** lernie reads no `models:` table at
/// all (see this module's header), so the block is yog's own hand-configuration:
/// authored by the §9.2 Declare control, edited by the §9.5 form, read here.
/// One home, one number, operator-correctable.
///
/// **brazen's served window is not a second home, and must not become a
/// seeded field.** brazen carries `Model.context_window` on `--list-models` for
/// the providers that serve one (Google) and `None` for the ones that do not
/// (Anthropic, OpenAI, Ollama) — its own empty-set rule: *"a harness
/// hand-configures only what no provider serves"*. That number is the
/// provider's fact and it changes without yog's involvement, so copying it into
/// a file at pick time is a snapshot that goes stale — the same reasoning §9.4
/// already applies to the model roster (*"a stored candidate list would be a
/// second representation of a fact the provider owns"*). If the figure should
/// ever prefer a served window over a declared one, the shape is a **query** at
/// read time over the model cache
/// ([`model_cache_at`](crate::config_edit::brazen::model_cache_at), already on
/// disk inside the workspace's wall), not a field this file carries.
pub fn context_windows(models_yaml: &str) -> std::collections::BTreeMap<String, u64> {
    let lines: Vec<&str> = models_yaml.lines().collect();
    let BlockKey::At(at) = block_key(&lines, MODELS) else {
        return std::collections::BTreeMap::new();
    };
    entries(&lines, at)
        .into_iter()
        .filter_map(|(model, i)| {
            let window: u64 = field(&lines, i, "context_window")?.0.parse().ok()?;
            let id = field(&lines, i, "model_id").map_or(model, |(id, _)| id);
            (window > 0).then_some((id, window))
        })
        .collect()
}

/// **The** provider-row judgement: does `provider` name no row in brazen's
/// effective table? Every site that asks it asks it here — the §9.2 Apply gate
/// over `models.yaml` ([`unknown_rows`]), the §9.4 pick gate
/// ([`plan`](crate::model_pick::plan)), the §9.4 role marks
/// ([`role_fault`](crate::model_pick::role_fault)), and the §9.5 pane's provider
/// control over both files ([`crate::config_edit::form`]) — so the four can
/// never disagree. Every one of them judges a file against the wall of the
/// workspace that holds it; the retired birth gate (bl-c3a9, retired bl-00ee) is
/// the one site that asked it where no wall existed yet.
///
/// `providers` is `bz --list-providers`' answer (built-ins included, which a
/// scan of `config.toml` would miss). An **empty** table is no answer rather
/// than an empty one — brazen could not be asked — so it judges nothing: no
/// surface may refuse on the strength of a question that went unanswered.
pub fn is_unknown_row(provider: &str, providers: &[String]) -> bool {
    !providers.is_empty() && !providers.iter().any(|p| p == provider)
}

/// The declared entries whose `provider:` names no row in brazen's effective
/// table — the §9.2 Apply gate's whole judgement, one [`is_unknown_row`] per
/// entry.
///
/// A file with no `models:` block declares nothing and is therefore always
/// clean, which is why the gate can run over every §9.2 file (a
/// `workflows/*.yaml` simply has nothing to check) instead of branching on
/// which file is open.
pub fn unknown_rows(models_yaml: &str, providers: &[String]) -> Vec<DeclaredModel> {
    declared(models_yaml)
        .into_iter()
        .filter(|d| is_unknown_row(&d.provider, providers))
        .collect()
}

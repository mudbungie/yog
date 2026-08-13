//! The `models.yaml` half of the §9.4 block grammar: declaring a model the
//! operator picked from a live `bz --list-models` roster, and **reading back**
//! what the file declares so a dead entry can be caught (§9.2 Apply gate,
//! §9.4 role rows).
//!
//! Its whole reason to exist is lernie's cross-check — a role naming a model
//! `models.yaml` does not declare is a hard load error, so the picker writes
//! this half FIRST and the assignment second (§9.4). The two facts brazen does
//! not publish are written as **declared defaults, not discoveries**, under a
//! comment that says so.
//!
//! The reader is the same anchored grammar as [`roles`](super::roles), applied
//! to the other file: `provider:` on a model entry is a brazen provider-row
//! NAME, so it is checkable against brazen's own effective table
//! ([`BzRunner::providers`](crate::config_edit::brazen::BzRunner::providers))
//! and, since bl-53be, is checked at both the §9.2 write and the §9.4 read.

use super::{BlockKey, GrammarError, MODELS, MODELS_YAML, block_key, entries, field, join};

/// The context window yog declares for a model discovered from `bz`
/// (§9.4). brazen publishes one only for the providers that serve it on their
/// list GET (Google today; Anthropic, OpenAI and Ollama serve none), so for the
/// rows the picker actually writes this is a **declared default, not a
/// discovery** — deliberately conservative, because under-stating a window
/// degrades to early compaction where over-stating it overflows the request.
///
/// **It is no longer unread.** Since bl-a48b the declared window is the
/// denominator of §5.1 #35's context-fullness figure
/// ([`context_windows`]), so a wrong default now shows up as a wrong
/// percentage — which is the operator's cue to correct the line the generated
/// note already tells them to edit. Seeding it from brazen's own
/// `Model.context_window` wherever the roster carries one is the honest
/// follow-up, and it belongs to the picker's write path, not to the reader.
pub const DEFAULT_CONTEXT_WINDOW: u32 = 200_000;

/// The comment yog writes above a generated entry, so the two declared-default
/// fields are never mistaken for facts brazen published (§9.4).
const GENERATED_NOTE: &str = "  # added by yog's model picker from `bz --list-models`.\n  \
     # brazen publishes no capabilities or context window, so the two lines\n  \
     # below are declared defaults, not discoveries — edit them here.";

/// The `models.yaml` entry for a model discovered live, at two-space indent.
fn model_entry(model: &str, provider: &str) -> String {
    format!(
        "{GENERATED_NOTE}\n  {model}:\n    provider: {provider}\n    model_id: {model}\n    \
         capabilities: []\n    context_window: {DEFAULT_CONTEXT_WINDOW}"
    )
}

/// Declare `model` on `provider` under the global `models:` block.
/// `Ok(None)` means nothing to write — the id is already declared **on this
/// same row**, and the operator's own capabilities/context window stand
/// untouched.
///
/// An id already declared on a *different* row has only its one `provider:`
/// line rewritten (bl-bd89). lernie refuses a config whose
/// `models.<m>.provider` differs from the `roles.<r>.provider` naming it, so
/// leaving a stale row here would brick the workspace just as surely as leaving
/// the model undeclared — the two are one fact, and the picker writes it once.
/// Everything else in the entry is the operator's and is preserved.
///
/// The entry is inserted **directly after the `models:` line**, not at EOF, so
/// a file that carries a later top-level key (`adapter:`) stays valid. A file
/// with no `models:` key at all gets one appended; an inline `models: {}` is
/// refused rather than transformed.
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
/// line is omitted: it names no row, so it can name no *wrong* row, and
/// lernie's own loader is the authority on a half-written entry.
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
/// **Why the declaration and not brazen's discovery.** brazen carries
/// `Model.context_window` on `--list-models` for the providers that serve one
/// (Google), and `None` for the ones that do not (Anthropic, OpenAI, Ollama) —
/// its own empty-set rule: *"a harness hand-configures only what no provider
/// serves"*. `models.yaml` **is** that hand-configuration: lernie's declared
/// field, written by the §9.4 picker, edited by the §9.5 form. One home, one
/// number, operator-correctable — where reading brazen's cache *as well* would
/// be two representations of one fact, drifting the moment either moves.
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
/// ([`plan`](crate::model_pick::plan)), and the §9.5 pane's provider control
/// over both files ([`crate::config_edit::form`]) — so the three can never
/// disagree. Every one of them judges a file against the wall of the workspace
/// that holds it; the retired birth gate (bl-c3a9, retired bl-00ee) is the one
/// site that asked it where no wall existed yet.
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

/// Why `model` cannot be fired as `models.yaml` stands, or `None` when it is
/// usable (§9.4 role rows). Two faults, because lernie refuses the config for
/// either: the id is not declared at all, or it is declared on a provider row
/// brazen's table does not have.
pub fn fault(models_yaml: &str, providers: &[String], model: &str) -> Option<String> {
    let Some(entry) = declared(models_yaml).into_iter().find(|d| d.model == model) else {
        return Some(format!(
            "{model} is not declared in {MODELS_YAML} — lernie refuses to load a \
             config whose role names an undeclared model"
        ));
    };
    if !is_unknown_row(&entry.provider, providers) {
        return None;
    }
    Some(format!(
        "{model} names provider row `{}`, which brazen's table does not have — \
         repoint it in the Config editors, or pick a live model here",
        entry.provider
    ))
}

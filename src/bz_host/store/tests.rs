use super::*;
use brazen::{CachedModels, Model, Secret};
use tempfile::TempDir;

fn env_at(home: &Path) -> Env {
    Env::from_pairs([("HOME", home.display().to_string())])
}

fn store(dir: &TempDir) -> WallCredStore {
    WallCredStore::new(dir.path().join("credentials"), env_at(dir.path()))
}

fn api_key(key: &str) -> Cred {
    Cred::ApiKey {
        key: Secret::new(key),
    }
}

#[test]
fn a_credential_round_trips_through_the_wall_at_0600() {
    let dir = TempDir::new().unwrap();
    let s = store(&dir);
    // A cold wall is a miss, not an error — the no-creds path.
    assert!(s.get("openai").is_none());
    s.put("openai", &api_key("sk-live")).unwrap();
    match s.get("openai") {
        Some(Cred::ApiKey { key }) => assert_eq!(key.expose(), "sk-live"),
        other => panic!("expected the stored key, got {other:?}"),
    }
    // The secret is owner-only from the moment it exists, and so is its dir.
    let creds = dir.path().join("credentials");
    let mode = |p: &Path| std::fs::metadata(p).unwrap().permissions().mode() & 0o777;
    assert_eq!(mode(&creds.join("openai.json")), 0o600);
    assert_eq!(mode(&creds), 0o700);
    // No temp survives the write (I3: temp-in-dir, then rename).
    let leftovers: Vec<_> = std::fs::read_dir(&creds)
        .unwrap()
        .filter_map(std::result::Result::ok)
        .filter(|e| e.file_name().to_string_lossy().starts_with('.'))
        .collect();
    assert!(leftovers.is_empty(), "a staging temp was left behind");
}

#[test]
fn one_wall_never_reads_anothers_sign_in() {
    let dir = TempDir::new().unwrap();
    let corp = WallCredStore::new(dir.path().join("corp"), env_at(dir.path()));
    let home = WallCredStore::new(dir.path().join("home"), env_at(dir.path()));
    corp.put("openai", &api_key("corp-key")).unwrap();
    assert!(corp.get("openai").is_some());
    assert!(home.get("openai").is_none());
}

#[test]
fn a_garbage_credential_file_reads_as_no_creds() {
    let dir = TempDir::new().unwrap();
    let s = store(&dir);
    std::fs::create_dir_all(dir.path().join("credentials")).unwrap();
    std::fs::write(dir.path().join("credentials/openai.json"), b"not json").unwrap();
    assert!(s.get("openai").is_none());
}

#[test]
fn a_put_into_an_unmakeable_dir_is_an_error_not_a_panic() {
    let dir = TempDir::new().unwrap();
    // A *file* where the credentials dir would go: `create_dir_all` fails.
    let wall = dir.path().join("wall");
    std::fs::write(&wall, b"").unwrap();
    let s = WallCredStore::new(wall.join("credentials"), env_at(dir.path()));
    assert!(s.put("openai", &api_key("k")).is_err());
}

#[test]
fn ambient_discovery_reads_the_snapshot_and_the_expanded_home() {
    let dir = TempDir::new().unwrap();
    let home = dir.path();
    let env = Env::from_pairs([
        ("HOME", home.display().to_string()),
        ("VENDOR_KEY", "sk-ambient".to_owned()),
    ]);
    let s = WallCredStore::new(home.join("credentials"), env);
    // An env-named key comes from the INJECTED snapshot, never the live env.
    let spec = AmbientSpec {
        format: AmbientFormat::ApiKeyEnv,
        path: "VENDOR_KEY".to_owned(),
    };
    match s.discover(&spec) {
        Some(Cred::ApiKey { key }) => assert_eq!(key.expose(), "sk-ambient"),
        other => panic!("expected the ambient key, got {other:?}"),
    }
    // A variable the snapshot does not carry is simply no creds.
    assert!(
        s.discover(&AmbientSpec {
            format: AmbientFormat::ApiKeyEnv,
            path: "ABSENT_KEY".to_owned(),
        })
        .is_none()
    );
    // A `~/` file path expands against the snapshot's HOME; absent is no creds.
    let claude = AmbientSpec {
        format: AmbientFormat::ClaudeCode,
        path: "~/.claude/creds.json".to_owned(),
    };
    assert!(s.discover(&claude).is_none());
    std::fs::create_dir_all(home.join(".claude")).unwrap();
    std::fs::write(home.join(".claude/creds.json"), br#"{"claudeAiOauth":{"accessToken":"a","refreshToken":"r","expiresAt":2000,"scopes":["s"]}}"#).unwrap();
    assert!(matches!(s.discover(&claude), Some(Cred::OAuth2 { .. })));
    // An absolute path passes through unexpanded.
    let abs = home.join(".claude/creds.json");
    assert!(matches!(
        s.discover(&AmbientSpec {
            format: AmbientFormat::ClaudeCode,
            path: abs.display().to_string(),
        }),
        Some(Cred::OAuth2 { .. })
    ));
}

#[test]
fn the_model_cache_round_trips_and_forgives_everything() {
    let dir = TempDir::new().unwrap();
    let cache = WallModelCache::new(dir.path().join("models"));
    assert!(cache.get("openai").is_none());
    let doc = CachedModels {
        models: vec![Model {
            id: "gpt-5".to_owned(),
            default: true,
            context_window: None,
            max_output_tokens: None,
            display_name: None,
        }],
        last_used: Some("gpt-5".to_owned()),
    };
    cache.put("openai", &doc);
    assert_eq!(cache.get("openai"), Some(doc));
    // Garbage is a cold cache, never an error.
    std::fs::write(dir.path().join("models/openai.json"), b"{{{").unwrap();
    assert!(cache.get("openai").is_none());
    // A cache that cannot be written is best-effort silence, not a failure.
    let blocked = dir.path().join("blocked");
    std::fs::write(&blocked, b"").unwrap();
    WallModelCache::new(blocked.join("models")).put("openai", &CachedModels::default());
}

#[test]
fn a_destination_with_no_file_name_still_stages_beside_itself() {
    // The total-function fallback in `write_atomic`: a degenerate destination
    // names its temp `.cred.yog-tmp-<pid>` rather than panicking on `None`.
    let dir = TempDir::new().unwrap();
    let dest = dir.path().join("..");
    assert!(write_atomic(&dest, b"x", 0o600).is_err());
}

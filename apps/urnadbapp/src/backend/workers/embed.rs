//! Offline text embedder bridge: spawn the urna query embedder as a
//! subprocess and parse the query vector back, with the same model gate
//! the urna CLI applies.
//!
//! Two scripts of the `urna` python package, routed like `urna ask`/
//! `retrieve`: potion corpora take `embed/potionqry.py` (static table, no
//! torch), any other manifest model and every multimodal SPACE (e.g. the
//! clip text tower for a "clip-vit-b32" band, with `--preset <space>`)
//! take the registry embedder `embed/presetqry.py`.
//!
//! This is a minimal port of `urna-cli`'s `cmd/embed_gate.rs` and
//! `cmd/pyenv.rs::resolve_interpreter`: interpreter resolution
//! (`URNA_PYTHON` → nearest `.venv` → `python3`), embedder script
//! discovery (repo layout → XDG data dir → exe share dir), the JSON
//! contract on stdout, and the blocking name/dim/model_hash gate (the CLI
//! bails on mismatch; so do we — never a wrong score).
//!
//! Everything here runs on the worker thread: spawning python blocks for
//! up to [`EMBED_TIMEOUT`].

use std::path::{Path, PathBuf};
use std::time::Duration;

/// Hard cap on one embedder run (the potion table load dominates; the CLI
/// has no timeout, but a wedged python must not wedge the worker forever).
const EMBED_TIMEOUT: Duration = Duration::from_secs(120);

/// Errors the embedder bridge reports to the UI (displayed verbatim).
#[derive(Debug, thiserror::Error)]
pub enum EmbedError {
    /// No embedder script found at any of the candidate locations.
    #[error(
        "offline embedder script not found (looked in the urna repo layout \
             and in <data root>/urna/python; `urna setup` installs it)"
    )]
    EmbedderNotFound,
    /// Spawning the interpreter failed (python3 absent, bad URNA_PYTHON…).
    #[error("failed to spawn embedder with {interpreter}: {source}")]
    Spawn {
        interpreter: String,
        source: std::io::Error,
    },
    /// The embedder ran past the timeout and was killed.
    #[error("embedder timed out after {}s", EMBED_TIMEOUT.as_secs())]
    Timeout,
    /// Non-zero exit: stderr carries the reason (missing table, no numpy…).
    #[error("embedder failed: {0}")]
    Failed(String),
    /// stdout was not the expected JSON document.
    #[error("invalid embedder output: {0}")]
    InvalidOutput(String),
    /// The embedder's identity does not match the corpus manifest (the
    /// CLI's layer 1/2/3 gate: name, dim, model_hash).
    #[error("model gate rejected the embedder: {0}")]
    Gate(String),
}

/// The embedder's JSON contract (shared by `potionqry.py` and
/// `presetqry.py`): the compact `model_hash` is the source of
/// truth for the manifest gate.
#[derive(serde::Deserialize)]
struct EmbedderOutput {
    model_hash: String,
    embedding_model: String,
    embedding_dim: usize,
    vector: Vec<f32>,
}

/// Placeholder hash legacy corpora carry; the CLI rejects it outright.
const PLACEHOLDER_MODEL_HASH: &str =
    "sha256:0000000000000000000000000000000000000000000000000000000000000000";

/// The python interpreter the embedder runs under: `URNA_PYTHON` wins,
/// then the venv `urna setup` manages (`<data root>/urna/venv`, which
/// carries the deps by construction), then the nearest `.venv/bin/python`
/// walking up to four ancestors of the current dir, then `python3` on
/// PATH. Mirrors `pyenv::resolve_interpreter`.
pub fn resolve_interpreter() -> String {
    resolve_interpreter_from(
        std::env::var("URNA_PYTHON").ok().or_else(setup_python),
        std::env::current_dir().ok(),
    )
}

/// The places urna lays its payload down, in the CLI's order
/// (`paths::data_roots`): `URNA_DATA_DIR`, `XDG_DATA_HOME`,
/// `~/.local/share`, `LOCALAPPDATA`, then `<exe>/../share`.
fn data_roots() -> Vec<PathBuf> {
    let env = |k: &str| {
        std::env::var_os(k)
            .filter(|v| !v.is_empty())
            .map(PathBuf::from)
    };
    let mut roots: Vec<PathBuf> = Vec::new();
    roots.extend(env("URNA_DATA_DIR"));
    roots.extend(env("XDG_DATA_HOME"));
    roots.extend(env("HOME").map(|h| h.join(".local").join("share")));
    roots.extend(env("LOCALAPPDATA"));
    roots.extend(
        std::env::current_exe()
            .ok()
            .and_then(|e| e.parent().map(|p| p.join("..").join("share"))),
    );
    roots.dedup();
    roots
}

/// The interpreter of the venv `urna setup` created, when one exists.
fn setup_python() -> Option<String> {
    data_roots()
        .into_iter()
        .map(|r| {
            let venv = r.join("urna").join("venv");
            if cfg!(windows) {
                venv.join("Scripts").join("python.exe")
            } else {
                venv.join("bin").join("python")
            }
        })
        .find(|p| p.is_file())
        .map(|p| p.to_string_lossy().into_owned())
}

/// Testable core of [`resolve_interpreter`].
fn resolve_interpreter_from(urna_python: Option<String>, start: Option<PathBuf>) -> String {
    if let Some(p) = urna_python {
        return p;
    }
    if let Some(mut dir) = start {
        for _ in 0..4 {
            let venv = dir.join(".venv").join("bin").join("python");
            if venv.exists() {
                return venv.to_string_lossy().into_owned();
            }
            if !dir.pop() {
                break;
            }
        }
    }
    "python3".into()
}

/// Script that can reproduce the corpus model, from the manifest's
/// `embedding_model`, routed like the CLI's `embed_and_search`: potion
/// corpora take the offline potion table (`embed/potionqry.py`), any other
/// model the registry embedder (`embed/presetqry.py`), which hands a model
/// no preset names to the sentence-transformers bridge itself.
fn embedder_script_rel(embedding_model: &str) -> PathBuf {
    let name = if embedding_model.starts_with("minishlab/potion") {
        "potionqry.py"
    } else {
        "presetqry.py"
    };
    PathBuf::from("embed").join(name)
}

/// The registry embedder: one script for every preset (clip, siglip2,
/// wemm, jina…), selected with `--preset`. Used for multimodal spaces.
fn space_embedder_script_rel() -> PathBuf {
    PathBuf::from("embed").join("presetqry.py")
}

/// Locate the embedder script for `embedding_model`.
pub fn default_embedder_path(embedding_model: &str) -> Option<PathBuf> {
    locate_script(&embedder_script_rel(embedding_model))
}

/// Locate the registry embedder used for multimodal spaces.
pub fn default_space_embedder_path() -> Option<PathBuf> {
    locate_script(&space_embedder_script_rel())
}

/// Resolve `rel`, a path inside the `urna` python package, with the CLI's
/// ladder (`embed_gate::installed_script_in`): the repo layout
/// (`rust/bridge/python/urna/<rel>`) from the cwd, its parent and the urna
/// checkout beside this repo, then `<root>/urna/python/urna/<rel>` in every
/// data root, where `urna setup` lays the payload down.
fn locate_script(rel: &Path) -> Option<PathBuf> {
    let package: PathBuf = ["rust", "bridge", "python", "urna"].iter().collect();
    let in_repo = package.join(rel);
    let cwd = std::env::current_dir().ok();
    let repos = [
        cwd.clone(),
        cwd.map(|p| p.join("..")),
        // Dev convenience: urnaui's workspace sits next to the urna
        // checkout in the standard hoff layout.
        Some(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../urna")),
    ];
    let installed = data_roots()
        .into_iter()
        .map(|r| r.join("urna").join("python").join("urna").join(rel));
    repos
        .into_iter()
        .flatten()
        .map(|r| r.join(&in_repo))
        .chain(installed)
        .find(|c| c.exists())
}

/// Run `program args…` with the embed timeout, returning stdout on
/// success; the shared subprocess helper drains the pipes so a long JSON
/// vector can never wedge the child.
fn run_with_timeout(program: &str, args: &[String]) -> Result<Vec<u8>, EmbedError> {
    use crate::backend::workers::subprocess::RunError;
    crate::backend::workers::subprocess::run(program, args, EMBED_TIMEOUT).map_err(|e| match e {
        RunError::Spawn(source) => EmbedError::Spawn {
            interpreter: program.to_string(),
            source,
        },
        RunError::Timeout => EmbedError::Timeout,
        RunError::Failed(stderr) => EmbedError::Failed(stderr),
    })
}

/// The CLI's blocking layer 1/2/3 gate: model name, dimension, and
/// `model_hash` must match the manifest; placeholder hashes are rejected.
/// Returns the vector on success.
fn gate(
    payload: EmbedderOutput,
    embedding_model: &str,
    embedding_dim: usize,
    model_hash: &str,
) -> Result<Vec<f32>, EmbedError> {
    if payload.embedding_model != embedding_model {
        return Err(EmbedError::Gate(format!(
            "model name mismatch: manifest={embedding_model}, embedder reports={}",
            payload.embedding_model
        )));
    }
    if payload.embedding_dim != embedding_dim || payload.vector.len() != embedding_dim {
        return Err(EmbedError::Gate(format!(
            "dim mismatch: manifest={embedding_dim}, embedder dim={}, vector len={}",
            payload.embedding_dim,
            payload.vector.len()
        )));
    }
    if model_hash == PLACEHOLDER_MODEL_HASH {
        return Err(EmbedError::Gate(
            "manifest carries the legacy placeholder model_hash; rebuild this corpus with a \
             real fingerprint"
                .to_string(),
        ));
    }
    if payload.model_hash != model_hash {
        return Err(EmbedError::Gate(format!(
            "model_hash mismatch: corpus was built with {model_hash}, embedder reports {}",
            payload.model_hash
        )));
    }
    Ok(payload.vector)
}

/// Embed `query` offline and validate the result against the corpus
/// identity. Returns the L2-normalized query vector. `truncated` is set
/// when the manifest records `full_dim` (the default space was built with
/// an MRL prefix): both embedders then cut the vector to `embedding_dim`.
pub fn embed_query(
    embedding_model: &str,
    embedding_dim: usize,
    model_hash: &str,
    truncated: bool,
    query: &str,
) -> Result<Vec<f32>, EmbedError> {
    let embedder = default_embedder_path(embedding_model).ok_or(EmbedError::EmbedderNotFound)?;
    let interpreter = resolve_interpreter();
    let mut args = vec![embedder.to_string_lossy().into_owned()];
    if truncated {
        args.push("--mrl-dim".to_string());
        args.push(embedding_dim.to_string());
    }
    args.push(embedding_model.to_string());
    args.push(query.to_string());
    let stdout = run_with_timeout(&interpreter, &args)?;
    let payload: EmbedderOutput =
        serde_json::from_slice(&stdout).map_err(|e| EmbedError::InvalidOutput(e.to_string()))?;
    gate(payload, embedding_model, embedding_dim, model_hash)
}

/// Embed `query` with the model of multimodal space `space` (a registry
/// preset name such as "clip-vit-b32") and validate dim + model_hash
/// against the space_table entry. The name gate does not apply: the
/// space is named after the preset, and the preset chooses the model.
/// Heavy presets that run remote code are opted into with the same env
/// vars the CLI honours (`URNA_ALLOW_REMOTE_CODE`, `URNA_ALLOW_HEAVY`).
pub fn embed_query_space(
    space: &str,
    dim: usize,
    model_hash: &str,
    query: &str,
) -> Result<Vec<f32>, EmbedError> {
    let embedder = default_space_embedder_path().ok_or(EmbedError::EmbedderNotFound)?;
    let interpreter = resolve_interpreter();
    let args = vec![
        embedder.to_string_lossy().into_owned(),
        "--preset".to_string(),
        space.to_string(),
        space.to_string(),
        query.to_string(),
    ];
    let stdout = run_with_timeout(&interpreter, &args)?;
    let payload: EmbedderOutput =
        serde_json::from_slice(&stdout).map_err(|e| EmbedError::InvalidOutput(e.to_string()))?;
    gate_space(payload, dim, model_hash)
}

/// The space gate: dim and `model_hash` against the space_table entry.
fn gate_space(
    payload: EmbedderOutput,
    dim: usize,
    model_hash: &str,
) -> Result<Vec<f32>, EmbedError> {
    if payload.embedding_dim != dim || payload.vector.len() != dim {
        return Err(EmbedError::Gate(format!(
            "dim mismatch: space={dim}, embedder dim={}, vector len={}",
            payload.embedding_dim,
            payload.vector.len()
        )));
    }
    if payload.model_hash != model_hash {
        return Err(EmbedError::Gate(format!(
            "model_hash mismatch: space was built with {model_hash}, embedder reports {}",
            payload.model_hash
        )));
    }
    Ok(payload.vector)
}

/// Cheap capability probe for the Open screen: interpreter resolves and
/// answers `--version`, and at least one embedder script exists. Returns a
/// short human-readable status (e.g. "Python 3.13.1 · embedder: …").
pub fn check_embedder() -> Result<String, String> {
    let interpreter = resolve_interpreter();
    let out = run_with_timeout(&interpreter, &["--version".to_string()])
        .map_err(|e| format!("{interpreter}: {e}"))?;
    let version = String::from_utf8_lossy(&out).trim().to_string();
    let embedder = default_embedder_path("potion")
        .or_else(|| default_embedder_path(""))
        .ok_or("embedder script not found".to_string())?;
    Ok(format!("{version} · embedder: {}", embedder.display()))
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn urna_python_env_wins() {
        let got = resolve_interpreter_from(Some("/opt/py/bin/python".into()), None);
        assert_eq!(got, "/opt/py/bin/python");
    }

    #[test]
    fn falls_back_to_python3_without_a_venv() {
        let start = Some(PathBuf::from("/urnaui-no-venv-here/a/b/c"));
        assert_eq!(resolve_interpreter_from(None, start), "python3");
    }

    #[test]
    fn discovers_a_venv_in_an_ancestor() {
        let base = std::env::temp_dir().join(format!("urnaui_interp_{}", std::process::id()));
        let venv_python = base.join(".venv").join("bin").join("python");
        std::fs::create_dir_all(venv_python.parent().unwrap()).unwrap();
        std::fs::write(&venv_python, b"").unwrap();
        let start = base.join("nested").join("deeper");
        std::fs::create_dir_all(&start).unwrap();

        let got = resolve_interpreter_from(None, Some(start));
        assert_eq!(got, venv_python.to_string_lossy());

        let _ = std::fs::remove_dir_all(&base);
    }

    fn output(model: &str, dim: usize, hash: &str) -> EmbedderOutput {
        EmbedderOutput {
            model_hash: hash.to_string(),
            embedding_model: model.to_string(),
            embedding_dim: dim,
            vector: vec![0.0; dim],
        }
    }

    #[test]
    fn dispatches_the_script_by_manifest_model() {
        assert!(embedder_script_rel("minishlab/potion-base-8M/v1").ends_with("embed/potionqry.py"));
        assert!(embedder_script_rel("all-MiniLM-L6-v2").ends_with("embed/presetqry.py"));
        assert!(space_embedder_script_rel().ends_with("embed/presetqry.py"));
    }

    #[test]
    fn gate_accepts_a_matching_embedder() {
        let v = gate(output("potion", 4, "sha256:abc"), "potion", 4, "sha256:abc").unwrap();
        assert_eq!(v.len(), 4);
    }

    #[test]
    fn gate_blocks_name_dim_and_hash_mismatches() {
        assert!(gate(output("other", 4, "sha256:abc"), "potion", 4, "sha256:abc").is_err());
        assert!(gate(output("potion", 8, "sha256:abc"), "potion", 4, "sha256:abc").is_err());
        assert!(gate(output("potion", 4, "sha256:xyz"), "potion", 4, "sha256:abc").is_err());
    }

    #[test]
    fn space_gate_checks_dim_and_hash_only() {
        // the name is the preset's, never the manifest's: not gated.
        let v = gate_space(
            output("open_clip/ViT-B-32", 4, "sha256:abc"),
            4,
            "sha256:abc",
        )
        .unwrap();
        assert_eq!(v.len(), 4);
        assert!(gate_space(output("x", 8, "sha256:abc"), 4, "sha256:abc").is_err());
        assert!(gate_space(output("x", 4, "sha256:xyz"), 4, "sha256:abc").is_err());
    }

    #[test]
    fn gate_blocks_the_placeholder_model_hash() {
        let err = gate(
            output("potion", 4, PLACEHOLDER_MODEL_HASH),
            "potion",
            4,
            PLACEHOLDER_MODEL_HASH,
        )
        .unwrap_err();
        assert!(err.to_string().contains("placeholder"));
    }
}

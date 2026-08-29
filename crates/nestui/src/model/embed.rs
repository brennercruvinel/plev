//! Offline text embedder bridge: spawn the nest potion embedder
//! (`python/forge/embed_query_potion.py`) as a subprocess and parse the
//! query vector back, with the same model gate the nest CLI applies.
//!
//! This is a minimal port of `nest-cli`'s `cmd/util.rs::embed_and_search`
//! and `cmd/pyenv.rs::resolve_interpreter`: interpreter resolution
//! (`NEST_PYTHON` → nearest `.venv` → `python3`), embedder script
//! discovery (repo layout → XDG data dir → exe share dir), the JSON
//! contract on stdout, and the blocking name/dim/model_hash gate (the CLI
//! bails on mismatch; so do we — never a wrong score).
//!
//! Everything here runs on the worker thread: spawning python blocks for
//! up to [`EMBED_TIMEOUT`].

use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

/// Hard cap on one embedder run (the potion table load dominates; the CLI
/// has no timeout, but a wedged python must not wedge the worker forever).
const EMBED_TIMEOUT: Duration = Duration::from_secs(120);

/// Poll interval while waiting for the embedder to exit.
const POLL: Duration = Duration::from_millis(50);

/// Errors the embedder bridge reports to the UI (displayed verbatim).
#[derive(Debug, thiserror::Error)]
pub enum EmbedError {
    /// No embedder script found at any of the candidate locations.
    #[error(
        "offline embedder script not found (looked in the repo layout, \
             $XDG_DATA_HOME/nest/forge and next to the executable)"
    )]
    EmbedderNotFound,
    /// Spawning the interpreter failed (python3 absent, bad NEST_PYTHON…).
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

/// The embedder's JSON contract (shared by `embed_query.py` and
/// `embed_query_potion.py`): the compact `model_hash` is the source of
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

/// The python interpreter the embedder runs under: `NEST_PYTHON` wins,
/// then the nearest `.venv/bin/python` walking up to four ancestors of the
/// current dir, then `python3` on PATH. Mirrors `pyenv::resolve_interpreter`.
pub fn resolve_interpreter() -> String {
    resolve_interpreter_from(
        std::env::var("NEST_PYTHON").ok(),
        std::env::current_dir().ok(),
    )
}

/// Testable core of [`resolve_interpreter`].
fn resolve_interpreter_from(nest_python: Option<String>, start: Option<PathBuf>) -> String {
    if let Some(p) = nest_python {
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

/// Locate the offline potion embedder script. Resolution order mirrors the
/// CLI (`default_potion_embedder_path`): the nest repo layout relative to
/// the cwd, the nest dev checkout sibling of this crate, the XDG data dir,
/// and `<exe>/../share/nest/forge/`.
pub fn default_embedder_path() -> Option<PathBuf> {
    let rel = PathBuf::from("python")
        .join("forge")
        .join("embed_query_potion.py");
    let data_home = std::env::var("XDG_DATA_HOME")
        .ok()
        .map(PathBuf::from)
        .or_else(|| {
            std::env::var("HOME")
                .ok()
                .map(|h| PathBuf::from(h).join(".local").join("share"))
        });
    let exe_share = std::env::current_exe()
        .ok()
        .and_then(|e| e.parent().map(|p| p.to_path_buf()))
        .map(|bin| bin.join("..").join("share"));
    let candidates = [
        std::env::current_dir().ok().map(|p| p.join(&rel)),
        std::env::current_dir()
            .ok()
            .map(|p| p.join("..").join(&rel)),
        // Dev convenience: nestui's workspace sits next to the nest
        // checkout in the standard hoff layout.
        Some(
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../../nest")
                .join(&rel),
        ),
        data_home.map(|d| d.join("nest").join("forge").join("embed_query_potion.py")),
        exe_share.map(|s| s.join("nest").join("forge").join("embed_query_potion.py")),
    ];
    candidates.into_iter().flatten().find(|c| c.exists())
}

/// Run `program args…` with a timeout, returning stdout on success.
fn run_with_timeout(program: &str, args: &[String]) -> Result<Vec<u8>, EmbedError> {
    let mut child = Command::new(program)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|source| EmbedError::Spawn {
            interpreter: program.to_string(),
            source,
        })?;
    let deadline = Instant::now() + EMBED_TIMEOUT;
    loop {
        if let Some(status) = child.try_wait().map_err(|source| EmbedError::Spawn {
            interpreter: program.to_string(),
            source,
        })? {
            let out = child
                .wait_with_output()
                .map_err(|source| EmbedError::Spawn {
                    interpreter: program.to_string(),
                    source,
                })?;
            if !status.success() {
                return Err(EmbedError::Failed(
                    String::from_utf8_lossy(&out.stderr).trim().to_string(),
                ));
            }
            return Ok(out.stdout);
        }
        if Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            return Err(EmbedError::Timeout);
        }
        std::thread::sleep(POLL);
    }
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
/// identity. Returns the L2-normalized query vector.
pub fn embed_query(
    embedding_model: &str,
    embedding_dim: usize,
    model_hash: &str,
    query: &str,
) -> Result<Vec<f32>, EmbedError> {
    let embedder = default_embedder_path().ok_or(EmbedError::EmbedderNotFound)?;
    let interpreter = resolve_interpreter();
    let args = vec![
        embedder.to_string_lossy().into_owned(),
        embedding_model.to_string(),
        query.to_string(),
    ];
    let stdout = run_with_timeout(&interpreter, &args)?;
    let payload: EmbedderOutput =
        serde_json::from_slice(&stdout).map_err(|e| EmbedError::InvalidOutput(e.to_string()))?;
    gate(payload, embedding_model, embedding_dim, model_hash)
}

/// Cheap capability probe for the Open screen: interpreter resolves and
/// answers `--version`, and the embedder script exists. Returns a short
/// human-readable status (e.g. "Python 3.13.1 · potion embedder found").
pub fn check_embedder() -> Result<String, String> {
    let interpreter = resolve_interpreter();
    let out = run_with_timeout(&interpreter, &["--version".to_string()])
        .map_err(|e| format!("{interpreter}: {e}"))?;
    let version = String::from_utf8_lossy(&out).trim().to_string();
    let embedder = default_embedder_path().ok_or("potion embedder script not found".to_string())?;
    Ok(format!("{version} · embedder: {}", embedder.display()))
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nest_python_env_wins() {
        let got = resolve_interpreter_from(Some("/opt/py/bin/python".into()), None);
        assert_eq!(got, "/opt/py/bin/python");
    }

    #[test]
    fn falls_back_to_python3_without_a_venv() {
        let start = Some(PathBuf::from("/nestui-no-venv-here/a/b/c"));
        assert_eq!(resolve_interpreter_from(None, start), "python3");
    }

    #[test]
    fn discovers_a_venv_in_an_ancestor() {
        let base = std::env::temp_dir().join(format!("nestui_interp_{}", std::process::id()));
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

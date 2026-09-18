//! Frame preview for media corpora: decode one frame of an inlined codec
//! stream (the forge writes every image item as one frame of an av1 /
//! avif stream inlined in 0x17) into PNG bytes for the image atlas.
//!
//! The decode shells out to `ffmpeg` and reads the blob IN PLACE through
//! its `subfile` protocol (a byte range of the .urna itself), so a
//! multi-GB media section is never copied out. The forge encodes stills
//! at 1 fps, all-intra (`keyint = 1`), so frame `n` sits at `n` seconds
//! and `-ss n` lands on it exactly (every frame is a keyframe).
//!
//! Everything here runs on the worker thread.

use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use super::backend::BlobRange;

/// Hard cap on one decode: a wedged ffmpeg must not wedge the worker.
const DECODE_TIMEOUT: Duration = Duration::from_secs(20);
const POLL: Duration = Duration::from_millis(20);

/// Errors the frame decoder reports to the UI (displayed verbatim).
#[derive(Debug, thiserror::Error)]
pub enum FrameError {
    #[error("ffmpeg not found on PATH (frame previews need ffmpeg with an av1 decoder)")]
    FfmpegMissing,
    #[error("ffmpeg failed: {0}")]
    Failed(String),
    #[error("ffmpeg timed out after {}s", DECODE_TIMEOUT.as_secs())]
    Timeout,
    #[error("ffmpeg produced no image")]
    Empty,
}

/// Whether `ffmpeg` answers on PATH (probed once per open for the media
/// status line).
pub fn ffmpeg_available() -> bool {
    Command::new("ffmpeg")
        .arg("-version")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|s| s.success())
}

/// The `subfile` input url for a blob's byte range inside `path`.
fn subfile_url(path: &str, range: BlobRange) -> String {
    format!(
        "subfile,,start,{},end,{},,:{}",
        range.abs_start,
        range.abs_start + range.len,
        path
    )
}

/// Decode frame `frame` of the stream at `range` in `path` to a PNG no
/// larger than `max_side` on its longer edge.
pub fn decode_frame(
    path: &str,
    range: BlobRange,
    frame: u64,
    max_side: u32,
) -> Result<Vec<u8>, FrameError> {
    let args = [
        "-v".to_string(),
        "error".to_string(),
        "-nostdin".to_string(),
        "-ss".to_string(),
        frame.to_string(),
        "-i".to_string(),
        subfile_url(path, range),
        "-frames:v".to_string(),
        "1".to_string(),
        "-vf".to_string(),
        // Fit inside max_side x max_side, keep aspect, even dims for the
        // encoder.
        format!(
            "scale='min({max_side},iw)':'min({max_side},ih)':force_original_aspect_ratio=decrease"
        ),
        "-f".to_string(),
        "image2".to_string(),
        "-c:v".to_string(),
        "png".to_string(),
        "pipe:1".to_string(),
    ];
    let mut child = Command::new("ffmpeg")
        .args(&args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                FrameError::FfmpegMissing
            } else {
                FrameError::Failed(e.to_string())
            }
        })?;
    let deadline = Instant::now() + DECODE_TIMEOUT;
    loop {
        if let Some(status) = child
            .try_wait()
            .map_err(|e| FrameError::Failed(e.to_string()))?
        {
            let out = child
                .wait_with_output()
                .map_err(|e| FrameError::Failed(e.to_string()))?;
            if !status.success() {
                return Err(FrameError::Failed(
                    String::from_utf8_lossy(&out.stderr).trim().to_string(),
                ));
            }
            if out.stdout.is_empty() {
                return Err(FrameError::Empty);
            }
            return Ok(out.stdout);
        }
        if Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            return Err(FrameError::Timeout);
        }
        std::thread::sleep(POLL);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn subfile_url_carries_the_absolute_range() {
        let url = subfile_url(
            "/data/cards.urna",
            BlobRange {
                abs_start: 100,
                len: 50,
            },
        );
        assert_eq!(url, "subfile,,start,100,end,150,,:/data/cards.urna");
    }

    #[test]
    fn decoding_a_bogus_range_fails_typed() {
        if !ffmpeg_available() {
            eprintln!("skipping: ffmpeg not on PATH");
            return;
        }
        let dir = std::env::temp_dir().join(format!("urnaui_frames_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("junk.bin");
        std::fs::write(&path, vec![0u8; 4096]).unwrap();
        let err = decode_frame(
            path.to_str().unwrap(),
            BlobRange {
                abs_start: 0,
                len: 4096,
            },
            0,
            64,
        )
        .unwrap_err();
        assert!(matches!(err, FrameError::Failed(_) | FrameError::Empty));
        let _ = std::fs::remove_dir_all(&dir);
    }
}

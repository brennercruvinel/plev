//! the stderr of `ffmpeg -i <video> -af silencedetect=... -f null -` read
//! into segments. ffmpeg logs the filter at `info`, on stderr, one key
//! per line after a tag whose address changes every run
//! (`[Parsed_silencedetect_0 @ 0x71903dd40] silence_start: 3.018594`), so
//! the keys are looked for anywhere in a line and the rest of the text is
//! ignored. the input's `Duration: hh:mm:ss.cc` line, from the same
//! stderr, gives the length without a second program.

use super::Segment;

/// what the stderr said: the input's length when ffmpeg printed one, and
/// the silences in the order they were found.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Detected {
    pub duration: Option<f64>,
    pub segments: Vec<Segment>,
}

/// reads `stderr`. a `silence_start` without a `silence_end` (older
/// ffmpeg at the end of a stream that stays silent; ffmpeg 9 prints the
/// end itself) closes at the duration when one was printed and is dropped
/// otherwise; an end without a start, a value that is not a number and
/// every other line are ignored.
pub fn parse(stderr: &str) -> Detected {
    let mut out = Detected::default();
    let mut open: Option<f64> = None;
    for line in stderr.lines() {
        if let Some(rest) = after(line, "Duration:") {
            out.duration = duration(rest);
        } else if let Some(start) = after(line, "silence_start:").and_then(number) {
            open = Some(start);
        } else if let Some(end) = after(line, "silence_end:").and_then(number)
            && let Some(start) = open.take()
        {
            out.segments.push(Segment { start, end });
        }
    }
    if let (Some(start), Some(duration)) = (open, out.duration)
        && duration > start
    {
        out.segments.push(Segment {
            start,
            end: duration,
        });
    }
    out
}

/// the text after the first `key` in `line`, trimmed.
fn after<'a>(line: &'a str, key: &str) -> Option<&'a str> {
    let at = line.find(key)?;
    Some(line[at + key.len()..].trim_start())
}

/// the leading number of `text` (up to a space or a `|`), finite.
fn number(text: &str) -> Option<f64> {
    let end = text
        .find(|c: char| c.is_whitespace() || c == '|')
        .unwrap_or(text.len());
    text[..end].parse().ok().filter(|n: &f64| n.is_finite())
}

/// `hh:mm:ss.cc` up to the comma, in seconds; `N/A` and anything else
/// give none.
fn duration(text: &str) -> Option<f64> {
    let field = text.split(',').next()?.trim();
    let mut parts = field.split(':');
    let h: f64 = parts.next()?.parse().ok()?;
    let m: f64 = parts.next()?.parse().ok()?;
    let s: f64 = parts.next()?.parse().ok()?;
    if parts.next().is_some() || !(h >= 0.0 && m >= 0.0 && s >= 0.0) {
        return None;
    }
    let secs = h * 3600.0 + m * 60.0 + s;
    secs.is_finite().then_some(secs)
}

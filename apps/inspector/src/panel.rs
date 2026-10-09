//! the panel: what the model says about one control at one instant. it
//! calls the model and shows its text; nothing here reads the session on
//! its own or works an answer out again.

use accord::{Millis, SemId};
use engine::drawing::text::{TextMeasurer, TextStyle};
use engine::widgets::content::{CodeLine, LineKind};
use record::ReadError;
use replay::{Control, Inspector, Why, prose};

/// one control at one instant, and what came after it.
#[derive(Clone, Debug, PartialEq)]
pub struct Panel {
    /// the control at the instant (`Inspector::control`): its node as the
    /// recorded state held it then, and the requests that touched it up to
    /// then, each as the window up to then tells it.
    pub at: Control,
    /// the requests that touched it after the instant, and those still
    /// open at it, as the whole session tells them: the same answer at the
    /// end of the session, less what the instant already told whole.
    pub later: Vec<Why>,
}

impl Panel {
    /// the control `id` at `t`, on the recorder's clock.
    pub fn of(inspector: &Inspector<'_>, id: &SemId, t: Millis) -> Result<Panel, ReadError> {
        let at = inspector.control(id, t)?;
        let whole = inspector.control(id, inspector.end())?;
        let later = whole
            .receipts
            .into_iter()
            .filter(|w| !at.receipts.contains(w))
            .collect();
        Ok(Panel { at, later })
    }

    pub fn id(&self) -> &SemId {
        &self.at.id
    }

    /// the panel as rows: the control at the instant as the model writes
    /// it (`text::control`), then each later request's causal chain
    /// (`text::why`) under one line that says what they are. the line
    /// that opens a control or a request is a header.
    pub fn rows(&self) -> Vec<CodeLine> {
        let mut rows: Vec<CodeLine> = prose::control(&self.at).lines().map(row).collect();
        if !self.later.is_empty() {
            rows.push(CodeLine::new("", LineKind::Context));
            rows.push(CodeLine::new(
                format!(
                    "after {}, as the whole session tells it: {} more",
                    prose::stamp(self.at.at),
                    self.later.len()
                ),
                LineKind::Header,
            ));
            for why in &self.later {
                rows.push(CodeLine::new("", LineKind::Context));
                rows.extend(prose::why(why).lines().map(|l| row(&format!("  {l}"))));
            }
        }
        rows
    }
}

/// one line of the model's text, a header when it opens a control or a
/// request.
fn row(line: &str) -> CodeLine {
    let head = line.starts_with("control ") || line.starts_with("  request ");
    let kind = if head {
        LineKind::Header
    } else {
        LineKind::Context
    };
    CodeLine::new(line, kind)
}

/// the rows as they fit `width` with `style`: a row wider than that goes
/// on in the rows under it, at the column its value starts (after the
/// label the model aligns it to), or two spaces in when that column
/// leaves less than half the width; a word wider than a whole row is cut
/// where it fits. no character of a row is lost, and the order holds.
pub fn wrap(rows: &[CodeLine], width: f32, style: &TextStyle) -> Vec<CodeLine> {
    let fits = |text: &str| TextMeasurer::measure_styled(text, style, None).0 <= width;
    let mut out = Vec::new();
    for row in rows {
        if fits(&row.text) {
            out.push(row.clone());
            continue;
        }
        let lead = row.text.len() - row.text.trim_start().len();
        let hang = value_column(&row.text)
            .filter(|&c| fits(&" ".repeat(c * 2)))
            .unwrap_or(lead + 2);
        let hang = " ".repeat(hang);
        let mut push = |text: String| out.push(CodeLine::new(text, row.kind));
        let mut line = row.text[..lead].to_owned();
        let mut fresh = true;
        for word in row.text[lead..].split(' ') {
            if fresh && word.is_empty() && line.len() > lead {
                continue;
            }
            let joined = if fresh {
                format!("{line}{word}")
            } else {
                format!("{line} {word}")
            };
            if fits(&joined) {
                line = joined;
                fresh = false;
                continue;
            }
            if !fresh {
                push(std::mem::replace(&mut line, hang.clone()));
            }
            let mut rest = word;
            while !rest.is_empty() && !fits(&format!("{line}{rest}")) {
                let cut = rest
                    .char_indices()
                    .map(|(i, c)| i + c.len_utf8())
                    .take_while(|&end| fits(&format!("{line}{}", &rest[..end])))
                    .last()
                    .unwrap_or_else(|| rest.chars().next().map_or(rest.len(), char::len_utf8));
                push(format!("{line}{}", &rest[..cut]));
                line = hang.clone();
                rest = &rest[cut..];
            }
            line.push_str(rest);
            fresh = false;
        }
        push(line);
    }
    out
}

/// where the value of a `label   value` row starts: after its first word
/// and the run of two or more spaces that follows it.
fn value_column(text: &str) -> Option<usize> {
    let lead = text.len() - text.trim_start().len();
    let label = text[lead..].find(' ')? + lead;
    let gap = text[label..].len() - text[label..].trim_start().len();
    (gap >= 2 && label + gap < text.len()).then_some(label + gap)
}

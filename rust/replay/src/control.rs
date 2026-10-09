//! `control`: one control at an instant: its node as the recorded state
//! held it, and every request that touched it up to then, each with its
//! causal chain.

use record::ReadError;
use accord::{Millis, Node, SemId};

use crate::index::{Inspector, Stamp};
use crate::why::Why;

/// a control at an instant.
#[derive(Clone, Debug, PartialEq, serde::Serialize)]
pub struct Control {
    pub id: SemId,
    pub t: Millis,
    pub at: Millis,
    /// the node the recorded state holds at that instant, with the source
    /// the app projected it from; `None` when the state does not hold it
    /// then (or holds no state yet).
    pub node: Option<Node>,
    pub focused: bool,
    /// the git commit of the build that recorded the session: the source
    /// of the node is a line of that commit, not of the tree as it is now.
    pub build: String,
    /// the requests that touched it up to that instant (their target, or a
    /// state they read), in the order they were asked, each as the window
    /// up to that instant tells it.
    pub receipts: Vec<Why>,
}

impl Inspector<'_> {
    /// the control `id` at `t` (on the recorder's clock).
    pub fn control(&self, id: &SemId, t: Millis) -> Result<Control, ReadError> {
        let state = self.session().state_at(t)?;
        let node = state.as_ref().and_then(|s| s.get(id)).cloned();
        let focused = state.as_ref().and_then(|s| s.focused()) == Some(id);
        let limit = self.session().until(t).len();
        let mut receipts = Vec::new();
        for touched in self.session().receipts_for(id, t) {
            if let Some(why) = self.why_upto(touched.origin, touched.receipt.id, limit)? {
                receipts.push(why);
            }
        }
        let Stamp { t, at } = self.stamp(t);
        Ok(Control {
            id: id.clone(),
            t,
            at,
            node,
            focused,
            build: self.session().build().git.clone(),
            receipts,
        })
    }
}

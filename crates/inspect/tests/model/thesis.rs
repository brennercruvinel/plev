//! the thesis session (generative-runtime-thesis.md, sections 8 and 11):
//! the agent observes A, B and C staged, a person unstages B, the agent's
//! commit comes back stale, the agent observes again and its commit
//! completes with a sha.

use rec::Session;
use sem::RequestId;

use crate::apps::{Lab, agent};

/// the session, and the ids of the person's unstage, the stale submit
/// and the completed one.
pub struct Thesis {
    pub session: Session,
    pub unstage: RequestId,
    pub stale: RequestId,
    pub done: RequestId,
}

pub fn record() -> Thesis {
    let mut lab = Lab::new(&["A", "B", "C"], "fix login", 1_000);
    let seen = lab.observe(1_000, agent());
    let unstage = lab.person_unstages(1_100, "B");
    let stale = lab.act(1_300, &seen, agent(), "commit.submit", "submit", None);
    let again = lab.observe(1_400, agent());
    let done = lab.act(1_500, &again, agent(), "commit.submit", "submit", None);
    Thesis {
        session: lab.session(),
        unstage,
        stale,
        done,
    }
}

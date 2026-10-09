//! the person's preferences of accessibility over every surface of the
//! host (task-70 phase 4): the last layer. each surface keeps the form it
//! was asked for, with the changes asked (`Adapting::asked`), and is
//! drawn with the operations of the preferences made over it again
//! (`Adapting::now`), in the name of whoever turned them on, after every
//! change, at every new form, and when the preferences change. only the
//! person turns them on: an agent is refused.

use accord::{Actor, Snapshot};
use plevir::{Adapted, Capabilities, Context, GenerationPolicy, Preferences, VerifiedIr};

use super::{Adapting, Change, SurfaceError, SurfaceHost};

impl Adapting {
    /// a surface as it takes a form: asked and drawn alike, no change.
    pub(super) fn fresh(form: Adapted) -> Adapting {
        Adapting {
            now: form.clone(),
            asked: form,
            changes: Vec::new(),
            preferred: 0,
            asks: Vec::new(),
        }
    }

    /// how many of the changes were asked (the rest are the preferences').
    pub(super) fn asked_len(&self) -> usize {
        self.changes.len() - self.preferred
    }

    /// the preferences made again over the surface as it was asked, by
    /// `by`, under `cx`: the drawn surface and the preferences' changes
    /// replaced. an operation a preference cannot make there (the top of
    /// a ladder, nothing to move) is left out.
    fn prefer(&mut self, preferences: Preferences, by: &Actor, cx: Context<'_>) {
        let asked = self.asked_len();
        self.changes.truncate(asked);
        let made = preferences.over(&self.asked, by, cx);
        self.preferred = made.applied.len();
        self.changes
            .extend(made.applied.into_iter().map(|applied| Change {
                op: applied.op,
                inverse: applied.inverse,
                by: by.clone(),
                preference: Some(applied.preference),
            }));
        self.now = made.adapted;
    }
}

impl SurfaceHost {
    /// the person's preferences of accessibility, turned on by `by`, for
    /// every surface open and every one that opens: each open surface
    /// takes the form its asked surface makes with them, at once,
    /// reconciled by `SemId` as a change is; a preference turned off
    /// leaves what others asked as it is. refused for an agent: they are
    /// the person's (a person, assistive technology, a test).
    pub fn prefer(
        &mut self,
        preferences: Preferences,
        by: Actor,
        app: &Snapshot,
    ) -> Result<(), SurfaceError> {
        if matches!(by, Actor::Agent { .. }) {
            return Err(SurfaceError::NotThePerson(by));
        }
        self.preferences = preferences;
        self.person = Some(by);
        let caps = Capabilities::from_snapshot(app);
        for at in 0..self.surfaces.len() {
            let id = self.surfaces[at].id().clone();
            let policy = self.policies.get(&id).cloned().unwrap_or_default();
            let cx = Context {
                caps: &caps,
                policy: &policy,
            };
            let next = self.adapting(at).clone();
            self.settle(at, next, cx, app)?;
        }
        Ok(())
    }

    /// the person's preferences of accessibility now: none until `prefer`.
    pub fn preferences(&self) -> Preferences {
        self.preferences
    }

    /// `ir`, a form to take over `app` under `policy`, with the
    /// preferences over it: what the surface draws from its first frame.
    pub(super) fn preferred(
        &self,
        ir: VerifiedIr,
        app: &Snapshot,
        policy: &GenerationPolicy,
    ) -> Adapting {
        let mut out = Adapting::fresh(Adapted::new(ir));
        if !self.preferences.is_empty() {
            let caps = Capabilities::from_snapshot(app);
            let cx = Context {
                caps: &caps,
                policy,
            };
            out.prefer(self.preferences, &self.by(), cx);
        }
        out
    }

    /// the surface at `at` takes the form `next` draws, the preferences
    /// made again over its asked surface, and keeps `next`.
    pub(super) fn settle(
        &mut self,
        at: usize,
        mut next: Adapting,
        cx: Context<'_>,
        app: &Snapshot,
    ) -> Result<(), SurfaceError> {
        next.prefer(self.preferences, &self.by(), cx);
        self.take_form(at, next.now.verified().clone(), app)?;
        let id = self.surfaces[at].id().clone();
        self.adapting.insert(id, next);
        Ok(())
    }

    /// who the preferences' operations are made by: whoever turned them
    /// on (a person, before anyone did: nothing is on then).
    fn by(&self) -> Actor {
        self.person.clone().unwrap_or(Actor::Human)
    }
}

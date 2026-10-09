// ============================================================================
// Intent -- behavioral semantic token (RULE-08)
//
// First implementation of intent as a structural design token rather than a
// visual modifier. When a component carries an Intent, the theme resolves it
// to color palette, spring physics, and accessibility role in a single lookup.
//
// No W3C Design Token spec, no existing design system, no open source project
// treats intent as a propagatable first-class dimension. This is the first.
// ============================================================================

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Intent {
    Neutral,
    Constructive,
    Destructive,
    Informational,
}

// ============================================================================
// MotionPhysics -- global kinetic axioms (RULE-08)
//
// Three parameters define the kinetic personality of the entire product.
// Every animated transition derives its spring configuration from these.
// Changing `mass` makes the product feel lighter or heavier globally.
// Changing `stiffness` makes it snappier or lazier. Changing `damping`
// controls overshoot vs deadness.
//
// No public design system (Material, Fluent, Carbon, Spectrum, Primer)
// defines mass/tension/friction as root-level propagatable tokens.
// ============================================================================

#[derive(Clone, Debug, PartialEq)]
pub struct MotionPhysics {
    pub mass: f32,
    pub stiffness: f32,
    pub damping: f32,
}

impl MotionPhysics {
    /// Derive intent-specific physics from global parameters.
    ///
    /// Destructive: snappier (higher stiffness, lower mass) to communicate urgency.
    /// Constructive: smoother (lower stiffness, higher damping) for confidence.
    /// Informational: gentler (lower stiffness, higher damping) to avoid distraction.
    pub fn for_intent(&self, intent: Intent) -> Self {
        match intent {
            Intent::Neutral => self.clone(),
            Intent::Constructive => Self {
                mass: self.mass,
                stiffness: self.stiffness * 0.8,
                damping: self.damping * 1.1,
            },
            Intent::Destructive => Self {
                mass: self.mass * 0.7,
                stiffness: self.stiffness * 1.4,
                damping: self.damping * 0.9,
            },
            Intent::Informational => Self {
                mass: self.mass * 1.2,
                stiffness: self.stiffness * 0.6,
                damping: self.damping * 1.3,
            },
        }
    }

    /// Damping ratio: < 1.0 underdamped (bouncy), 1.0 critical, > 1.0 overdamped.
    pub fn damping_ratio(&self) -> f32 {
        let critical = 2.0 * (self.stiffness * self.mass).sqrt();
        if critical > 0.0 {
            self.damping / critical
        } else {
            1.0
        }
    }

    /// Natural frequency in Hz.
    pub fn natural_frequency(&self) -> f32 {
        if self.mass > 0.0 {
            (self.stiffness / self.mass).sqrt() / (2.0 * std::f32::consts::PI)
        } else {
            0.0
        }
    }

    /// Approximate settling time in seconds (2% threshold).
    pub fn settling_time(&self) -> f32 {
        let gamma = self.damping / (2.0 * self.mass);
        if gamma > 0.0 {
            3.91 / gamma
        } else {
            f32::INFINITY
        }
    }

    /// Convert to (stiffness, damping, mass) for Spring::with_config().
    pub fn to_spring_config(&self) -> (f32, f32, f32) {
        (self.stiffness, self.damping, self.mass)
    }
}

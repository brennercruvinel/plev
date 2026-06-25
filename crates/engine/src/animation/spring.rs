use super::tween::Interpolate;

pub trait SpringInterpolate: Interpolate {
    fn add(&self, other: &Self) -> Self;
    fn sub(&self, other: &Self) -> Self;
    fn scale(&self, s: f32) -> Self;
    fn magnitude_sq(&self) -> f32;
}

impl SpringInterpolate for f32 {
    fn add(&self, other: &Self) -> Self {
        self + other
    }
    fn sub(&self, other: &Self) -> Self {
        self - other
    }
    fn scale(&self, s: f32) -> Self {
        self * s
    }
    fn magnitude_sq(&self) -> f32 {
        self * self
    }
}

impl<const N: usize> SpringInterpolate for [f32; N] {
    fn add(&self, other: &Self) -> Self {
        std::array::from_fn(|i| self[i] + other[i])
    }
    fn sub(&self, other: &Self) -> Self {
        std::array::from_fn(|i| self[i] - other[i])
    }
    fn scale(&self, s: f32) -> Self {
        std::array::from_fn(|i| self[i] * s)
    }
    fn magnitude_sq(&self) -> f32 {
        self.iter().map(|x| x * x).sum()
    }
}

#[derive(Clone, Debug)]
pub struct Spring<T: SpringInterpolate> {
    value: T,
    velocity: T,
    target: T,
    pub stiffness: f32,
    pub damping: f32,
    pub mass: f32,
    rest_threshold: f32,
    at_rest: bool,
}

impl<T: SpringInterpolate> Spring<T> {
    pub fn new(initial: T) -> Self
    where
        T: Default,
    {
        Self {
            value: initial.clone(),
            velocity: T::default(),
            target: initial,
            stiffness: 100.0,
            damping: 10.0,
            mass: 1.0,
            rest_threshold: 0.001,
            at_rest: true,
        }
    }

    pub fn with_config(mut self, stiffness: f32, damping: f32, mass: f32) -> Self {
        self.stiffness = stiffness;
        self.damping = damping;
        self.mass = mass;
        self
    }

    /// Configure spring from theme MotionPhysics.
    pub fn with_motion(self, motion: &crate::theme::MotionPhysics) -> Self {
        let (s, d, m) = motion.to_spring_config();
        self.with_config(s, d, m)
    }

    pub fn set_target(&mut self, target: T) {
        self.target = target;
        self.at_rest = false;
    }

    pub fn damping_ratio(&self) -> f32 {
        let omega_0 = (self.stiffness / self.mass).sqrt();
        if omega_0 > 0.0 {
            self.damping / (2.0 * (self.stiffness * self.mass).sqrt())
        } else {
            1.0
        }
    }

    pub fn tick(&mut self, dt: f32) {
        if self.at_rest || dt <= 0.0 {
            return;
        }

        let omega_0_sq = self.stiffness / self.mass;
        let gamma = self.damping / (2.0 * self.mass);
        let gamma_sq = gamma * gamma;
        let disc = gamma_sq - omega_0_sq;

        let displacement = self.value.sub(&self.target);

        let (pos_pos, pos_vel, vel_pos, vel_vel) = if disc < -1e-6 {
            let omega_d = (-disc).sqrt();
            let exp = (-gamma * dt).exp();
            let cos_wd = (omega_d * dt).cos();
            let sin_wd = (omega_d * dt).sin();
            let g_over_wd = gamma / omega_d;
            (
                exp * (cos_wd + g_over_wd * sin_wd),
                exp * sin_wd / omega_d,
                -exp * omega_0_sq * sin_wd / omega_d,
                exp * (cos_wd - g_over_wd * sin_wd),
            )
        } else if disc > 1e-6 {
            let s = disc.sqrt();
            let r1 = -gamma + s;
            let r2 = -gamma - s;
            let e1 = (r1 * dt).exp();
            let e2 = (r2 * dt).exp();
            let inv = 1.0 / (r1 - r2);
            (
                (r1 * e2 - r2 * e1) * inv,
                (e1 - e2) * inv,
                r1 * r2 * (e2 - e1) * inv,
                (r1 * e1 - r2 * e2) * inv,
            )
        } else {
            let exp = (-gamma * dt).exp();
            (
                exp * (1.0 + gamma * dt),
                exp * dt,
                -exp * gamma * gamma * dt,
                exp * (1.0 - gamma * dt),
            )
        };

        let new_disp = displacement
            .scale(pos_pos)
            .add(&self.velocity.scale(pos_vel));
        let new_vel = displacement
            .scale(vel_pos)
            .add(&self.velocity.scale(vel_vel));

        self.value = self.target.add(&new_disp);
        self.velocity = new_vel;

        if new_disp.magnitude_sq() < self.rest_threshold * self.rest_threshold
            && self.velocity.magnitude_sq() < self.rest_threshold * self.rest_threshold
        {
            self.value = self.target.clone();
            self.velocity = new_disp.scale(0.0);
            self.at_rest = true;
        }
    }

    pub fn get(&self) -> T {
        self.value.clone()
    }

    pub fn is_animating(&self) -> bool {
        !self.at_rest
    }
}

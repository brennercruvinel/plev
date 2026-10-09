#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum Easing {
    #[default]
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
    EaseInCubic,
    EaseOutCubic,
    EaseInOutCubic,
    EaseInQuart,
    EaseOutQuart,
    EaseInOutQuart,
    EaseInQuint,
    EaseOutQuint,
    EaseInOutQuint,
    EaseInSine,
    EaseOutSine,
    EaseInOutSine,
    EaseInExpo,
    EaseOutExpo,
    EaseInOutExpo,
    EaseInCirc,
    EaseOutCirc,
    EaseInOutCirc,
    EaseInBack,
    EaseOutBack,
    EaseInOutBack,
    EaseInElastic,
    EaseOutElastic,
    EaseInOutElastic,
    EaseInBounce,
    EaseOutBounce,
    EaseInOutBounce,
    Step,
    Hold,
    CubicBezier(f32, f32, f32, f32),
}

pub fn ease(t: f32, easing: Easing) -> f32 {
    let t = t.clamp(0.0, 1.0);
    match easing {
        Easing::Linear => t,

        Easing::EaseIn => t * t,
        Easing::EaseOut => t * (2.0 - t),
        Easing::EaseInOut => {
            if t < 0.5 {
                2.0 * t * t
            } else {
                -1.0 + (4.0 - 2.0 * t) * t
            }
        }

        Easing::EaseInCubic => t * t * t,
        Easing::EaseOutCubic => {
            let t1 = t - 1.0;
            1.0 + t1 * t1 * t1
        }
        Easing::EaseInOutCubic => {
            if t < 0.5 {
                4.0 * t * t * t
            } else {
                let t1 = 2.0 * t - 2.0;
                1.0 + t1 * t1 * t1 / 2.0
            }
        }

        Easing::EaseInQuart => t * t * t * t,
        Easing::EaseOutQuart => {
            let t1 = t - 1.0;
            1.0 - t1 * t1 * t1 * t1
        }
        Easing::EaseInOutQuart => {
            if t < 0.5 {
                8.0 * t * t * t * t
            } else {
                let t1 = t - 1.0;
                1.0 - 8.0 * t1 * t1 * t1 * t1
            }
        }

        Easing::EaseInQuint => t * t * t * t * t,
        Easing::EaseOutQuint => {
            let t1 = t - 1.0;
            1.0 + t1 * t1 * t1 * t1 * t1
        }
        Easing::EaseInOutQuint => {
            if t < 0.5 {
                16.0 * t * t * t * t * t
            } else {
                let t1 = 2.0 * t - 2.0;
                1.0 + t1 * t1 * t1 * t1 * t1 / 2.0
            }
        }

        Easing::EaseInSine => 1.0 - (t * std::f32::consts::FRAC_PI_2).cos(),
        Easing::EaseOutSine => (t * std::f32::consts::FRAC_PI_2).sin(),
        Easing::EaseInOutSine => -(((t * std::f32::consts::PI).cos() - 1.0) / 2.0),

        Easing::EaseInExpo => {
            if t == 0.0 {
                0.0
            } else {
                (2.0_f32).powf(10.0 * t - 10.0)
            }
        }
        Easing::EaseOutExpo => {
            if t == 1.0 {
                1.0
            } else {
                1.0 - (2.0_f32).powf(-10.0 * t)
            }
        }
        Easing::EaseInOutExpo => {
            if t == 0.0 {
                0.0
            } else if t == 1.0 {
                1.0
            } else if t < 0.5 {
                (2.0_f32).powf(20.0 * t - 10.0) / 2.0
            } else {
                (2.0 - (2.0_f32).powf(-20.0 * t + 10.0)) / 2.0
            }
        }

        Easing::EaseInCirc => 1.0 - (1.0 - t * t).sqrt(),
        Easing::EaseOutCirc => (1.0 - (t - 1.0) * (t - 1.0)).sqrt(),
        Easing::EaseInOutCirc => {
            if t < 0.5 {
                (1.0 - (1.0 - (2.0 * t) * (2.0 * t)).sqrt()) / 2.0
            } else {
                ((1.0 - (-2.0 * t + 2.0) * (-2.0 * t + 2.0)).sqrt() + 1.0) / 2.0
            }
        }

        Easing::EaseInBack => {
            let c1 = 1.70158;
            let c3 = c1 + 1.0;
            c3 * t * t * t - c1 * t * t
        }
        Easing::EaseOutBack => {
            let c1 = 1.70158;
            let c3 = c1 + 1.0;
            let t1 = t - 1.0;
            1.0 + c3 * t1 * t1 * t1 + c1 * t1 * t1
        }
        Easing::EaseInOutBack => {
            let c1 = 1.70158;
            let c2 = c1 * 1.525;
            if t < 0.5 {
                ((2.0 * t) * (2.0 * t) * ((c2 + 1.0) * 2.0 * t - c2)) / 2.0
            } else {
                ((2.0 * t - 2.0) * (2.0 * t - 2.0) * ((c2 + 1.0) * (2.0 * t - 2.0) + c2) + 2.0)
                    / 2.0
            }
        }

        Easing::EaseInElastic => {
            if t == 0.0 {
                0.0
            } else if t == 1.0 {
                1.0
            } else {
                let c4 = (2.0 * std::f32::consts::PI) / 3.0;
                -(2.0_f32.powf(10.0 * t - 10.0)) * ((10.0 * t - 10.75) * c4).sin()
            }
        }
        Easing::EaseOutElastic => {
            if t == 0.0 {
                0.0
            } else if t == 1.0 {
                1.0
            } else {
                let c4 = (2.0 * std::f32::consts::PI) / 3.0;
                2.0_f32.powf(-10.0 * t) * ((10.0 * t - 0.75) * c4).sin() + 1.0
            }
        }
        Easing::EaseInOutElastic => {
            if t == 0.0 {
                0.0
            } else if t == 1.0 {
                1.0
            } else {
                let c5 = (2.0 * std::f32::consts::PI) / 4.5;
                if t < 0.5 {
                    -(2.0_f32.powf(20.0 * t - 10.0) * ((20.0 * t - 11.125) * c5).sin()) / 2.0
                } else {
                    (2.0_f32.powf(-20.0 * t + 10.0) * ((20.0 * t - 11.125) * c5).sin()) / 2.0 + 1.0
                }
            }
        }

        Easing::EaseOutBounce => bounce_out(t),
        Easing::EaseInBounce => 1.0 - bounce_out(1.0 - t),
        Easing::EaseInOutBounce => {
            if t < 0.5 {
                (1.0 - bounce_out(1.0 - 2.0 * t)) / 2.0
            } else {
                (1.0 + bounce_out(2.0 * t - 1.0)) / 2.0
            }
        }

        Easing::Step => t.round(),
        Easing::Hold => {
            if t >= 1.0 {
                1.0
            } else {
                0.0
            }
        }

        Easing::CubicBezier(x1, y1, x2, y2) => cubic_bezier(t, x1, y1, x2, y2),
    }
}

fn bounce_out(t: f32) -> f32 {
    let n1 = 7.5625;
    let d1 = 2.75;
    if t < 1.0 / d1 {
        n1 * t * t
    } else if t < 2.0 / d1 {
        let t = t - 1.5 / d1;
        n1 * t * t + 0.75
    } else if t < 2.5 / d1 {
        let t = t - 2.25 / d1;
        n1 * t * t + 0.9375
    } else {
        let t = t - 2.625 / d1;
        n1 * t * t + 0.984375
    }
}

fn cubic_bezier(t: f32, x1: f32, y1: f32, x2: f32, y2: f32) -> f32 {
    let mut guess = t;
    for _ in 0..8 {
        let x = cubic_bezier_sample(guess, x1, x2) - t;
        let dx = cubic_bezier_slope(guess, x1, x2);
        if dx.abs() < 1e-7 {
            break;
        }
        guess -= x / dx;
        guess = guess.clamp(0.0, 1.0);
    }
    cubic_bezier_sample(guess, y1, y2)
}

fn cubic_bezier_sample(t: f32, p1: f32, p2: f32) -> f32 {
    let t2 = t * t;
    let t3 = t2 * t;
    let mt = 1.0 - t;
    let mt2 = mt * mt;
    3.0 * mt2 * t * p1 + 3.0 * mt * t2 * p2 + t3
}

fn cubic_bezier_slope(t: f32, p1: f32, p2: f32) -> f32 {
    let mt = 1.0 - t;
    3.0 * mt * mt * p1 + 6.0 * mt * t * (p2 - p1) + 3.0 * t * t * (1.0 - p2)
}

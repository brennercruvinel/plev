pub trait IntoF32 {
    fn into_f32(self) -> f32;
}

impl IntoF32 for f32 {
    fn into_f32(self) -> f32 {
        self
    }
}
impl IntoF32 for f64 {
    fn into_f32(self) -> f32 {
        self as f32
    }
}
impl IntoF32 for i32 {
    fn into_f32(self) -> f32 {
        self as f32
    }
}
impl IntoF32 for i64 {
    fn into_f32(self) -> f32 {
        self as f32
    }
}
impl IntoF32 for u32 {
    fn into_f32(self) -> f32 {
        self as f32
    }
}
impl IntoF32 for usize {
    fn into_f32(self) -> f32 {
        self as f32
    }
}

pub trait IntoRadius {
    fn into_radius(self) -> f32;
}

impl IntoRadius for f32 {
    fn into_radius(self) -> f32 {
        self
    }
}
impl IntoRadius for i32 {
    fn into_radius(self) -> f32 {
        self as f32
    }
}
impl IntoRadius for &str {
    fn into_radius(self) -> f32 {
        match self {
            "none" => 0.0,
            "sm" => 2.0,
            "md" | "base" => 4.0,
            "lg" => 8.0,
            "xl" => 12.0,
            "2xl" => 16.0,
            "3xl" => 24.0,
            "full" => 9999.0,
            _ => self.parse::<f32>().unwrap_or(0.0),
        }
    }
}

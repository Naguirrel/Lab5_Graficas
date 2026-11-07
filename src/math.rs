use std::ops::{Add, Mul, Sub};

#[derive(Clone, Copy, Debug, Default)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vec3 {
    pub const fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    pub fn dot(self, other: Self) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    pub fn length(self) -> f32 {
        self.dot(self).sqrt()
    }

    pub fn normalized(self) -> Self {
        let length = self.length();
        if length < 1e-8 {
            Self::default()
        } else {
            self * (1.0 / length)
        }
    }

    pub fn rotate_y(self, angle: f32) -> Self {
        let (s, c) = angle.sin_cos();
        Self::new(c * self.x + s * self.z, self.y, -s * self.x + c * self.z)
    }

    pub fn rotate_x(self, angle: f32) -> Self {
        let (s, c) = angle.sin_cos();
        Self::new(self.x, c * self.y - s * self.z, s * self.y + c * self.z)
    }

    pub fn rotate_z(self, angle: f32) -> Self {
        let (s, c) = angle.sin_cos();
        Self::new(c * self.x - s * self.y, s * self.x + c * self.y, self.z)
    }
}

impl Add for Vec3 {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self::new(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z)
    }
}
impl Sub for Vec3 {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Self::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z)
    }
}
impl Mul<f32> for Vec3 {
    type Output = Self;
    fn mul(self, rhs: f32) -> Self {
        Self::new(self.x * rhs, self.y * rhs, self.z * rhs)
    }
}

// Los colores se calculan como RGB de 0 a 1 y se convierten a bytes al final.
pub type Color = Vec3;
pub fn rgb(r: u8, g: u8, b: u8) -> Color {
    Vec3::new(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0)
}
pub fn mix(a: Color, b: Color, amount: f32) -> Color {
    let t = amount.clamp(0.0, 1.0);
    a * (1.0 - t) + b * t
}
pub fn smoothstep(low: f32, high: f32, value: f32) -> f32 {
    let t = ((value - low) / (high - low)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}
pub fn pack(color: Color) -> u32 {
    let byte = |v: f32| (v.clamp(0.0, 1.0) * 255.0).round() as u32;
    (byte(color.x) << 16) | (byte(color.y) << 8) | byte(color.z)
}

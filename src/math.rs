use image::{imageops::sample_nearest, DynamicImage, Rgba};

/// spherical coord without radius
#[derive(Debug)]
pub struct SphericalAngle {
    /// rotation on pole axis
    pub theta: f32,
    /// angle on equator
    pub phi: f32,
}

impl SphericalAngle {
    pub fn from_normalized_vector(value: Vector3) -> Self {
        let theta = value.y.atan2(value.x);
        let phi = value.z.asin();
        Self { theta, phi }
    }
    pub fn to_uv(&self) -> (f32, f32) {
        use std::f32::consts::PI;
        (self.theta / (2.0 * PI) + 0.5, self.phi / PI + 0.5)
    }
}

#[derive(clap::ValueEnum, Clone, Debug)]
pub enum Interpolation {
    Linear,
    Nearest,
}
impl Interpolation {
    pub fn sample(&self, img: &DynamicImage, uv: (f32, f32)) -> Rgba<u8> {
        use image::imageops::sample_bilinear;
        match self {
            Self::Linear => sample_bilinear(img, uv.0, uv.1),
            Self::Nearest => sample_nearest(img, uv.0, uv.1),
        }
        .unwrap_or(Rgba([0, 0, 0, 255]))
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Vector3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vector3 {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }
    /// return a normalized version of this vector
    pub fn normalize(&self) -> Self {
        let len = self.len();
        Self {
            x: self.x / len,
            y: self.y / len,
            z: self.z / len,
        }
    }
    /// make this vector3 normalized
    // pub fn normalize_mut(&mut self) {
    //     let len = self.len();
    //     self.x /= len;
    //     self.y /= len;
    //     self.z /= len;
    // }
    pub fn len(&self) -> f32 {
        (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }
    // pub fn len_squared(&self) -> f32 {
    //     self.x * self.x + self.y * self.y + self.z * self.z
    // }
}

// impl std::ops::Add for Vector3 {
//     type Output = Vector3;

//     fn add(self, rhs: Self) -> Self::Output {
//         Vector3 {
//             x: self.x + rhs.x,
//             y: self.y + rhs.y,
//             z: self.z + rhs.z,
//         }
//     }
// }

// impl std::ops::Sub for Vector3 {
//     type Output = Vector3;

//     fn sub(self, rhs: Self) -> Self::Output {
//         Vector3 {
//             x: self.x - rhs.x,
//             y: self.y - rhs.y,
//             z: self.z - rhs.z,
//         }
//     }
// }

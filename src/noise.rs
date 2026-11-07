use crate::math::{Vec3, smoothstep};

// Ruido de valor 3D: valores pseudoaleatorios en una rejilla, interpolados.
// No hay archivos de imagen, mapas de textura ni tablas de colores precargadas.
fn hash(x: i32, y: i32, z: i32) -> f32 {
    let mut h = (x as u32).wrapping_mul(374_761_393)
        ^ (y as u32).wrapping_mul(668_265_263)
        ^ (z as u32).wrapping_mul(2_147_483_647);
    h = (h ^ (h >> 13)).wrapping_mul(1_274_126_177);
    (h ^ (h >> 16)) as f32 / u32::MAX as f32
}

pub fn noise(p: Vec3) -> f32 {
    let (x, y, z) = (p.x.floor() as i32, p.y.floor() as i32, p.z.floor() as i32);
    let u = smoothstep(0.0, 1.0, p.x - p.x.floor());
    let v = smoothstep(0.0, 1.0, p.y - p.y.floor());
    let w = smoothstep(0.0, 1.0, p.z - p.z.floor());
    let lerp = |a: f32, b: f32, t: f32| a + (b - a) * t;
    let a = lerp(hash(x, y, z), hash(x + 1, y, z), u);
    let b = lerp(hash(x, y + 1, z), hash(x + 1, y + 1, z), u);
    let c = lerp(hash(x, y, z + 1), hash(x + 1, y, z + 1), u);
    let d = lerp(hash(x, y + 1, z + 1), hash(x + 1, y + 1, z + 1), u);
    lerp(lerp(a, b, v), lerp(c, d, v), w)
}

// fBm: superpone ruido grande, mediano y pequeno con menor peso cada vez.
pub fn fbm(mut p: Vec3, octaves: usize) -> f32 {
    let (mut sum, mut amplitude, mut weight) = (0.0, 0.5, 0.0);
    for _ in 0..octaves {
        sum += noise(p) * amplitude;
        weight += amplitude;
        p = p * 2.03 + Vec3::new(17.1, 9.2, 13.7);
        amplitude *= 0.5;
    }
    sum / weight
}

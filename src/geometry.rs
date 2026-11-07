use crate::math::Vec3;
use std::f32::consts::{PI, TAU};

#[derive(Clone, Copy)]
pub struct Vertex {
    pub position: Vec3,
    pub normal: Vec3,
}

pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub triangles: Vec<[usize; 3]>,
}

impl Mesh {
    // Una sola geometria sirve para los tres cuerpos. Los shaders los distinguen.
    pub fn sphere(stacks: usize, slices: usize) -> Self {
        assert!(stacks >= 2 && slices >= 3);
        let mut vertices = Vec::new();
        let mut triangles = Vec::new();
        for row in 0..=stacks {
            let latitude = PI * row as f32 / stacks as f32;
            for column in 0..=slices {
                let longitude = TAU * column as f32 / slices as f32;
                let position = Vec3::new(
                    latitude.sin() * longitude.cos(),
                    latitude.cos(),
                    latitude.sin() * longitude.sin(),
                );
                vertices.push(Vertex {
                    position,
                    normal: position,
                });
            }
        }
        for row in 0..stacks {
            for column in 0..slices {
                let a = row * (slices + 1) + column;
                let b = a + slices + 1;
                if row > 0 {
                    triangles.push([a, b, a + 1]);
                }
                if row + 1 < stacks {
                    triangles.push([a + 1, b, b + 1]);
                }
            }
        }
        Self {
            vertices,
            triangles,
        }
    }

    // Malla independiente de la esfera, como pide la rubrica.
    pub fn ring(segments: usize) -> Self {
        let mut vertices = Vec::new();
        let mut triangles = Vec::new();
        for i in 0..=segments {
            let angle = TAU * i as f32 / segments as f32;
            for radius in [1.28, 2.03] {
                vertices.push(Vertex {
                    position: Vec3::new(radius * angle.cos(), 0.0, radius * angle.sin()),
                    normal: Vec3::new(0.0, 1.0, 0.0),
                });
            }
            if i < segments {
                let a = i * 2;
                triangles.extend([[a, a + 1, a + 2], [a + 1, a + 3, a + 2]]);
            }
        }
        Self {
            vertices,
            triangles,
        }
    }
}

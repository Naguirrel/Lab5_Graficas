use crate::{
    framebuffer::Framebuffer,
    geometry::Mesh,
    math::{Vec3, pack},
    shaders::{Fragment, Shader, shade_with_light},
};

#[derive(Clone, Copy)]
pub struct Transform {
    pub center: Vec3,
    pub scale: f32,
    pub rotation_y: f32,
    pub rotation_x: f32,
    pub rotation_z: f32,
}

impl Transform {
    pub fn orient(self, p: Vec3) -> Vec3 {
        p.rotate_y(self.rotation_y)
            .rotate_x(self.rotation_x)
            .rotate_z(self.rotation_z)
    }
}

#[derive(Clone, Copy)]
struct ScreenVertex {
    x: f32,
    y: f32,
    depth: f32,
    position: Vec3,
    normal: Vec3,
}

fn edge(a: ScreenVertex, b: ScreenVertex, x: f32, y: f32) -> f32 {
    (x - a.x) * (b.y - a.y) - (y - a.y) * (b.x - a.x)
}

pub fn draw_mesh(
    fb: &mut Framebuffer,
    mesh: &Mesh,
    transform: Transform,
    shader: Shader,
    layers: u8,
) {
    draw_mesh_lit(
        fb,
        mesh,
        transform,
        shader,
        layers,
        Vec3::new(-0.6, 0.7, 1.0),
    );
}

pub fn draw_mesh_lit(
    fb: &mut Framebuffer,
    mesh: &Mesh,
    transform: Transform,
    shader: Shader,
    layers: u8,
    light: Vec3,
) {
    let light = light.normalized();
    let screen: Vec<_> = mesh
        .vertices
        .iter()
        .map(|v| {
            let oriented = transform.orient(v.position) * transform.scale;
            ScreenVertex {
                x: transform.center.x + oriented.x,
                y: transform.center.y - oriented.y,
                depth: transform.center.z + oriented.z,
                position: v.position,
                normal: transform.orient(v.normal),
            }
        })
        .collect();

    for &[a, b, c] in &mesh.triangles {
        let [a, b, c] = [screen[a], screen[b], screen[c]];
        let area = edge(a, b, c.x, c.y);
        if area.abs() < 1e-6 {
            continue;
        }
        // Las esferas tienen normales radiales: solo dibujamos la cara visible.
        if shader != Shader::Ring && a.normal.z + b.normal.z + c.normal.z < 0.0 {
            continue;
        }
        let min_x = a.x.min(b.x).min(c.x).floor().max(0.0) as i32;
        let max_x = a.x.max(b.x).max(c.x).ceil().min((fb.width - 1) as f32) as i32;
        let min_y = a.y.min(b.y).min(c.y).floor().max(0.0) as i32;
        let max_y = a.y.max(b.y).max(c.y).ceil().min((fb.height - 1) as f32) as i32;
        for y in min_y..=max_y {
            for x in min_x..=max_x {
                let px = x as f32 + 0.5;
                let py = y as f32 + 0.5;
                let w0 = edge(b, c, px, py) / area;
                let w1 = edge(c, a, px, py) / area;
                let w2 = 1.0 - w0 - w1;
                if w0 < -1e-6 || w1 < -1e-6 || w2 < -1e-6 {
                    continue;
                }
                let depth = a.depth * w0 + b.depth * w1 + c.depth * w2;
                let index = y as usize * fb.width + x as usize;
                if !fb.visible(index, depth) {
                    continue;
                }
                // Baricentros: interpolamos datos de los 3 vertices en este pixel.
                let local = a.position * w0 + b.position * w1 + c.position * w2;
                let fragment = Fragment {
                    position: if shader == Shader::Ring {
                        local
                    } else {
                        local.normalized()
                    },
                    normal: (a.normal * w0 + b.normal * w1 + c.normal * w2).normalized(),
                };
                // Este llamado es el cambio principal respecto al Lab 4:
                // antes, todos los pixeles tenian MODEL_COLOR; ahora se calculan.
                if let Some(color) = shade_with_light(shader, fragment, layers, light) {
                    fb.point(index, depth, pack(color));
                }
            }
        }
    }
}

use crate::{
    math::{Color, Vec3, mix, rgb, smoothstep},
    noise::{fbm, noise},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Shader {
    Star,
    Rocky,
    Gas,
    Ocean,
    Ice,
    Desert,
    Ring,
    Moon,
    Collector,
    Energy,
}

impl Shader {
    pub fn is_sphere(self) -> bool {
        matches!(
            self,
            Self::Star
                | Self::Rocky
                | Self::Gas
                | Self::Ocean
                | Self::Ice
                | Self::Desert
                | Self::Moon
        )
    }
}

// Un fragmento es un candidato a pixel de un triangulo rasterizado.
// position esta en el modelo: el patron gira pegado a la superficie.
// normal esta orientada en la escena: permite calcular la iluminacion.
#[derive(Clone, Copy)]
pub struct Fragment {
    pub position: Vec3,
    pub normal: Vec3,
}

fn lighting(color: Color, normal: Vec3, light: Vec3) -> Color {
    let diffuse = normal.dot(light).max(0.0);
    color * (0.12 + 0.88 * diffuse)
}

fn atmosphere(color: Color, normal: Vec3, tint: Color, strength: f32) -> Color {
    let rim = (1.0 - normal.z.abs()).powf(3.0);
    mix(color, tint, rim * strength)
}

pub fn shade(shader: Shader, fragment: Fragment, layers: u8) -> Option<Color> {
    shade_with_light(
        shader,
        fragment,
        layers,
        Vec3::new(-0.6, 0.7, 1.0).normalized(),
    )
}

// En la vista del sistema, cada cuerpo recibe la direccion hacia el Sol.
pub fn shade_with_light(
    shader: Shader,
    fragment: Fragment,
    layers: u8,
    light: Vec3,
) -> Option<Color> {
    let layers = layers.clamp(1, 4);
    match shader {
        Shader::Star => Some(star(fragment, layers)),
        Shader::Rocky => Some(rocky(fragment, layers, light)),
        Shader::Gas => Some(gas(fragment, layers, light)),
        Shader::Ocean => Some(ocean(fragment, layers, light)),
        Shader::Ice => Some(ice(fragment, layers, light)),
        Shader::Desert => Some(desert(fragment, layers, light)),
        Shader::Ring => ring(fragment, light),
        Shader::Moon => Some(moon(fragment, light)),
        Shader::Collector => Some(collector(fragment)),
        Shader::Energy => Some(energy(fragment)),
    }
}

fn collector(f: Fragment) -> Color {
    let u = f.position.x / 0.46 + 0.5;
    let v = f.position.y / 0.28 + 0.5;
    let edge = u.min(1.0 - u).min(v.min(1.0 - v));
    if edge < 0.055 {
        return rgb(213, 163, 75) * (0.62 + f.normal.z.abs() * 0.38);
    }
    let cell_x = (u * 8.0).fract();
    let cell_y = (v * 4.0).fract();
    if cell_x < 0.08 || cell_y < 0.10 {
        return rgb(70, 146, 185);
    }
    let sheen = (u * 9.0 + v * 5.0).sin() * 0.5 + 0.5;
    mix(rgb(10, 24, 49), rgb(29, 82, 128), sheen) * (0.65 + 0.35 * f.normal.z.abs())
}

fn energy(f: Fragment) -> Color {
    let pulse = (f.position.z * 95.0).sin() * 0.5 + 0.5;
    mix(rgb(213, 102, 29), rgb(255, 228, 127), pulse)
}

// ASTER: estrella de plasma naranja. No depende de una luz externa.
fn star(f: Fragment, layers: u8) -> Color {
    let p = f.position;
    // CAPA 1: conveccion; grandes regiones de plasma de distintas temperaturas.
    let convection = fbm(p * 4.5 + Vec3::new(3.0, 8.0, 2.0), 4);
    let mut color = mix(
        rgb(155, 25, 7),
        rgb(255, 175, 36),
        smoothstep(0.25, 0.72, convection),
    );
    if layers >= 2 {
        // CAPA 2: granulos pequenos y calientes sobre el plasma.
        let granules = fbm(p * 47.0, 3);
        color = mix(
            color * 0.8,
            rgb(255, 234, 142),
            smoothstep(0.30, 0.78, granules) * 0.8,
        );
    }
    if layers >= 3 {
        // CAPA 3: filamentos luminosos retorcidos por ruido de baja frecuencia.
        let warped = p * 12.0 + Vec3::new(convection * 6.0, convection * 3.0, 0.0);
        let ridge = (fbm(warped, 3) - 0.50).abs();
        let filaments = 1.0 - smoothstep(0.012, 0.065, ridge);
        color = mix(color, rgb(255, 231, 119), filaments * 0.65);
    }
    if layers >= 4 {
        // CAPA 4: manchas estelares con centro oscuro y penumbra irregular.
        let spots = fbm(p * 7.0 + Vec3::new(20.0, 4.0, 6.0), 3);
        let mask = smoothstep(0.61, 0.74, spots);
        color = mix(color, rgb(69, 14, 12), mask * 0.92);
    }
    // Oscurecimiento de limbo: conserva volumen sin tratarla como roca iluminada.
    color * (0.68 + 0.32 * f.normal.z.abs().sqrt())
}

fn crater_mask(p: Vec3) -> (f32, f32) {
    // Centros sobre la esfera. Cada crater cambia solo el color, no la geometria.
    let centers = [
        (Vec3::new(-0.42, 0.28, 0.88), 0.19),
        (Vec3::new(0.52, 0.44, 0.72), 0.12),
        (Vec3::new(-0.14, -0.62, 0.77), 0.15),
        (Vec3::new(0.72, -0.27, 0.65), 0.10),
        (Vec3::new(-0.71, -0.18, -0.70), 0.17),
        (Vec3::new(0.31, 0.81, -0.49), 0.13),
        (Vec3::new(0.18, -0.41, -0.90), 0.20),
    ];
    let (mut bowl, mut rim): (f32, f32) = (0.0, 0.0);
    for (center, radius) in centers {
        let d = (p - center.normalized()).length() / radius;
        bowl = bowl.max(1.0 - smoothstep(0.40, 0.95, d));
        rim = rim.max((1.0 - smoothstep(0.06, 0.21, (d - 1.0).abs())) * 0.75);
    }
    (bowl, rim)
}

// IGNIS: planeta volcanico de basalto, cobre y fisuras turquesa.
fn rocky(f: Fragment, layers: u8, light: Vec3) -> Color {
    let p = f.position;
    // CAPA 1: distribucion de roca oscura y minerales cobrizos.
    let terrain = fbm(p * 3.8 + Vec3::new(4.0, 2.0, 7.0), 5);
    let mut color = mix(
        rgb(25, 31, 40),
        rgb(180, 108, 65),
        smoothstep(0.30, 0.70, terrain),
    );
    if layers >= 2 {
        // CAPA 2: grano mineral a escala mas fina que las regiones de roca.
        let grain = fbm(p * 42.0, 3);
        color = mix(color * 0.55, mix(color, rgb(216, 184, 133), 0.32), grain);
    }
    if layers >= 3 {
        // CAPA 3: cuencas oscuras y bordes claros de crateres.
        let (bowl, rim) = crater_mask(p);
        color = mix(color, rgb(15, 20, 25), bowl * 0.85);
        color = mix(color, rgb(177, 151, 117), rim * 0.75);
    }
    let mut color = lighting(color, f.normal, light);
    if layers >= 4 {
        // CAPA 4: red de fracturas con halo y centro emisivo.
        let field = fbm(p * 8.5 + Vec3::new(6.0, 9.0, 3.0), 4);
        let distance = (field - 0.5).abs();
        let cracks = 1.0 - smoothstep(0.008, 0.025, distance);
        let hot_core = 1.0 - smoothstep(0.002, 0.008, distance);
        color = mix(color, rgb(4, 133, 137), cracks * 0.84);
        color = mix(color, rgb(128, 255, 218), hot_core * 0.87);
    }
    atmosphere(color, f.normal, rgb(31, 126, 147), 0.27)
}

// ZEPHIR: gigante gaseoso de bandas azuladas, crema y tormentas rosas.
fn gas(f: Fragment, layers: u8, light: Vec3) -> Color {
    let p = f.position;
    let turbulence = fbm(p * 5.0 + Vec3::new(1.0, 6.0, 2.0), 4);
    // CAPA 1: bandas por latitud, deformadas para que no sean lineas perfectas.
    let latitude = p.y + (turbulence - 0.5) * 0.10;
    let bands = (latitude * 28.0).sin() * 0.5 + 0.5;
    let mut color = mix(
        rgb(28, 73, 106),
        rgb(184, 219, 216),
        smoothstep(0.1, 0.9, bands),
    );
    if layers >= 2 {
        // CAPA 2: cinturones anchos de distinta composicion y color.
        let belt = (latitude * 10.0 + 0.7).sin();
        let tint = mix(rgb(116, 64, 102), rgb(232, 192, 143), bands);
        color = mix(color, tint, smoothstep(0.20, 0.85, belt) * 0.88);
    }
    if layers >= 3 {
        // CAPA 3: filamentos y nubes finas dentro de las bandas.
        let clouds = fbm(Vec3::new(p.x * 26.0, p.y * 95.0, p.z * 26.0), 3);
        color = mix(color * 0.78, mix(color, rgb(243, 236, 210), 0.55), clouds);
    }
    if layers >= 4 {
        // CAPA 4: tormenta oval con anillos de color que simulan un remolino.
        let longitude = p.x.atan2(p.z);
        let dx = longitude - 0.20;
        let dy = p.y + 0.23;
        let distance = ((dx / 0.35).powi(2) + (dy / 0.14).powi(2)).sqrt();
        let eye = 1.0 - smoothstep(0.78, 1.12, distance);
        let spiral = (distance * 23.0 + dy.atan2(dx) * 2.0 + turbulence * 4.0).sin() * 0.5 + 0.5;
        let storm = mix(rgb(112, 40, 83), rgb(250, 170, 134), spiral);
        color = mix(color, storm, eye * 0.95);
    }
    atmosphere(
        lighting(color, f.normal, light),
        f.normal,
        rgb(125, 194, 232),
        0.45,
    )
}

fn ring(f: Fragment, light: Vec3) -> Option<Color> {
    let p = f.position;
    let radius = (p.x * p.x + p.z * p.z).sqrt();
    // Una division real: el fragmento se descarta y deja ver el fondo/planeta.
    if (1.66..1.72).contains(&radius) || (1.90..1.925).contains(&radius) {
        return None;
    }
    let stripes = (radius * 235.0).sin() * 0.5 + 0.5;
    let broad = (radius * 27.0).sin() * 0.5 + 0.5;
    let grain = noise(p * 130.0);
    let color = mix(rgb(77, 106, 135), rgb(221, 198, 161), broad);
    let color = color * (0.62 + stripes * 0.23 + grain * 0.15);
    // Sombra analitica del planeta sobre el anillo, sin mapa de sombras.
    let scene_p = p.rotate_x(0.55).rotate_z(-0.32);
    let toward_light = -scene_p.dot(light);
    let closest = scene_p + light * toward_light;
    let shadow = toward_light > 0.0 && closest.dot(closest) < 1.0;
    Some(color * if shadow { 0.22 } else { 0.94 })
}

fn moon(f: Fragment, light: Vec3) -> Color {
    let p = f.position;
    let ground = fbm(p * 7.0, 4);
    let mut color = mix(rgb(72, 80, 100), rgb(190, 200, 210), ground);
    let (bowl, rim) = crater_mask(p);
    color = mix(color, rgb(35, 40, 57), bowl * 0.8);
    color = mix(color, rgb(222, 219, 203), rim * 0.6);
    lighting(color, f.normal, light)
}

// PELAGIA: oceanos azules, continentes verdes y una cubierta de nubes.
fn ocean(f: Fragment, layers: u8, light: Vec3) -> Color {
    let p = f.position;
    let elevation = fbm(p * 3.4 + Vec3::new(5.2, 1.6, 8.3), 5);
    let land = smoothstep(0.49, 0.54, elevation);
    // CAPA 1: distribucion continental y oceanos profundos.
    let mut color = mix(rgb(8, 43, 113), rgb(50, 116, 65), land);
    if layers >= 2 {
        // CAPA 2: aguas someras turquesa, costas y relieve mineral continental.
        let shallows = smoothstep(0.39, 0.50, elevation) * (1.0 - land);
        color = mix(color, rgb(27, 174, 182), shallows * 0.9);
        let mountains = smoothstep(0.55, 0.72, elevation + fbm(p * 25.0, 3) * 0.10);
        color = mix(color, rgb(163, 143, 101), mountains * land);
    }
    if layers >= 3 {
        // CAPA 3: hielo polar de borde irregular, presente en ambos hemisferios.
        let polar = p.y.abs() + (fbm(p * 14.0, 3) - 0.5) * 0.16;
        color = mix(color, rgb(223, 240, 243), smoothstep(0.70, 0.88, polar));
    }
    if layers >= 4 {
        // CAPA 4: sistemas de nubes que cubren tierra y mar.
        let clouds = fbm(p * 8.0 + Vec3::new(3.0, 9.0, 4.0), 5);
        color = mix(
            color,
            rgb(246, 248, 244),
            smoothstep(0.48, 0.69, clouds) * 0.9,
        );
    }
    let mut color = lighting(color, f.normal, light);
    if layers >= 2 {
        let half = (light + Vec3::new(0.0, 0.0, 1.0)).normalized();
        let reflection = f.normal.dot(half).max(0.0).powf(70.0) * (1.0 - land);
        color = mix(color, rgb(221, 247, 255), reflection * 0.6);
    }
    atmosphere(color, f.normal, rgb(62, 145, 232), 0.40)
}

// NIVIS: glaciares lavanda, grietas azules y auroras polares.
fn ice(f: Fragment, layers: u8, light: Vec3) -> Color {
    let p = f.position;
    // CAPA 1: placas de hielo con distinto espesor.
    let plates = fbm(p * 4.8 + Vec3::new(6.0, 3.0, 2.0), 4);
    let mut color = mix(
        rgb(69, 73, 142),
        rgb(200, 212, 238),
        smoothstep(0.25, 0.75, plates),
    );
    if layers >= 2 {
        // CAPA 2: red de fisuras profundas dentro de las placas.
        let cracks = 1.0 - smoothstep(0.012, 0.055, (fbm(p * 11.0, 4) - 0.5).abs());
        color = mix(color, rgb(23, 81, 139), cracks * 0.88);
    }
    if layers >= 3 {
        // CAPA 3: escarcha y nieve granular acumulada en zonas frias.
        let frost = fbm(p * 55.0, 3);
        color = mix(
            color,
            rgb(242, 244, 255),
            smoothstep(0.38, 0.70, frost) * 0.58,
        );
    }
    let mut color = lighting(color, f.normal, light);
    if layers >= 4 {
        // CAPA 4: cortinas de aurora, coloreadas cerca de los polos.
        let longitude = p.x.atan2(p.z);
        let ribbon = (longitude * 13.0 + p.y * 23.0 + plates * 7.0).sin() * 0.5 + 0.5;
        let polar = smoothstep(0.48, 0.82, p.y.abs()) * (1.0 - smoothstep(0.93, 1.0, p.y.abs()));
        let tint = mix(rgb(79, 237, 207), rgb(196, 111, 242), ribbon);
        color = mix(color, tint, ribbon * polar * 0.78);
    }
    atmosphere(color, f.normal, rgb(161, 166, 248), 0.42)
}

// DUNA: arena rojiza, dunas doradas, salares y polvo arrastrado por el viento.
fn desert(f: Fragment, layers: u8, light: Vec3) -> Color {
    let p = f.position;
    let terrain = fbm(p * 4.0 + Vec3::new(2.0, 7.0, 11.0), 4);
    // CAPA 1: distribucion de arena y mesetas de terracota.
    let mut color = mix(
        rgb(130, 43, 32),
        rgb(225, 152, 78),
        smoothstep(0.25, 0.73, terrain),
    );
    if layers >= 2 {
        // CAPA 2: crestas de dunas curvadas por variaciones del terreno.
        let dunes = (p.y * 135.0 + p.x * 23.0 + terrain * 28.0).sin() * 0.5 + 0.5;
        color = mix(color * 0.77, mix(color, rgb(255, 204, 125), 0.5), dunes);
    }
    if layers >= 3 {
        // CAPA 3: cuencas salinas claras con limites irregulares.
        let basin = fbm(p * 6.2 + Vec3::new(11.0, 2.0, 7.0), 4);
        color = mix(
            color,
            rgb(236, 213, 184),
            smoothstep(0.53, 0.65, basin) * 0.9,
        );
    }
    if layers >= 4 {
        // CAPA 4: tormentas de polvo en filamentos alargados.
        let dust = fbm(
            Vec3::new(p.x * 8.0, p.y * 29.0, p.z * 8.0) + Vec3::new(4.0, 0.0, 3.0),
            4,
        );
        color = mix(
            color,
            rgb(245, 179, 112),
            smoothstep(0.48, 0.70, dust) * 0.66,
        );
    }
    atmosphere(
        lighting(color, f.normal, light),
        f.normal,
        rgb(211, 117, 68),
        0.24,
    )
}

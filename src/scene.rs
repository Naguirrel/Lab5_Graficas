use crate::{
    framebuffer::Framebuffer,
    geometry::Mesh,
    math::Vec3,
    overlay,
    renderer::{Transform, draw_mesh_lit},
    shaders::Shader,
};
use std::f32::consts::TAU;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Body {
    Star,
    Rocky,
    Gas,
    Ocean,
    Ice,
    City,
}

impl Body {
    pub const ALL: [Self; 6] = [
        Self::Star,
        Self::Rocky,
        Self::Gas,
        Self::Ocean,
        Self::Ice,
        Self::City,
    ];
    pub fn slug(self) -> &'static str {
        match self {
            Self::Star => "estrella",
            Self::Rocky => "rocoso",
            Self::Gas => "gaseoso",
            Self::Ocean => "oceanico",
            Self::Ice => "helado",
            Self::City => "coruscant",
        }
    }
    pub fn short_name(self) -> &'static str {
        match self {
            Self::Star => "ASTER",
            Self::Rocky => "IGNIS",
            Self::Gas => "ZEPHIR",
            Self::Ocean => "PELAGIA",
            Self::Ice => "NIVIS",
            Self::City => "CORUSCANT",
        }
    }
    pub fn name(self) -> &'static str {
        match self {
            Self::Star => "ASTER - Estrella",
            Self::Rocky => "IGNIS - Planeta rocoso",
            Self::Gas => "ZEPHIR - Gigante gaseoso",
            Self::Ocean => "PELAGIA - Planeta oceanico",
            Self::Ice => "NIVIS - Planeta helado",
            Self::City => "CORUSCANT - Planeta ciudad",
        }
    }
    pub fn shader(self) -> Shader {
        match self {
            Self::Star => Shader::Star,
            Self::Rocky => Shader::Rocky,
            Self::Gas => Shader::Gas,
            Self::Ocean => Shader::Ocean,
            Self::Ice => Shader::Ice,
            Self::City => Shader::City,
        }
    }
    pub fn spin_rate(self) -> f32 {
        match self {
            Self::Star => 0.12,
            Self::Rocky => 0.34,
            Self::Gas => 0.46,
            Self::Ocean => 0.27,
            Self::Ice => 0.19,
            Self::City => 0.23,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum View {
    Individual,
    System,
}

impl View {
    pub fn slug(self, body: Body) -> &'static str {
        match self {
            Self::System => "sistema",
            Self::Individual => body.slug(),
        }
    }
}

#[derive(Clone, Copy)]
pub struct Orbit {
    pub body: Body,
    pub radius: f32,
    pub period: f32,
    pub phase: f32,
}

pub const ORBITS: [Orbit; 5] = [
    Orbit {
        body: Body::City,
        radius: 0.20,
        period: 18.0,
        phase: -2.5,
    },
    Orbit {
        body: Body::Rocky,
        radius: 0.37,
        period: 28.0,
        phase: -0.65,
    },
    Orbit {
        body: Body::Ocean,
        radius: 0.56,
        period: 40.0,
        phase: 2.15,
    },
    Orbit {
        body: Body::Gas,
        radius: 0.74,
        period: 56.0,
        phase: 0.45,
    },
    Orbit {
        body: Body::Ice,
        radius: 0.92,
        period: 76.0,
        phase: 3.55,
    },
];

impl Orbit {
    // Coordenadas: X derecha, Y arriba, Z hacia la camara. Trayectoria circular
    // en un plano 3D inclinado: en pantalla se proyecta como una elipse.
    pub fn position(self, time: f32) -> Vec3 {
        let angle = self.phase + TAU * (time / self.period).rem_euclid(1.0);
        let (s, c) = angle.sin_cos();
        Vec3::new(c, s * 0.62, s * (1.0_f32 - 0.62 * 0.62).sqrt()) * self.radius
    }
}

pub const MOON_PERIOD: f32 = 7.0;

pub fn moon_offset(time: f32, planet_radius: f32) -> Vec3 {
    let angle = 0.7 + TAU * (time / MOON_PERIOD).rem_euclid(1.0);
    let (s, c) = angle.sin_cos();
    Vec3::new(c, s * 0.38, s * (1.0_f32 - 0.38 * 0.38).sqrt()) * (planet_radius * 2.15)
}

pub fn moon_center(planet_center: Vec3, time: f32, planet_radius: f32) -> Vec3 {
    planet_center + screen_offset(moon_offset(time, planet_radius))
}

fn screen_offset(world: Vec3) -> Vec3 {
    Vec3::new(world.x, -world.y, world.z)
}

pub fn system_center(width: usize, height: usize) -> Vec3 {
    Vec3::new(width as f32 * 0.5, height as f32 * 0.52, 0.0)
}

pub fn system_scale(width: usize, height: usize) -> f32 {
    (width as f32 * 0.44).min(height as f32 * 0.58)
}

pub struct Scene {
    pub sphere: Mesh,
    pub ring: Mesh,
    small_sphere: Mesh,
    collector: Mesh,
    energy_link: Mesh,
}

#[derive(Clone, Copy)]
pub struct Settings {
    pub body: Body,
    pub angle: f32,
    pub layers: u8,
    pub rings: bool,
    pub moon: bool,
    pub time: f32,
    pub orbits: bool,
    pub dyson: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            body: Body::Star,
            angle: 0.0,
            layers: 4,
            rings: true,
            moon: true,
            time: 0.0,
            orbits: true,
            dyson: true,
        }
    }
}

#[derive(Clone, Copy)]
struct Placement {
    center: Vec3,
    radius: f32,
    sun: Option<Vec3>,
}

struct Label {
    center: Vec3,
    radius: f32,
    text: &'static str,
    color: u32,
}

fn draw_labels(fb: &mut Framebuffer, labels: &[Label]) {
    let mut occupied: Vec<(i32, i32, i32, i32)> = Vec::new();
    for label in labels {
        let width = label.text.len() as i32 * 6 + 6;
        let height = 13;
        let x = label.center.x as i32;
        let y = label.center.y as i32;
        let r = label.radius.ceil() as i32;
        let candidates = [
            (x - width / 2, y + r + 9),
            (x - width / 2, y - r - height - 9),
            (x + r + 9, y - height / 2),
            (x - r - width - 9, y - height / 2),
            (x - width / 2, y + r + 30),
            (x - width / 2, y - r - height - 30),
        ];
        let clamp = |(a, b): (i32, i32)| {
            (
                a.clamp(8, (fb.width as i32 - width - 8).max(8)),
                b.clamp(75, (fb.height as i32 - 90 - height).max(75)),
            )
        };
        let fits = |(a, b): (i32, i32)| {
            let boxes_clear = occupied.iter().all(|&(ox, oy, ow, oh)| {
                a + width + 4 < ox || a > ox + ow + 4 || b + height + 4 < oy || b > oy + oh + 4
            });
            let bodies_clear = labels.iter().all(|body| {
                let px = body.center.x.clamp(a as f32, (a + width) as f32);
                let py = body.center.y.clamp(b as f32, (b + height) as f32);
                (px - body.center.x).powi(2) + (py - body.center.y).powi(2)
                    > (body.radius + 3.0).powi(2)
            });
            boxes_clear && bodies_clear
        };
        let (a, b) = candidates
            .into_iter()
            .map(clamp)
            .find(|&p| fits(p))
            .unwrap_or_else(|| clamp(candidates[0]));
        overlay::rect(fb, a, b, width, height, 0x0A101D);
        overlay::text(fb, a + 3, b + 3, label.text, 1, label.color);
        occupied.push((a, b, width, height));
    }
}

impl Placement {
    fn light_at(self, center: Vec3) -> Vec3 {
        match self.sun {
            Some(sun) => Vec3::new(sun.x - center.x, center.y - sun.y, sun.z - center.z),
            None => Vec3::new(-0.6, 0.7, 1.0),
        }
    }
}

impl Default for Scene {
    fn default() -> Self {
        Self::new()
    }
}

impl Scene {
    pub fn new() -> Self {
        Self {
            sphere: Mesh::sphere(72, 144),
            small_sphere: Mesh::sphere(28, 56),
            ring: Mesh::ring(256),
            collector: Mesh::collector_panel(),
            energy_link: Mesh::energy_link(),
        }
    }

    pub fn render(&self, fb: &mut Framebuffer, settings: Settings) {
        fb.clear_space();
        let center = Vec3::new(fb.width as f32 * 0.5, fb.height as f32 * 0.5, 0.0);
        let fraction = match settings.body {
            Body::Star if settings.dyson => 0.23,
            Body::Gas => 0.245,
            Body::Rocky if settings.moon => 0.27,
            _ => 0.32,
        };
        let radius = fb.height.min(fb.width) as f32 * fraction;
        self.draw_body(fb, settings, center, radius);
    }

    pub fn draw_body(&self, fb: &mut Framebuffer, settings: Settings, center: Vec3, radius: f32) {
        self.draw_instance(
            fb,
            settings,
            Placement {
                center,
                radius,
                sun: None,
            },
            &self.sphere,
        );
    }

    fn draw_instance(
        &self,
        fb: &mut Framebuffer,
        settings: Settings,
        placement: Placement,
        mesh: &Mesh,
    ) {
        let Placement { center, radius, .. } = placement;
        let transform = Transform {
            center,
            scale: radius,
            rotation_y: settings.angle + settings.time * settings.body.spin_rate(),
            rotation_x: 0.0,
            rotation_z: if settings.body == Body::Gas {
                -0.32
            } else {
                0.12
            },
        };
        let light = placement.light_at(center);
        draw_mesh_lit(
            fb,
            mesh,
            transform,
            settings.body.shader(),
            settings.layers,
            light,
        );
        if settings.body == Body::Star && settings.dyson {
            self.draw_dyson(fb, settings, center, radius);
        }
        if settings.body == Body::Gas && settings.rings {
            let transform = Transform {
                rotation_y: 0.0,
                rotation_x: 0.55,
                rotation_z: -0.32,
                ..transform
            };
            draw_mesh_lit(fb, &self.ring, transform, Shader::Ring, 4, light);
        }
        if settings.body == Body::Rocky && settings.moon {
            // Transformacion jerarquica: centro lunar = centro ACTUAL del planeta
            // + orbita local. La luna no orbita el origen del sistema.
            let moon_center = moon_center(center, settings.time, radius);
            let transform = Transform {
                center: moon_center,
                scale: radius * 0.23,
                rotation_y: settings.time * 0.7,
                rotation_z: 0.0,
                ..transform
            };
            draw_mesh_lit(
                fb,
                mesh,
                transform,
                Shader::Moon,
                4,
                placement.light_at(moon_center),
            );
        }
    }

    fn draw_dyson(&self, fb: &mut Framebuffer, settings: Settings, center: Vec3, radius: f32) {
        // Tres enjambres inclinados. Las placas orbitan como objetos separados
        // y el z-buffer decide cuales quedan delante y detras del Sol.
        for (band, (tilt_x, tilt_z)) in [(0.45, -0.35), (1.05, 0.65), (1.5, 1.9)]
            .into_iter()
            .enumerate()
        {
            for index in 0..12 {
                let angle = TAU * index as f32 / 12.0
                    + band as f32 * 0.31
                    + settings.time * if band == 1 { -0.16 } else { 0.13 };
                let orbit = Transform {
                    center,
                    scale: radius,
                    rotation_y: angle,
                    rotation_x: tilt_x,
                    rotation_z: tilt_z,
                };
                let offset = orbit.orient(Vec3::new(0.0, 0.0, 1.57)) * radius;
                let panel = Transform {
                    center: center + screen_offset(offset),
                    ..orbit
                };
                draw_mesh_lit(
                    fb,
                    &self.collector,
                    panel,
                    Shader::Collector,
                    4,
                    Vec3::new(0.0, 0.0, 1.0),
                );
                if index % 4 == band {
                    draw_mesh_lit(
                        fb,
                        &self.energy_link,
                        orbit,
                        Shader::Energy,
                        4,
                        Vec3::new(0.0, 0.0, 1.0),
                    );
                }
            }
        }
    }

    pub fn render_system(&self, fb: &mut Framebuffer, settings: Settings) {
        fb.clear_space();
        let sun = system_center(fb.width, fb.height);
        let scale = system_scale(fb.width, fb.height);
        let unit = fb.width.min(fb.height) as f32;
        let sun_radius = unit * if settings.dyson { 0.057 } else { 0.067 };
        if settings.orbits {
            for orbit in ORBITS {
                let mut previous = sun + screen_offset(orbit.position(0.0) * scale);
                for step in 1..=360 {
                    let point = sun
                        + screen_offset(orbit.position(step as f32 / 360.0 * orbit.period) * scale);
                    overlay::line(fb, (previous.x, previous.y), (point.x, point.y), 0x273348);
                    previous = point;
                }
            }
        }
        // Las guias se dibujan antes que los cuerpos para no tapar superficies.
        if settings.moon && settings.orbits {
            let parent = ORBITS[1];
            let center = sun + screen_offset(parent.position(settings.time) * scale);
            let radius = unit * 0.029;
            let mut previous = center + screen_offset(moon_offset(0.0, radius));
            for step in 1..=100 {
                let point =
                    center + screen_offset(moon_offset(step as f32 / 100.0 * MOON_PERIOD, radius));
                overlay::line(fb, (previous.x, previous.y), (point.x, point.y), 0x405066);
                previous = point;
            }
        }
        self.draw_instance(
            fb,
            Settings {
                body: Body::Star,
                ..settings
            },
            Placement {
                center: sun,
                radius: sun_radius,
                sun: None,
            },
            &self.small_sphere,
        );
        let mut labels = vec![Label {
            center: sun,
            radius: sun_radius * if settings.dyson { 1.65 } else { 1.0 },
            text: "ASTER",
            color: 0xEEC17B,
        }];
        for orbit in ORBITS {
            let center = sun + screen_offset(orbit.position(settings.time) * scale);
            let size = match orbit.body {
                Body::City => 0.029,
                Body::Rocky => 0.029,
                Body::Ocean => 0.034,
                Body::Gas => 0.039,
                Body::Ice => 0.032,
                Body::Star => unreachable!(),
            };
            let radius = unit * size;
            self.draw_instance(
                fb,
                Settings {
                    body: orbit.body,
                    ..settings
                },
                Placement {
                    center,
                    radius,
                    sun: Some(sun),
                },
                &self.small_sphere,
            );
            let envelope = if orbit.body == Body::Gas && settings.rings {
                radius * 2.03
            } else {
                radius
            };
            labels.push(Label {
                center,
                radius: envelope,
                text: orbit.body.short_name(),
                color: 0xBCCADA,
            });
            if orbit.body == Body::Rocky && settings.moon {
                let moon = moon_center(center, settings.time, radius);
                labels.push(Label {
                    center: moon,
                    radius: radius * 0.23,
                    text: "LUNA",
                    color: 0x8996AA,
                });
            }
        }
        draw_labels(fb, &labels);
    }

    pub fn render_view(
        &self,
        fb: &mut Framebuffer,
        settings: Settings,
        view: View,
        animate: bool,
        speed: f32,
    ) {
        match view {
            View::Individual => self.render(fb, settings),
            View::System => self.render_system(fb, settings),
        }
        let title = match view {
            View::System => "ASTER / SISTEMA SOLAR",
            View::Individual => settings.body.name(),
        };
        overlay::text(fb, 26, 25, title, 2, 0xE4EAF3);
        let subtitle = if view == View::System {
            "UNA ESTRELLA / CINCO PLANETAS / UNA LUNA"
        } else {
            "SHADER PROCEDURAL / SIN TEXTURAS NI MATERIALES"
        };
        overlay::text(fb, 27, 50, subtitle, 1, 0x8194AD);
        let status = format!(
            "{} | X{speed:.2} | T {:.1} S | CAPAS {}/4",
            if animate { "EN MARCHA" } else { "PAUSA" },
            settings.time,
            settings.layers
        );
        let bottom = fb.height as i32 - 65;
        overlay::rect(fb, 0, bottom - 12, fb.width as i32, 77, 0x090F1C);
        overlay::line(
            fb,
            (26.0, (bottom - 12) as f32),
            (fb.width as f32 - 26.0, (bottom - 12) as f32),
            0x26364E,
        );
        overlay::text(
            fb,
            26,
            bottom,
            "0 SISTEMA | 1 ASTER | 2 IGNIS | 3 ZEPHIR | 4 PELAGIA | 5 NIVIS | 6 CORUSCANT",
            1,
            0xC2CDDC,
        );
        overlay::text(
            fb,
            26,
            bottom + 19,
            "ESPACIO PAUSA | +/- VELOCIDAD | T REINICIAR | O ORBITAS | L CAPAS | R ANILLOS | M LUNA | D DYSON | S CAPTURA",
            1,
            0x8194AD,
        );
        overlay::text(fb, 26, bottom + 38, &status, 1, 0xE0B777);
    }

    pub fn gallery(&self, fb: &mut Framebuffer) {
        fb.clear_space();
        overlay::text(
            fb,
            28,
            24,
            "ATLAS / UNA ESTRELLA Y CINCO PLANETAS",
            2,
            0xDFE8F4,
        );
        let cell = fb.width as f32 / 3.0;
        let row = (fb.height as f32 - 65.0) / 2.0;
        for (i, body) in Body::ALL.into_iter().enumerate() {
            let center = Vec3::new(
                cell * ((i % 3) as f32 + 0.5),
                65.0 + row * ((i / 3) as f32 + 0.5),
                0.0,
            );
            let fraction = match body {
                Body::Star => 0.23,
                Body::Gas => 0.22,
                Body::Rocky => 0.20,
                _ => 0.32,
            };
            let radius = cell.min(row) * fraction;
            self.draw_body(
                fb,
                Settings {
                    body,
                    ..Settings::default()
                },
                center,
                radius,
            );
            let label = format!("{} / {}", i + 1, body.short_name());
            overlay::text(
                fb,
                center.x as i32 - label.len() as i32 * 6,
                (65.0 + row * (i / 3) as f32 + 20.0) as i32,
                &label,
                2,
                0xBCCADA,
            );
        }
    }
}

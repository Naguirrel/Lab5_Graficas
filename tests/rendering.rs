use lab5_shaders::{
    framebuffer::Framebuffer,
    geometry::Mesh,
    math::{Vec3, pack},
    noise::fbm,
    renderer::{Transform, draw_mesh},
    scene::{Body, Scene, Settings},
    shaders::Shader,
};

#[test]
fn fast_key_taps_are_preserved_without_operating_system_repeats() {
    use lab5_shaders::input::{KeyQueue, Keyboard};
    use minifb::{InputCallback, Key};
    let queue = KeyQueue::default();
    let mut input = Keyboard::new(queue.clone());
    input.set_key_state(Key::Key2, true);
    input.add_char('2' as u32);
    input.set_key_state(Key::Key2, true);
    input.set_key_state(Key::Key2, false);
    assert_eq!(*queue.borrow(), vec![Key::Key2]);
    queue.borrow_mut().clear();
    input.add_char('3' as u32);
    assert_eq!(*queue.borrow(), vec![Key::Key3]);
    queue.borrow_mut().clear();
    input.set_key_state(Key::Key2, true);
    input.set_key_state(Key::Key2, false);
    assert_eq!(*queue.borrow(), vec![Key::Key2]);
}

#[test]
fn depth_buffer_keeps_nearer_surface_independent_of_order() {
    for depths in [[-2.0, 3.0], [3.0, -2.0]] {
        let mut fb = Framebuffer::new(1, 1);
        for depth in depths {
            fb.point(0, depth, if depth > 0.0 { 0xABCDEF } else { 0x123456 });
        }
        assert_eq!(fb.pixels[0], 0xABCDEF);
    }
}

#[test]
fn noise_is_continuous_at_cell_boundaries_and_in_range() {
    for i in -10..10 {
        let p = Vec3::new(i as f32, 0.37, -1.2);
        let a = fbm(p - Vec3::new(0.00001, 0.0, 0.0), 4);
        let b = fbm(p + Vec3::new(0.00001, 0.0, 0.0), 4);
        assert!((0.0..=1.0).contains(&a));
        assert!((a - b).abs() < 0.001);
    }
}

#[test]
fn rings_occlude_and_are_occluded_independent_of_draw_order() {
    let sphere = Mesh::sphere(24, 48);
    let ring = Mesh::ring(96);
    let transform = Transform {
        center: Vec3::new(80.0, 80.0, 0.0),
        scale: 35.0,
        rotation_x: 0.0,
        rotation_y: 0.0,
        rotation_z: -0.32,
    };
    let ring_transform = Transform {
        rotation_x: 0.55,
        ..transform
    };
    let mut outputs = Vec::new();
    for reverse in [false, true] {
        let mut fb = Framebuffer::new(160, 160);
        fb.clear_space();
        if reverse {
            draw_mesh(&mut fb, &ring, ring_transform, Shader::Ring, 4);
        }
        draw_mesh(&mut fb, &sphere, transform, Shader::Gas, 4);
        if !reverse {
            draw_mesh(&mut fb, &ring, ring_transform, Shader::Ring, 4);
        }
        outputs.push(fb.pixels);
    }
    assert_eq!(outputs[0], outputs[1]);
    // Ademas de igualdad, debe haber oclusion en ambos sentidos: si el anillo
    // estuviera de frente sin cruzar la esfera, la igualdad sola no lo detecta.
    let mut background = Framebuffer::new(160, 160);
    background.clear_space();
    let mut only_sphere = Framebuffer::new(160, 160);
    only_sphere.clear_space();
    draw_mesh(&mut only_sphere, &sphere, transform, Shader::Gas, 4);
    let mut only_ring = Framebuffer::new(160, 160);
    only_ring.clear_space();
    draw_mesh(&mut only_ring, &ring, ring_transform, Shader::Ring, 4);
    let (mut ring_in_front, mut sphere_in_front) = (0, 0);
    for (i, &bg) in background.pixels.iter().enumerate() {
        let s = only_sphere.pixels[i];
        let r = only_ring.pixels[i];
        if s != bg && r != bg && s != r {
            ring_in_front += usize::from(outputs[0][i] == r);
            sphere_in_front += usize::from(outputs[0][i] == s);
        }
    }
    assert!(ring_in_front > 20 && sphere_in_front > 20);
}

#[test]
fn every_layer_changes_each_body_and_full_render_is_not_flat() {
    let scene = Scene::new();
    for body in Body::ALL {
        let mut previous = Vec::new();
        for layers in 1..=4 {
            let mut fb = Framebuffer::new(180, 140);
            scene.render(
                &mut fb,
                Settings {
                    body,
                    layers,
                    dyson: false,
                    ..Settings::default()
                },
            );
            if layers > 1 {
                let changed = fb
                    .pixels
                    .iter()
                    .zip(&previous)
                    .filter(|(a, b)| a != b)
                    .count();
                assert!(
                    changed > 100,
                    "{body:?}: capa {layers} no cambia suficientes pixeles"
                );
            }
            previous = fb.pixels;
        }
        let unique: std::collections::HashSet<_> = previous.into_iter().collect();
        assert!(unique.len() > 500, "{body:?}: imagen casi plana");
    }
}

#[test]
fn shaders_remain_finite_at_poles_and_multiple_rotations() {
    use lab5_shaders::shaders::{Fragment, shade};
    for shader in [
        Shader::Star,
        Shader::Rocky,
        Shader::Gas,
        Shader::Ocean,
        Shader::Ice,
        Shader::City,
        Shader::Moon,
    ] {
        for p in [
            Vec3::new(0.0, 1.0, 0.0),
            Vec3::new(0.0, -1.0, 0.0),
            Vec3::new(0.0, 0.0, 1.0),
        ] {
            for angle in [0.0, 1.0, 3.0] {
                let color = shade(
                    shader,
                    Fragment {
                        position: p,
                        normal: p.rotate_y(angle),
                    },
                    4,
                )
                .unwrap();
                assert!(color.x.is_finite() && color.y.is_finite() && color.z.is_finite());
                let _ = pack(color);
            }
        }
    }
}

#[test]
fn collector_panels_render_both_sides_and_obey_the_depth_buffer() {
    let sphere = Mesh::sphere(24, 48);
    let panel = Mesh::collector_panel();
    let transform = Transform {
        center: Vec3::new(50.0, 50.0, 0.0),
        scale: 32.0,
        rotation_x: 0.0,
        rotation_y: 0.0,
        rotation_z: 0.0,
    };
    let mut star = Framebuffer::new(100, 100);
    draw_mesh(&mut star, &sphere, transform, Shader::Star, 4);
    let center_pixel = star.pixels[50 * 100 + 50];
    for angle in [0.0, std::f32::consts::PI] {
        for depth in [-40.0, 40.0] {
            let panel_transform = Transform {
                center: Vec3::new(50.0, 50.0, depth),
                rotation_y: angle,
                ..transform
            };
            let mut images = Vec::new();
            for panel_first in [true, false] {
                let mut fb = Framebuffer::new(100, 100);
                if panel_first {
                    draw_mesh(&mut fb, &panel, panel_transform, Shader::Collector, 4);
                }
                draw_mesh(&mut fb, &sphere, transform, Shader::Star, 4);
                if !panel_first {
                    draw_mesh(&mut fb, &panel, panel_transform, Shader::Collector, 4);
                }
                images.push(fb.pixels);
            }
            assert_eq!(images[0], images[1]);
            assert_eq!(images[0][50 * 100 + 50] == center_pixel, depth < 0.0);
        }
    }
}

#[test]
fn five_planet_orbits_are_closed_distinct_and_constant_in_radius() {
    use lab5_shaders::scene::ORBITS;
    assert_eq!(ORBITS.len(), 5);
    let planets: std::collections::HashSet<_> = ORBITS.iter().map(|o| o.body.slug()).collect();
    assert_eq!(planets.len(), 5);
    assert!(!ORBITS.iter().any(|o| o.body == Body::Star));
    for orbit in ORBITS {
        assert!((orbit.position(0.0) - orbit.position(orbit.period)).length() < 1e-5);
        assert!(
            (orbit.position(0.0) - orbit.position(orbit.period * 0.5)).length()
                > orbit.radius * 1.99
        );
        for step in 0..100 {
            let time = step as f32 * 0.37;
            assert!((orbit.position(time).length() - orbit.radius).abs() < 1e-5);
        }
    }
}

#[test]
fn moon_follows_moving_parent_and_completes_its_own_orbit() {
    use lab5_shaders::scene::{MOON_PERIOD, ORBITS, moon_center};
    let parent = ORBITS.iter().find(|o| o.body == Body::Rocky).unwrap();
    let start = parent.position(0.0) * 400.0;
    let end = parent.position(MOON_PERIOD) * 400.0;
    let moon_start = moon_center(start, 0.0, 24.0);
    let moon_end = moon_center(end, MOON_PERIOD, 24.0);
    // Un periodo lunar cierra la orbita LOCAL mientras el padre sigue viajando.
    assert!((start - end).length() > 100.0);
    assert!(((moon_end - moon_start) - (end - start)).length() < 0.001);
    for step in 0..200 {
        let time = step as f32 * 0.19;
        let center = parent.position(time) * 400.0;
        assert!(((moon_center(center, time, 24.0) - center).length() - 24.0 * 2.15).abs() < 0.001);
    }
}

#[test]
fn system_frames_are_deterministic_and_advance_with_time() {
    let scene = Scene::new();
    let mut a = Framebuffer::new(480, 340);
    let mut repeat = Framebuffer::new(480, 340);
    let mut b = Framebuffer::new(480, 340);
    scene.render_system(&mut a, Settings::default());
    scene.render_system(&mut repeat, Settings::default());
    scene.render_system(
        &mut b,
        Settings {
            time: 8.0,
            ..Settings::default()
        },
    );
    assert_eq!(a.pixels, repeat.pixels);
    let changed = a
        .pixels
        .iter()
        .zip(&b.pixels)
        .filter(|(x, y)| x != y)
        .count();
    assert!(changed > 1000);
}

#[test]
fn external_light_changes_the_day_side_but_does_not_dim_the_star() {
    use lab5_shaders::shaders::{Fragment, shade_with_light};
    let fragment = Fragment {
        position: Vec3::new(0.0, 0.0, 1.0),
        normal: Vec3::new(0.0, 0.0, 1.0),
    };
    let front = Vec3::new(0.0, 0.0, 1.0);
    let back = Vec3::new(0.0, 0.0, -1.0);
    let day = shade_with_light(Shader::Ocean, fragment, 1, front).unwrap();
    let night = shade_with_light(Shader::Ocean, fragment, 1, back).unwrap();
    assert!(day.length() > night.length() * 3.0);
    assert_eq!(
        pack(shade_with_light(Shader::Star, fragment, 4, front).unwrap()),
        pack(shade_with_light(Shader::Star, fragment, 4, back).unwrap())
    );
}

#[test]
fn city_windows_and_avenues_remain_bright_on_the_night_side() {
    use lab5_shaders::shaders::{Fragment, shade_with_light};
    let (mut buildings, mut illuminated) = (0.0, 0.0);
    for row in -8..=8 {
        for col in -8..=8 {
            let (x, y) = (col as f32 / 10.0, row as f32 / 10.0);
            if x * x + y * y >= 0.90 {
                continue;
            }
            let p = Vec3::new(x, y, (1.0 - x * x - y * y).sqrt());
            let fragment = Fragment {
                position: p,
                normal: p,
            };
            let light = Vec3::new(0.0, 0.0, -1.0);
            buildings += shade_with_light(Shader::City, fragment, 2, light)
                .unwrap()
                .length();
            illuminated += shade_with_light(Shader::City, fragment, 4, light)
                .unwrap()
                .length();
        }
    }
    assert!(illuminated > buildings * 1.5);
}

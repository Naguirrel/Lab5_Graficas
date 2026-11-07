use lab5_shaders::{
    framebuffer::Framebuffer,
    input::{KeyQueue, Keyboard},
    scene::{Body, Scene, Settings, View},
};
use minifb::{Key, Window, WindowOptions};
use std::{env, path::PathBuf, time::Instant};

const WIDTH: usize = 1120;
const HEIGHT: usize = 800;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut settings = Settings::default();
    let mut view = View::System;
    let mut animate = true;
    let mut speed = 1.0_f32;
    let mut export_only = false;
    let mut export_all = false;
    let mut output = None;
    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--body" => {
                settings.body = match args.next().as_deref() {
                    Some("estrella") => Body::Star,
                    Some("rocoso") => Body::Rocky,
                    Some("gaseoso") => Body::Gas,
                    Some("oceanico") => Body::Ocean,
                    Some("helado") => Body::Ice,
                    Some("desertico") => Body::Desert,
                    _ => {
                        return Err(
                            "usa --body estrella|rocoso|gaseoso|oceanico|helado|desertico".into(),
                        );
                    }
                };
                view = View::Individual;
            }
            "--view" => {
                view = match args.next().as_deref() {
                    Some("sistema") => View::System,
                    Some("individual") => View::Individual,
                    _ => return Err("usa --view sistema|individual".into()),
                };
            }
            "--time" => {
                settings.time = args.next().ok_or("falta tiempo en segundos")?.parse()?;
                if !settings.time.is_finite() || !(0.0..=1_000_000.0).contains(&settings.time) {
                    return Err("el tiempo debe estar entre 0 y 1000000 segundos".into());
                }
            }
            "--speed" => {
                speed = args.next().ok_or("falta velocidad")?.parse()?;
                if !speed.is_finite() || !(0.25..=4.0).contains(&speed) {
                    return Err("la velocidad debe estar entre 0.25 y 4".into());
                }
            }
            "--paused" => animate = false,
            "--no-orbits" => settings.orbits = false,
            "--no-dyson" => settings.dyson = false,
            "--layers" => {
                settings.layers = args.next().ok_or("falta numero de capas")?.parse()?;
                if !(1..=4).contains(&settings.layers) {
                    return Err("capas debe estar entre 1 y 4".into());
                }
            }
            "--angle" => {
                settings.angle = args.next().ok_or("falta angulo en radianes")?.parse()?;
                if !settings.angle.is_finite() {
                    return Err("el angulo debe ser finito".into());
                }
            }
            "--no-rings" => settings.rings = false,
            "--no-moon" => settings.moon = false,
            "--output" => output = Some(PathBuf::from(args.next().ok_or("falta ruta PNG")?)),
            "--export-only" => export_only = true,
            "--export-all" => {
                export_all = true;
                export_only = true;
            }
            "--help" | "-h" => {
                println!(
                    "Lab 5 - Shaders por software\n\
                    cargo run --release -- [opciones]\n\
                    --view sistema|individual (predeterminado: sistema animado)\n\
                    --body estrella|rocoso|gaseoso|oceanico|helado|desertico\n\
                    --time SEG / --speed 0.25..4 / --paused / --no-orbits\n\
                    --layers 1|2|3|4      Capas acumuladas del shader\n\
                    --angle RAD           Rotacion inicial\n\
                    --no-rings / --no-moon / --no-dyson\n\
                    --output ruta.png     Guarda la vista inicial\n\
                    --export-only         Exporta sin abrir ventana\n\
                    --export-all          Exporta seis cuerpos, comparativa y sistema\n\
                    Ventana: 0 sistema, 1..6 cuerpo, L capas, Espacio pausa,\n\
                    +/- velocidad, T reiniciar, O orbitas, flechas girar/avanzar,\n\
                    R anillos, M luna, D placas Dyson, S/F12 captura, Esc salir."
                );
                return Ok(());
            }
            _ => return Err(format!("opcion desconocida: {arg}; usa --help").into()),
        }
    }

    let scene = Scene::new();
    let mut fb = Framebuffer::new(WIDTH, HEIGHT);
    if export_all {
        for body in Body::ALL {
            scene.render_view(
                &mut fb,
                Settings { body, ..settings },
                View::Individual,
                false,
                speed,
            );
            let path = PathBuf::from(format!("screenshots/{}.png", body.slug()));
            fb.save_png(&path)?;
            println!("Guardado: {}", path.display());
        }
        let mut gallery = Framebuffer::new(1800, 1200);
        scene.gallery(&mut gallery);
        gallery.save_png(&PathBuf::from("screenshots/comparativa.png"))?;
        println!("Guardado: screenshots/comparativa.png");
        scene.render_view(&mut fb, settings, View::System, false, speed);
        fb.save_png(&PathBuf::from("screenshots/sistema.png"))?;
        println!("Guardado: screenshots/sistema.png");
        return Ok(());
    }
    scene.render_view(&mut fb, settings, view, animate && !export_only, speed);
    if output.is_some() || export_only {
        let path = output.unwrap_or_else(|| {
            PathBuf::from(format!("screenshots/{}.png", view.slug(settings.body)))
        });
        fb.save_png(&path)?;
        println!("Guardado: {}", path.display());
    }
    if export_only {
        return Ok(());
    }

    println!(
        "0: sistema | 1: estrella | 2: rocoso | 3: gaseoso | 4: oceanico | 5: helado | 6: desertico"
    );
    println!("Espacio: pausa | +/-: velocidad | T: reiniciar | O: orbitas | L: capas");
    println!(
        "Flechas: girar/avanzar | R: anillos | M: luna | D: Dyson | S/F12: captura | Esc: salir"
    );
    let mut window = Window::new("Lab 5 - Shaders", WIDTH, HEIGHT, WindowOptions::default())?;
    window.set_target_fps(30);
    let key_queue = KeyQueue::default();
    window.set_input_callback(Box::new(Keyboard::new(key_queue.clone())));
    let mut last_frame = Instant::now();
    let mut dirty = true;
    while window.is_open() && !window.is_key_down(Key::Escape) {
        let pressed = std::mem::take(&mut *key_queue.borrow_mut());
        if pressed.contains(&Key::Escape) {
            break;
        }
        let now = Instant::now();
        let elapsed = (now - last_frame).as_secs_f32();
        last_frame = now;
        for (key, body) in [
            (Key::Key1, Body::Star),
            (Key::Key2, Body::Rocky),
            (Key::Key3, Body::Gas),
            (Key::Key4, Body::Ocean),
            (Key::Key5, Body::Ice),
            (Key::Key6, Body::Desert),
        ] {
            if pressed.contains(&key) {
                settings.body = body;
                view = View::Individual;
                dirty = true;
            }
        }
        if pressed.contains(&Key::Key0) {
            view = View::System;
            dirty = true;
        }
        if pressed.contains(&Key::O) {
            settings.orbits = !settings.orbits;
            dirty = true;
        }
        if pressed.contains(&Key::D) {
            settings.dyson = !settings.dyson;
            dirty = true;
        }
        if pressed.contains(&Key::T) {
            settings.time = 0.0;
            settings.angle = 0.0;
            dirty = true;
        }
        if pressed.contains(&Key::Equal) || pressed.contains(&Key::NumPadPlus) {
            speed = (speed * 2.0).min(4.0);
            dirty = true;
        }
        if pressed.contains(&Key::Minus) || pressed.contains(&Key::NumPadMinus) {
            speed = (speed * 0.5).max(0.25);
            dirty = true;
        }
        if pressed.contains(&Key::L) {
            settings.layers = settings.layers % 4 + 1;
            dirty = true;
        }
        if pressed.contains(&Key::R) {
            settings.rings = !settings.rings;
            dirty = true;
        }
        if pressed.contains(&Key::M) {
            settings.moon = !settings.moon;
            dirty = true;
        }
        if pressed.contains(&Key::Space) {
            animate = !animate;
            dirty = true;
        }
        let turn =
            i32::from(window.is_key_down(Key::Right)) - i32::from(window.is_key_down(Key::Left));
        let tap =
            i32::from(pressed.contains(&Key::Right)) - i32::from(pressed.contains(&Key::Left));
        if tap != 0 {
            if view == View::System {
                settings.time = (settings.time + tap as f32).max(0.0);
            } else {
                settings.angle += tap as f32 * 0.08;
            }
            dirty = true;
        }
        if turn != 0 {
            if view == View::System {
                settings.time = (settings.time + elapsed * turn as f32 * 4.0).max(0.0);
            } else {
                settings.angle += elapsed * turn as f32 * 0.8;
            }
            dirty = true;
        }
        if animate {
            settings.time += elapsed * speed;
            dirty = true;
        }
        if dirty {
            scene.render_view(&mut fb, settings, view, animate, speed);
            window.set_title(&format!(
                "Lab 5 | {} | {} | x{speed:.2} | Capas {}/4 | 0 sistema - 1..6 cuerpo",
                if view == View::System {
                    "Sistema solar"
                } else {
                    settings.body.name()
                },
                if animate { "Animando" } else { "Pausa" },
                settings.layers
            ));
            dirty = false;
        }
        if pressed.contains(&Key::F12) || pressed.contains(&Key::S) {
            let path = PathBuf::from(format!(
                "screenshots/captura_{}.png",
                view.slug(settings.body)
            ));
            fb.save_png(&path)?;
            println!("Captura: {}", path.display());
        }
        window.update_with_buffer(&fb.pixels, WIDTH, HEIGHT)?;
    }
    Ok(())
}

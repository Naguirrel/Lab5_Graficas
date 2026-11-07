//! Rotulos y guias dibujados con pixeles; la fuente tambien es codigo.
use crate::framebuffer::Framebuffer;

pub fn pixel(fb: &mut Framebuffer, x: i32, y: i32, color: u32) {
    if x >= 0 && y >= 0 && x < fb.width as i32 && y < fb.height as i32 {
        fb.pixels[y as usize * fb.width + x as usize] = color;
    }
}

pub fn line(fb: &mut Framebuffer, a: (f32, f32), b: (f32, f32), color: u32) {
    let steps = (b.0 - a.0).abs().max((b.1 - a.1).abs()).ceil() as usize;
    for i in 0..=steps {
        let t = i as f32 / steps.max(1) as f32;
        pixel(
            fb,
            (a.0 + (b.0 - a.0) * t).round() as i32,
            (a.1 + (b.1 - a.1) * t).round() as i32,
            color,
        );
    }
}

pub fn rect(fb: &mut Framebuffer, x: i32, y: i32, width: i32, height: i32, color: u32) {
    for row in y.max(0)..(y + height).min(fb.height as i32) {
        for col in x.max(0)..(x + width).min(fb.width as i32) {
            pixel(fb, col, row, color);
        }
    }
}

pub fn text(fb: &mut Framebuffer, x: i32, y: i32, value: &str, scale: i32, color: u32) {
    for (i, c) in value.to_ascii_uppercase().chars().enumerate() {
        for (row, bits) in glyph(c).into_iter().enumerate() {
            for col in 0..5 {
                if bits & (1 << (4 - col)) != 0 {
                    rect(
                        fb,
                        x + (i as i32 * 6 + col) * scale,
                        y + row as i32 * scale,
                        scale,
                        scale,
                        color,
                    );
                }
            }
        }
    }
}

fn glyph(c: char) -> [u8; 7] {
    match c {
        'A' => [14, 17, 17, 31, 17, 17, 17],
        'B' => [30, 17, 17, 30, 17, 17, 30],
        'C' => [14, 17, 16, 16, 16, 17, 14],
        'D' => [30, 17, 17, 17, 17, 17, 30],
        'E' => [31, 16, 16, 30, 16, 16, 31],
        'F' => [31, 16, 16, 30, 16, 16, 16],
        'G' => [14, 17, 16, 23, 17, 17, 15],
        'H' => [17, 17, 17, 31, 17, 17, 17],
        'I' => [31, 4, 4, 4, 4, 4, 31],
        'J' => [7, 2, 2, 2, 18, 18, 12],
        'K' => [17, 18, 20, 24, 20, 18, 17],
        'L' => [16, 16, 16, 16, 16, 16, 31],
        'M' => [17, 27, 21, 21, 17, 17, 17],
        'N' => [17, 25, 21, 19, 17, 17, 17],
        'O' => [14, 17, 17, 17, 17, 17, 14],
        'P' => [30, 17, 17, 30, 16, 16, 16],
        'Q' => [14, 17, 17, 17, 21, 18, 13],
        'R' => [30, 17, 17, 30, 20, 18, 17],
        'S' => [15, 16, 16, 14, 1, 1, 30],
        'T' => [31, 4, 4, 4, 4, 4, 4],
        'U' => [17, 17, 17, 17, 17, 17, 14],
        'V' => [17, 17, 17, 17, 17, 10, 4],
        'W' => [17, 17, 17, 21, 21, 21, 10],
        'X' => [17, 17, 10, 4, 10, 17, 17],
        'Y' => [17, 17, 10, 4, 4, 4, 4],
        'Z' => [31, 1, 2, 4, 8, 16, 31],
        '0' => [14, 17, 19, 21, 25, 17, 14],
        '1' => [4, 12, 4, 4, 4, 4, 14],
        '2' => [14, 17, 1, 2, 4, 8, 31],
        '3' => [30, 1, 1, 14, 1, 1, 30],
        '4' => [2, 6, 10, 18, 31, 2, 2],
        '5' => [31, 16, 16, 30, 1, 1, 30],
        '6' => [14, 16, 16, 30, 17, 17, 14],
        '7' => [31, 1, 2, 4, 8, 8, 8],
        '8' => [14, 17, 17, 14, 17, 17, 14],
        '9' => [14, 17, 17, 15, 1, 1, 14],
        '-' => [0, 0, 0, 31, 0, 0, 0],
        '+' => [0, 4, 4, 31, 4, 4, 0],
        ':' => [0, 4, 4, 0, 4, 4, 0],
        '.' => [0, 0, 0, 0, 0, 4, 4],
        '/' => [1, 2, 2, 4, 8, 8, 16],
        '|' => [4, 4, 4, 4, 4, 4, 4],
        '[' => [14, 8, 8, 8, 8, 8, 14],
        ']' => [14, 2, 2, 2, 2, 2, 14],
        _ => [0; 7],
    }
}

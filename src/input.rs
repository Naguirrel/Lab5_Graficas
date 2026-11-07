use minifb::{InputCallback, Key};
use std::{cell::RefCell, rc::Rc};

pub type KeyQueue = Rc<RefCell<Vec<Key>>>;

pub struct Keyboard {
    queue: KeyQueue,
    held: [bool; 512],
}

impl Keyboard {
    pub fn new(queue: KeyQueue) -> Self {
        Self {
            queue,
            held: [false; 512],
        }
    }
}

impl InputCallback for Keyboard {
    fn add_char(&mut self, character: u32) {
        // Algunos teclados virtuales envian texto sin codigo de tecla fisico.
        // Admitimos ambas rutas sin duplicar un evento del teclado normal.
        let key = match char::from_u32(character).map(|c| c.to_ascii_lowercase()) {
            Some('0') => Key::Key0,
            Some('1') => Key::Key1,
            Some('2') => Key::Key2,
            Some('3') => Key::Key3,
            Some('4') => Key::Key4,
            Some('5') => Key::Key5,
            Some('6') => Key::Key6,
            Some('o') => Key::O,
            Some('d') => Key::D,
            Some('t') => Key::T,
            Some('+') | Some('=') => Key::Equal,
            Some('-') => Key::Minus,
            Some('l') => Key::L,
            Some('r') => Key::R,
            Some('m') => Key::M,
            Some('s') => Key::S,
            Some(' ') => Key::Space,
            _ => return,
        };
        if !self.held[key as usize] && !self.queue.borrow().contains(&key) {
            self.queue.borrow_mut().push(key);
        }
    }

    fn set_key_state(&mut self, key: Key, down: bool) {
        let index = key as usize;
        // Conserva la pulsacion aunque el usuario suelte la tecla antes del
        // siguiente cuadro. Un render costoso no debe perder los controles.
        if down && !self.held[index] {
            self.queue.borrow_mut().push(key);
        }
        self.held[index] = down;
    }
}

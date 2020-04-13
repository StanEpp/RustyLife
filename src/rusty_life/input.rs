extern crate sdl2;

use sdl2::event::Event as sdlEvent;
use sdl2::keyboard::*;
use sdl2::mouse::*;

const KEY_MAP_SIZE : usize = 32;

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum Key {
    W,
    A,
    S,
    D,
    R,
    N,
    F,
    I,
    LSHIFT,
    RSHIFT,
    SPACE,
    ESC,
    NumPLUS,
    NumMINUS,
    MouseLeftButton,
    MouseRightButton,
    MouseWheelUp,
    MouseWheelDown,
    NONE,
}

#[derive(Debug, Copy, Clone)]
pub struct InputMap {
    pub keys_pressed : [bool; KEY_MAP_SIZE],
    pub keys_hold : [bool; KEY_MAP_SIZE],
    pub mouse_x_dt : i32,
    pub mouse_y_dt : i32,
}

pub struct Input {
    event_pump : sdl2::EventPump,
    input_map : InputMap,
}

impl Input {
    pub fn new(sdl_context : &sdl2::Sdl) -> Input {
        let event_pump = sdl_context.event_pump().unwrap();

        Input {
            event_pump : event_pump,
            input_map : InputMap{
                keys_pressed : [false; KEY_MAP_SIZE],
                keys_hold : [false; KEY_MAP_SIZE],
                mouse_x_dt : 0,
                mouse_y_dt : 0,
            },
        }
    }

    pub fn get_input_map(self : &Self) -> InputMap {
        self.input_map.clone()
    }

    pub fn update_input(self : &mut Self) {
        self.input_map.keys_pressed = [false; KEY_MAP_SIZE];

        for event in self.event_pump.poll_iter() {
            match event {
                sdlEvent::KeyDown {keycode : Some(k), ..} => {
                    match k {
                        Keycode::W => {
                            self.input_map.keys_pressed[Key::W] = !self.input_map.keys_hold[Key::W];
                            self.input_map.keys_hold[Key::W] = true;
                        },
                        Keycode::A => {
                            self.input_map.keys_pressed[Key::A] = !self.input_map.keys_hold[Key::A];
                            self.input_map.keys_hold[Key::A] = true;
                        },
                        Keycode::S => {
                            self.input_map.keys_pressed[Key::S] = !self.input_map.keys_hold[Key::S];
                            self.input_map.keys_hold[Key::S] = true;
                        },
                        Keycode::D => {
                            self.input_map.keys_pressed[Key::D] = !self.input_map.keys_hold[Key::D];
                            self.input_map.keys_hold[Key::D] = true;
                        },
                        Keycode::R => {
                            self.input_map.keys_pressed[Key::R] = !self.input_map.keys_hold[Key::R];
                            self.input_map.keys_hold[Key::R] = true;
                        },
                        Keycode::N => {
                            self.input_map.keys_pressed[Key::N] = !self.input_map.keys_hold[Key::N];
                            self.input_map.keys_hold[Key::N] = true;
                        },
                        Keycode::F => {
                            self.input_map.keys_pressed[Key::F] = !self.input_map.keys_hold[Key::F];
                            self.input_map.keys_hold[Key::F] = true;
                        },
                        Keycode::I => {
                            self.input_map.keys_pressed[Key::I] = !self.input_map.keys_hold[Key::I];
                            self.input_map.keys_hold[Key::I] = true;
                        },
                        Keycode::LShift => {
                            self.input_map.keys_pressed[Key::LSHIFT] = !self.input_map.keys_hold[Key::LSHIFT];
                            self.input_map.keys_hold[Key::LSHIFT] = true;
                        },
                        Keycode::RShift => {
                            self.input_map.keys_pressed[Key::RSHIFT] = !self.input_map.keys_hold[Key::RSHIFT];
                            self.input_map.keys_hold[Key::RSHIFT] = true;
                        },
                        Keycode::KpPlus => {
                            self.input_map.keys_pressed[Key::NumPLUS] = !self.input_map.keys_hold[Key::NumPLUS];
                            self.input_map.keys_hold[Key::NumPLUS] = true;
                        },
                        Keycode::KpMinus => {
                            self.input_map.keys_pressed[Key::NumMINUS] = !self.input_map.keys_hold[Key::NumMINUS];
                            self.input_map.keys_hold[Key::NumMINUS] = true;
                        },
                        Keycode::Space => {
                            self.input_map.keys_pressed[Key::SPACE] = !self.input_map.keys_hold[Key::SPACE];
                            self.input_map.keys_hold[Key::SPACE] = true;
                        },
                        Keycode::Escape => {
                            self.input_map.keys_pressed[Key::ESC] = !self.input_map.keys_hold[Key::ESC];
                            self.input_map.keys_hold[Key::ESC] = true;
                        },
                        _ => (),
                    }
                },
                sdlEvent::KeyUp {keycode : Some(k), ..} => {
                    match k {
                        Keycode::W => self.input_map.keys_hold[Key::W] = false,
                        Keycode::A => self.input_map.keys_hold[Key::A] = false,
                        Keycode::S => self.input_map.keys_hold[Key::S] = false,
                        Keycode::D => self.input_map.keys_hold[Key::D] = false,
                        Keycode::R => self.input_map.keys_hold[Key::R] = false,
                        Keycode::N => self.input_map.keys_hold[Key::N] = false,
                        Keycode::F => self.input_map.keys_hold[Key::F] = false,
                        Keycode::I => self.input_map.keys_hold[Key::I] = false,
                        Keycode::KpPlus => self.input_map.keys_hold[Key::NumPLUS] = false,
                        Keycode::KpMinus => self.input_map.keys_hold[Key::NumMINUS] = false,
                        Keycode::LShift => self.input_map.keys_hold[Key::LSHIFT] = false,
                        Keycode::RShift => self.input_map.keys_hold[Key::RSHIFT] = false,
                        Keycode::Space => self.input_map.keys_hold[Key::SPACE] = false,
                        Keycode::Escape => self.input_map.keys_hold[Key::ESC] = false,
                        _ => ()
                    }
                },
                sdlEvent::MouseButtonDown {mouse_btn, ..} => {
                    match mouse_btn {
                        MouseButton::Left => {
                            self.input_map.keys_pressed[Key::MouseLeftButton] = !self.input_map.keys_hold[Key::MouseLeftButton];
                            self.input_map.keys_hold[Key::MouseLeftButton] = true;
                        },
                        MouseButton::Right => {
                            self.input_map.keys_pressed[Key::MouseLeftButton] = !self.input_map.keys_hold[Key::MouseLeftButton];
                            self.input_map.keys_hold[Key::MouseLeftButton] = true;
                        },
                        _ => ()
                    }
                },
                sdlEvent::MouseButtonUp {mouse_btn, ..} => {
                    match mouse_btn {
                        MouseButton::Left => self.input_map.keys_hold[Key::MouseLeftButton] = false,
                        MouseButton::Right => self.input_map.keys_hold[Key::MouseLeftButton] = false,
                        _ => ()
                    }
                },
                sdlEvent::MouseWheel {y, ..} => {
                    match y {
                        y if y > 0 => self.input_map.keys_pressed[Key::MouseWheelUp] = true,
                        y if y < 0 => self.input_map.keys_pressed[Key::MouseWheelDown] = true,
                        _ => {
                            self.input_map.keys_pressed[Key::MouseWheelUp] = false;
                            self.input_map.keys_pressed[Key::MouseWheelDown] = false;
                        },
                    }
                },
                _ => (),
            }
        }

        let state = self.event_pump.relative_mouse_state();
        let xrel = state.x();
        let yrel = state.y();
        self.input_map.mouse_x_dt = xrel;
        self.input_map.mouse_y_dt = yrel;
    }
}



impl std::ops::IndexMut<Key> for [bool; KEY_MAP_SIZE] {
    fn index_mut(&mut self, idx : Key) -> &mut bool {
        let this : &mut [bool] = self;
        &mut this[idx as usize]
    }
}

impl std::ops::Index<Key> for [bool; KEY_MAP_SIZE] {
    type Output = bool;

    fn index(&self, idx : Key) -> &bool {
        let this : &[bool] = self;
        &this[idx as usize]
    }
}

impl std::fmt::Display for Key {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Key::W => write!(f, "Key::W"),
            Key::A => write!(f, "Key::A"),
            Key::S => write!(f, "Key::S"),
            Key::D => write!(f, "Key::D"),
            Key::R => write!(f, "Key::R"),
            Key::N => write!(f, "Key::N"),
            Key::F => write!(f, "Key::F"),
            Key::I => write!(f, "Key::I"),
            Key::LSHIFT => write!(f, "Key::LSHIFT"),
            Key::RSHIFT => write!(f, "Key::RSHIFT"),
            Key::SPACE => write!(f, "Key::SPACE"),
            Key::ESC => write!(f, "Key::ESC"),
            Key::NumPLUS => write!(f, "Key::NumPLUS"),
            Key::NumMINUS => write!(f, "Key::NumMINUS"),
            Key::MouseLeftButton => write!(f, "Key::MouseLeftButton"),
            Key::MouseRightButton => write!(f, "Key::MouseRightButton"),
            Key::MouseWheelUp => write!(f, "Key::MouseWheelUp"),
            Key::MouseWheelDown => write!(f, "Key::MouseWheelDown"),
            Key::NONE => write!(f, "Key::NONE"),
        }
    }
}
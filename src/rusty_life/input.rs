extern crate sdl2;

use sdl2::event::Event as sdlEvent;
use sdl2::keyboard::*;
use sdl2::mouse::*;

const KEY_MAP_SIZE : usize = 32;

macro_rules! display_enum {
    (pub enum $name:ident {
        $($variant:ident),*,
    }) => {
        pub enum $name {
            $($variant),*
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                match self {
                    $($name::$variant => write!(f, stringify!($name::$variant))),*
                }
            }
        }
    };
}

macro_rules! match_key_down {
    ($s:ident, $keycode:ident, $(($sdlKey:ident, $key:ident)),*) => {
        match $keycode {
            $(Keycode::$sdlKey => {
                $s.input_map.keys_pressed[Key::$key] = !$s.input_map.keys_hold[Key::$key];
                $s.input_map.keys_hold[Key::$key] = true;
            }),*
            _ => (),
        }
    }
}

macro_rules! match_key_up {
    ($s:ident, $keycode:ident, $(($sdlKey:ident, $key:ident)),*) => {
        match $keycode {
            $(Keycode::$sdlKey => $s.input_map.keys_hold[Key::$key] = false,)*
            _ => (),
        }
    }
}

display_enum!{
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
                    match_key_down!(self, k,
                        (W, W),
                        (A, A),
                        (S,S),
                        (D,D),
                        (R,R),
                        (N,N),
                        (F,F),
                        (I,I),
                        (LShift, LSHIFT),
                        (RShift, RSHIFT),
                        (KpPlus, NumPLUS),
                        (KpMinus, NumMINUS),
                        (Space, SPACE),
                        (Escape, ESC)
                    );
                },
                sdlEvent::KeyUp {keycode : Some(k), ..} => {
                    match_key_up!(self, k,
                        (W, W),
                        (A, A),
                        (S,S),
                        (D,D),
                        (R,R),
                        (N,N),
                        (F,F),
                        (I,I),
                        (LShift, LSHIFT),
                        (RShift, RSHIFT),
                        (KpPlus, NumPLUS),
                        (KpMinus, NumMINUS),
                        (Space, SPACE),
                        (Escape, ESC)
                    );
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
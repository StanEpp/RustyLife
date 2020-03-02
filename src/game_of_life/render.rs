extern crate sdl2;

use sdl2::*;

pub struct Renderer {
    sdl_context : Sdl,
    sdl_window : sdl2::video::Window,
    event_pump : sdl2::EventPump,
}

impl Renderer {
    pub fn new(name : &str,
               window_size : (u32, u32)) -> Renderer {
        let sdl_context = sdl2::init().unwrap();
        let video_subsystem = sdl_context.video().unwrap();

        let sdl_window = video_subsystem.window(name, window_size.0, window_size.1)
                                        .position_centered()
                                        .resizable()
                                        .build()
                                        .unwrap();

        let event_pump = sdl_context.event_pump().unwrap();

        Renderer {
            sdl_context : sdl_context,
            sdl_window : sdl_window,
            event_pump : event_pump
        }
    }

    pub fn render(self : &mut Self) {

    }

    pub fn input<F>(self : &mut Self, mut f : F) where
        F: FnMut(sdl2::event::Event)
    {
        for event in self.event_pump.poll_iter() {
            f(event);
        }
    }

    pub fn exit(self : &mut Self) {
    }
}
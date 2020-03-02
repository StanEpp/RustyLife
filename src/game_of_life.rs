extern crate sdl2;

mod grid;
mod render;

use sdl2::mouse::*;
use sdl2::event::*;
use sdl2::keyboard::*;

pub struct GameOfLife {
    window_size : (u32, u32),
    renderer : render::Renderer,
    grid : grid::Grid
}

impl GameOfLife {

    pub fn new (board_size : (u32, u32),
                name : &str,
                window_size : (u32, u32)) -> GameOfLife {

        let board_size = (board_size.0 - board_size.0 % 16 + 16,
                          board_size.1 - board_size.1 % 16 + 16);

        let grid = grid::Grid::new(12., 2., board_size);
        let renderer = render::Renderer::new(name, window_size);

        Self{window_size : window_size,
             renderer : renderer,
             grid : grid
            }
    }

    pub fn run(self : &mut Self) {
        for i in 0..=1000 {
            for j in 0..=1000{
                self.grid.set_cell(0 + i * 7, 2 + j * 5, true);
                self.grid.set_cell(1 + i * 7, 2 + j * 5, true);
                self.grid.set_cell(2 + i * 7, 2 + j * 5, true);
                self.grid.set_cell(2 + i * 7, 1 + j *  5, true);
                self.grid.set_cell(1 + i * 7, 0 + j *  5, true);
            }
        }

        let mut sim_timer = std::time::Instant::now();
        let mut fps_timer = std::time::Instant::now();
        let mut fps_counter = 0;
        let mut render = true;
        let mut run_additional_lifecycle = false;
        let mut run = true;
        let mut w = self.window_size.0 as f32;

        while run {
            self.renderer.input(|event|{
                match event {
                    Event::Quit {..} => {
                        run = false;
                    }
                    Event::MouseWheel {direction : dir, ..} => {
                            match dir {
                                MouseWheelDirection::Normal  => w -= 300.,
                                MouseWheelDirection::Flipped => w += 300.,
                                _ => ()
                            }
                            // let h = w * (self.window_size.1 as f32 / self.window_size.0 as f32);
                    }
                    Event::MouseButtonDown {mouse_btn : btn, ..} => {
                        match btn {
                            MouseButton::Right => (),
                            _ => ()
                        }
                    }
                    Event::KeyDown {keycode : Some(k), ..} => {
                        match k {
                            Keycode::W => (),
                            Keycode::A => (),
                            Keycode::S => (),
                            Keycode::D => (),
                            Keycode::R => render = !render,
                            Keycode::Space => (),
                            Keycode::N => run_additional_lifecycle = true,
                            Keycode::Escape => run = false,
                            _ => ()
                        }
                    }
                    _ => ()
                }
            });

            if run_additional_lifecycle {
                self.grid.run_lifecycle();
                run_additional_lifecycle = false;
            }

            if sim_timer.elapsed().as_millis() >= 10 {
                self.grid.run_lifecycle();
                sim_timer = std::time::Instant::now();
            }

            if render {
                self.renderer.render();
            }

            fps_counter = fps_counter + 1;
            if fps_timer.elapsed().as_millis() >= 1000 {
                println!("FPS: {}", fps_counter);
                fps_counter = 0;
                fps_timer = std::time::Instant::now();
            }

        }
    }
}

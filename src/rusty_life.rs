extern crate sdl2;
extern crate rand;

use rand::Rng;

mod grid;
mod render;
mod input;
mod view;

pub struct RustyLife {
    renderer : render::Renderer,
    grid : grid::Grid,
    input : input::Input,
    view : view::OrthoView,
}

impl RustyLife {

    pub fn new (board_size : (u32, u32),
                name : &str,
                window_size : (u32, u32)) -> RustyLife {

        let mut board_size = board_size;
        board_size.0 = match board_size.0 % 16 {
            0 => board_size.0,
            x => board_size.0 + (16 - x)
        };
        board_size.1 = match board_size.1 % 16 {
            0 => board_size.1,
            x => board_size.1 + (16 - x)
        };

        let mut grid = grid::Grid::new(board_size);
        let renderer = render::Renderer::new(name,
            window_size,
            grid.num_rows as u32,
            grid.num_cols as u32);

        let input = renderer.create_input();
        let view = view::OrthoView::new(window_size);


        // Randomly initialize grid
        let mut rng = rand::thread_rng();
        for _ in 0..(board_size.0 * board_size.1 / 2) {
            let col = rng.gen_range(0, board_size.0);
            let row = rng.gen_range(0, board_size.1);
            grid.set_cell(col as usize, row as usize, true);
        }

        // Or use armada of Gliders
        // for i in 0..=10000 {
        //     for j in 0..=10000{
        //         grid.set_cell(0 + i * 7, 2 + j * 5, true);
        //         grid.set_cell(1 + i * 7, 2 + j * 5, true);
        //         grid.set_cell(2 + i * 7, 2 + j * 5, true);
        //         grid.set_cell(2 + i * 7, 1 + j *  5, true);
        //         grid.set_cell(1 + i * 7, 0 + j *  5, true);
        //     }
        // }

        Self{renderer : renderer,
             input : input,
             grid : grid,
             view : view,
            }
    }

    pub fn run(self : &mut Self) {
        let mut sim_timer = std::time::Instant::now();
        let mut fps_timer = std::time::Instant::now();
        let mut frame_timer = std::time::Instant::now();
        let mut fps_counter = 0;
        let mut fps = 0;
        let mut sim_step_ms = 10_u128;
        let mut render = true;
        let mut run_sim = true;
        let mut run = true;

        while run {
            self.input.update_input();
            let input_map = self.input.get_input_map();

            if input_map.keys_pressed[input::Key::ESC] {
                run = false;
            }
            if input_map.keys_pressed[input::Key::N] {
                self.grid.run_lifecycle();
            }
            if input_map.keys_pressed[input::Key::R] {
                render = !render;
            }
            if input_map.keys_pressed[input::Key::SPACE] {
                run_sim = !run_sim;
            }

            if input_map.keys_hold[input::Key::LSHIFT] &&
               input_map.keys_pressed[input::Key::NumPLUS] {
                if sim_step_ms < std::u128::MAX-10 {
                    sim_step_ms += 10;
                }
            } else if input_map.keys_pressed[input::Key::NumPLUS] {
                if sim_step_ms < std::u128::MAX {
                    sim_step_ms += 1;
                }
            }

            if input_map.keys_hold[input::Key::LSHIFT] &&
               input_map.keys_pressed[input::Key::NumMINUS] {
                if sim_step_ms > 10 {
                    sim_step_ms -= 10;
                }
            } else if input_map.keys_pressed[input::Key::NumMINUS] {
                if sim_step_ms > 0 {
                    sim_step_ms -= 1;
                }
            }

            if run_sim && sim_timer.elapsed().as_millis() >= sim_step_ms {
                self.grid.run_lifecycle();
                sim_timer = std::time::Instant::now();
            }

            if render {
                let frame_duration = frame_timer.elapsed();
                frame_timer = std::time::Instant::now();
                self.view.update(&input_map, &frame_duration);
                self.renderer.render(&self.grid.cells, &self.view, &frame_duration);
            }

            fps_counter = fps_counter + 1;
            if fps_timer.elapsed().as_millis() >= 1000 {
                fps = fps_counter;
                fps_counter = 0;
                fps_timer = std::time::Instant::now();
            }

            print!("sim_step: {}ms    fps: {}    \r", sim_step_ms, fps);
        }
    }
}
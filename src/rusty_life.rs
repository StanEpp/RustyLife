extern crate sdl2;
extern crate rand;

use rand::Rng;
use std::io::{Write, stdout};

mod grid;
mod render;
mod input;
mod view;
mod file_reader;

macro_rules! enum_str {
    (enum $name:ident {
        $($variant:ident),*,
    }) => {
        enum $name {
            $($variant),*
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                match self {
                    $($name::$variant => write!(f, stringify!($variant))),*
                }
            }
        }
    };
}

enum_str!{
enum SimStatus {
    PAUSED,
    RUNNING,
}
}

struct Statistics {
    sim_step_ms : u128,
    generation : u128,
    fps : u64,
    rendering : bool,
    sim_status : SimStatus,
    board_width : u128,
    board_height : u128,
    resolution_width : u32,
    resolution_height : u32,
}

impl Statistics {
    fn new() -> Statistics {
        Statistics {
            sim_step_ms : 10,
            generation : 0,
            fps : 0,
            rendering : true,
            sim_status : SimStatus::PAUSED,
            board_width : 0,
            board_height : 0,
            resolution_width : 0,
            resolution_height : 0,
        }
    }
}

pub struct RustyLife {
    renderer : render::Renderer,
    grid : grid::Grid,
    input : input::Input,
    view : view::OrthoView,
    stats : Statistics,
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

        let mut stats = Statistics::new();
        stats.board_width = board_size.0 as u128;
        stats.board_height = board_size.1 as u128;
        stats.resolution_width = window_size.0 as u32;
        stats.resolution_height = window_size.1 as u32;

        let mut grid = grid::Grid::new(board_size);
        let renderer = render::Renderer::new(name,
            window_size,
            grid.num_rows as u32,
            grid.num_cols as u32);

        let input = renderer.create_input();
        let view = view::OrthoView::new(window_size);

        // match file_reader::read_rle("./foo.rle") {
        //     Some(p) => {
        //         for v in &p.pattern {
        //             grid.set_cell(v.0 as usize, v.1 as usize, true);
        //         }
        //     }
        //     _ => println!("Couldn't load!"),
        // }

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
             stats : stats
            }
    }

    pub fn run(self : &mut Self) {
        match crossterm::execute!(stdout(), crossterm::cursor::SavePosition) {
            Err(_) => (),
            Ok(_) => (),
        }

        let mut sim_timer = std::time::Instant::now();
        let mut frame_counter_timer = std::time::Instant::now();
        let mut frame_timer = std::time::Instant::now();
        let mut fps_counter = 0;
        let mut run = true;

        while run {
            self.input.update_input();
            let input_map = self.input.get_input_map();

            if input_map.keys_pressed[input::Key::ESC] {
                run = false;
            }
            if input_map.keys_pressed[input::Key::N] {
                self.grid.run_lifecycle();
                self.stats.generation += 1;
            }
            if input_map.keys_pressed[input::Key::R] {
                self.stats.rendering = !self.stats.rendering;
            }
            if input_map.keys_pressed[input::Key::SPACE] {
                match self.stats.sim_status {
                    SimStatus::RUNNING => self.stats.sim_status = SimStatus::PAUSED,
                    _ => self.stats.sim_status = SimStatus::RUNNING,
                }
            }

            if input_map.keys_hold[input::Key::LSHIFT] &&
               input_map.keys_pressed[input::Key::NumPLUS] {
                if self.stats.sim_step_ms < std::u128::MAX-10 {
                    self.stats.sim_step_ms += 10;
                }
            } else if input_map.keys_pressed[input::Key::NumPLUS] {
                if self.stats.sim_step_ms < std::u128::MAX {
                    self.stats.sim_step_ms += 1;
                }
            }

            if input_map.keys_hold[input::Key::LSHIFT] &&
               input_map.keys_pressed[input::Key::NumMINUS] {
                if self.stats.sim_step_ms > 10 {
                    self.stats.sim_step_ms -= 10;
                }
            } else if input_map.keys_pressed[input::Key::NumMINUS] {
                if self.stats.sim_step_ms > 0 {
                    self.stats.sim_step_ms -= 1;
                }
            }

            match self.stats.sim_status {
                SimStatus::RUNNING => {
                    if sim_timer.elapsed().as_millis() >= self.stats.sim_step_ms {
                        self.grid.run_lifecycle();
                        self.stats.generation += 1;
                        sim_timer = std::time::Instant::now();
                    }
                },
                _ => (),
            }

            if self.stats.rendering {
                let frame_duration = frame_timer.elapsed();
                frame_timer = std::time::Instant::now();
                self.view.update(&input_map, &frame_duration);
                self.renderer.render(&self.grid.cells, &self.view, &frame_duration);
            }

            fps_counter = fps_counter + 1;
            if frame_counter_timer.elapsed().as_millis() >= 1000 {
                self.stats.fps = fps_counter;
                fps_counter = 0;
                frame_counter_timer = std::time::Instant::now();
            }

            match self.print_statistics() {
                Err(err) => println!("Error printing Stats: \n\t{}", err),
                _ => (),
            }
        }
    }

    fn print_statistics(self : &Self) -> crossterm::Result<()> {
        use crossterm::*;
        let mut stdout = stdout();
        queue!(stdout, cursor::RestorePosition)?;
        queue!(stdout, cursor::SavePosition)?;
        queue!(stdout, style::Print("----------------------------Rusty Life---------------------------------\n"))?;
        queue!(stdout, style::Print(format!("| sim_step: {}ms                    ", self.stats.sim_step_ms)))?;
        queue!(stdout, cursor::MoveToColumn(40))?;
        queue!(stdout, style::Print(format!("board size: {}x{}                   ", self.stats.board_width, self.stats.board_height)))?;
        queue!(stdout, cursor::MoveToColumn(71))?;
        queue!(stdout, style::Print("|\n"))?;

        queue!(stdout, style::Print(format!("| generation: {}                    ", self.stats.generation)))?;
        queue!(stdout, cursor::MoveToColumn(40))?;
        queue!(stdout, style::Print(format!("resolution: {}x{}                   ", self.stats.resolution_width, self.stats.resolution_height)))?;
        queue!(stdout, cursor::MoveToColumn(71))?;
        queue!(stdout, style::Print("|\n"))?;

        queue!(stdout, style::Print(format!("| fps: {}                           ", self.stats.fps)))?;
        queue!(stdout, cursor::MoveToColumn(71))?;
        queue!(stdout, style::Print("|\n"))?;

        queue!(stdout, style::Print(format!("| rendering: {}                     ", self.stats.rendering)))?;
        queue!(stdout, cursor::MoveToColumn(71))?;
        queue!(stdout, style::Print("|\n"))?;

        queue!(stdout, style::Print(format!("| status: {}                        ", self.stats.sim_status)))?;
        queue!(stdout, cursor::MoveToColumn(71))?;
        queue!(stdout, style::Print("|\n"))?;

        queue!(stdout, style::Print("-----------------------------------------------------------------------\n"))?;
        stdout.flush()?;
        Ok(())
    }
}
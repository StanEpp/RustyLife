extern crate sfml;
extern crate rodio;
extern crate bit_vec;

mod grid;

use rodio::Sink;
use sfml::graphics::{RenderWindow, RenderTarget, View, Transformable,
                     RectangleShape, Color, PrimitiveType, RenderStates};
use sfml::system::{Vector2f, Vector2i};
use sfml::window::{Event, Style, Key};
use std::io::BufReader;
use std::vec::Vec;

pub enum ActiveGrid {
    A, B
}

pub struct GameOfLife<'a> {
    window_size : (u32, u32),
    window : RenderWindow,
    grid_a : grid::Grid,
    grid_b : grid::Grid,
    active_grid : ActiveGrid,
    cell_sprites : Vec<RectangleShape<'a>>
}

impl<'a> GameOfLife<'a> {

    pub fn new (board_size : (u32, u32),
                name : String,
                window_size : (u32, u32)) -> GameOfLife<'a> {

        let mut wnd = RenderWindow::new(window_size,
                                        &name,
                                        Style::CLOSE,
                                        &Default::default());
        wnd.set_framerate_limit(144);

        let grid_a = grid::Grid::new(12., 2., board_size);
        let grid_b = grid_a.clone();

        Self{window_size : window_size,
             window : wnd,
             grid_a : grid_a,
             grid_b : grid_b,
             active_grid : ActiveGrid::A,
             cell_sprites : Vec::new()}
    }

    fn swap_grid(self : &mut Self) {
        self.active_grid = match &self.active_grid {
            ActiveGrid::A => ActiveGrid::B,
            ActiveGrid::B => ActiveGrid::A,
        }
    }

    // fn check_neighbours() ->

    fn run_one_lifecycle(self : &mut Self) {
        let mut checked_cells = std::collections::HashSet::new();

        match &self.active_grid {
            ActiveGrid::A => {
                self.grid_b.cells.clear();
                for key in &self.grid_a.cells {
                    let (col, row) = self.grid_a.key_to_coord(*key);
                    let mut keys = [(0_usize, 0_usize); 9];

                    keys[8] = (col, row);

                    if col > 0 && col < self.grid_a.num_cols-1 &&
                       row > 0 && row < self.grid_a.num_rows-1 {
                        keys[0] = (col+1, row);
                        keys[1] = (col-1, row);
                        keys[2] = (col, row+1);
                        keys[3] = (col, row-1);
                        keys[4] = (col+1, row+1);
                        keys[5] = (col-1, row+1);
                        keys[6] = (col+1, row-1);
                        keys[7] = (col-1, row-1);
                    } else if col > 0 && col < self.grid_a.num_cols-1 &&
                              row == 0 {
                        keys[0] = (col, 1);
                        keys[1] = (col+1, 1);
                        keys[2] = (col-1, 1);
                        keys[3] = (col-1, 0);
                        keys[4] = (col+1, 0);
                        keys[5] = (col, self.grid_a.num_rows-1);
                        keys[6] = (col+1, self.grid_a.num_rows-1);
                        keys[7] = (col-1, self.grid_a.num_rows-1);
                    } else if col > 0 && col < self.grid_a.num_cols-1 &&
                              row == self.grid_a.num_rows-1 {
                        keys[0] = (col+1, self.grid_a.num_rows-2);
                        keys[1] = (col, self.grid_a.num_rows-2);
                        keys[2] = (col-1, self.grid_a.num_rows-2);
                        keys[3] = (col+1, self.grid_a.num_rows-1);
                        keys[4] = (col-1, self.grid_a.num_rows-1);
                        keys[5] = (col+1, 0);
                        keys[6] = (col, 0);
                        keys[7] = (col-1, 0);
                    } else if col == 0 &&
                              row > 0 && row < self.grid_a.num_rows-1 {
                        keys[0] = (1, row+1);
                        keys[1] = (1, row);
                        keys[2] = (1, row-1);
                        keys[3] = (0, row-1);
                        keys[4] = (0, row+1);
                        keys[5] = (self.grid_a.num_cols-1, row+1);
                        keys[6] = (self.grid_a.num_cols-1, row);
                        keys[7] = (self.grid_a.num_cols-1, row-1);
                    } else if col == self.grid_a.num_cols-1 &&
                              row > 0 && row < self.grid_a.num_rows-1 {
                        keys[0] = (self.grid_a.num_cols-2, row+1);
                        keys[1] = (self.grid_a.num_cols-2, row);
                        keys[2] = (self.grid_a.num_cols-2, row-1);
                        keys[3] = (self.grid_a.num_cols-1, row-1);
                        keys[4] = (self.grid_a.num_cols-1, row+1);
                        keys[5] = (0, row+1);
                        keys[6] = (0, row);
                        keys[7] = (0, row-1);
                    } else if col == 0 &&
                              row == 0 {
                        keys[0] = (1, 0);
                        keys[1] = (1, 1);
                        keys[2] = (0, 1);
                        keys[3] = (0, self.grid_a.num_rows-1);
                        keys[4] = (1, self.grid_a.num_rows-1);
                        keys[5] = (self.grid_a.num_cols-1, 0);
                        keys[6] = (self.grid_a.num_cols-1, 1);
                        keys[7] = (self.grid_a.num_cols-1, self.grid_a.num_rows-1);
                    } else if col == self.grid_a.num_cols-1 &&
                              row == self.grid_a.num_rows-1 {
                        keys[0] = (self.grid_a.num_cols-2, self.grid_a.num_rows-1);
                        keys[1] = (self.grid_a.num_cols-2, self.grid_a.num_rows-2);
                        keys[2] = (self.grid_a.num_cols-1, self.grid_a.num_rows-2);
                        keys[3] = (0, self.grid_a.num_rows-2);
                        keys[4] = (0, self.grid_a.num_rows-1);
                        keys[5] = (self.grid_a.num_cols-1, 0);
                        keys[6] = (self.grid_a.num_cols-2, 0);
                        keys[7] = (0, 0);
                    } else if col == 0 &&
                              row == self.grid_a.num_rows-1 {
                        keys[0] = (0, self.grid_a.num_rows-2);
                        keys[1] = (1, self.grid_a.num_rows-2);
                        keys[2] = (1, self.grid_a.num_rows-1);
                        keys[3] = (self.grid_a.num_cols-1, 0);
                        keys[4] = (0, 0);
                        keys[5] = (1, 0);
                        keys[6] = (self.grid_a.num_cols-1, self.grid_a.num_rows-1);
                        keys[7] = (self.grid_a.num_cols-1, self.grid_a.num_rows-2);
                    } else if col == self.grid_a.num_cols-1 &&
                              row == 0 {
                        keys[0] = (self.grid_a.num_cols-2, 0);
                        keys[1] = (self.grid_a.num_cols-2, 1);
                        keys[2] = (self.grid_a.num_cols-1, 1);
                        keys[3] = (0, 0);
                        keys[4] = (0, 1);
                        keys[5] = (self.grid_a.num_cols-1, self.grid_a.num_rows-1);
                        keys[6] = (self.grid_a.num_cols-2, self.grid_a.num_rows-1);
                        keys[7] = (0, self.grid_a.num_rows-1);
                    } else {
                        return;
                    }

                    for (col, row) in &keys {
                        if !checked_cells.contains(&(*col, *row)) {
                            self.grid_b.set_cell(*col, *row, self.grid_a.rule_result(*col, *row).unwrap());
                            checked_cells.insert((*col, *row));
                        }
                    }
                }
                self.grid_a.cells.clear();
            },
            ActiveGrid::B => {
                self.grid_a.cells.clear();
                for key in &self.grid_b.cells {
                    let (col, row) = self.grid_b.key_to_coord(*key);
                    let mut keys = [(0_usize, 0_usize); 9];

                    if col > 0 && col < self.grid_b.num_cols-1 &&
                       row > 0 && row < self.grid_b.num_rows-1 {
                        keys[0] = (col+1, row);
                        keys[1] = (col-1, row);
                        keys[2] = (col, row+1);
                        keys[3] = (col, row-1);
                        keys[4] = (col+1, row+1);
                        keys[5] = (col-1, row+1);
                        keys[6] = (col+1, row-1);
                        keys[7] = (col-1, row-1);
                    } else if col > 0 && col < self.grid_b.num_cols-1 &&
                              row == 0 {
                        keys[0] = (col, 1);
                        keys[1] = (col+1, 1);
                        keys[2] = (col-1, 1);
                        keys[3] = (col-1, 0);
                        keys[4] = (col+1, 0);
                        keys[5] = (col, self.grid_b.num_rows-1);
                        keys[6] = (col+1, self.grid_b.num_rows-1);
                        keys[7] = (col-1, self.grid_b.num_rows-1);
                    } else if col > 0 && col < self.grid_b.num_cols-1 &&
                              row == self.grid_b.num_rows-1 {
                        keys[0] = (col+1, self.grid_b.num_rows-2);
                        keys[1] = (col, self.grid_b.num_rows-2);
                        keys[2] = (col-1, self.grid_b.num_rows-2);
                        keys[3] = (col+1, self.grid_b.num_rows-1);
                        keys[4] = (col-1, self.grid_b.num_rows-1);
                        keys[5] = (col+1, 0);
                        keys[6] = (col, 0);
                        keys[7] = (col-1, 0);
                    } else if col == 0 &&
                              row > 0 && row < self.grid_b.num_rows-1 {
                        keys[0] = (1, row+1);
                        keys[1] = (1, row);
                        keys[2] = (1, row-1);
                        keys[3] = (0, row-1);
                        keys[4] = (0, row+1);
                        keys[5] = (self.grid_b.num_cols-1, row+1);
                        keys[6] = (self.grid_b.num_cols-1, row);
                        keys[7] = (self.grid_b.num_cols-1, row-1);
                    } else if col == self.grid_b.num_cols-1 &&
                              row > 0 && row < self.grid_b.num_rows-1 {
                        keys[0] = (self.grid_b.num_cols-2, row+1);
                        keys[1] = (self.grid_b.num_cols-2, row);
                        keys[2] = (self.grid_b.num_cols-2, row-1);
                        keys[3] = (self.grid_b.num_cols-1, row-1);
                        keys[4] = (self.grid_b.num_cols-1, row+1);
                        keys[5] = (0, row+1);
                        keys[6] = (0, row);
                        keys[7] = (0, row-1);
                    } else if col == 0 &&
                              row == 0 {
                        keys[0] = (1, 0);
                        keys[1] = (1, 1);
                        keys[2] = (0, 1);
                        keys[3] = (0, self.grid_b.num_rows-1);
                        keys[4] = (1, self.grid_b.num_rows-1);
                        keys[5] = (self.grid_b.num_cols-1, 0);
                        keys[6] = (self.grid_b.num_cols-1, 1);
                        keys[7] = (self.grid_b.num_cols-1, self.grid_b.num_rows-1);
                    } else if col == self.grid_b.num_cols-1 &&
                              row == self.grid_b.num_rows-1 {
                        keys[0] = (self.grid_b.num_cols-2, self.grid_b.num_rows-1);
                        keys[1] = (self.grid_b.num_cols-2, self.grid_b.num_rows-2);
                        keys[2] = (self.grid_b.num_cols-1, self.grid_b.num_rows-2);
                        keys[3] = (0, self.grid_b.num_rows-2);
                        keys[4] = (0, self.grid_b.num_rows-1);
                        keys[5] = (self.grid_b.num_cols-1, 0);
                        keys[6] = (self.grid_b.num_cols-2, 0);
                        keys[7] = (0, 0);
                    } else if col == 0 &&
                              row == self.grid_b.num_rows-1 {
                        keys[0] = (0, self.grid_b.num_rows-2);
                        keys[1] = (1, self.grid_b.num_rows-2);
                        keys[2] = (1, self.grid_b.num_rows-1);
                        keys[3] = (self.grid_b.num_cols-1, 0);
                        keys[4] = (0, 0);
                        keys[5] = (1, 0);
                        keys[6] = (self.grid_b.num_cols-1, self.grid_b.num_rows-1);
                        keys[7] = (self.grid_b.num_cols-1, self.grid_b.num_rows-2);
                    } else if col == self.grid_b.num_cols-1 &&
                              row == 0 {
                        keys[0] = (self.grid_b.num_cols-2, 0);
                        keys[1] = (self.grid_b.num_cols-2, 1);
                        keys[2] = (self.grid_b.num_cols-1, 1);
                        keys[3] = (0, 0);
                        keys[4] = (0, 1);
                        keys[5] = (self.grid_b.num_cols-1, self.grid_b.num_rows-1);
                        keys[6] = (self.grid_b.num_cols-2, self.grid_b.num_rows-1);
                        keys[7] = (0, self.grid_b.num_rows-1);
                    } else {
                        return;
                    }

                    for (col, row) in &keys {
                        if !checked_cells.contains(&(*col, *row)) {
                            self.grid_a.set_cell(*col, *row, self.grid_b.rule_result(*col, *row).unwrap());
                            checked_cells.insert((*col, *row));
                        }
                    }
                }
                self.grid_b.cells.clear();
            }
        }
    }

    fn render_cells(self : &mut Self) {
        let grid = match &self.active_grid {
            ActiveGrid::A => &self.grid_a,
            ActiveGrid::B => &self.grid_b
        };

        self.cell_sprites.clear();

        for key in &grid.cells {
            let (col, row) = self.grid_a.key_to_coord(*key);
            let mut cell = RectangleShape::with_size(Vector2f::new(12., 12.));
            cell.set_position(Vector2f::from(grid.cell_to_world(col, row)));
            self.cell_sprites.push(cell);
        }

        for cell in self.cell_sprites.iter() {
            self.window.draw(cell);
        }
    }

    pub fn run(self : &mut Self) {
        let device = rodio::default_output_device().unwrap();
        let sink = Sink::new(&device);

        let mut w = self.window_size.0 as f32;

        let file = std::fs::File::open("examples/test.mp3").unwrap();
        sink.append(rodio::Decoder::new(BufReader::new(file)).unwrap());

        let mut view = View::new(Vector2f::new(0.0, 0.0), Vector2f::new(1920.0, 1080.0));

        let mut block = sfml::graphics::RectangleShape::with_size(Vector2f::new(12., 12.));

        for i in 0..=10 {
            for j in 0..=10{
                self.grid_a.set_cell(10 + i * 50, 10 + j * 5, true);
                self.grid_a.set_cell(11 + i * 50, 10 + j * 5, true);
                self.grid_a.set_cell(12 + i * 50, 10 + j * 5, true);
                self.grid_a.set_cell(12 + i * 50, 9 + j *  5, true);
                self.grid_a.set_cell(11 + i * 50, 8 + j *  5, true);
            }
        }

        let mut start = std::time::Instant::now();
        let mut fps = std::time::Instant::now();
        let mut fpsCounter = 0;

        while self.window.is_open() {
            sink.play();
            while let Some(event) = self.window.poll_event() {
                match event {
                    Event::Closed => {
                        self.window.close();
                        sink.stop();
                    }
                    Event::MouseWheelScrolled {wheel:_, delta, x:_, y:_} => {
                        if delta > 0. {
                            w -= 30.;
                        } else {
                            w += 30.;
                        }
                        let h = w * (self.window_size.1 as f32 / self.window_size.0 as f32);
                        view.set_size(Vector2f::new(w, h));
                    }
                    Event::KeyPressed {code, alt:_, ctrl:_, shift:_, system:_} => {
                        match code {
                            Key::W => {
                                view.move_(Vector2f::new(0., -3.));
                            }
                            Key::A => {
                                view.move_(Vector2f::new(-3., 0.));
                            }
                            Key::S => {
                                view.move_(Vector2f::new(0., 3.));
                            }
                            Key::D => {
                                view.move_(Vector2f::new(3., 0.));
                            }
                            Key::Space => {
                                view.set_center((0.,0.));
                                view.set_size(Vector2f::new(self.window_size.0 as f32, self.window_size.1 as f32));
                                w = self.window_size.0 as f32;
                            }
                            Key::Escape => {
                                self.window.close();
                                sink.stop();
                            }
                            _ => {}
                        }
                    }
                    Event::MouseButtonPressed {button : _, x, y} => {
                        let w_c = self.window.map_pixel_to_coords_current_view(Vector2i::new(x, y));
                        match self.grid_a.world_to_cell(w_c.x, w_c.y) {
                            Some((col, row)) => {
                                let (w_x, w_y) = self.grid_a.cell_to_world(col, row);
                                block.set_position(Vector2f::new(w_x, w_y));
                                self.grid_a.set_cell(col, row, true);
                                self.grid_b.set_cell(col, row, true);
                                println!("Mapped: {} {}", col, row);
                            }
                            None => {}
                        }
                        println!("Mapped: {} {}", w_c.x, w_c.y);
                    }
                    _ => {}
                }
            }

            if start.elapsed().as_millis() >= 10 {
                self.run_one_lifecycle();
                self.swap_grid();
                start = std::time::Instant::now();
            }


            self.window.set_view(&view);

            self.window.set_active(true);

            self.window.draw_primitives(&self.grid_a.horizontal_lines, PrimitiveType::Quads, RenderStates::default());
            self.window.draw_primitives(&self.grid_a.vertical_lines, PrimitiveType::Quads, RenderStates::default());
            self.render_cells();

            self.window.display();

            fpsCounter = fpsCounter + 1;
            if fps.elapsed().as_millis() >= 1000 {
                println!("{}", fpsCounter);
                fpsCounter = 0;
                fps = std::time::Instant::now();
            }

            self.window.clear(Color::BLACK);

        }
    }
}
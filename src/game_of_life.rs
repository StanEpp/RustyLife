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

pub struct GameOfLife<'a> {
    window_size : (u32, u32),
    window : RenderWindow,
    grid : grid::Grid,
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

        let grid = grid::Grid::new(12., 2., board_size);

        Self{window_size : window_size,
             window : wnd,
             grid : grid,
             cell_sprites : Vec::new()}
    }

    fn render_cells(self : &mut Self) {
        self.cell_sprites.clear();

        for idx in &self.grid.living_cells {
            let (col, row) = self.grid.idx_to_coord(*idx);
            let mut cell = RectangleShape::with_size(Vector2f::new(12., 12.));
            cell.set_position(Vector2f::from(self.grid.cell_to_world(col, row)));
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

        for i in 0..=100 {
            for j in 0..=100{
                self.grid.set_cell(10 + i * 7, 10 + j * 5, true);
                self.grid.set_cell(11 + i * 7, 10 + j * 5, true);
                self.grid.set_cell(12 + i * 7, 10 + j * 5, true);
                self.grid.set_cell(12 + i * 7, 9 + j *  5, true);
                self.grid.set_cell(11 + i * 7, 8 + j *  5, true);
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
                        match self.grid.world_to_cell(w_c.x, w_c.y) {
                            Some((col, row)) => {
                                let (w_x, w_y) = self.grid.cell_to_world(col, row);
                                block.set_position(Vector2f::new(w_x, w_y));
                                self.grid.set_cell(col, row, true);
                                self.grid.set_cell(col, row, true);
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
                self.grid.run_lifecycle();
                start = std::time::Instant::now();
            }


            self.window.set_view(&view);

            self.window.set_active(true);

            self.window.draw_primitives(&self.grid.horizontal_lines, PrimitiveType::Quads, RenderStates::default());
            self.window.draw_primitives(&self.grid.vertical_lines, PrimitiveType::Quads, RenderStates::default());
            // self.render_cells();

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
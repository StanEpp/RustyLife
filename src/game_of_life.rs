extern crate rodio;
extern crate sfml;

mod grid;

use rodio::Sink;
use sfml::graphics::{RenderWindow, RenderTarget, View, Transformable,
                     Color, PrimitiveType, RenderStates, VertexArray};
use sfml::system::{Vector2f, Vector2i};
use sfml::window::{Event, Style, Key};
use std::io::BufReader;

pub struct GameOfLife {
    window_size : (u32, u32),
    window : RenderWindow,
    grid : grid::Grid,
    cell_sprites : sfml::graphics::VertexArray,
    num_living_cells_curr : usize,
    num_living_cells_prev : usize
}

impl GameOfLife {

    pub fn new (board_size : (usize, usize),
                name : String,
                window_size : (u32, u32)) -> GameOfLife {

        let mut wnd = RenderWindow::new(window_size,
                                        &name,
                                        Style::CLOSE,
                                        &Default::default());
        wnd.set_framerate_limit(144);

        let grid = grid::Grid::new(12., 2., board_size);
        let cell_sprites = VertexArray::new(PrimitiveType::Triangles,
                                            (board_size.0 * board_size.1 * 6) as usize);

        Self{window_size : window_size,
             window : wnd,
             grid : grid,
             cell_sprites : cell_sprites,
             num_living_cells_curr : 0,
             num_living_cells_prev : 0,
            }
    }

    fn render_cells(self : &mut Self) {
        // self.num_living_cells_curr = self.grid.num_living_cells();
        // let mut i = 0_usize;
        // for worker in &self.grid.worker {
        //     let w = worker.lock().unwrap();
        //     for idx in &w.living_cells {
        //         let (col, row) = w.idx_to_coord(*idx);
        //         let p = Vector2f::from(self.grid.cell_to_world(col, row));
        //         self.cell_sprites[i*6].position.x = p.x;
        //         self.cell_sprites[i*6].position.y = p.y;

        //         self.cell_sprites[i*6 + 1].position.x = p.x + 12.0;
        //         self.cell_sprites[i*6 + 1].position.y = p.y;

        //         self.cell_sprites[i*6 + 2].position.x = p.x;
        //         self.cell_sprites[i*6 + 2].position.y = p.y + 12.0;

        //         self.cell_sprites[i*6 + 3].position.x = p.x + 12.0;
        //         self.cell_sprites[i*6 + 3].position.y = p.y;

        //         self.cell_sprites[i*6 + 4].position.x = p.x + 12.0;
        //         self.cell_sprites[i*6 + 4].position.y = p.y + 12.0;

        //         self.cell_sprites[i*6 + 5].position.x = p.x;
        //         self.cell_sprites[i*6 + 5].position.y = p.y + 12.0;
        //         i += 1;
        //     }
        // }

        // if i < self.num_living_cells_prev {
        //     for j in i..self.num_living_cells_prev {
        //         self.cell_sprites[j*6].position.x = 0.0;
        //         self.cell_sprites[j*6].position.y = 0.0;
        //         self.cell_sprites[j*6 + 1].position.x = 0.0;
        //         self.cell_sprites[j*6 + 1].position.y = 0.0;
        //         self.cell_sprites[j*6 + 2].position.x = 0.0;
        //         self.cell_sprites[j*6 + 2].position.y = 0.0;
        //         self.cell_sprites[j*6 + 3].position.x = 0.0;
        //         self.cell_sprites[j*6 + 3].position.y = 0.0;
        //         self.cell_sprites[j*6 + 4].position.x = 0.0;
        //         self.cell_sprites[j*6 + 4].position.y = 0.0;
        //         self.cell_sprites[j*6 + 5].position.x = 0.0;
        //         self.cell_sprites[j*6 + 5].position.y = 0.0;
        //     }
        // }
        // self.num_living_cells_prev = i + 1;

        // self.window.draw(&self.cell_sprites);
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
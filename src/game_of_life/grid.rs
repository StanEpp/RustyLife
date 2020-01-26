extern crate bit_vec;
extern crate sfml;

use sfml::graphics::{Vertex};
use sfml::system::{Vector2f};
use std::collections::HashSet;

#[derive(Clone)]
pub struct Grid {
    pub cell_size : f32,
    pub line_width : f32,
    pub horizontal_lines: Vec<Vertex>,
    pub vertical_lines: Vec<Vertex>,
    pub cells : HashSet<usize>,
    pub num_cols : usize,
    pub num_rows : usize
}

impl Grid {
    pub fn new (cell_size : f32, line_width : f32, board_size : (u32, u32)) -> Self {
        let mut hl = Vec::new();
        let top_left_x = (board_size.0 as f32 / 2.0_f32).floor() * (cell_size + line_width) +
                       (board_size.0 % 2) as f32 * line_width / 2.0_f32 +
                       (board_size.0 % 2) as f32 * (cell_size / 2.0_f32 + line_width);
        let top_left_x = top_left_x * -1.;

        let top_left_y = (board_size.1 as f32 / 2.0_f32).floor() * (cell_size + line_width) +
                       (board_size.1 % 2) as f32 * line_width as f32 / 2.0_f32 +
                       (board_size.1 % 2) as f32 * (cell_size / 2.0_f32 + line_width);
        let top_left_y = top_left_y * -1.;

        for i in 0..board_size.1+1 {
            let off = i as f32 * (cell_size + line_width);
            hl.push(Vertex::with_pos(Vector2f::new(top_left_x, top_left_y + off)));
            hl.push(Vertex::with_pos(Vector2f::new(top_left_x, top_left_y + line_width + off)));
            hl.push(Vertex::with_pos(Vector2f::new(top_left_x.abs() + line_width, top_left_y + line_width + off)));
            hl.push(Vertex::with_pos(Vector2f::new(top_left_x.abs() + line_width, top_left_y + off)));
        }

        let mut vl = Vec::new();
        for i in 0..board_size.0 + 1 {
            let off = i as f32 * (cell_size + line_width);
            vl.push(Vertex::with_pos(Vector2f::new(top_left_x + off, top_left_y)));
            vl.push(Vertex::with_pos(Vector2f::new(top_left_x + off, top_left_y * -1. + line_width)));
            vl.push(Vertex::with_pos(Vector2f::new(top_left_x + line_width + off, top_left_y * -1. + line_width)));
            vl.push(Vertex::with_pos(Vector2f::new(top_left_x + line_width + off, top_left_y)));
        }

        Self{cell_size : cell_size,
             line_width : line_width,
             horizontal_lines : hl,
             vertical_lines : vl,
             cells : HashSet::new(),
             num_cols : board_size.0 as usize,
             num_rows : board_size.1 as usize
        }
    }

    pub fn coord_to_key(self : &Self, col : usize, row : usize) -> usize {
        self.num_cols * row  + col
    }

    pub fn key_to_coord(self : &Self, key : usize) -> (usize, usize) {
        let row = key / self.num_cols;
        let col = key - (row * self.num_cols);
        (col, row)
    }

    pub fn world_to_cell(self : &Self, x : f32, y : f32) -> Option<(usize, usize)> {
        let x_n = x - self.horizontal_lines[0].position.x;
        let y_n = y - self.horizontal_lines[0].position.y;
        let s = self.line_width + self.cell_size;
        let col = (x_n / s).floor();
        let row = (y_n / s).floor();

        if x_n - col * s >= self.line_width &&
           y_n - row * s >= self.line_width &&
           col >= 0. && col <= (self.num_cols - 1) as f32 &&
           row >= 0. && row <= (self.num_rows - 1) as f32 {
            return Some((col as usize, row as usize));
        }
        None
    }

    pub fn cell_to_world(self : &Self, col : usize, row : usize) -> (f32, f32) {
        let s = self.line_width + self.cell_size;
        let x = self.horizontal_lines[0].position.x + col as f32 * s + self.line_width;
        let y = self.horizontal_lines[0].position.y + row as f32 * s + self.line_width;
        (x, y)
    }

    pub fn cell(self : &Self, col : usize, row : usize) -> Option<bool> {
        if col < self.num_cols && row < self.num_rows {
            match self.cells.get(&self.coord_to_key(col, row)) {
                Some(_) => { return Some(true); },
                _ => { return Some(false); }
            }
        }
        None
    }

    pub fn set_cell(self : &mut Self, col : usize, row : usize, value : bool) {
        if col < self.num_cols && row < self.num_rows {
            let key = self.coord_to_key(col, row);
            if value {
                self.cells.insert(key);
            } else {
                self.cells.remove(&key);
            }
        }
    }

    pub fn rule_result(self : &Self, col : usize, row : usize) -> Option<bool> {
        let key = self.coord_to_key(col, row);

        match self.num_neighbors(col, row) {
            Some(num_neighbors) => {
                match self.cells.get(&key) {
                    Some(_) => {
                        if num_neighbors < 2 || num_neighbors > 3 {
                            return Some(false);
                        } else {
                            return Some(true);
                        }
                    },
                    _ => {
                        if num_neighbors == 3 {
                            return Some(true);
                        } else {
                            return Some(false);
                        }
                    }
                }
            },
            None => None
        }
    }

    fn num_neighbors(self : &Self, col : usize, row : usize) -> Option<usize> {
        let mut keys = vec![0_usize; 8];

        if col > 0 && col < self.num_cols-1 &&
           row > 0 && row < self.num_rows-1 {
            keys[0] = self.coord_to_key(col+1, row);
            keys[1] = self.coord_to_key(col-1, row);
            keys[2] = self.coord_to_key(col, row+1);
            keys[3] = self.coord_to_key(col, row-1);
            keys[4] = self.coord_to_key(col+1, row+1);
            keys[5] = self.coord_to_key(col-1, row+1);
            keys[6] = self.coord_to_key(col+1, row-1);
            keys[7] = self.coord_to_key(col-1, row-1);
        } else if col > 0 && col < self.num_cols-1 &&
                  row == 0 {
            keys[0] = self.coord_to_key(col, 1);
            keys[1] = self.coord_to_key(col+1, 1);
            keys[2] = self.coord_to_key(col-1, 1);
            keys[3] = self.coord_to_key(col-1, 0);
            keys[4] = self.coord_to_key(col+1, 0);
            keys[5] = self.coord_to_key(col, self.num_rows-1);
            keys[6] = self.coord_to_key(col+1, self.num_rows-1);
            keys[7] = self.coord_to_key(col-1, self.num_rows-1);
        } else if col > 0 && col < self.num_cols-1 &&
                  row == self.num_rows-1 {
            keys[0] = self.coord_to_key(col+1, self.num_rows-2);
            keys[1] = self.coord_to_key(col, self.num_rows-2);
            keys[2] = self.coord_to_key(col-1, self.num_rows-2);
            keys[3] = self.coord_to_key(col+1, self.num_rows-1);
            keys[4] = self.coord_to_key(col-1, self.num_rows-1);
            keys[5] = self.coord_to_key(col+1, 0);
            keys[6] = self.coord_to_key(col, 0);
            keys[7] = self.coord_to_key(col-1, 0);
        } else if col == 0 &&
                  row > 0 && row < self.num_rows-1 {
            keys[0] = self.coord_to_key(1, row+1);
            keys[1] = self.coord_to_key(1, row);
            keys[2] = self.coord_to_key(1, row-1);
            keys[3] = self.coord_to_key(0, row-1);
            keys[4] = self.coord_to_key(0, row+1);
            keys[5] = self.coord_to_key(self.num_cols-1, row+1);
            keys[6] = self.coord_to_key(self.num_cols-1, row);
            keys[7] = self.coord_to_key(self.num_cols-1, row-1);
        } else if col == self.num_cols-1 &&
                  row > 0 && row < self.num_rows-1 {
            keys[0] = self.coord_to_key(self.num_cols-2, row+1);
            keys[1] = self.coord_to_key(self.num_cols-2, row);
            keys[2] = self.coord_to_key(self.num_cols-2, row-1);
            keys[3] = self.coord_to_key(self.num_cols-1, row-1);
            keys[4] = self.coord_to_key(self.num_cols-1, row+1);
            keys[5] = self.coord_to_key(0, row+1);
            keys[6] = self.coord_to_key(0, row);
            keys[7] = self.coord_to_key(0, row-1);
        } else if col == 0 &&
                  row == 0 {
            keys[0] = self.coord_to_key(1, 0);
            keys[1] = self.coord_to_key(1, 1);
            keys[2] = self.coord_to_key(0, 1);
            keys[3] = self.coord_to_key(0, self.num_rows-1);
            keys[4] = self.coord_to_key(1, self.num_rows-1);
            keys[5] = self.coord_to_key(self.num_cols-1, 0);
            keys[6] = self.coord_to_key(self.num_cols-1, 1);
            keys[7] = self.coord_to_key(self.num_cols-1, self.num_rows-1);
        } else if col == self.num_cols-1 &&
                  row == self.num_rows-1 {
            keys[0] = self.coord_to_key(self.num_cols-2, self.num_rows-1);
            keys[1] = self.coord_to_key(self.num_cols-2, self.num_rows-2);
            keys[2] = self.coord_to_key(self.num_cols-1, self.num_rows-2);
            keys[3] = self.coord_to_key(0, self.num_rows-2);
            keys[4] = self.coord_to_key(0, self.num_rows-1);
            keys[5] = self.coord_to_key(self.num_cols-1, 0);
            keys[6] = self.coord_to_key(self.num_cols-2, 0);
            keys[7] = self.coord_to_key(0, 0);
        } else if col == 0 &&
                  row == self.num_rows-1 {
            keys[0] = self.coord_to_key(0, self.num_rows-2);
            keys[1] = self.coord_to_key(1, self.num_rows-2);
            keys[2] = self.coord_to_key(1, self.num_rows-1);
            keys[3] = self.coord_to_key(self.num_cols-1, 0);
            keys[4] = self.coord_to_key(0, 0);
            keys[5] = self.coord_to_key(1, 0);
            keys[6] = self.coord_to_key(self.num_cols-1, self.num_rows-1);
            keys[7] = self.coord_to_key(self.num_cols-1, self.num_rows-2);
        } else if col == self.num_cols-1 &&
                  row == 0 {
            keys[0] = self.coord_to_key(self.num_cols-2, 0);
            keys[1] = self.coord_to_key(self.num_cols-2, 1);
            keys[2] = self.coord_to_key(self.num_cols-1, 1);
            keys[3] = self.coord_to_key(0, 0);
            keys[4] = self.coord_to_key(0, 1);
            keys[5] = self.coord_to_key(self.num_cols-1, self.num_rows-1);
            keys[6] = self.coord_to_key(self.num_cols-2, self.num_rows-1);
            keys[7] = self.coord_to_key(0, self.num_rows-1);
        } else {
            return None;
        }

        let mut num_neighbors = 0_usize;
        for k in &keys {
            num_neighbors = num_neighbors + self.cells.contains(k) as usize;
        }

        Some(num_neighbors)
    }
}
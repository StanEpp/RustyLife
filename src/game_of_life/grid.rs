extern crate bit_vec;
extern crate sfml;
use bit_vec::BitVec;

use std::collections::HashSet;
use sfml::graphics::{Vertex};
use sfml::system::{Vector2f};


#[derive(Clone)]
pub struct Grid {
    pub cell_size : f32,
    pub line_width : f32,
    pub horizontal_lines: Vec<Vertex>,
    pub vertical_lines: Vec<Vertex>,
    // pub cells : Vec<bool>,
    pub cells : BitVec,
    pub living_cells : HashSet<usize>,
    pub living_cells_old : HashSet<usize>,
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

        for i in 0..=board_size.1 {
            let off = i as f32 * (cell_size + line_width);
            hl.push(Vertex::with_pos(Vector2f::new(top_left_x, top_left_y + off)));
            hl.push(Vertex::with_pos(Vector2f::new(top_left_x, top_left_y + line_width + off)));
            hl.push(Vertex::with_pos(Vector2f::new(top_left_x.abs() + line_width, top_left_y + line_width + off)));
            hl.push(Vertex::with_pos(Vector2f::new(top_left_x.abs() + line_width, top_left_y + off)));
        }

        let mut vl = Vec::new();
        for i in 0..=board_size.0 {
            let off = i as f32 * (cell_size + line_width);
            vl.push(Vertex::with_pos(Vector2f::new(top_left_x + off, top_left_y)));
            vl.push(Vertex::with_pos(Vector2f::new(top_left_x + off, top_left_y * -1. + line_width)));
            vl.push(Vertex::with_pos(Vector2f::new(top_left_x + line_width + off, top_left_y * -1. + line_width)));
            vl.push(Vertex::with_pos(Vector2f::new(top_left_x + line_width + off, top_left_y)));
        }

        let cells = BitVec::from_elem((board_size.1 * board_size.0) as usize, false);
        // let cells = vec![false ; (board_size.1 * board_size.0) as usize];

        Self{cell_size : cell_size,
             line_width : line_width,
             horizontal_lines : hl,
             vertical_lines : vl,
             cells : cells,
             living_cells : HashSet::new(),
             living_cells_old : HashSet::new(),
             num_cols : board_size.0 as usize,
             num_rows : board_size.1 as usize
        }
    }

    #[inline]
    fn coord_to_idx(self : &Self, col : usize, row : usize) -> usize {
        self.num_cols * row  + col
    }

    pub fn idx_to_coord(self : &Self, idx : usize) -> (usize, usize) {
        let row = idx / self.num_cols;
        let col = idx - (row * self.num_cols);
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

    pub fn cell(self : &Self, col : usize, row : usize) -> bool {
        self.cells[self.coord_to_idx(col, row)]
    }

    pub fn clear_grid(self : &mut Self) {
        for idx in &self.living_cells {
            // self.cells[*idx] = false;
            self.cells.set(*idx, false);
        }
        self.living_cells.clear();
    }

    pub fn set_cell(self : &mut Self, col : usize, row : usize, value : bool) {
        if col < self.num_cols && row < self.num_rows {
            let idx = self.coord_to_idx(col, row);
            // self.cells[idx] = value;
            self.cells.set(idx, value);
            if value {
                self.living_cells.insert(idx);
            }
        }
    }

    fn rule_result(self : &Self, idx : usize) -> Option<bool> {
        let (col, row) = self.idx_to_coord(idx);

        match self.count_neighbors(col, row) {
            Some(num_neighbors) => {
                if self.cells[idx] == true {
                    if num_neighbors < 2 || num_neighbors > 3 {
                        Some(false)
                    } else {
                        Some(true)
                    }
                } else {
                    if num_neighbors == 3 {
                        Some(true)
                    } else {
                        Some(false)
                    }
                }
            },
            None => None
        }
    }

    fn count_neighbors(self : &Self, col : usize, row : usize) -> Option<u8> {
        let mut num_neighbors = 0_u8;

        if col > 0 && col < self.num_cols-1 &&
           row > 0 && row < self.num_rows-1 {
            num_neighbors += self.cells[self.coord_to_idx(col+1, row)] as u8;
            num_neighbors += self.cells[self.coord_to_idx(col-1, row)] as u8;
            num_neighbors += self.cells[self.coord_to_idx(col, row+1)] as u8;
            num_neighbors += self.cells[self.coord_to_idx(col, row-1)] as u8;
            num_neighbors += self.cells[self.coord_to_idx(col+1, row+1)] as u8;
            num_neighbors += self.cells[self.coord_to_idx(col-1, row+1)] as u8;
            num_neighbors += self.cells[self.coord_to_idx(col+1, row-1)] as u8;
            num_neighbors += self.cells[self.coord_to_idx(col-1, row-1)] as u8;
        } else if col > 0 && col < self.num_cols-1 &&
                  row == 0 {
            num_neighbors += self.cells[self.coord_to_idx(col, 1)] as u8;
            num_neighbors += self.cells[self.coord_to_idx(col+1, 1)] as u8;
            num_neighbors += self.cells[self.coord_to_idx(col-1, 1)] as u8;
            num_neighbors += self.cells[self.coord_to_idx(col-1, 0)] as u8;
            num_neighbors += self.cells[self.coord_to_idx(col+1, 0)] as u8;
            num_neighbors += self.cells[self.coord_to_idx(col, self.num_rows-1)] as u8;
            num_neighbors += self.cells[self.coord_to_idx(col+1, self.num_rows-1)] as u8;
            num_neighbors += self.cells[self.coord_to_idx(col-1, self.num_rows-1)] as u8;
        } else if col > 0 && col < self.num_cols-1 &&
                  row == self.num_rows-1 {
            num_neighbors += self.cells[self.coord_to_idx(col+1, self.num_rows-2)] as u8;
            num_neighbors += self.cells[self.coord_to_idx(col, self.num_rows-2)] as u8;
            num_neighbors += self.cells[self.coord_to_idx(col-1, self.num_rows-2)] as u8;
            num_neighbors += self.cells[self.coord_to_idx(col+1, self.num_rows-1)] as u8;
            num_neighbors += self.cells[self.coord_to_idx(col-1, self.num_rows-1)] as u8;
            num_neighbors += self.cells[self.coord_to_idx(col+1, 0)] as u8;
            num_neighbors += self.cells[self.coord_to_idx(col, 0)] as u8;
            num_neighbors += self.cells[self.coord_to_idx(col-1, 0)] as u8;
        } else if col == 0 &&
                  row > 0 && row < self.num_rows-1 {
            num_neighbors += self.cells[self.coord_to_idx(1, row+1)] as u8;
            num_neighbors += self.cells[self.coord_to_idx(1, row)] as u8;
            num_neighbors += self.cells[self.coord_to_idx(1, row-1)] as u8;
            num_neighbors += self.cells[self.coord_to_idx(0, row-1)] as u8;
            num_neighbors += self.cells[self.coord_to_idx(0, row+1)] as u8;
            num_neighbors += self.cells[self.coord_to_idx(self.num_cols-1, row+1)] as u8;
            num_neighbors += self.cells[self.coord_to_idx(self.num_cols-1, row)] as u8;
            num_neighbors += self.cells[self.coord_to_idx(self.num_cols-1, row-1)] as u8;
        } else if col == self.num_cols-1 &&
                  row > 0 && row < self.num_rows-1 {
            num_neighbors += self.cells[self.coord_to_idx(self.num_cols-2, row+1)] as u8;
            num_neighbors += self.cells[self.coord_to_idx(self.num_cols-2, row)] as u8;
            num_neighbors += self.cells[self.coord_to_idx(self.num_cols-2, row-1)] as u8;
            num_neighbors += self.cells[self.coord_to_idx(self.num_cols-1, row-1)] as u8;
            num_neighbors += self.cells[self.coord_to_idx(self.num_cols-1, row+1)] as u8;
            num_neighbors += self.cells[self.coord_to_idx(0, row+1)] as u8;
            num_neighbors += self.cells[self.coord_to_idx(0, row)] as u8;
            num_neighbors += self.cells[self.coord_to_idx(0, row-1)] as u8;
        } else if col == 0 &&
                  row == 0 {
            num_neighbors += self.cells[self.coord_to_idx(1, 0)] as u8;
            num_neighbors += self.cells[self.coord_to_idx(1, 1)] as u8;
            num_neighbors += self.cells[self.coord_to_idx(0, 1)] as u8;
            num_neighbors += self.cells[self.coord_to_idx(0, self.num_rows-1)] as u8;
            num_neighbors += self.cells[self.coord_to_idx(1, self.num_rows-1)] as u8;
            num_neighbors += self.cells[self.coord_to_idx(self.num_cols-1, 0)] as u8;
            num_neighbors += self.cells[self.coord_to_idx(self.num_cols-1, 1)] as u8;
            num_neighbors += self.cells[self.coord_to_idx(self.num_cols-1, self.num_rows-1)] as u8;
        } else if col == self.num_cols-1 &&
                  row == self.num_rows-1 {
            num_neighbors += self.cells[self.coord_to_idx(self.num_cols-2, self.num_rows-1)] as u8;
            num_neighbors += self.cells[self.coord_to_idx(self.num_cols-2, self.num_rows-2)] as u8;
            num_neighbors += self.cells[self.coord_to_idx(self.num_cols-1, self.num_rows-2)] as u8;
            num_neighbors += self.cells[self.coord_to_idx(0, self.num_rows-2)] as u8;
            num_neighbors += self.cells[self.coord_to_idx(0, self.num_rows-1)] as u8;
            num_neighbors += self.cells[self.coord_to_idx(self.num_cols-1, 0)] as u8;
            num_neighbors += self.cells[self.coord_to_idx(self.num_cols-2, 0)] as u8;
            num_neighbors += self.cells[self.coord_to_idx(0, 0)] as u8;
        } else if col == 0 &&
                  row == self.num_rows-1 {
            num_neighbors += self.cells[self.coord_to_idx(0, self.num_rows-2)] as u8;
            num_neighbors += self.cells[self.coord_to_idx(1, self.num_rows-2)] as u8;
            num_neighbors += self.cells[self.coord_to_idx(1, self.num_rows-1)] as u8;
            num_neighbors += self.cells[self.coord_to_idx(self.num_cols-1, 0)] as u8;
            num_neighbors += self.cells[self.coord_to_idx(0, 0)] as u8;
            num_neighbors += self.cells[self.coord_to_idx(1, 0)] as u8;
            num_neighbors += self.cells[self.coord_to_idx(self.num_cols-1, self.num_rows-1)] as u8;
            num_neighbors += self.cells[self.coord_to_idx(self.num_cols-1, self.num_rows-2)] as u8;
        } else if col == self.num_cols-1 &&
                  row == 0 {
            num_neighbors += self.cells[self.coord_to_idx(self.num_cols-2, 0)] as u8;
            num_neighbors += self.cells[self.coord_to_idx(self.num_cols-2, 1)] as u8;
            num_neighbors += self.cells[self.coord_to_idx(self.num_cols-1, 1)] as u8;
            num_neighbors += self.cells[self.coord_to_idx(0, 0)] as u8;
            num_neighbors += self.cells[self.coord_to_idx(0, 1)] as u8;
            num_neighbors += self.cells[self.coord_to_idx(self.num_cols-1, self.num_rows-1)] as u8;
            num_neighbors += self.cells[self.coord_to_idx(self.num_cols-2, self.num_rows-1)] as u8;
            num_neighbors += self.cells[self.coord_to_idx(0, self.num_rows-1)] as u8;
        } else {
            return None;
        }

        Some(num_neighbors)
    }

    fn get_surrounding_cell_idx(self : &Self, idx : usize) -> [usize; 9] {
        let mut keys = [0_usize; 9];
        let (col, row) = self.idx_to_coord(idx);

        keys[8] = idx;

        if col > 0 && col < self.num_cols-1 &&
           row > 0 && row < self.num_rows-1 {
            keys[0] = self.coord_to_idx(col+1, row);
            keys[1] = self.coord_to_idx(col-1, row);
            keys[2] = self.coord_to_idx(col, row+1);
            keys[3] = self.coord_to_idx(col, row-1);
            keys[4] = self.coord_to_idx(col+1, row+1);
            keys[5] = self.coord_to_idx(col-1, row+1);
            keys[6] = self.coord_to_idx(col+1, row-1);
            keys[7] = self.coord_to_idx(col-1, row-1);
        } else if col > 0 && col < self.num_cols-1 &&
                  row == 0 {
            keys[0] = self.coord_to_idx(col, 1);
            keys[1] = self.coord_to_idx(col+1, 1);
            keys[2] = self.coord_to_idx(col-1, 1);
            keys[3] = self.coord_to_idx(col-1, 0);
            keys[4] = self.coord_to_idx(col+1, 0);
            keys[5] = self.coord_to_idx(col, self.num_rows-1);
            keys[6] = self.coord_to_idx(col+1, self.num_rows-1);
            keys[7] = self.coord_to_idx(col-1, self.num_rows-1);
        } else if col > 0 && col < self.num_cols-1 &&
                  row == self.num_rows-1 {
            keys[0] = self.coord_to_idx(col+1, self.num_rows-2);
            keys[1] = self.coord_to_idx(col, self.num_rows-2);
            keys[2] = self.coord_to_idx(col-1, self.num_rows-2);
            keys[3] = self.coord_to_idx(col+1, self.num_rows-1);
            keys[4] = self.coord_to_idx(col-1, self.num_rows-1);
            keys[5] = self.coord_to_idx(col+1, 0);
            keys[6] = self.coord_to_idx(col, 0);
            keys[7] = self.coord_to_idx(col-1, 0);
        } else if col == 0 &&
                  row > 0 && row < self.num_rows-1 {
            keys[0] = self.coord_to_idx(1, row+1);
            keys[1] = self.coord_to_idx(1, row);
            keys[2] = self.coord_to_idx(1, row-1);
            keys[3] = self.coord_to_idx(0, row-1);
            keys[4] = self.coord_to_idx(0, row+1);
            keys[5] = self.coord_to_idx(self.num_cols-1, row+1);
            keys[6] = self.coord_to_idx(self.num_cols-1, row);
            keys[7] = self.coord_to_idx(self.num_cols-1, row-1);
        } else if col == self.num_cols-1 &&
                  row > 0 && row < self.num_rows-1 {
            keys[0] = self.coord_to_idx(self.num_cols-2, row+1);
            keys[1] = self.coord_to_idx(self.num_cols-2, row);
            keys[2] = self.coord_to_idx(self.num_cols-2, row-1);
            keys[3] = self.coord_to_idx(self.num_cols-1, row-1);
            keys[4] = self.coord_to_idx(self.num_cols-1, row+1);
            keys[5] = self.coord_to_idx(0, row+1);
            keys[6] = self.coord_to_idx(0, row);
            keys[7] = self.coord_to_idx(0, row-1);
        } else if col == 0 &&
                  row == 0 {
            keys[0] = self.coord_to_idx(1, 0);
            keys[1] = self.coord_to_idx(1, 1);
            keys[2] = self.coord_to_idx(0, 1);
            keys[3] = self.coord_to_idx(0, self.num_rows-1);
            keys[4] = self.coord_to_idx(1, self.num_rows-1);
            keys[5] = self.coord_to_idx(self.num_cols-1, 0);
            keys[6] = self.coord_to_idx(self.num_cols-1, 1);
            keys[7] = self.coord_to_idx(self.num_cols-1, self.num_rows-1);
        } else if col == self.num_cols-1 &&
                  row == self.num_rows-1 {
            keys[0] = self.coord_to_idx(self.num_cols-2, self.num_rows-1);
            keys[1] = self.coord_to_idx(self.num_cols-2, self.num_rows-2);
            keys[2] = self.coord_to_idx(self.num_cols-1, self.num_rows-2);
            keys[3] = self.coord_to_idx(0, self.num_rows-2);
            keys[4] = self.coord_to_idx(0, self.num_rows-1);
            keys[5] = self.coord_to_idx(self.num_cols-1, 0);
            keys[6] = self.coord_to_idx(self.num_cols-2, 0);
            keys[7] = self.coord_to_idx(0, 0);
        } else if col == 0 &&
                  row == self.num_rows-1 {
            keys[0] = self.coord_to_idx(0, self.num_rows-2);
            keys[1] = self.coord_to_idx(1, self.num_rows-2);
            keys[2] = self.coord_to_idx(1, self.num_rows-1);
            keys[3] = self.coord_to_idx(self.num_cols-1, 0);
            keys[4] = self.coord_to_idx(0, 0);
            keys[5] = self.coord_to_idx(1, 0);
            keys[6] = self.coord_to_idx(self.num_cols-1, self.num_rows-1);
            keys[7] = self.coord_to_idx(self.num_cols-1, self.num_rows-2);
        } else if col == self.num_cols-1 &&
                  row == 0 {
            keys[0] = self.coord_to_idx(self.num_cols-2, 0);
            keys[1] = self.coord_to_idx(self.num_cols-2, 1);
            keys[2] = self.coord_to_idx(self.num_cols-1, 1);
            keys[3] = self.coord_to_idx(0, 0);
            keys[4] = self.coord_to_idx(0, 1);
            keys[5] = self.coord_to_idx(self.num_cols-1, self.num_rows-1);
            keys[6] = self.coord_to_idx(self.num_cols-2, self.num_rows-1);
            keys[7] = self.coord_to_idx(0, self.num_rows-1);
        }

        keys
    }

    pub fn run_lifecycle(self : &mut Self) {
        for idx in &self.living_cells {
            let indices = self.get_surrounding_cell_idx(*idx);
            for idx in &indices {
                if self.rule_result(*idx).unwrap() {
                    self.living_cells_old.insert(*idx);
                }
            }
        }

        self.clear_grid();
        for idx in &self.living_cells_old {
            self.cells.set(*idx, true);
        }

        std::mem::swap(&mut self.living_cells, &mut self.living_cells_old);
    }
}
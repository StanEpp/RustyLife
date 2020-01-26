extern crate bit_vec;
extern crate sfml;

use bit_vec::BitVec;
use sfml::graphics::{Vertex};
use sfml::system::{Vector2f};

#[derive(Clone)]
pub struct Grid {
    pub cell_size : f32,
    pub line_width : f32,
    pub horizontal_lines: Vec<Vertex>,
    pub vertical_lines: Vec<Vertex>,
    pub cells : Vec<BitVec>
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

        let cells = vec![BitVec::from_elem(board_size.1 as usize, false) ; board_size.0 as usize];

        Self{cell_size : cell_size,
             line_width : line_width,
             horizontal_lines : hl,
             vertical_lines : vl,
             cells : cells
        }
    }

    pub fn world_to_cell(self : &Self, x : f32, y : f32) -> Option<(usize, usize)> {
        let x_n = x - self.horizontal_lines[0].position.x;
        let y_n = y - self.horizontal_lines[0].position.y;
        let s = self.line_width + self.cell_size;
        let col = (x_n / s).floor();
        let row = (y_n / s).floor();

        if x_n - col * s >= self.line_width &&
           y_n - row * s >= self.line_width &&
           col >= 0. && col <= (self.cells.len() - 1) as f32 &&
           row >= 0. && row <= (self.cells[0].len() - 1) as f32 {
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
        self.cells[col][row]
    }

    pub fn set_cell(self : &mut Self, col : usize, row : usize, value : bool) {
        self.cells[col].set(row, value);
    }

    pub fn rule_result(self : &Self, col : usize, row : usize) -> Option<bool> {
        match self.num_neighbors(col, row) {
            Some(num_neighbors) => {
                if self.cells[col][row] == true {
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

    fn num_neighbors(self : &Self, col : usize, row : usize) -> Option<usize> {
        let num_cols = self.cells.len();
        let num_rows = self.cells[0].len();

        let mut num_neighbors = 0_usize;

        if col > 0 && col < num_cols-1 &&
           row > 0 && row < num_rows-1 {
            num_neighbors = num_neighbors + self.cells[col+1][row] as usize;
            num_neighbors = num_neighbors + self.cells[col-1][row] as usize;
            num_neighbors = num_neighbors + self.cells[col][row+1] as usize;
            num_neighbors = num_neighbors + self.cells[col][row-1] as usize;
            num_neighbors = num_neighbors + self.cells[col+1][row+1] as usize;
            num_neighbors = num_neighbors + self.cells[col-1][row+1] as usize;
            num_neighbors = num_neighbors + self.cells[col+1][row-1] as usize;
            num_neighbors = num_neighbors + self.cells[col-1][row-1] as usize;
        } else if col > 0 && col < num_cols-1 &&
                  row == 0 {
            num_neighbors = num_neighbors + self.cells[col][1] as usize;
            num_neighbors = num_neighbors + self.cells[col+1][1] as usize;
            num_neighbors = num_neighbors + self.cells[col-1][1] as usize;
            num_neighbors = num_neighbors + self.cells[col-1][0] as usize;
            num_neighbors = num_neighbors + self.cells[col+1][0] as usize;
            num_neighbors = num_neighbors + self.cells[col][num_rows-1] as usize;
            num_neighbors = num_neighbors + self.cells[col+1][num_rows-1] as usize;
            num_neighbors = num_neighbors + self.cells[col-1][num_rows-1] as usize;
        } else if col > 0 && col < num_cols-1 &&
                  row == num_rows-1 {
            num_neighbors = num_neighbors + self.cells[col+1][num_rows-2] as usize;
            num_neighbors = num_neighbors + self.cells[col][num_rows-2] as usize;
            num_neighbors = num_neighbors + self.cells[col-1][num_rows-2] as usize;
            num_neighbors = num_neighbors + self.cells[col+1][num_rows-1] as usize;
            num_neighbors = num_neighbors + self.cells[col-1][num_rows-1] as usize;
            num_neighbors = num_neighbors + self.cells[col+1][0] as usize;
            num_neighbors = num_neighbors + self.cells[col][0] as usize;
            num_neighbors = num_neighbors + self.cells[col-1][0] as usize;
        } else if col == 0 &&
                  row > 0 && row < num_rows-1 {
            num_neighbors = num_neighbors + self.cells[1][row+1] as usize;
            num_neighbors = num_neighbors + self.cells[1][row] as usize;
            num_neighbors = num_neighbors + self.cells[1][row-1] as usize;
            num_neighbors = num_neighbors + self.cells[0][row-1] as usize;
            num_neighbors = num_neighbors + self.cells[0][row+1] as usize;
            num_neighbors = num_neighbors + self.cells[num_cols-1][row+1] as usize;
            num_neighbors = num_neighbors + self.cells[num_cols-1][row] as usize;
            num_neighbors = num_neighbors + self.cells[num_cols-1][row-1] as usize;
        } else if col == num_cols-1 &&
                  row > 0 && row < num_rows-1 {
            num_neighbors = num_neighbors + self.cells[num_cols-2][row+1] as usize;
            num_neighbors = num_neighbors + self.cells[num_cols-2][row] as usize;
            num_neighbors = num_neighbors + self.cells[num_cols-2][row-1] as usize;
            num_neighbors = num_neighbors + self.cells[num_cols-1][row-1] as usize;
            num_neighbors = num_neighbors + self.cells[num_cols-1][row+1] as usize;
            num_neighbors = num_neighbors + self.cells[0][row+1] as usize;
            num_neighbors = num_neighbors + self.cells[0][row] as usize;
            num_neighbors = num_neighbors + self.cells[0][row-1] as usize;
        } else if col == 0 &&
                  row == 0 {
            num_neighbors = num_neighbors + self.cells[1][0] as usize;
            num_neighbors = num_neighbors + self.cells[1][1] as usize;
            num_neighbors = num_neighbors + self.cells[0][1] as usize;
            num_neighbors = num_neighbors + self.cells[0][num_rows-1] as usize;
            num_neighbors = num_neighbors + self.cells[1][num_rows-1] as usize;
            num_neighbors = num_neighbors + self.cells[num_cols-1][0] as usize;
            num_neighbors = num_neighbors + self.cells[num_cols-1][1] as usize;
            num_neighbors = num_neighbors + self.cells[num_cols-1][num_rows-1] as usize;
        } else if col == num_cols-1 &&
                  row == num_rows-1 {
            num_neighbors = num_neighbors + self.cells[num_cols-2][num_rows-1] as usize;
            num_neighbors = num_neighbors + self.cells[num_cols-2][num_rows-2] as usize;
            num_neighbors = num_neighbors + self.cells[num_cols-1][num_rows-2] as usize;
            num_neighbors = num_neighbors + self.cells[0][num_rows-2] as usize;
            num_neighbors = num_neighbors + self.cells[0][num_rows-1] as usize;
            num_neighbors = num_neighbors + self.cells[num_cols-1][0] as usize;
            num_neighbors = num_neighbors + self.cells[num_cols-2][0] as usize;
            num_neighbors = num_neighbors + self.cells[0][0] as usize;
        } else if col == 0 &&
                  row == num_rows-1 {
            num_neighbors = num_neighbors + self.cells[0][num_rows-2] as usize;
            num_neighbors = num_neighbors + self.cells[1][num_rows-2] as usize;
            num_neighbors = num_neighbors + self.cells[1][num_rows-1] as usize;
            num_neighbors = num_neighbors + self.cells[num_cols-1][0] as usize;
            num_neighbors = num_neighbors + self.cells[0][0] as usize;
            num_neighbors = num_neighbors + self.cells[1][0] as usize;
            num_neighbors = num_neighbors + self.cells[num_cols-1][num_rows-1] as usize;
            num_neighbors = num_neighbors + self.cells[num_cols-1][num_rows-2] as usize;
        } else if col == num_cols-1 &&
                  row == 0 {
            num_neighbors = num_neighbors + self.cells[num_cols-2][0] as usize;
            num_neighbors = num_neighbors + self.cells[num_cols-2][1] as usize;
            num_neighbors = num_neighbors + self.cells[num_cols-1][1] as usize;
            num_neighbors = num_neighbors + self.cells[0][0] as usize;
            num_neighbors = num_neighbors + self.cells[0][1] as usize;
            num_neighbors = num_neighbors + self.cells[num_cols-1][num_rows-1] as usize;
            num_neighbors = num_neighbors + self.cells[num_cols-2][num_rows-1] as usize;
            num_neighbors = num_neighbors + self.cells[0][num_rows-1] as usize;
        } else {
            return None;
        }

        Some(num_neighbors)
    }
}
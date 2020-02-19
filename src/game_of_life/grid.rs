extern crate sfml;

use sfml::graphics::{Vertex};
use sfml::system::{Vector2f};

#[derive(Clone)]
pub struct Grid {
    pub cell_size : f32,
    pub line_width : f32,
    pub horizontal_lines: Vec<Vertex>,
    pub vertical_lines: Vec<Vertex>,
    pub cells : Vec<u16>,
    pub cells_shadow : Vec<u16>,
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
        let size = (board_size.1 * board_size.0) as usize / (std::mem::size_of::<u16>() * 8) ;
        let cells = vec![0_u16 ; size];

        Self{cell_size : cell_size,
             line_width : line_width,
             horizontal_lines : hl,
             vertical_lines : vl,
             cells : cells.clone(),
             cells_shadow : cells,
             num_cols : board_size.0 as usize,
             num_rows : board_size.1 as usize
        }
    }

    #[inline]
    fn coord_to_idx(self : &Self, col : usize, row : usize) -> usize {
        (self.num_cols / 16) * row  + (col / 16)
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

    pub fn set_cell(self : &mut Self, col : usize, row : usize, value : bool) {
        if col < self.num_cols && row < self.num_rows {
            let idx = self.coord_to_idx(col, row);
            let u = self.cells[idx];
            if value {
                self.cells[idx] = u | (0x1 << (15 - col) as u16);
            } else {
                self.cells[idx] = u & !((0x1 << (15 - col) as u16));
            }
        }
    }

    pub fn run_lifecycle(self : &mut Self) {
        let num_cols_c = self.num_cols / (std::mem::size_of::<u16>() * 8);
        let grid_size_c = self.cells.len();

        let m1 = 0b010000000000000000000000000000000_u64;
        let m2 = 0b101000000000000000000000000000000_u64;
        for idx in 0..self.cells.len() {
            let mut col = (idx + num_cols_c - 1) % num_cols_c;
            let row_off = (idx / num_cols_c) * num_cols_c;
            let row_above_off = (row_off + grid_size_c - num_cols_c) % grid_size_c;
            let row_below_off = (row_off + grid_size_c + num_cols_c) % grid_size_c;

            let mut u_a = self.cells[col + row_above_off] as u64;
            let mut u = self.cells[col + row_off] as u64;
            let mut u_b = self.cells[col + row_below_off] as u64;

            u_a <<= 16;
            u <<= 16;
            u_b <<= 16;

            col = (col + 1) % num_cols_c;
            let curr_col = col;
            u_a |= self.cells[col + row_above_off] as u64;
            u   |= self.cells[col + row_off] as u64;
            u_b |= self.cells[col + row_below_off] as u64;

            u_a <<= 16;
            u <<= 16;
            u_b <<= 16;

            col = (col + 1) % num_cols_c;
            u_a |= self.cells[col + row_above_off] as u64;
            u   |= self.cells[col + row_off] as u64;
            u_b |= self.cells[col + row_below_off] as u64;

            // println!("{:#048b}\n{:#048b}\n{:#048b}", u_a, u, u_b);
            let mut result = 0_64;
            for _ in 0..=15 {
                let mut alive_cells = (u_a & m2) + (u & m2) + (u_b & m2);
                alive_cells >>= 30;
                alive_cells = (alive_cells & 0b11_u64) + (alive_cells  >> 2) +
                              ((u_a >> 31) & 0b1_u64) + ((u_b >> 31) & 0b1_u64);

                result <<= 1;
                if alive_cells == 3 ||
                  (alive_cells == 2 && (u & m1) == m1) {
                    result |= 0b1_u64;
                }
                u_a <<= 1;
                u   <<= 1;
                u_b <<= 1;
                // print!("{} ", alive_cells);
            }
            // println!();
            self.cells_shadow[curr_col + row_off] = result as u16;
        }

        std::mem::swap(&mut self.cells, &mut self.cells_shadow);
        self.cells_shadow.iter_mut().for_each(|i| *i = 0);
    }
}
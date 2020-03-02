extern crate rayon;

use rayon::prelude::*;

#[derive(Clone)]
pub struct Grid {
    pub cells : Vec<u16>,
    pub num_cols : usize,
    pub num_rows : usize
}

impl Grid {
    pub fn new (cell_size : f32, line_width : f32, board_size : (u32, u32)) -> Self {
        let size = (board_size.1 * board_size.0) as usize / (std::mem::size_of::<u16>() * 8) ;
        let cells = vec![0_u16 ; size];

        Self{cells : cells.clone(),
             num_cols : board_size.0 as usize,
             num_rows : board_size.1 as usize
        }
    }

    #[inline]
    fn coord_to_idx(self : &Self, col : usize, row : usize) -> usize {
        (self.num_cols / 16) * row  + (col / 16)
    }

    pub fn set_cell(self : &mut Self, col : usize, row : usize, value : bool) {
        if col < self.num_cols && row < self.num_rows {
            let idx = self.coord_to_idx(col, row);
            let col = col % 16;
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

        (0..self.cells.len())
        .into_par_iter()
        .map(|idx| {
            let mut col = (idx + num_cols_c - 1) % num_cols_c;
            let row_off = (idx / num_cols_c) * num_cols_c;
            let row_above_off = (row_off + grid_size_c - num_cols_c) % grid_size_c;
            let row_below_off = (row_off + grid_size_c + num_cols_c) % grid_size_c;

            let mut u_a = self.cells[col + row_above_off] as u64; // above u
            let mut u = self.cells[col + row_off] as u64;
            let mut u_b = self.cells[col + row_below_off] as u64; // below u

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
                alive_cells = (alive_cells & 0b11_u64) + (alive_cells >> 2) +
                              ((u_a >> 31) & 0b1_u64) + ((u_b >> 31) & 0b1_u64);

                result <<= 1;
                if alive_cells == 3 ||
                  (alive_cells == 2 && (u & m1) == m1) {
                    result |= 0b1_u64;
                }
                u_a <<= 1;
                u   <<= 1;
                u_b <<= 1;
            }
            (curr_col + row_off, result as u16)
        })
        .collect::<Vec<_>>()
        .iter()
        .for_each(|(idx, result)|{
            self.cells[*idx] = *result;
        });
    }
}
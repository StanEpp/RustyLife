extern crate bit_vec;
extern crate sfml;
extern crate rayon;

use bit_vec::BitVec;
use std::collections::HashSet;
use std::sync::{Arc, RwLock, RwLockReadGuard, Mutex, mpsc};
use sfml::graphics::{Vertex};
use sfml::system::{Vector2f};


const NUM_THREADS : usize = 4;


pub struct GridWorker {
    pub living_cells : HashSet<usize>,
    pub living_cells_tmp : HashSet<usize>,
    pub cells : Arc<RwLock<BitVec>>,
    pub sender : mpsc::Sender<usize>,
    pub col_range : (usize, usize),
    pub row_range : (usize, usize),
    pub num_cols : usize,
    pub num_rows : usize
}

pub struct Grid {
    pub cell_size : f32,
    pub line_width : f32,
    pub horizontal_lines: Vec<Vertex>,
    pub vertical_lines: Vec<Vertex>,

    pub living_cells : HashSet<usize>,
    pub living_cells_tmp : HashSet<usize>,
    pub cells : Arc<RwLock<BitVec>>,
    pub thread_pool : rayon::ThreadPool,
    pub worker : Vec<Arc<Mutex<GridWorker>>>,
    pub recv : Mutex<mpsc::Receiver<usize>>,
    pub num_cols : usize,
    pub num_rows : usize
}

#[inline]
fn coord_to_idx(num_cols : usize, col : usize, row : usize) -> usize {
    num_cols * row  + col
}

pub fn idx_to_coord(num_cols : usize, idx : usize) -> (usize, usize) {
    let row = idx / num_cols;
    let col = idx - (row * num_cols);
    (col, row)
}

struct GridPatch {
    pub start_col : usize,
    pub end_col : usize,
    pub start_row : usize,
    pub end_row : usize
}

fn subdivide_grid(num_cols : usize, num_rows : usize, n : usize) -> Option<Vec<GridPatch>> {
    // Check if sqrt(n) is an integer
    let e = 0.00001;
    let sqrt = (n as f64).sqrt();
    let frac = sqrt.fract();
    if frac >= e {
        return None;
    }

    // Check if grid is big enough
    let sqrt = sqrt as usize;
    if num_cols / sqrt < 1 || num_rows / sqrt < 1 {
        return None;
    }

    // Compute square patches
    let mut ret = Vec::new();
    let cols_per_worker = num_cols / sqrt as usize;
    let rows_per_worker = num_rows / sqrt as usize;
    for row in 0..sqrt-1 {
        for col in 0..sqrt-1 {
            ret.push(GridPatch{
                start_col : col * cols_per_worker,
                end_col :  col * cols_per_worker + cols_per_worker,
                start_row : row * rows_per_worker,
                end_row : row * rows_per_worker + rows_per_worker
            });
        }
        ret.push(GridPatch{
            start_col : (sqrt-1) * cols_per_worker,
            end_col : (sqrt-1) * cols_per_worker + cols_per_worker + num_cols % sqrt,
            start_row : row * rows_per_worker,
            end_row : row * rows_per_worker + rows_per_worker
        });
    }

    for col in 0..sqrt-1 {
        ret.push(GridPatch {
            start_col : col * cols_per_worker,
            end_col : col * cols_per_worker + cols_per_worker,
            start_row : (sqrt-1) * rows_per_worker,
            end_row : (sqrt-1) * rows_per_worker + rows_per_worker + num_rows % sqrt
        });
    }
    ret.push(GridPatch {
        start_col : (sqrt-1) * cols_per_worker,
        end_col : (sqrt-1) * cols_per_worker + cols_per_worker + num_cols % sqrt,
        start_row : (sqrt-1) * rows_per_worker,
        end_row : (sqrt-1) * rows_per_worker + rows_per_worker + num_rows % sqrt
    });

    Some(ret)
}

impl GridWorker {
    pub fn idx_to_coord(self : &Self, idx : usize) -> (usize, usize) {
        let row = idx / self.num_cols;
        let col = idx - (row * self.num_cols);
        (col, row)
    }

    fn get_surrounding_cell_idx(self : &Self, idx : usize) -> [usize; 9] {
        let mut keys = [0_usize; 9];
        let (col, row) = self.idx_to_coord(idx);

        keys[8] = idx;

        if col > 0 && col < self.num_cols-1 &&
           row > 0 && row < self.num_rows-1 {
            keys[0] = coord_to_idx(self.num_cols, col+1, row);
            keys[1] = coord_to_idx(self.num_cols, col-1, row);
            keys[2] = coord_to_idx(self.num_cols, col, row+1);
            keys[3] = coord_to_idx(self.num_cols, col, row-1);
            keys[4] = coord_to_idx(self.num_cols, col+1, row+1);
            keys[5] = coord_to_idx(self.num_cols, col-1, row+1);
            keys[6] = coord_to_idx(self.num_cols, col+1, row-1);
            keys[7] = coord_to_idx(self.num_cols, col-1, row-1);
        } else if col > 0 && col < self.num_cols-1 &&
                  row == 0 {
            keys[0] = coord_to_idx(self.num_cols, col, 1);
            keys[1] = coord_to_idx(self.num_cols, col+1, 1);
            keys[2] = coord_to_idx(self.num_cols, col-1, 1);
            keys[3] = coord_to_idx(self.num_cols, col-1, 0);
            keys[4] = coord_to_idx(self.num_cols, col+1, 0);
            keys[5] = coord_to_idx(self.num_cols, col, self.num_rows-1);
            keys[6] = coord_to_idx(self.num_cols, col+1, self.num_rows-1);
            keys[7] = coord_to_idx(self.num_cols, col-1, self.num_rows-1);
        } else if col > 0 && col < self.num_cols-1 &&
                  row == self.num_rows-1 {
            keys[0] = coord_to_idx(self.num_cols, col+1, self.num_rows-2);
            keys[1] = coord_to_idx(self.num_cols, col, self.num_rows-2);
            keys[2] = coord_to_idx(self.num_cols, col-1, self.num_rows-2);
            keys[3] = coord_to_idx(self.num_cols, col+1, self.num_rows-1);
            keys[4] = coord_to_idx(self.num_cols, col-1, self.num_rows-1);
            keys[5] = coord_to_idx(self.num_cols, col+1, 0);
            keys[6] = coord_to_idx(self.num_cols, col, 0);
            keys[7] = coord_to_idx(self.num_cols, col-1, 0);
        } else if col == 0 &&
                  row > 0 && row < self.num_rows-1 {
            keys[0] = coord_to_idx(self.num_cols, 1, row+1);
            keys[1] = coord_to_idx(self.num_cols, 1, row);
            keys[2] = coord_to_idx(self.num_cols, 1, row-1);
            keys[3] = coord_to_idx(self.num_cols, 0, row-1);
            keys[4] = coord_to_idx(self.num_cols, 0, row+1);
            keys[5] = coord_to_idx(self.num_cols, self.num_cols-1, row+1);
            keys[6] = coord_to_idx(self.num_cols, self.num_cols-1, row);
            keys[7] = coord_to_idx(self.num_cols, self.num_cols-1, row-1);
        } else if col == self.num_cols-1 &&
                  row > 0 && row < self.num_rows-1 {
            keys[0] = coord_to_idx(self.num_cols, self.num_cols-2, row+1);
            keys[1] = coord_to_idx(self.num_cols, self.num_cols-2, row);
            keys[2] = coord_to_idx(self.num_cols, self.num_cols-2, row-1);
            keys[3] = coord_to_idx(self.num_cols, self.num_cols-1, row-1);
            keys[4] = coord_to_idx(self.num_cols, self.num_cols-1, row+1);
            keys[5] = coord_to_idx(self.num_cols, 0, row+1);
            keys[6] = coord_to_idx(self.num_cols, 0, row);
            keys[7] = coord_to_idx(self.num_cols, 0, row-1);
        } else if col == 0 &&
                  row == 0 {
            keys[0] = coord_to_idx(self.num_cols, 1, 0);
            keys[1] = coord_to_idx(self.num_cols, 1, 1);
            keys[2] = coord_to_idx(self.num_cols, 0, 1);
            keys[3] = coord_to_idx(self.num_cols, 0, self.num_rows-1);
            keys[4] = coord_to_idx(self.num_cols, 1, self.num_rows-1);
            keys[5] = coord_to_idx(self.num_cols, self.num_cols-1, 0);
            keys[6] = coord_to_idx(self.num_cols, self.num_cols-1, 1);
            keys[7] = coord_to_idx(self.num_cols, self.num_cols-1, self.num_rows-1);
        } else if col == self.num_cols-1 &&
                  row == self.num_rows-1 {
            keys[0] = coord_to_idx(self.num_cols, self.num_cols-2, self.num_rows-1);
            keys[1] = coord_to_idx(self.num_cols, self.num_cols-2, self.num_rows-2);
            keys[2] = coord_to_idx(self.num_cols, self.num_cols-1, self.num_rows-2);
            keys[3] = coord_to_idx(self.num_cols, 0, self.num_rows-2);
            keys[4] = coord_to_idx(self.num_cols, 0, self.num_rows-1);
            keys[5] = coord_to_idx(self.num_cols, self.num_cols-1, 0);
            keys[6] = coord_to_idx(self.num_cols, self.num_cols-2, 0);
            keys[7] = coord_to_idx(self.num_cols, 0, 0);
        } else if col == 0 &&
                  row == self.num_rows-1 {
            keys[0] = coord_to_idx(self.num_cols, 0, self.num_rows-2);
            keys[1] = coord_to_idx(self.num_cols, 1, self.num_rows-2);
            keys[2] = coord_to_idx(self.num_cols, 1, self.num_rows-1);
            keys[3] = coord_to_idx(self.num_cols, self.num_cols-1, 0);
            keys[4] = coord_to_idx(self.num_cols, 0, 0);
            keys[5] = coord_to_idx(self.num_cols, 1, 0);
            keys[6] = coord_to_idx(self.num_cols, self.num_cols-1, self.num_rows-1);
            keys[7] = coord_to_idx(self.num_cols, self.num_cols-1, self.num_rows-2);
        } else if col == self.num_cols-1 &&
                  row == 0 {
            keys[0] = coord_to_idx(self.num_cols, self.num_cols-2, 0);
            keys[1] = coord_to_idx(self.num_cols, self.num_cols-2, 1);
            keys[2] = coord_to_idx(self.num_cols, self.num_cols-1, 1);
            keys[3] = coord_to_idx(self.num_cols, 0, 0);
            keys[4] = coord_to_idx(self.num_cols, 0, 1);
            keys[5] = coord_to_idx(self.num_cols, self.num_cols-1, self.num_rows-1);
            keys[6] = coord_to_idx(self.num_cols, self.num_cols-2, self.num_rows-1);
            keys[7] = coord_to_idx(self.num_cols, 0, self.num_rows-1);
        }

        keys
    }

    fn count_neighbors(self : &Self, col : usize, row : usize) -> Option<u8> {
        let mut num_neighbors = 0_u8;
        let cells = self.cells.read().unwrap();

        if col > 0 && col < self.num_cols-1 &&
           row > 0 && row < self.num_rows-1 {
            num_neighbors += cells[coord_to_idx(self.num_cols, col+1, row)] as u8;
            num_neighbors += cells[coord_to_idx(self.num_cols, col-1, row)] as u8;
            num_neighbors += cells[coord_to_idx(self.num_cols, col, row+1)] as u8;
            num_neighbors += cells[coord_to_idx(self.num_cols, col, row-1)] as u8;
            num_neighbors += cells[coord_to_idx(self.num_cols, col+1, row+1)] as u8;
            num_neighbors += cells[coord_to_idx(self.num_cols, col-1, row+1)] as u8;
            num_neighbors += cells[coord_to_idx(self.num_cols, col+1, row-1)] as u8;
            num_neighbors += cells[coord_to_idx(self.num_cols, col-1, row-1)] as u8;
        } else if col > 0 && col < self.num_cols-1 &&
                  row == 0 {
            num_neighbors += cells[coord_to_idx(self.num_cols, col, 1)] as u8;
            num_neighbors += cells[coord_to_idx(self.num_cols, col+1, 1)] as u8;
            num_neighbors += cells[coord_to_idx(self.num_cols, col-1, 1)] as u8;
            num_neighbors += cells[coord_to_idx(self.num_cols, col-1, 0)] as u8;
            num_neighbors += cells[coord_to_idx(self.num_cols, col+1, 0)] as u8;
            num_neighbors += cells[coord_to_idx(self.num_cols, col, self.num_rows-1)] as u8;
            num_neighbors += cells[coord_to_idx(self.num_cols, col+1, self.num_rows-1)] as u8;
            num_neighbors += cells[coord_to_idx(self.num_cols, col-1, self.num_rows-1)] as u8;
        } else if col > 0 && col < self.num_cols-1 &&
                  row == self.num_rows-1 {
            num_neighbors += cells[coord_to_idx(self.num_cols, col+1, self.num_rows-2)] as u8;
            num_neighbors += cells[coord_to_idx(self.num_cols, col, self.num_rows-2)] as u8;
            num_neighbors += cells[coord_to_idx(self.num_cols, col-1, self.num_rows-2)] as u8;
            num_neighbors += cells[coord_to_idx(self.num_cols, col+1, self.num_rows-1)] as u8;
            num_neighbors += cells[coord_to_idx(self.num_cols, col-1, self.num_rows-1)] as u8;
            num_neighbors += cells[coord_to_idx(self.num_cols, col+1, 0)] as u8;
            num_neighbors += cells[coord_to_idx(self.num_cols, col, 0)] as u8;
            num_neighbors += cells[coord_to_idx(self.num_cols, col-1, 0)] as u8;
        } else if col == 0 &&
                  row > 0 && row < self.num_rows-1 {
            num_neighbors += cells[coord_to_idx(self.num_cols, 1, row+1)] as u8;
            num_neighbors += cells[coord_to_idx(self.num_cols, 1, row)] as u8;
            num_neighbors += cells[coord_to_idx(self.num_cols, 1, row-1)] as u8;
            num_neighbors += cells[coord_to_idx(self.num_cols, 0, row-1)] as u8;
            num_neighbors += cells[coord_to_idx(self.num_cols, 0, row+1)] as u8;
            num_neighbors += cells[coord_to_idx(self.num_cols, self.num_cols-1, row+1)] as u8;
            num_neighbors += cells[coord_to_idx(self.num_cols, self.num_cols-1, row)] as u8;
            num_neighbors += cells[coord_to_idx(self.num_cols, self.num_cols-1, row-1)] as u8;
        } else if col == self.num_cols-1 &&
                  row > 0 && row < self.num_rows-1 {
            num_neighbors += cells[coord_to_idx(self.num_cols, self.num_cols-2, row+1)] as u8;
            num_neighbors += cells[coord_to_idx(self.num_cols, self.num_cols-2, row)] as u8;
            num_neighbors += cells[coord_to_idx(self.num_cols, self.num_cols-2, row-1)] as u8;
            num_neighbors += cells[coord_to_idx(self.num_cols, self.num_cols-1, row-1)] as u8;
            num_neighbors += cells[coord_to_idx(self.num_cols, self.num_cols-1, row+1)] as u8;
            num_neighbors += cells[coord_to_idx(self.num_cols, 0, row+1)] as u8;
            num_neighbors += cells[coord_to_idx(self.num_cols, 0, row)] as u8;
            num_neighbors += cells[coord_to_idx(self.num_cols, 0, row-1)] as u8;
        } else if col == 0 &&
                  row == 0 {
            num_neighbors += cells[coord_to_idx(self.num_cols, 1, 0)] as u8;
            num_neighbors += cells[coord_to_idx(self.num_cols, 1, 1)] as u8;
            num_neighbors += cells[coord_to_idx(self.num_cols, 0, 1)] as u8;
            num_neighbors += cells[coord_to_idx(self.num_cols, 0, self.num_rows-1)] as u8;
            num_neighbors += cells[coord_to_idx(self.num_cols, 1, self.num_rows-1)] as u8;
            num_neighbors += cells[coord_to_idx(self.num_cols, self.num_cols-1, 0)] as u8;
            num_neighbors += cells[coord_to_idx(self.num_cols, self.num_cols-1, 1)] as u8;
            num_neighbors += cells[coord_to_idx(self.num_cols, self.num_cols-1, self.num_rows-1)] as u8;
        } else if col == self.num_cols-1 &&
                  row == self.num_rows-1 {
            num_neighbors += cells[coord_to_idx(self.num_cols, self.num_cols-2, self.num_rows-1)] as u8;
            num_neighbors += cells[coord_to_idx(self.num_cols, self.num_cols-2, self.num_rows-2)] as u8;
            num_neighbors += cells[coord_to_idx(self.num_cols, self.num_cols-1, self.num_rows-2)] as u8;
            num_neighbors += cells[coord_to_idx(self.num_cols, 0, self.num_rows-2)] as u8;
            num_neighbors += cells[coord_to_idx(self.num_cols, 0, self.num_rows-1)] as u8;
            num_neighbors += cells[coord_to_idx(self.num_cols, self.num_cols-1, 0)] as u8;
            num_neighbors += cells[coord_to_idx(self.num_cols, self.num_cols-2, 0)] as u8;
            num_neighbors += cells[coord_to_idx(self.num_cols, 0, 0)] as u8;
        } else if col == 0 &&
                  row == self.num_rows-1 {
            num_neighbors += cells[coord_to_idx(self.num_cols, 0, self.num_rows-2)] as u8;
            num_neighbors += cells[coord_to_idx(self.num_cols, 1, self.num_rows-2)] as u8;
            num_neighbors += cells[coord_to_idx(self.num_cols, 1, self.num_rows-1)] as u8;
            num_neighbors += cells[coord_to_idx(self.num_cols, self.num_cols-1, 0)] as u8;
            num_neighbors += cells[coord_to_idx(self.num_cols, 0, 0)] as u8;
            num_neighbors += cells[coord_to_idx(self.num_cols, 1, 0)] as u8;
            num_neighbors += cells[coord_to_idx(self.num_cols, self.num_cols-1, self.num_rows-1)] as u8;
            num_neighbors += cells[coord_to_idx(self.num_cols, self.num_cols-1, self.num_rows-2)] as u8;
        } else if col == self.num_cols-1 &&
                  row == 0 {
            num_neighbors += cells[coord_to_idx(self.num_cols, self.num_cols-2, 0)] as u8;
            num_neighbors += cells[coord_to_idx(self.num_cols, self.num_cols-2, 1)] as u8;
            num_neighbors += cells[coord_to_idx(self.num_cols, self.num_cols-1, 1)] as u8;
            num_neighbors += cells[coord_to_idx(self.num_cols, 0, 0)] as u8;
            num_neighbors += cells[coord_to_idx(self.num_cols, 0, 1)] as u8;
            num_neighbors += cells[coord_to_idx(self.num_cols, self.num_cols-1, self.num_rows-1)] as u8;
            num_neighbors += cells[coord_to_idx(self.num_cols, self.num_cols-2, self.num_rows-1)] as u8;
            num_neighbors += cells[coord_to_idx(self.num_cols, 0, self.num_rows-1)] as u8;
        } else {
            return None;
        }

        Some(num_neighbors)
    }

    fn set_living_cell(self : &mut Self, col : usize, row : usize) {
        if col >= self.col_range.0 && col < self.col_range.1 &&
           row >= self.row_range.0 && row < self.row_range.1 {
            self.living_cells.insert(coord_to_idx(self.num_cols, col, row));
        }
    }

    pub fn cell_in_range(self : &Self, col : usize, row : usize) -> bool {
        if col >= self.col_range.0 && col < self.col_range.1 &&
           row >= self.row_range.0 && row < self.row_range.1 {
            true
        } else {
            false
        }
    }

    pub fn idx_in_range(self : &Self, idx : usize) -> bool {
        let (col, row) = self.idx_to_coord(idx);
        self.cell_in_range(col, row)
    }

    fn rule_result(self : &Self, idx : usize, cells : &RwLockReadGuard<BitVec>) -> Option<bool> {
        let (col, row) = self.idx_to_coord(idx);

        match self.count_neighbors(col, row) {
            Some(num_neighbors) => {
                if cells[idx] == true {
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

    pub fn run_lifecycle(self : &mut Self) {
        let cells = self.cells.read().unwrap();

        for idx in &self.living_cells {
            let indices = self.get_surrounding_cell_idx(*idx);
            for idx in &indices {
                if self.idx_in_range(*idx) {
                    if self.rule_result(*idx, &cells).unwrap() {
                        self.living_cells_tmp.insert(*idx);
                        self.sender.send(*idx);
                    }
                }
            }
        }

        for col in self.col_range.0..self.col_range.1 {
            let idx = coord_to_idx(self.num_cols, col, self.row_range.0);
            if self.rule_result(idx, &cells).unwrap() {
                self.living_cells_tmp.insert(idx);
                self.sender.send(idx);
            }
            let idx = coord_to_idx(self.num_cols, col, self.row_range.1-1);
            if self.rule_result(idx, &cells).unwrap() {
                self.living_cells_tmp.insert(idx);
                self.sender.send(idx);
            }
        }

        for row in self.row_range.0..self.row_range.1 {
            let idx = coord_to_idx(self.num_cols, self.col_range.0, row);
            if self.rule_result(idx, &cells).unwrap() {
                self.living_cells_tmp.insert(idx);
                self.sender.send(idx);
            }
            let idx = coord_to_idx(self.num_cols, self.col_range.1-1, row);
            if self.rule_result(idx, &cells).unwrap() {
                self.living_cells_tmp.insert(idx);
                self.sender.send(idx);
            }
        }

        self.living_cells.clear();

        std::mem::swap(&mut self.living_cells, &mut self.living_cells_tmp);
    }
}

impl Grid {
    pub fn new (cell_size : f32, line_width : f32, board_size : (usize, usize)) -> Self {
        let num_cols = board_size.0;
        let num_rows = board_size.1;

        let mut hl = Vec::new();
        let top_left_x = (num_cols as f32 / 2.0_f32).floor() * (cell_size + line_width) +
                       (num_cols % 2) as f32 * line_width / 2.0_f32 +
                       (num_cols % 2) as f32 * (cell_size / 2.0_f32 + line_width);
        let top_left_x = top_left_x * -1.;

        let top_left_y = (num_rows as f32 / 2.0_f32).floor() * (cell_size + line_width) +
                       (num_rows % 2) as f32 * line_width as f32 / 2.0_f32 +
                       (num_rows % 2) as f32 * (cell_size / 2.0_f32 + line_width);
        let top_left_y = top_left_y * -1.;

        for i in 0..=num_rows {
            let off = i as f32 * (cell_size + line_width);
            hl.push(Vertex::with_pos(Vector2f::new(top_left_x, top_left_y + off)));
            hl.push(Vertex::with_pos(Vector2f::new(top_left_x, top_left_y + line_width + off)));
            hl.push(Vertex::with_pos(Vector2f::new(top_left_x.abs() + line_width, top_left_y + line_width + off)));
            hl.push(Vertex::with_pos(Vector2f::new(top_left_x.abs() + line_width, top_left_y + off)));
        }

        let mut vl = Vec::new();
        for i in 0..=num_cols {
            let off = i as f32 * (cell_size + line_width);
            vl.push(Vertex::with_pos(Vector2f::new(top_left_x + off, top_left_y)));
            vl.push(Vertex::with_pos(Vector2f::new(top_left_x + off, top_left_y * -1. + line_width)));
            vl.push(Vertex::with_pos(Vector2f::new(top_left_x + line_width + off, top_left_y * -1. + line_width)));
            vl.push(Vertex::with_pos(Vector2f::new(top_left_x + line_width + off, top_left_y)));
        }

        let cells = Arc::new(RwLock::new(BitVec::from_elem((num_rows * num_cols) as usize, false)));

        let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(NUM_THREADS)
        .thread_name(|idx|{ format!["GridWorker#{}", idx] })
        .build().unwrap();

        let (sender, receiver) = mpsc::channel();


        let mut workers = Vec::<Arc<Mutex<GridWorker>>>::new();
        let patches = subdivide_grid(num_cols, num_rows, NUM_THREADS).unwrap();
        for patch in patches {
            workers.push(Arc::new(Mutex::new(GridWorker{
                living_cells : HashSet::new(),
                living_cells_tmp : HashSet::new(),
                cells : cells.clone(),
                sender : sender.clone(),
                col_range : (patch.start_col, patch.end_col),
                row_range : (patch.start_row, patch.end_row),
                num_cols : num_cols,
                num_rows : num_rows
            })));
        }

        Self{cell_size : cell_size,
             line_width : line_width,
             horizontal_lines : hl,
             vertical_lines : vl,
             cells : cells,
             thread_pool : pool,
             worker : workers,
             living_cells : HashSet::new(),
             living_cells_tmp : HashSet::new(),
             recv : Mutex::new(receiver),
             num_cols : num_cols,
             num_rows : num_rows
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
        self.cells.read().unwrap()[coord_to_idx(self.num_cols, col, row)]
    }

    fn coord_to_worker(self : &Self, col : usize, row : usize) -> usize {
        let sqrt = (NUM_THREADS as f64).sqrt() as usize;
        let cols_per_worker = self.num_cols / sqrt;
        let rows_per_worker = self.num_rows / sqrt;
        sqrt * (row / rows_per_worker) + (col / cols_per_worker)
    }

    pub fn set_cell(self : &mut Self, col : usize, row : usize, value : bool) {
        let mut cells = self.cells.write().unwrap();
        if col < self.num_cols && row < self.num_rows {
            let idx = coord_to_idx(self.num_cols, col, row);
            cells.set(idx, value);
            if value {
                let worker_idx = self.coord_to_worker(col, row);
                self.worker[worker_idx].lock().unwrap().set_living_cell(col, row);
                self.living_cells.insert(idx);
            }
        }
    }

    pub fn run_lifecycle(self : &mut Self) {
        // rayon::scope(|s|{
        //     for idx in 0..NUM_THREADS {
        //         let worker = self.worker[idx].clone();
        //         s.spawn(move |_|{ worker.lock().unwrap().run_lifecycle(); });
        //     }
        // });

        let mut arr = vec![];
        for i in 0..NUM_THREADS {
            let worker = self.worker[i].clone();
            arr.push(std::thread::spawn(move ||{ worker.lock().unwrap().run_lifecycle(); }));
        }
        for h in arr {
            let _ = h.join();
        }

        // println!("FINISHED!");

        let mut cells = self.cells.write().unwrap();
        for idx in &self.living_cells {
            cells.set(*idx, false);
        }
        self.living_cells.clear();

        let recv = self.recv.lock().unwrap();
        let mut i = 0;
        loop {
            let r = recv.try_recv();
            if r.is_err() {
                break;
            }
            i += 1;
            let idx = r.unwrap();
            cells.set(idx, true);
            self.living_cells_tmp.insert(idx);
        }
        std::mem::swap(&mut self.living_cells, &mut self.living_cells_tmp);
        // println!("FINISHED 22!");
    }
}










#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_subdivied_grid() {
        // Check invalid arguments
        let patches = subdivide_grid(100, 100, 2);
        assert!(patches.is_none());
        let patches = subdivide_grid(100, 100, 5);
        assert!(patches.is_none());
        let patches = subdivide_grid(100, 100, 8);
        assert!(patches.is_none());
        let patches = subdivide_grid(4, 3, 16);
        assert!(patches.is_none());
        let patches = subdivide_grid(3, 4, 16);
        assert!(patches.is_none());
        let patches = subdivide_grid(3, 3, 16);
        assert!(patches.is_none());


        let patches = subdivide_grid(100, 100, 4);
        assert!(patches.is_some());
        let patches = patches.unwrap();

        assert_eq!(patches.len(), 4);
        assert_eq!(patches[0].start_col, 0);
        assert_eq!(patches[0].end_col, 50);
        assert_eq!(patches[0].start_row, 0);
        assert_eq!(patches[0].end_row, 50);
        assert_eq!(patches[1].start_col, 50);
        assert_eq!(patches[1].end_col, 100);
        assert_eq!(patches[1].start_row, 0);
        assert_eq!(patches[1].end_row, 50);
        assert_eq!(patches[2].start_col, 0);
        assert_eq!(patches[2].end_col, 50);
        assert_eq!(patches[2].start_row, 50);
        assert_eq!(patches[2].end_row, 100);
        assert_eq!(patches[3].start_col, 50);
        assert_eq!(patches[3].end_col, 100);
        assert_eq!(patches[3].start_row, 50);
        assert_eq!(patches[3].end_row, 100);

        let patches = subdivide_grid(524, 647, 9);
        assert!(patches.is_some());
        let patches = patches.unwrap();

        assert_eq!(patches.len(), 9);
        assert_eq!(patches[0].start_col, 0);
        assert_eq!(patches[0].end_col, 174);
        assert_eq!(patches[0].start_row, 0);
        assert_eq!(patches[0].end_row, 215);
        assert_eq!(patches[1].start_col, 174);
        assert_eq!(patches[1].end_col, 348);
        assert_eq!(patches[1].start_row, 0);
        assert_eq!(patches[1].end_row, 215);
        assert_eq!(patches[2].start_col, 348);
        assert_eq!(patches[2].end_col, 524);
        assert_eq!(patches[2].start_row, 0);
        assert_eq!(patches[2].end_row, 215);

        assert_eq!(patches[3].start_col, 0);
        assert_eq!(patches[3].end_col, 174);
        assert_eq!(patches[3].start_row, 215);
        assert_eq!(patches[3].end_row, 430);
        assert_eq!(patches[4].start_col, 174);
        assert_eq!(patches[4].end_col, 348);
        assert_eq!(patches[4].start_row, 215);
        assert_eq!(patches[4].end_row, 430);
        assert_eq!(patches[5].start_col, 348);
        assert_eq!(patches[5].end_col, 524);
        assert_eq!(patches[5].start_row, 215);
        assert_eq!(patches[5].end_row, 430);

        assert_eq!(patches[6].start_col, 0);
        assert_eq!(patches[6].end_col, 174);
        assert_eq!(patches[6].start_row, 430);
        assert_eq!(patches[6].end_row, 647);
        assert_eq!(patches[7].start_col, 174);
        assert_eq!(patches[7].end_col, 348);
        assert_eq!(patches[7].start_row, 430);
        assert_eq!(patches[7].end_row, 647);
        assert_eq!(patches[8].start_col, 348);
        assert_eq!(patches[8].end_col, 524);
        assert_eq!(patches[8].start_row, 430);
        assert_eq!(patches[8].end_row, 647);
    }
}
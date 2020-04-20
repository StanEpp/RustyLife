use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;

#[derive(Default)]
pub struct Pattern {
    pub pattern : std::vec::Vec<(usize, usize)>,
    pub width : usize,
    pub height : usize,
    pub name : String,
    pub author : String,
}


pub fn read_rle<P>(filepath : P) -> Option<Pattern>
    where P: AsRef<Path> {
    let filepath_str = String::from(filepath.as_ref().to_str().unwrap());
    let file = match File::open(filepath) {
        Ok(f) => f,
        Err(err) => {
            println!("Error reading RLE file \"{}\": \n\t{}", filepath_str, err);
            return None;
        }
    };

    let lines = io::BufReader::new(file).lines();

    let mut p = Pattern::default();

    let mut header_finished = false;
    let mut row = 0;
    let mut col = 0;

    'scan_lines: for line in lines {
        let l = match line {
            Ok(l) => l,
            Err(_) => continue,
        };
        let l = l.trim();
        let mut last_idx : i32 = -1;

        if header_finished {
            let l : String = l.split_whitespace().collect();
            for c in l.char_indices() {
                match c {
                    (_, '0'..='9') => (),
                    (idx, '$') => {
                        let start_idx = (last_idx + 1) as usize;
                        last_idx = idx as i32;
                        if start_idx == idx {
                            row += 1;
                        } else {
                            let num : usize = l.get(start_idx..idx).unwrap().parse().unwrap();
                            row += num;
                        }
                        col = 0;
                    },
                    (idx, 'o') => {
                        let start_idx = (last_idx + 1) as usize;
                        if start_idx == idx {
                            p.pattern.push((col, row));
                            col += 1;
                        } else {
                            let num : usize = l.get(start_idx..idx).unwrap().parse().unwrap();
                            for k in 0..num {
                                p.pattern.push((col + k, row));
                            }
                            col += num;
                        }
                        last_idx = idx as i32;
                    },
                    (idx, 'b') => {
                        let start_idx = (last_idx + 1) as usize;
                        if start_idx == idx {
                            col += 1;
                        } else {
                            let num : usize = l.get(start_idx..idx).unwrap().parse().unwrap();
                            col += num;
                        }
                        last_idx = idx as i32;
                    },
                    (_, '!') => break 'scan_lines,
                    (_, err_c) => {
                        println!("Error reading RLE file \"{}\": ", filepath_str);
                        println!("\tInvalid character '{}'", err_c);
                        return None;
                    }
                }
            }
        } else {
            match l.get(..2) {
                Some("#O") => p.author = String::from(l.get(2..).unwrap()),
                Some("#N") => p.name = String::from(l.get(2..).unwrap()),
                Some(tstr) => {
                    if tstr.get(..1).unwrap() == "x" {
                        let l : String = l.split_whitespace().collect();
                        let mut it = l.split(',');
                        p.width = it.next().unwrap().trim_matches(|c| c == 'x' || c == '=').parse().unwrap();
                        p.height = it.next().unwrap().trim_matches(|c| c == 'y' || c == '=').parse().unwrap();
                        header_finished = true;
                    }
                },
                _ => (),
            }
        }
    }

    Some(p)
}
use text_io::scan;
use std::{io::BufRead};
use sprs::{CsMat};
use bitflags::bitflags;

type Pattern = (i32, i32, i32);
type Piece = (usize, usize, Pattern);

type Prio = usize;
const NO_PRIO: Prio = std::usize::MAX;

pub struct SpGSearxMatrix {
    value_matrix: CsMat<bool>,
    // exploration_matrix: Vec<Prio>,
    pub nonzeros: usize,
    pub numrows: usize,
    pub numcols: usize,
    patterns: Vec<Pattern>,
    found_pieces: Vec<Piece>,
}

bitflags! {
    pub struct SpGSearxPatternsFlags: u64 {
        const NoFlags               = 0b0000_0000;
    }
}

impl SpGSearxMatrix {
    pub fn from_file(path: &str) -> SpGSearxMatrix {
        let f64_value_matrix: CsMat<f64> = sprs::io::read_matrix_market(path).unwrap().to_csc();

        // println!("{:?}", (f64_value_matrix.rows(), f64_value_matrix.cols(), f64_value_matrix.nnz()));
        // f64_value_matrix.iter().for_each(|(&val, (row, col))| {
        //     println!("{}: [{},{}]", val, row, col);
        // });

        let mut value_matrix = CsMat::empty(f64_value_matrix.storage(), f64_value_matrix.inner_dims());

        f64_value_matrix.iter().for_each(|(_, (row, col))| {
            value_matrix.insert(row, col, false);
        });

        let (numrows, numcols) = (value_matrix.rows(), value_matrix.cols());
        let nonzeros = value_matrix.nnz();

        // println!("{:?}", (numrows, numcols, nonzeros));
        // value_matrix.iter().for_each(|(&val, (row, col))| {
        //     println!("{}: [{},{}]", val, row, col);
        // });


        return SpGSearxMatrix {
            value_matrix: value_matrix,
            // exploration_matrix: exploration_matrix,
            nonzeros: nonzeros,
            numrows: numrows,
            numcols: numcols,
            patterns: vec![],
            found_pieces: vec![],
        };
    }

    pub fn load_patterns(&mut self, patterns_file_path: &str) {
        // Open patterns file
        let patterns_file = std::fs::File::open(patterns_file_path).unwrap();
        let lines: Vec<String> = std::io::BufReader::new(patterns_file).lines().collect::<Result<_,_>>().unwrap();

        // Set patterns
        self.patterns = lines
            .iter()
            .map(|x| {
                let (i,j,k): Pattern;
                scan!(x.bytes() => "({},{},{})", i, j, k);
    
                (i, j, k)
            })
            .collect();
    }

    pub fn print_patterns(&self) {
        println!("--- Patterns ---");
        println!("N\tI\tJ");
        self.patterns.iter().for_each(|&(i,j,k)| {
            println!("{}\t{}\t{}", i, j, k);
        });
    }

    pub fn search_patterns(&mut self, flags: SpGSearxPatternsFlags) {
        // Parse possible flags
        // let skip_on_invalidation = flags.contains(SpGSearxPatternsFlags::SkipOnInvalidation);
        
        // Generate access positions
        let mut nonzero_positions: Vec<(usize, usize)> = Vec::new();
        self.value_matrix.iter().for_each(|(_, (row, col))| {
            nonzero_positions.push((row,col));
        });
        
        // First pass looking for patterns
        nonzero_positions.iter().enumerate().for_each(|(_it, &(row, col))| {
            let mut found: bool = false;
            let mut piece: Piece = (0,0,(0,0,0));
            'pattern_search: for pattern in self.patterns.iter(){
                let result = check_pattern(&self.value_matrix, (row,col), pattern);
                match result {
                    None => continue,
                    Some(found_piece) => {
                        found = true;
                        piece = found_piece;
                        break 'pattern_search;
                    },
                }
            }

            if found {
                // set places to found and add piece
                let (x,y,(n,i,j)) = piece;
                for ii in 0..n {
                    let pos_val = self.value_matrix.get_mut((x as i64 + (i as i64 * ii as i64)) as usize, (y as i64 + (j as i64 * ii as i64)) as usize).unwrap();
                    *pos_val = true;
                }

                self.found_pieces.push(piece);
            }
        });

        // Add last nonzeros
        self.value_matrix.iter().for_each(|(&val, (row, col))| {
            if !val {
                self.found_pieces.push((row, col, (1, 0, 0)));
            }
        });

    }

    pub fn print_pieces(&self) {
        println!("Row\tCol\tN\tI\tJ");
    
        self.found_pieces.iter().for_each(|(row, col, (n, i, j))| {
            println!("{}\t{}\t{}\t{}\t{}", row, col, n, i, j);
        });
    }

    pub fn get_piece_list(&self) -> Vec<Piece> {
        return self.found_pieces.clone();
    }

}

#[inline(always)]
#[allow(dead_code)]
fn check_pattern(csmat: &CsMat<bool>, curr_pos: (usize, usize), pattern: &Pattern) -> Option<Piece> {
    // println!("Checking pattern {:?}", pattern);
    let &(n,i,j) = pattern;
    let (x,y) = curr_pos;

    // println!("{:?}", (x,y,n,i,j));

    let max_pos_x = x as i64 + n as i64 * i as i64;
    let max_pos_y = y as i64 + n as i64 * j as i64;

    // Discard out-of-bounds patterns
    if *csmat.get(x,y).unwrap() || max_pos_x < 0 || max_pos_x >= csmat.rows() as i64 || max_pos_y < 0 || max_pos_y >= csmat.cols() as i64 {
        return None;
    }

    for ii in 0..n {
        let position = csmat.get((x as i64 + (i as i64 * ii as i64)) as usize, (y as i64 + (j as i64 * ii as i64)) as usize);
        match position {
            Some(&is_in_pat) => {
                if is_in_pat {
                    return None;
                } else {
                    continue;
                }
            },
            None    => return None,
        }
    }

    return Some((x,y,(n,i,j)));
}
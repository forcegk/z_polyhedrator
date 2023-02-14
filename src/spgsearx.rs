use text_io::scan;
use std::{io::BufRead};
use sprs::{CsMat};
use bitflags::bitflags;

type Pattern = (i32, i32, i32);
type Piece = (usize, usize, Pattern);

type Prio = usize;
const NO_PRIO: Prio = std::usize::MAX;

pub struct SpGSearxMatrix {
    value_matrix: CsMat<f64>,
    exploration_matrix: Vec<Prio>,
    nonzeros: usize,
    numrows: usize,
    numcols: usize,
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
        let value_matrix = sprs::io::read_matrix_market(path).unwrap().to_csr();
        let (numrows, numcols) = (value_matrix.rows(), value_matrix.cols());
        let nonzeros = value_matrix.nnz();
        let exploration_matrix = vec![NO_PRIO; nonzeros];

        return SpGSearxMatrix {
            value_matrix: value_matrix,
            exploration_matrix: exploration_matrix,
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
        
        // First pass looking for patterns
        self.value_matrix.iter().enumerate().for_each(|(_it, (_, (row, col)))| {
            // let value = self.value_matrix.get(row, col);
            // self.patterns.iter().for_each(|pattern|{
            let mut found: bool = false;
            let mut piece: Piece = (0,0,(0,0,0));
            for pattern in self.patterns.iter(){
                let result = check_pattern(&self.value_matrix, (row,col), pattern);
                match result {
                    None => continue,
                    Some(found_piece) => {
                        found = true;
                        piece = found_piece;
                        break;
                    },
                }
            }

            if found {
                // set places to found and add piece
            }

        });
    }

    pub fn print_pieces(&self) {
        println!("Row\tCol\tN\tI\tJ");
    
        self.found_pieces.iter().for_each(|(row, col, (n, i, j))| {
            println!("{}\t{}\t{}\t{}\t{}", row, col, n, i, j);
        });
    }

}

#[inline(always)]
fn check_pattern(csmat: &CsMat<f64>, curr_pos: (usize, usize), pattern: &Pattern) -> Option<Piece> {
    // println!("Checking pattern {:?}", pattern);
    let &(n,i,j) = pattern;
    let (x,y) = curr_pos;

    // Discard out-of-bounds patterns
    if (x as i64 + n as i64 * i as i64) >= csmat.rows() as i64 || (y as i64 + n as i64 * j as i64) >= csmat.cols() as i64 {
        return None;
    }

    for ii in 0..n {
        let position = csmat.get(x as usize + (i as i64 * ii as i64) as usize, y as usize + (j as i64 * ii as i64) as usize);
        match position {
            Some(_) => continue,
            None    => return None,
        }
    }

    return Some((x,y,(n,i,j)));
}
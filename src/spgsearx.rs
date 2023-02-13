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
        
        
    }

    pub fn print_pieces(&self) {
        println!("Row\tCol\tN\tI\tJ");
    
        self.found_pieces.iter().for_each(|(row, col, (n, i, j))| {
            println!("{}\t{}\t{}\t{}\t{}", row, col, n, i, j);
        });
    }

}
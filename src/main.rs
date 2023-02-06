extern crate sprs;
extern crate text_io;

use std::{env, io::BufRead};
use sprs::TriMatI;
use text_io::scan;

type Priority = u32;
type Piece = (u32, i32, i32);

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 3 {
        println!("Usage: {} <patterns_file> <matrixmarket_file>", args[0]);
        return;
    }

    let patterns_file_path = &args[1];
    let matrixmarket_file_path = &args[2];

    /* -------- PARSE -------- */

    println!("Opening patterns file: {}", patterns_file_path);
    let patterns_file = std::fs::File::open(patterns_file_path).unwrap();
    let lines: Vec<String> = std::io::BufReader::new(patterns_file).lines().collect::<Result<_,_>>().unwrap();
    let patterns: Vec<Piece> = lines
        .iter()
        .map(|x| {
            let (i,j,k): Piece;
            scan!(x.bytes() => "({},{},{})", i, j, k);

            (i, j, k)
    })
    .collect();

    println!("Opening matrixmarket file: {}", matrixmarket_file_path);
    let base_matrix: TriMatI<f64,usize> = sprs::io::read_matrix_market(matrixmarket_file_path).unwrap();
    let nonzeros: usize = base_matrix.nnz();


    let mut explored_prio: Vec<Priority> = Vec::with_capacity(nonzeros);
    
}

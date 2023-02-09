extern crate sprs;
extern crate text_io;

use std::{env};

use crate::spsearx::{BaseMatrix,SearchPatternsFlags};

mod spsearx;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 3 {
        println!("Usage: {} <patterns_file> <matrixmarket_file>", args[0]);
        return;
    }
    let patterns_file_path = &args[1];
    let matrixmarket_file_path = &args[2];

    /* -------- PARSE -------- */
    println!("Opening matrixmarket file: {}", matrixmarket_file_path);
    let mut base_matrix: BaseMatrix = BaseMatrix::from(matrixmarket_file_path);
    
    println!("Opening patterns file: {}", patterns_file_path);
    base_matrix.load_patterns(patterns_file_path);

    // base_matrix.print_patterns();

    base_matrix.search_patterns(SearchPatternsFlags::NoFlags
        | SearchPatternsFlags::SkipOnInvalidation
        | SearchPatternsFlags::SkipOnPatternSearch
        | SearchPatternsFlags::PrintInformation
    );

    println!("\n\n---------------------------------------------------------\n");
    base_matrix.print_pieces();
}

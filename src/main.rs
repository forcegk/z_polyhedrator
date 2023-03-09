extern crate sprs;
extern crate text_io;

use std::{env};

mod spisearx;
use spisearx::{SpISearxMatrix,SpISearxPatternsFlags};

mod spgsearx;
use spgsearx::{SpGSearxMatrix,SpGSearxPatternsFlags};

use crate::spfgen::SPFGen;

mod spfgen;

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
    // let mut base_matrix: SpISearxMatrix = SpISearxMatrix::from_file(matrixmarket_file_path);
    let mut base_matrix: SpGSearxMatrix = SpGSearxMatrix::from_file(matrixmarket_file_path);
    
    println!("Opening patterns file: {}", patterns_file_path);
    base_matrix.load_patterns(patterns_file_path);

    // base_matrix.print_patterns();

    // base_matrix.search_patterns(spisearx::SpISearxPatternsFlags::NoFlags
    //     | spisearx::SpISearxPatternsFlags::SkipOnInvalidation
    //     | spisearx::SpISearxPatternsFlags::SkipOnPatternSearch
    //     // | spisearx::SpISearxPatternsFlags::PrintInformation
    // );

    base_matrix.search_patterns(spgsearx::SpGSearxPatternsFlags::NoFlags);

    println!("\n\n---------------------------------------------------------\n");
    // base_matrix.print_pieces();
    // println!("DONE");

    let spfgen = SPFGen::from_piece_list(base_matrix.get_piece_list());

    spfgen.print_ast_list();

    spfgen.print_uwc_list();

}

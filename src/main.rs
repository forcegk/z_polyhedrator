extern crate sprs;
extern crate text_io;

use std::{env, path::PathBuf};

mod spisearx;
#[allow(unused_imports)]
use spisearx::{SpISearxMatrix,SpISearxPatternsFlags};

mod spgsearx;
#[allow(unused_imports)]
use spgsearx::{SpGSearxMatrix,SpGSearxPatternsFlags};

mod spfgen;
#[allow(unused_imports)]
use spfgen::SPFGen;

mod utils;

fn main() {
    let args: Vec<String> = env::args().collect();

    // if args.len() < 3 {
    //     println!("Usage: {} <patterns_file> <matrixmarket_file> [<output_file>]", args[0]);
    //     return;
    // }
    // let patterns_file_path = &args[1];
    // let matrixmarket_file_path = &args[2];

    let flags = xflags::parse_or_exit! {
        /// File containing pattern list
        required patterns_file_path: PathBuf

        /// Input MatrixMarket file
        required matrixmarket_file_path: PathBuf
        
        /// Write to custom SPF file. By default writes to matrix_market_file.mtx.spf
        optional -w,--write-spf output_spf_file_path: PathBuf
    };

    let patterns_file_path = flags.patterns_file_path.to_str().unwrap();
    let matrixmarket_file_path = flags.matrixmarket_file_path.to_str().unwrap();
    
    let output_spf_file_path: String;
    match flags.write_spf {
        Some(path) => output_spf_file_path = String::from(path.to_str().unwrap()),
        None => output_spf_file_path = {
            let mut lstr = String::from(flags.matrixmarket_file_path.file_name().unwrap().to_str().unwrap());
            lstr.push_str(".spf");
            lstr
        }
    }

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

    base_matrix.search_patterns(spgsearx::SpGSearxPatternsFlags::NoFlags
        | spgsearx::SpGSearxPatternsFlags::PatternFirst
        // | spgsearx::SpGSearxPatternsFlags::CellFirst
    );

    println!("\n\n---------------------------------------------------------\n");
    // base_matrix.print_pieces();

    let spfgen = SPFGen::from_piece_list(base_matrix.get_piece_list(), base_matrix.numrows, base_matrix.numcols, base_matrix.nonzeros);

    spfgen.print_ast_list();

    spfgen.print_uwc_list(true);

    spfgen.print_distinct_uwc_list(true);

    spfgen.write_spf(matrixmarket_file_path, output_spf_file_path.as_str());
}

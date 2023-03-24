extern crate sprs;
extern crate text_io;

use std::{path::PathBuf, process::exit};
use colored::Colorize;

mod spsearx;
#[allow(unused_imports)]
use spsearx::{SpSearxMatrix,SpSearxPatternsFlags};

mod spaugment;
#[allow(unused_imports)]

mod spfgen;
#[allow(unused_imports)]
use spfgen::SPFGen;

mod utils;

fn main() {
    let flags = xflags::parse_or_exit! {
        /// File containing pattern list
        required patterns_file_path: PathBuf

        /// Input MatrixMarket file
        required matrixmarket_file_path: PathBuf

        /// Print patterns parsed from pattern list
        optional --print-pattern-list

        /// [2D SEARCH] Search Flags. Valid options: {[PatternFirst], CellFirst} where [] = default.
        optional --search-flags search_flags: String

        /// Write to custom SPF file. By default writes to matrix_market_file.mtx.spf
        optional -w,--write-spf output_spf_file_path: PathBuf

        /// Print piece list (AST list)
        optional --print-ast-list

        /// Print uwc lists
        optional --print-uwc-list

        /// Augment dimensionality
        optional -a, --augment-dimensionality augment_dimensionality: usize
    };

    let patterns_file_path = flags.patterns_file_path.to_str().unwrap();
    let matrixmarket_file_path = flags.matrixmarket_file_path.to_str().unwrap();
    
    let output_spf_file_path: (bool, String);
    output_spf_file_path = {
        if flags.write_spf.as_ref().is_some() {
            (true, String::from(flags.write_spf.unwrap().to_str().unwrap()))
        } else {
            (false, String::new())
        }
    };

    let search_flags = {
        let mut l_search_flags = spsearx::SpSearxPatternsFlags::NoFlags;

        if flags.search_flags.is_some() {
            match flags.search_flags.unwrap().as_str() {
                "PatternFirst" => l_search_flags |= spsearx::SpSearxPatternsFlags::PatternFirst,
                "CellFirst" => l_search_flags |= spsearx::SpSearxPatternsFlags::CellFirst,
                def => {
                    eprintln!("invalid value `{}` for `--search-flags`. Valid options: {{[PatternFirst], CellFirst}} where [] = default.", def);
                    exit(-1);
                }
            }
        } else {
            l_search_flags |= spsearx::SpSearxPatternsFlags::PatternFirst;
        }

        l_search_flags
    };

    /* -------- PARSE -------- */
    eprintln!("{} Opening matrixmarket file: {}", "[INFO]".cyan().bold(), matrixmarket_file_path);
    let mut base_matrix: SpSearxMatrix = SpSearxMatrix::from_file(matrixmarket_file_path);
    
    eprintln!("{} Opening patterns file: {}", "[INFO]".cyan().bold(), patterns_file_path);
    base_matrix.load_patterns(patterns_file_path);


    if flags.print_pattern_list {
        eprintln!("--- Pattern list ---");
        base_matrix.print_patterns();
    }

    base_matrix.search_patterns(search_flags);

    if flags.print_ast_list {
        base_matrix.print_pieces();
    }

    let augment_dimensionality: usize = {
        if flags.augment_dimensionality.is_some() {
            flags.augment_dimensionality.unwrap()
        } else {
            1usize
        }
    };

    if augment_dimensionality > 1 {
        // Augment dimensionality
    }

    if flags.print_uwc_list || output_spf_file_path.0 {
        let spfgen = SPFGen::from_piece_list(base_matrix.get_piece_list(), base_matrix.numrows, base_matrix.numcols, base_matrix.nonzeros);

        // Already done before
        // if flags.print_ast_list {
        //     spfgen.print_ast_list();
        // }

        if flags.print_uwc_list {
            spfgen.print_uwc_list(true);
            spfgen.print_distinct_uwc_list(true);
        }

        if output_spf_file_path.0 {
            spfgen.write_spf(matrixmarket_file_path, output_spf_file_path.1.as_str());
        }
    }
}

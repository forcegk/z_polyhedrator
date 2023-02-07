extern crate sprs;
extern crate text_io;

use std::{env, io::BufRead};
use sprs::{TriMatI, TriMat, CsMat};
use text_io::scan;

type Priority = u32;
type Piece = (i32, i32, i32);

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
    let base_matrix: CsMat<f64> = sprs::io::read_matrix_market(matrixmarket_file_path).unwrap().to_csr();
    let nonzeros: usize = base_matrix.nnz();
    let numrows: usize = base_matrix.rows();
    let numcols: usize = base_matrix.cols();
    
    /* -------- EXPLORE -------- */
    // Create a matrix of the same size as the base matrix with u32 values for the priority computation
    let mut explored_matrix: TriMat<u32> = TriMat::new((numrows, numcols));
    explored_matrix.reserve_exact(nonzeros);
    base_matrix.iter().for_each(|(_, (row, col))| {
        explored_matrix.add_triplet(row, col, std::u32::MAX);
    });

    // explored_matrix.set_triplet(triplet_index, row, col, val)
    // explored_matrix.find_locations(row, col)
    // dbg!(explored_matrix.find_locations(3, 3));  //-- [src\main.rs:49] explored_matrix.find_locations(3, 3) = []
    // dbg!(explored_matrix.find_locations(1, 1));  //-- [src\main.rs:49] explored_matrix.find_locations(3, 3) = []
    // dbg!(explored_matrix.find_locations(1, 1)[0].0);

    // explored_matrix.set_triplet(explored_matrix.find_locations(1, 1)[0], 1, 1, 45);

    let mut flag_invalidate = false;
    base_matrix.iter().enumerate().for_each(|(it, (_, (row, col)))| {

        // If invalidation has occurred, only check data with MAX value 
        if flag_invalidate && explored_matrix.data()[it] != std::u32::MAX {
            return;
        }

        // TODO remove this patterns.clone()     vvvvvvv
        let mut found: bool;
        'outer: for (prio, (n, i, j)) in patterns.iter().enumerate() {
            found = true;
            for ii in 0..*n {
                let location = explored_matrix.find_locations(((row as i32) + (ii* (*i))) as usize % numrows, (col as i32 + (ii* (*j))) as usize % numcols);

                if location.is_empty() {
                    found = false;
                    break;
                }

                if location[0].0 < prio {
                    break 'outer;
                }
            }
            if found {
                for ii in 0..*n {
                    // We have the certainty that each location is nonzero
                    let location = explored_matrix.find_locations(((row as i32) + (ii* (*i))) as usize % numrows, (col as i32 + (ii* (*j))) as usize % numcols);

                    explored_matrix.set_triplet(location[0], ((row as i32) + (ii* (*i))) as usize % numrows, (col as i32 + (ii* (*j))) as usize % numcols, prio as u32);
                }
                break 'outer;
            }
        }
    });

    base_matrix.iter().enumerate().for_each(|(it, (_, (row, col)))| {
        println!("Iteration {:?} = {:?}. Value is {}", it, (row,col), explored_matrix.data()[it]);
    });
}

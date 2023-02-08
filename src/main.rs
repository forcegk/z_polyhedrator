extern crate sprs;
extern crate text_io;

use std::{env, io::BufRead};
use sprs::{TriMat, CsMat};
use text_io::scan;

type Piece = (i32, i32, i32);

#[derive(Clone, Copy, Debug)]
struct Prio {
    prio: usize,
    src_row: i32,
    src_col: i32
}
const NO_PRIO: Prio = Prio {
    prio: std::usize::MAX,
    src_row: -1,
    src_col: -1,
};

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
    let mut explored_matrix: TriMat<Prio> = TriMat::new((numrows, numcols));
    explored_matrix.reserve_exact(nonzeros);
    base_matrix.iter().for_each(|(_, (row, col))| {
        explored_matrix.add_triplet(row, col, NO_PRIO);
    });

    // explored_matrix.set_triplet(triplet_index, row, col, val)
    // explored_matrix.find_locations(row, col)
    // dbg!(explored_matrix.find_locations(3, 3));  //-- [src\main.rs:49] explored_matrix.find_locations(3, 3) = []
    // dbg!(explored_matrix.find_locations(1, 1));  //-- [src\main.rs:49] explored_matrix.find_locations(3, 3) = []
    // dbg!(explored_matrix.find_locations(1, 1)[0].0);

    // explored_matrix.set_triplet(explored_matrix.find_locations(1, 1)[0], 1, 1, 45);

    let mut flag_invalidate = true;
    let mut max_prio_on_prev_it: usize = 0;
    let mut temp_max_prio_on_prev_it: usize = max_prio_on_prev_it;

    while flag_invalidate {
        flag_invalidate = false;
        base_matrix.iter().enumerate().for_each(|(it, (_, (row, col)))| {

            // When invalidation has occurred, only check data that has not been assigned to a pattern
            if explored_matrix.data()[it].prio <= max_prio_on_prev_it {
                // println!("Second try abort on [{},{}]!. max_prio was {:?} and cell had {:?}", row, col, max_prio_on_prev_it, explored_matrix.data()[it].prio);
                return;
            } else {
                // println!("Going through on [{},{}]!. max_prio was {:?} and cell had {:?}", row, col, max_prio_on_prev_it, explored_matrix.data()[it].prio)
            }

            // TODO -- Maybe in the future change this enumerate for a proper heuristic of priority
            // TODO -- improve search by discarding patterns under max_prio_on_prev_it
            'outer: for (priority, (n, i, j)) in patterns.iter().enumerate() {
                for ii in 0..*n {
                    let curr_row = ((row as i32) + (ii* (*i))) as usize;
                    let curr_col = ((col as i32) + (ii* (*j))) as usize;

                    let location = explored_matrix.find_locations(
                        curr_row + curr_col/numcols,
                        curr_col % numcols
                    );

                    // If there is no piece simply go to the next pattern
                    if location.is_empty() {
                        // println!("Continuing cuz empty {:?}", (n,i,j));
                        continue 'outer;
                    }

                    // This cell's value has a upper-most priority than the current exploration level
                    if explored_matrix.data()[location[0].0].prio <= priority {
                        // no need to set found to false
                        // println!("Breaking cuz breaks pattern {:?}", (n,i,j));
                        continue 'outer;
                    }
                }

                println!("Found match with pattern {:?} on [{},{}]!", (n,i,j), row, col);

                // The flow does not reach this code unless there is a piece match
                temp_max_prio_on_prev_it = std::cmp::max(temp_max_prio_on_prev_it, priority);

                // Perform invalidation of affected patterns
                for ii in 0..*n {
                    let curr_row = ((row as i32) + (ii* (*i))) as usize;
                    let curr_col = ((col as i32) + (ii* (*j))) as usize;

                    let abs_row = curr_row + curr_col/numcols;
                    let abs_col = curr_col % numcols;

                    let location = explored_matrix.find_locations(
                        abs_row,
                        abs_col
                    );

                    // If cell is currenly inside a pattern, invalidate that pattern
                    let curr_cell_data = explored_matrix.data()[location[0].0];
                    if curr_cell_data.prio != NO_PRIO.prio {
                        // Invalidation has been performed
                        flag_invalidate = true;

                        let (cc_n, cc_i, cc_j) = patterns.get(curr_cell_data.prio).unwrap();
                        println!("Performing invalidation of pattern {:?} starting at {:?}", (cc_n, cc_i, cc_j), (curr_cell_data.src_row, curr_cell_data.src_col));
                        for jj in 0..*cc_n {
                            let cc_curr_row = ((curr_cell_data.src_row as i32) + (jj* (*cc_i))) as usize;
                            let cc_curr_col = ((curr_cell_data.src_col as i32) + (jj* (*cc_j))) as usize;

                            let cc_abs_row = cc_curr_row + cc_curr_col/numcols;
                            let cc_abs_col = cc_curr_col % numcols;

                            let cc_location = explored_matrix.find_locations(
                                cc_abs_row,
                                cc_abs_col
                            );

                            explored_matrix.set_triplet(cc_location[0], 
                                cc_abs_row,
                                cc_abs_col,
                                NO_PRIO
                            );
                        }
                    }
                }

                // Tag pattern on the explored matrix
                for ii in 0..*n {
                    let curr_row = ((row as i32) + (ii* (*i))) as usize;
                    let curr_col = ((col as i32) + (ii* (*j))) as usize;

                    let abs_row = curr_row + curr_col/numcols;
                    let abs_col = curr_col % numcols;
                    
                    // We have the certainty that each location is nonzero
                    let location = explored_matrix.find_locations(
                        abs_row,
                        abs_col
                    );

                    // println!("Row: {:?} Col: {:?} || Base_Row: {:?} Base_Col: {:?} || Prio: {:?}", abs_row, abs_col, row, col, priority);

                    explored_matrix.set_triplet(location[0], 
                        abs_row,
                        abs_col,
                        Prio { prio: priority, src_row: row as i32, src_col: col as i32}
                    );
                }
                break 'outer;
            }
        });

        println!("\n------ NEW ITERATION ------\n");

        max_prio_on_prev_it = temp_max_prio_on_prev_it;
    }

    // DEBUG
    base_matrix.iter().enumerate().for_each(|(it, (_, (row, col)))| {
        println!("Iteration {:?} = {:?}. Value is [{:?}] [pattern = {:?}]", it, (row,col), explored_matrix.data()[it], {
            if explored_matrix.data()[it].prio < patterns.len() {
                let (i,j,k) = patterns.get(explored_matrix.data()[it].prio).unwrap();
                (*i, *j, *k)
            } else {
                (-1, -1, -1)
            }
        });
    });

    // Traverse matrix reading patterns (on nonzero hit add to pattern table and invalidate pattern until empty)
    // TODO
}

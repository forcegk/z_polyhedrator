use sprs::{CsMat,TriMat};
use std::{io::BufRead};
use text_io::scan;
use bitflags::bitflags;

/* DATATYPES */
type Pattern = (i32, i32, i32);
type Piece = (usize, usize, Pattern);

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
const ALREADY_DUMPED: i32 = -5;
const DUMPED_PRIO: Prio = Prio {
    src_row: ALREADY_DUMPED,
    src_col: ALREADY_DUMPED,
    .. NO_PRIO
};

/* FUNCTIONS */
pub struct SpISearxMatrix {
    value_matrix: CsMat<f64>,
    exploration_matrix: TriMat<Prio>,
    nonzeros: usize,
    numrows: usize,
    numcols: usize,
    patterns: Vec<Pattern>,
}

bitflags! {
    pub struct SpISearxPatternsFlags: u64 {
        const NoFlags               = 0b0000_0000;
        const SkipOnInvalidation    = 0b0000_0001;
        const SkipOnPatternSearch   = 0b0000_0010;
        const PrintInformation      = 0b0000_0100;
    }
}

impl SpISearxMatrix {
    pub fn from_file(path: &str) -> SpISearxMatrix {
        let value_matrix = sprs::io::read_matrix_market(path).unwrap().to_csr();
        let (numrows, numcols) = (value_matrix.rows(), value_matrix.cols());
        let nonzeros = value_matrix.nnz();
        let mut exploration_matrix = TriMat::new((numrows, numcols));
        exploration_matrix.reserve_exact(nonzeros);
        value_matrix.iter().for_each(|(_, (row, col))| {
            exploration_matrix.add_triplet(row, col, NO_PRIO);
        });

        return SpISearxMatrix {
            value_matrix: value_matrix,
            exploration_matrix: exploration_matrix,
            nonzeros: nonzeros,
            numrows: numrows,
            numcols: numcols,
            patterns: vec![], 
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

    pub fn search_patterns(&mut self, flags: SpISearxPatternsFlags) {

        let skip_on_invalidation = flags.contains(SpISearxPatternsFlags::SkipOnInvalidation);
        let skip_on_pattern_search = flags.contains(SpISearxPatternsFlags::SkipOnPatternSearch);
        let print_information = flags.contains(SpISearxPatternsFlags::PrintInformation);
        
        let mut flag_invalidate = true;
        let mut max_prio_on_prev_it: usize = 0;
        let mut temp_max_prio_on_prev_it: usize = max_prio_on_prev_it;

        while flag_invalidate {

            if print_information {
                println!("\n------ ITERATION WITH max_prio = {} ------\n", max_prio_on_prev_it);
            }

            flag_invalidate = false;
            self.value_matrix.iter().enumerate().for_each(|(_it, (_, (row, col)))| {

                if skip_on_invalidation && (self.exploration_matrix.data()[_it].prio <= max_prio_on_prev_it) {
                    // When invalidation has occurred, only check data that has not been assigned to a pattern
                    // This leads to improved performance while producing slightly different (although correct) results 
                    return;
                }

                // TODO -- Maybe in the future change this enumerate for a proper heuristic of priority
                // TODO -- improve search by discarding patterns under max_prio_on_prev_it
                'outer: for (priority, (n, i, j)) in self.patterns.iter().enumerate() {

                    if skip_on_pattern_search && (priority < max_prio_on_prev_it) {
                        // This leads to improved performance while producing slightly different (although correct) results 
                        continue 'outer;
                    }

                    for ii in 0..*n {
                        let curr_row = ((row as i32) + (ii* (*i))) as usize;
                        let curr_col = ((col as i32) + (ii* (*j))) as usize;

                        let location = self.exploration_matrix.find_locations(
                            curr_row + curr_col/self.numcols,
                            curr_col % self.numcols
                        );

                        // If there is no piece simply go to the next pattern
                        if location.is_empty() {
                            // println!("Continuing cuz empty {:?}", (n,i,j));
                            continue 'outer;
                        }

                        // This cell's value has a upper-most priority than the current exploration level
                        if self.exploration_matrix.data()[location[0].0].prio <= priority {
                            // no need to set found to false
                            // println!("Breaking cuz breaks pattern {:?}", (n,i,j));
                            continue 'outer;
                        }
                    }

                    if print_information {
                        println!("Found match with pattern {:?} on [{},{}]!", (n,i,j), row, col);
                    }

                    // The flow does not reach this code unless there is a piece match
                    temp_max_prio_on_prev_it = std::cmp::max(temp_max_prio_on_prev_it, priority);

                    // Perform invalidation of affected patterns
                    for ii in 0..*n {
                        let curr_row = ((row as i32) + (ii* (*i))) as usize;
                        let curr_col = ((col as i32) + (ii* (*j))) as usize;

                        let abs_row = curr_row + curr_col/self.numcols;
                        let abs_col = curr_col % self.numcols;

                        let location = self.exploration_matrix.find_locations(
                            abs_row,
                            abs_col
                        );

                        // If cell is currenly inside a pattern, invalidate that pattern
                        let curr_cell_data = self.exploration_matrix.data()[location[0].0];
                        if curr_cell_data.prio != NO_PRIO.prio {
                            // Invalidation has been performed
                            flag_invalidate = true;

                            let &(cc_n, cc_i, cc_j) = self.patterns.get(curr_cell_data.prio).unwrap();
                            
                            if print_information {
                                println!("Performing invalidation of pattern {:?} starting at {:?}", (cc_n, cc_i, cc_j), (curr_cell_data.src_row, curr_cell_data.src_col));
                            }

                            for jj in 0..cc_n {
                                let cc_curr_row = ((curr_cell_data.src_row as i32) + (jj* (cc_i))) as usize;
                                let cc_curr_col = ((curr_cell_data.src_col as i32) + (jj* (cc_j))) as usize;

                                let cc_abs_row = cc_curr_row + cc_curr_col/self.numcols;
                                let cc_abs_col = cc_curr_col % self.numcols;

                                let cc_location = self.exploration_matrix.find_locations(
                                    cc_abs_row,
                                    cc_abs_col
                                );

                                self.exploration_matrix.set_triplet(cc_location[0], 
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

                        let abs_row = curr_row + curr_col/self.numcols;
                        let abs_col = curr_col % self.numcols;
                        
                        // We have the certainty that each location is nonzero
                        let location = self.exploration_matrix.find_locations(
                            abs_row,
                            abs_col
                        );

                        // println!("Row: {:?} Col: {:?} || Base_Row: {:?} Base_Col: {:?} || Prio: {:?}", abs_row, abs_col, row, col, priority);

                        self.exploration_matrix.set_triplet(location[0], 
                            abs_row,
                            abs_col,
                            Prio { prio: priority, src_row: row as i32, src_col: col as i32}
                        );
                    }
                    break 'outer;
                }
            });

            max_prio_on_prev_it = temp_max_prio_on_prev_it;
        }
    }


    pub fn get_piece_list(&mut self) -> Vec<Piece> {
        let mut piece_list: Vec<Piece> = vec![];

        // Copy this to allow multiple get_piece // print_piece list calls
        let mut local_exploration_matrix = TriMat::new((self.numrows, self.numcols));
        local_exploration_matrix.reserve_exact(self.nonzeros);

        // This into_iter() should consume the resource but it does not ????
        // whatever let's use it for now...
        self.exploration_matrix.into_iter().for_each(|(data, (row, col))| {
            local_exploration_matrix.add_triplet(row, col, data)
        });

        self.value_matrix.iter().enumerate().for_each(|(it, (_, (row, col)))| {
    
            let curr_cell_data = local_exploration_matrix.data()[it];
    
            // It it has already been dumped
            if curr_cell_data.src_col == ALREADY_DUMPED || curr_cell_data.src_row == ALREADY_DUMPED {
                return;
            }
            
            // If does not belong to any piece we have to treat it differently
            if curr_cell_data.prio == NO_PRIO.prio {
                // println!("{}\t{}\t{}\t{}\t{}", row, col, 1, 0, 0);
                piece_list.push((row, col, (1, 0, 0)));
                return;
            } else {
                let &(n,i,j) = self.patterns.get(curr_cell_data.prio).unwrap(); 
                // println!("{}\t{}\t{}\t{}\t{}", row, col, n, i, j);
                piece_list.push((row, col, (n, i, j)));
            }
    
            // Mark rest of the cells in the pattern as dumped
            let &(cc_n, cc_i, cc_j) = self.patterns.get(curr_cell_data.prio).unwrap();
            for jj in 0..cc_n {
                let cc_curr_row = ((curr_cell_data.src_row as i32) + (jj* (cc_i))) as usize;
                let cc_curr_col = ((curr_cell_data.src_col as i32) + (jj* (cc_j))) as usize;
    
                let cc_abs_row = cc_curr_row + cc_curr_col/self.numcols;
                let cc_abs_col = cc_curr_col % self.numcols;
    
                let cc_location = local_exploration_matrix.find_locations(
                    cc_abs_row,
                    cc_abs_col
                );
    
                local_exploration_matrix.set_triplet(cc_location[0], 
                    cc_abs_row,
                    cc_abs_col,
                    &DUMPED_PRIO
                );
            }
        });

        return piece_list;
    }

    pub fn print_pieces(&mut self) {
        println!("Row\tCol\tN\tI\tJ");

        let piece_list: Vec<Piece> = self.get_piece_list();

        piece_list.iter().for_each(|(row, col, (n, i, j))| {
            println!("{}\t{}\t{}\t{}\t{}", row, col, n, i, j);
        });      
    }
}


// Leftover things
// // DEBUG
// base_matrix.iter().enumerate().for_each(|(it, (_, (row, col)))| {
//     println!("Iteration {:?} = {:?}. Value is [{:?}] [pattern = {:?}]", it, (row,col), explored_matrix.data()[it], {
//         if explored_matrix.data()[it].prio < patterns.len() {
//             let (i,j,k) = *patterns.get(explored_matrix.data()[it].prio).unwrap();
//             (i, j, k)
//         } else {
//             (-1, -1, -1)
//         }
//     });
// });
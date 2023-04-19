use std::cmp::max_by_key;
use std::collections::HashSet;
use std::time::Instant;

use colored::Colorize;
use itertools::Itertools;
use linked_hash_map::LinkedHashMap;
use sprs::CsMat;

use crate::utils::{Pattern,Piece,Uwc,OriginUwc};
use crate::utils::orig_uwc_to_piece_1d;

//                    N    I    J    Order  Sub-Pattern
type MetaPattern = ((i32, i32, i32),  i32,  Option<i32> );
// If Option is None -> N,I,J describe the base pattern

//                         X     Y
type MetaPatternPiece = (usize,usize);

pub struct SpAugment {
    origin_uwc_list: Vec<(OriginUwc, i32)>,
    pub nrows: usize,
    pub ncols: usize,
    pub nnz: usize,
    meta_patterns: LinkedHashMap<i32, MetaPattern>,
    meta_pattern_pieces: LinkedHashMap<MetaPatternPiece, i32>
}

impl SpAugment {
    pub fn from_1d_uwc_list(origin_uwc_list: Vec<(OriginUwc, i32)>, nrows: usize, ncols: usize, nnz: usize) -> Self {

        println!("Printing orig_uwc_list from spaugment!");
        origin_uwc_list.iter().for_each(|((x,y,(u,w,c)), id)| {
            print!("({:?},{:?})\t   =\t{:?}\t{:?}\t{:?}\t{:?}\n", x, y, id, u, w, c);
        });

        let meta_pattern_pieces = origin_uwc_list
            .iter()
            .map(|(ouwc, id)| {
                let (x,y,_) = orig_uwc_to_piece_1d(ouwc);
                ((x,y), *id)
            })
            .collect::<LinkedHashMap<MetaPatternPiece, i32>>();

        println!("MP_Pieces:\n{:?}", meta_pattern_pieces);

        // NOTE HERE! The key -1 does not have to be in this list! It is only included on the distinct patterns list from spfgen temporarily, but it is NOT necessary.
        let mut meta_patterns: LinkedHashMap<i32, MetaPattern> = LinkedHashMap::new();
        for ((_,_,(n,i,j)), id) in origin_uwc_list.iter().map(|(ouwc, id)| (orig_uwc_to_piece_1d(ouwc), id)) {
            if meta_patterns.get(id).is_some() { continue; }
            else {
                meta_patterns.insert(*id, ((n,i,j), 1, None));
            }
        };

        println!("MP_Dict:\n{:?}\nLen: {}", meta_patterns, meta_patterns.len());

        SpAugment { 
            origin_uwc_list: origin_uwc_list,
            nrows: nrows, 
            ncols: ncols,
            nnz: nnz,
            meta_patterns: meta_patterns,
            meta_pattern_pieces: meta_pattern_pieces
        }
    }

    pub fn augment_dimensionality(&mut self, target_dim: i32) {
        let single_compensation: i32 = match self.meta_patterns.get(&-1) {
            Some(_) => -1i32,
            None => 0i32
        };

        println!("\n------- AUGMENT DIMENSIONALITY -------\n");

        println!("Compensation {} (this accounts for elements with id=-1)", single_compensation);

        let mut start_ptr: usize = 0;
        let mut end_ptr: usize = 0;

        // PRECONDITION: Metapatterns are not necessarily ordered, but same metapatterns are consecutive in the list

        // FIXME Maybe generalize this to support any-dimensional input
        for curr_dim in 2..=target_dim {
            // Set ptr to current dimensionality
            start_ptr = end_ptr;
            println!("Searching for {}D", curr_dim);

            // Aux variables for processing
            let mut origins_list: Vec<(i32, i32)> = vec![];
            let mut curr_id: i32 = 0; // Zero here to save an iteration (only if meta_pattern_pieces is sorted). Not really needed apart from that
            
            // Advance pointer to current dimensionality pieces
            let mut metapat_pieces_iter = self.meta_pattern_pieces.iter()
                .filter(|(_,id)| **id != -1)
                .skip(start_ptr);

            let mut new_metapat_pieces: LinkedHashMap<MetaPatternPiece, i32> = LinkedHashMap::new();
            let mut new_metapats: LinkedHashMap<i32, MetaPattern> = LinkedHashMap::new();

            // This loop loads all consecutive metapatterns with the same id into origins_list, the executes the "if opt.is_none() [...]"
            loop {
                let opt = metapat_pieces_iter.next();
                
                if opt.is_none() || matches!(opt, Some((_,id)) if curr_id != *id) {
                
                    println!("\n------- compute_metapatterns for id = {} -------", curr_id);

                    // Compute metapatterns FIXME parametrize max and min strides
                    compute_metapatterns(&mut origins_list);

                    // And prepare for next batch
                    origins_list.clear();

                    // Breaking if end was reached
                    if opt.is_none() { break; }

                    // Else update curr_id
                    let (_,id) = opt.unwrap();
                    curr_id = *id;
                }

                // Append consecutive same-id metapattern pieces to origins_list
                let ((x,y),_) = opt.unwrap();
                origins_list.push((*x as i32, *y as i32));

                start_ptr += 1;
            }

            println!("Startptr: {:?}. Curr_id: {:?}", start_ptr, curr_id);

            // TODO Add new_metapat_pieces and new_metapats to current ones
            
        }
    }
}

#[inline(always)]
#[allow(dead_code)]
fn compute_metapatterns(origins_list: &mut Vec<(i32, i32)>) -> Option<(LinkedHashMap<i32, MetaPattern>, LinkedHashMap<MetaPatternPiece, i32>)> {

    println!("Metapatterns: {:?}", origins_list);

    // No feasible higher order metapatterns
    if origins_list.len() <= 1 {
        return None;
    }

    let mut MetaPatternList: LinkedHashMap<i32, MetaPattern> = LinkedHashMap::new();
    let mut MetaPatternPieceList: LinkedHashMap<MetaPatternPiece, i32> = LinkedHashMap::new();

    let fn_tuple_sub = |(x1,y1):(i32,i32),(x2,y2):(i32,i32)| (x1-x2, y1-y2);

    let (_,max_col) = *origins_list.iter().max_by_key(|(_,col)| *col).unwrap();
    let (max_row,_) = *origins_list.iter().max_by_key(|(row,_)| *row).unwrap();

    println!("Max col = {}, Max row = {}", max_col, max_row);

    // Get all strides between pieces
    let strides = origins_list
        .iter()
        .tuple_combinations()
        .map(|(a,b)| fn_tuple_sub (*b, *a))
        .collect::<Vec<(i32,i32)>>();

    println!("STRIDES: {:?}", strides);

    let mut occurrences = strides
        .iter()
        .into_group_map_by(|x| **x)
        .into_iter()
        .map(|(k,v)| (k, v.len() as u32))
        .sorted_by_key(|(_,reps)| std::cmp::Reverse(*reps))
        .collect::<LinkedHashMap<(i32,i32),u32>>();

    println!("OCCURRENCES: {:?}", occurrences);

    // let mut basepat: usize = 0;
    // loop {
    //     if origins_list.get(basepat).is_none() { break; }

    //     // // Normalize distances to first pattern
    //     // let base = *origins_list.get(basepat).unwrap();
    //     // for origin in origins_list.iter_mut() {
    //     //     *origin = fn_tuple_sub (*origin, base);
    //     // }
    //     // println!("Normalized for #{}: {:?}", basepat, origins_list);

    //     basepat += 1;
    // }

    // let mut value_matrix = CsMat::empty(f64_value_matrix.storage(), f64_value_matrix.inner_dims());

    // Compose sparse matrix with origins_list
    let mut expl_matrix: CsMat<bool> = CsMat::zero((max_row as usize, max_col as usize));
    for (row,col) in origins_list {
        expl_matrix.insert(*row as usize, *col as usize, false);
    }

    println!("Mat = {:?}", expl_matrix);

    let fn_best_piece = |p1:Piece,p2:Piece| std::cmp::max_by_key(p1, p2, |(_,_,(n,_,_))| *n);

    'outer: loop {
        // println!("(row = {}, col = {}, reps = {})", *row, *col, *reps);
        
        // Most repeated pattern first (MRPF)
        let mut occ_it_peekable = occurrences.iter_mut().peekable();
        'inner: for ((offset_x,offset_y), reps) in occ_it_peekable {
            let mut found: bool = false;
            let mut best_piece: Piece = (0,0,(0,0,0));

            // Continue to keep searching. Break to accept best_piece.
            'innermost: for (value, (expl_row,expl_col)) in expl_matrix.iter() {
                
                    let result: Option<Piece> = check_metapattern_reps(&expl_matrix, (expl_row, expl_col), &(*reps as i32, *offset_x, *offset_y));
                    match result {
                        None => continue,
                        Some((x,y,(n,i,j))) => {
                            // Update best piece if needed
                            best_piece = fn_best_piece(best_piece, (expl_row, expl_col, (n, *offset_x, *offset_y)));
                            
                            // Check if we need to keep searching
                            if n < *reps as i32 {
                                match occ_it_peekable.peek() {
                                    Some(((next_offset_x,next_offset_y), next_reps)) => {
                                        if n < **next_reps as i32 {
                                            // continue searching
                                            continue 'innermost;
                                        }
                                    },
                                    None => break 'innermost, // this is the last one so commit with whatever we have
                                }
                            }



                            
                        },
                    }
            }

        }

        // reorder occurrences



            // let result: Option<Piece> = check_metapattern_reps(&expl_matrix, (*row as usize, *col as usize), pattern);
            
            // loop {
                //     max_n = new_max_n;
                //     new_max_n = check_metapattern_reps(&expl_matrix, (*row as usize,*col as usize), &(max_n as i32,*row,*col)) as i64;
                // }

        break;
    }
    // Begin strided search



    None // TODO FIX




}

#[inline(always)]
#[allow(dead_code)]
fn check_metapattern_reps(csmat: &CsMat<bool>, curr_pos: (usize, usize), pattern: &Pattern) -> Option<Piece> {
    let &(max_n,i,j) = pattern;
    let (x,y) = curr_pos;

    // Discard already dumped patterns without computing bounds first
    if *csmat.get(x,y).unwrap() {
        return None;
    }

    // We can start on the next pattern (length would be 2 if only the first iteration completes)
    for ii in 1..max_n {
        let position = csmat.get((x as i64 + (i as i64 * ii as i64)) as usize, (y as i64 + (j as i64 * ii as i64)) as usize);
        match position {
            Some(&is_in_pat) => {
                if is_in_pat {
                    return Some((x,y,(ii,i,j)));
                } else {
                    continue;
                }
            },
            None => return Some((x,y,(ii,i,j))),
        }
    }

    return Some((x,y,(max_n,i,j)));
}
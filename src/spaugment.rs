use std::collections::HashSet;
use std::time::Instant;

use itertools::Itertools;
use linked_hash_map::LinkedHashMap;

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

        println!("Compensation {}", single_compensation);

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
            let mut curr_id: i32 = 0; // Zero here to save an iteration. Not really needed
            
            // Advance pointer to current dimensionality pieces
            let mut metapat_pieces_iter = self.meta_pattern_pieces.iter()
                .filter(|(_,id)| **id != -1)
                .skip(start_ptr);
            // TODO a filter could be added here to skip single nonzeros or some other piece types

            let mut new_metapat_pieces: LinkedHashMap<MetaPatternPiece, i32> = LinkedHashMap::new();
            let mut new_metapats: LinkedHashMap<i32, MetaPattern> = LinkedHashMap::new();

            loop {
                let opt = metapat_pieces_iter.next();
                
                if opt.is_none() || matches!(opt, Some((_,id)) if curr_id != *id) {
                    // Compute metapatterns FIXME parametrize max and min strides
                    compute_metapatterns(&mut origins_list, 100, 1);

                    // And prepare for next batch
                    origins_list.clear();

                    // Breaking if end was reached
                    if opt.is_none() { break; }

                    // Else update curr_id
                    let (_,id) = opt.unwrap();
                    curr_id = *id;
                }

                let ((x,y),_) = opt.unwrap();
                origins_list.push((*x as i32, *y as i32));

                start_ptr += 1;
            }

            println!("Startptr: {:?}. Curr_id: {:?}", start_ptr, curr_id);

            // Add new_metapat_pieces and new_metapats to current ones
            
        }
    }
}

#[inline(always)]
#[allow(dead_code)]
fn compute_metapatterns(origins_list: &mut Vec<(i32, i32)>, max_stride: usize, min_stride: usize) -> Option<(LinkedHashMap<i32, MetaPattern>, LinkedHashMap<MetaPatternPiece, i32>)> {

    println!("Metapatterns: {:?}", origins_list);

    // No feasible higher order metapatterns
    if origins_list.len() <= 1 {
        return None;
    }

    let fn_tuple_sub = |(x1,y1):(i32,i32),(x2,y2):(i32,i32)| (x1-x2, y1-y2);
    let mut basepat: usize = 0;

    let start = Instant::now();

    // Get all strides between pieces
    let strides = origins_list
        .iter()
        .tuple_combinations()
        .map(|(a,b)| fn_tuple_sub (*b, *a))
        .collect::<Vec<(i32,i32)>>();

    let duration_strides = start.elapsed();
    let start = Instant::now();

    println!("STRIDES: {:?}", strides);

    let occurrences = strides
        .iter()
        .into_group_map_by(|x| **x)
        .into_iter()
        .map(|(k,v)| (k, v.len() as u32))
        .sorted_by_key(|(_,reps)| std::cmp::Reverse(*reps))
        .collect::<LinkedHashMap<(i32,i32),u32>>();

    println!("OCCURRENCES: {:?}", occurrences);

    let duration = start.elapsed();

    println!("Elapsed: strides={:?}, group={:?}", duration_strides, duration);

    loop {
        if origins_list.get(basepat).is_none() { break; }

        // Normalize distances to first pattern
        let base = *origins_list.get(basepat).unwrap();
        for origin in origins_list.iter_mut() {
            *origin = fn_tuple_sub (*origin, base);
        }
        
        println!("Normalized for #{}: {:?}", basepat, origins_list);

        basepat += 1;
    }

    // Begin strided search



    None // TODO FIX




}
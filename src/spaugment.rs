use linked_hash_map::LinkedHashMap;

use crate::utils::{Pattern,Piece,Uwc,OriginUwc};

pub struct SpAugment {
    origin_uwc_list: Vec<(OriginUwc, i32)>,
    pub nrows: usize,
    pub ncols: usize,
    pub nnz: usize,
}

impl SpAugment {
    pub fn from_uwc_list(origin_uwc_list: Vec<(OriginUwc, i32)>, nrows: usize, ncols: usize, nnz: usize) -> Self {

        eprintln!("Printing orig_uwc_list from spaugment!");
        origin_uwc_list.iter().for_each(|((x,y,(u,w,c)), id)| {
            print!("({:?},{:?})\t   =\t{:?}\t{:?}\t{:?}\t{:?}]\n", x, y, id, u, w, c);
        });

        SpAugment { origin_uwc_list: origin_uwc_list, nrows: nrows, ncols: ncols, nnz: nnz }
    }

    pub fn augment_dimensionality(&mut self) {
    }

    // pub fn load_meta_patterns(&mut self, meta_pattern_list: Vec<MetaPattern>) {}


}
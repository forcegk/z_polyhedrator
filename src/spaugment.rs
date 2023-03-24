use linked_hash_map::LinkedHashMap;
use crate::utils;

type MetaPattern = (i32,i32);       // Meta-Pattern just describes directionality, and not repeating times.
type Uwc = (Vec<Vec<i32>>, Vec<i32>, Vec<i32>);

pub struct SpAugment {
    uwc_list: Vec<(usize, Uwc)>,
    pub nrows: usize,
    pub ncols: usize,
    pub nnz: usize,
    pub inc_nnz: usize,
    meta_patterns: Vec<(usize,MetaPattern)> // The Meta-Pattern list is only a priority list for meta-pattern search. 
}

impl SpAugment {
    pub fn from_uwc_list(uwc_list: Vec<Uwc>, nrows: usize, ncols: usize, nnz: usize) -> Self {
        
    }
}
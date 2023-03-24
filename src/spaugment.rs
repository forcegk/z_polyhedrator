use linked_hash_map::LinkedHashMap;
use crate::utils;

type Uwc = (Vec<Vec<i32>>, Vec<i32>, Vec<i32>);

pub struct SpAugment {
    uwc_list: Vec<(usize, Uwc)>,
}
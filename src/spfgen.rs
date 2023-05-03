use std::{fs::{File, self}, path::PathBuf, io::{SeekFrom, Seek}};

use byteorder::{WriteBytesExt, LittleEndian};
use linked_hash_map::LinkedHashMap;
use linked_hash_set::LinkedHashSet;
use sprs::{CsMat, TriMat};

use crate::utils::{Pattern,Piece,Uwc,OriginUwc, MetaPattern, MetaPatternPiece};
use crate::utils::{pattern_to_uwc,convex_hull_1d};

pub struct SPFGen {
    ast_list: Vec<Piece>,
    uwc_list: Vec<(Uwc, i32)>,
    pub nrows: usize,
    pub ncols: usize,
    pub nnz: usize,
    pub inc_nnz: usize,
    distinct_patterns: LinkedHashMap<Pattern, i32>,
    distinct_uwc: LinkedHashMap<Uwc, i32>
}

impl SPFGen {
    pub fn from_piece_list(ast_list: Vec<Piece>, nrows: usize, ncols: usize, nnz: usize) -> Self {
        let ninc_nnz = ast_list.iter().filter(|(_,_,(n,_,_))| *n == 1).count();
        let inc_nnz = nnz - ninc_nnz;

        // Create distinct pattern LinkedHashMap indexed 0..used_patterns with the help of an intermediate HashSet
        let mut distinct_patterns: LinkedHashMap<Pattern, i32> = ast_list
            .iter()
            .map(|(_,_,pattern)| *pattern)
            .filter(|(i,_,_)| *i > 1)
            .collect::<LinkedHashSet<Pattern>>()
            .into_iter()
            .enumerate()
            .map(|(idx, pattern)| (pattern, idx as i32))
            .collect();

        // Finally insert (1,0,0) pattern. (Has to be the last one):
        // This can not be removed, as we always take into account the single nonzero values,
        // even in the case of no single points remaining.
        distinct_patterns.insert((1,0,0), -1i32);

        let uwc_list: Vec<(Uwc, i32)> = ast_list
            .iter()
            .map(|(_,_,pattern)| (pattern_to_uwc(pattern), *distinct_patterns.get(pattern).unwrap()))
            .collect();

        let distinct_uwc: LinkedHashMap<Uwc, i32> = distinct_patterns
            .iter()
            .map(|(pattern, id)| (pattern_to_uwc(pattern), *id))
            .collect();

        // println!("nrows = {}, ncols = {}, nnz = {}, inc_nnz = {}", nrows, ncols, nnz, inc_nnz);
        // println!("\nLen={}. {:?}\n", distinct_patterns.len(), distinct_patterns);
        // println!("\nLen={}. {:?}\n", distinct_uwc.len(), distinct_uwc);

        return SPFGen {
            ast_list,
            uwc_list,
            nrows,
            ncols,
            nnz,
            inc_nnz,
            distinct_patterns,
            distinct_uwc
        };
    }

    // pub fn from_metapatterns_list(meta_patterns: LinkedHashMap<i32, MetaPattern>, meta_pattern_pieces: LinkedHashMap<MetaPatternPiece, i32>) -> Self {


    //     return SPFGen { ast_list: (), uwc_list: (), nrows: (), ncols: (), nnz: (), inc_nnz: (), distinct_patterns: (), distinct_uwc: () }
    // }

    #[allow(dead_code)]
    pub fn print_ast_list(&self) {
        println!("AST_List:\nRow\tCol\tN\tI\tJ");
        self.ast_list.iter().for_each(|(row, col, (n, i, j))| {
            println!("{}\t{}\t{}\t{}\t{}", row, col, n, i, j);
        });   
    }

    pub fn print_uwc_list(&self, show_eqs: bool) {
        println!("Uwc List:\nid\tU\t\tw\tc");
        self.uwc_list.iter().for_each(|((u,w,c), id)| {
            print!("{:?}\t{:?}\t{:?}\t{:?}{}", id, u, w, c, { if show_eqs { format_eqs(u, w) } else { "\n".to_string() } });
        });
    }

    pub fn print_distinct_uwc_list(&self, show_eqs: bool) {
        println!("Distinct Uwc List:\nid\tU\t\tw\tc");
        self.distinct_uwc.iter().for_each(|((u,w,c), id)| {
            print!("{:?}\t{:?}\t{:?}\t{:?}{}", id, u, w, c, { if show_eqs { format_eqs(u, w) } else { "\n".to_string() } });
        });
    }

    #[allow(dead_code)]
    pub fn get_uwc_list(&self) -> Vec<(Uwc, i32)> {
        self.uwc_list.clone()
    }

    pub fn get_orig_uwc_list(&self) -> Vec<(OriginUwc, i32)> {
        let mut retvec: Vec<(OriginUwc, i32)> = vec![];

        for idx in 0..self.uwc_list.len() {
            let (uwc, id) = self.uwc_list.get(idx).unwrap();
            let (x,y, _) = self.ast_list.get(idx).unwrap();
            retvec.push(((*x, *y, uwc.clone()), *id));
        }

        retvec
    }

    pub fn write_spf(&self, input_value_matrix: &str, output_file_path: &str) {
        // Read matrixmarket f64 value matrix
        let f64_value_matrix: CsMat<f64> = crate::utils::read_matrix_market_csr(input_value_matrix);

        // Quick sanity check
        if f64_value_matrix.nnz() != self.nnz {
            panic!("NNZ of value matrix and pattern list do not match. Maybe double check your params?");
        }

        let mut file = File::create(output_file_path).expect(format!("Unable to create file {}", output_file_path).as_str());

        let path = PathBuf::from(output_file_path);
        eprintln!("Writing to file {}", fs::canonicalize(&path).unwrap().display());

        // Write header
        file.write_i32::<LittleEndian>(self.nnz as i32).unwrap();
        file.write_i32::<LittleEndian>(self.inc_nnz as i32).unwrap();
        file.write_i32::<LittleEndian>(self.nrows as i32).unwrap();
        file.write_i32::<LittleEndian>(self.ncols as i32).unwrap();

        // Write dimensions
        file.write_i16::<LittleEndian>(2i16).unwrap();
        // number of base shapes is actual found shapes, not unfound ones. Also we have to take into account removing the single nonzeros
        file.write_i32::<LittleEndian>((self.distinct_patterns.len()-1) as i32).unwrap();
        // write zero hierarchical shapes
        file.write_i32::<LittleEndian>(0i32).unwrap();
        // write TEMPORARY ZERO as pointer to start of data. Will need to fseek to position 26 later
        //  (python code `f.seek ( 26 )` on write_spf func at around line 810)
        file.write_i32::<LittleEndian>(0i32).unwrap();

        // Get index of single nonzeros (not in a pattern to filter them out of the next foreach)
        // let ninc_nonzero_pattern_id = self.distinct_patterns.get(&(1,0,0)).unwrap();
        let ninc_nonzero_pattern_id = -1i32;
        // This has been brought from down below, as it is useful for the following computation
        // We also know that regular pieces are at the end of the list
        let piece_cutoff = self.uwc_list.iter().filter(|(_, id)| *id != ninc_nonzero_pattern_id).count();
        // println!("Piece cutoff = {}", piece_cutoff);

        let shape_dims_max: i16 = {
            if piece_cutoff == 0 { 0i16 }
            else { 1i16 }
        };

        // Write maximum dimensionality of iP for vertex_rec (FIXME this currently only supports 1d)
        file.write_i16::<LittleEndian>(shape_dims_max).unwrap();

        // FIXME Get shape_dims_max from previous it
        for _ in 0..shape_dims_max {
            file.write_i32::<LittleEndian>(0i32).unwrap();
        }

        // println!("Writing u={:?}, w={:?}, c={:?} with id={:?}", u, w, c, id);
        self.distinct_uwc.iter().filter(|(_,id)| **id != ninc_nonzero_pattern_id).for_each(|((u,w,c), id)| {
            // Write shape id
            file.write_i16::<LittleEndian>(*id as i16).unwrap();
            // Write type of encoding. 0 = vertex_rec, 1 = vertex_gen, 2 = ineqs . FIXME only writes vertex_rec
            file.write_i16::<LittleEndian>(0i16).unwrap();
            // Write dimension of i_p. Hardcodec for vertex_rec
            file.write_i16::<LittleEndian>(u[0].len() as i16).unwrap();
            // println!("    - Dimension of i_p = {}", u[0].len());
            
            // Get convex_hull (FIXME: Current dimensionality == 1 so dense ch == non-dense ch. Therefore:)
            let ch: Vec<i32> = convex_hull_1d(u, w, false);
            
            // Write minimal point
            // FIXME for higher dimensionality
            for _ in 0..1 {
                file.write_i32::<LittleEndian>(ch[0]).unwrap();
                // println!("    - Minimal point from ch[0] = {}", ch[0]);
            }

            // Write lenghts along axis            
            // FIXME for higher dimensionality
            for _ in 0..1 {
                // taking shortcut as all input are 1-d
                file.write_i32::<LittleEndian>(w[0]-w[1]).unwrap();
                // println!("    - Lenghts along axes from from w[0]-[w1] = {}  ==  {} = ch[-1]-ch[0]", w[0]-w[1], ch[ch.len()-1]-ch[0]);
            }

            // "Hardcoded stride at this time"
            for _ in 0..u[0].len() {
                file.write_i32::<LittleEndian>(1i32).unwrap();
            }

            // Write lattice
            for cc in c {
                file.write_i32::<LittleEndian>(*cc).unwrap();
            }
            // println!("c = {:?}", c);
        });

        // Write total number of origins
        file.write_i32::<LittleEndian>(piece_cutoff as i32).unwrap();

        // VERY IMPORTANT! Remember that uwc_list and ast_list have to be in the same order for this to be coherent
        let mut data_offset: i32 = 0;
        for idx in 0..piece_cutoff {
            let ((u,w,_), id) = &self.uwc_list[idx];

            // Write shape id
            file.write_i16::<LittleEndian>(*id as i16).unwrap();

            // Get convex_hull (FIXME: Current dimensionality == 1 so dense ch == non-dense ch. Therefore:)
            let ch: Vec<i32> = convex_hull_1d(u, w, true);

            // Write coordinates of AST's starting point
            file.write_i32::<LittleEndian>(self.ast_list[idx].0 as i32).unwrap(); // row
            file.write_i32::<LittleEndian>(self.ast_list[idx].1 as i32).unwrap(); // col
            file.write_i32::<LittleEndian>(data_offset).unwrap();                 // data offset
            data_offset += ch.len() as i32;   // Offset in elements. no judgment about data type
        }

        // Codes here:
        //  -> CSR = 0
        //  -> ??? = 1
        //  -> COO = 2
        let csr_size = self.nrows + 1 + (self.nnz - self.inc_nnz);
        let coo_size = 2 * (self.nnz - self.inc_nnz);

        let uninc_format: u8 = { if csr_size <= coo_size { 0u8 }
                                 else { 2u8 }};

        eprint!("Writing uninc_format = {} to offset 0x{:X}... ", uninc_format, file.seek(SeekFrom::Current(0)).unwrap());
        file.write_u8(uninc_format).unwrap();

        // TODO write CSR // COO dump codes
        match uninc_format {
            0 => {  eprintln!("Writing CSR");
                    // panic!("Not implemented!")
                    let mut local_csr_mat: TriMat<u8> = TriMat::new((self.nrows, self.ncols));
                    eprint!("Writing points: [");
                    for csr_idx in piece_cutoff..self.uwc_list.len() {
                        print!("({},{}) ", self.ast_list[csr_idx].0, self.ast_list[csr_idx].1);
                        local_csr_mat.add_triplet(self.ast_list[csr_idx].0, self.ast_list[csr_idx].1, 1u8);
                    }
                    let local_csr_mat: CsMat<u8> = local_csr_mat.to_csr();

                    eprintln!("\x08] with:\nind_ptr: {:?}", local_csr_mat.proper_indptr());
                    eprintln!("indices: {:?}", local_csr_mat.indices());

                    // Write rowptr/indptr
                    local_csr_mat.proper_indptr().iter().for_each(|iptr_val| {
                        file.write_i32::<LittleEndian>(*iptr_val as i32).unwrap();
                    });
                    // Write colptr/indices
                    local_csr_mat.indices().iter().for_each(|ind_val| {
                        file.write_i32::<LittleEndian>(*ind_val as i32).unwrap();
                    });
                 },
            2 => {  eprintln!("Writing COO");
                    eprint!("Writing Rowptr: ");
                    for coo_idx in piece_cutoff..self.uwc_list.len() {
                        print!("{} ", self.ast_list[coo_idx].0);
                        file.write_i32::<LittleEndian>(self.ast_list[coo_idx].0 as i32).unwrap(); // Write rowptr
                    }
                    eprint!("\nWriting Colptr: ");
                    for coo_idx in piece_cutoff..self.uwc_list.len() {
                        eprint!("{} ", self.ast_list[coo_idx].1);
                        file.write_i32::<LittleEndian>(self.ast_list[coo_idx].1 as i32).unwrap(); // Write colptr
                    }
                    eprintln!();
                 },
            _ => { panic!("The hell you did here man") }
        }

        // Save current position for later
        let curr_pos = file.seek(SeekFrom::Current(0)).unwrap();

        // And rewrite pointer to start of data
        file.seek(SeekFrom::Start(26)).unwrap();
        file.write_i32::<LittleEndian>(curr_pos as i32).unwrap();
        file.seek(SeekFrom::Start(curr_pos)).unwrap();

        // f.write( struct.pack( len(self.mask)*"d", *mat.data[self.reorder] ) )
        self.ast_list.iter().for_each(|(row,col,(n,i,j))| {
            for ii in 0..*n {
                let position = f64_value_matrix.get((*row as i64 + (*i as i64 * ii as i64)) as usize, (*col as i64 + (*j as i64 * ii as i64)) as usize).unwrap();
                file.write_f64::<LittleEndian>(*position).unwrap();
            }
        });

        // ES VERDAD QUE NO HAY QUE HACER FILE CLOSE :)
    }
}

#[inline(always)]
#[allow(dead_code)]
fn format_eqs(u: &Vec<Vec<i32>>, w: &Vec<i32>) -> String {
    let mut str_list: Vec<String> = vec![];

    let idx_values = vec!["i", "j", "k", "l"];

    for i in 0..u.len() {
        str_list.push("   ===   ".to_string());
        for j in 0..u[i].len(){
            str_list.push(format!("{sign}{variable} {weight} >= 0", sign={
                match u[i][j] {
                    1 => "+".to_string(),
                    -1 => "-".to_string(),
                    0 => "".to_string(),
                    _ => u[i][j].to_string()
                }
            }, variable={
                match u[i][j] {
                    0 => "",
                    _ => idx_values[j]
                }
            }, weight={
                match w[i] {
                    0 => "\x08".to_string(),    // This adds a backspace for avoiding double space between variable and >=
                    _ => format!("{} {}", {if w[i] < 0 {"-"} else {"+"}}, w[i].abs())
                }
            }));
        }
    }
    str_list.push("\n".to_string());

    str_list.join("")
}
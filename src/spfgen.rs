use std::{fs::{File, self}, path::PathBuf, collections::{HashSet,HashMap}};

use byteorder::{WriteBytesExt, LittleEndian};

type Pattern = (i32, i32, i32);
type Piece = (usize, usize, Pattern);
type Uwc = (Vec<Vec<i32>>, Vec<i32>, Vec<i32>);

pub struct SPFGen {
    ast_list: Vec<Piece>,
    uwc_list: Vec<(usize, Uwc)>,
    pub nrows: usize,
    pub ncols: usize,
    pub nnz: usize,
    pub inc_nnz: usize,
    distinct_patterns: HashMap<Pattern, usize>
}

impl SPFGen {
    pub fn from_piece_list(ast_list: Vec<Piece>, nrows: usize, ncols: usize, nnz: usize) -> Self {
        let ninc_nnz = ast_list.iter().filter(|(_,_,(n,_,_))| *n == 1).count();
        let inc_nnz = nnz - ninc_nnz;
        let mut distinct_patterns: HashMap<Pattern, usize> = HashMap::new();

        // Create distinct pattern HashMap indexed 0..used_patterns with the help of an intermediate HashSet
        ast_list.iter().map(|(_,_,pattern)| *pattern).filter(|(i,_,_)| *i > 1).collect::<HashSet<Pattern>>().into_iter().enumerate().for_each(|(idx, pattern)| {
            distinct_patterns.insert(pattern, idx);
        });
        // Finally insert (1,0,0) pattern. (Has to be the last one):
        distinct_patterns.insert((1,0,0), distinct_patterns.len());

        let uwc_list: Vec<(usize, Uwc)> = ast_list.iter().map(|(row,col,pattern)| (*distinct_patterns.get(pattern).unwrap(), ast_to_uwc((*row,*col,*pattern)))).collect();

        // println!("nrows = {}, ncols = {}, nnz = {}, inc_nnz = {}", nrows, ncols, nnz, inc_nnz);
        // println!("\n{:?}", distinct_patterns);

        return SPFGen {
            ast_list,
            uwc_list,
            nrows,
            ncols,
            nnz,
            inc_nnz,
            distinct_patterns
        };
    }

    pub fn print_ast_list(&self) {
        println!("AST_List:\nRow\tCol\tN\tI\tJ");
        self.ast_list.iter().for_each(|(row, col, (n, i, j))| {
            println!("{}\t{}\t{}\t{}\t{}", row, col, n, i, j);
        });   
    }

    pub fn print_uwc_list(&self, show_eqs: bool) {
        println!("Uwc_List:\nid\tU\t\tw\tc");
        self.uwc_list.iter().for_each(|(id,(u,w,c))| {
            print!("{:?}\t{:?}\t{:?}\t{:?}{}", id, u, w, c, {
                if show_eqs {
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
                } else { "\n".to_string() }
            });
        });
    }

    pub fn write_spf(&self, file_path: &str) {
        
        let mut file = File::create(file_path).expect(format!("Unable to create file {}", file_path).as_str());
        
        let path = PathBuf::from(file_path);
        println!("Writing to file {}", fs::canonicalize(&path).unwrap().display());
        
        // Get index of single nonzeros (not in a pattern to filter them out of the next foreach)
        let ninc_nonzero_pattern_id = self.distinct_patterns.get(&(1,0,0)).unwrap();

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

        // Write maximum dimensionality of iP for vertex_rec (FIXME this currently only supports 1d)
        file.write_i16::<LittleEndian>(0i16).unwrap();

        // FIXME Get shape_dims_max from previous it
        for _ in 0..1 {
            file.write_i32::<LittleEndian>(0i32).unwrap();
        }

        self.uwc_list.iter().filter(|(id,_)| id != ninc_nonzero_pattern_id).for_each(|(id,(u,w,c))| {
            // Write shape id
            file.write_i16::<LittleEndian>(*id as i16).unwrap();
            // Write type of encoding. 0 = vertex_rec, 1 = vertex_gen, 2 = ineqs . FIXME only writes vertex_rec
            file.write_i16::<LittleEndian>(0i16).unwrap();
            // Write dimension of i_p. Hardcodec for vertex_rec
            file.write_i16::<LittleEndian>(u[0].len() as i16).unwrap();
            
            // Get convex_hull (FIXME: Current dimensionality == 1 so dense ch == non-dense ch. Therefore:)
            let ch: Vec<i32> = (w[1]..w[0]).collect();
            // FIXME for higher dimensionality
            for _ in 0..1 {
                file.write_i32::<LittleEndian>(ch[0]).unwrap();
            }

            // Write lenghts along axis            
            // FIXME for higher dimensionality
            for _ in 0..1 {
                // taking shortcut as all input are 1-d
                file.write_i32::<LittleEndian>(w[0]-w[1]).unwrap();
            }

            // "Hardcoded stride at this time"
            for _ in 0..u[0].len() {
                file.write_i32::<LittleEndian>(1i32).unwrap();
            }

            // Write lattice
            for cc in c {
                file.write_i32::<LittleEndian>(*cc).unwrap();
            }

        });

        // ast_list.iter().filter(|(_,_,(n,_,_))| *n == 1).count();    filter single values



    }
}

// TODO fix this for n-dimensional (currently 1D only)
#[inline(always)]
#[allow(dead_code)]
fn ast_to_uwc(ast: Piece) -> Uwc {
    let (_, _, (n, i, j)) = ast;

    let it_range = n-1;

    // TODO fix here for n-dimensional (currently 1D only)
    let u = vec![ vec![-1], vec![1] ];
    let w = vec![ it_range, 0 ];
    let c = vec![ i, j ];

    return (u, w, c);
}
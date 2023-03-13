use std::{fs::{File, self}, path::PathBuf, collections::{HashMap, HashSet}};

use byteorder::{WriteBytesExt, LittleEndian};

type Pattern = (i32, i32, i32);
type Piece = (usize, usize, Pattern);
type Uwc = (Vec<Vec<i32>>, Vec<i32>, Vec<i32>);

pub struct SPFGen {
    ast_list: Vec<Piece>,
    uwc_list: Vec<Uwc>,
    pub nrows: usize,
    pub ncols: usize,
    pub nnz: usize,
    pub inc_nnz: usize,
    distinct_patterns: HashMap<Pattern, usize>
}

impl SPFGen {
    pub fn from_piece_list(ast_list: Vec<Piece>, nrows: usize, ncols: usize, nnz: usize) -> Self {
        let uwc_list: Vec<Uwc> = ast_list.iter().map(|ast| ast_to_uwc(*ast)).collect();
        let ninc_nnz = ast_list.iter().filter(|(_,_,(n,_,_))| *n == 1).count();
        let inc_nnz = nnz - ninc_nnz;
        let mut distinct_patterns: HashMap<Pattern, usize> = HashMap::new();

        // Create distinct pattern HashMap indexed 0..used_patterns with the help of an intermediate HashSet
        ast_list.iter().map(|(_,_,pattern)| *pattern).collect::<HashSet<Pattern>>().into_iter().enumerate().for_each(|(idx, pattern)| {
            distinct_patterns.insert(pattern, idx);
        });

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
        println!("Uwc_List:\nU\t\tw\tc");
        self.uwc_list.iter().for_each(|(U,w,c)| {
            print!("{:?}\t{:?}\t{:?}{}", U, w, c, {
                if show_eqs {
                    let mut str_list: Vec<String> = vec![];

                    let idx_values = vec!["i", "j", "k", "l"];

                    // str_list.push("   ".to_string());
                    for i in 0..U.len() {
                        str_list.push("   ===   ".to_string());
                        for j in 0..U[i].len(){
                            str_list.push(format!("{sign}{variable} {weight} >= 0", sign={
                                match U[i][j] {
                                    1 => "+".to_string(),
                                    -1 => "-".to_string(),
                                    0 => "".to_string(),
                                    _ => U[i][j].to_string()
                                }
                            }, variable={
                                match U[i][j] {
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
        print!("Writing to file {}", fs::canonicalize(&path).unwrap().display());

        // Write header
        file.write_i32::<LittleEndian>(self.nnz as i32).unwrap();
        file.write_i32::<LittleEndian>(self.inc_nnz as i32).unwrap();
        file.write_i32::<LittleEndian>(self.nrows as i32).unwrap();
        file.write_i32::<LittleEndian>(self.ncols as i32).unwrap();

        // Write dimensions
        file.write_i16::<LittleEndian>(2i16).unwrap();
        // number of base shapes is actual found shapes, not unfound ones
        file.write_i32::<LittleEndian>(self.distinct_patterns.len() as i32).unwrap();

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
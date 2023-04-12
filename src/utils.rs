use num_traits::{Num, NumCast};
use sprs::{CsMat};
use stringreader::StringReader;
use std::{io::BufReader, process::{Command, Stdio}};
use colored::Colorize;

/* COMMON TYPES */
pub type Pattern = (i32, i32, i32);
pub type Piece = (usize, usize, Pattern);
pub type Uwc = (Vec<Vec<i32>>, Vec<i32>, Vec<i32>);
pub type OriginUwc = (usize, usize, Uwc);

pub fn read_matrix_market_csr<T: Num+NumCast+Clone>(path: &str) -> CsMat<T> {
    let value_matrix: CsMat<T> = {
        match sprs::io::read_matrix_market(path) {
            Ok(mat) => {mat.to_csr()},
            Err(_) => {
                eprintln!(
                    "\n{} MatrixMarket file was incompatible with {} crate. Trying to convert it on the fly...",
                    "[IMPORTANT]".bold().red(),
                    "sprs".green()
                );

                let cmd_output = Command::new("python3")
                    .arg("./utils/transcode_mm.py").arg(path).arg("stdout")
                    .stdout(Stdio::piped())
                    .output()
                    .expect("Failed to execute python3 script");

                let stdout = String::from_utf8(cmd_output.stdout).unwrap();
                
                let streader = StringReader::new(&stdout);
                let mut bufreader = BufReader::new(streader);

                eprintln!(
                    "{} MatrixMarket file was converted succesfully. If the files will be accessed often, seriously consider transcoding it with the tool located on {} for efficient CPU usage and faster runtime.\n",
                    "[INFO]".bold().blue(),
                    "./utils/transcode_mm.py".bright_blue()
                );

                sprs::io::read_matrix_market_from_bufread(&mut bufreader).unwrap().to_csr()
            },
        }
    };
    return value_matrix;
}

#[inline(always)]
#[allow(dead_code)]
pub fn flatten<T>(nested: Vec<Vec<T>>) -> Vec<T> {
    nested.into_iter().flatten().collect()
}

// TODO fix this for n-dimensional (currently 1D only)
#[inline(always)]
#[allow(dead_code)]
pub fn pattern_to_uwc(pattern: &Pattern) -> Uwc {
    let (n, i, j) = pattern;

    let it_range = n-1;

    // TODO fix here for n-dimensional (currently 1D only)
    let u = vec![ vec![-1], vec![1] ];
    let w = vec![ it_range, 0 ];
    let c = vec![ *i, *j ];

    return (u, w, c);
}

#[inline(always)]
#[allow(dead_code)]
pub fn orig_uwc_to_piece_1d(uwc: &OriginUwc) -> Piece {
    let (x, y, (_,w,c)) = uwc;
    (*x,*y,(w[0]+1,c[0],c[1]))
}

#[inline(always)]
#[allow(dead_code)]
pub fn convex_hull_1d(_u: &Vec<Vec<i32>>, w: &Vec<i32>, _dense: bool) -> Vec<i32>{
    // FIXME: Current dimensionality == 1 so dense ch == non-dense ch. Therefore :)
    (w[1]..=w[0]).collect::<Vec<i32>>()
}
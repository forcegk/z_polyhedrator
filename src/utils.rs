use num_traits::{Num, NumCast};
use sprs::{CsMat};
use stringreader::StringReader;
use std::{io::BufReader, process::{Command, Stdio}};
use colored::Colorize;

use std::io::prelude::*;

/* COMMON TYPES */
pub type Pattern = (i32, i32, i32);
pub type Piece = (usize, usize, Pattern);
pub type Uwc = (Vec<Vec<i32>>, Vec<i32>, Vec<i32>);
pub type OriginUwc = (usize, usize, Uwc);

//                         N    I    J    Order  Sub-Pattern
pub type MetaPattern = ( (i32, i32, i32),  i32,  Option<i32> );
// If Option is None -> N,I,J describe the base pattern

//                             X     Y
pub type MetaPatternPiece = (usize,usize);

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
pub fn pause() {
    let mut stdin = std::io::stdin();
    let mut stdout = std::io::stdout();

    // Lock the line and manually flush
    write!(stdout, "Press any key to continue...").unwrap();
    stdout.flush().unwrap();

    // Read byte and discard
    let _ = stdin.read(&mut [0u8]).unwrap();
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
pub fn convex_hull_1d(_u: &Vec<Vec<i32>>, w: &Vec<i32>, _dense: bool) -> Vec<Vec<i32>>{
    // FIXME: Current dimensionality == 1 so dense ch == non-dense ch. Therefore :)
    (w[1]..=w[0]).map(|w| vec![w]).collect::<Vec<Vec<i32>>>()
}

#[inline(always)]
#[allow(dead_code)]
pub fn convex_hull_rectangle_nd(u: &Vec<Vec<i32>>, w: &Vec<i32>, dense: bool) -> Vec<Vec<i32>> {
    // This code only works for u values like [[-1,0],[0,-1],[1,0],[0,1]]. No values other than 1, 0 or -1 are accepted to this point

    let dims = u[0].len();
    if dims < 2 {
        return convex_hull_1d(u, w, dense);
    }

    // FIXME DEBUG
    let dims = 3;
    // let u = vec![vec![-1,0], vec![0,-1], vec![1,0], vec![0,1]];
    let u = vec![vec![-1,0,0],vec![0,-1,0],vec![0,0,-1],vec![1,0,0],vec![0,1,0],vec![0,0,1]];
    // let w = vec![3,7,0,0];
    let w = vec![3,7,2,0,0,0];

    // let (u_up, u_low) = u.split_at(u.len()/2);
    let (w_up, w_low) = w.split_at(w.len()/2);

    // eprintln!("[ch] u_up = {:?}, u_low = {:?}", u_up, u_low);
    // eprintln!("[ch] w_up = {:?}, w_low = {:?}", w_up, w_low);

    // let ranges: Vec<i32> = w_up.iter().zip(w_low.iter()).map(|(u,l)| *u + *l + 1).collect();

    // eprintln!("[ch] Ranges = {:?}", ranges);

    // let length = {
    //     if dense {
    //         ranges.iter().fold(1usize, |len,val| len as usize * (*val as usize))
    //     } else {
    //         // Initial an final values for last dimension are always two
    //         2usize * ranges.split_last().unwrap().1.iter().fold(1usize, |len,val| len as usize * (*val as usize))
    //     }
    // };

    // eprintln!("[ch] Length = {:?} for dense = {:?}", length, dense);

    let mut ch : Vec<Vec<i32>> = vec![vec![]];

    for idx in 0..dims-{if dense {0} else {1}}{
        ch = c![ {let mut v = cur.clone(); v.push(i); v}, for i in -w_low[idx]..=w_up[idx], for cur in &ch ];
    }

    if !dense {
        ch = c![ {let mut v = cur.clone(); v.push(i); v}, for i in vec![-w_low[dims-1],w_up[dims-1]], for cur in &ch ];
    }

    // eprintln!("[CH] {:?}. Lenght = {:?}", ch, ch.len());

    return ch;
}
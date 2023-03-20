use num_traits::{Num, NumCast};
use sprs::{CsMat};
use stringreader::StringReader;
use std::{io::BufReader, process::{Command, Stdio}};
use colored::Colorize;

pub fn read_matrix_market_csr<T: Num+NumCast+Clone>(path: &str) -> CsMat<T> {
    let value_matrix: CsMat<T> = {
        match sprs::io::read_matrix_market(path) {
            Ok(mat) => {mat.to_csr()},
            Err(_) => {
                println!(
                    "\n{} MatrixMarket file was incompatible with sprs crate. Trying to convert it on the fly...",
                    "[IMPORTANT]".bold().red()
                );

                let cmd_output = Command::new("python3")
                    .arg("./utils/transcode_mm.py").arg(path).arg("stdout")
                    .stdout(Stdio::piped())
                    .output()
                    .expect("Failed to execute python3 script");

                let stdout = String::from_utf8(cmd_output.stdout).unwrap();
                
                let streader = StringReader::new(&stdout);
                let mut bufreader = BufReader::new(streader);

                println!(
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
use std::fmt::format;

type Pattern = (i32, i32, i32);
type Piece = (usize, usize, Pattern);
type Uwc = (Vec<Vec<i32>>, Vec<i32>, Vec<i32>);

pub struct SPFGen {
    ast_list: Vec<Piece>,
    uwc_list: Vec<Uwc>
}

impl SPFGen {
    pub fn from_piece_list(ast_list: Vec<Piece>) -> Self {
        let uwc_list: Vec<Uwc> = ast_list.iter().map(|ast| ast_to_uwc(*ast)).collect();
        return SPFGen {
            ast_list,
            uwc_list
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
type Pattern = (i32, i32, i32);
type Piece = (usize, usize, Pattern);
type Uwc = (Vec<Vec<i32>>, Vec<i32>, Vec<i32>);

pub struct SPFGen {
    ast_list: Vec<Piece>,
    uwc_list: Vec<Uwc>
}

impl SPFGen {
    pub fn from_piece_list(ast_list: Vec<Piece>) -> SPFGen {

        let mut uwc_list: Vec<Uwc>;

        return SPFGen {
            ast_list,
        };
    }

    pub fn print_ast_list(&self) {
        println!("AST_List:\nRow\tCol\tN\tI\tJ");
        self.ast_list.iter().for_each(|(row, col, (n, i, j))| {
            println!("{}\t{}\t{}\t{}\t{}", row, col, n, i, j);
        });   
    }

    // pub fn print_uwc_list(&self) {

    // }
}

#[inline(always)]
#[allow(dead_code)]
fn ast_to_uwc(ast: Piece) {
    let (row, col, (n, i, j)) = ast;

    // OLLIÑO! This can be negative if i < 0
    // let row_range: i32 = (row as i32 + n*i) - row as i32;
    let row_range = n-1;

    // 1st i >= 0    --   2nd -i + row_range >= 0
    // 1st -i >= 0   --   2nd -i - row_range >= 0 

    let mut uwc: Uwc = ()

    let mut uwc: Uwc = (vec![vec![0; n as usize]; n as usize], vec![0; n as usize], vec![0; n as usize]);
    let mut uwc_row: Vec<i32> = vec![0; n as usize];
    let mut uwc_col: Vec<i32> = vec![0; n as usize];
    let mut uwc_val: Vec<i32> = vec![0; n as usize];

    uwc_row[i as usize] = 1;
    uwc_col[j as usize] = 1;
    uwc_val[n as usize] = 1;

    uwc.0[i as usize][j as usize] = 1;
    uwc.1[i as usize] = 1;
    uwc.2[j as usize] = 1;

    // return uwc;
}
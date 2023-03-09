type Pattern = (i32, i32, i32);
type Piece = (usize, usize, Pattern);
type Uwc = (Vec<Vec<i32>>, Vec<i32>, Vec<i32>);

pub struct SPFGen {
    ast_list: Vec<Piece>,
    uwc_list: Vec<Uwc>
}

impl SPFGen {
    pub fn from_piece_list(ast_list: Vec<Piece>) -> SPFGen {

        let mut uwc_list: Vec<Uwc> = ast_list.iter().map(|ast| ast_to_uwc(*ast)).collect();

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

    pub fn print_uwc_list(&self) {
        println!("Uwc_List:\nU\t\tw\tc");
        self.uwc_list.iter().for_each(|(U,w,c)| {
            println!("{:?}\t{:?}\t{:?}", U, w, c);
        });   
    }


    // pub fn print_uwc_list(&self) {

    // }
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
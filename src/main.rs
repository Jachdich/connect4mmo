use std::fmt;
#[derive(Copy, Clone)]
enum Piece {
    None,
    Red,
    Yellow,
}

impl fmt::Display for Piece {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Piece::None => write!(f, " "),
            Piece::Red => write!(f, "x"),
            Piece::Yellow => write!(f, "o"),
        }
    }
}
struct Board {
    data: [[Piece; 7]; 6],
}

impl Board {
    fn new() -> Self {
        let data: [[Piece; 7]; 6] = [[Piece::None; 7]; 6];
        Board { data }
    }
    fn print_board(&self) {
        println!("| 1 | 2 | 3 | 4 | 5 | 6 | 7 |");
        println!("|---|---|---|---|---|---|---|");
        for col in self.data.iter() {
            for row in col.iter() {
                print!("| {} ", row);
            }
            println!("|\n|---|---|---|---|---|---|---|");
        }
    }
}

fn main() {
    let mut board = Board::new();
    loop {
        board.print_board();
        let mut s = String::new();
        std::io::stdin().read_line(&mut s).expect("what");
        if let Some('\n') = s.chars().next_back() {
            s.pop();
        }
        if let Some('\r') = s.chars().next_back() {
            s.pop();
        }
    }
}

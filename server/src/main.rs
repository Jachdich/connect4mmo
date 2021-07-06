use std::fmt;
use std::net::{TcpListener, TcpStream, Shutdown};
use std::io::Write;
use std::io::Read;

#[derive(Copy, Clone, PartialEq, Debug)]
enum Piece {
    None,
    Red,
    Yellow,
}

impl Piece {
    fn from_u8(a: u8) -> Self {
        match a {
            0 => Piece::None,
            1 => Piece::Red,
            2 => Piece::Yellow,
            _ => Piece::None,
        }
    }

    fn to_u8(&self) -> u8 {
        match self {
            Piece::None => 0,
            Piece::Red => 1,
            Piece::Yellow => 2,
        }
    }
}

impl fmt::Display for Piece {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Piece::None => write!(f, " "),
            Piece::Red => write!(f, "\x1b[101m \x1b[0m"),
            Piece::Yellow => write!(f, "\x1b[103m \x1b[0m"),
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
    
    fn from_buffer(buf: &[u8]) -> Self {
        let mut data =  [[Piece::None; 7]; 6];
        for x in 0..7 {
            for y in 0..6 {
                data[y][x] = Piece::from_u8(buf[y * 7 + x]);
            }
        }
        Board { data }
    }

    fn to_buffer(&self) -> [u8; 7 * 6] {
        let mut buf = [0 as u8; 7 * 6];
        for x in 0..7 {
            for y in 0..6 {
               buf[y * 7 + x] = self.data[y][x].to_u8();
            }
        }
        buf
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

    fn place(&mut self, num: usize, colour: Piece) {
        if self.data[0][num - 1] != Piece::None {
            println!("This column is full!");
            return;
        }
        let mut n = 0;
        while n < 6 && self.data[n][num - 1] == Piece::None {
            n += 1;
        }
        self.data[n - 1][num - 1] = colour;
    }

    fn check_win(&self) {
        for x in 0..7 {
            for y in 0..6 {
                
            }
        }
    }
}

fn run_game(mut conns: Vec<TcpStream>) {
    let mut board = Board::new();
    let mut next_player = Piece::Red;
    loop {
        let mut buf: [u8; 1] = [0; 1];
        let idx = if next_player == Piece::Red { 0 } else { 1 };
        let not_idx = if next_player == Piece::Red { 1 } else { 0 };
        conns[idx].write(&[1 as u8; 1]);
        conns[not_idx].write(&[2 as u8; 1]);
        match conns[idx].read(&mut buf) {
            Ok(size) => {
                let pos = buf[0];
                println!("Got pos {} from {}", pos, next_player);
                if pos == 0 {
                    //disconnected
                    conns[idx].shutdown(Shutdown::Both).unwrap();
                    conns[not_idx].shutdown(Shutdown::Both).unwrap();
                } else {
                    board.place(pos as usize, next_player);
                    board.print_board();
                }
            },
            Err(_) => {
                eprintln!("An error occurred, terminating connection with {}", conns[idx].peer_addr().unwrap());
                conns[idx].shutdown(Shutdown::Both).unwrap();
                conns[not_idx].shutdown(Shutdown::Both).unwrap();
            }
        }
        for mut conn in &conns {
            conn.write(&[0 as u8; 1]).unwrap();
            conn.write(&board.to_buffer()).unwrap();
        }

        next_player = if next_player == Piece::Red { Piece::Yellow } else { Piece::Red };
    }
}

fn assign_match(conns: &mut Vec<TcpStream>) {
    if conns.len() >= 2 {
        //make a game with the first two
        let mut new_conns = Vec::<TcpStream>::new();
        new_conns.push(conns.remove(0));
        new_conns.push(conns.remove(0));
        std::thread::spawn(move || {
            run_game(new_conns);
        });
    }
}

fn main() {
    let listener = TcpListener::bind("0.0.0.0:42069").unwrap();
    let mut conns: Vec<TcpStream> = Vec::new();
    for stream in listener.incoming() {
        conns.push(stream.unwrap());
        assign_match(&mut conns);
    }
}

use std::fmt;
use std::net::{TcpStream};
use std::io::{Read, Write, stdout};
use std::thread;
use termion::raw::IntoRawMode;
use termion::input::TermRead;

#[derive(Copy, Clone, PartialEq)]
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

}

impl fmt::Display for Piece {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use termion::color::{Bg, LightRed, LightYellow, Reset};
        match self {
            Piece::None => write!(f, " "),
            Piece::Red => write!(f, "{} {}", Bg(LightRed), Bg(Reset)),
            Piece::Yellow => write!(f, "{} {}", Bg(LightYellow), Bg(Reset)),
        }
    }
}
struct Board {
    data: [[Piece; 7]; 6],
}

impl Board {
    fn new() -> Self {
        Board { data: [[Piece::None; 7]; 6] }
    }

    fn update_from_buffer(&mut self, buf: &[u8]) {
        let new_board = Board::from_buffer(buf);
        self.data = new_board.data;
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
    
    fn print_board(&self, num: usize) {
        for n in 1..8 {
            print!("|{} {} {}", 
                if n == num { termion::style::Invert.to_string() } else { termion::style::NoInvert.to_string() },
                n,
                termion::style::NoInvert,
            );
        }
        println!("|\r");
        for n in 1..8 {
            print!("|{}---{}",
                if n == num { termion::style::Invert.to_string() } else { termion::style::NoInvert.to_string() },
                termion::style::NoInvert
            );
        }
        println!("|\r");
        for col in self.data.iter() {
            for row in col.iter() {
                print!("| {} ", row);
            }
            println!("|\r\n|---|---|---|---|---|---|---|\r");
        }
    }

}

use std::sync::mpsc;
use std::sync::Arc;
use std::sync::Mutex;

enum LocalMessage {
    Event(termion::event::Event),
    ChangeTurn(bool),
}

fn recv_thread(mut stream: TcpStream, tx: mpsc::Sender<LocalMessage>, board: Arc<Mutex<Board>>) {
    loop {
        let mut type_of_message = [0 as u8; 1];
        stream.read(&mut type_of_message).unwrap();
        if type_of_message[0] == 0 { //update board
            let mut buf = [0 as u8; 7 * 6];
            stream.read(&mut buf).unwrap();
            let mut b = board.lock().unwrap();
            (*b).update_from_buffer(&buf);
        } else if type_of_message[0] == 1 { //our turn
            tx.send(LocalMessage::ChangeTurn(true)).unwrap();
        } else if type_of_message[0] == 2 { //not our turn lol
            tx.send(LocalMessage::ChangeTurn(false)).unwrap();
        }
    }
}

fn run_events(tx: mpsc::Sender<LocalMessage>) {
    let stdin = std::io::stdin();

    for event in stdin.events() {
        tx.send(LocalMessage::Event(event.as_ref().unwrap().clone())).unwrap();
    }
}

use termion::event::Key;
use termion::event::Event;

fn main() {
    let mut stream = TcpStream::connect("127.0.0.1:42069").unwrap();
    let other_stream = stream.try_clone().unwrap();
    let mut screen = termion::screen::AlternateScreen::from(stdout()).into_raw_mode().unwrap();

    let board = Arc::new(Mutex::new(Board::new()));
    let other_board = Arc::clone(&board);
    
    let (tx, rx): (mpsc::Sender<LocalMessage>, mpsc::Receiver<LocalMessage>) = mpsc::channel();
    let other_tx = tx.clone();
    thread::spawn(move || {
        recv_thread(other_stream, tx, other_board);
    });
    thread::spawn(move || {
        run_events(other_tx);
    });
    let mut our_turn = false;
    let mut num = 1;
    loop {
        let message = rx.recv().unwrap();
        if let LocalMessage::Event(e) = message {
            match e {
                Event::Key(Key::Char('\n')) => {
                    if our_turn {
                        let buf = [num as u8; 1];
                        stream.write(&buf).unwrap();
                    }
                }
                Event::Key(Key::Ctrl('c')) => break,
                Event::Key(Key::Left)  => { if num > 1 && our_turn { num -= 1; } }
                Event::Key(Key::Right) => { if num < 7 && our_turn { num += 1; } }
                
                _ => ()
            }
            
        } else if let LocalMessage::ChangeTurn(turn) = message {
            our_turn = turn;
        }
        write!(screen, "{}", termion::clear::All).unwrap();
        let b = board.lock().unwrap();
        if our_turn {
            (*b).print_board(num);
        } else {
            (*b).print_board(0);
        }
        drop(b)
    }
}

use core::panic;
use std::{
    collections::VecDeque,
    io::{self, Read},
};

use clap::Parser;

#[derive(Parser)]
struct Cli {
    #[clap(short, long)]
    code: bool,

    #[arg()]
    file: String,
}

fn main() {
    let args = Cli::parse();
    let code = if args.code {
        args.file
    } else {
        std::fs::read_to_string(&args.file).unwrap_or_else(|e| {
            eprintln!("Error reading file: {}", e);
            std::process::exit(1);
        })
    };

    let mut hny = HnyFuck::from_str(&code);
    hny.run();
}

const SHIFT_LEFT: (&str, &str) = ("Happy", "New");
const SHIFT_RIGHT: (&str, &str) = ("New", "Year");
const INCREMENT: (&str, &str) = ("Year", "Happy");
const DECREMENT: (&str, &str) = ("Happy", "Year");
const OUTPUT: (&str, &str) = ("Year", "New");
const INPUT: (&str, &str) = ("New", "Happy");
const LOOP_START: (&str, &str) = ("Happy", "Happy");
const LOOP_END: (&str, &str) = ("New", "New");

fn from_brainfuck(code: &str) -> HnyFuck {
    let hny_code = code
        .chars()
        .map(|c| match c {
            '>' => SHIFT_RIGHT,
            '<' => SHIFT_LEFT,
            '+' => INCREMENT,
            '-' => DECREMENT,
            '.' => OUTPUT,
            ',' => INPUT,
            '[' => LOOP_START,
            ']' => LOOP_END,
            _ => panic!("Invalid character"),
        })
        .map(|(a, b)| format!("{} {}", a, b))
        .collect::<Vec<String>>()
        .join(" ");

    HnyFuck::new(TokenStream::from_str(&hny_code))
}

#[derive(Debug, Clone)]
struct TokenStream {
    tokens: VecDeque<String>,
}

impl TokenStream {
    fn new() -> TokenStream {
        TokenStream {
            tokens: VecDeque::new(),
        }
    }

    fn push(&mut self, token: String) {
        self.tokens.push_back(token);
    }

    fn from_str(input: &str) -> TokenStream {
        let mut stream = TokenStream::new();
        for token in input.split_whitespace() {
            stream.push(token.to_string());
        }
        stream
    }

    fn next(&mut self) -> Option<String> {
        self.tokens.pop_front()
    }

    fn next2(&mut self) -> Option<(String, String)> {
        let first = self.next();
        let second = self.next();
        match (first, second) {
            (Some(f), Some(s)) => Some((f, s)),
            _ => None,
        }
    }

    fn peek(&self) -> Option<&String> {
        self.tokens.front()
    }

    fn peekn(&self, n: usize) -> Option<&String> {
        self.tokens.get(n)
    }
}

#[derive(Debug)]
struct InputStream {
    stdin: io::Bytes<io::Stdin>,
}

impl InputStream {
    fn new() -> InputStream {
        InputStream {
            stdin: io::stdin().bytes(),
        }
    }

    fn next(&mut self) -> Option<u8> {
        self.stdin.next().and_then(|result| result.ok())
    }
}

#[derive(Debug)]
struct State {
    state: VecDeque<u8>,
    index: usize,
    input: InputStream,
}

impl State {
    fn new() -> State {
        let mut state = VecDeque::new();
        state.push_back(0);
        State {
            state,
            index: 0,
            input: InputStream::new(),
        }
    }

    fn shift_left(&mut self) {
        match self.index {
            0 => self.state.push_front(0),
            _ => self.index -= 1,
        }
    }

    fn shiht_right(&mut self) {
        match self.index {
            i if i == self.state.len() - 1 => {
                self.state.push_back(0);
                self.index += 1;
            }
            _ => self.index += 1,
        }
    }

    fn increment(&mut self) {
        if let Some(cell) = self.state.get_mut(self.index) {
            *cell += 1;
        }
    }

    fn decrement(&mut self) {
        if let Some(cell) = self.state.get_mut(self.index) {
            *cell -= 1;
        }
    }

    fn output(&mut self) {
        if let Some(cell) = self.state.get(self.index) {
            print!("{}", *cell as char);
        }
    }

    fn input(&mut self) {
        if let Some(cell) = self.state.get_mut(self.index) {
            if let Some(byte) = self.input.next() {
                *cell = byte;
            }
        }
    }

    fn cond(&self) -> bool {
        self.state.get(self.index).map_or(false, |cell| *cell != 0)
    }
}

#[derive(Debug)]
struct HnyFuck {
    stream: TokenStream,
    state: State,
}

impl HnyFuck {
    fn new(stream: TokenStream) -> Self {
        Self {
            stream,
            state: State::new(),
        }
    }

    fn from_str(input: &str) -> Self {
        Self::new(TokenStream::from_str(input))
    }

    fn run(&mut self) {
        while let Some((first, second)) = self.stream.next2() {
            match (first.as_str(), second.as_str()) {
                SHIFT_LEFT => self.state.shift_left(),
                SHIFT_RIGHT => self.state.shiht_right(),
                INCREMENT => self.state.increment(),
                DECREMENT => self.state.decrement(),
                OUTPUT => self.state.output(),
                INPUT => self.state.input(),
                LOOP_START => {
                    let mut token_stream = TokenStream::new();
                    let mut depth = 1;
                    while let Some((token1, token2)) = self.stream.next2() {
                        match (token1.as_str(), token2.as_str()) {
                            LOOP_START => depth += 1,
                            LOOP_END => {
                                depth -= 1;
                                if depth == 0 {
                                    break;
                                }
                            }
                            _ => (),
                        }
                        token_stream.push(token1);
                        token_stream.push(token2);
                    }

                    let state = std::mem::replace(&mut self.state, State::new());

                    let mut nest = Self {
                        stream: token_stream.clone(),
                        state,
                    };

                    while {
                        nest.run();

                        nest.stream = token_stream.clone();

                        nest.state.cond()
                    } {}

                    self.state = nest.state;
                }
                _ => panic!("Invalid token"),
            }
        }
    }
}

#[cfg(test)]
mod test {
    #[allow(unused_imports)]
    use super::*;

    #[test]
    fn test_increment() {
        let mut hny = from_brainfuck("+++++");
        hny.run();
        assert_eq!(hny.state.state[0], 5);
    }

    #[test]
    fn test_decrement() {
        let mut hny = from_brainfuck("+++++-----");
        hny.run();
        assert_eq!(hny.state.state[0], 0);
    }

    #[test]
    fn test_shift_left_right() {
        let mut hny = from_brainfuck("+++++>+++++<");
        dbg!(&hny);
        hny.run();
        assert_eq!(hny.state.state[0], 5);
        assert_eq!(hny.state.state[1], 5);
    }

    #[test]
    fn test_loop() {
        let mut hny = from_brainfuck("+++++[>+++++<-]");
        hny.run();
        assert_eq!(hny.state.state[0], 0);
        assert_eq!(hny.state.state[1], 25);
    }

    #[test]
    fn test_state_increment() {
        let mut state = State::new();
        state.increment();
        assert_eq!(state.state[0], 1);
    }

    #[test]
    fn test_state_decrement() {
        let mut state = State::new();
        state.increment();
        state.decrement();
        assert_eq!(state.state[0], 0);
    }

    #[test]
    fn test_state_shift_left() {
        let mut state = State::new();
        state.increment();
        state.shift_left();
        assert_eq!(state.state[0], 0);
        assert_eq!(state.state[1], 1);
    }

    #[test]
    fn test_state_shift_right() {
        let mut state = State::new();
        state.increment();
        state.shiht_right();
        state.increment();
        assert_eq!(state.state[0], 1);
        assert_eq!(state.state[1], 1);
    }

    #[test]
    fn test_state_cond() {
        let mut state = State::new();
        assert!(!state.cond());
        state.increment();
        assert!(state.cond());
    }

    #[test]
    fn happy_new_year() {
        let code =
            "Year Happy Year Happy Year Happy Year Happy Year Happy Year Happy Year Happy Year Happy Year Happy Year Happy Happy Happy New Year Year Happy New Year Year Happy Year Happy Year Happy New Year Year Happy Year Happy Year Happy Year Happy Year Happy Year Happy Year Happy New Year Year Happy Year Happy Year Happy Year Happy Year Happy Year Happy Year Happy Year Happy Year Happy Year Happy Happy New Happy New Happy New Happy New Happy Year New New New Year New Year New Year Year Happy Year Happy Year New New Year Happy Year Happy Year Happy Year Year New Year Happy Year Happy Year Happy Year Happy Year Happy Year Happy Year Happy Year Happy Year Happy Year Happy Year Happy Year Happy Year Happy Year Happy Year Happy Year New Year New Year Happy Year Happy Year Happy Year Happy Year Happy Year Happy Year Happy Year Happy Year Happy Year New Happy New Happy New Year Happy Year Happy Year New New Year Year Happy Year Happy Year Happy Year Happy Year Happy Year Happy Year New New Year Happy Year Happy Year Happy Year Happy Year Happy Year Happy Year Happy Year Happy Year Happy Year Happy Year Happy Year Happy Year Happy Year Happy Year Happy Year Happy Year Happy Year Happy Year Happy Year Happy Year Year New Year Happy Year Happy Year Happy Year Happy Year Happy Year Happy Year Happy Year Happy Year Happy Year Happy Year Happy Year Happy Year Happy Year Happy Year Happy Year Happy Year Happy Year Happy Year New Happy New Happy New Year New New Year Year Happy Year Happy Year Happy Year Happy Year Happy Year Happy Year Happy Year Happy Year Happy Year Happy Year Happy Year New Year Happy Year Happy Year Happy Year Happy Year Happy Year Happy Year Happy Year Happy Year Happy Year Happy Year Happy Year Happy Year New Happy Year Happy Year Happy Year Happy Year Year New New Year Happy Year Happy Year Happy Year Happy Year Happy Year Year New Happy New Happy New Year Happy Year New";

        let mut hny = HnyFuck::new(TokenStream::from_str(code));
        hny.run();
    }
}

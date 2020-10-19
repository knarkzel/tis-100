use owo_colors::OwoColorize;
use rustyline::Editor;
use std::path::PathBuf;
use anyhow::{Result, anyhow};
use crate::{Instruction::*, Port::*};

#[derive(Debug, Default)]
struct Node {
    acc: isize,
    bak: isize,
    instructions: Vec<Instruction>,
}

impl Node {
    fn new() -> Self {
        Self::default()
    }
    fn parse(&mut self, line: &str) -> Result<()> {
        let words = line.to_lowercase().split_whitespace().map(|x| x.to_string()).collect::<Vec<_>>();
        let instruction = match &*words[0] {
            "mov" => {
                let first = Port::parse(words[1].trim_end_matches(','))?;
                let second = Port::parse(&words[2])?;
                Mov(first, second)
            }
            _ => return Err(anyhow!("Failed to read `{}`", line.red())),
        };
        self.instructions.push(instruction);
        Ok(())
    }
    fn show(&self) {
        // println!("{}: {}", "ACC".green(), self.acc);
        // println!("{}: {}", "BAK".green(), self.bak);
        println!("{}: {:#?}", "INSTRUCTIONS".green(), self.instructions);
    }
}

#[derive(Debug, Default)]
struct Label {
    name: String,
    line: usize,
}

#[allow(dead_code)]
#[derive(Debug)]
enum Port {
    Acc,
    Left,
    Right,
    Up,
    Down,
    Any,
    Last,
    Value(isize),
}

impl Port {
    fn parse(token: &str) -> Result<Self> {
        let port = match token {
            "acc" => Acc,
            "left" => Left,
            "right" => Right,
            "up" => Up,
            "down" => Down,
            "any" => Any,
            "last" => Last,
            value => {
                let parsed = value.parse::<isize>();
                match parsed {
                    Ok(number) => return Ok(Value(number)),
                    Err(_) => return Err(anyhow!("Failed to parse token {}", token.red())),
                }
            }
        };
        Ok(port)
    }
}

#[allow(dead_code)]
#[derive(Debug)]
enum Instruction {
    Nop,
    Swp,
    Sav,
    Neg,
    Comment(String),
    Label(Label),
    Jmp(Label),
    Jez(Label),
    Jnz(Label),
    Jgz(Label),
    Jlz(Label),
    Jro(Port),
    Add(Port),
    Sub(Port),
    Mov(Port, Port),
    Write(PathBuf),
}

fn main() {
    let mut node = Node::new();
    let mut rl = Editor::<()>::new();
    loop {
        let readline = rl.readline("> ");
        match readline {
            Ok(line) => {
                let line = line.trim();
                if line.is_empty() {
                    continue;
                } else if line.starts_with("#") {
                    let comment = Comment(line[1..].to_string());
                    node.instructions.push(comment);
                    continue;
                }
                let parse = node.parse(line);
                if let Err(parse) = parse {
                    println!("{}", parse);
                    node.show();
                }
            }
            Err(_) => break,
        }
    }
}

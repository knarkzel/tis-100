use owo_colors::OwoColorize;
use rustyline::Editor;
use anyhow::{Result, anyhow};
use crate::{Instruction::*, Port::*};
use serde::{Serialize, Deserialize};
use ron::{ser, de};
use std::fs;

#[derive(Debug, Default, Serialize, Deserialize)]
struct Node {
    acc: isize,
    bak: isize,
    pc: usize,
    instructions: Vec<Instruction>,
}

impl Node {
    fn new() -> Self {
        Self::default()
    }
    fn parse(&mut self, line: &str, i: &mut usize) -> Result<()> {
        macro_rules! push {
            ($e:expr) => {
                let instructions = &mut self.instructions;
                instructions.push($e);
                *i += 1;
            };
        }
        let words = line.to_lowercase().split_whitespace().map(|x| x.to_string()).collect::<Vec<_>>();
        if line.starts_with("#") {
            push!(Comment(line[1..].to_string()));
            return Ok(());
        } else if line.contains(':') {
            let label = Label {
                name: line.trim_end_matches(':').to_string(),
                line: *i,
            };
            push!(Label(label));
            return Ok(());
        }
        match &*words[0] {
            "show" => {
                self.show();
            }
            "clear" => {
                self.instructions = vec![];
            }
            "save" => {
                let title = if words.len() > 1 {
                    format!("{}.ron", words[1])
                } else {
                    format!("node.ron")
                };
                let content = ser::to_string_pretty(self, ser::PrettyConfig::default())?;
                fs::write(title, content)?;
            }
            "load" => {
                let title = if words.len() > 1 {
                    format!("{}.ron", words[1])
                } else {
                    format!("node.ron")
                };
                let file = fs::File::open(title)?;
                let node: Self = de::from_reader(file)?;
                *self = node;
            }
            "nop" => (),
            "mov" => {
                let first = Port::parse(words[1].trim_end_matches(','))?;
                let second = Port::parse(&words[2])?;
                push!(Mov(first, second));
            }
            "swp" => {
                let temp = self.acc;
                self.acc = self.bak;
                self.bak = temp;
            }
            "sav" => {
                self.bak = self.acc;    
            }
            _ => return Err(anyhow!("Failed to read `{}`", line.red())),
        };
        Ok(())
    }
    fn show(&self) {
        println!("{}: {}", "ACC".green(), self.acc);
        println!("{}: {}", "BAK".green(), self.bak);
        println!("{}: {:#?}", "INSTRUCTIONS".green(), self.instructions);
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct Label {
    name: String,
    line: usize,
}

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize)]
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
#[derive(Debug, Serialize, Deserialize)]
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
}

fn main() {
    let mut node = Node::new();
    let mut rl = Editor::<()>::new();
    let mut i = 0;
    loop {
        let readline = rl.readline("> ");
        match readline {
            Ok(line) => {
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }
                let parse = node.parse(line, &mut i);
                if let Err(parse) = parse {
                    println!("{}", parse);
                }
            }
            Err(_) => break,
        }
    }
}

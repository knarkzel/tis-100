use owo_colors::OwoColorize;
use rustyline::Editor;
use anyhow::{Result, anyhow};
use crate::{Instruction::*, Port::*};
use serde::{Serialize, Deserialize};
use ron::{ser, de};
use std::fs;
use std::collections::HashMap;

#[derive(Debug, Default, Serialize, Deserialize)]
struct Program {
    pos: Pos,
    nodes: HashMap<Pos, Node>,
}

impl Program {
    fn get_node(&mut self, pos: Pos) -> &mut Node {
        self.nodes.entry(pos).or_insert(Node::new())
    }
    fn check_commands(&mut self, line: &str) -> Result<()> {
        let node = self.get_node(self.pos);
        let words = line.to_lowercase().split_whitespace().map(|x| x.to_string()).collect::<Vec<_>>();
        match line {
            "show" => {
                node.show();
            }
            "clear" => {
                node.instructions = vec![];
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
            "inst" => {
                for inst in node.instructions.iter() {
                    println!("{:?}", inst.green());
                }
            }
            _ => {
                node.parse(line, words)?;
            }
        }
        Ok(())
    }
}

#[derive(Debug, Default, Eq, PartialEq, Hash, Serialize, Deserialize, Copy, Clone)]
struct Pos(isize, isize);

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
    fn parse(&mut self, line: &str, words: Vec<String>) -> Result<()> {
        macro_rules! push {
            ($e:expr) => {
                let instructions = &mut self.instructions;
                instructions.push($e);
                return Ok(());
            };
        }
        macro_rules! jump {
            ($i:ident) => {
                let label = Label(words[1].clone());
                push!($i(label));
            };
        }

        if line.starts_with("#") {
            push!(Comment(line[1..].to_string()));
        } else if line.contains(':') {
            let label = Label(line.trim_end_matches(':').to_string());
            push!(CreateLabel(label));
        }

        match &*words[0] {
            "nop" => (),
            "mov" => {
                let first = Port::parse(words[1].trim_end_matches(','))?;
                let second = Port::parse(&words[2])?;
                push!(Mov(first, second));
            }
            "swp" => {
                push!(Swp);
            }
            "sav" => {
                push!(Sav);
            }
            "neg" => {
                push!(Neg);
            }
            "jmp" => {
                jump!(Jmp);
            }
            "jez" => {
                jump!(Jez);
            }
            "jnz" => {
                jump!(Jnz);
            }
            "jgz" => {
                jump!(Jgz);
            }
            "jlz" => {
                jump!(Jlz);
            }
            "jro" => {
                let port = Port::parse(&words[1])?;
                push!(Jro(port));
            }
            "add" => {
                let port = Port::parse(&words[1])?;
                push!(Add(port));
            }
            "sub" => {
                let port = Port::parse(&words[1])?;
                push!(Sub(port));
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

#[derive(Debug, Serialize, Deserialize)]
struct Label(String);

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
    Nil,
    Value(isize),
}

impl Port {
    fn parse(token: &str) -> Result<Self> {
        let port = match token {
            "acc" => Acc,
            "nil" => Nil,
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
    CreateLabel(Label),
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
    let mut program = Program::default();
    let mut rl = Editor::<()>::new();
    loop {
        let readline = rl.readline("> ");
        match readline {
            Ok(line) => {
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }
                let error = program.check_commands(line);
                if let Err(error) = error {
                    println!("{}", error);
                }
            }
            Err(_) => break,
        }
    }
}

use crate::{Instruction::*, Port::*};
use anyhow::{anyhow, Result};
use owo_colors::OwoColorize;
use ron::{de, ser};
use rustyline::Editor;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;

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
        let words = line
            .to_lowercase()
            .split_whitespace()
            .map(|x| x.to_string())
            .collect::<Vec<_>>();
        match line {
            "show" => {
                node.show();
            }
            "help" => {
                println!("{}", "Following commands are available for nodes:".green());
                println!("# <comment>");
                println!("<label>:");
                println!("add <port>");
                println!("sub <port>");
                println!("nop");
                println!("sav");
                println!("neg");
                println!("neg");
                println!("jmp <label>");
                println!("jez <label>");
                println!("jnz <label>");
                println!("jgz <label>");
                println!("jlz <label>");
                println!("jro <port>");
                println!("");
                println!("{}", "Other commands:".green());
                println!("show");
                println!("clear");
                println!("save");
                println!("load");
                println!("inst");
                println!("up");
                println!("down");
                println!("left");
                println!("right");
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
                let program: Self = de::from_reader(file)?;
                *self = program;
            }
            "inst" => {
                for inst in node.instructions.iter() {
                    println!("{:?}", inst.green());
                }
            }
            "left" => self.pos.0 -= 1,
            "right" => self.pos.0 += 1,
            "up" => self.pos.1 -= 1,
            "down" => self.pos.1 += 1,
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
    label_positions: Vec<(Label, usize)>,
    instructions: Vec<Instruction>,
}

impl Node {
    fn new() -> Self {
        Self::default()
    }
    fn run_instruction(&mut self) {
        macro_rules! step {
            () => {
                self.pc += 1;
                if self.pc > self.instructions.len() {
                    self.pc = 0;
                }
            };
        }
        macro_rules! jump_label {
            ($i:ident) => {
                let position = self.label_positions.iter().position(|x| x.0 == *$i);
                if let Some(position) = position {
                    self.pc = position;
                } else {
                    step!();
                }
            };
        }
        let instruction = self.instructions.get(self.pc);
        if let Some(instruction) = instruction {
            match &*instruction {
                Swp => {
                    let temp = self.bak;
                    self.bak = self.acc;
                    self.acc = temp;
                    step!();
                }
                Sav => {
                    self.bak = self.acc;
                    step!();
                }
                Neg => {
                    if self.acc < 0 {
                        self.acc -= 1;
                    }
                    step!();
                }
                CreateLabel(label) => {
                    if self.label_positions.iter().all(|x| x.0 != *label) {
                        self.label_positions.push(((*label).clone(), self.pc));
                    }
                    step!();
                }
                Jmp(label) => {
                    jump_label!(label);
                }
                Jez(label) => {
                    if self.acc == 0 {
                        jump_label!(label);
                    } else {
                        step!();
                    }
                }
                Jnz(label) => {
                    if self.acc != 0 {
                        jump_label!(label);
                    } else {
                        step!();
                    }
                }
                Jgz(label) => {
                    if self.acc > 0 {
                        jump_label!(label);
                    } else {
                        step!();
                    }
                }
                Jlz(label) => {
                    if self.acc <= 0 {
                        jump_label!(label);
                    } else {
                        step!();
                    }
                }
                Jro(_port) => {
                    todo!();
                }
                Add(_port) => {
                    todo!();
                }
                Sub(_port) => {
                    todo!();
                }
                Mov(_from_port, _to_port) => {
                    todo!();
                }
                _ => {
                    step!();
                }
            }
        }
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
            "pop" => {
                let inst = self.instructions.pop();
                if let Some(inst) = inst {
                    println!("{:?}", inst.green())
                }
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

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
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

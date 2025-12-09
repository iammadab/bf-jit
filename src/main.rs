use std::{
    fmt::{self, Display},
    fs,
    io::Read,
};

const MEMORY_SIZE: usize = 30_000;

#[derive(PartialEq, Clone, Hash, Eq, Debug)]
enum Opcode {
    IncPtr,
    DecPtr,
    IncData,
    DecData,
    ReadStdin,
    WriteStdout,
    JumpIfDataZero(usize),
    JumpIfDataNotZero(usize),
}

impl Display for Opcode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Opcode::IncPtr => write!(f, ">"),
            Opcode::DecPtr => write!(f, "<"),
            Opcode::IncData => write!(f, "+"),
            Opcode::DecData => write!(f, "-"),
            Opcode::ReadStdin => write!(f, ","),
            Opcode::WriteStdout => write!(f, "."),
            Opcode::JumpIfDataZero(_) => write!(f, "["),
            Opcode::JumpIfDataNotZero(_) => write!(f, "]"),
        }
    }
}

struct Program {
    instructions: Vec<Opcode>,
}

impl Program {
    fn from_source(source: String) -> Self {
        let mut instructions = Vec::with_capacity(source.len());

        let mut bracket_stack = vec![];

        for c in source.chars() {
            let insn = match c {
                '>' => Opcode::IncPtr,
                '<' => Opcode::DecPtr,
                '+' => Opcode::IncData,
                '-' => Opcode::DecData,
                ',' => Opcode::ReadStdin,
                '.' => Opcode::WriteStdout,
                '[' => Opcode::JumpIfDataZero(instructions.len()),
                ']' => Opcode::JumpIfDataNotZero(instructions.len()),
                _ => continue,
            };

            if let Opcode::JumpIfDataZero(opening_pc) = insn {
                bracket_stack.push(opening_pc);
            }

            if let Opcode::JumpIfDataNotZero(closing_pc) = insn {
                if bracket_stack.is_empty() {
                    panic!("unmatched ']' at pc={}", closing_pc);
                }

                let opening_pc = bracket_stack.pop().unwrap();
                instructions[opening_pc] = Opcode::JumpIfDataZero(closing_pc);
                instructions.push(Opcode::JumpIfDataNotZero(opening_pc));
                continue;
            }

            instructions.push(insn);
        }

        Self { instructions }
    }

    fn execute(&self) {
        let mut memory = [0_u8; MEMORY_SIZE];
        let mut pc = 0;
        let mut data_ptr = 0;

        #[cfg(feature = "tracing")]
        let mut insn_count = std::collections::HashMap::new();

        while pc < self.instructions.len() {
            let insn = &self.instructions[pc];

            #[cfg(feature = "tracing")]
            insn_count.entry(insn).and_modify(|v| *v += 1).or_insert(1);

            match insn {
                // advance the data ptr to the right by 1
                Opcode::IncPtr => data_ptr += 1,
                // advance the data ptr to the left by 1
                Opcode::DecPtr => data_ptr -= 1,
                // increment the memory slot at the data ptr
                Opcode::IncData => memory[data_ptr] = memory[data_ptr].wrapping_add(1),
                // decrement the memory slot at the data ptr
                Opcode::DecData => memory[data_ptr] = memory[data_ptr].wrapping_sub(1),
                // print the content of the data ptr to stdout
                Opcode::WriteStdout => print!("{}", memory[data_ptr] as char),
                // read from stdin and write to memory slot at data ptr
                Opcode::ReadStdin => memory[data_ptr] = read_byte(),
                // jumps to the matching `]`
                // if the current data location is zero
                Opcode::JumpIfDataZero(closing_pc) => {
                    if memory[data_ptr] == 0 {
                        pc = *closing_pc;
                    }
                }
                // jumps to the matching '['
                // if the current data location is not zero
                Opcode::JumpIfDataNotZero(opening_pc) => {
                    if memory[data_ptr] != 0 {
                        pc = *opening_pc;
                    }
                }
            }

            pc += 1;
        }

        #[cfg(feature = "tracing")]
        {
            // print tracing report
            for (k, v) in &insn_count {
                println!("{} -> {}", k, comma_format(*v));
            }
            println!("Total: {}", comma_format(insn_count.values().sum::<u64>()));
        }
    }
}

fn main() {
    let args = std::env::args().collect::<Vec<String>>();
    let source = fs::read_to_string(&args[1]).unwrap();

    let program = Program::from_source(source);
    program.execute();
}

fn read_byte() -> u8 {
    let mut buf = [0u8; 1];
    match std::io::stdin().read(&mut buf) {
        Ok(1) => buf[0],
        _ => 0,
    }
}

#[cfg(feature = "tracing")]
fn comma_format(n: u64) -> String {
    let s = n.to_string();
    let mut out = vec![];
    for (i, c) in s.chars().rev().enumerate() {
        if i != 0 && i % 3 == 0 {
            out.push(',');
        }
        out.push(c);
    }
    out.into_iter().rev().collect()
}

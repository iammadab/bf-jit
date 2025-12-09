use std::{fs, io::Read};

const MEMORY_SIZE: usize = 30_000;

enum Opcode {
    IncPtr,
    DecPtr,
    IncData,
    DecData,
    ReadStdin,
    WriteStdout,
    JumpIfDataZero,
    JumpIfDataNotZero,
}

struct Program {
    instructions: Vec<char>,
    jump_table: Vec<usize>,
}

impl Program {
    fn from_source(source: String) -> Self {
        let mut instructions = Vec::with_capacity(source.len());

        for c in source.chars() {
            match c {
                '>' | '<' | '+' | '-' | '.' | ',' | '[' | ']' => instructions.push(c),
                _ => {}
            }
        }

        let mut pc = 0;

        // compute jump table
        // given that the program isn't changing, we can compute
        // '[' and ']' match pc values once
        // then amortize across loop iterations
        let mut jump_table = vec![0; instructions.len()];
        while pc < instructions.len() {
            let insn = instructions[pc];
            if insn == '[' {
                // find pc for matching ']'
                let mut bracket_nesting = 1;
                let mut seek = pc;

                while bracket_nesting > 0 {
                    seek += 1;
                    if seek >= instructions.len() {
                        panic!("unmatched '[' at pc={}", pc);
                    }

                    match instructions[seek] {
                        ']' => bracket_nesting -= 1,
                        '[' => bracket_nesting += 1,
                        _ => {}
                    }
                }

                // map '[' to ']' and ']' to '['
                jump_table[pc] = seek;
                jump_table[seek] = pc;
            }
            pc += 1;
        }

        Self {
            instructions,
            jump_table,
        }
    }

    fn execute(&self) {
        let mut memory = [0_u8; MEMORY_SIZE];
        let mut pc = 0;
        let mut data_ptr = 0;

        #[cfg(feature = "tracing")]
        let mut insn_count = std::collections::HashMap::new();

        while pc < self.instructions.len() {
            let insn = self.instructions[pc];

            #[cfg(feature = "tracing")]
            insn_count.entry(insn).and_modify(|v| *v += 1).or_insert(1);

            match insn {
                // advance the data ptr to the right by 1
                '>' => data_ptr += 1,
                // advance the data ptr to the left by 1
                '<' => data_ptr -= 1,
                // increment the memory slot at the data ptr
                '+' => memory[data_ptr] = memory[data_ptr].wrapping_add(1),
                // decrement the memory slot at the data ptr
                '-' => memory[data_ptr] = memory[data_ptr].wrapping_sub(1),
                // print the content of the data ptr to stdout
                '.' => print!("{}", memory[data_ptr] as char),
                // read from stdin and write to memory slot at data ptr
                ',' => memory[data_ptr] = read_byte(),
                // jumps to the matching `]`
                // if the current data location is zero
                '[' => {
                    if memory[data_ptr] == 0 {
                        pc = self.jump_table[pc];
                    }
                }
                // jumps to the matching '['
                // if the current data location is not zero
                ']' => {
                    if memory[data_ptr] != 0 {
                        pc = self.jump_table[pc];
                    }
                }
                _ => {}
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

use std::{fs, io::Read};

const MEMORY_SIZE: usize = 30_000;

fn main() {
    let mut memory = [0; MEMORY_SIZE];

    let args = std::env::args().collect::<Vec<String>>();
    let program = fs::read_to_string(&args[1]).unwrap();

    let mut instructions = Vec::with_capacity(program.len());

    for c in program.chars() {
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

    // reset PC
    pc = 0;

    // interpret
    let mut data_ptr = 0;

    while pc < instructions.len() {
        let insn = instructions[pc];
        match insn {
            // advance the data ptr to the right by 1
            '>' => data_ptr += 1,
            // advance the data ptr to the left by 1
            '<' => data_ptr -= 1,
            // increment the memory slot at the data ptr
            '+' => memory[data_ptr] += 1,
            // decrement the memory slot at the data ptr
            '-' => memory[data_ptr] -= 1,
            // print the content of the data ptr to stdout
            '.' => print!("{}", memory[data_ptr] as char),
            // read from stdin and write to memory slot at data ptr
            ',' => memory[data_ptr] = read_byte(),
            // jumps to the matching `]`
            // if the current data location is zero
            '[' => {
                if memory[data_ptr] == 0 {
                    pc = jump_table[pc];
                }
            }
            // jumps to the matching '['
            // if the current data location is not zero
            ']' => {
                if memory[data_ptr] != 0 {
                    pc = jump_table[pc];
                }
            }
            _ => {}
        }

        pc += 1;
    }
}

fn read_byte() -> u8 {
    let mut buf = [0u8; 1];
    match std::io::stdin().read(&mut buf) {
        Ok(1) => buf[0],
        _ => 0,
    }
}

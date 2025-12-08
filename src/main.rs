const MEMORY_SIZE: usize = 30_000;

fn main() {
    let mut memory = [0; 30_000];

    let program = "++++++++ ++++++++ ++++++++ ++++++++ ++++++++ ++++++++
>+++++
[<+.>-]";

    let mut instructions = Vec::with_capacity(program.len());

    for c in program.chars() {
        match c {
            '>' | '<' | '+' | '-' | '.' | ',' | '[' | ']' => instructions.push(c),
            _ => {}
        }
    }

    // interpret
    let mut pc = 0;
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
            '.' => println!("{}", memory[data_ptr]),
            // read from stdin and write to memory slot at data ptr
            ',' => memory[data_ptr] = read_byte(),
            // jumps to the matching `]`
            // if the current data location is zero
            '[' => {
                if memory[data_ptr] == 0 {
                    let mut bracket_nesting = 1;
                    // pc points to '['
                    let saved_pc = pc;

                    pc += 1;
                    while bracket_nesting > 1 && pc < instructions.len() {
                        match instructions[pc] {
                            ']' => bracket_nesting -= 1,
                            '[' => bracket_nesting += 1,
                            _ => {}
                        }
                    }

                    if bracket_nesting != 0 {
                        panic!("unmatched '[' at pc={}", saved_pc);
                    }

                    // TODO insert break here
                }
            }
            // jumps to the matching '['
            // if the current data location is not zero
            ']' => {
                if memory[data_ptr] != 0 {
                    let mut bracket_nesting = 1;
                    // pc points to ']'
                    let saved_pc = pc;

                    while bracket_nesting > 1 && pc > 0 {
                        pc -= 1;
                        match instructions[pc] {
                            '[' => bracket_nesting -= 1,
                            ']' => bracket_nesting += 1,
                            _ => {}
                        }
                    }

                    if bracket_nesting != 0 {
                        panic!("unmatched ']' at pc={}", saved_pc);
                    }

                    // TODO insert break here
                }
            }
            _ => {}
        }

        // I don't think I should increment the PC in all cases
        // I think for all the others I need to
        // but I need to be careful around the branch statements
        pc += 1;
    }
}

fn read_byte() -> u8 {
    let mut buf = [0u8; 1];
    match io::stdin().read(&mut buf) {
        Ok(1) => buf[0],
        _ => 0,
    }
}

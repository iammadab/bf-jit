use crate::Program;
use crate::parser::Opcode;
use std::io::Read;

pub(crate) fn interpret(program: &Program) {
    let mut memory = [0_u8; 30_000];
    let mut pc = 0;
    let mut data_ptr = 0;

    #[cfg(feature = "tracing")]
    let mut insn_count = std::collections::HashMap::new();

    while pc < program.instructions.len() {
        let insn = &program.instructions[pc];

        #[cfg(feature = "tracing")]
        insn_count
            .entry(format!("{}", insn))
            .and_modify(|v| *v += 1)
            .or_insert(1);

        match insn {
            // advance the data ptr to the right by 1
            Opcode::IncPtr(count) => data_ptr += *count as usize,
            // advance the data ptr to the left by 1
            Opcode::DecPtr(count) => data_ptr -= *count as usize,
            // increment the memory slot at the data ptr
            Opcode::IncData(count) => memory[data_ptr] = memory[data_ptr].wrapping_add(*count),
            // decrement the memory slot at the data ptr
            Opcode::DecData(count) => memory[data_ptr] = memory[data_ptr].wrapping_sub(*count),
            // print the content of the data ptr to stdout
            Opcode::WriteStdout => print!("{}", memory[data_ptr] as char),
            // read from stdin and write to memory slot at data ptr
            Opcode::ReadStdin => memory[data_ptr] = read_byte(),
            // set the current memory value to 0
            Opcode::LoopSetToZero => memory[data_ptr] = 0,
            // advance the data ptr by +/- stride
            Opcode::LoopMovePtr(stride, positive) => {
                while memory[data_ptr] != 0 {
                    if *positive {
                        data_ptr += *stride as usize
                    } else {
                        data_ptr -= *stride as usize
                    }
                }
            }
            // add the current of src data to the +/- stride memory slot
            Opcode::LoopMoveData(stride, positive) => {
                if memory[data_ptr] != 0 {
                    let new_addr = if *positive {
                        data_ptr + *stride as usize
                    } else {
                        data_ptr - *stride as usize
                    };

                    memory[new_addr] += memory[data_ptr];
                    memory[data_ptr] = 0;
                }
            }
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

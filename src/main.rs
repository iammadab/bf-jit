use std::{
    fmt::{self, Display},
    fs,
    io::Read,
    iter::Peekable,
    str::Chars,
};

const MEMORY_SIZE: usize = 30_000;

#[derive(PartialEq, Clone, Hash, Eq, Debug)]
enum Opcode {
    IncPtr(u8),
    DecPtr(u8),
    IncData(u8),
    DecData(u8),
    ReadStdin,
    WriteStdout,
    LoopSetToZero,
    LoopMovePtr(u8, bool),
    LoopMoveData(u8, bool),
    JumpIfDataZero(usize),
    JumpIfDataNotZero(usize),
}

impl Display for Opcode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Opcode::IncPtr(_) => write!(f, ">"),
            Opcode::DecPtr(_) => write!(f, "<"),
            Opcode::IncData(_) => write!(f, "+"),
            Opcode::DecData(_) => write!(f, "-"),
            Opcode::ReadStdin => write!(f, ","),
            Opcode::WriteStdout => write!(f, "."),
            Opcode::LoopSetToZero => writeln!(f, "LOOP_SET_TO_ZERO"),
            Opcode::LoopMovePtr(_, _) => write!(f, "LOOP_MOVE_PTR"),
            Opcode::LoopMoveData(_, _) => write!(f, "LOOP_MOVE_DATA"),
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

        let mut source_iter = source.chars().into_iter().peekable();

        while source_iter.peek().is_some() {
            let insn = match source_iter.next().unwrap() {
                '>' => Opcode::IncPtr(count_occ('>', &mut source_iter)),
                '<' => Opcode::DecPtr(count_occ('<', &mut source_iter)),
                '+' => Opcode::IncData(count_occ('+', &mut source_iter)),
                '-' => Opcode::DecData(count_occ('-', &mut source_iter)),
                ',' => Opcode::ReadStdin,
                '.' => Opcode::WriteStdout,
                '[' => Opcode::JumpIfDataZero(instructions.len()),
                ']' => Opcode::JumpIfDataNotZero(instructions.len()),
                _ => continue,
            };

            if let Opcode::JumpIfDataZero(opening_pc) = insn {
                bracket_stack.push(opening_pc);
            }

            // at this point we get a closing square bracket
            // this means we have closed some loop
            // we also have the opening and closing value
            // hence we can get some slice of instructions
            // we pass this to the loop optimizer
            // it gives us either nothing or something
            // if it gives us something we replace the range with it
            // if it gives us nothing we continue as normal
            // biggest uncertainy is the replace range
            // v.truncate(len) then push
            // next is to just implement loop optimization
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

        // ensure we closed all loops
        if !bracket_stack.is_empty() {
            panic!("unmatched '[' at pc={}", bracket_stack[0]);
        }

        Self { instructions }
    }

    fn optimize_loops(insn: &[Opcode]) -> Option<Opcode> {
        match insn {
            // LOOP_SET_TO_ZERO
            // [-] -> [ DEC_DATA(1) ]
            // This idiom decrements the current cell until it reaches zero.
            // Effectively: value_at(data_ptr) = 0.
            [Opcode::DecData(1)] => Some(Opcode::LoopSetToZero),

            // LOOP_MOV_DATA
            // [->>>+<<<] -> [DEC_DATA(1), INC_PTR(n), INC_DATA(1), DEC_PTR(n)]
            // [-<<<+>>>] -> [DEC_DATA(1), DEC_PTR(n), INC_DATA(1), INC_PTR(n)]
            // These instruction patterns transfer the value at data_ptr to data_ptr ± n:
            // they decrement the source cell to zero, and increment the target cell by the
            // original value (i.e. dst = dst_old + src_old, src = 0).
            [
                Opcode::DecData(1),
                Opcode::IncPtr(n),
                Opcode::IncData(1),
                Opcode::DecPtr(m),
            ] if n == m => Some(Opcode::LoopMoveData(*n, true)),

            [
                Opcode::DecData(1),
                Opcode::DecPtr(n),
                Opcode::IncData(1),
                Opcode::IncPtr(m),
            ] if n == m => Some(Opcode::LoopMoveData(*n, false)),

            // LOOP_MOVE_PTR
            // [>>>..] -> [INC_PTR(n)]
            // [<<<..] -> [DEC_PTR(n)]
            // These loops move the data pointer in strides of n (either +n or -n),
            // continuing as long as the current cell is non-zero. The loop stops
            // when the pointer lands on a cell containing 0.y stride n, until it lands on a cell that contains a 0
            [Opcode::IncPtr(n)] => Some(Opcode::LoopMovePtr(*n, true)),
            [Opcode::DecPtr(n)] => Some(Opcode::LoopMovePtr(*n, false)),

            _ => None,
        }
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
                // TODO add documentation
                Opcode::LoopSetToZero => memory[data_ptr] = 0,
                // TODO add documentation
                Opcode::LoopMovePtr(stride, positive) => {
                    while memory[data_ptr] != 0 {
                        if *positive {
                            data_ptr += *stride as usize
                        } else {
                            data_ptr -= *stride as usize
                        }
                    }
                }
                // TODO add documentation
                Opcode::LoopMoveData(stride, positive) => {
                    let new_addr = if *positive {
                        data_ptr + *stride as usize
                    } else {
                        data_ptr - *stride as usize
                    };

                    memory[new_addr] = memory[data_ptr];
                    memory[data_ptr] = 0;
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

fn count_occ(val: char, iterator: &mut Peekable<Chars>) -> u8 {
    let mut count = 1;
    while let Some(c) = iterator.peek() {
        if *c == val {
            // consume
            iterator.next();
            count += 1;
        } else {
            break;
        }
    }
    count
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

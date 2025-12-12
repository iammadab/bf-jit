use std::fmt::{self, Display};
use std::iter::Peekable;
use std::str::Chars;

#[derive(PartialEq, Clone, Hash, Eq, Debug)]
pub(crate) enum Opcode {
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
            Opcode::LoopSetToZero => write!(f, "LOOP_SET_TO_ZERO"),
            Opcode::LoopMovePtr(_, _) => write!(f, "LOOP_MOVE_PTR"),
            Opcode::LoopMoveData(_, _) => write!(f, "LOOP_MOVE_DATA"),
            Opcode::JumpIfDataZero(_) => write!(f, "["),
            Opcode::JumpIfDataNotZero(_) => write!(f, "]"),
        }
    }
}

pub(crate) struct Program {
    pub(crate) instructions: Vec<Opcode>,
}

impl Program {
    pub(crate) fn from_source(source: String) -> Self {
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

            if let Opcode::JumpIfDataNotZero(closing_pc) = insn {
                if bracket_stack.is_empty() {
                    panic!("unmatched ']' at pc={}", closing_pc);
                }

                let opening_pc = bracket_stack.pop().unwrap();

                let loop_slice = &instructions[opening_pc + 1..];
                let optimized_loop = Self::optimize_loops(loop_slice);

                if let Some(loop_insn) = optimized_loop {
                    instructions.truncate(opening_pc);
                    instructions.push(loop_insn)
                } else {
                    instructions[opening_pc] = Opcode::JumpIfDataZero(closing_pc);
                    instructions.push(Opcode::JumpIfDataNotZero(opening_pc));
                }

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loop_optimization() {
        let program = Program::from_source(String::from("[-]"));
        assert_eq!(program.instructions.len(), 1);
        assert_eq!(program.instructions[0], Opcode::LoopSetToZero);

        let program = Program::from_source(String::from("[>>]"));
        assert_eq!(program.instructions.len(), 1);
        assert_eq!(program.instructions[0], Opcode::LoopMovePtr(2, true));

        let program = Program::from_source(String::from("[->>>+<<<]"));
        assert_eq!(program.instructions.len(), 1);
        assert_eq!(program.instructions[0], Opcode::LoopMoveData(3, true));

        let program = Program::from_source(String::from(">>>[-<<<<<<+>>>>>>]"));
        assert_eq!(program.instructions.len(), 2);
        assert_eq!(program.instructions[0], Opcode::IncPtr(3));
        assert_eq!(program.instructions[1], Opcode::LoopMoveData(6, false));
    }
}

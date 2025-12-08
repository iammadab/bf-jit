const MEMORY_SIZE: usize = 30_000;

fn main() {
    let memory = [0; 30_000];

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
}

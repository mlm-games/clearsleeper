const LIMIT: usize = 20;

fn widest_tuple(source: &str) -> usize {
    let start = source
        .find("add_chained_systems(")
        .expect("schedule registration call");
    let bytes = source.as_bytes();
    let mut widths: Vec<usize> = Vec::new();
    let mut stack: Vec<usize> = Vec::new();
    let mut i = start;
    let mut in_string = false;
    let mut in_line_comment = false;
    let mut in_block_comment = false;
    while i < bytes.len() {
        let c = bytes[i];
        let next = bytes.get(i + 1).copied().unwrap_or(0);
        if in_line_comment {
            if c == b'\n' {
                in_line_comment = false;
            }
            i += 1;
            continue;
        }
        if in_block_comment {
            if c == b'*' && next == b'/' {
                in_block_comment = false;
                i += 2;
                continue;
            }
            i += 1;
            continue;
        }
        if in_string {
            if c == b'\\' {
                i += 2;
                continue;
            }
            if c == b'"' {
                in_string = false;
            }
            i += 1;
            continue;
        }
        match c {
            b'/' if next == b'/' => {
                in_line_comment = true;
                i += 2;
                continue;
            }
            b'/' if next == b'*' => {
                in_block_comment = true;
                i += 2;
                continue;
            }
            b'"' => in_string = true,
            b'(' => stack.push(0),
            b',' => {
                if let Some(top) = stack.last_mut() {
                    *top += 1;
                }
            }
            b')' => {
                if stack.len() > 1 {
                    let commas = stack.pop().unwrap_or(0);
                    widths.push(commas + 1);
                } else {
                    stack.pop();
                }
            }
            _ => {}
        }
        i += 1;
    }
    widths.into_iter().max().unwrap_or(0)
}

#[test]
fn schedule_tuples_stay_within_bevy_arity() {
    let source = include_str!("../src/sim/mod.rs");
    assert!(
        source.contains("add_chained_systems("),
        "schedule registration call disappeared"
    );
    let widest = widest_tuple(source);
    assert!(
        widest <= LIMIT,
        "widest schedule tuple has {widest} elements, bevy_ecs implements IntoScheduleConfigs for 1..={LIMIT}"
    );
}
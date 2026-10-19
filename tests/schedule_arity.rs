const LIMIT: usize = 20;

/// One open parenthesis: how many commas its group holds, whether anything
/// but whitespace and comments followed the last of them, and whether the
/// group is a tuple at all (an array or block holds members too, but its
/// length is not what the arity limit is about).
struct Group {
    commas: usize,
    content: bool,
    paren: bool,
}

impl Group {
    fn members(&self) -> usize {
        match (self.commas, self.content) {
            (0, false) => 0,
            (n, true) => n + 1,
            (n, false) => n,
        }
    }
}

fn widest_tuple(source: &str) -> usize {
    let start = source
        .find("add_chained_systems(")
        .expect("schedule registration call");
    let bytes = source.as_bytes();
    let mut widths: Vec<usize> = Vec::new();
    let mut stack: Vec<Group> = Vec::new();
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
            b'"' => {
                in_string = true;
                mark_content(&mut stack);
            }
            b'(' => {
                mark_content(&mut stack);
                stack.push(Group {
                    commas: 0,
                    content: false,
                    paren: true,
                });
            }
            b'[' | b'{' => {
                mark_content(&mut stack);
                stack.push(Group {
                    commas: 0,
                    content: false,
                    paren: false,
                });
            }
            b',' => {
                if let Some(top) = stack.last_mut() {
                    top.commas += 1;
                    top.content = false;
                }
            }
            b')' | b']' | b'}' => {
                if let Some(group) = stack.pop()
                    && group.paren
                    && !stack.is_empty()
                {
                    widths.push(group.members());
                }
            }
            _ => {
                if !c.is_ascii_whitespace() {
                    mark_content(&mut stack);
                }
            }
        }
        i += 1;
    }
    widths.into_iter().max().unwrap_or(0)
}

fn mark_content(stack: &mut [Group]) {
    if let Some(top) = stack.last_mut() {
        top.content = true;
    }
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

#[test]
fn the_counter_reads_members_not_commas() {
    let source = "x.add_chained_systems((a, b, c), (d, e));";
    assert_eq!(widest_tuple(source), 3);

    let source = "x.add_chained_systems((a, b, c,), (d, e,));";
    assert_eq!(widest_tuple(source), 3, "a trailing comma is not a member");

    let source = "x.add_chained_systems((only,), ());";
    assert_eq!(widest_tuple(source), 1, "an empty group has no members");

    let source = "x.add_chained_systems((a, [0, 1, 2, 3, 4], c));";
    assert_eq!(
        widest_tuple(source),
        3,
        "an array is not a tuple member list"
    );

    let source = "x.add_chained_systems((a, b /* , c */));";
    assert_eq!(widest_tuple(source), 2, "a comment is not a member");
}

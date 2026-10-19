use clearsleeper::trace::{self, TickLine};

fn main() {
    let mut args = std::env::args().skip(1);
    let (left, right) = match (args.next(), args.next()) {
        (Some(left), Some(right)) if args.next().is_none() => (left, right),
        _ => {
            eprintln!("usage: trace-diff <left.trace> <right.trace>");
            std::process::exit(2);
        }
    };
    let left = read(&left);
    let right = read(&right);
    let Some(report) = trace::diff_report(&left, &right) else {
        return;
    };
    println!("{}", report.message());
    std::process::exit(1);
}

fn read(path: &str) -> Vec<TickLine> {
    let text = match std::fs::read_to_string(path) {
        Ok(text) => text,
        Err(error) => {
            eprintln!("trace-diff: {path}: {error}");
            std::process::exit(1);
        }
    };
    match trace::parse_text(&text) {
        Ok(lines) => lines,
        Err(error) => {
            eprintln!("trace-diff: {path}: {error}");
            std::process::exit(1);
        }
    }
}

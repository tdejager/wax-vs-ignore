use std::path::Path;
// Use items from the library crate of this package
use wax_vs_ignore::{collect_with_ignore, collect_with_wax, PATTERNS};

fn main() {
    let root = Path::new(".");
    match collect_with_wax(root, PATTERNS) {
        Ok(v) => println!("wax total matches: {}", v.len()),
        Err(e) => eprintln!("wax error: {}", e),
    }
    match collect_with_ignore(root, PATTERNS) {
        Ok(v) => println!("ignore total matches: {}", v.len()),
        Err(e) => eprintln!("ignore error: {}", e),
    }
}

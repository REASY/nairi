use std::env;
use std::fs;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: dump <file.smali>");
        std::process::exit(1);
    }
    let code = fs::read_to_string(&args[1]).unwrap();
    let mut parser = tree_sitter::Parser::new();
    parser.set_language(tree_sitter_smali::language()).unwrap();
    let tree = parser.parse(&code, None).unwrap();
    let root = tree.root_node();

    fn print_errors(node: tree_sitter::Node, code: &str, depth: usize) {
        if node.is_error() || node.is_missing() {
            println!(
                "{:indent$}Error node: '{}' at {}:{}",
                "",
                node.utf8_text(code.as_bytes()).unwrap().replace('\n', " "),
                node.start_position().row,
                node.start_position().column,
                indent = depth * 2
            );
        }
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            print_errors(child, code, depth + 1);
        }
    }
    print_errors(root, &code, 0);
}

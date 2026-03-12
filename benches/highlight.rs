use fluxbench::Bencher;
use fluxbench::bench;

fn generate_rust_code(lines: usize) -> String {
    let mut code = String::new();
    for i in 0..lines {
        code.push_str(&format!(
            "fn func_{i}(x: i32, y: &str) -> Option<Vec<String>> {{\n    let result = x + {i}; // comment\n    println!(\"value: {{}}\", result);\n    Some(vec![y.to_string()])\n}}\n\n"
        ));
    }
    code
}

#[bench]
fn highlight_100_lines(b: &mut Bencher) {
    let code = generate_rust_code(17);
    b.iter(|| oxidoc_highlight::highlight(&code, "rust"));
}

#[bench]
fn highlight_1000_lines(b: &mut Bencher) {
    let code = generate_rust_code(167);
    b.iter(|| oxidoc_highlight::highlight(&code, "rust"));
}

#[bench]
fn highlight_js_500_lines(b: &mut Bencher) {
    let mut code = String::new();
    for i in 0..100 {
        code.push_str(&format!(
            "function func_{i}(x, y) {{\n  const result = x + {i};\n  console.log(`value: ${{result}}`);\n  return {{ name: \"test\", value: result }};\n}}\n\n"
        ));
    }
    b.iter(|| oxidoc_highlight::highlight(&code, "javascript"));
}

fn main() {
    fluxbench::run().unwrap();
}

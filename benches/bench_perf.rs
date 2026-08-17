/// Chronos Performance Benchmark Suite
/// Standalone benchmark measuring lexer and parser execution latency.
///
/// Build: rustc -O benches/bench_perf.rs -o benches/bench_perf.exe
/// Run:   ./benches/bench_perf.exe

use std::time::{Instant, SystemTime};

fn tokenize(input: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut chars = input.chars().peekable();

    while let Some(&c) = chars.peek() {
        match c {
            '\'' => {
                chars.next();
                while let Some(&ch) = chars.peek() {
                    if ch == '\'' { chars.next(); break; }
                    current.push(ch);
                    chars.next();
                }
            }
            '"' => {
                chars.next();
                while let Some(&ch) = chars.peek() {
                    if ch == '"' { chars.next(); break; }
                    if ch == '\\' { chars.next(); if let Some(&esc) = chars.peek() { current.push(esc); chars.next(); } continue; }
                    current.push(ch);
                    chars.next();
                }
            }
            '\\' => {
                chars.next();
                if let Some(&next) = chars.peek() {
                    current.push(next);
                    chars.next();
                }
            }
            ' ' | '\t' => {
                if !current.is_empty() { tokens.push(current.clone()); current.clear(); }
                chars.next();
            }
            '|' | '&' | ';' | '>' | '<' => {
                if !current.is_empty() { tokens.push(current.clone()); current.clear(); }
                let op = c;
                chars.next();
                if let Some(&next) = chars.peek() {
                    if next == op || (op == '>' && next == '>') || (op == '2' && next == '>') {
                        tokens.push(format!("{}{}", op, next));
                        chars.next();
                    } else {
                        tokens.push(op.to_string());
                    }
                } else {
                    tokens.push(op.to_string());
                }
            }
            '2' if chars.clone().nth(1) == Some('>') => {
                if !current.is_empty() { tokens.push(current.clone()); current.clear(); }
                chars.next(); chars.next();
                tokens.push("2>".to_string());
            }
            _ => {
                current.push(c);
                chars.next();
            }
        }
    }
    if !current.is_empty() { tokens.push(current); }
    tokens
}

enum Redirect {
    None,
    Overwrite(String),
    Append(String),
}

struct Command {
    name: String,
    args: Vec<String>,
    stdout: Redirect,
    stderr: Redirect,
}

fn parse(tokens: Vec<String>) -> Option<Command> {
    if tokens.is_empty() { return None; }
    let name = tokens[0].clone();
    let mut args = Vec::new();
    let mut stdout = Redirect::None;
    let mut stderr = Redirect::None;
    let mut iter = tokens.into_iter().skip(1);
    while let Some(token) = iter.next() {
        match token.as_str() {
            ">" | "1>" => { if let Some(f) = iter.next() { stdout = Redirect::Overwrite(f); } }
            ">>" | "1>>" => { if let Some(f) = iter.next() { stdout = Redirect::Append(f); } }
            "2>" => { if let Some(f) = iter.next() { stderr = Redirect::Overwrite(f); } }
            "2>>" => { if let Some(f) = iter.next() { stderr = Redirect::Append(f); } }
            _ => { args.push(token); }
        }
    }
    Some(Command { name, args, stdout, stderr })
}

struct BenchResult {
    name: String,
    category: String,
    iterations: usize,
    total_ns: u128,
    mean_ns: f64,
    p50_ns: f64,
    p95_ns: f64,
    p99_ns: f64,
    min_ns: u128,
    max_ns: u128,
    throughput_ops_sec: f64,
}

fn run_bench<F: FnMut()>(name: &str, category: &str, iterations: usize, mut f: F) -> BenchResult {
    let warmup = std::cmp::max(100, iterations / 10);
    for _ in 0..warmup {
        f();
    }

    let mut timings: Vec<u128> = Vec::with_capacity(iterations);
    let total_start = Instant::now();

    for _ in 0..iterations {
        let start = Instant::now();
        f();
        timings.push(start.elapsed().as_nanos());
    }

    let total_elapsed = total_start.elapsed();
    timings.sort();

    let total_ns = total_elapsed.as_nanos();
    let mean_ns = total_ns as f64 / iterations as f64;
    let p50_idx = iterations / 2;
    let p95_idx = std::cmp::min((iterations as f64 * 0.95) as usize, iterations - 1);
    let p99_idx = std::cmp::min((iterations as f64 * 0.99) as usize, iterations - 1);

    BenchResult {
        name: name.to_string(),
        category: category.to_string(),
        iterations,
        total_ns,
        mean_ns,
        p50_ns: timings[p50_idx] as f64,
        p95_ns: timings[p95_idx] as f64,
        p99_ns: timings[p99_idx] as f64,
        min_ns: *timings.first().unwrap(),
        max_ns: *timings.last().unwrap(),
        throughput_ops_sec: if total_ns > 0 {
            iterations as f64 / (total_ns as f64 / 1_000_000_000.0)
        } else { 0.0 },
    }
}

fn main() {
    let iterations = 10_000;
    let mut results = Vec::new();

    // A. Simple Inputs
    results.push(run_bench("lexer_empty", "A_simple", iterations, || { let _ = tokenize(""); }));
    results.push(run_bench("lexer_single_token", "A_simple", iterations, || { let _ = tokenize("ls"); }));
    results.push(run_bench("lexer_simple_command", "A_simple", iterations, || { let _ = tokenize("echo hello world"); }));
    results.push(run_bench("parser_single_command", "A_simple", iterations, || { let t = tokenize("ls"); let _ = parse(t); }));
    results.push(run_bench("e2e_simple_echo", "A_simple", iterations, || { let t = tokenize("echo hello world"); let _ = parse(t); }));

    // B. Complex Inputs
    results.push(run_bench("lexer_pipeline_5_stage", "B_complex", iterations, || { let _ = tokenize("cat file.txt | grep pattern | sort | uniq -c | head -n 10"); }));
    results.push(run_bench("lexer_dual_redirect", "B_complex", iterations, || { let _ = tokenize("cmd arg1 arg2 > out.txt 2> err.txt"); }));
    results.push(run_bench("lexer_operator_chain", "B_complex", iterations, || { let _ = tokenize("cmd1 && cmd2 || cmd3 ; cmd4 | cmd5 &"); }));
    results.push(run_bench("lexer_mixed_quotes", "B_complex", iterations, || { let _ = tokenize(r#"echo "hello world" 'single quoted' "with \"escape""#); }));
    results.push(run_bench("lexer_variables", "B_complex", iterations, || { let _ = tokenize("echo $HOME ${USER} $PATH"); }));
    results.push(run_bench("parser_dual_redirect", "B_complex", iterations, || { let t = tokenize("echo data > out.txt 2> err.txt"); let _ = parse(t); }));
    results.push(run_bench("e2e_grep_redirect", "B_complex", iterations, || { let t = tokenize("grep -rn --color=always 'pattern' dir/ > output.txt"); let _ = parse(t); }));

    // C. Stress / Scale Inputs
    let input_100: String = (0..100).map(|i| format!("arg{}", i)).collect::<Vec<_>>().join(" ");
    results.push(run_bench("lexer_100_tokens", "C_stress", iterations, || { let _ = tokenize(&input_100); }));
    let input_1000: String = (0..1000).map(|i| format!("tok{}", i)).collect::<Vec<_>>().join(" ");
    results.push(run_bench("lexer_1000_tokens", "C_stress", iterations / 10, || { let _ = tokenize(&input_1000); }));
    let deep_pipe: String = (0..50).map(|_| "cmd").collect::<Vec<_>>().join(" | ");
    results.push(run_bench("lexer_50_pipe_chain", "C_stress", iterations, || { let _ = tokenize(&deep_pipe); }));
    results.push(run_bench("lexer_16_quoted_strings", "C_stress", iterations, || { let _ = tokenize(r#""a" "b" "c" "d" "e" "f" "g" "h" "i" "j" "k" "l" "m" "n" "o" "p""#); }));
    let long_string = "a".repeat(10_000);
    let long_cmd = format!("echo {}", long_string);
    results.push(run_bench("lexer_10k_char_token", "C_stress", iterations / 10, || { let _ = tokenize(&long_cmd); }));

    // D. Malformed / Adversarial
    results.push(run_bench("lexer_unclosed_single_quote", "D_malformed", iterations, || { let _ = tokenize("echo 'unclosed string"); }));
    results.push(run_bench("lexer_unclosed_double_quote", "D_malformed", iterations, || { let _ = tokenize("echo \"unclosed string"); }));
    results.push(run_bench("lexer_trailing_backslash", "D_malformed", iterations, || { let _ = tokenize("echo trail\\"); }));
    results.push(run_bench("lexer_operator_only", "D_malformed", iterations, || { let _ = tokenize("> >> | || && ; &"); }));
    results.push(run_bench("lexer_whitespace_only", "D_malformed", iterations, || { let _ = tokenize("      \t\t\t     "); }));

    // E. End-to-End Full Pipeline
    results.push(run_bench("e2e_rm_rf_root", "E_e2e", iterations, || { let t = tokenize("rm -rf /"); let _ = parse(t); }));
    results.push(run_bench("e2e_complex_find_pipe", "E_e2e", iterations, || { let t = tokenize("find . -name '*.rs' | xargs grep -l 'test' | sort > results.txt 2> errors.txt"); let _ = parse(t); }));
    results.push(run_bench("e2e_append_redirect", "E_e2e", iterations, || { let t = tokenize("echo data >> log.txt"); let _ = parse(t); }));

    println!("══════════════════════════════════════════════════════════════════════════════════════════════════════════");
    println!("  CHRONOS PERFORMANCE BENCHMARK RESULTS");
    println!("══════════════════════════════════════════════════════════════════════════════════════════════════════════");
    println!("{:<35} {:>10} {:>10} {:>10} {:>10} {:>10} {:>10} {:>14}",
        "Benchmark", "Iters", "Mean(ns)", "p50(ns)", "p95(ns)", "p99(ns)", "Max(ns)", "Throughput");
    println!("{}", "─".repeat(115));
    let mut last_cat = String::new();
    for r in &results {
        if r.category != last_cat {
            if !last_cat.is_empty() { println!(); }
            last_cat = r.category.clone();
        }
        println!("{:<35} {:>10} {:>10.0} {:>10.0} {:>10.0} {:>10.0} {:>10} {:>11.0} op/s",
            r.name, r.iterations, r.mean_ns, r.p50_ns, r.p95_ns, r.p99_ns, r.max_ns, r.throughput_ops_sec);
    }
}

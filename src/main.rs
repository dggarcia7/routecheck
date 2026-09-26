mod router;

use router::{find_shadowed, match_segments, method_matches, parse_route_line, split_path, Route};
use std::env;
use std::fs;
use std::process::ExitCode;

fn usage() {
    eprintln!("usage: routecheck <routes-file> <method> <path>");
    eprintln!("       routecheck --check <routes-file>");
    eprintln!("example: routecheck routes.txt GET /users/42");
}

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();

    if args.len() == 3 && args[1] == "--check" {
        return run_check(&args[2]);
    }

    if args.len() != 4 {
        usage();
        return ExitCode::FAILURE;
    }
    let routes_file = &args[1];
    let method = &args[2];
    let path = &args[3];

    let routes = match load_routes(routes_file) {
        Ok(routes) => routes,
        Err(e) => {
            eprintln!("routecheck: {e}");
            return ExitCode::FAILURE;
        }
    };

    let path_segs = split_path(path);

    for route in &routes {
        if !method_matches(&route.method, method) {
            continue;
        }
        if let Some(params) = match_segments(&route.segments, &path_segs) {
            println!("match: {} {} ({routes_file}:{})", route.method, route.pattern, route.line);
            if params.is_empty() {
                println!("params: none");
            } else {
                for (name, value) in &params {
                    println!("  {name} = {value}");
                }
            }
            return ExitCode::SUCCESS;
        }
    }

    println!("no match for {method} {path}");
    ExitCode::FAILURE
}

fn run_check(routes_file: &str) -> ExitCode {
    let routes = match load_routes(routes_file) {
        Ok(routes) => routes,
        Err(e) => {
            eprintln!("routecheck: {e}");
            return ExitCode::FAILURE;
        }
    };

    let issues = find_shadowed(&routes);
    if issues.is_empty() {
        println!("no shadowed or duplicate routes found in {routes_file}");
        return ExitCode::SUCCESS;
    }

    for issue in &issues {
        println!(
            "line {}: {} {} is never reached, shadowed by line {} ({} {})",
            issue.shadowed.line,
            issue.shadowed.method,
            issue.shadowed.pattern,
            issue.shadowed_by.line,
            issue.shadowed_by.method,
            issue.shadowed_by.pattern,
        );
    }
    ExitCode::FAILURE
}

fn load_routes(path: &str) -> Result<Vec<Route>, String> {
    let contents = fs::read_to_string(path).map_err(|e| format!("cannot read {path}: {e}"))?;
    let mut routes = Vec::new();
    for (i, raw_line) in contents.lines().enumerate() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        routes.push(parse_route_line(line, i + 1)?);
    }
    Ok(routes)
}

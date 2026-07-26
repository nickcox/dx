//! Binary entry point. Everything lives in the `dx` library so tests and
//! benches can reach it.

fn main() {
    let code = dx::cli::run();
    std::process::exit(code);
}

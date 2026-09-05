//! Binary entrypoint for `forgum`.

fn main() -> std::process::ExitCode {
    forgum_engine::runner::run()
}

fn main() {
    eprintln!(
        "warning: `aps` is deprecated and will be removed in a future release, use `apss` instead."
    );
    std::process::exit(apss::run());
}

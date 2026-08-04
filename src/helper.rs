use std::fs;

pub fn init_tracing(port: &str) {
    fs::create_dir_all("logs").expect("Failed to create logs directory");

    let file_appender = tracing_appender::rolling::never("logs", format!("node_{}.log", port));

    let (non_blocking_writer, _guard) = tracing_appender::non_blocking(file_appender);

    tracing_subscriber::fmt()
        .with_writer(non_blocking_writer)
        .with_target(false)
        .with_ansi(false)
        .init();
}

pub fn print_banner(port: &str) {
    println!("🚀 Node started on port {}", port);
    println!("📂 Background events are writing via tracing to 'logs/node_{}.log'", port);
    println!("Type 'help' for commands");
}
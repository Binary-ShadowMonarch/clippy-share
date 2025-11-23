#[tokio::main]
async fn main() {
    println!("Starting daemon...");
    let daemon = core_daemon::CoreDaemon::new();
    daemon.run().await;
}

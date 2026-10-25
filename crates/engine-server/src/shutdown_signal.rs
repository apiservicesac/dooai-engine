use tokio::signal::unix::{SignalKind, signal};

/// Completes when the process is asked to stop (Ctrl-C or SIGTERM from Docker).
pub struct ShutdownSignal;

impl ShutdownSignal {
    pub async fn wait() {
        let mut terminate = signal(SignalKind::terminate()).expect("SIGTERM handler installs");
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {}
            _ = terminate.recv() => {}
        }
    }
}

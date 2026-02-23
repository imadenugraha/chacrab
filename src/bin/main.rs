#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_target(false)
        .without_time()
        .init();

    if let Err(err) = chacrab::cli::commands::run().await {
        if !matches!(err, chacrab::core::errors::ChacrabError::Config(_)) {
            tracing::debug!(error = ?err, "chacrab command failed");
        } else {
            tracing::debug!("chacrab command failed with redacted configuration error");
        }
        std::process::exit(1);
    }
}

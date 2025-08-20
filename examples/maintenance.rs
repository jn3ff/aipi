#[cfg(feature = "dev-tools")]
use aipi::models::maintenance;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, Layer, fmt as trace_fmt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    setup_tracing();
    run_maintenance().await;
    Ok(())
}

fn setup_tracing() {
    let filter = EnvFilter::try_new("info,aipi=trace,reqwest=warn,hyper=warn").unwrap();
    let json_layer = trace_fmt::layer().json().with_filter(filter);

    tracing_subscriber::registry().with(json_layer).init();
}

#[cfg(feature = "dev-tools")]
async fn run_maintenance() -> () {
    maintenance::fetch_and_display_metadata().await;
}

#[cfg(not(feature = "dev-tools"))]
async fn run_maintenance() -> () {
    eprintln!("maintenance script does nothing when feature dev-tools is not used");
}

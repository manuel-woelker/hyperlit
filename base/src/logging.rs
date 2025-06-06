use tracing_subscriber::EnvFilter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

pub fn init_logging() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer(), /*.with_span_events(FmtSpan::ENTER)*/
        )
        .with(
            EnvFilter::builder()
                .parse("INFO,hyperlit_core=DEBUG")
                .unwrap(),
        )
        .init();
}

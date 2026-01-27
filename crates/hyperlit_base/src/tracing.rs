use crate::error::HyperlitResult;
pub use tracing::instrument;
pub use tracing::{debug, error, info, trace, warn};
use tracing_error::ErrorLayer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

pub fn init_tracing() -> HyperlitResult<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(ErrorLayer::default())
        .init();
    Ok(())
}

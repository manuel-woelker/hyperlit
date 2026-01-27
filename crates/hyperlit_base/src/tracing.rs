use crate::error::HyperlitResult;
pub use tracing::instrument;
pub use tracing::{debug, error, info, trace, warn};

pub fn init_tracing() -> HyperlitResult<()> {
    tracing_subscriber::fmt::init();
    Ok(())
}

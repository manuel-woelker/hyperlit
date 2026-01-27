use crate::error::Result;
pub use tracing::instrument;
pub use tracing::{debug, error, info, trace, warn};

pub fn init_tracing() -> Result<()> {
    tracing_subscriber::fmt::init();
    Ok(())
}

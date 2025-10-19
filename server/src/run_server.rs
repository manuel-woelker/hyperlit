use crate::server::HyperlitServer;
use hyperlit_base::result::HyperlitResult;
use hyperlit_pal_real::PalReal;

pub fn run_hyperlit_server() -> HyperlitResult<()> {
    let pal = PalReal::new();
    HyperlitServer::new(pal).run()?;
    std::thread::park();
    Ok(())
}

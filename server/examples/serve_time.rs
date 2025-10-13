use hyperlit_pal_real::PalReal;
use hyperlit_server::server::HyperlitServer;

fn main() {
    let pal = PalReal::new();
    let result = HyperlitServer::new(pal).run();
    if let Err(e) = result {
        eprintln!("Error: {}", e);
    }
    std::thread::park()
}

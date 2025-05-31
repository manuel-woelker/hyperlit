use hyperlit_base::result::HyperlitResult;

pub struct Runner {

}

impl Runner {
    pub fn new() -> Self {
        Self {}
    }

    pub fn run(&self) -> HyperlitResult<()> {
        Ok(())
    }
}
use hyperlit_base::error::err;
use hyperlit_base::result::HyperlitResult;
use hyperlit_core::config::HyperlitConfig;
use hyperlit_export_html::html_exporter::export_book_to_html;
use hyperlit_model::book::Book;
use hyperlit_model::value::Value;
use hyperlit_pal::{FilePath, Pal, PalHandle};
use parking_lot::{
    MappedRwLockReadGuard, MappedRwLockWriteGuard, RwLock, RwLockReadGuard, RwLockWriteGuard,
};

pub struct HyperlitEngine {
    #[allow(dead_code)]
    pal: PalHandle,
    state: RwLock<Result<EngineState, String>>,
}

struct EngineState {
    #[allow(dead_code)]
    config: HyperlitConfig,
    book: Book,
}

impl HyperlitEngine {
    pub fn new_handle(pal: PalHandle) -> Self {
        Self {
            pal,
            state: RwLock::new(Err("not initialized".to_string())),
        }
    }

    pub fn new(pal: impl Pal + 'static) -> Self {
        Self::new_handle(PalHandle::new(pal))
    }

    pub fn init(&self) {
        let result = (|| -> HyperlitResult<EngineState> {
            let mut config_file = self.pal.read_file(&FilePath::from("hyperlit.toml"))?;
            let mut config_string = String::new();
            config_file.read_to_string(&mut config_string)?;
            let config = HyperlitConfig::from_string(&config_string)?;
            let book = Book::new(Value::String(config.title.clone()));
            Ok(EngineState { config, book })
        })()
        .map_err(|err| err.to_string());
        *self.state.write() = result;
    }

    fn read(&self) -> HyperlitResult<MappedRwLockReadGuard<'_, EngineState>> {
        let read_guard = self.state.read();

        let mapped_guard = match read_guard.as_ref() {
            Ok(_state) => RwLockReadGuard::map(read_guard, |state| state.as_ref().unwrap()),
            Err(err) => {
                return Err(err!("Could not acquire read lock: {:?}", err));
            }
        };
        Ok(mapped_guard)
    }

    #[allow(dead_code)]
    fn write(&self) -> HyperlitResult<MappedRwLockWriteGuard<'_, EngineState>> {
        let write_guard = self.state.write();

        let mapped_guard = match write_guard.as_ref() {
            Ok(_state) => RwLockWriteGuard::map(write_guard, |state| state.as_mut().unwrap()),
            Err(err) => {
                return Err(err!("Could not acquire write lock: {:?}", err));
            }
        };
        Ok(mapped_guard)
    }

    pub fn render_book_html(&self) -> HyperlitResult<String> {
        let read = self.read()?;
        let book = &read.book;
        export_book_to_html(book)
    }
}

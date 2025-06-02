use std::path::Path;
use hyperlit_base::err;
use hyperlit_base::result::HyperlitResult;
use hyperlit_base::shared_string::SharedString;

pub trait FileSource {
    fn filepath(&self) -> HyperlitResult<SharedString>;
    fn open(&self) -> HyperlitResult<Box<dyn std::io::Read>>;
}

impl <T: AsRef<Path>> FileSource for T {
    fn filepath(&self) -> HyperlitResult<SharedString> {
        Ok(self.as_ref().to_str().ok_or_else(|| err!("Invalid filepath"))?.to_string().into())
    }
    fn open(&self) -> HyperlitResult<Box<dyn std::io::Read>> {
        Ok(Box::new(std::fs::File::open(self)?))
    }
}

pub struct InMemoryFileSource {
    pub data: SharedString,
    pub filepath: SharedString,
}

impl InMemoryFileSource {
    pub fn new<P: Into<SharedString>, T: Into<SharedString>>(filepath: P, data: T) -> Self {
        Self { data: data.into(), filepath: filepath.into() }
    }
}

impl FileSource for InMemoryFileSource {
    fn filepath(&self) -> HyperlitResult<SharedString> {
        Ok(self.filepath.clone())
    }

    fn open(&self) -> HyperlitResult<Box<dyn std::io::Read>> {
        Ok(Box::new(std::io::Cursor::new(self.data.clone())))
    }
}


#[cfg(test)]
mod tests {
    use std::io::Read;
    use std::path::PathBuf;
    use crate::file_source::{FileSource, InMemoryFileSource};

    #[test]
    fn test_in_memory() {
        let source = InMemoryFileSource::new("foo", "bar");
        assert_eq!(source.filepath().unwrap(), "foo");
        let mut content = String::new();
        source.open().unwrap().read_to_string(&mut content).unwrap();
        assert_eq!(content, "bar");
    }

    #[test]
    fn test_file_path() {
        let source = PathBuf::from("test/file_source_sample.txt");
        let mut content = String::new();
        source.open().unwrap().read_to_string(&mut content).unwrap();
        assert_eq!(content, "asdf");
    }

}
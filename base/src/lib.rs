// #![allow(clippy::redundant_closure)] // Gives false positives for context! macro

use relative_path::RelativePathBuf;

pub mod error;
pub mod id;
pub mod id_generator;
pub mod logging;
pub mod result;
pub mod shared_string;

pub type FilePath = RelativePathBuf;

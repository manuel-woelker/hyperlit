/* 📖 # What is the Platform Abstraction Layer?

The PAL provides a trait-based abstraction over filesystem operations, enabling testable code.
Key benefits:
- Testability: MockPal allows deterministic unit tests without filesystem access
- Flexibility: Switch between real filesystem and in-memory implementations
- Consistency: All filesystem operations use the same error handling

This follows the Dependency Inversion Principle—code depends on abstractions (Pal trait),
not concrete implementations (RealPal or MockPal).
*/

mod file_path;
pub mod mock;
pub mod real_pal;
mod traits;

pub use file_path::FilePath;
pub use mock::MockPal;
pub use real_pal::RealPal;
pub use traits::{FileChangeCallback, FileChangeEvent, Pal, PalHandle, ReadSeek};

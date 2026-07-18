//! Performance architecture — lazy loading, caching, background indexing,
//! progress reporting, memory limits, and cancellation support.

pub mod cache;
pub mod background_indexer;
pub mod lazy_loader;
pub mod progress;
pub mod memory_limit;
pub mod cancellation;

pub use cache::RetrievalCache;
pub use background_indexer::BackgroundIndexer;
pub use lazy_loader::LazyPackLoader;
pub use progress::{ProgressReporter, ProgressEvent, ProgressReporterHandle};
pub use memory_limit::MemoryLimit;
pub use cancellation::CancellationToken;
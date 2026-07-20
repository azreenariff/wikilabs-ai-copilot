//! Performance architecture — lazy loading, caching, background indexing,
//! progress reporting, memory limits, and cancellation support.

pub mod background_indexer;
pub mod cache;
pub mod cancellation;
pub mod lazy_loader;
pub mod memory_limit;
pub mod progress;

pub use background_indexer::BackgroundIndexer;
pub use cache::RetrievalCache;
pub use cancellation::CancellationToken;
pub use lazy_loader::LazyPackLoader;
pub use memory_limit::MemoryLimit;
pub use progress::{ProgressEvent, ProgressReporter, ProgressReporterHandle};

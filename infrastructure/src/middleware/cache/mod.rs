//! Valkey read cache: clear and refill namespaces after successful writes.

pub mod cache_invalidation;

pub use cache_invalidation::{CacheInvalidation, InvalidationRule, invalidate_cache_on_write, namespaces_for};

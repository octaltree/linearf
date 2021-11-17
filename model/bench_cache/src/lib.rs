//! When you change queries in succession, including when you type normally,
//! performance suffers because they lock together.

pub mod cache;
pub mod cache2;
pub mod cache_chunks;
mod fuse;

// Cache module for SQL CLI

#[cfg(feature = "redis-cache")]
pub mod redis_cache;

#[cfg(feature = "redis-cache")]
pub use redis_cache::RedisCache;

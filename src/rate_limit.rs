use std::fs;

use axum::http::StatusCode;
use redis::{Client, RedisError};
use serde::Deserialize;

pub struct RateLimit {
    pub status_code: StatusCode,
    pub limit: u32,
    pub remaining: u32,
    pub time_to_reset: u32,
}

#[derive(Deserialize)]
pub struct RateLimitConfig {
    pub limit_type: LimitType,
    pub limit_by: LimitBy,
    pub limit: i32,
    pub window: i32,
}

#[derive(Deserialize)]
pub enum LimitType {
    Message,
    Auth,
}

#[derive(Deserialize)]
pub enum LimitBy {
    IP,
    User,
}

pub fn init_redis_connection(redis_host: String) -> Result<Client, RedisError> {
    let connection = redis::Client::open(format!("redis://{}", redis_host))?;

    Ok(connection)
}

pub fn get_rate_limiter_configuration() -> RateLimitConfig {
    let rate_limiter_config =
        fs::read_to_string("rate_limiter.yaml").expect("Failed to read rate_limiter.yaml file.");

    let rate_limiter_config: RateLimitConfig = serde_yaml::from_str(&rate_limiter_config)
        .expect("Failed to parse rate_limiter.yaml file.");

    rate_limiter_config
}

pub async fn rate_limit(limit_key: &str, client: Client) -> Result<RateLimit, RedisError> {
    let mut redis = client.get_async_connection().await.unwrap();

    redis::cmd("INCR")
        .arg(&limit_key)
        .query_async::<_, u32>(&mut redis)
        .await?;
    redis::cmd("EXPIRE")
        .arg(&limit_key)
        .arg(60)
        .query_async::<_, u32>(&mut redis)
        .await?;
    let count: u32 = redis::cmd("GET")
        .arg(&limit_key)
        .query_async(&mut redis)
        .await?;

    if count > 10 {
        let ttl = redis::cmd("TTL")
            .arg(&limit_key)
            .query_async::<_, u32>(&mut redis)
            .await?;

        return Ok(RateLimit {
            status_code: StatusCode::TOO_MANY_REQUESTS,
            limit: 10,
            remaining: 0,
            time_to_reset: ttl,
        });
    }

    Ok(RateLimit {
        status_code: StatusCode::OK,
        limit: 10,
        remaining: 10 - count,
        time_to_reset: 0,
    })
}

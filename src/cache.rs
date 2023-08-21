use axum::http::StatusCode;
use redis::{Client, RedisError};

pub struct RateLimit {
    pub status_code: StatusCode,
    pub limit: u32,
    pub remaining: u32,
    pub time_to_reset: u32,
}

pub fn init_redis_connection(redis_host: String) -> Result<Client, RedisError> {
    let connection = redis::Client::open(format!("redis://{}", redis_host))?;

    Ok(connection)
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

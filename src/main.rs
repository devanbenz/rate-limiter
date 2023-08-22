use std::{fmt::Debug, net::TcpListener, sync::Arc};

use axum::{
    body::Full,
    extract::State,
    http::{Request, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use rate_limiter::{
    configuration,
    rate_limit::{init_redis_connection, rate_limit},
};

pub struct AppState {
    redis_client: redis::Client,
}

async fn rate_limit_middleware<B>(
    State(state): State<Arc<AppState>>,
    request: Request<B>,
    next: Next<B>,
) -> Result<Response, StatusCode>
where
    B: Send + Sync + 'static,
    B: Debug,
{
    let key = format!(
        "{:?}",
        request
            .headers()
            .get("host")
            .expect("Failed to get host header.")
    );

    let rate_limiter = rate_limit(&key, state.redis_client.clone())
        .await
        .expect("Failed to rate limit.");

    if rate_limiter.status_code == StatusCode::TOO_MANY_REQUESTS {
        return Ok(Response::builder()
            .status(rate_limiter.status_code)
            .header("X-RateLimit-Limit", rate_limiter.limit)
            .header("X-RateLimit-Remaining", rate_limiter.remaining)
            .header("X-RateLimit-Reset", rate_limiter.time_to_reset)
            .body(Full::from("Rate limit exceeded."))
            .unwrap()
            .into_response());
    }

    let response = next.run(request).await;

    Ok(response)
}

#[tokio::main]
async fn main() {
    let configuration = configuration::get_config().expect("Failed to read configuration.");

    let redis_client =
        init_redis_connection(configuration.redis_host).expect("Failed to connect to Redis.");
    let app_state = Arc::new(AppState {
        redis_client: redis_client,
    });

    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .layer(middleware::from_fn_with_state(
            app_state.clone(),
            rate_limit_middleware,
        ))
        .with_state(app_state);

    let listener =
        TcpListener::bind(format!("0.0.0.0:{}", configuration.application_port)).unwrap();

    axum::Server::from_tcp(listener)
        .unwrap()
        .serve(app.into_make_service())
        .await
        .unwrap();
}

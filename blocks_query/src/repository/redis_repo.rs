


#[derive(Clone)]
pub struct RateLimitingService {
    redis_connection_manager: ConnectionManager,
}

impl RateLimitingService {
    pub fn new(redis_connection_manager: ConnectionManager) -> Self {
        RateLimitingService {
            redis_connection_manager,
        }
    }

    pub async fn assert_rate_limit_not_exceeded(&self, ip_addr: String) -> Result<(), CustomError> {
        let current_minute = Utc::now().minute();
        let rate_limit_key = format!("{}:{}:{}", RATE_LIMIT_KEY_PREFIX, ip_addr, current_minute);

        let (count,): (u64,) = redis::pipe()
            .atomic()
            .incr(&rate_limit_key, 1)
            .expire(&rate_limit_key, 60)
            .ignore()
            .query_async(&mut self.redis_connection_manager.clone())
            .await?;

        if count > MAX_REQUESTS_PER_MINUTE {
            Err(TooManyRequests {
                actual_count: count,
                permitted_count: MAX_REQUESTS_PER_MINUTE,
            })
        } else {
            Ok(())
        }
    }
}


pub struct RedisRepo {

}

impl RedisRepo {
    fn init() -> Self {
        let redis_uri = env::var("REDIS_URI").expect("REDIS_URI env var should be specified");
        let redis_client = Client::open(redis_uri)
        let redis_connection_manager = redis_client
        .get_tokio_connection_manager()
        .await
        .expect("Can't create Redis connection manager");
        let rate_limiting_service = Arc::new(RateLimitingService::new(redis_connection_manager));

    }
}

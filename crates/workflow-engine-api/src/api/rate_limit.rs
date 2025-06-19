use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    error::ErrorTooManyRequests,
    Error, HttpMessage,
};
use futures_util::future::LocalBoxFuture;
use std::{
    collections::HashMap,
    future::{ready, Ready},
    rc::Rc,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

#[derive(Clone)]
pub struct RateLimitConfig {
    pub requests_per_minute: u32,
    pub burst_size: u32,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            requests_per_minute: 60,
            burst_size: 10,
        }
    }
}

struct TokenBucket {
    tokens: f64,
    last_update: Instant,
    rate: f64,
    capacity: f64,
}

impl TokenBucket {
    fn new(rate: f64, capacity: f64) -> Self {
        Self {
            tokens: capacity,
            last_update: Instant::now(),
            rate,
            capacity,
        }
    }

    fn consume(&mut self, tokens: f64) -> bool {
        self.refill();
        if self.tokens >= tokens {
            self.tokens -= tokens;
            true
        } else {
            false
        }
    }

    fn refill(&mut self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_update).as_secs_f64();
        self.tokens = (self.tokens + elapsed * self.rate).min(self.capacity);
        self.last_update = now;
    }
}

pub struct RateLimiter {
    buckets: Arc<Mutex<HashMap<String, TokenBucket>>>,
    config: RateLimitConfig,
}

impl RateLimiter {
    pub fn new(config: RateLimitConfig) -> Self {
        Self {
            buckets: Arc::new(Mutex::new(HashMap::new())),
            config,
        }
    }

    fn check_rate_limit(&self, key: &str) -> bool {
        let mut buckets = self.buckets.lock().unwrap();
        let rate = self.config.requests_per_minute as f64 / 60.0;
        let capacity = self.config.burst_size as f64;

        let bucket = buckets
            .entry(key.to_string())
            .or_insert_with(|| TokenBucket::new(rate, capacity));

        bucket.consume(1.0)
    }

    pub fn cleanup_old_buckets(&self) {
        let mut buckets = self.buckets.lock().unwrap();
        let now = Instant::now();
        buckets.retain(|_, bucket| {
            now.duration_since(bucket.last_update) < Duration::from_secs(300)
        });
    }
}

// Middleware factory
pub struct RateLimitMiddleware {
    limiter: Arc<RateLimiter>,
}

impl RateLimitMiddleware {
    pub fn new(config: RateLimitConfig) -> Self {
        let limiter = Arc::new(RateLimiter::new(config));

        // Spawn a cleanup task
        let limiter_clone = limiter.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(60));
            loop {
                interval.tick().await;
                limiter_clone.cleanup_old_buckets();
            }
        });

        Self { limiter }
    }
}

impl<S, B> Transform<S, ServiceRequest> for RateLimitMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = RateLimitMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(RateLimitMiddlewareService {
            service: Rc::new(service),
            limiter: self.limiter.clone(),
        }))
    }
}

pub struct RateLimitMiddlewareService<S> {
    service: Rc<S>,
    limiter: Arc<RateLimiter>,
}

impl<S, B> Service<ServiceRequest> for RateLimitMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let service = self.service.clone();
        let limiter = self.limiter.clone();

        Box::pin(async move {
            // Skip rate limiting for health check
            if req.path() == "/api/v1/health" {
                return service.call(req).await;
            }

            // Get client identifier (IP address or user ID from JWT)
            let client_id = if let Some(claims) = req.extensions().get::<crate::api::auth::Claims>() {
                claims.sub.clone()
            } else {
                req.connection_info()
                    .peer_addr()
                    .unwrap_or("unknown")
                    .to_string()
            };

            // Check rate limit
            if limiter.check_rate_limit(&client_id) {
                service.call(req).await
            } else {
                Err(ErrorTooManyRequests("Rate limit exceeded"))
            }
        })
    }
}
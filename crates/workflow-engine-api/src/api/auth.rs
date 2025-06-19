use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    error::ErrorUnauthorized,
    Error, HttpMessage,
};
use futures_util::future::LocalBoxFuture;
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::{
    future::{ready, Ready},
    rc::Rc,
    time::{SystemTime, UNIX_EPOCH},
};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
    pub iat: usize,
}

pub struct JwtAuth {
    secret: String,
}

impl JwtAuth {
    pub fn new(secret: String) -> Self {
        Self { secret }
    }

    pub fn generate_token(&self, user_id: &str) -> Result<String, jsonwebtoken::errors::Error> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as usize;

        let claims = Claims {
            sub: user_id.to_owned(),
            exp: now + 86400, // 24 hours
            iat: now,
        };

        encode(
            &Header::new(Algorithm::HS256),
            &claims,
            &EncodingKey::from_secret(self.secret.as_ref()),
        )
    }

    pub fn validate_token(&self, token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
        decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.secret.as_ref()),
            &Validation::new(Algorithm::HS256),
        )
        .map(|data| data.claims)
    }
}

// Middleware factory
pub struct JwtMiddleware {
    auth: Rc<JwtAuth>,
}

impl JwtMiddleware {
    pub fn new(secret: String) -> Self {
        Self {
            auth: Rc::new(JwtAuth::new(secret)),
        }
    }
}

impl<S, B> Transform<S, ServiceRequest> for JwtMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = JwtMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(JwtMiddlewareService {
            service: Rc::new(service),
            auth: self.auth.clone(),
        }))
    }
}

pub struct JwtMiddlewareService<S> {
    service: Rc<S>,
    auth: Rc<JwtAuth>,
}

impl<S, B> Service<ServiceRequest> for JwtMiddlewareService<S>
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
        let auth = self.auth.clone();

        Box::pin(async move {
            // Skip auth for health check and auth endpoints
            let path = req.path();
            if path == "/api/v1/health" || path == "/api/v1/auth/login" {
                return service.call(req).await;
            }

            // Extract token from Authorization header
            let token = match req.headers().get("Authorization") {
                Some(header) => match header.to_str() {
                    Ok(value) => {
                        if value.starts_with("Bearer ") {
                            &value[7..]
                        } else {
                            return Err(ErrorUnauthorized("Invalid authorization header"));
                        }
                    }
                    Err(_) => return Err(ErrorUnauthorized("Invalid authorization header")),
                },
                None => return Err(ErrorUnauthorized("Missing authorization header")),
            };

            // Validate token
            match auth.validate_token(token) {
                Ok(claims) => {
                    req.extensions_mut().insert(claims);
                    service.call(req).await
                }
                Err(_) => Err(ErrorUnauthorized("Invalid token")),
            }
        })
    }
}
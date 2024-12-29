use crate::auth::jwt::Claims;
use crate::config::Config;
use actix_web::{Error, HttpMessage, dev::ServiceRequest};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use jsonwebtoken::{DecodingKey, Validation, decode};

pub async fn jwt_middleware(
    req: ServiceRequest,
    auth: BearerAuth,
) -> Result<ServiceRequest, (Error, ServiceRequest)> {
    let token = auth.token();
    let config = Config::from_env();

    let token_data = match decode::<Claims>(
        token,
        &DecodingKey::from_secret(config.jwt_secret.as_bytes()),
        &Validation::default(),
    ) {
        Ok(data) => data,
        Err(_) => return Err((actix_web::error::ErrorUnauthorized("Invalid token"), req)),
    };

    // Attach user info to the request extensions for further use
    req.extensions_mut().insert(token_data.claims.sub);

    Ok(req)
}

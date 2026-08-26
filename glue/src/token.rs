use crate::errors::{NanoServiceError, NanoServiceErrorStatus};
use dotenv::dotenv;
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct HeaderToken {
    pub unique_id: String,
}

impl HeaderToken {
    pub fn get_key() -> Result<String, NanoServiceError> {
        dotenv().ok();
        std::env::var("JWT_SECRET")
            .map_err(|e| NanoServiceError::new(e.to_string(), NanoServiceErrorStatus::Unauthorized))
    }
    
    pub fn encode(self) -> Result<String, NanoServiceError> {
        let key_str = Self::get_key()?;
        let key = EncodingKey::from_secret(key_str.as_ref());
        return match encode(&Header::default(), &self, &key) {
            Ok(token) => Ok(token),
            Err(error) => Err(NanoServiceError::new(
                error.to_string(),
                NanoServiceErrorStatus::Unauthorized,
            )),
        };
    }
    
    pub fn decode(token: &str) -> Result<Self, NanoServiceError> {
        let key_str = Self::get_key()?;
        let key = DecodingKey::from_secret(key_str.as_ref());
        let mut validation = Validation::new(Algorithm::HS256);
        validation.required_spec_claims.remove("exp");
        match decode::<Self>(token, &key, &validation) {
            Ok(token_data) => return Ok(token_data.claims),
            Err(error) => {
                return Err(NanoServiceError::new(
                    error.to_string(),
                    NanoServiceErrorStatus::Unauthorized,
                ));
            }
        };
    }
}

#[cfg(feature = "actix")]
mod actix_impl {
    use super::HeaderToken;
    use crate::errors::{NanoServiceError, NanoServiceErrorStatus};
    pub use actix_web::{FromRequest as ActixFromRequest, HttpRequest, dev::Payload};
    use futures::future::{Ready, err, ok};

    impl ActixFromRequest for HeaderToken {
        type Error = NanoServiceError;
        // The Ready struct wraps the Result and implements the Future trait
        type Future = Ready<Result<HeaderToken, NanoServiceError>>;
        // from_request function returns a Future that is going to be by tokio
        fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
            let raw_data = match req.headers().get("token") {
                Some(data) => data.to_str().expect("convert token to str"),
                None => {
                    // Lowercase err and lowercase ok just wrap the error or result in a Ready struct
                    return err(NanoServiceError {
                        status: NanoServiceErrorStatus::Unauthorized,
                        message: "token not found in request header".to_string(),
                    });
                }
            };
            let token = match HeaderToken::decode(raw_data) {
                Ok(token) => token,
                Err(_) => {
                    return err(NanoServiceError {
                        status: NanoServiceErrorStatus::Unauthorized,
                        message: "token not a valid string".to_string(),
                    });
                }
            };
            ok(token) // lowercase ok just wraps the result in a Ready struct
        }
    }
}

// #[cfg(feature = "actix")]
// pub use actix_impl::ActixFromRequest;
// impl<T> Future for Ready<T> {
//     type Output = T;
//     #[inline]
//     fn poll(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<T> {
//         Poll::Ready(self.0.take().expect("Ready polled after completion"))
//     }
// }

pub struct HeaderToken {
    pub message: String,
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
                Some(data) => data,
                None => {
                    // Lowercase err and lowercase ok just wrap the error or result in a Ready struct
                    return err(NanoServiceError {
                        status: NanoServiceErrorStatus::Unauthorized,
                        message: "token not found in request header".to_string(),
                    });
                }
            };
            let message = match raw_data.to_str() {
                Ok(token) => token.to_string(),
                Err(_) => {
                    return err(NanoServiceError {
                        status: NanoServiceErrorStatus::Unauthorized,
                        message: "token not a valid string".to_string(),
                    });
                }
            };
            ok(HeaderToken { message }) // lowercase ok just wraps the result in a Ready struct
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

use actix_web::HttpResponse;
use actix_web::ResponseError;
use actix_web::http::StatusCode;
use loams::ErrorKind;
use platform::PlatformError;
use serde_json::json;

#[derive(Debug)]
pub enum ApiError {
    NotFound,
    Invalid(String),
    Platform(PlatformError),
}

impl From<PlatformError> for ApiError {
    fn from(error: PlatformError) -> Self {
        match error.kind() {
            Some(ErrorKind::NotFound) => Self::NotFound,
            Some(ErrorKind::InvalidArgument) => Self::Invalid(error.to_string()),
            _ => Self::Platform(error),
        }
    }
}

impl From<domain::DomainError> for ApiError {
    fn from(error: domain::DomainError) -> Self {
        Self::Invalid(error.to_string())
    }
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound => f.write_str("not found"),
            Self::Invalid(message) => f.write_str(message),
            Self::Platform(error) => error.fmt(f),
        }
    }
}

impl std::error::Error for ApiError {}

impl ResponseError for ApiError {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::NotFound => StatusCode::NOT_FOUND,
            Self::Invalid(_) => StatusCode::BAD_REQUEST,
            Self::Platform(error) => match error.kind() {
                Some(ErrorKind::AlreadyExists | ErrorKind::Aborted) => StatusCode::CONFLICT,
                Some(ErrorKind::PermissionDenied) => StatusCode::FORBIDDEN,
                Some(ErrorKind::Unauthenticated | ErrorKind::TokenExpired) => {
                    StatusCode::UNAUTHORIZED
                }
                Some(ErrorKind::FailedPrecondition) => StatusCode::PRECONDITION_FAILED,
                Some(ErrorKind::ResourceExhausted) => StatusCode::TOO_MANY_REQUESTS,
                Some(ErrorKind::Unavailable) => StatusCode::SERVICE_UNAVAILABLE,
                Some(ErrorKind::DeadlineExceeded) => StatusCode::GATEWAY_TIMEOUT,
                Some(ErrorKind::Unimplemented | ErrorKind::FeatureNotInVariant) => {
                    StatusCode::NOT_IMPLEMENTED
                }
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            },
        }
    }

    fn error_response(&self) -> HttpResponse {
        let status = self.status_code();
        let (code, message) = match self {
            Self::NotFound => ("not_found", "not found".to_owned()),
            Self::Invalid(message) => ("invalid_argument", message.clone()),
            Self::Platform(error) if status.is_server_error() => {
                log::error!("loams call failed: {error:?}");
                ("internal", "internal error".to_owned())
            }
            Self::Platform(error) => ("upstream", error.to_string()),
        };
        HttpResponse::build(status).json(json!({"error": {"code": code, "message": message}}))
    }
}

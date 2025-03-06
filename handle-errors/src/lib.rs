use argon2::Error as ArgonError;
use reqwest::Error as ReqwestError;
use reqwest_middleware::Error as MiddlewareReqwestError;
use tracing::{event, instrument, Level};
use warp::{
    filters::{body::BodyDeserializeError, cors::CorsForbidden},
    http::StatusCode,
    reject::Reject,
    Rejection, Reply,
};

#[derive(Debug)]
pub enum Error {
    ParseError(std::num::ParseIntError),
    MissingParameters,
    WrongPassword,
    CannotDecryptToken,
    Unauthorized,
    ArgonLibraryError(ArgonError),
    DatabaseQueryError(sqlx::Error),
    MigrationError(sqlx::migrate::MigrateError),
    ReqwestAPIError(ReqwestError),
    MiddlewareReqwestAPIError(MiddlewareReqwestError),
    ClientError(APILayerError),
    ServerError(APILayerError),
    EnvironmentError(std::env::VarError),
}

#[derive(Debug, Clone)]
pub struct APILayerError {
    pub status: u16,
    pub message: String,
}

impl std::fmt::Display for APILayerError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Status: {}, Message: {}", self.status, self.message)
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::ParseError(ref err) => {
                write!(f, "Cannot parse parameter: {}", err)
            }
            Error::MissingParameters => write!(f, "Missing parameter"),
            Error::WrongPassword => write!(f, "Wrong password"),
            Error::CannotDecryptToken => write!(f, "Cannot decrypt error"),
            Error::Unauthorized => write!(f, "No permission to change the underlying resource"),
            Error::ArgonLibraryError(_) => {
                write!(f, "Cannot verifiy password")
            }
            Error::DatabaseQueryError(_) => {
                write!(f, "Cannot update, invalid data")
            }
            Error::MigrationError(_) => write!(f, "Cannot migrate data"),
            Error::ReqwestAPIError(err) => {
                write!(f, "External API error: {}", err)
            }
            Error::MiddlewareReqwestAPIError(err) => {
                write!(f, "External API error: {}", err)
            }
            Error::ClientError(err) => {
                write!(f, "External Client error: {}", err)
            }
            Error::ServerError(err) => {
                write!(f, "External Server error: {}", err)
            }
            Error::EnvironmentError(err) => {
                write!(f, "Environment variable error: {}", err)
            }
        }
    }
}

const DUPLICATE_KEY: u32 = 23505;

impl Reject for Error {}
impl Reject for APILayerError {}

#[instrument]
pub async fn return_error(r: Rejection) -> Result<impl Reply, Rejection> {
    if let Some(crate::Error::DatabaseQueryError(e)) = r.find() {
        event!(Level::ERROR, "Database query error");

        match e {
            sqlx::Error::Database(err) => {
                if err.code().unwrap().parse::<u32>().unwrap() == DUPLICATE_KEY {
                    Ok(warp::reply::with_status(
                        "Account already exsists".to_string(),
                        StatusCode::UNPROCESSABLE_ENTITY,
                    ))
                } else {
                    Ok(warp::reply::with_status(
                        "Cannot update data".to_string(),
                        StatusCode::UNPROCESSABLE_ENTITY,
                    ))
                }
            }
            _ => Ok(warp::reply::with_status(
                "Cannot update data".to_string(),
                StatusCode::UNPROCESSABLE_ENTITY,
            )),
        }
    } else if let Some(crate::Error::ReqwestAPIError(e)) = r.find() {
        event!(Level::ERROR, "{}", e);
        Ok(warp::reply::with_status(
            "Internal Server Error".to_string(),
            StatusCode::INTERNAL_SERVER_ERROR,
        ))
    } else if let Some(crate::Error::Unauthorized) = r.find() {
        event!(Level::ERROR, "Not matching account id");
        Ok(warp::reply::with_status(
            "No permission to change underlying resource".to_string(),
            StatusCode::UNAUTHORIZED,
        ))
    } else if let Some(crate::Error::WrongPassword) = r.find() {
        event!(Level::ERROR, "Entered wrong password");
        Ok(warp::reply::with_status(
            "Wrong E-Mail/Password combination".to_string(),
            StatusCode::UNAUTHORIZED,
        ))
    } else if let Some(crate::Error::MiddlewareReqwestAPIError(e)) = r.find() {
        event!(Level::ERROR, "{}", e);
        Ok(warp::reply::with_status(
            "Internal Server Error".to_string(),
            StatusCode::INTERNAL_SERVER_ERROR,
        ))
    } else if let Some(crate::Error::ClientError(e)) = r.find() {
        event!(Level::ERROR, "{}", e);
        Ok(warp::reply::with_status(
            "Internal Server Error".to_string(),
            StatusCode::INTERNAL_SERVER_ERROR,
        ))
    } else if let Some(crate::Error::ServerError(e)) = r.find() {
        event!(Level::ERROR, "{}", e);
        Ok(warp::reply::with_status(
            "Internal Server Error".to_string(),
            StatusCode::INTERNAL_SERVER_ERROR,
        ))
    } else if let Some(error) = r.find::<CorsForbidden>() {
        event!(Level::ERROR, "CORS forbidden error: {}", error);
        Ok(warp::reply::with_status(
            error.to_string(),
            StatusCode::FORBIDDEN,
        ))
    } else if let Some(error) = r.find::<BodyDeserializeError>() {
        event!(Level::ERROR, "Cannot deserizalize request body: {}", error);
        Ok(warp::reply::with_status(
            error.to_string(),
            StatusCode::UNPROCESSABLE_ENTITY,
        ))
    } else if let Some(error) = r.find::<Error>() {
        event!(Level::ERROR, "{}", error);
        Ok(warp::reply::with_status(
            error.to_string(),
            StatusCode::UNPROCESSABLE_ENTITY,
        ))
    } else {
        event!(Level::WARN, "Requested route was not found");
        Ok(warp::reply::with_status(
            "Route not found".to_string(),
            StatusCode::NOT_FOUND,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::num::ParseIntError;
    use warp::reject;

    #[tokio::test]
    async fn test_return_error_database_query_error() {
        let error = Error::DatabaseQueryError(sqlx::Error::RowNotFound);
        let rejection = reject::custom(error);
        let result = return_error(rejection).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_return_error_reqwest_api_error() {
        let error = Error::ReqwestAPIError(reqwest::Error::from(std::io::Error::new(
            std::io::ErrorKind::Other,
            "test error",
        )));
        let rejection = reject::custom(error);
        let result = return_error(rejection).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_return_error_unauthorized() {
        let error = Error::Unauthorized;
        let rejection = reject::custom(error);
        let result = return_error(rejection).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_return_error_wrong_password() {
        let error = Error::WrongPassword;
        let rejection = reject::custom(error);
        let result = return_error(rejection).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_return_error_middleware_reqwest_error() {
        let error = Error::MiddlewareReqwestAPIError(reqwest_middleware::Error::Middleware(Box::new(
            std::io::Error::new(std::io::ErrorKind::Other, "test error"),
        )));
        let rejection = reject::custom(error);
        let result = return_error(rejection).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_return_error_client_error() {
        let error = Error::ClientError(APILayerError {
            status: 400,
            message: "Bad Request".to_string(),
        });
        let rejection = reject::custom(error);
        let result = return_error(rejection).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_return_error_server_error() {
        let error = Error::ServerError(APILayerError {
            status: 500,
            message: "Internal Server Error".to_string(),
        });
        let rejection = reject::custom(error);
        let result = return_error(rejection).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_return_error_cors_forbidden() {
        let error = warp::reject::custom(warp::cors::CorsForbidden);
        let result = return_error(error).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_return_error_body_deserialize() {
        let error = warp::reject::custom(warp::filters::body::BodyDeserializeError::Json(None));
        let result = return_error(error).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_return_error_parse_error() {
        let parse_error = "abc".parse::<i32>().unwrap_err();
        let error = Error::ParseError(parse_error);
        let rejection = reject::custom(error);
        let result = return_error(rejection).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_return_error_missing_parameters() {
        let error = Error::MissingParameters;
        let rejection = reject::custom(error);
        let result = return_error(rejection).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_return_error_cannot_decrypt_token() {
        let error = Error::CannotDecryptToken;
        let rejection = reject::custom(error);
        let result = return_error(rejection).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_return_error_argon_library_error() {
        let error = Error::ArgonLibraryError(argon2::Error::AdParam);
        let rejection = reject::custom(error);
        let result = return_error(rejection).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_return_error_migration_error() {
        let error = Error::MigrationError(sqlx::migrate::MigrateError::VersionMissing(1));
        let rejection = reject::custom(error);
        let result = return_error(rejection).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_return_error_environment_error() {
        let error = Error::EnvironmentError(std::env::VarError::NotPresent);
        let rejection = reject::custom(error);
        let result = return_error(rejection).await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_error_display() {
        let parse_error = "abc".parse::<i32>().unwrap_err();
        let error = Error::ParseError(parse_error);
        assert!(!error.to_string().is_empty());

        let error = Error::MissingParameters;
        assert_eq!(error.to_string(), "Missing parameter");

        let error = Error::WrongPassword;
        assert_eq!(error.to_string(), "Wrong password");

        let error = Error::CannotDecryptToken;
        assert_eq!(error.to_string(), "Cannot decrypt error");

        let error = Error::Unauthorized;
        assert_eq!(error.to_string(), "No permission to change the underlying resource");
    }

    #[test]
    fn test_api_layer_error_display() {
        let error = APILayerError {
            status: 400,
            message: "Bad Request".to_string(),
        };
        assert_eq!(error.to_string(), "Status: 400, Message: Bad Request");
    }
}

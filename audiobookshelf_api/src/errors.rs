use std::fmt::Display;

use reqwest::StatusCode;
use thiserror::Error;

#[derive(Error, Debug)]
pub struct ResponseError {
    pub status: StatusCode,
    pub response: String,
}

impl Display for ResponseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Response failed. Status={}, response={}",
            self.status, self.response
        )
    }
}

#[derive(Error, Debug)]
pub enum APIError {
    #[error("Connection failed")]
    NetworkError(reqwest::Error),
    #[error("Connection failed")]
    UnknownError(Box<dyn std::error::Error>),
    #[error("Invalid Response")]
    InvalidResponseSchema(serde_path_to_error::Error<serde_json::Error>),
    #[error("Invalid Request")]
    InvalidRequestSchema(serde_json::Error),
}

#[derive(Error, Debug)]
pub enum AuthError {
    #[error("Invalid credentials")]
    InvalidCredentials,
}

#[derive(Error, Debug)]
pub enum FusedError<T> {
    #[error("API Error")]
    APIError(#[from] APIError),
    #[error("Domain Error")]
    DomainError(T),
}

impl<T> FusedError<T> {
    pub fn make<A>(result: Result<Result<A, T>, APIError>) -> Result<A, FusedError<T>> {
        match result {
            Ok(Ok(value)) => Ok(value),
            Ok(Err(error)) => Err(FusedError::DomainError(error)),
            Err(error) => Err(FusedError::APIError(error)),
        }
    }

    pub fn unmake<A>(result: Result<A, FusedError<T>>) -> Result<Result<A, T>, APIError> {
        match result {
            Ok(value) => Ok(Ok(value)),
            Err(FusedError::DomainError(error)) => Ok(Err(error)),
            Err(FusedError::APIError(error)) => Err(error),
        }
    }

    pub fn map_domain_error<A>(self, func: impl Fn(T) -> A) -> FusedError<A> {
        match self {
            FusedError::DomainError(err) => FusedError::DomainError(func(err)),
            FusedError::APIError(err) => FusedError::APIError(err),
        }
    }
}

impl FusedError<ResponseError> {
    pub fn to_api_error(self) -> APIError {
        match self {
            FusedError::APIError(error) => error,
            FusedError::DomainError(error) => APIError::UnknownError(error.into()),
        }
    }
}

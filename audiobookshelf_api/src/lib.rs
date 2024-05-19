pub mod errors;
pub mod schema;

use errors::{APIError, AuthError, FusedError, ResponseError};
use reqwest::{self, StatusCode, Url};
use schema::{AuthRequest, AuthResponse, Libraries, Library, LibraryWithFilters};

pub struct ClientConfig {
    pub root_url: Url,
}

pub struct UserClient {
    client: reqwest::Client,
    token: String,
    config: ClientConfig,
}

impl ClientConfig {
    fn login_url(&self) -> Url {
        self.root_url.join("login").unwrap()
    }

    fn libraries_url(&self) -> Url {
        self.root_url.join("api/libraries").unwrap()
    }

    fn library_url(&self, id: &str) -> Url {
        self.root_url
            .join("api/libraries/")
            .unwrap()
            .join(id)
            .unwrap()
    }
}

impl UserClient {
    pub fn from_token(config: ClientConfig, token: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            config,
            token,
        }
    }

    pub async fn auth(
        config: ClientConfig,
        username: String,
        password: String,
    ) -> Result<Self, FusedError<AuthError>> {
        let client = reqwest::Client::new();
        let url = config.login_url();

        let body = serde_json::to_string(&AuthRequest { username, password }).unwrap();
        let response: AuthResponse = Self::send(
            client
                .post(url)
                .header("Content-Type", "application/json")
                .body(body),
        )
        .await
        .map_err(|error| match error {
            FusedError::APIError(error) => FusedError::APIError(error),
            FusedError::DomainError(error) if error.status == StatusCode::UNAUTHORIZED => {
                FusedError::DomainError(AuthError::InvalidCredentials)
            }
            _ => FusedError::APIError(error.to_api_error()),
        })?;

        Ok(Self {
            client: reqwest::Client::new(),
            config,
            token: response.user.token,
        })
    }

    pub async fn libraries(&self) -> Result<Vec<Library>, APIError> {
        let request_builder = self
            .client
            .get(self.config.libraries_url())
            .bearer_auth(self.token.clone())
            .header("Content-Type", "application/json");

        let result: Libraries = Self::send(request_builder)
            .await
            .map_err(FusedError::to_api_error)?;

        Ok(result.libraries)
    }

    pub async fn library(&self, id: &str) -> Result<LibraryWithFilters, APIError> {
        let request_builder = self
            .client
            .get(self.config.library_url(id))
            .query(&[("include", "filterdata")])
            .bearer_auth(self.token.clone())
            .header("Content-Type", "application/json");

        Self::send::<LibraryWithFilters>(request_builder)
            .await
            .map_err(FusedError::to_api_error)
    }
    async fn send<ResponseSchema>(
        request_builder: reqwest::RequestBuilder,
    ) -> Result<ResponseSchema, FusedError<ResponseError>>
    where
        ResponseSchema: for<'a> serde::Deserialize<'a>,
    {
        let response = request_builder
            .send()
            .await
            .map_err(APIError::NetworkError)?;

        let status = response.status();
        if response.status().is_success() {
            let body = response.text().await.map_err(APIError::NetworkError)?;
            let result = serde_json::from_str(&body).map_err(APIError::InvalidResponseSchema)?;
            Ok(result)
        } else {
            Err(FusedError::DomainError(ResponseError {
                status,
                response: response
                    .text()
                    .await
                    .map_err(|e| APIError::NetworkError(e))?,
            }))
        }
    }
}

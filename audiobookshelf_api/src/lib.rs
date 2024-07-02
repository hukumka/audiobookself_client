pub mod errors;
pub mod params;
pub mod schema;

use errors::{APIError, AuthError, FusedError, ResponseError};
use params::{LibraryItemParams, PlayLibraryItemParams};
use reqwest::{self, StatusCode, Url};
use schema::{
    AuthRequest, AuthResponse, Id, Libraries, Library, LibraryItem, LibraryItemMinified,
    LibraryWithFilters, PaginatedResponse, PlaybackSessionExtended, UserData,
};

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

    fn me_url(&self) -> Url {
        self.root_url.join("api/me").unwrap()
    }

    fn libraries_url(&self) -> Url {
        self.root_url.join("api/libraries").unwrap()
    }

    fn library_url(&self, id: &str) -> Url {
        Url::parse(&format!("{root}/api/libraries/{id}", root = self.root_url)).unwrap()
    }

    fn library_items_url(&self, id: &str) -> Url {
        Url::parse(&format!(
            "{root}/api/libraries/{id}/items",
            root = self.root_url
        ))
        .unwrap()
    }

    fn library_item_url(&self, id: &str) -> Url {
        Url::parse(&format!("{root}/api/items/{id}", root = self.root_url)).unwrap()
    }

    fn library_item_play_url(&self, id: &str) -> Url {
        Url::parse(&format!("{root}/api/items/{id}/play", root = self.root_url)).unwrap()
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

    pub async fn me(&self) -> Result<UserData, APIError> {
        let request_builder = self
            .client
            .get(self.config.me_url())
            .bearer_auth(self.token.clone())
            .header("Content-Type", "application/json");

        let response = Self::send(request_builder)
            .await
            .map_err(FusedError::to_api_error)?;

        Ok(response)
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

    pub async fn library(&self, id: &Id<Library>) -> Result<LibraryWithFilters, APIError> {
        let request_builder = self
            .client
            .get(self.config.library_url(id.as_str()))
            .query(&[("include", "filterdata")])
            .bearer_auth(self.token.clone())
            .header("Content-Type", "application/json");

        Self::send::<LibraryWithFilters>(request_builder)
            .await
            .map_err(FusedError::to_api_error)
    }

    pub async fn library_items(
        &self,
        id: &Id<Library>,
        params: LibraryItemParams,
    ) -> Result<Vec<LibraryItemMinified>, APIError> {
        let request_builder = self
            .client
            .get(self.config.library_items_url(id.as_str()))
            .query(&params.build_query())
            .bearer_auth(self.token.clone())
            .header("Content-Type", "application/json");

        let result = Self::send::<PaginatedResponse<LibraryItemMinified>>(request_builder)
            .await
            .map_err(FusedError::to_api_error)?;
        Ok(result.results)
    }

    pub async fn library_item(&self, id: &Id<LibraryItem>) -> Result<LibraryItem, APIError> {
        let request_builder = self
            .client
            .get(self.config.library_item_url(id.as_str()))
            .query(&[("include", "authors")])
            .bearer_auth(self.token.clone())
            .header("Content-Type", "application/json");

        Self::send::<LibraryItem>(request_builder)
            .await
            .map_err(FusedError::to_api_error)
    }

    /// Receive data neccesary to play media item.
    ///
    /// Note: despite name `play` suggesting that it is statefull, it does not update user media progress. That sould be done manually by using `library_item/`
    pub async fn library_item_play(
        &self,
        id: &Id<LibraryItem>,
        params: &PlayLibraryItemParams,
    ) -> Result<PlaybackSessionExtended, APIError> {
        let body = serde_json::to_string(params).unwrap();
        let request_builder = self
            .client
            .post(self.config.library_item_play_url(id.as_str()))
            .query(&[("include", "authors")])
            .bearer_auth(self.token.clone())
            .body(body)
            .header("Content-Type", "application/json");

        Self::send::<PlaybackSessionExtended>(request_builder)
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
            let json_deserializer = &mut serde_json::Deserializer::from_str(&body);
            let result = serde_path_to_error::deserialize(json_deserializer);
            match result {
                Ok(result) => Ok(result),
                Err(err) => Err(FusedError::APIError(APIError::InvalidResponseSchema(err))),
            }
        } else {
            Err(FusedError::DomainError(ResponseError {
                status,
                response: response.text().await.map_err(APIError::NetworkError)?,
            }))
        }
    }
}

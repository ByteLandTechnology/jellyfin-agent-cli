//! Jellyfin API HTTP client.
//!
//! Provides a type-safe client for making requests to the Jellyfin API.

#[cfg(not(windows))]
use std::sync::Once;

use reqwest::{Client, ClientBuilder};
use serde::{Serialize, de::DeserializeOwned};
use tracing::debug;

use jellyfin_core::{Config, EnvConfig, JellyfinError, Result};

use crate::types::*;

#[cfg(windows)]
fn ensure_rustls_provider() {}

#[cfg(not(windows))]
fn ensure_rustls_provider() {
    static INIT: Once = Once::new();

    INIT.call_once(|| {
        let _ = rustls::crypto::ring::default_provider().install_default();
    });
}

/// Jellyfin API client
#[derive(Debug, Clone)]
pub struct JellyfinClient {
    /// HTTP client
    client: Client,
    /// Server URL
    server_url: String,
    /// Access token
    token: Option<String>,
    /// Authenticated user ID
    user_id: Option<String>,
    /// Client name for identification
    client_name: String,
    /// Device name
    device_name: String,
    /// Device ID
    device_id: String,
    /// API version
    api_version: String,
}

impl JellyfinClient {
    /// Create a new client with default settings
    pub fn new(server_url: String) -> Result<Self> {
        ensure_rustls_provider();

        let client = ClientBuilder::new()
            .build()
            .map_err(|e| JellyfinError::internal(format!("Failed to create HTTP client: {}", e)))?;

        Ok(Self {
            client,
            server_url: Self::normalize_url(server_url)?,
            token: None,
            user_id: None,
            client_name: "jellyfin-cli".to_string(),
            device_name: Self::get_device_name(),
            device_id: Self::get_device_id(),
            api_version: "v1".to_string(),
        })
    }

    /// Create a client from config
    pub async fn from_config(server_name: Option<&str>) -> Result<Self> {
        let config = Config::load()?;
        let env = EnvConfig::load();

        // Determine which server to use
        let server_name = match (server_name, env.server.as_deref()) {
            (Some(name), _) => name.to_string(),
            (None, Some(env_server)) => {
                // Try to match env server URL to a configured server
                let server_url = Self::normalize_url(env_server.to_string())?;
                if let Some((name, _)) = config.servers.iter().find(|(_, s)| {
                    Self::normalize_url(s.url.clone()).as_ref().ok() == Some(&server_url)
                }) {
                    name.clone()
                } else {
                    // Use env server URL directly
                    return Self::new(env_server.to_string());
                }
            }
            (None, None) => config.default_server.clone(),
        };

        // Get server config
        let server_config = config
            .get_server(&server_name)
            .ok_or_else(|| JellyfinError::required_field(format!("server '{}'", server_name)))?;

        let mut client = Self::new(server_config.url.clone())?;

        // Set token from config or env
        let token = env.token.or_else(|| {
            server_config.token.clone().or_else(|| {
                // Try to load from credentials file
                Config::load_credentials()
                    .ok()
                    .and_then(|creds| creds.tokens.get(&server_name).cloned())
            })
        });

        if let Some(token) = token {
            client.set_token(token);
            // Try to fetch and cache user ID
            if let Ok(user) = client.get_current_user().await {
                client.user_id = Some(user.id);
            }
        }

        Ok(client)
    }

    /// Set the authentication token
    pub fn set_token(&mut self, token: String) {
        self.token = Some(token);
    }

    /// Get the current token
    pub fn token(&self) -> Option<&str> {
        self.token.as_deref()
    }

    /// Check if authenticated
    pub fn is_authenticated(&self) -> bool {
        self.token.is_some()
    }

    /// Get the server URL
    pub fn server_url(&self) -> &str {
        &self.server_url
    }

    /// Get the user ID
    pub fn user_id(&self) -> Option<&str> {
        self.user_id.as_deref()
    }

    /// Normalize server URL
    fn normalize_url(mut url: String) -> Result<String> {
        // Remove trailing slash
        if url.ends_with('/') {
            url.pop();
        }

        // Add protocol if missing
        if !url.starts_with("http://") && !url.starts_with("https://") {
            url = format!("http://{}", url);
        }

        Ok(url)
    }

    /// Get device name
    fn get_device_name() -> String {
        std::env::var("HOSTNAME")
            .or_else(|_| std::env::var("COMPUTERNAME"))
            .unwrap_or_else(|_| "unknown".to_string())
    }

    /// Get device ID (persistent)
    fn get_device_id() -> String {
        // Use a machine ID if available, otherwise generate one
        // For now, generate a stable ID based on hostname
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let hostname = Self::get_device_name();
        let mut hasher = DefaultHasher::new();
        hostname.hash(&mut hasher);
        format!("{:016x}", hasher.finish())
    }

    /// Build API URL
    fn api_url(&self, path: &str) -> Result<String> {
        let path = path.trim_start_matches('/');
        Ok(format!("{}/{}", self.server_url, path))
    }

    /// Get authorization header value
    fn auth_header(&self) -> String {
        format!(
            "MediaBrowser Client={}, Device={}, DeviceId={}, Version={}, Token={}",
            self.client_name,
            self.device_name,
            self.device_id,
            self.api_version,
            self.token.as_deref().unwrap_or("")
        )
    }

    /// Build a request with authentication
    fn request(&self, method: reqwest::Method, path: &str) -> Result<reqwest::RequestBuilder> {
        let url = self.api_url(path)?;
        debug!("{} {}", method, url);

        let mut builder = self
            .client
            .request(method.clone(), &url)
            .header("X-Emby-Authorization", self.auth_header())
            .header("X-Application", "jellyfin-cli/0.1.0");

        if let Some(token) = &self.token {
            builder = builder.header("X-MediaBrowser-Token", token);
        }

        Ok(builder)
    }

    /// Execute a request and deserialize response
    async fn execute<T: DeserializeOwned>(&self, builder: reqwest::RequestBuilder) -> Result<T> {
        let response = builder.send().await.map_err(JellyfinError::from)?;

        // Check for authentication errors
        if response.status() == reqwest::StatusCode::UNAUTHORIZED {
            return Err(JellyfinError::auth_failed("Invalid or expired token"));
        }

        // Check for not found
        if response.status() == reqwest::StatusCode::NOT_FOUND {
            return Err(JellyfinError::not_found("resource".to_string()));
        }

        // Check for other errors
        if !response.status().is_success() {
            let status = response.status();
            let text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unable to read error".to_string());
            debug!("API error: {} - {}", status, text);
            return Err(JellyfinError::api_error(format!("{}: {}", status, text)));
        }

        response.json().await.map_err(JellyfinError::from)
    }

    /// GET request
    pub async fn get<T: DeserializeOwned, Q: Serialize>(&self, path: &str, query: &Q) -> Result<T> {
        let builder = self.request(reqwest::Method::GET, path)?.query(query);
        self.execute(builder).await
    }

    /// GET request without query
    pub async fn get_raw<T: DeserializeOwned>(&self, path: &str) -> Result<T> {
        let builder = self.request(reqwest::Method::GET, path)?;
        self.execute(builder).await
    }

    /// POST request
    pub async fn post<T: DeserializeOwned, B: Serialize>(&self, path: &str, body: &B) -> Result<T> {
        let builder = self.request(reqwest::Method::POST, path)?.json(body);
        self.execute(builder).await
    }

    /// POST request without body
    pub async fn post_raw<T: DeserializeOwned>(&self, path: &str) -> Result<T> {
        let builder = self.request(reqwest::Method::POST, path)?;
        self.execute(builder).await
    }

    /// DELETE request
    pub async fn delete<T: DeserializeOwned>(&self, path: &str) -> Result<T> {
        let builder = self.request(reqwest::Method::DELETE, path)?;
        self.execute(builder).await
    }

    /// DELETE request expecting no response body (204 No Content)
    pub async fn delete_void(&self, path: &str) -> Result<()> {
        let builder = self.request(reqwest::Method::DELETE, path)?;
        let response = builder.send().await.map_err(JellyfinError::from)?;

        if response.status() == reqwest::StatusCode::NO_CONTENT {
            return Ok(());
        }

        // Check for authentication errors
        if response.status() == reqwest::StatusCode::UNAUTHORIZED {
            return Err(JellyfinError::auth_failed("Invalid or expired token"));
        }

        // Check for not found
        if response.status() == reqwest::StatusCode::NOT_FOUND {
            return Err(JellyfinError::not_found("resource".to_string()));
        }

        // Check for other errors
        if !response.status().is_success() {
            let status = response.status();
            let text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unable to read error".to_string());
            debug!("API error: {} - {}", status, text);
            return Err(JellyfinError::api_error(format!("{}: {}", status, text)));
        }

        Ok(())
    }

    // ===== Authentication =====

    /// Authenticate by username and password
    pub async fn authenticate(&mut self, username: &str, password: &str) -> Result<AuthResponse> {
        #[derive(Serialize)]
        struct AuthRequest {
            #[serde(rename = "Username")]
            username: String,
            #[serde(rename = "Pw")]
            password: String,
        }

        let request = AuthRequest {
            username: username.to_string(),
            password: password.to_string(),
        };

        let response: AuthResponse = self.post("/Users/AuthenticateByName", &request).await?;

        // Cache the user ID from the auth response
        if let Some(user) = &response.user {
            self.user_id = Some(user.id.clone());
        }

        Ok(response)
    }

    /// Get current user info
    pub async fn get_current_user(&self) -> Result<UserInfo> {
        match &self.user_id {
            Some(id) => self.get_raw(&format!("/Users/{}", id)).await,
            None => self.get_raw("/Users/Me").await,
        }
    }

    // ===== Server =====

    /// Get public server info
    pub async fn get_server_info(&self) -> Result<ServerInfo> {
        self.get_raw("/System/Info/Public").await
    }

    /// Get full server info (requires auth)
    pub async fn get_full_server_info(&self) -> Result<ServerInfo> {
        self.get_raw("/System/Info").await
    }

    // ===== Items =====

    /// Query items
    pub async fn get_items(&self, query: &ItemQuery) -> Result<ItemQueryResult> {
        self.get("/Items", query).await
    }

    /// Get item by ID
    pub async fn get_item(&self, item_id: &str) -> Result<BaseItemDto> {
        let path = match &self.user_id {
            Some(id) => format!("/Users/{}/Items/{}", id, item_id),
            None => format!("/Users/Me/Items/{}", item_id),
        };
        self.get_raw(&path).await
    }

    /// Search items
    pub async fn search_items(&self, search_term: &str) -> Result<ItemQueryResult> {
        let query = ItemQuery {
            search_term: Some(search_term.to_string()),
            recursive: Some(true),
            enable_user_data: Some(true),
            ..Default::default()
        };
        self.get("/Items", &query).await
    }

    /// Get resume items
    pub async fn get_resume_items(&self) -> Result<ItemQueryResult> {
        let query = ItemQuery {
            recursive: Some(true),
            sort_by: Some("DatePlayed".to_string()),
            sort_order: Some("Descending".to_string()),
            filters: Some(vec!["IsResumable".to_string()]),
            enable_user_data: Some(true),
            ..Default::default()
        };
        let path = match &self.user_id {
            Some(id) => format!("/Users/{}/Items", id),
            None => "/Users/Me/Items".to_string(),
        };
        self.get(&path, &query).await
    }

    /// Get latest items
    pub async fn get_latest_items(&self) -> Result<Vec<BaseItemDto>> {
        let user_id = self.user_id.as_deref().unwrap_or("Me");
        self.get_raw(&format!("/Users/{}/Items/Latest", user_id))
            .await
    }

    /// Refresh item metadata
    pub async fn refresh_item(&self, item_id: &str) -> Result<()> {
        #[derive(Serialize)]
        struct RefreshRequest {
            #[serde(rename = "Recursive")]
            recursive: bool,
            #[serde(rename = "ImageRefreshMode")]
            image_refresh_mode: String,
            #[serde(rename = "MetadataRefreshMode")]
            metadata_refresh_mode: String,
        }

        let request = RefreshRequest {
            recursive: true,
            image_refresh_mode: "Default".to_string(),
            metadata_refresh_mode: "Default".to_string(),
        };

        self.post_void(&format!("/Items/{}/Refresh", item_id), &request)
            .await
    }

    /// Delete an item
    pub async fn delete_item(&self, item_id: &str) -> Result<()> {
        self.delete_void(&format!("/Items/{}", item_id)).await
    }

    // ===== Libraries =====

    /// Get media libraries (views)
    pub async fn get_libraries(&self) -> Result<Vec<BaseItemDto>> {
        use serde::Deserialize;

        #[derive(Deserialize)]
        struct ViewsResponse {
            #[serde(rename = "Items")]
            items: Vec<BaseItemDto>,
        }

        let path = match &self.user_id {
            Some(id) => format!("/Users/{}/Views", id),
            None => "/Users/Me/Views".to_string(),
        };
        let response: ViewsResponse = self.get_raw(&path).await?;
        Ok(response.items)
    }

    /// Get items in a library
    pub async fn get_library_items(&self, library_id: &str) -> Result<ItemQueryResult> {
        let query = ItemQuery {
            parent_id: Some(library_id.to_string()),
            recursive: Some(true),
            sort_by: Some("SortName".to_string()),
            sort_order: Some("Ascending".to_string()),
            enable_user_data: Some(true),
            ..Default::default()
        };
        self.get("/Items", &query).await
    }

    // ===== Users =====

    /// Get all users
    pub async fn get_users(&self) -> Result<Vec<UserInfo>> {
        // /Users returns an array directly, not UserListResult
        self.get_raw("/Users").await
    }

    /// Get user by ID
    pub async fn get_user(&self, user_id: &str) -> Result<UserInfo> {
        self.get_raw(&format!("/Users/{}", user_id)).await
    }

    /// Create a new user
    pub async fn create_user(&self, name: &str, password: &str) -> Result<UserInfo> {
        #[derive(Serialize)]
        struct CreateUserRequest {
            #[serde(rename = "Name")]
            name: String,
            #[serde(rename = "Password")]
            password: String,
        }

        let request = CreateUserRequest {
            name: name.to_string(),
            password: password.to_string(),
        };

        self.post("/Users/New", &request).await
    }

    /// Delete a user
    pub async fn delete_user(&self, user_id: &str) -> Result<()> {
        self.delete_void(&format!("/Users/{}", user_id)).await
    }

    // ===== Playback =====

    /// Get playback info for an item
    pub async fn get_playback_info(&self, item_id: &str) -> Result<serde_json::Value> {
        self.get_raw(&format!("/Items/{}/PlaybackInfo", item_id))
            .await
    }

    /// Get stream URL for an item
    pub fn get_stream_url(&self, item_id: &str, media_source_id: Option<&str>) -> String {
        let mut url = format!("{}/Videos/{}/stream?static=true", self.server_url, item_id);
        if let Some(source_id) = media_source_id {
            url.push_str(&format!("&MediaSourceId={}", source_id));
        }
        url.push_str(&format!("&api_key={}", self.token.as_deref().unwrap_or("")));
        url
    }

    /// Report playback start
    pub async fn report_playback_start(
        &self,
        item_id: &str,
        session_id: Option<&str>,
    ) -> Result<()> {
        #[derive(Serialize)]
        struct PlaybackStartRequest {
            #[serde(rename = "ItemId")]
            item_id: String,
            #[serde(rename = "SessionId", skip_serializing_if = "Option::is_none")]
            session_id: Option<String>,
        }

        let request = PlaybackStartRequest {
            item_id: item_id.to_string(),
            session_id: session_id.map(|s| s.to_string()),
        };

        self.post_void("/Sessions/Playing", &request).await?;
        Ok(())
    }

    /// Report playback progress
    pub async fn report_playback_progress(
        &self,
        item_id: &str,
        position_ticks: u64,
        is_paused: bool,
    ) -> Result<()> {
        #[derive(Serialize)]
        struct PlaybackProgressRequest {
            #[serde(rename = "ItemId")]
            item_id: String,
            #[serde(rename = "PositionTicks")]
            position_ticks: u64,
            #[serde(rename = "IsPaused")]
            is_paused: bool,
        }

        let request = PlaybackProgressRequest {
            item_id: item_id.to_string(),
            position_ticks,
            is_paused,
        };

        self.post_void("/Sessions/Playing/Progress", &request)
            .await?;
        Ok(())
    }

    /// Report playback stopped
    pub async fn report_playback_stopped(&self, item_id: &str, position_ticks: u64) -> Result<()> {
        #[derive(Serialize)]
        struct PlaybackStopRequest {
            #[serde(rename = "ItemId")]
            item_id: String,
            #[serde(rename = "PositionTicks")]
            position_ticks: u64,
        }

        let request = PlaybackStopRequest {
            item_id: item_id.to_string(),
            position_ticks,
        };

        self.post_void("/Sessions/Playing/Stopped", &request)
            .await?;
        Ok(())
    }

    // ===== Remote Control =====

    /// Send pause command to a remote session
    pub async fn send_pause(&self, session_id: &str) -> Result<()> {
        let path = format!("/Sessions/{}/Playing/Pause", session_id);
        self.post_void(&path, &EmptyBody).await
    }

    /// Send unpause command to a remote session
    pub async fn send_unpause(&self, session_id: &str) -> Result<()> {
        let path = format!("/Sessions/{}/Playing/Unpause", session_id);
        self.post_void(&path, &EmptyBody).await
    }

    /// Send stop command to a remote session
    pub async fn send_stop(&self, session_id: &str) -> Result<()> {
        let path = format!("/Sessions/{}/Playing/Stop", session_id);
        self.post_void(&path, &EmptyBody).await
    }

    /// Send seek command to a remote session
    pub async fn send_seek(&self, session_id: &str, seek_position_ticks: u64) -> Result<()> {
        #[derive(Serialize)]
        struct SeekRequest {
            #[serde(rename = "SeekPositionTicks")]
            seek_position_ticks: u64,
        }

        let path = format!("/Sessions/{}/Playing/Seek", session_id);
        self.post_void(&path, &SeekRequest { seek_position_ticks })
            .await
    }

    /// POST request without expected response body
    async fn post_void<B: Serialize>(&self, path: &str, body: &B) -> Result<()> {
        let builder = self.request(reqwest::Method::POST, path)?.json(body);
        let response = builder.send().await.map_err(JellyfinError::from)?;

        if response.status() == reqwest::StatusCode::UNAUTHORIZED {
            return Err(JellyfinError::auth_failed("Invalid or expired token"));
        }
        if !response.status().is_success() {
            let status = response.status();
            let text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unable to read error".to_string());
            return Err(JellyfinError::api_error(format!("{}: {}", status, text)));
        }
        Ok(())
    }

    // ===== Download =====

    /// Get a download stream for a media item.
    /// Returns the raw response for streaming download.
    pub async fn download_stream(&self, item_id: &str) -> Result<reqwest::Response> {
        let path = format!("/Items/{}/Download", item_id);
        let builder = self.request(reqwest::Method::GET, &path)?;
        let response = builder.send().await.map_err(JellyfinError::from)?;

        if response.status() == reqwest::StatusCode::UNAUTHORIZED {
            return Err(JellyfinError::auth_failed("Invalid or expired token"));
        }
        if response.status() == reqwest::StatusCode::NOT_FOUND {
            return Err(JellyfinError::not_found(format!(
                "item {} for download",
                item_id
            )));
        }
        if !response.status().is_success() {
            let status = response.status();
            let text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unable to read error".to_string());
            return Err(JellyfinError::api_error(format!("{}: {}", status, text)));
        }

        Ok(response)
    }

    // ===== Favorites =====

    /// Add item to favorites
    pub async fn add_favorite(&self, item_id: &str) -> Result<()> {
        let path = format!("/UserLibrary/Favorite/{}", item_id);
        #[derive(Serialize)]
        struct EmptyBody {}
        self.post_void(&path, &EmptyBody {}).await
    }

    /// Remove item from favorites
    pub async fn remove_favorite(&self, item_id: &str) -> Result<()> {
        let path = format!("/UserLibrary/Favorite/{}", item_id);
        self.delete(&path).await
    }

    /// Get favorite items
    pub async fn get_favorites(&self) -> Result<ItemQueryResult> {
        let query = ItemQuery {
            is_favorite: Some(true),
            recursive: Some(true),
            enable_user_data: Some(true),
            ..Default::default()
        };
        let path = match &self.user_id {
            Some(id) => format!("/Users/{}/Items", id),
            None => "/Users/Me/Items".to_string(),
        };
        self.get(&path, &query).await
    }

    // ===== Ratings =====

    /// Rate an item (like/unlike or score 1-10)
    pub async fn rate_item(&self, item_id: &str, rating: Option<f64>) -> Result<()> {
        #[derive(Serialize)]
        struct RatingRequest {
            #[serde(rename = "Like")]
            like: Option<bool>,
            #[serde(rename = "Rating", skip_serializing_if = "Option::is_none")]
            rating: Option<f64>,
        }

        let request = RatingRequest {
            like: rating.map(|r| r > 0.0),
            rating,
        };

        let path = format!("/UserLibrary/Rating/{}", item_id);
        self.post_void(&path, &request).await
    }

    // ===== Libraries =====

    /// Add a virtual folder (media library)
    pub async fn add_library(
        &self,
        name: &str,
        collection_type: &str,
        _paths: Vec<String>,
    ) -> Result<()> {
        // Jellyfin VirtualFolders API accepts name and collectionType as query params
        // The paths parameter caused issues in testing, so we omit it for now
        let url = format!(
            "/Library/VirtualFolders?name={}&collectionType={}",
            name, collection_type
        );

        // POST with empty body to create virtual folder
        #[derive(Serialize)]
        struct EmptyBody {}
        let builder = self
            .request(reqwest::Method::POST, &url)?
            .json(&EmptyBody {});
        let response = builder.send().await.map_err(JellyfinError::from)?;

        if response.status() == reqwest::StatusCode::UNAUTHORIZED {
            return Err(JellyfinError::auth_failed("Invalid or expired token"));
        }
        if !response.status().is_success() {
            let status = response.status();
            let text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unable to read error".to_string());
            return Err(JellyfinError::api_error(format!("{}: {}", status, text)));
        }
        Ok(())
    }

    /// Remove a virtual folder
    pub async fn remove_library(&self, name: &str) -> Result<()> {
        let path = format!("/Library/VirtualFolders/{}", urlencoding::encode(name));
        self.delete(&path).await
    }

    // ===== Scheduled Tasks =====

    /// Get all scheduled tasks
    pub async fn get_scheduled_tasks(&self) -> Result<Vec<ScheduledTaskInfo>> {
        self.get_raw("/ScheduledTasks").await
    }

    /// Get a specific task
    pub async fn get_scheduled_task(&self, task_id: &str) -> Result<ScheduledTaskInfo> {
        self.get_raw(&format!("/ScheduledTasks/{}", task_id)).await
    }

    /// Start a scheduled task
    pub async fn start_scheduled_task(&self, task_id: &str) -> Result<()> {
        let path = format!("/ScheduledTasks/{}/Start", task_id);
        self.post_void(&path, &EmptyBody {}).await
    }

    /// Stop a scheduled task
    pub async fn stop_scheduled_task(&self, task_id: &str) -> Result<()> {
        let path = format!("/ScheduledTasks/{}/Stop", task_id);
        self.post_void(&path, &EmptyBody {}).await
    }

    // ===== System =====

    /// Restart the server
    pub async fn restart_server(&self) -> Result<()> {
        self.post_void("/System/Restart", &EmptyBody {}).await
    }

    /// Shutdown the server
    pub async fn shutdown_server(&self) -> Result<()> {
        self.post_void("/System/Shutdown", &EmptyBody {}).await
    }

    // ===== Devices =====

    /// Get all devices
    pub async fn get_devices(&self) -> Result<DeviceListResult> {
        self.get_raw("/Devices").await
    }

    /// Get device by ID
    pub async fn get_device(&self, device_id: &str) -> Result<DeviceInfo> {
        self.get_raw(&format!("/Devices/{}", device_id)).await
    }

    // ===== Playlists =====

    /// Get all playlists
    pub async fn get_playlists(&self) -> Result<Vec<BaseItemDto>> {
        // Playlists are retrieved via Items API with Playlist filter
        // Note: Jellyfin 10.11.8 requires actual user ID, not "Me"
        let user_id = self.user_id.as_deref().unwrap_or("Me");
        let url = format!("/Users/{}/Items", user_id);

        // Build query parameters manually to avoid serialization issues
        let full_url = format!(
            "{}?IncludeItemTypes=Playlist&Recursive=true&SortBy=SortName&EnableUserData=true",
            url
        );

        // Use get_raw for simple JSON array response
        let result: ItemQueryResult = self.get_raw(&full_url).await?;
        Ok(result.items)
    }

    /// Get playlist by ID
    pub async fn get_playlist(&self, playlist_id: &str) -> Result<PlaylistInfo> {
        self.get_raw(&format!("/Playlists/{}", playlist_id)).await
    }

    /// Create a playlist
    pub async fn create_playlist(
        &self,
        name: &str,
        media_ids: Option<Vec<String>>,
    ) -> Result<CreatePlaylistResponse> {
        #[derive(Serialize)]
        struct CreatePlaylistRequest {
            #[serde(rename = "Name")]
            name: String,
            #[serde(rename = "MediaIds", skip_serializing_if = "Option::is_none")]
            media_ids: Option<Vec<String>>,
            #[serde(rename = "PlaylistMediaType")]
            playlist_media_type: Option<String>,
        }

        let request = CreatePlaylistRequest {
            name: name.to_string(),
            media_ids,
            playlist_media_type: None,
        };

        self.post("/Playlists", &request).await
    }

    /// Add items to playlist
    pub async fn add_to_playlist(&self, playlist_id: &str, media_ids: Vec<String>) -> Result<()> {
        #[derive(Serialize)]
        struct AddItemsRequest {
            #[serde(rename = "MediaIds")]
            media_ids: Vec<String>,
        }

        let request = AddItemsRequest { media_ids };
        self.post_void(&format!("/Playlists/{}", playlist_id), &request)
            .await
    }

    /// Remove items from playlist
    pub async fn remove_from_playlist(
        &self,
        playlist_id: &str,
        media_ids: Vec<String>,
    ) -> Result<()> {
        #[derive(Serialize)]
        struct RemoveItemsRequest {
            #[serde(rename = "MediaIds")]
            media_ids: Vec<String>,
        }

        let request = RemoveItemsRequest { media_ids };
        // Use DELETE with body
        let builder = self
            .request(
                reqwest::Method::DELETE,
                &format!("/Playlists/{}", playlist_id),
            )?
            .json(&request);
        let response = builder.send().await.map_err(JellyfinError::from)?;

        if response.status() == reqwest::StatusCode::UNAUTHORIZED {
            return Err(JellyfinError::auth_failed("Invalid or expired token"));
        }
        if !response.status().is_success() {
            let status = response.status();
            let text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unable to read error".to_string());
            return Err(JellyfinError::api_error(format!("{}: {}", status, text)));
        }
        Ok(())
    }

    /// Delete a playlist
    pub async fn delete_playlist(&self, playlist_id: &str) -> Result<()> {
        // Playlists are deleted via Items endpoint
        self.delete_void(&format!("/Items/{}", playlist_id)).await
    }

    // ===== Notifications =====

    /// Get notification summary
    pub async fn get_notifications(&self) -> Result<NotificationResult> {
        let user_id = self.user_id.as_deref().unwrap_or("Me");
        self.get_raw(&format!("/Users/{}/Notifications", user_id))
            .await
    }

    /// Mark notification as read
    pub async fn mark_notification_read(&self, notification_id: &str) -> Result<()> {
        let user_id = self.user_id.as_deref().unwrap_or("Me");
        let path = format!("/Users/{}/Notifications/{}/Read", user_id, notification_id);
        self.post_void(&path, &EmptyBody {}).await
    }

    /// Mark all notifications as read
    pub async fn mark_all_notifications_read(&self) -> Result<()> {
        let user_id = self.user_id.as_deref().unwrap_or("Me");
        self.post_void(
            &format!("/Users/{}/Notifications/Read", user_id),
            &EmptyBody {},
        )
        .await
    }

    // ===== Plugins =====

    /// Get all plugins
    pub async fn get_plugins(&self) -> Result<Vec<PluginInfo>> {
        self.get_raw("/Plugins").await
    }

    /// Get plugin by ID
    pub async fn get_plugin(&self, plugin_id: &str) -> Result<PluginInfo> {
        self.get_raw(&format!("/Plugins/{}", plugin_id)).await
    }

    /// Uninstall a plugin
    pub async fn uninstall_plugin(&self, plugin_id: &str) -> Result<()> {
        self.delete(&format!("/Plugins/{}", plugin_id)).await
    }

    // ===== Channels =====

    /// Get all channels
    pub async fn get_channels(&self) -> Result<ChannelResult> {
        self.get_raw("/Channels").await
    }

    /// Get channel items
    pub async fn get_channel_items(&self, channel_id: &str) -> Result<ChannelItemResult> {
        self.get_raw(&format!("/Channels/{}/Items", channel_id))
            .await
    }

    // ===== Genres =====

    /// Get all genres
    pub async fn get_genres(&self) -> Result<ItemQueryResult> {
        self.get("/Genres", &ItemQuery::default()).await
    }

    // ===== Studios =====

    /// Get all studios
    pub async fn get_studios(&self) -> Result<ItemQueryResult> {
        self.get("/Studios", &ItemQuery::default()).await
    }

    /// Get studio by name
    pub async fn get_studio(&self, studio_name: &str) -> Result<ItemQueryResult> {
        let query = ItemQuery {
            filters: Some(vec![studio_name.to_string()]),
            ..Default::default()
        };
        self.get("/Studios", &query).await
    }

    // ===== Actors =====

    /// Get all actors (artists for music)
    pub async fn get_actors(&self) -> Result<ItemQueryResult> {
        self.get("/Artists", &ItemQuery::default()).await
    }

    /// Get actor by name
    pub async fn get_actor(&self, actor_name: &str) -> Result<ItemQueryResult> {
        let query = ItemQuery {
            filters: Some(vec![actor_name.to_string()]),
            ..Default::default()
        };
        self.get("/Artists", &query).await
    }

    // ===== Sessions =====

    /// Get all sessions
    pub async fn get_sessions(&self) -> Result<Vec<SessionInfo>> {
        self.get_raw("/Sessions").await
    }

    // ===== Activity Log =====

    /// Get activity log entries
    pub async fn get_activity_log(&self) -> Result<ActivityLogResult> {
        self.get_raw("/System/ActivityLog/Entries").await
    }

    // ===== Remote Search =====

    /// Search remote providers
    pub async fn remote_search(
        &self,
        query: &RemoteSearchQuery,
    ) -> Result<Vec<RemoteSearchResult>> {
        self.post("/Search/Remote", query).await
    }

    // ===== Items Additional =====

    /// Get item download URL
    pub fn get_download_url(&self, item_id: &str) -> String {
        format!(
            "{}/Items/{}/Download?api_key={}",
            self.server_url,
            item_id,
            self.token.as_deref().unwrap_or("")
        )
    }
}

/// Empty query type for requests without parameters
#[derive(Serialize)]
pub struct EmptyQuery;

/// Empty body type for POST requests without a body
#[derive(Serialize)]
pub struct EmptyBody;

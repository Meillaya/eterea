use base64::Engine;
use chrono::{DateTime, Duration, Utc};
use eterea_core::models::BookmarkBuilder;
use eterea_core::Bookmark;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use sha2::Digest;
use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::time::{Duration as StdDuration, Instant};
use url::Url;
use uuid::Uuid;

const X_AUTH_URL: &str = "https://x.com/i/oauth2/authorize";
const X_TOKEN_URL: &str = "https://api.x.com/2/oauth2/token";
const X_USERS_ME_URL: &str = "https://api.x.com/2/users/me";
const X_SYNC_METADATA_KEY: &str = "x_sync_status";
const DEFAULT_REDIRECT_URI: &str = "http://127.0.0.1:38347/callback";
const DEFAULT_SCOPES: &str = "tweet.read users.read bookmark.read";

#[derive(Debug, Clone)]
pub struct XSessionToken {
    pub access_token: String,
    pub expires_at: Option<DateTime<Utc>>,
}

impl XSessionToken {
    pub fn is_valid(&self) -> bool {
        self.expires_at
            .map(|expires_at| expires_at > Utc::now() + Duration::seconds(30))
            .unwrap_or(true)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PersistedXSyncStatus {
    pub last_attempted_at: Option<String>,
    pub last_synced_at: Option<String>,
    pub last_status: Option<String>,
    pub last_error: Option<String>,
    pub last_imported_count: Option<usize>,
    pub last_skipped_count: Option<usize>,
    pub total_fetched: Option<usize>,
}

#[derive(Debug, Clone, Serialize)]
pub struct XSyncStatus {
    pub configured: bool,
    pub connected: bool,
    pub last_attempted_at: Option<String>,
    pub last_synced_at: Option<String>,
    pub last_status: Option<String>,
    pub last_error: Option<String>,
    pub last_imported_count: Option<usize>,
    pub last_skipped_count: Option<usize>,
    pub total_fetched: Option<usize>,
}

#[derive(Debug, Clone, Serialize)]
pub struct XImportSummary {
    pub imported_count: usize,
    pub skipped_count: usize,
    pub total_fetched: usize,
    pub last_synced_at: String,
    pub reauthenticated: bool,
    pub status_message: String,
}

#[derive(Debug)]
pub struct XImportOutcome {
    pub session: XSessionToken,
    pub bookmarks: Vec<Bookmark>,
    pub reauthenticated: bool,
}

#[derive(Debug, Clone)]
pub struct XClientConfig {
    client_id: Option<String>,
    redirect_uri: String,
    scopes: &'static str,
}

impl XClientConfig {
    pub fn from_env() -> Self {
        let client_id = std::env::var("ETEREA_X_CLIENT_ID")
            .ok()
            .or_else(|| option_env!("ETEREA_X_CLIENT_ID").map(str::to_string))
            .filter(|value| !value.trim().is_empty());
        let redirect_uri = std::env::var("ETEREA_X_REDIRECT_URI")
            .ok()
            .or_else(|| option_env!("ETEREA_X_REDIRECT_URI").map(str::to_string))
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| DEFAULT_REDIRECT_URI.to_string());

        Self {
            client_id,
            redirect_uri,
            scopes: DEFAULT_SCOPES,
        }
    }

    pub fn is_configured(&self) -> bool {
        self.client_id.is_some()
    }

    pub fn client_id(&self) -> Result<&str, String> {
        self.client_id
            .as_deref()
            .ok_or_else(|| {
                "X import is not configured in this build. Set ETEREA_X_CLIENT_ID at build or runtime."
                    .to_string()
            })
    }

    pub fn redirect_uri(&self) -> &str {
        &self.redirect_uri
    }

    pub fn scopes(&self) -> &str {
        self.scopes
    }
}

pub fn load_sync_status(metadata_json: Option<String>, session: Option<&XSessionToken>) -> XSyncStatus {
    let persisted = metadata_json
        .and_then(|json| serde_json::from_str::<PersistedXSyncStatus>(&json).ok())
        .unwrap_or_default();

    XSyncStatus {
        configured: XClientConfig::from_env().is_configured(),
        connected: session.map(XSessionToken::is_valid).unwrap_or(false),
        last_attempted_at: persisted.last_attempted_at,
        last_synced_at: persisted.last_synced_at,
        last_status: persisted.last_status,
        last_error: persisted.last_error,
        last_imported_count: persisted.last_imported_count,
        last_skipped_count: persisted.last_skipped_count,
        total_fetched: persisted.total_fetched,
    }
}

pub fn metadata_key() -> &'static str {
    X_SYNC_METADATA_KEY
}

pub fn build_failed_sync_status_from_previous(
    previous_json: Option<String>,
    message: impl Into<String>,
) -> PersistedXSyncStatus {
    let mut status = previous_json
        .and_then(|json| serde_json::from_str::<PersistedXSyncStatus>(&json).ok())
        .unwrap_or_default();
    status.last_attempted_at = Some(Utc::now().to_rfc3339());
    status.last_status = Some("failed".to_string());
    status.last_error = Some(message.into());
    status
}

pub fn serialize_sync_status(status: &PersistedXSyncStatus) -> Result<String, String> {
    serde_json::to_string(status).map_err(|error| error.to_string())
}

pub async fn import_bookmarks_from_x(
    existing_session: Option<XSessionToken>,
) -> Result<XImportOutcome, String> {
    let config = XClientConfig::from_env();
    let client = reqwest::Client::builder()
        .user_agent("Eterea/0.1 (+https://github.com/eterea)")
        .redirect(reqwest::redirect::Policy::limited(5))
        .build()
        .map_err(|error| error.to_string())?;

    let mut reauthenticated = false;
    let session = match existing_session {
        Some(session) if session.is_valid() => session,
        _ => {
            reauthenticated = true;
            authenticate(&client, &config).await?
        }
    };

    let user = fetch_authenticated_user(&client, &session.access_token).await?;
    let bookmarks = fetch_all_bookmarks(&client, &session.access_token, &user).await?;

    Ok(XImportOutcome {
        session,
        bookmarks,
        reauthenticated,
    })
}

pub fn build_success_sync_status(summary: &XImportSummary) -> PersistedXSyncStatus {
    PersistedXSyncStatus {
        last_attempted_at: Some(summary.last_synced_at.clone()),
        last_synced_at: Some(summary.last_synced_at.clone()),
        last_status: Some("success".to_string()),
        last_error: None,
        last_imported_count: Some(summary.imported_count),
        last_skipped_count: Some(summary.skipped_count),
        total_fetched: Some(summary.total_fetched),
    }
}

async fn authenticate(client: &reqwest::Client, config: &XClientConfig) -> Result<XSessionToken, String> {
    let client_id = config.client_id()?;
    let code_verifier = format!("{}{}", Uuid::new_v4().simple(), Uuid::new_v4().simple());
    let code_challenge = pkce_challenge(&code_verifier);
    let state = Uuid::new_v4().simple().to_string();

    let authorization_url = build_authorization_url(config, client_id, &state, &code_challenge)?;
    open::that(&authorization_url).map_err(|error| error.to_string())?;

    let redirect_uri = config.redirect_uri().to_string();
    let expected_state = state.clone();
    let callback_url = tauri::async_runtime::spawn_blocking(move || {
        wait_for_oauth_callback(&redirect_uri, &expected_state, StdDuration::from_secs(180))
    })
    .await
    .map_err(|error| error.to_string())??;

    let callback = Url::parse(&callback_url).map_err(|error| error.to_string())?;
    let code = callback
        .query_pairs()
        .find(|(key, _)| key == "code")
        .map(|(_, value)| value.into_owned())
        .ok_or_else(|| "X did not return an authorization code.".to_string())?;

    exchange_code_for_token(client, config, client_id, &code, &code_verifier).await
}

fn build_authorization_url(
    config: &XClientConfig,
    client_id: &str,
    state: &str,
    code_challenge: &str,
) -> Result<String, String> {
    let mut url = Url::parse(X_AUTH_URL).map_err(|error| error.to_string())?;
    url.query_pairs_mut()
        .append_pair("response_type", "code")
        .append_pair("client_id", client_id)
        .append_pair("redirect_uri", config.redirect_uri())
        .append_pair("scope", config.scopes())
        .append_pair("state", state)
        .append_pair("code_challenge", code_challenge)
        .append_pair("code_challenge_method", "S256");
    Ok(url.to_string())
}

fn pkce_challenge(verifier: &str) -> String {
    let digest = sha2::Sha256::digest(verifier.as_bytes());
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(digest)
}

fn wait_for_oauth_callback(
    redirect_uri: &str,
    expected_state: &str,
    timeout: StdDuration,
) -> Result<String, String> {
    let redirect = Url::parse(redirect_uri).map_err(|error| error.to_string())?;
    let host = redirect.host_str().unwrap_or("127.0.0.1");
    let port = redirect
        .port_or_known_default()
        .ok_or_else(|| "Redirect URI must include an explicit localhost port.".to_string())?;
    let expected_path = redirect.path().to_string();

    let listener = TcpListener::bind((host, port)).map_err(|error| {
        format!("Failed to bind OAuth callback listener on {host}:{port}: {error}")
    })?;
    listener
        .set_nonblocking(true)
        .map_err(|error| error.to_string())?;

    let deadline = Instant::now() + timeout;
    while Instant::now() < deadline {
        match listener.accept() {
            Ok((mut stream, _addr)) => {
                let _ = stream.set_read_timeout(Some(StdDuration::from_secs(5)));
                let _ = stream.set_write_timeout(Some(StdDuration::from_secs(5)));

                let mut buffer = [0_u8; 4096];
                let read = stream.read(&mut buffer).map_err(|error| error.to_string())?;
                let request = String::from_utf8_lossy(&buffer[..read]);
                let request_line = request
                    .lines()
                    .next()
                    .ok_or_else(|| "Received malformed OAuth callback request.".to_string())?;
                let path = request_line
                    .split_whitespace()
                    .nth(1)
                    .ok_or_else(|| "OAuth callback request is missing a path.".to_string())?;
                let callback_url = format!("http://localhost{path}");
                let callback = Url::parse(&callback_url).map_err(|error| error.to_string())?;

                let body = if callback.path() != expected_path {
                    failure_page("OAuth callback path did not match the configured redirect URI.")
                } else if let Some(error) = callback
                    .query_pairs()
                    .find(|(key, _)| key == "error")
                    .map(|(_, value)| value.into_owned())
                {
                    failure_page(&format!("X denied access: {error}"))
                } else {
                    let returned_state = callback
                        .query_pairs()
                        .find(|(key, _)| key == "state")
                        .map(|(_, value)| value.into_owned())
                        .unwrap_or_default();

                    if returned_state != expected_state {
                        failure_page("OAuth state mismatch. Please retry the import.")
                    } else {
                        success_page()
                    }
                };

                let response = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                    body.len(),
                    body
                );
                let _ = stream.write_all(response.as_bytes());
                let _ = stream.flush();

                if callback.path() != expected_path {
                    return Err("OAuth callback path mismatch.".to_string());
                }
                if let Some(error) = callback
                    .query_pairs()
                    .find(|(key, _)| key == "error")
                    .map(|(_, value)| value.into_owned())
                {
                    return Err(format!("X denied access: {error}"));
                }
                let returned_state = callback
                    .query_pairs()
                    .find(|(key, _)| key == "state")
                    .map(|(_, value)| value.into_owned())
                    .unwrap_or_default();
                if returned_state != expected_state {
                    return Err("OAuth state mismatch. Please retry the import.".to_string());
                }

                return Ok(callback.to_string());
            }
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                std::thread::sleep(StdDuration::from_millis(50));
            }
            Err(error) => return Err(error.to_string()),
        }
    }

    Err("Timed out waiting for the X login to complete.".to_string())
}

async fn exchange_code_for_token(
    client: &reqwest::Client,
    config: &XClientConfig,
    client_id: &str,
    code: &str,
    code_verifier: &str,
) -> Result<XSessionToken, String> {
    let response = client
        .post(X_TOKEN_URL)
        .form(&[
            ("grant_type", "authorization_code"),
            ("code", code),
            ("client_id", client_id),
            ("redirect_uri", config.redirect_uri()),
            ("code_verifier", code_verifier),
        ])
        .send()
        .await
        .map_err(|error| error.to_string())?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(format!("X token exchange failed ({status}): {body}"));
    }

    let token: TokenResponse = response.json().await.map_err(|error| error.to_string())?;
    let expires_at = token
        .expires_in
        .map(|seconds| Utc::now() + Duration::seconds(seconds as i64));

    Ok(XSessionToken {
        access_token: token.access_token,
        expires_at,
    })
}

async fn fetch_authenticated_user(
    client: &reqwest::Client,
    access_token: &str,
) -> Result<ApiUser, String> {
    let response = client
        .get(X_USERS_ME_URL)
        .query(&[("user.fields", "profile_image_url")])
        .bearer_auth(access_token)
        .send()
        .await
        .map_err(|error| error.to_string())?;

    ensure_success(response, "failed to fetch the authenticated X user").await?
        .json::<UserLookupResponse>()
        .await
        .map(|payload| payload.data)
        .map_err(|error| error.to_string())
}

async fn fetch_all_bookmarks(
    client: &reqwest::Client,
    access_token: &str,
    current_user: &ApiUser,
) -> Result<Vec<Bookmark>, String> {
    let mut pagination_token: Option<String> = None;
    let mut all_bookmarks = Vec::new();

    loop {
        let mut request = client
            .get(format!(
                "https://api.x.com/2/users/{}/bookmarks",
                current_user.id
            ))
            .query(&[
                ("max_results", "100"),
                ("tweet.fields", "created_at,author_id,entities,attachments"),
                ("user.fields", "profile_image_url"),
                ("media.fields", "url,preview_image_url,type"),
                ("expansions", "author_id,attachments.media_keys"),
            ])
            .bearer_auth(access_token);

        if let Some(token) = pagination_token.as_deref() {
            request = request.query(&[("pagination_token", token)]);
        }

        let response = request.send().await.map_err(|error| error.to_string())?;
        let payload = ensure_success(response, "failed to fetch X bookmarks")
            .await?
            .json::<BookmarksResponse>()
            .await
            .map_err(|error| error.to_string())?;

        let mut users = HashMap::new();
        users.insert(current_user.id.clone(), current_user.clone());
        if let Some(included_users) = payload.includes.as_ref().and_then(|includes| includes.users.as_ref()) {
            for user in included_users {
                users.insert(user.id.clone(), user.clone());
            }
        }

        let mut media = HashMap::new();
        if let Some(included_media) = payload.includes.as_ref().and_then(|includes| includes.media.as_ref()) {
            for item in included_media {
                media.insert(item.media_key.clone(), item.clone());
            }
        }

        if let Some(tweets) = payload.data {
            for tweet in tweets {
                all_bookmarks.push(map_tweet_to_bookmark(&tweet, &users, &media));
            }
        }

        pagination_token = payload.meta.and_then(|meta| meta.next_token);
        if pagination_token.is_none() {
            break;
        }
    }

    Ok(all_bookmarks)
}

fn map_tweet_to_bookmark(
    tweet: &ApiTweet,
    users: &HashMap<String, ApiUser>,
    media_map: &HashMap<String, ApiMedia>,
) -> Bookmark {
    let author = tweet
        .author_id
        .as_ref()
        .and_then(|author_id| users.get(author_id))
        .cloned()
        .unwrap_or_else(|| ApiUser {
            id: tweet.author_id.clone().unwrap_or_default(),
            username: "unknown".to_string(),
            name: "Unknown author".to_string(),
            profile_image_url: None,
        });

    let tweeted_at = tweet
        .created_at
        .as_deref()
        .and_then(parse_api_datetime)
        .unwrap_or_else(Utc::now);
    let tweet_url = format!("https://x.com/{}/status/{}", author.username, tweet.id);

    let mut builder = BookmarkBuilder::new()
        .tweet_url(tweet_url)
        .content(tweet.text.clone())
        .tweeted_at(tweeted_at)
        .author_handle(author.username.clone())
        .author_name(author.name.clone())
        .author_profile_url(format!("https://x.com/{}", author.username))
        .author_profile_image(author.profile_image_url.clone().unwrap_or_default());

    if let Some(entities) = &tweet.entities {
        if let Some(hashtags) = &entities.hashtags {
            for hashtag in hashtags {
                builder = builder.add_tag(hashtag.tag.to_lowercase());
            }
        }
    }

    if let Some(attachments) = &tweet.attachments {
        if let Some(media_keys) = &attachments.media_keys {
            for media_key in media_keys {
                if let Some(media) = media_map.get(media_key) {
                    if let Some(url) = media.url.clone().or_else(|| media.preview_image_url.clone()) {
                        builder = builder.add_media(url);
                    }
                }
            }
        }
    }

    builder.build().unwrap_or_else(|_| {
        BookmarkBuilder::new()
            .tweet_url(format!("https://x.com/i/web/status/{}", tweet.id))
            .content(tweet.text.clone())
            .tweeted_at(tweeted_at)
            .author_handle(author.username)
            .author_name(author.name)
            .build()
            .expect("fallback bookmark build should succeed")
    })
}

fn parse_api_datetime(value: &str) -> Option<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(value)
        .map(|dt| dt.with_timezone(&Utc))
        .or_else(|_| DateTime::parse_from_str(value, "%a %b %d %H:%M:%S %z %Y").map(|dt| dt.with_timezone(&Utc)))
        .ok()
}

async fn ensure_success(
    response: reqwest::Response,
    context: &str,
) -> Result<reqwest::Response, String> {
    if response.status().is_success() {
        return Ok(response);
    }

    let status = response.status();
    let body = response.text().await.unwrap_or_default();
    let message = match status {
        StatusCode::UNAUTHORIZED => "Your X session expired. Sign in again to continue.".to_string(),
        StatusCode::FORBIDDEN => "X denied access to bookmarks for this app or account.".to_string(),
        StatusCode::TOO_MANY_REQUESTS => "X rate-limited the request. Please wait and try again.".to_string(),
        _ => format!("{context} ({status}): {body}"),
    };

    Err(message)
}

fn success_page() -> String {
    r#"<!doctype html><html><body style=\"font-family: sans-serif; background:#0a0a0f; color:#f0f0f5; display:flex; align-items:center; justify-content:center; min-height:100vh;\"><div style=\"max-width:420px; text-align:center;\"><h1>Connected to X</h1><p>You can return to Eterea. The import will continue automatically.</p></div></body></html>"#.to_string()
}

fn failure_page(message: &str) -> String {
    format!(
        "<!doctype html><html><body style=\"font-family: sans-serif; background:#0a0a0f; color:#f0f0f5; display:flex; align-items:center; justify-content:center; min-height:100vh;\"><div style=\"max-width:480px; text-align:center;\"><h1>Connection failed</h1><p>{}</p><p>You can close this window and try again in Eterea.</p></div></body></html>",
        html_escape(message)
    )
}

fn html_escape(message: &str) -> String {
    message
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

#[derive(Debug, Deserialize)]
struct TokenResponse {
    access_token: String,
    expires_in: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct UserLookupResponse {
    data: ApiUser,
}

#[derive(Debug, Clone, Deserialize)]
struct ApiUser {
    id: String,
    username: String,
    name: String,
    #[serde(default)]
    profile_image_url: Option<String>,
}

#[derive(Debug, Deserialize)]
struct BookmarksResponse {
    #[serde(default)]
    data: Option<Vec<ApiTweet>>,
    #[serde(default)]
    includes: Option<ApiIncludes>,
    #[serde(default)]
    meta: Option<ApiMeta>,
}

#[derive(Debug, Deserialize)]
struct ApiIncludes {
    #[serde(default)]
    users: Option<Vec<ApiUser>>,
    #[serde(default)]
    media: Option<Vec<ApiMedia>>,
}

#[derive(Debug, Deserialize)]
struct ApiMeta {
    #[serde(default)]
    next_token: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ApiTweet {
    id: String,
    text: String,
    #[serde(default)]
    created_at: Option<String>,
    #[serde(default)]
    author_id: Option<String>,
    #[serde(default)]
    entities: Option<ApiEntities>,
    #[serde(default)]
    attachments: Option<ApiAttachments>,
}

#[derive(Debug, Deserialize)]
struct ApiEntities {
    #[serde(default)]
    hashtags: Option<Vec<ApiHashtag>>,
}

#[derive(Debug, Deserialize)]
struct ApiHashtag {
    tag: String,
}

#[derive(Debug, Deserialize)]
struct ApiAttachments {
    #[serde(default)]
    media_keys: Option<Vec<String>>,
}

#[derive(Debug, Clone, Deserialize)]
struct ApiMedia {
    media_key: String,
    #[serde(default)]
    url: Option<String>,
    #[serde(default)]
    preview_image_url: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn authorization_url_contains_expected_oauth_parameters() {
        let config = XClientConfig {
            client_id: Some("client-id".to_string()),
            redirect_uri: DEFAULT_REDIRECT_URI.to_string(),
            scopes: DEFAULT_SCOPES,
        };

        let url = build_authorization_url(&config, "client-id", "state-123", "challenge-456")
            .expect("auth url should build");
        let parsed = Url::parse(&url).expect("url should parse");
        let pairs: HashMap<_, _> = parsed.query_pairs().into_owned().collect();

        assert_eq!(pairs.get("client_id"), Some(&"client-id".to_string()));
        assert_eq!(pairs.get("state"), Some(&"state-123".to_string()));
        assert_eq!(pairs.get("code_challenge"), Some(&"challenge-456".to_string()));
        assert_eq!(pairs.get("code_challenge_method"), Some(&"S256".to_string()));
        assert_eq!(pairs.get("scope"), Some(&DEFAULT_SCOPES.to_string()));
    }

    #[test]
    fn maps_bookmark_payload_into_local_bookmark() {
        let tweet = ApiTweet {
            id: "123".to_string(),
            text: "hello #rust".to_string(),
            created_at: Some("2024-05-01T14:30:00Z".to_string()),
            author_id: Some("42".to_string()),
            entities: Some(ApiEntities {
                hashtags: Some(vec![ApiHashtag {
                    tag: "Rust".to_string(),
                }]),
            }),
            attachments: Some(ApiAttachments {
                media_keys: Some(vec!["media-1".to_string()]),
            }),
        };

        let mut users = HashMap::new();
        users.insert(
            "42".to_string(),
            ApiUser {
                id: "42".to_string(),
                username: "rustlang".to_string(),
                name: "Rust".to_string(),
                profile_image_url: Some("https://pbs.twimg.com/profile.jpg".to_string()),
            },
        );

        let mut media = HashMap::new();
        media.insert(
            "media-1".to_string(),
            ApiMedia {
                media_key: "media-1".to_string(),
                url: Some("https://pbs.twimg.com/media/test.jpg".to_string()),
                preview_image_url: None,
            },
        );

        let bookmark = map_tweet_to_bookmark(&tweet, &users, &media);
        assert_eq!(bookmark.author_handle, "rustlang");
        assert_eq!(bookmark.tweet_url, "https://x.com/rustlang/status/123");
        assert_eq!(bookmark.tags, vec!["rust"]);
        assert_eq!(bookmark.media.len(), 1);
    }
}

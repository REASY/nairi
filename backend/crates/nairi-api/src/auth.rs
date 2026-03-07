use std::time::{Duration, Instant};

use axum::http::{HeaderMap, header};
use base64::Engine as _;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use chrono::Utc;
use dashmap::DashMap;
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use reqwest::{Client, Url};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tracing::error;
use uuid::Uuid;

const GOOGLE_AUTHORIZE_URL: &str = "https://accounts.google.com/o/oauth2/v2/auth";
const GOOGLE_TOKEN_URL: &str = "https://oauth2.googleapis.com/token";
const GOOGLE_TOKEN_INFO_URL: &str = "https://oauth2.googleapis.com/tokeninfo";
const OAUTH_STATE_TTL: Duration = Duration::from_secs(10 * 60);

#[derive(Debug, Clone)]
pub struct AuthSettings {
    pub google_client_id: String,
    pub google_client_secret: String,
    pub google_redirect_uri: String,
    pub allowed_google_hosted_domain: Option<String>,
    pub post_login_redirect_url: String,
    pub session_cookie_name: String,
    pub session_cookie_secure: bool,
    pub session_cookie_domain: Option<String>,
    pub session_ttl_seconds: usize,
    pub session_signing_key: String,
}

impl AuthSettings {
    pub fn from_env() -> Result<Self, String> {
        let google_client_id = read_required_env("GOOGLE_OAUTH_CLIENT_ID")?;
        let google_client_secret = read_required_env("GOOGLE_OAUTH_CLIENT_SECRET")?;
        let google_redirect_uri = read_required_env("GOOGLE_OAUTH_REDIRECT_URI")?;
        let session_signing_key = read_required_env("SESSION_SIGNING_KEY")?;

        let allowed_google_hosted_domain = std::env::var("ALLOWED_GOOGLE_HOSTED_DOMAIN")
            .ok()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty());

        let post_login_redirect_url = std::env::var("AUTH_POST_LOGIN_REDIRECT_URL")
            .unwrap_or_else(|_| "http://localhost:5173/".to_string());

        let session_cookie_name =
            std::env::var("SESSION_COOKIE_NAME").unwrap_or_else(|_| "nairi_session".to_string());

        let session_cookie_domain = std::env::var("SESSION_COOKIE_DOMAIN")
            .ok()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty());

        let session_ttl_seconds = std::env::var("SESSION_TTL_SECONDS")
            .ok()
            .and_then(|value| value.parse::<usize>().ok())
            .unwrap_or(8 * 60 * 60);

        let session_cookie_secure = std::env::var("SESSION_COOKIE_SECURE")
            .ok()
            .map(|value| parse_bool_env(&value))
            .unwrap_or_else(|| !google_redirect_uri.starts_with("http://"));

        Ok(Self {
            google_client_id,
            google_client_secret,
            google_redirect_uri,
            allowed_google_hosted_domain,
            post_login_redirect_url,
            session_cookie_name,
            session_cookie_secure,
            session_cookie_domain,
            session_ttl_seconds,
            session_signing_key,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthUser {
    pub sub: String,
    pub email: String,
    pub name: Option<String>,
    pub picture: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct GoogleCallbackQuery {
    pub code: Option<String>,
    pub state: Option<String>,
    pub error: Option<String>,
    pub error_description: Option<String>,
}

#[derive(Debug)]
pub enum AuthError {
    InvalidRequest(&'static str),
    Unauthorized(&'static str),
    Upstream(&'static str),
    Internal(&'static str),
}

impl AuthError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::InvalidRequest(code) => code,
            Self::Unauthorized(code) => code,
            Self::Upstream(code) => code,
            Self::Internal(code) => code,
        }
    }
}

#[derive(Debug, Clone)]
struct PendingAuthorization {
    nonce: String,
    code_verifier: String,
    created_at: Instant,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SessionClaims {
    sub: String,
    email: String,
    name: Option<String>,
    picture: Option<String>,
    iat: usize,
    exp: usize,
}

#[derive(Debug, Deserialize)]
struct GoogleTokenResponse {
    id_token: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GoogleTokenInfo {
    iss: Option<String>,
    aud: Option<String>,
    exp: Option<String>,
    sub: Option<String>,
    email: Option<String>,
    email_verified: Option<String>,
    name: Option<String>,
    picture: Option<String>,
    nonce: Option<String>,
    hd: Option<String>,
}

#[derive(Clone)]
pub struct AuthService {
    settings: AuthSettings,
    client: Client,
    pending: std::sync::Arc<DashMap<String, PendingAuthorization>>,
}

impl AuthService {
    pub fn new(settings: AuthSettings) -> Self {
        Self {
            settings,
            client: Client::new(),
            pending: std::sync::Arc::new(DashMap::new()),
        }
    }

    pub fn post_login_redirect_url(&self) -> &str {
        &self.settings.post_login_redirect_url
    }

    pub fn begin_google_login(&self) -> Result<String, AuthError> {
        self.prune_pending_authorizations();

        let state = Uuid::new_v4().simple().to_string();
        let nonce = Uuid::new_v4().simple().to_string();
        let nonce_for_query = nonce.clone();
        let code_verifier = format!("{}{}", Uuid::new_v4().simple(), Uuid::new_v4().simple());
        let code_challenge = pkce_code_challenge(&code_verifier);

        self.pending.insert(
            state.clone(),
            PendingAuthorization {
                nonce,
                code_verifier,
                created_at: Instant::now(),
            },
        );

        let mut url = Url::parse(GOOGLE_AUTHORIZE_URL)
            .map_err(|_| AuthError::Internal("invalid_google_authorize_url"))?;
        url.query_pairs_mut()
            .append_pair("response_type", "code")
            .append_pair("client_id", &self.settings.google_client_id)
            .append_pair("redirect_uri", &self.settings.google_redirect_uri)
            .append_pair("scope", "openid email profile")
            .append_pair("state", &state)
            .append_pair("nonce", &nonce_for_query)
            .append_pair("code_challenge", &code_challenge)
            .append_pair("code_challenge_method", "S256");

        Ok(url.to_string())
    }

    pub async fn complete_google_login(
        &self,
        query: GoogleCallbackQuery,
    ) -> Result<AuthUser, AuthError> {
        if query.error.is_some() {
            return Err(AuthError::InvalidRequest("google_oauth_error"));
        }

        let state = query
            .state
            .as_deref()
            .ok_or(AuthError::InvalidRequest("missing_oauth_state"))?;
        let code = query
            .code
            .as_deref()
            .ok_or(AuthError::InvalidRequest("missing_oauth_code"))?;

        let pending = self
            .pending
            .remove(state)
            .map(|(_, value)| value)
            .ok_or(AuthError::InvalidRequest("invalid_oauth_state"))?;

        if pending.created_at.elapsed() > OAUTH_STATE_TTL {
            return Err(AuthError::InvalidRequest("expired_oauth_state"));
        }

        let id_token = self
            .exchange_google_code(code, &pending.code_verifier)
            .await?
            .id_token
            .ok_or(AuthError::Upstream("missing_google_id_token"))?;

        let token_info = self.fetch_google_token_info(&id_token).await?;
        self.validate_google_token_info(&token_info, &pending.nonce)?;

        Ok(AuthUser {
            sub: token_info
                .sub
                .ok_or(AuthError::Upstream("missing_google_sub"))?,
            email: token_info
                .email
                .ok_or(AuthError::Upstream("missing_google_email"))?,
            name: token_info.name,
            picture: token_info.picture,
        })
    }

    pub fn issue_session_cookie(&self, user: &AuthUser) -> Result<String, AuthError> {
        let now = unix_timestamp();
        let claims = SessionClaims {
            sub: user.sub.clone(),
            email: user.email.clone(),
            name: user.name.clone(),
            picture: user.picture.clone(),
            iat: now,
            exp: now + self.settings.session_ttl_seconds,
        };

        let token = encode(
            &Header::new(Algorithm::HS256),
            &claims,
            &EncodingKey::from_secret(self.settings.session_signing_key.as_bytes()),
        )
            .map_err(|error| {
                error!(error = %error, "failed to sign session token");
                AuthError::Internal("session_signing_failed")
            })?;

        Ok(self.make_session_cookie(&token, self.settings.session_ttl_seconds))
    }

    pub fn clear_session_cookie(&self) -> String {
        self.make_session_cookie("", 0)
    }

    pub fn authenticate_request(&self, headers: &HeaderMap) -> Result<AuthUser, AuthError> {
        let token = extract_cookie(headers, &self.settings.session_cookie_name)
            .ok_or(AuthError::Unauthorized("missing_session_cookie"))?;

        let validation = Validation::new(Algorithm::HS256);
        let claims = decode::<SessionClaims>(
            &token,
            &DecodingKey::from_secret(self.settings.session_signing_key.as_bytes()),
            &validation,
        )
            .map_err(|_| AuthError::Unauthorized("invalid_session_cookie"))?
            .claims;

        Ok(AuthUser {
            sub: claims.sub,
            email: claims.email,
            name: claims.name,
            picture: claims.picture,
        })
    }

    async fn exchange_google_code(
        &self,
        code: &str,
        code_verifier: &str,
    ) -> Result<GoogleTokenResponse, AuthError> {
        let response = self
            .client
            .post(GOOGLE_TOKEN_URL)
            .form(&[
                ("grant_type", "authorization_code"),
                ("code", code),
                ("client_id", self.settings.google_client_id.as_str()),
                ("client_secret", self.settings.google_client_secret.as_str()),
                ("redirect_uri", self.settings.google_redirect_uri.as_str()),
                ("code_verifier", code_verifier),
            ])
            .send()
            .await
            .map_err(|error| {
                error!(error = %error, "google token exchange request failed");
                AuthError::Upstream("google_token_exchange_failed")
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response
                .text()
                .await
                .unwrap_or_else(|_| "<unreadable body>".to_string());
            error!(status = %status, body = %body, "google token exchange rejected");
            return Err(AuthError::Upstream("google_token_exchange_rejected"));
        }

        response
            .json::<GoogleTokenResponse>()
            .await
            .map_err(|error| {
                error!(error = %error, "failed to decode google token response");
                AuthError::Upstream("invalid_google_token_response")
            })
    }

    async fn fetch_google_token_info(&self, id_token: &str) -> Result<GoogleTokenInfo, AuthError> {
        let response = self
            .client
            .get(GOOGLE_TOKEN_INFO_URL)
            .query(&[("id_token", id_token)])
            .send()
            .await
            .map_err(|error| {
                error!(error = %error, "google token info request failed");
                AuthError::Upstream("google_tokeninfo_failed")
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response
                .text()
                .await
                .unwrap_or_else(|_| "<unreadable body>".to_string());
            error!(status = %status, body = %body, "google token info rejected");
            return Err(AuthError::Upstream("google_tokeninfo_rejected"));
        }

        response.json::<GoogleTokenInfo>().await.map_err(|error| {
            error!(error = %error, "failed to decode google token info response");
            AuthError::Upstream("invalid_google_tokeninfo_response")
        })
    }

    fn validate_google_token_info(
        &self,
        token_info: &GoogleTokenInfo,
        expected_nonce: &str,
    ) -> Result<(), AuthError> {
        let issuer = token_info
            .iss
            .as_deref()
            .ok_or(AuthError::Upstream("missing_google_issuer"))?;
        if issuer != "https://accounts.google.com" && issuer != "accounts.google.com" {
            return Err(AuthError::Unauthorized("invalid_google_issuer"));
        }

        let audience = token_info
            .aud
            .as_deref()
            .ok_or(AuthError::Upstream("missing_google_audience"))?;
        if audience != self.settings.google_client_id {
            return Err(AuthError::Unauthorized("invalid_google_audience"));
        }

        let nonce = token_info
            .nonce
            .as_deref()
            .ok_or(AuthError::Unauthorized("missing_google_nonce"))?;
        if nonce != expected_nonce {
            return Err(AuthError::Unauthorized("invalid_google_nonce"));
        }

        let exp = token_info
            .exp
            .as_deref()
            .ok_or(AuthError::Upstream("missing_google_expiration"))?
            .parse::<usize>()
            .map_err(|_| AuthError::Upstream("invalid_google_expiration"))?;

        if exp <= unix_timestamp() {
            return Err(AuthError::Unauthorized("expired_google_token"));
        }

        let verified_email = token_info
            .email_verified
            .as_deref()
            .map(parse_bool_string)
            .unwrap_or(false);
        if !verified_email {
            return Err(AuthError::Unauthorized("google_email_not_verified"));
        }

        if let Some(allowed_domain) = self.settings.allowed_google_hosted_domain.as_deref() {
            let hosted_domain = token_info
                .hd
                .as_deref()
                .ok_or(AuthError::Unauthorized("missing_google_hosted_domain"))?;
            if hosted_domain != allowed_domain {
                return Err(AuthError::Unauthorized("invalid_google_hosted_domain"));
            }
        }

        Ok(())
    }

    fn make_session_cookie(&self, value: &str, max_age: usize) -> String {
        let mut cookie = format!(
            "{}={}; Path=/; HttpOnly; SameSite=Lax; Max-Age={}",
            self.settings.session_cookie_name, value, max_age
        );

        if self.settings.session_cookie_secure {
            cookie.push_str("; Secure");
        }

        if let Some(domain) = self.settings.session_cookie_domain.as_deref() {
            cookie.push_str("; Domain=");
            cookie.push_str(domain);
        }

        cookie
    }

    fn prune_pending_authorizations(&self) {
        self.pending
            .retain(|_, pending| pending.created_at.elapsed() < OAUTH_STATE_TTL);
    }
}

fn parse_bool_env(value: &str) -> bool {
    matches!(
        value.trim().to_ascii_lowercase().as_str(),
        "1" | "true" | "yes" | "on"
    )
}

fn parse_bool_string(value: &str) -> bool {
    matches!(
        value.trim().to_ascii_lowercase().as_str(),
        "1" | "true" | "yes"
    )
}

fn pkce_code_challenge(code_verifier: &str) -> String {
    let digest = Sha256::digest(code_verifier.as_bytes());
    URL_SAFE_NO_PAD.encode(digest)
}

fn read_required_env(name: &str) -> Result<String, String> {
    std::env::var(name).map_err(|_| format!("Missing required environment variable: {name}"))
}

fn unix_timestamp() -> usize {
    Utc::now().timestamp().max(0) as usize
}

fn extract_cookie(headers: &HeaderMap, cookie_name: &str) -> Option<String> {
    for header_value in headers.get_all(header::COOKIE) {
        let raw_value = header_value.to_str().ok()?;
        for part in raw_value.split(';') {
            let mut pieces = part.trim().splitn(2, '=');
            let name = pieces.next()?.trim();
            let value = pieces.next()?.trim();
            if name == cookie_name {
                return Some(value.to_string());
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::parse_bool_env;

    #[test]
    fn parse_bool_env_understands_common_values() {
        assert!(parse_bool_env("true"));
        assert!(parse_bool_env("1"));
        assert!(parse_bool_env("YES"));
        assert!(!parse_bool_env("false"));
        assert!(!parse_bool_env("0"));
    }
}

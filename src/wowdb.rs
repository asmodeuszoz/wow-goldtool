//! HTTP client for TheWoWDB: URL building, conditional GETs and rate pacing.
//!
//! Rate model (docs/API.md): a burst of 120 requests per IP, refilled at 2 requests per
//! second; past that the API answers 429 with Retry-After and reports
//! X-RateLimit-Remaining. The pacing below keeps a headroom under the burst and then
//! spaces requests on a fixed schedule, so the app never plans on the whole burst.
//!
//! This module never touches the database and never parses an item payload: it hands
//! back the raw response plus the headers the sync engine needs. sync.rs is the only
//! place where HTTP and storage meet (docs/DEVELOPMENT.md module map).

use crate::errors::AppError;
use crate::errors::AppResult;

pub use reqwest::blocking::Client;
use reqwest::blocking::ClientBuilder;
use reqwest::blocking::RequestBuilder;
pub use reqwest::blocking::Response;
pub use reqwest::header::HeaderMap;
pub use reqwest::header::HeaderName;

use std::thread::sleep;
use std::time::Duration;
use std::time::Instant;
use std::env::var;

/// Endpoint root. The api/v1 segment is pinned on purpose: the source ships breaking
/// changes under a new api/v2 path, so an upgrade must never happen silently.
pub const API_BASE: &str = "https://thewowdb.com";
pub const API_VERSION: &str = "forever";

/// Identify ourselves: the terms ask for an honest User-Agent, and it is what a polite
/// crawl looks like in the upstream access log.
pub const USER_AGENT: &str = concat!("wow-goldtool ", env!("CARGO_PKG_VERSION"), "; data source: thewowdb.com");

/// How many requests may fire back to back before the schedule kicks in. The documented
/// burst is 120 per IP; we keep 30 of headroom so a parallel GUI worker cannot exhaust it.
pub const BURST_HEADROOM: u64 = 90;
/// Spacing of the steady schedule, milliseconds per request (2 requests per second).
pub const PER_REQUEST_MS: u64 = 500;
/// What we do when the API answered 429 and did not say how long to wait.
pub const FALLBACK_RETRY_MS: u64 = 2000;

/// Env switch: tests and CI point the crawler at a local fixture server so they
/// never hammer the upstream API (docs/DEVELOPMENT.md).
pub const ENV_FIXTURE_KEY: &str = "WGT_FIXTURE";

/// Where the URLs of this process point. Upstream unless the fixture switch is set.
pub fn base_url() -> String {
    match var(ENV_FIXTURE_KEY) {
        Err(_) => return API_BASE.to_string(),
        Ok(value) => return value,
    }
}

/// One page of the item list endpoint, page numbers are 1-based.
pub fn page_url(page: u32, per_page: u16) -> String {
    return format!("{}/api/v1/{}/item/?page={}&per_page={}", base_url(), API_VERSION, page, per_page);
}

/// One record, e.g. id 25 is "Worn Shortsword".
pub fn item_url(id: i64) -> String {
    return format!("{}/api/v1/{}/item/{}", base_url(), API_VERSION, id);
}

/// Cross-kind live search over items, spells, quests, npcs, zones and sounds.
pub fn search_url(query: String) -> String {
    return format!("{}/api/v1/{}/search/?q={}", base_url(), API_VERSION, query);
}

/// Pacing state for one crawl run. Plain data so the sync loop can log it and the CLI
/// can print it.
pub struct RateBudget {
    pub burst: u64,
    pub per_request_ms: u64,
    pub sent: u64,
    pub started: Instant,
}

impl RateBudget {
    pub fn new() -> RateBudget {
        return RateBudget {
            burst: BURST_HEADROOM,
            per_request_ms: PER_REQUEST_MS,
            sent: 0,
            started: Instant::now(),
        };
    }

    /// Sleep until the schedule says the next request is legal, then count it.
    pub fn wait_and_count(&mut self) {
        self.sent += 1;
        if self.sent <= self.burst {
            return;
        }
        let due: Duration = Duration::from_millis(self.per_request_ms * (self.sent - self.burst));
        let passed: Duration = self.started.elapsed();
        if passed < due {
            sleep(due - passed);
        }
    }

    /// Slow the schedule down after a 429: the caller sleeps the Retry-After and we
    /// re-anchor the schedule so the rest of the run stays spaced out.
    pub fn penalize(&mut self, millis: u64) {
        self.per_request_ms += millis;
    }
}

/// Build the shared HTTP client. Keep-alive pooling matters: a crawl is hundreds of
/// requests against one host.
pub fn build_client() -> AppResult<Client> {
    let builder = ClientBuilder::new().user_agent(USER_AGENT);
    return match builder.build() {
        Ok(client) => Ok(client),
        Err(problem) => Err(AppError::network(format!("{}", problem))),
    };
}

/// A response normalised for the sync engine and the live lookups.
pub struct ApiResponse {
    pub status: u16,
    pub body: String,
    pub etag: String,
    pub retry_after: Option<u32>,
    pub remaining: Option<u32>,
    pub unchanged: bool,
}

/// Read one response header as plain text; missing or non-text headers read as empty.
pub fn header_text(map: &HeaderMap, name: HeaderName) -> String {
    let found = map.get(name);
    return match found {
        Some(value) => match value.to_str() {
            Ok(text) => text.to_string(),
            Err(_) => "".to_string(),
        },
        None => "".to_string(),
    };
}

/// ETag for conditional revalidation (If-None-Match).
pub fn etag_of(map: &HeaderMap) -> String {
    return header_text(map, HeaderName::from_static("etag"));
}

/// Seconds the API asked us to wait after a 429.
pub fn retry_after_of(map: &HeaderMap) -> Option<u32> {
    return parse_count(header_text(map, HeaderName::from_static("retry-after")));
}

/// Requests left in the current burst window, when the API reports it.
pub fn remaining_of(map: &HeaderMap) -> Option<u32> {
    return parse_count(header_text(map, HeaderName::from_static("x-ratelimit-remaining")));
}

/// Parse a header that should hold a small unsigned integer; anything else is None.
pub fn parse_count(raw: String) -> Option<u32> {
    return match raw.trim().parse() {
        Ok(value) => Some(value),
        Err(_) => None,
    };
}

/// Classify an HTTP status against the documented rate model, so sync.rs can tell a
/// pause (429) apart from a real failure.
pub enum HttpKind {
    Success,
    NotModified,
    RateLimited,
    /// The upstream has no record for this id (404). Not a failure: a removed
    /// record is a fact about the catalogue, so the crawl keeps going (v0.5.3).
    Missing,
    Failed,
}

pub fn classify(status: u16) -> HttpKind {
    if status == 200 {
        return HttpKind::Success;
    }
    if status == 304 {
        return HttpKind::NotModified;
    }
    if status == 429 {
        return HttpKind::RateLimited;
    }
    if status == 404 {
        return HttpKind::Missing;
    }
    return HttpKind::Failed;
}

/// GET one URL with optional ETag revalidation. Pacing is the caller's job: it must
/// have called wait_and_count for this request already.
pub fn get_json(client: Client, url: String, etag: String) -> AppResult<ApiResponse> {
    let base = client.get(url);
    let request: RequestBuilder = if etag == "" {
        base
    } else {
        base.header("If-None-Match", etag)
    };

    let response: Response = match request.send() {
        Ok(value) => value,
        Err(problem) => return Err(AppError::network(format!("{}", problem))),
    };

    let status: u16 = response.status().as_u16();
    let headers = response.headers();
    let etag_new: String = etag_of(headers);
    let retry: Option<u32> = retry_after_of(headers);
    let left: Option<u32> = remaining_of(headers);
    let body: String = match response.text() {
        Ok(text) => text,
        Err(problem) => return Err(AppError::network(format!("{}", problem))),
    };

    return Ok(ApiResponse {
        status,
        body,
        etag: etag_new,
        retry_after: retry,
        remaining: left,
        unchanged: status == 304,
    });
}

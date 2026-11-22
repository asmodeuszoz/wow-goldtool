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

/// The tooltip door (v0.8.0, ADR 0010, docs/API.md): the description text the
/// catalogue wants lives HERE, not on the detail record (measured: the forever/
/// item body carries no description field, and the tooltip body answers `html`
/// with the card's own markup, incl. `Sell Price: 7c`). The URL is the one docs/
/// API.md prints and the probe asked: `/api/v1/tooltip/<version>/item/<id>`. Its
/// 404 answers upstream's own JSON (`No item … in forever.`), so the stage gives
/// the checked third state, exactly like the detail and icon doors do.
pub fn tooltip_url(id: i64) -> String {
    return format!("{}/api/v1/tooltip/{}/item/{}", base_url(), API_VERSION, id);
}

/// The SECOND source's door (v0.10.0, ADR 0021): the recipe files ride raw
///.githubusercontent.com - the MIT dataset whose text is measured at the repo's
/// addon/LICENSE and whose shapes docs/API.md's second-source section records.
/// This host is NOT the rate-limited API: the politeness schedule still rides
/// (one host, one courtesy - the icon stage set the precedent for hosts beyond
/// the API), but the burst headroom charges the same budget. The fixture re-base
/// keeps the SAME suffix: a fixture answering `/data/<file>.json` speaks the
/// live path shape, so CI's smoke cannot drift from what the wire really is.
pub const RECIPE_SOURCE_BASE: &str = "https://raw.githubusercontent.com/Nyte-fyre/ReagentRoute/main";
pub fn recipe_file_url(file: String) -> String {
    let base: String = match var(ENV_FIXTURE_KEY) {
        Err(_) => RECIPE_SOURCE_BASE.to_string(),
        Ok(value) => value,
    };
    return format!("{}/data/{}", base, file);
}

/// Cross-kind live search over items, spells, quests, npcs, zones and sounds.
/// The query is percent-encoded on the way in: the live probe asked with
/// `?q=Worn%20Shortsword` and got a 200 back (docs/API.md measures it), and a
/// raw space is not a URL character. Upstream's own `more` URLs use `+` for
/// spaces, but those arrive INSIDE the answer and stay verbatim (the door prints
/// the answer as it came).
pub fn search_url(query: String) -> String {
    return format!("{}/api/v1/{}/search/?q={}", base_url(), API_VERSION, encode_query(query));
}

/// Percent-encode one query value: letters, digits and the three harmless
/// punctuation marks ride through; a space rides as `%20` like the measured
/// probe; everything else rides as its UTF-8 bytes, two hex digits each. The
/// rule is RFC-3911-lite for a query value, and the measured ask grounds the
/// shape: `?q=Worn%20Shortsword` answered 200 (docs/API.md).
pub fn encode_query(text: String) -> String {
    let mut out: String = "".to_string();
    for one in text.chars() {
        if (one >= 'a' && one <= 'z') || (one >= 'A' && one <= 'Z') || (one >= '0' && one <= '9') || one == '-' || one == '.' || one == '_' {
            out = format!("{}{}", out, one);
            continue;
        }
        if one == ' ' {
            out = format!("{}{}", out, "%20");
            continue;
        }
        for byte in one.to_string().as_bytes() {
            out = format!("{}{}", out, format!("%{:02X}", byte));
        }
    }
    return out;
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

/// Resolve a stored icon_url into something fetchable (v0.6.0). Shapes are measured,
/// not guessed (docs/API.md): every answered detail record's `icon_url` is ABSOLUTE on
/// one of two hosts (`render.worldofwarcraft.com/...` or `thewowdb.com/assets/icons/...`),
/// and the fixture's payloads use the host-root-relative form (`/icon/...`) against the
/// server this process already points at. Anything else resolves to None: inventing a
/// host for a half-formed shape is not our call to make, and the stage answers "nothing
/// to fetch" for those instead of a check-mark.
pub fn icon_target(stored: Option<String>) -> Option<String> {
    match stored {
        None => return None,
        Some(text) => {
            if text.starts_with("http") {
                return Some(text);
            }
            if text.starts_with("/") {
                return Some(format!("{}{}", base_url(), text));
            }
            return None;
        },
    }
}

/// A binary response (icon bytes): ApiResponse's body is TEXT because every JSON
/// endpoint speaks text - icon files are not text and must not go through a lossy
/// decode. No ETag handling here on purpose: v0.6.0 copies an icon exactly ONCE
/// (the pending rule excludes rows that already have `icon_at`), so there is no
/// revalidation pass whose stored copy this could invalidate.
pub struct BinaryResponse {
    pub status: u16,
    pub bytes: Vec<u8>,
    pub retry_after: Option<u32>,
}

/// GET one URL for bytes. Pacing is the caller's job like get_json.
/// bytes() hands back a bytes::Bytes and `From<Bytes> for Vec<u8>` is defined in the
/// crate (verified against the pinned source) - that conversion is the copy.
pub fn get_bytes(client: Client, url: String) -> AppResult<BinaryResponse> {
    let response: Response = match client.get(url).send() {
        Err(problem) => return Err(AppError::network(format!("{}", problem))),
        Ok(value) => value,
    };
    let status: u16 = response.status().as_u16();
    let headers = response.headers();
    let retry: Option<u32> = retry_after_of(headers);
    let payload: Vec<u8> = match response.bytes() {
        Err(problem) => return Err(AppError::network(format!("{}", problem))),
        Ok(raw) => Vec::from(raw),
    };
    return Ok(BinaryResponse {
        status,
        bytes: payload,
        retry_after: retry,
    });
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

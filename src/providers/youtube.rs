use crate::model::{Track, TrackSource};
use serde_json::{Value, json};
use std::{
    hash::{DefaultHasher, Hash, Hasher},
    sync::{LazyLock, RwLock},
};

const PARAMS_SONGS: &str = "EgWKAQIIAWoSEAQQAxAFEBAQChAJEBUQDhAR";
const PARAMS_VIDEOS: &str = "EgWKAQIQAWoSEAQQAxAFEBAQChAJEBUQDhAR";

const INNERTUBE_SEARCH_URL: &str = "https://music.youtube.com/youtubei/v1/search";
const INNERTUBE_GUIDE_URL: &str = "https://music.youtube.com/youtubei/v1/guide";
const INNERTUBE_PLAYER_URL: &str = "https://www.youtube.com/youtubei/v1/player?prettyPrint=false";
const VISIONOS_USER_AGENT: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 15_7_3) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/26.0 Safari/605.1.15";

static HTTP_CLIENT: LazyLock<reqwest::blocking::Client> =
    LazyLock::new(reqwest::blocking::Client::new);

static VISITOR_DATA: LazyLock<RwLock<Option<String>>> = LazyLock::new(|| RwLock::new(None));

fn get_or_fetch_visitor_data() -> String {
    if let Ok(guard) = VISITOR_DATA.read() {
        if let Some(ref vd) = *guard {
            return vd.clone();
        }
    }

    let body = json!({
        "context": {
            "client": {
                "clientName": "WEB_REMIX",
                "clientVersion": "1.20260707.12.00",
                "hl": "en",
                "gl": "US"
            }
        }
    });

    if let Ok(res) = HTTP_CLIENT
        .post(INNERTUBE_GUIDE_URL)
        .header("Content-Type", "application/json")
        .header("User-Agent", "Mozilla/5.0")
        .json(&body)
        .send()
    {
        if let Ok(val) = res.json::<Value>() {
            if let Some(vd) = val
                .pointer("/responseContext/visitorData")
                .and_then(|v| v.as_str())
            {
                let vd_str = vd.to_string();
                if let Ok(mut guard) = VISITOR_DATA.write() {
                    *guard = Some(vd_str.clone());
                }
                return vd_str;
            }
        }
    }

    String::new()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SearchFilter {
    #[default]
    All,
    Songs,
    Videos,
}

pub fn search_all(query: &str) -> Result<Vec<Track>, Box<dyn std::error::Error + Send + Sync>> {
    search_filtered(query, SearchFilter::All)
}

pub fn search_filtered(
    query: &str,
    filter: SearchFilter,
) -> Result<Vec<Track>, Box<dyn std::error::Error + Send + Sync>> {
    match filter {
        SearchFilter::Songs => search_with_params(query, Some(PARAMS_SONGS)),
        SearchFilter::Videos => search_with_params(query, Some(PARAMS_VIDEOS)),
        SearchFilter::All => {
            let mut tracks = Vec::new();
            let mut seen_ids = std::collections::HashSet::new();

            // Fetch songs first
            if let Ok(song_tracks) = search_with_params(query, Some(PARAMS_SONGS)) {
                for t in song_tracks {
                    if let TrackSource::Stream { ref id, .. } = t.source {
                        if seen_ids.insert(id.clone()) {
                            tracks.push(t);
                        }
                    }
                }
            }

            // Fetch videos next
            if let Ok(video_tracks) = search_with_params(query, Some(PARAMS_VIDEOS)) {
                for t in video_tracks {
                    if let TrackSource::Stream { ref id, .. } = t.source {
                        if seen_ids.insert(id.clone()) {
                            tracks.push(t);
                        }
                    }
                }
            }

            // Fallback if both returned nothing
            if tracks.is_empty() {
                if let Ok(fallback_tracks) = search_with_params(query, None) {
                    for t in fallback_tracks {
                        if let TrackSource::Stream { ref id, .. } = t.source {
                            if seen_ids.insert(id.clone()) {
                                tracks.push(t);
                            }
                        }
                    }
                }
            }

            Ok(tracks)
        }
    }
}

fn search_with_params(
    query: &str,
    params: Option<&str>,
) -> Result<Vec<Track>, Box<dyn std::error::Error + Send + Sync>> {
    let mut body = json!({
        "context": {
            "client": {
                "clientName": "WEB_REMIX",
                "clientVersion": "1.20260707.12.00",
                "hl": "en",
                "gl": "US"
            }
        },
        "query": query,
    });

    if let Some(p) = params {
        body["params"] = json!(p);
    }

    let response: Value = HTTP_CLIENT
        .post(INNERTUBE_SEARCH_URL)
        .header("Content-Type", "application/json")
        .header(
            "User-Agent",
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:128.0) Gecko/20100101 Firefox/128.0",
        )
        .json(&body)
        .send()?
        .json()?;

    if let Some(vd) = response
        .pointer("/responseContext/visitorData")
        .and_then(|v| v.as_str())
    {
        if let Ok(mut guard) = VISITOR_DATA.write() {
            *guard = Some(vd.to_string());
        }
    }

    let mut tracks = Vec::new();

    let mut renderers = Vec::new();
    collect_renderers(&response, &mut renderers);

    for renderer in renderers {
        let video_id = renderer
            .pointer("/playlistItemData/videoId")
            .and_then(|v| v.as_str())
            .unwrap_or_default();

        if video_id.is_empty() {
            continue;
        }

        // Title
        let title = renderer
            .pointer("/flexColumns/0/musicResponsiveListItemFlexColumnRenderer/text/runs/0/text")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown Title")
            .to_string();

        // Artist, Album & Duration
        let mut artist = String::new();
        let mut album = String::new();
        let mut duration_secs = 0u32;

        if let Some(runs) = renderer
            .pointer("/flexColumns/1/musicResponsiveListItemFlexColumnRenderer/text/runs")
            .and_then(|v| v.as_array())
        {
            let mut fields: Vec<String> = vec![String::new()];
            for r in runs {
                if let Some(txt) = r.get("text").and_then(|t| t.as_str()) {
                    if txt.trim() == "•" {
                        fields.push(String::new());
                    } else if let Some(last) = fields.last_mut() {
                        last.push_str(txt);
                    }
                }
            }

            for field in fields {
                let trimmed = field.trim();
                if trimmed.is_empty() || trimmed == "Song" || trimmed == "Video" {
                    continue;
                }
                if trimmed.contains(':')
                    && duration_secs == 0
                    && parse_duration_to_secs(trimmed) > 0
                {
                    duration_secs = parse_duration_to_secs(trimmed);
                } else if artist.is_empty()
                    && !trimmed.ends_with("views")
                    && !trimmed.ends_with("plays")
                {
                    artist = trimmed.to_string();
                } else if album.is_empty()
                    && !trimmed.ends_with("views")
                    && !trimmed.ends_with("plays")
                {
                    album = trimmed.to_string();
                }
            }
        }

        // Fallback for the Artist
        if artist.is_empty() {
            if let Some(label) = renderer
                .pointer("/overlay/musicItemThumbnailOverlayRenderer/content/musicPlayButtonRenderer/accessibilityPlayData/accessibilityData/label")
                .and_then(|v| v.as_str())
            {
                if let Some((_, artist_part)) = label.split_once(" - ") {
                    artist = artist_part.trim().to_string();
                }
            }
        }

        if artist.is_empty() {
            artist = "YouTube Music".to_string();
        }
        if album.is_empty() {
            album = title.clone();
        }

        let mut hasher = DefaultHasher::new();
        video_id.hash(&mut hasher);
        let id = hasher.finish();

        tracks.push(Track {
            id,
            title,
            artist,
            album,
            duration_secs,
            source: TrackSource::Stream {
                provider: "youtube".to_string(),
                id: video_id.to_string(),
                direct_url: None,
                hint_extension: "webm".to_string(),
            },
        });
    }

    Ok(tracks)
}

fn collect_renderers<'a>(value: &'a Value, out: &mut Vec<&'a Value>) {
    match value {
        Value::Object(map) => {
            if let Some(r) = map.get("musicResponsiveListItemRenderer") {
                out.push(r);
            }
            for v in map.values() {
                collect_renderers(v, out);
            }
        }
        Value::Array(arr) => {
            for v in arr {
                collect_renderers(v, out);
            }
        }
        _ => {}
    }
}

fn parse_duration_to_secs(dur: &str) -> u32 {
    let parts: Vec<&str> = dur.split(':').collect();
    match parts.len() {
        2 => {
            let m = parts[0].parse::<u32>().unwrap_or(0);
            let s = parts[1].parse::<u32>().unwrap_or(0);
            m * 60 + s
        }
        3 => {
            let h = parts[0].parse::<u32>().unwrap_or(0);
            let m = parts[1].parse::<u32>().unwrap_or(0);
            let s = parts[2].parse::<u32>().unwrap_or(0);
            h * 3600 + m * 60 + s
        }
        _ => 0,
    }
}

pub fn get_stream_url(
    video_id: &str,
) -> Result<(String, Option<u64>), Box<dyn std::error::Error + Send + Sync>> {
    let visitor_data = get_or_fetch_visitor_data();

    let body = json!({
        "context": {
            "client": {
                "clientName": "VISIONOS",
                "clientVersion": "1.02",
                "deviceMake": "Apple",
                "deviceModel": "RealityDevice17,1",
                "userAgent": VISIONOS_USER_AGENT,
                "osName": "visionOS",
                "osVersion": "26.5.23O471",
                "visitorData": visitor_data,
                "hl": "en",
                "gl": "US"
            }
        },
        "videoId": video_id,
        "racyCheckOk": true,
        "contentCheckOk": true
    });

    let response: Value = HTTP_CLIENT
        .post(INNERTUBE_PLAYER_URL)
        .header("Content-Type", "application/json")
        .header("User-Agent", VISIONOS_USER_AGENT)
        .header("X-YouTube-Client-Name", "101")
        .header("X-YouTube-Client-Version", "1.02")
        .header("X-Goog-Visitor-Id", &visitor_data)
        .header("Origin", "https://www.youtube.com")
        .json(&body)
        .send()?
        .json()?;

    let formats = response
        .pointer("/streamingData/adaptiveFormats")
        .and_then(|v| v.as_array())
        .ok_or("No streaming formats found")?;

    let best_audio = formats
        .iter()
        .filter(|f| {
            f.pointer("/mimeType")
                .and_then(|m| m.as_str())
                .map(|m| m.starts_with("audio/webm"))
                .unwrap_or(false)
                && f.pointer("/url").is_some()
        })
        .max_by_key(|f| f.pointer("/bitrate").and_then(|b| b.as_u64()).unwrap_or(0))
        .or_else(|| {
            formats
                .iter()
                .filter(|f| {
                    f.pointer("/mimeType")
                        .and_then(|m| m.as_str())
                        .map(|m| m.starts_with("audio/"))
                        .unwrap_or(false)
                        && f.pointer("/url").is_some()
                })
                .max_by_key(|f| f.pointer("/bitrate").and_then(|b| b.as_u64()).unwrap_or(0))
        })
        .ok_or("No direct audio format available")?;

    let stream_url = best_audio
        .pointer("/url")
        .and_then(|u| u.as_str())
        .ok_or("Audio format missing direct stream URL")?;

    let content_length = best_audio
        .pointer("/contentLength")
        .and_then(|c| c.as_str())
        .and_then(|c| c.parse::<u64>().ok());

    Ok((stream_url.to_string(), content_length))
}

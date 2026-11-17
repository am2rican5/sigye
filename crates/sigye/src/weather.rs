//! Weather data fetching for dynamic weather background.
//!
//! Fetches weather data from wttr.in in a background thread. The background style
//! is resolved from that data whenever it is asked for, so dawn, day, dusk and
//! night follow the clock rather than the time of the last fetch.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, PoisonError, RwLock};
use std::thread;
use std::time::{Duration, Instant};

use chrono::Timelike;
use serde::Deserialize;
use sigye_core::{BackgroundStyle, TimeOfDay};

/// How long to wait after a successful fetch before fetching again.
const FETCH_INTERVAL: Duration = Duration::from_secs(30 * 60);

/// How long to wait after a failed fetch before trying again.
const RETRY_INTERVAL: Duration = Duration::from_secs(60);

/// How often the fetch thread wakes to check whether to stop or fetch.
const POLL_INTERVAL: Duration = Duration::from_secs(1);

/// Timeout for HTTP requests.
const REQUEST_TIMEOUT: Duration = Duration::from_secs(10);

/// Civil twilight duration in minutes (~30 minutes before sunrise / after sunset).
const CIVIL_TWILIGHT_MINUTES: u32 = 30;

/// Simplified weather condition categories.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WeatherCondition {
    Clear,
    PartlyCloudy,
    Cloudy,
    Rain,
    HeavyRain,
    Thunderstorm,
    Snow,
    Fog,
    Windy,
}

/// Parsed weather data from wttr.in API.
#[derive(Debug, Clone)]
pub struct WeatherData {
    /// Current weather condition.
    pub condition: WeatherCondition,
    /// Temperature in Celsius.
    pub temp_c: i32,
    /// Latitude (for aurora calculation).
    pub latitude: f32,
    /// Sunrise as wttr.in reports it (e.g., "06:45 AM").
    pub sunrise: Option<String>,
    /// Sunset as wttr.in reports it (e.g., "07:30 PM").
    pub sunset: Option<String>,
}

impl WeatherData {
    /// The background for this weather at `minutes` past local midnight.
    ///
    /// Sunrise and sunset are in the location's local time but are compared with
    /// this machine's clock, so a location in another time zone is misjudged.
    fn background_at(&self, minutes: u32) -> BackgroundStyle {
        let sunrise = self.sunrise.as_deref().and_then(parse_time_to_minutes);
        let sunset = self.sunset.as_deref().and_then(parse_time_to_minutes);
        let time_of_day = time_of_day(
            minutes,
            sunrise.unwrap_or(6 * 60),
            sunset.unwrap_or(18 * 60),
        );
        map_weather_to_background(self, time_of_day)
    }
}

impl Default for WeatherData {
    fn default() -> Self {
        Self {
            condition: WeatherCondition::Clear,
            temp_c: 20,
            latitude: 0.0,
            sunrise: None,
            sunset: None,
        }
    }
}

/// wttr.in JSON response structure (partial - only fields we need).
#[derive(Debug, Deserialize)]
struct WttrResponse {
    current_condition: Vec<CurrentCondition>,
    nearest_area: Option<Vec<NearestArea>>,
    weather: Option<Vec<DailyWeather>>,
}

#[derive(Debug, Deserialize)]
struct CurrentCondition {
    #[serde(rename = "weatherCode")]
    weather_code: String,
    #[serde(rename = "temp_C")]
    temp_c: String,
    #[serde(rename = "windspeedKmph")]
    windspeed_kmph: String,
}

#[derive(Debug, Deserialize)]
struct NearestArea {
    latitude: String,
}

#[derive(Debug, Deserialize)]
struct DailyWeather {
    astronomy: Vec<Astronomy>,
}

#[derive(Debug, Deserialize)]
struct Astronomy {
    sunrise: String,
    sunset: String,
}

/// Weather monitor that fetches weather data in a background thread.
#[derive(Debug)]
pub struct WeatherMonitor {
    /// Latest weather fetched, if any fetch has succeeded yet.
    weather: Arc<RwLock<Option<WeatherData>>>,
    /// Location string (empty for auto-detect).
    location: String,
    /// Cleared to tell the fetch thread to exit.
    running: Arc<AtomicBool>,
}

impl WeatherMonitor {
    /// Create a new weather monitor.
    pub fn new(location: String) -> Self {
        Self {
            weather: Arc::new(RwLock::new(None)),
            location,
            running: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Start the background fetching thread.
    pub fn start(&self) {
        if self.running.swap(true, Ordering::Relaxed) {
            return; // Already running
        }

        let weather = self.weather.clone();
        let location = self.location.clone();
        let running = self.running.clone();

        thread::spawn(move || {
            let mut next_fetch = Instant::now();
            while running.load(Ordering::Relaxed) {
                if Instant::now() >= next_fetch {
                    next_fetch = Instant::now()
                        + match fetch_weather(&location) {
                            Ok(data) => {
                                *weather.write().unwrap_or_else(PoisonError::into_inner) =
                                    Some(data);
                                FETCH_INTERVAL
                            }
                            // Keep showing the last weather fetched: a slightly stale
                            // guess beats a placeholder. Retry soon.
                            Err(_) => RETRY_INTERVAL,
                        };
                }
                thread::sleep(POLL_INTERVAL);
            }
        });
    }

    /// Stop the background thread.
    pub fn stop(&self) {
        self.running.store(false, Ordering::Relaxed);
    }

    /// Get the background style for the latest weather at the current time, or
    /// Starfield until the first fetch succeeds.
    pub fn get_background(&self) -> BackgroundStyle {
        let now = chrono::Local::now();
        let minutes = now.hour() * 60 + now.minute();
        self.weather
            .read()
            .unwrap_or_else(PoisonError::into_inner)
            .as_ref()
            .map_or(BackgroundStyle::Starfield, |weather| {
                weather.background_at(minutes)
            })
    }

    /// Get the sunrise and sunset times (if available).
    pub fn get_sunrise_sunset(&self) -> Option<(String, String)> {
        let weather = self.weather.read().unwrap_or_else(PoisonError::into_inner);
        let weather = weather.as_ref()?;
        Some((weather.sunrise.clone()?, weather.sunset.clone()?))
    }
}

impl Drop for WeatherMonitor {
    fn drop(&mut self) {
        self.stop();
    }
}

/// Fetch the current weather for `location` (empty for auto-detect by IP).
fn fetch_weather(location: &str) -> Result<WeatherData, String> {
    let url = if location.is_empty() {
        "https://wttr.in/?format=j1".to_string()
    } else {
        format!("https://wttr.in/{}?format=j1", url_encode(location))
    };

    let agent = ureq::Agent::config_builder()
        .timeout_global(Some(REQUEST_TIMEOUT))
        .build()
        .new_agent();

    let response: WttrResponse = agent
        .get(&url)
        .call()
        .map_err(|e| format!("HTTP error: {e}"))?
        .body_mut()
        .read_json()
        .map_err(|e| format!("JSON parse error: {e}"))?;

    parse_weather_response(response)
}

/// Parse a WttrResponse into WeatherData.
fn parse_weather_response(response: WttrResponse) -> Result<WeatherData, String> {
    // Extract current condition
    let current = response
        .current_condition
        .first()
        .ok_or("No current condition")?;

    let temp_c = current.temp_c.parse().unwrap_or(15);
    let wind_kmph: u32 = current.windspeed_kmph.parse().unwrap_or(0);
    let condition = parse_weather_code(&current.weather_code);

    // Check for high wind override
    let condition = if wind_kmph > 50
        && !matches!(
            condition,
            WeatherCondition::Thunderstorm | WeatherCondition::HeavyRain
        ) {
        WeatherCondition::Windy
    } else {
        condition
    };

    // Get latitude for aurora calculation
    let latitude = response
        .nearest_area
        .as_ref()
        .and_then(|areas| areas.first())
        .and_then(|area| area.latitude.parse().ok())
        .unwrap_or(0.0);

    let astronomy = response
        .weather
        .as_ref()
        .and_then(|days| days.first())
        .and_then(|day| day.astronomy.first());

    Ok(WeatherData {
        condition,
        temp_c,
        latitude,
        sunrise: astronomy.map(|astro| astro.sunrise.clone()),
        sunset: astronomy.map(|astro| astro.sunset.clone()),
    })
}

/// Classify `minutes` past midnight given sunrise and sunset in the same units.
fn time_of_day(minutes: u32, sunrise: u32, sunset: u32) -> TimeOfDay {
    if (sunrise.saturating_sub(CIVIL_TWILIGHT_MINUTES)..sunrise).contains(&minutes) {
        TimeOfDay::Dawn
    } else if (sunset..sunset + CIVIL_TWILIGHT_MINUTES).contains(&minutes) {
        TimeOfDay::Dusk
    } else if (sunrise..sunset).contains(&minutes) {
        TimeOfDay::Day
    } else {
        TimeOfDay::Night
    }
}

/// Parse time string like "06:45 AM" to minutes since midnight.
fn parse_time_to_minutes(time_str: &str) -> Option<u32> {
    let parts: Vec<&str> = time_str.split_whitespace().collect();
    if parts.len() != 2 {
        return None;
    }

    let time_parts: Vec<&str> = parts[0].split(':').collect();
    if time_parts.len() != 2 {
        return None;
    }

    let mut hours: u32 = time_parts[0].parse().ok()?;
    let minutes: u32 = time_parts[1].parse().ok()?;
    let is_pm = parts[1].to_uppercase() == "PM";

    if is_pm && hours != 12 {
        hours += 12;
    } else if !is_pm && hours == 12 {
        hours = 0;
    }

    Some(hours * 60 + minutes)
}

/// Percent-encode a location for the wttr.in URL path (spaces become `+`), so
/// non-ASCII names such as "서울" and characters like `?`, `#` and `/` survive.
fn url_encode(s: &str) -> String {
    let mut encoded = String::with_capacity(s.len());
    for byte in s.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                encoded.push(byte as char);
            }
            b' ' => encoded.push('+'),
            _ => encoded.push_str(&format!("%{byte:02X}")),
        }
    }
    encoded
}

/// Map wttr.in weather code to our simplified condition.
/// See: <https://www.worldweatheronline.com/developer/api/docs/weather-icons.aspx>
fn parse_weather_code(code: &str) -> WeatherCondition {
    match code {
        // Clear/Sunny
        "113" => WeatherCondition::Clear,

        // Partly cloudy
        "116" => WeatherCondition::PartlyCloudy,

        // Cloudy/Overcast
        "119" | "122" => WeatherCondition::Cloudy,

        // Fog/Mist
        "143" | "248" | "260" => WeatherCondition::Fog,

        // Light rain/drizzle
        "176" | "263" | "266" | "293" | "296" | "353" => WeatherCondition::Rain,

        // Heavy rain
        "299" | "302" | "305" | "308" | "356" | "359" => WeatherCondition::HeavyRain,

        // Thunderstorm
        "200" | "386" | "389" | "392" | "395" => WeatherCondition::Thunderstorm,

        // Snow (various types)
        "179" | "182" | "185" | "227" | "230" | "281" | "284" | "311" | "314" | "317" | "320"
        | "323" | "326" | "329" | "332" | "335" | "338" | "350" | "362" | "365" | "368" | "371"
        | "374" | "377" => WeatherCondition::Snow,

        // Default to cloudy for unknown codes
        _ => WeatherCondition::Cloudy,
    }
}

/// Map weather data at the given time of day to the appropriate background style.
fn map_weather_to_background(weather: &WeatherData, time_of_day: TimeOfDay) -> BackgroundStyle {
    // Twilight for clear or partly cloudy conditions during dawn/dusk
    if time_of_day == TimeOfDay::Dawn
        && matches!(
            weather.condition,
            WeatherCondition::Clear | WeatherCondition::PartlyCloudy
        )
    {
        return BackgroundStyle::TwilightDawn;
    }
    if time_of_day == TimeOfDay::Dusk
        && matches!(
            weather.condition,
            WeatherCondition::Clear | WeatherCondition::PartlyCloudy
        )
    {
        return BackgroundStyle::TwilightDusk;
    }

    // Special case: Aurora for clear nights at high latitudes (> 55°)
    if time_of_day == TimeOfDay::Night
        && weather.condition == WeatherCondition::Clear
        && weather.latitude.abs() > 55.0
    {
        return BackgroundStyle::Aurora;
    }

    // Night + Clear = Starfield
    if time_of_day == TimeOfDay::Night && weather.condition == WeatherCondition::Clear {
        return BackgroundStyle::Starfield;
    }

    // Very cold conditions get Frost (below -10°C)
    if weather.temp_c < -10 {
        return BackgroundStyle::Frost;
    }

    // Map by condition
    match weather.condition {
        WeatherCondition::Clear => BackgroundStyle::Sunny,
        WeatherCondition::PartlyCloudy => BackgroundStyle::Cloudy,
        WeatherCondition::Cloudy => BackgroundStyle::Cloudy,
        WeatherCondition::Rain => BackgroundStyle::Rainy,
        WeatherCondition::HeavyRain => BackgroundStyle::Stormy,
        WeatherCondition::Thunderstorm => BackgroundStyle::Stormy,
        WeatherCondition::Snow => BackgroundStyle::Snowfall,
        WeatherCondition::Fog => BackgroundStyle::Foggy,
        WeatherCondition::Windy => BackgroundStyle::Windy,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_weather_code() {
        assert_eq!(parse_weather_code("113"), WeatherCondition::Clear);
        assert_eq!(parse_weather_code("116"), WeatherCondition::PartlyCloudy);
        assert_eq!(parse_weather_code("200"), WeatherCondition::Thunderstorm);
        assert_eq!(parse_weather_code("227"), WeatherCondition::Snow);
        assert_eq!(parse_weather_code("999"), WeatherCondition::Cloudy); // Unknown
    }

    #[test]
    fn test_map_weather_to_background() {
        let sunny_day = WeatherData {
            condition: WeatherCondition::Clear,

            temp_c: 25,
            latitude: 40.0,
            ..Default::default()
        };
        assert_eq!(
            map_weather_to_background(&sunny_day, TimeOfDay::Day),
            BackgroundStyle::Sunny
        );

        let clear_night = WeatherData {
            condition: WeatherCondition::Clear,

            temp_c: 15,
            latitude: 40.0,
            ..Default::default()
        };
        assert_eq!(
            map_weather_to_background(&clear_night, TimeOfDay::Night),
            BackgroundStyle::Starfield
        );

        let aurora_night = WeatherData {
            condition: WeatherCondition::Clear,

            temp_c: -5,
            latitude: 65.0, // High latitude
            ..Default::default()
        };
        assert_eq!(
            map_weather_to_background(&aurora_night, TimeOfDay::Night),
            BackgroundStyle::Aurora
        );

        let very_cold = WeatherData {
            condition: WeatherCondition::Clear,

            temp_c: -15, // Below -10°C
            latitude: 40.0,
            ..Default::default()
        };
        assert_eq!(
            map_weather_to_background(&very_cold, TimeOfDay::Day),
            BackgroundStyle::Frost
        );

        // Twilight tests
        let dawn = WeatherData {
            condition: WeatherCondition::Clear,

            temp_c: 15,
            latitude: 40.0,
            ..Default::default()
        };
        assert_eq!(
            map_weather_to_background(&dawn, TimeOfDay::Dawn),
            BackgroundStyle::TwilightDawn
        );

        let dusk = WeatherData {
            condition: WeatherCondition::PartlyCloudy,

            temp_c: 20,
            latitude: 40.0,
            ..Default::default()
        };
        assert_eq!(
            map_weather_to_background(&dusk, TimeOfDay::Dusk),
            BackgroundStyle::TwilightDusk
        );
    }

    #[test]
    fn time_of_day_has_half_hour_twilight_around_sunrise_and_sunset() {
        let (sunrise, sunset) = (6 * 60, 18 * 60);
        let at = |h: u32, m: u32| time_of_day(h * 60 + m, sunrise, sunset);
        assert_eq!(at(5, 29), TimeOfDay::Night);
        assert_eq!(at(5, 30), TimeOfDay::Dawn);
        assert_eq!(at(6, 0), TimeOfDay::Day);
        assert_eq!(at(17, 59), TimeOfDay::Day);
        assert_eq!(at(18, 0), TimeOfDay::Dusk);
        assert_eq!(at(18, 30), TimeOfDay::Night);
    }

    #[test]
    fn background_follows_the_clock_between_fetches() {
        let weather = WeatherData {
            sunrise: Some("06:00 AM".into()),
            sunset: Some("06:00 PM".into()),
            latitude: 40.0,
            ..Default::default()
        };
        assert_eq!(
            weather.background_at(5 * 60 + 45),
            BackgroundStyle::TwilightDawn
        );
        assert_eq!(weather.background_at(12 * 60), BackgroundStyle::Sunny);
        assert_eq!(
            weather.background_at(18 * 60 + 10),
            BackgroundStyle::TwilightDusk
        );
        assert_eq!(weather.background_at(23 * 60), BackgroundStyle::Starfield);
    }

    #[test]
    fn parses_condition_wind_latitude_and_astronomy() {
        let response = WttrResponse {
            current_condition: vec![CurrentCondition {
                weather_code: "113".into(),
                temp_c: "-3".into(),
                windspeed_kmph: "60".into(),
            }],
            nearest_area: Some(vec![NearestArea {
                latitude: "64.1".into(),
            }]),
            weather: Some(vec![DailyWeather {
                astronomy: vec![Astronomy {
                    sunrise: "10:20 AM".into(),
                    sunset: "04:05 PM".into(),
                }],
            }]),
        };

        let weather = parse_weather_response(response).unwrap();

        assert_eq!(weather.condition, WeatherCondition::Windy);
        assert_eq!(weather.temp_c, -3);
        assert_eq!(weather.latitude, 64.1);
        assert_eq!(weather.sunrise.as_deref(), Some("10:20 AM"));
        assert_eq!(weather.sunset.as_deref(), Some("04:05 PM"));
    }

    #[test]
    fn test_parse_time_to_minutes() {
        assert_eq!(parse_time_to_minutes("06:45 AM"), Some(6 * 60 + 45));
        assert_eq!(parse_time_to_minutes("12:00 PM"), Some(12 * 60));
        assert_eq!(parse_time_to_minutes("12:00 AM"), Some(0));
        assert_eq!(parse_time_to_minutes("06:30 PM"), Some(18 * 60 + 30));
    }

    #[test]
    fn test_url_encode() {
        assert_eq!(url_encode("New York"), "New+York");
        assert_eq!(url_encode("Seoul, Korea"), "Seoul%2C+Korea");
        assert_eq!(url_encode("서울"), "%EC%84%9C%EC%9A%B8");
        assert_eq!(url_encode("a/b?c#d&e"), "a%2Fb%3Fc%23d%26e");
    }

    #[test]
    fn test_weather_monitor_creation() {
        let monitor = WeatherMonitor::new("Seoul".to_string());
        assert_eq!(monitor.get_background(), BackgroundStyle::Starfield);
    }
}

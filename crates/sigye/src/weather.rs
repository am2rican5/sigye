//! Weather data fetching for dynamic weather background.
//!
//! Fetches weather data from wttr.in API and maps conditions to background styles.

use std::sync::{Arc, RwLock};
use std::thread;
use std::time::{Duration, Instant};

use chrono::Timelike;
use serde::Deserialize;
use sigye_core::{BackgroundStyle, TimeOfDay};

/// How often to fetch new weather data (30 minutes).
const FETCH_INTERVAL: Duration = Duration::from_secs(30 * 60);

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
    #[allow(dead_code)]
    VeryCold,
}

/// Parsed weather data from wttr.in API.
#[derive(Debug, Clone)]
pub struct WeatherData {
    /// Current weather condition.
    pub condition: WeatherCondition,
    /// Temperature in Celsius.
    pub temp_c: i32,
    /// Wind speed in km/h (used during fetch for condition override).
    #[allow(dead_code)]
    pub wind_kmph: u32,
    /// Time of day for weather-aware rendering.
    pub time_of_day: TimeOfDay,
    /// Latitude (for aurora calculation).
    pub latitude: f32,
    /// Timestamp when this data was fetched.
    pub fetched_at: Instant,
}

impl WeatherData {
    /// Check if this weather data is still fresh (less than 30 minutes old).
    pub fn is_fresh(&self) -> bool {
        self.fetched_at.elapsed() < FETCH_INTERVAL
    }
}

impl Default for WeatherData {
    fn default() -> Self {
        Self {
            condition: WeatherCondition::Clear,
            temp_c: 20,
            wind_kmph: 0,
            time_of_day: TimeOfDay::Day,
            latitude: 0.0,
            fetched_at: Instant::now(),
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
    /// Current weather data (if available).
    weather_data: Arc<RwLock<Option<WeatherData>>>,
    /// Current resolved background style.
    resolved_background: Arc<RwLock<BackgroundStyle>>,
    /// Cached background for when lock is contended.
    cached_background: Arc<RwLock<BackgroundStyle>>,
    /// Location string (empty for auto-detect).
    location: String,
    /// Flag to signal thread termination.
    running: Arc<RwLock<bool>>,
}

impl WeatherMonitor {
    /// Create a new weather monitor.
    pub fn new(location: String) -> Self {
        Self {
            weather_data: Arc::new(RwLock::new(None)),
            resolved_background: Arc::new(RwLock::new(BackgroundStyle::Starfield)),
            cached_background: Arc::new(RwLock::new(BackgroundStyle::Starfield)),
            location,
            running: Arc::new(RwLock::new(false)),
        }
    }

    /// Start the background fetching thread.
    pub fn start(&self) {
        if let Ok(mut running) = self.running.write() {
            if *running {
                return; // Already running
            }
            *running = true;
        }

        let weather_data = self.weather_data.clone();
        let resolved_bg = self.resolved_background.clone();
        let cached_bg = self.cached_background.clone();
        let location = self.location.clone();
        let running = self.running.clone();

        thread::spawn(move || {
            // Fetch immediately on start
            fetch_and_update(&location, &weather_data, &resolved_bg, &cached_bg);

            let mut last_fetch = Instant::now();

            loop {
                // Check if we should stop
                if let Ok(is_running) = running.read()
                    && !*is_running
                {
                    break;
                }

                // Fetch new data if interval elapsed
                if last_fetch.elapsed() >= FETCH_INTERVAL {
                    fetch_and_update(&location, &weather_data, &resolved_bg, &cached_bg);
                    last_fetch = Instant::now();
                }

                // Sleep for a bit before checking again
                thread::sleep(Duration::from_secs(60));
            }
        });
    }

    /// Stop the background thread.
    pub fn stop(&self) {
        if let Ok(mut running) = self.running.write() {
            *running = false;
        }
    }

    /// Get the currently resolved background style.
    /// Uses try_read with fallback to cached values to avoid blocking.
    pub fn get_background(&self) -> BackgroundStyle {
        // Try non-blocking read first
        if let Ok(bg) = self.resolved_background.try_read() {
            return *bg;
        }
        // Fall back to cached background
        if let Ok(cached) = self.cached_background.read() {
            return *cached;
        }
        // Last resort: return default
        BackgroundStyle::Starfield
    }

    /// Get the current weather data (if available).
    #[allow(dead_code)]
    pub fn get_weather_data(&self) -> Option<WeatherData> {
        self.weather_data.read().ok().and_then(|w| w.clone())
    }

    /// Get the current time of day for weather-aware rendering.
    #[allow(dead_code)]
    pub fn get_time_of_day(&self) -> TimeOfDay {
        self.weather_data
            .read()
            .ok()
            .and_then(|w| w.as_ref().map(|d| d.time_of_day))
            .unwrap_or(TimeOfDay::Day)
    }
}

impl Default for WeatherMonitor {
    fn default() -> Self {
        Self::new(String::new())
    }
}

impl Drop for WeatherMonitor {
    fn drop(&mut self) {
        self.stop();
    }
}

/// Fetch weather data and update shared state.
fn fetch_and_update(
    location: &str,
    weather_data: &Arc<RwLock<Option<WeatherData>>>,
    resolved_bg: &Arc<RwLock<BackgroundStyle>>,
    cached_bg: &Arc<RwLock<BackgroundStyle>>,
) {
    match fetch_weather(location) {
        Ok(data) => {
            let background = map_weather_to_background(&data);

            if let Ok(mut wd) = weather_data.write() {
                *wd = Some(data);
            }
            if let Ok(mut bg) = resolved_bg.write() {
                *bg = background;
            }
            if let Ok(mut cb) = cached_bg.write() {
                *cb = background;
            }
        }
        Err(_e) => {
            // On error, keep existing data if fresh, otherwise use fallback
            let should_fallback = weather_data
                .read()
                .map(|w| w.as_ref().map(|d| !d.is_fresh()).unwrap_or(true))
                .unwrap_or(true);

            if should_fallback {
                if let Ok(mut bg) = resolved_bg.write() {
                    *bg = BackgroundStyle::Starfield;
                }
                if let Ok(mut cb) = cached_bg.write() {
                    *cb = BackgroundStyle::Starfield;
                }
            }
        }
    }
}

/// Fetch weather data from wttr.in API.
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

    // Determine time of day (day, night, dawn, dusk)
    let time_of_day = determine_time_of_day(&response);

    Ok(WeatherData {
        condition,
        temp_c,
        wind_kmph,
        time_of_day,
        latitude,
        fetched_at: Instant::now(),
    })
}

/// Determine the current time of day based on sunrise/sunset.
fn determine_time_of_day(response: &WttrResponse) -> TimeOfDay {
    let Some(weather) = response.weather.as_ref().and_then(|w| w.first()) else {
        return TimeOfDay::Day; // Default to day
    };

    let Some(astronomy) = weather.astronomy.first() else {
        return TimeOfDay::Day;
    };

    // Parse times (format: "06:45 AM")
    let now = chrono::Local::now();
    let current_minutes = now.hour() * 60 + now.minute();

    let sunrise_mins = parse_time_to_minutes(&astronomy.sunrise).unwrap_or(6 * 60);
    let sunset_mins = parse_time_to_minutes(&astronomy.sunset).unwrap_or(18 * 60);

    // Calculate twilight boundaries
    let dawn_start = sunrise_mins.saturating_sub(CIVIL_TWILIGHT_MINUTES);
    let dusk_end = sunset_mins + CIVIL_TWILIGHT_MINUTES;

    if current_minutes >= dawn_start && current_minutes < sunrise_mins {
        TimeOfDay::Dawn
    } else if current_minutes >= sunset_mins && current_minutes < dusk_end {
        TimeOfDay::Dusk
    } else if current_minutes >= sunrise_mins && current_minutes < sunset_mins {
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

/// Simple URL encoding for location strings.
fn url_encode(s: &str) -> String {
    s.replace(' ', "+").replace(',', "%2C")
}

/// Map wttr.in weather code to our simplified condition.
/// See: https://www.worldweatheronline.com/developer/api/docs/weather-icons.aspx
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

/// Map weather data to the appropriate background style.
fn map_weather_to_background(weather: &WeatherData) -> BackgroundStyle {
    // Twilight for clear or partly cloudy conditions during dawn/dusk
    if weather.time_of_day == TimeOfDay::Dawn
        && matches!(
            weather.condition,
            WeatherCondition::Clear | WeatherCondition::PartlyCloudy
        )
    {
        return BackgroundStyle::TwilightDawn;
    }
    if weather.time_of_day == TimeOfDay::Dusk
        && matches!(
            weather.condition,
            WeatherCondition::Clear | WeatherCondition::PartlyCloudy
        )
    {
        return BackgroundStyle::TwilightDusk;
    }

    // Special case: Aurora for clear nights at high latitudes (> 55°)
    if weather.time_of_day == TimeOfDay::Night
        && weather.condition == WeatherCondition::Clear
        && weather.latitude.abs() > 55.0
    {
        return BackgroundStyle::Aurora;
    }

    // Night + Clear = Starfield
    if weather.time_of_day == TimeOfDay::Night && weather.condition == WeatherCondition::Clear {
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
        WeatherCondition::VeryCold => BackgroundStyle::Frost,
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
            time_of_day: TimeOfDay::Day,
            temp_c: 25,
            latitude: 40.0,
            ..Default::default()
        };
        assert_eq!(
            map_weather_to_background(&sunny_day),
            BackgroundStyle::Sunny
        );

        let clear_night = WeatherData {
            condition: WeatherCondition::Clear,
            time_of_day: TimeOfDay::Night,
            temp_c: 15,
            latitude: 40.0,
            ..Default::default()
        };
        assert_eq!(
            map_weather_to_background(&clear_night),
            BackgroundStyle::Starfield
        );

        let aurora_night = WeatherData {
            condition: WeatherCondition::Clear,
            time_of_day: TimeOfDay::Night,
            temp_c: -5,
            latitude: 65.0, // High latitude
            ..Default::default()
        };
        assert_eq!(
            map_weather_to_background(&aurora_night),
            BackgroundStyle::Aurora
        );

        let very_cold = WeatherData {
            condition: WeatherCondition::Clear,
            time_of_day: TimeOfDay::Day,
            temp_c: -15, // Below -10°C
            latitude: 40.0,
            ..Default::default()
        };
        assert_eq!(
            map_weather_to_background(&very_cold),
            BackgroundStyle::Frost
        );

        // Twilight tests
        let dawn = WeatherData {
            condition: WeatherCondition::Clear,
            time_of_day: TimeOfDay::Dawn,
            temp_c: 15,
            latitude: 40.0,
            ..Default::default()
        };
        assert_eq!(
            map_weather_to_background(&dawn),
            BackgroundStyle::TwilightDawn
        );

        let dusk = WeatherData {
            condition: WeatherCondition::PartlyCloudy,
            time_of_day: TimeOfDay::Dusk,
            temp_c: 20,
            latitude: 40.0,
            ..Default::default()
        };
        assert_eq!(
            map_weather_to_background(&dusk),
            BackgroundStyle::TwilightDusk
        );
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
    }

    #[test]
    fn test_weather_monitor_creation() {
        let monitor = WeatherMonitor::new("Seoul".to_string());
        assert_eq!(monitor.get_background(), BackgroundStyle::Starfield);
    }
}

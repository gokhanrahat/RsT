use dioxus::prelude::*;
use reqwest::Client;
use serde::Deserialize;

// --- Data Structures ---

#[derive(Debug, Clone)]
struct CityInfo {
    name: &'static str,
    lat: f64,
    lon: f64,
}

const CITIES: &[CityInfo] = &[
    CityInfo { name: "Istanbul",  lat: 41.0082, lon: 28.9784 },
    CityInfo { name: "Ankara",    lat: 39.9334, lon: 32.8597 },
    CityInfo { name: "Izmir",     lat: 38.4192, lon: 27.1287 },
    CityInfo { name: "Antalya",   lat: 36.8969, lon: 30.7133 },
    CityInfo { name: "Bursa",     lat: 40.1885, lon: 29.0610 },
    CityInfo { name: "Trabzon",   lat: 41.0015, lon: 39.7178 },
    CityInfo { name: "Konya",     lat: 37.8713, lon: 32.4846 },
    CityInfo { name: "Gaziantep", lat: 37.0662, lon: 37.3833 },
    CityInfo { name: "Bodrum",    lat: 37.0344, lon: 27.4305 },
    CityInfo { name: "Cappadocia",lat: 38.6431, lon: 34.8307 },
];

#[derive(Deserialize, Debug, Clone)]
struct WeatherResponse {
    current: CurrentWeather,
    daily: DailyWeather,
}

#[derive(Deserialize, Debug, Clone)]
struct CurrentWeather {
    temperature_2m: f64,
    relative_humidity_2m: f64,
    wind_speed_10m: f64,
    weather_code: u32,
}

#[derive(Deserialize, Debug, Clone)]
struct DailyWeather {
    time: Vec<String>,
    temperature_2m_max: Vec<f64>,
    temperature_2m_min: Vec<f64>,
    weather_code: Vec<u32>,
}

// --- Weather Code Helpers ---

fn weather_emoji(code: u32) -> &'static str {
    match code {
        0 => "☀️",
        1..=3 => "⛅",
        45 | 48 => "🌫️",
        51..=67 => "🌧️",
        71..=77 => "❄️",
        80..=82 => "🌦️",
        85 | 86 => "🌨️",
        95..=99 => "⛈️",
        _ => "🌡️",
    }
}

fn weather_description(code: u32) -> &'static str {
    match code {
        0 => "Clear sky",
        1 => "Mainly clear",
        2 => "Partly cloudy",
        3 => "Overcast",
        45 | 48 => "Foggy",
        51..=55 => "Drizzle",
        61..=65 => "Rain",
        66 | 67 => "Freezing Rain",
        71..=77 => "Snow",
        80..=82 => "Rain showers",
        85 | 86 => "Snow showers",
        95 => "Thunderstorm",
        96 | 99 => "Thunderstorm with hail",
        _ => "Unknown",
    }
}

// --- API Fetch ---

async fn fetch_weather(city: &CityInfo) -> Result<WeatherResponse, String> {
    let url = format!(
        "https://api.open-meteo.com/v1/forecast?\
         latitude={}&longitude={}\
         &current=temperature_2m,relative_humidity_2m,wind_speed_10m,weather_code\
         &daily=temperature_2m_max,temperature_2m_min,weather_code\
         &timezone=Europe%2FIstanbul\
         &forecast_days=5",
        city.lat, city.lon
    );

    Client::new()
        .get(&url)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json::<WeatherResponse>()
        .await
        .map_err(|e| e.to_string())
}

// --- Main ---

fn main() {
    dioxus::launch(App);
}

// --- Root Component ---

fn App() -> Element {
    let mut selected_city = use_signal(|| 0usize);
    let mut weather: Signal<Option<Result<WeatherResponse, String>>> = use_signal(|| None);
    let mut loading = use_signal(|| false);

    // Fetch weather whenever selected city changes
    use_effect(move || {
        let idx = selected_city();
        let city = &CITIES[idx];
        let lat = city.lat;
        let lon = city.lon;

        spawn(async move {
            loading.set(true);
            weather.set(None);

            let url = format!(
                "https://api.open-meteo.com/v1/forecast?\
                 latitude={}&longitude={}\
                 &current=temperature_2m,relative_humidity_2m,wind_speed_10m,weather_code\
                 &daily=temperature_2m_max,temperature_2m_min,weather_code\
                 &timezone=Europe%2FIstanbul\
                 &forecast_days=5",
                lat, lon
            );

            let result = async {
                Client::new()
                    .get(&url)
                    .send()
                    .await
                    .map_err(|e| e.to_string())?
                    .json::<WeatherResponse>()
                    .await
                    .map_err(|e| e.to_string())
            }
            .await;

            weather.set(Some(result));
            loading.set(false);
        });
    });

    rsx! {
        div {
            style: "font-family: 'Segoe UI', sans-serif; max-width: 700px; margin: 0 auto; padding: 24px; background: #f0f4ff; min-height: 100vh;",

            // Header
            h1 {
                style: "text-align: center; color: #1a237e; font-size: 2rem; margin-bottom: 8px;",
                "🇹🇷 Turkey Weather"
            }
            p {
                style: "text-align: center; color: #555; margin-bottom: 24px;",
                "Select a city to see current weather and 5-day forecast"
            }

            // City Selector
            div {
                style: "display: flex; flex-wrap: wrap; gap: 10px; justify-content: center; margin-bottom: 28px;",
                for (i, city) in CITIES.iter().enumerate() {
                    button {
                        style: if selected_city() == i {
                            "padding: 8px 16px; border-radius: 20px; border: none; background: #1a237e; color: white; cursor: pointer; font-size: 0.95rem; font-weight: bold;"
                        } else {
                            "padding: 8px 16px; border-radius: 20px; border: 2px solid #1a237e; background: white; color: #1a237e; cursor: pointer; font-size: 0.95rem;"
                        },
                        onclick: move |_| selected_city.set(i),
                        "{city.name}"
                    }
                }
            }

            // Weather Display
            if loading() {
                div {
                    style: "text-align: center; font-size: 1.2rem; color: #555; padding: 40px;",
                    "⏳ Loading weather data..."
                }
            }

            if let Some(result) = weather.read().as_ref() {
                match result {
                    Ok(data) => rsx! {
                        // Current Weather Card
                        div {
                            style: "background: linear-gradient(135deg, #1a237e, #3949ab); color: white; border-radius: 16px; padding: 28px; margin-bottom: 24px; box-shadow: 0 4px 15px rgba(0,0,0,0.2);",
                            h2 {
                                style: "margin: 0 0 4px 0; font-size: 1.6rem;",
                                "{weather_emoji(data.current.weather_code)}  {CITIES[selected_city()].name}"
                            }
                            p {
                                style: "margin: 0 0 16px 0; opacity: 0.8;",
                                "{weather_description(data.current.weather_code)}"
                            }
                            div {
                                style: "font-size: 4rem; font-weight: bold; margin-bottom: 16px;",
                                "{data.current.temperature_2m:.1}°C"
                            }
                            div {
                                style: "display: flex; gap: 24px; font-size: 0.95rem; opacity: 0.9;",
                                span { "💧 Humidity: {data.current.relative_humidity_2m:.0}%" }
                                span { "💨 Wind: {data.current.wind_speed_10m:.1} km/h" }
                            }
                        }

                        // 5-Day Forecast
                        h3 {
                            style: "color: #1a237e; margin-bottom: 12px;",
                            "5-Day Forecast"
                        }
                        div {
                            style: "display: flex; gap: 12px; overflow-x: auto; padding-bottom: 8px;",
                            for i in 0..data.daily.time.len() {
                                div {
                                    style: "background: white; border-radius: 12px; padding: 16px; min-width: 110px; text-align: center; box-shadow: 0 2px 8px rgba(0,0,0,0.08); flex: 1;",
                                    p {
                                        style: "font-weight: bold; color: #1a237e; margin: 0 0 8px 0; font-size: 0.85rem;",
                                        "{data.daily.time[i]}"
                                    }
                                    p {
                                        style: "font-size: 2rem; margin: 0 0 8px 0;",
                                        "{weather_emoji(data.daily.weather_code[i])}"
                                    }
                                    p {
                                        style: "color: #e53935; font-weight: bold; margin: 0;",
                                        "↑ {data.daily.temperature_2m_max[i]:.1}°"
                                    }
                                    p {
                                        style: "color: #1e88e5; margin: 0;",
                                        "↓ {data.daily.temperature_2m_min[i]:.1}°"
                                    }
                                }
                            }
                        }
                    },
                    Err(e) => rsx! {
                        div {
                            style: "background: #ffebee; border-radius: 12px; padding: 20px; color: #c62828; text-align: center;",
                            "❌ Error fetching weather: {e}"
                        }
                    }
                }
            }

            // Footer
            p {
                style: "text-align: center; color: #aaa; font-size: 0.8rem; margin-top: 32px;",
                "Data provided by Open-Meteo.com • Free & No API key required"
            }
        }
    }
}
use dioxus::prelude::*;
use serde::Deserialize;

// --- Veri Yapıları ---

#[derive(Debug, Clone)]
struct SehirBilgisi {
    ad: &'static str,
    enlem: f64,
    boylam: f64,
}

const SEHIRLER: &[SehirBilgisi] = &[
    SehirBilgisi { ad: "İstanbul",   enlem: 41.0082, boylam: 28.9784 },
    SehirBilgisi { ad: "Ankara",     enlem: 39.9334, boylam: 32.8597 },
    SehirBilgisi { ad: "İzmir",      enlem: 38.4192, boylam: 27.1287 },
    SehirBilgisi { ad: "Antalya",    enlem: 36.8969, boylam: 30.7133 },
    SehirBilgisi { ad: "Bursa",      enlem: 40.1885, boylam: 29.0610 },
    SehirBilgisi { ad: "Trabzon",    enlem: 41.0015, boylam: 39.7178 },
    SehirBilgisi { ad: "Konya",      enlem: 37.8713, boylam: 32.4846 },
    SehirBilgisi { ad: "Gaziantep",  enlem: 37.0662, boylam: 37.3833 },
    SehirBilgisi { ad: "Bodrum",     enlem: 37.0344, boylam: 27.4305 },
    SehirBilgisi { ad: "Kapadokya",  enlem: 38.6431, boylam: 34.8307 },
];

#[derive(Deserialize, Debug, Clone)]
struct HavaDurumuYaniti {
    current: AnlikHava,
    daily: GunlukHava,
}

#[derive(Deserialize, Debug, Clone)]
struct AnlikHava {
    temperature_2m: f64,
    relative_humidity_2m: f64,
    wind_speed_10m: f64,
    weather_code: u32,
}

#[derive(Deserialize, Debug, Clone)]
struct GunlukHava {
    time: Vec<String>,
    temperature_2m_max: Vec<f64>,
    temperature_2m_min: Vec<f64>,
    weather_code: Vec<u32>,
}

// --- Hava Durumu Kodu Yardımcıları ---

fn hava_emoji(kod: u32) -> &'static str {
    match kod {
        0        => "☀️",
        1..=3    => "⛅",
        45 | 48  => "🌫️",
        51..=67  => "🌧️",
        71..=77  => "❄️",
        80..=82  => "🌦️",
        85 | 86  => "🌨️",
        95..=99  => "⛈️",
        _        => "🌡️",
    }
}

fn hava_aciklamasi(kod: u32) -> &'static str {
    match kod {
        0        => "Açık hava",
        1        => "Çoğunlukla açık",
        2        => "Parçalı bulutlu",
        3        => "Kapalı",
        45 | 48  => "Sisli",
        51..=55  => "Çisenti",
        61..=65  => "Yağmurlu",
        66 | 67  => "Dondurucu yağmur",
        71..=77  => "Karlı",
        80..=82  => "Sağanak yağışlı",
        85 | 86  => "Kar sağanağı",
        95       => "Fırtınalı",
        96 | 99  => "Dolu fırtınası",
        _        => "Bilinmiyor",
    }
}

fn gun_adi(tarih: &str) -> String {
    // "2024-01-15" formatından gün adını çıkar
    let parcalar: Vec<&str> = tarih.split('-').collect();
    if parcalar.len() != 3 { return tarih.to_string(); }

    let yil: i32 = parcalar[0].parse().unwrap_or(2024);
    let ay: u32  = parcalar[1].parse().unwrap_or(1);
    let gun: u32 = parcalar[2].parse().unwrap_or(1);

    // Zeller formülüyle haftanın gününü hesapla
    let (y, m) = if ay < 3 { (yil - 1, ay + 12) } else { (yil, ay) };
    let k = y % 100;
    let j = y / 100;
    let h = (gun as i32 + ((13 * (m as i32 + 1)) / 5) + k + k/4 + j/4 - 2*j).rem_euclid(7);

    let gun_adi = match h {
        0 => "Cmt", 1 => "Paz", 2 => "Pzt",
        3 => "Sal", 4 => "Çar", 5 => "Per",
        6 => "Cum", _ => "---",
    };

    format!("{}\n{}.{}", gun_adi, gun, ay)
}

// --- Ana Fonksiyon ---

fn main() {
    dioxus::launch(Uygulama);
}

// --- Kök Bileşen ---

fn Uygulama() -> Element {
    let mut secili_sehir = use_signal(|| 0usize);
    let mut hava: Signal<Option<Result<HavaDurumuYaniti, String>>> = use_signal(|| None);
    let mut yukleniyor = use_signal(|| false);

    use_effect(move || {
        let idx = secili_sehir();
        let enlem = SEHIRLER[idx].enlem;
        let boylam = SEHIRLER[idx].boylam;

        spawn(async move {
            yukleniyor.set(true);
            hava.set(None);

            let url = format!(
                "https://api.open-meteo.com/v1/forecast?\
                 latitude={}&longitude={}\
                 &current=temperature_2m,relative_humidity_2m,wind_speed_10m,weather_code\
                 &daily=temperature_2m_max,temperature_2m_min,weather_code\
                 &timezone=Europe%2FIstanbul\
                 &forecast_days=5",
                enlem, boylam
            );

            let sonuc = async {
                reqwest::Client::new()
                    .get(&url)
                    .send()
                    .await
                    .map_err(|e: reqwest::Error| e.to_string())?
                    .json::<HavaDurumuYaniti>()
                    .await
                    .map_err(|e: reqwest::Error| e.to_string())
            }.await;

            hava.set(Some(sonuc));
            yukleniyor.set(false);
        });
    });

    rsx! {
        div {
            style: "font-family: 'Segoe UI', sans-serif; max-width: 720px; margin: 0 auto; padding: 24px; background: #eef2ff; min-height: 100vh;",

            // Başlık
            h1 {
                style: "text-align: center; color: #1a237e; font-size: 2rem; margin-bottom: 4px;",
                "🇹🇷 Türkiye Hava Durumu"
            }
            p {
                style: "text-align: center; color: #666; margin-bottom: 24px; font-size: 0.95rem;",
                "Anlık hava durumu ve 5 günlük tahmin için şehir seçin"
            }

            // Şehir Seçici
            div {
                style: "display: flex; flex-wrap: wrap; gap: 10px; justify-content: center; margin-bottom: 28px;",
                for (i, sehir) in SEHIRLER.iter().enumerate() {
                    button {
                        style: if secili_sehir() == i {
                            "padding: 8px 18px; border-radius: 20px; border: none; \
                             background: #1a237e; color: white; cursor: pointer; \
                             font-size: 0.9rem; font-weight: bold; transition: all 0.2s;"
                        } else {
                            "padding: 8px 18px; border-radius: 20px; \
                             border: 2px solid #1a237e; background: white; \
                             color: #1a237e; cursor: pointer; font-size: 0.9rem; transition: all 0.2s;"
                        },
                        onclick: move |_| secili_sehir.set(i),
                        "{sehir.ad}"
                    }
                }
            }

            // Yükleniyor
            if yukleniyor() {
                div {
                    style: "text-align: center; font-size: 1.2rem; color: #555; padding: 48px;",
                    "⏳ Hava durumu bilgisi alınıyor..."
                }
            }

            // Hava Durumu Gösterimi
            if let Some(sonuc) = hava.read().as_ref() {
                match sonuc {
                    Ok(veri) => rsx! {

                        // Anlık Hava Kartı
                        div {
                            style: "background: linear-gradient(135deg, #1a237e, #3949ab); \
                                    color: white; border-radius: 20px; padding: 28px; \
                                    margin-bottom: 24px; box-shadow: 0 6px 20px rgba(0,0,0,0.2);",

                            h2 {
                                style: "margin: 0 0 4px 0; font-size: 1.6rem;",
                                "{hava_emoji(veri.current.weather_code)}  {SEHIRLER[secili_sehir()].ad}"
                            }
                            p {
                                style: "margin: 0 0 20px 0; opacity: 0.85; font-size: 1rem;",
                                "{hava_aciklamasi(veri.current.weather_code)}"
                            }
                            div {
                                style: "font-size: 4.5rem; font-weight: bold; margin-bottom: 20px; letter-spacing: -2px;",
                                "{veri.current.temperature_2m:.1}°C"
                            }
                            div {
                                style: "display: flex; gap: 28px; font-size: 0.95rem; opacity: 0.9; flex-wrap: wrap;",
                                span { "💧 Nem: {veri.current.relative_humidity_2m:.0}%" }
                                span { "💨 Rüzgar: {veri.current.wind_speed_10m:.1} km/sa" }
                            }
                        }

                        // 5 Günlük Tahmin
                        h3 {
                            style: "color: #1a237e; margin-bottom: 14px; font-size: 1.1rem;",
                            "📅 5 Günlük Tahmin"
                        }
                        div {
                            style: "display: flex; gap: 12px; overflow-x: auto; padding-bottom: 8px;",
                            for i in 0..veri.daily.time.len() {
                                div {
                                    style: "background: white; border-radius: 14px; padding: 16px 12px; \
                                            min-width: 110px; text-align: center; flex: 1; \
                                            box-shadow: 0 2px 10px rgba(0,0,0,0.08);",
                                    p {
                                        style: "font-weight: bold; color: #1a237e; margin: 0 0 10px 0; \
                                                font-size: 0.8rem; white-space: pre-line; line-height: 1.5;",
                                        "{gun_adi(&veri.daily.time[i])}"
                                    }
                                    p {
                                        style: "font-size: 2rem; margin: 0 0 10px 0;",
                                        "{hava_emoji(veri.daily.weather_code[i])}"
                                    }
                                    p {
                                        style: "color: #e53935; font-weight: bold; margin: 0 0 4px 0; font-size: 0.95rem;",
                                        "↑ {veri.daily.temperature_2m_max[i]:.1}°"
                                    }
                                    p {
                                        style: "color: #1e88e5; margin: 0; font-size: 0.95rem;",
                                        "↓ {veri.daily.temperature_2m_min[i]:.1}°"
                                    }
                                }
                            }
                        }
                    },
                    Err(hata) => rsx! {
                        div {
                            style: "background: #ffebee; border-radius: 14px; padding: 20px; \
                                    color: #c62828; text-align: center; font-size: 1rem;",
                            "❌ Hava durumu alınamadı: {hata}"
                        }
                    }
                }
            }

            // Alt Bilgi
            p {
                style: "text-align: center; color: #aaa; font-size: 0.78rem; margin-top: 36px;",
                "Veriler Open-Meteo.com tarafından sağlanmaktadır • Ücretsiz, API anahtarı gerekmez"
            }
        }
    }
}

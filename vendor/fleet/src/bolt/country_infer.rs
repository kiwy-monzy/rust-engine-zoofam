//! Rough ISO 3166-1 alpha-2 from coordinates for Bolt query params (no network).

/// (lat_min, lat_max, lng_min, lng_max) — use more specific boxes before broader ones.
/// A bounding box `(min_lat, max_lat, min_lng, max_lng)` and the ISO-3166-1
/// alpha-2 code it implies.
type Region = ((f64, f64, f64, f64), &'static str);

const REGIONS: &[Region] = &[
    // West / Central Africa
    ((4.0, 14.0, 2.5, 15.0), "ng"),
    ((1.0, 12.0, -4.0, 2.5), "gh"),
    ((6.0, 13.0, -3.5, 1.5), "bf"),
    ((4.0, 11.5, -9.0, -4.5), "ci"),
    ((12.0, 25.0, -18.0, -5.0), "ml"),
    ((0.5, 5.5, 5.5, 9.0), "bj"),
    ((1.0, 6.8, 8.0, 16.5), "cm"),
    ((-5.0, 5.5, 8.5, 19.0), "cd"),
    ((-13.0, -5.0, 11.0, 31.0), "zm"),
    ((-18.0, -8.0, 19.0, 34.0), "zw"),
    ((-5.0, 5.5, 29.0, 41.0), "ug"),
    ((-5.0, 5.5, 33.0, 42.5), "ke"),
    ((-12.0, -1.0, 29.0, 41.0), "tz"),
    ((-27.0, -21.5, 19.0, 33.5), "bw"),
    ((-35.0, -22.0, 16.0, 33.5), "za"),
    ((-18.0, -8.0, 11.0, 25.0), "na"),
    ((11.5, 15.0, -18.0, -11.0), "mr"),
    ((18.5, 27.5, -18.0, -8.0), "ma"),
    // Europe (Bolt markets, approximate)
    ((49.5, 61.0, -11.0, 2.0), "gb"),
    ((49.0, 55.5, 3.0, 7.5), "nl"),
    ((47.0, 55.5, 5.5, 15.5), "de"),
    ((41.5, 51.5, -5.5, 10.0), "fr"),
    ((36.0, 44.0, -10.0, 5.0), "es"),
    ((35.5, 48.0, 6.5, 19.0), "it"),
    ((45.5, 49.5, 5.5, 18.5), "at"),
    ((45.5, 49.5, 12.0, 19.0), "hu"),
    ((48.5, 51.5, 14.0, 25.0), "pl"),
    ((55.5, 58.5, 20.5, 28.5), "ee"),
    ((55.5, 58.5, 20.5, 27.5), "lv"),
    ((53.5, 56.5, 20.5, 27.0), "lt"),
    ((57.5, 60.5, 10.5, 20.0), "se"),
    ((57.5, 62.5, 4.0, 12.0), "no"),
    ((54.5, 58.5, 8.0, 16.0), "dk"),
    ((59.0, 70.5, 4.0, 32.0), "fi"),
    ((45.0, 48.5, 8.5, 18.5), "ch"),
    ((50.5, 54.5, 2.0, 7.0), "be"),
    ((49.5, 51.5, 14.0, 19.0), "cz"),
    ((43.5, 48.5, 13.0, 23.5), "ro"),
    ((41.0, 44.5, 19.0, 23.5), "al"),
    ((33.5, 42.5, 19.5, 30.5), "gr"),
    ((35.5, 43.5, 25.5, 45.5), "tr"),
    ((59.0, 70.0, -10.0, 3.0), "ie"),
    ((49.0, 55.0, 9.0, 20.0), "sk"),
    ((45.0, 47.5, 13.0, 17.0), "si"),
    ((42.5, 47.0, 13.0, 20.0), "hr"),
    ((42.0, 44.5, 18.5, 23.5), "me"),
    ((41.5, 44.5, 20.0, 23.0), "xk"),
    ((42.0, 47.5, 18.0, 23.5), "rs"),
    ((43.0, 47.5, 21.5, 30.0), "bg"),
    ((46.5, 49.5, 16.0, 23.5), "ua"),
    ((51.0, 56.0, 23.0, 33.0), "by"),
    // Americas
    ((24.0, 50.0, -125.0, -66.0), "us"),
    ((13.0, 33.0, -119.0, -86.0), "mx"),
    ((-56.0, 13.0, -75.0, -34.0), "br"),
    ((-56.0, -21.0, -74.0, -53.0), "ar"),
    ((-19.0, 5.0, -82.0, -34.0), "co"),
    // Middle East / Asia samples
    ((22.5, 32.0, 34.0, 39.5), "eg"),
    ((29.0, 37.5, 34.0, 43.0), "jo"),
    ((29.0, 37.5, 38.0, 49.0), "sa"),
    ((24.0, 32.0, 50.0, 57.0), "ae"),
];

pub fn infer_country_iso2(lat: f64, lng: f64) -> &'static str {
    for ((min_lat, max_lat, min_lng, max_lng), code) in REGIONS {
        if lat >= *min_lat && lat <= *max_lat && lng >= *min_lng && lng <= *max_lng {
            return code;
        }
    }
    "us"
}

pub fn timezone_for_country_iso2(country: &str) -> &'static str {
    let c = country.as_bytes();
    if c.len() != 2 {
        return "UTC";
    }
    let a = c[0].to_ascii_lowercase();
    let b = c[1].to_ascii_lowercase();
    match (a, b) {
        (b't', b'z') => "Africa/Dar_es_Salaam",
        (b'n', b'g') => "Africa/Lagos",
        (b'k', b'e') => "Africa/Nairobi",
        (b'u', b'g') => "Africa/Kampala",
        (b'z', b'a') => "Africa/Johannesburg",
        (b'g', b'h') => "Africa/Accra",
        (b'z', b'm') => "Africa/Lusaka",
        (b'z', b'w') => "Africa/Harare",
        (b'b', b'w') => "Africa/Gaborone",
        (b'n', b'a') => "Africa/Windhoek",
        (b'g', b'b') => "Europe/London",
        (b'd', b'e') => "Europe/Berlin",
        (b'f', b'r') => "Europe/Paris",
        (b'n', b'l') => "Europe/Amsterdam",
        (b'e', b's') => "Europe/Madrid",
        (b'i', b't') => "Europe/Rome",
        (b'p', b'l') => "Europe/Warsaw",
        (b'e', b'e') => "Europe/Tallinn",
        (b'l', b'v') => "Europe/Riga",
        (b'l', b't') => "Europe/Vilnius",
        (b's', b'e') => "Europe/Stockholm",
        (b'n', b'o') => "Europe/Oslo",
        (b'd', b'k') => "Europe/Copenhagen",
        (b'f', b'i') => "Europe/Helsinki",
        (b'u', b's') => "America/New_York",
        (b'm', b'x') => "America/Mexico_City",
        (b'b', b'r') => "America/Sao_Paulo",
        (b'a', b'e') => "Asia/Dubai",
        (b's', b'a') => "Asia/Riyadh",
        (b'e', b'g') => "Africa/Cairo",
        (b't', b'r') => "Europe/Istanbul",
        _ => "UTC",
    }
}

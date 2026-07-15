pub const MESSAGE_TYPE_PRE_ALERT: &str = "pre-alert";
pub const MESSAGE_TYPE_RED_ALERT: &str = "red-alert";
pub const MESSAGE_TYPE_HOSTILE_AIRCRAFT: &str = "hostile-aircraft";
pub const MESSAGE_TYPE_END: &str = "end";
pub const MESSAGE_TYPE_UNCONFIGURED: &str = "unconfigured";

const PRE_ALERT_KEYS: &[&str] = &[
    "בדקות הקרובות צפויות להתקבל התרעות באזורך",
    "בדקות הקרובות צפויות להתקבל התרעות באיזורים הבאים:",
    "שהייה בסמיכות למרחב מוגן",
    "התקרבו למרחב מוגן",
];

const RED_ALERT_KEYS: &[&str] = &[
    "ירי רקטות וטילים",
    "ירי רקטות וטילים - היכנסו למרחב המוגן",
];

const HOSTILE_AIRCRAFT_KEYS: &[&str] = &["חדירת כלי טיס עוין"];

const END_KEYS: &[&str] = &[
    "ירי רקטות וטילים - האירוע הסתיים",
    "האירוע הסתיים",
    "ניתן לצאת מהמרחב המוגן אך יש להישאר בקרבתו",
    "סיום שהייה בסמיכות למרחב מוגן",
];

pub fn resolve_message_type(title: &str) -> &'static str {
    if PRE_ALERT_KEYS.contains(&title) {
        MESSAGE_TYPE_PRE_ALERT
    } else if RED_ALERT_KEYS.contains(&title) {
        MESSAGE_TYPE_RED_ALERT
    } else if HOSTILE_AIRCRAFT_KEYS.contains(&title) {
        MESSAGE_TYPE_HOSTILE_AIRCRAFT
    } else if END_KEYS.contains(&title) {
        MESSAGE_TYPE_END
    } else {
        MESSAGE_TYPE_UNCONFIGURED
    }
}

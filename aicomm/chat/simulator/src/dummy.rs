use fake::Dummy;
use rand::seq::SliceRandom as _;

pub struct AppVersion;
pub struct SystemOs;
pub struct SystemArch;
pub struct SystemLocale;
pub struct SystemTimezone;
pub struct RegionName;
pub struct MessageType;

impl Dummy<AppVersion> for String {
    fn dummy_with_rng<R: rand::Rng + ?Sized>(_: &AppVersion, rng: &mut R) -> Self {
        // generate x.y.z
        let major = rng.gen_range(1..=4);
        let minor = rng.gen_range(0..=99);
        let patch = rng.gen_range(0..=99);
        format!("{}.{}.{}", major, minor, patch)
    }
}

impl Dummy<SystemOs> for String {
    fn dummy_with_rng<R: rand::Rng + ?Sized>(_: &SystemOs, rng: &mut R) -> Self {
        let os = ["macOS", "Linux", "Windows", "iOS", "Android"]
            .choose(rng)
            .unwrap();
        os.to_string()
    }
}

impl Dummy<SystemArch> for String {
    fn dummy_with_rng<R: rand::Rng + ?Sized>(_: &SystemArch, rng: &mut R) -> Self {
        let arch = ["x86_64", "aarch64"].choose(rng).unwrap();
        arch.to_string()
    }
}

impl Dummy<SystemLocale> for String {
    fn dummy_with_rng<R: rand::Rng + ?Sized>(_: &SystemLocale, rng: &mut R) -> Self {
        let locale = [
            "en_US", "en_GB", "fr_FR", "ru_RU", "zh_CN", "ja_JP", "ko_KR", "zh_TW", "zh_HK",
        ]
            .choose(rng)
            .unwrap();
        locale.to_string()
    }
}

impl Dummy<SystemTimezone> for String {
    fn dummy_with_rng<R: rand::Rng + ?Sized>(_: &SystemTimezone, rng: &mut R) -> Self {
        let timezone = [
            "America/New_York",
            "America/Los_Angeles",
            "America/Chicago",
            "America/Denver",
            "America/Phoenix",
            "America/Anchorage",
            "America/Adak",
            "Pacific/Honolulu",
            "Pacific/Midway",
            "Pacific/Guam",
            "Pacific/Tongatapu",
            "Pacific/Auckland",
            "Pacific/Fiji",
            "Pacific/Chatham",
            "Pacific/Norfolk",
            "Pacific/Kiritimati",
            "Asia/Tokyo",
            "Asia/Shanghai",
            "Asia/Dubai",
            "Asia/Kolkata",
            "Asia/Colombo",
            "Asia/Kathmandu",
            "Asia/Dhaka",
            "Asia/Almaty",
            "Asia/Bangkok",
            "Asia/Ho_Chi_Minh",
            "Asia/Jakarta",
            "Asia/Karachi",
            "Asia/Tehran",
            "Asia/Baghdad",
            "Europe/London",
            "Europe/Paris",
            "Europe/Berlin",
            "Europe/Moscow",
            "Europe/Istanbul",
            "Europe/Athens",
            "Europe/Zurich",
            "Europe/Stockholm",
            "Europe/Oslo",
            "Europe/Helsinki",
            "Europe/Bucharest",
            "Europe/Sofia",
            "Europe/Athens",
            "Europe/Istanbul",
            "Europe/Athens",
            "Europe/Zurich",
            "Europe/Stockholm",
            "Europe/Oslo",
            "Europe/Helsinki",
            "Europe/Bucharest",
            "Europe/Sofia",
            "Europe/Athens",
            "Europe/Istanbul",
            "Europe/Athens",
            "Europe/Zurich",
            "Europe/Stockholm",
            "Europe/Oslo",
            "Europe/Helsinki",
            "Europe/Bucharest",
            "Europe/Sofia",
            "Europe/Athens",
            "Europe/Istanbul",
            "Europe/Athens",
            "Europe/Zurich",
            "Europe/Stockholm",
            "Europe/Oslo",
            "Europe/Helsinki",
        ]
            .choose(rng)
            .unwrap();
        timezone.to_string()
    }
}

impl Dummy<RegionName> for String {
    fn dummy_with_rng<R: rand::Rng + ?Sized>(_: &RegionName, rng: &mut R) -> Self {
        let region = [
            "New York", "Los Angeles", "Chicago", "Denver", "Phoenix", "Anchorage", "Adak",
            "London", "Paris", "Berlin", "Madrid", "Shanghai", "Tokyo", "Seoul", "Hong Kong",
            "Singapore", "Dubai", "Istanbul", "Kolkata", "Kuala Lumpur", "Taipei", "Seoul",
            "Shanghai", "Tokyo", "Hong Kong", "Singapore", "Dubai", "Istanbul", "Kolkata",
            "Kuala Lumpur", "Taipei",
        ]
            .choose(rng)
            .unwrap();
        region.to_string()
    }
}

impl Dummy<MessageType> for String {
    fn dummy_with_rng<R: rand::Rng + ?Sized>(_: &MessageType, rng: &mut R) -> Self {
        let message_type = ["text", "image", "video", "audio"]
            .choose(rng)
            .unwrap();
        message_type.to_string()
    }
}
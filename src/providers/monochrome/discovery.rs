pub const INSTANCES: [&str; 11] = [
    "https://hifi-api-workers.bgkhthyhvf.workers.dev",
    "https://api.monochrome.tf",
    "https://monochrome-api.samidy.com",
    "https://eu-central.monochrome.tf",
    "https://us-west.monochrome.tf",
    "https://arran.monochrome.tf",
    "https://hifi.geeked.wtf",
    "https://wolf.qqdl.site",
    "https://maus.qqdl.site",
    "https://vogel.qqdl.site",
    "https://katze.qqdl.site",
];

pub fn default_instances() -> &'static [&'static str] {
    &INSTANCES
}

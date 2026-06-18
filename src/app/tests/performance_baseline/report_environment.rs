use std::env;
use std::fs;
use std::thread;

pub(crate) fn environment_json(storage_mode: &str, release_evidence: bool) -> String {
    let generated_at = optional_env("ETEREA_PERF_GENERATED_AT_UTC", "not-recorded");
    let kernel = optional_env("ETEREA_PERF_KERNEL", "unknown");
    let rustc = optional_env("ETEREA_PERF_RUSTC", "unknown");
    format!(
        concat!(
            "{{\"generated_at_utc\":\"{}\",\"os\":\"{}\",\"kernel\":\"{}\",",
            "\"machine\":\"{}\",\"rustc\":\"{}\",\"cargo_profile\":\"test\",",
            "\"storage_mode\":\"{}\",\"release_evidence\":{}}}"
        ),
        json_escape(&generated_at),
        env::consts::OS,
        json_escape(&kernel),
        env::consts::ARCH,
        json_escape(&rustc),
        json_escape(storage_mode),
        release_evidence
    )
}

pub(crate) fn hardware_json() -> String {
    let logical_cpus = thread::available_parallelism()
        .map(|count| count.get().to_string())
        .unwrap_or_else(|_| "unknown".to_string());
    format!(
        concat!(
            "{{\"cpu_model\":\"{}\",\"logical_cpus\":\"{}\",",
            "\"memory_kb\":\"{}\"}}"
        ),
        json_escape(&cpu_model()),
        json_escape(&logical_cpus),
        json_escape(&memory_kb())
    )
}

fn cpu_model() -> String {
    fs::read_to_string("/proc/cpuinfo")
        .ok()
        .and_then(|contents| {
            contents.lines().find_map(|line| {
                line.strip_prefix("model name").and_then(|value| {
                    value
                        .split_once(':')
                        .map(|(_, model)| model.trim().to_string())
                })
            })
        })
        .unwrap_or_else(|| optional_env("ETEREA_PERF_CPU_MODEL", "unknown"))
}

fn memory_kb() -> String {
    fs::read_to_string("/proc/meminfo")
        .ok()
        .and_then(|contents| {
            contents.lines().find_map(|line| {
                line.strip_prefix("MemTotal:")
                    .map(|value| value.trim().to_string())
            })
        })
        .unwrap_or_else(|| optional_env("ETEREA_PERF_MEMORY_KB", "unknown"))
}

fn json_escape(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

fn optional_env(name: &str, fallback: &str) -> String {
    match env::var(name) {
        Ok(value) => value,
        Err(_) => fallback.to_string(),
    }
}

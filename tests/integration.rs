use std::path::PathBuf;
use std::sync::Once;
use std::time::Instant;

static BUILD: Once = Once::new();

fn binary_path() -> PathBuf {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target")
        .join("release")
        .join("tty");
    BUILD.call_once(|| {
        let status = std::process::Command::new("cargo")
            .args(["build", "--release"])
            .status()
            .expect("failed to build");
        assert!(status.success(), "cargo build --release failed");
    });
    path
}

fn run_tty(query: &str) -> (String, std::time::Duration) {
    let api_key =
        std::env::var("GROQ_API_KEY").expect("GROQ_API_KEY must be set for integration tests");
    let bin = binary_path();

    let start = Instant::now();
    let output = std::process::Command::new(&bin)
        .arg(query)
        .env("GROQ_API_KEY", &api_key)
        .output()
        .expect("failed to run tty");
    let duration = start.elapsed();

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    assert!(
        output.status.success(),
        "tty failed on \"{query}\": {}",
        String::from_utf8_lossy(&output.stderr)
    );

    (stdout, duration)
}

fn run_tty_think(query: &str) -> (String, std::time::Duration) {
    let api_key =
        std::env::var("GROQ_API_KEY").expect("GROQ_API_KEY must be set for integration tests");
    let bin = binary_path();

    let start = Instant::now();
    let output = std::process::Command::new(&bin)
        .args(["--think", query])
        .env("GROQ_API_KEY", &api_key)
        .output()
        .expect("failed to run tty");
    let duration = start.elapsed();

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    assert!(
        output.status.success(),
        "tty --think failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    (stdout, duration)
}

const FAST_CASES: &[(&str, &[&str])] = &[
    ("list files in current directory", &["ls"]),
    ("show running docker containers", &["docker ps", "docker container"]),
    ("find all mp4 files on this system", &["find", "mp4"]),
    ("show disk usage sorted by size", &["du", "sort"]),
    ("what is my public ip", &["curl"]),
    ("count lines in all rust files", &["wc", ".rs"]),
    ("show system memory usage", &["free", "mem"]),
];

#[test]
#[ignore]
fn fast_mode_commands() {
    println!("\n--- Fast Mode Integration Tests ---");
    let mut total_ms = 0u128;

    for (i, (prompt, expected_substrings)) in FAST_CASES.iter().enumerate() {
        if i > 0 {
            std::thread::sleep(std::time::Duration::from_millis(500));
        }
        let (command, duration) = run_tty(prompt);
        let ms = duration.as_millis();
        total_ms += ms;

        let has_match = expected_substrings
            .iter()
            .any(|sub| command.to_lowercase().contains(sub));
        println!("  [{ms:>5}ms] \"{prompt}\" -> {command}");
        assert!(
            has_match,
            "command '{command}' does not contain any of {expected_substrings:?} for prompt '{prompt}'"
        );
    }

    let avg_ms = total_ms / FAST_CASES.len() as u128;
    println!("\n  Average: {avg_ms}ms across {} tests", FAST_CASES.len());
}

#[test]
#[ignore]
fn fast_mode_under_500ms() {
    // Warm up
    let _ = run_tty("list files");

    let (_, duration) = run_tty("list files");
    assert!(
        duration.as_millis() < 500,
        "fast mode took {}ms, expected <500ms",
        duration.as_millis()
    );
}

#[test]
#[ignore]
fn think_mode_works() {
    let (command, duration) = run_tty_think("set up a reverse ssh tunnel to 10.0.0.5");
    println!(
        "  [{:>5}ms] think mode -> {command}",
        duration.as_millis()
    );
    assert!(
        command.to_lowercase().contains("ssh"),
        "think mode command should contain 'ssh', got: {command}"
    );
}

#[test]
#[ignore]
fn pipe_mode_no_ui() {
    let (stdout, _) = run_tty("list files");
    assert!(!stdout.is_empty(), "pipe mode should output the command");
    assert!(
        !stdout.contains("[enter]"),
        "pipe mode should not contain UI prompts"
    );
}

#[test]
#[ignore]
fn speed_benchmark() {
    let prompts = [
        "list files",
        "show disk space",
        "find png files",
        "kill process on port 8080",
        "compress this directory",
    ];

    // Warm up
    let _ = run_tty("list files");

    println!("\n--- Speed Benchmark ---");
    let mut times = Vec::new();
    for (i, prompt) in prompts.iter().enumerate() {
        if i > 0 {
            std::thread::sleep(std::time::Duration::from_millis(500));
        }
        let (command, duration) = run_tty(prompt);
        let ms = duration.as_millis();
        times.push(ms);
        println!("  [{ms:>5}ms] \"{prompt}\" -> {command}");
    }

    let avg = times.iter().sum::<u128>() / times.len() as u128;
    let max = *times.iter().max().unwrap();
    let min = *times.iter().min().unwrap();
    println!("\n  Min: {min}ms | Avg: {avg}ms | Max: {max}ms");
    println!("  Target: <500ms");

    assert!(avg < 1000, "average latency {avg}ms is too high");
}

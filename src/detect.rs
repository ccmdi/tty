use std::path::PathBuf;

pub struct Environment {
    pub os: String,
    pub shell: String,
    pub tools: Vec<String>,
}

const KNOWN_TOOLS: &[(&str, &str)] = &[
    ("es", "Everything Search CLI for fast file lookup"),
    ("fd", "fast find alternative"),
    ("rg", "ripgrep, fast grep alternative"),
    ("fzf", "fuzzy finder"),
    ("jq", "JSON processor"),
    ("yq", "YAML processor"),
    ("bat", "cat with syntax highlighting"),
    ("eza", "modern ls replacement"),
    ("zoxide", "smart cd"),
    ("ffmpeg", "media processing"),
    ("docker", "container runtime"),
    ("kubectl", "Kubernetes CLI"),
    ("git", "version control"),
    ("curl", "HTTP client"),
    ("wget", "HTTP downloader"),
    ("htop", "interactive process viewer"),
    ("tmux", "terminal multiplexer"),
];

pub fn detect() -> Environment {
    Environment {
        os: detect_os(),
        shell: detect_shell(),
        tools: detect_tools(),
    }
}

fn detect_os() -> String {
    if cfg!(target_os = "windows") {
        return "windows".into();
    }
    // Check for WSL
    if std::fs::read_to_string("/proc/version")
        .map(|v| v.to_lowercase().contains("microsoft"))
        .unwrap_or(false)
    {
        return "linux (WSL2)".into();
    }
    if cfg!(target_os = "macos") {
        "macos".into()
    } else {
        "linux".into()
    }
}

fn detect_shell() -> String {
    std::env::var("SHELL")
        .ok()
        .and_then(|s| PathBuf::from(&s).file_name().map(|n| n.to_string_lossy().into_owned()))
        .unwrap_or_else(|| "sh".into())
}

fn detect_tools() -> Vec<String> {
    let path_var = std::env::var("PATH").unwrap_or_default();
    let dirs: Vec<PathBuf> = std::env::split_paths(&path_var).collect();

    KNOWN_TOOLS
        .iter()
        .filter(|(name, _)| {
            dirs.iter().any(|dir| {
                let p = dir.join(name);
                p.exists() || dir.join(format!("{name}.exe")).exists()
            })
        })
        .map(|(name, _)| name.to_string())
        .collect()
}

pub fn tool_description(name: &str) -> Option<&'static str> {
    KNOWN_TOOLS
        .iter()
        .find(|(n, _)| *n == name)
        .map(|(_, desc)| *desc)
}

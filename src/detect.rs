use std::path::PathBuf;

pub struct Environment {
    pub os: String,
    pub shell: String,
    pub tools: Vec<String>,
}

const KNOWN_TOOLS: &[(&str, &str)] = &[
    ("es", "Everything Search CLI (voidtools). Instantly finds files/folders by name across all drives. Usage: `es <search text>` for basic search, `es /ad <text>` for folders only, `es /a-d <text>` for files only, `es -n 10 <text>` to limit results, `es ext:py <text>` to filter by extension. Supports Everything search syntax. Extremely fast, prefer over find/locate."),
    ("fd", "fast find alternative. Usage: `fd <pattern>` to search, `fd -t d <pattern>` for directories, `fd -e rs` for extension filter"),
    ("rg", "ripgrep, fast grep alternative. Usage: `rg <pattern>` to search file contents, `rg -l <pattern>` for filenames only, `rg --type rust <pattern>` for file type filter"),
    ("fzf", "fuzzy finder for interactive selection. Pipe results into it: `cmd | fzf`"),
    ("jq", "JSON processor. Usage: `jq '.field' file.json`"),
    ("yq", "YAML processor"),
    ("bat", "cat with syntax highlighting. Usage: `bat <file>`"),
    ("eza", "modern ls replacement. Usage: `eza -la`"),
    ("zoxide", "smart cd based on frecency. Usage: `z <partial-path>`"),
    ("ffmpeg", "media processing. Usage: `ffmpeg -i input.mp4 output.mkv`"),
    ("docker", "container runtime"),
    ("kubectl", "Kubernetes CLI"),
    ("git", "version control"),
    ("curl", "HTTP client. Usage: `curl <url>`"),
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

fn is_local_path(dir: &PathBuf) -> bool {
    let s = dir.to_string_lossy();
    !s.starts_with("/mnt/")
}

fn detect_tools() -> Vec<String> {
    let path_var = std::env::var("PATH").unwrap_or_default();
    let dirs: Vec<PathBuf> = std::env::split_paths(&path_var)
        .filter(|d| is_local_path(d))
        .collect();

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

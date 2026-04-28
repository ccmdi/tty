# tty

Natural language to shell commands. Describe what you want, get a command instantly, confirm, run.

## Usage

```
tty "find all rust files over 1MB"
> find . -name '*.rs' -size +1M  [enter] run  [esc] cancel
```

### Examples

```bash
# file operations
tty "delete all .tmp files in this directory"
tty "rename all .jpeg files to .jpg"
tty "find the 10 largest files in this project"

# git
tty "undo my last commit but keep the changes"
tty "show commits from this week that touched src/"
tty "diff what I have staged"

# system
tty "what's using port 3000"
tty "how much disk space is left"
tty "kill all node processes"

# data processing
tty "extract emails from log.txt"
tty "convert data.csv to json"
tty "count lines of code by file extension"

# networking
tty "download this url and save as page.html"
tty "check if example.com is responding"
```

### Research mode

For tasks that need context from your system, `-r` lets the model inspect your environment before suggesting a command (read-only, sandboxed):

```bash
tty -r "open my most recent pdf"
tty -r "which python package is biggest"
tty -r "what branch has the most recent commit"
```

### Pipe mode

When piped, outputs just the raw command with no UI:

```bash
tty "list files" | sh
tty "find todo comments" | pbcopy
```

### Flags

| Flag | Description |
|---|---|
| `--think` | Extended reasoning before suggesting a command |
| `-r` / `--research` | Multi-turn mode: inspects your system first |
| `--show-reasoning` | Display the model's explanation |
| `--debug` | Show timing diagnostics |

### Zsh integration

Add this to your `.zshrc` to translate natural language inline with Ctrl+G:

```zsh
tty-widget() {
  [[ -z "$BUFFER" ]] && return
  local result=$(tty "$BUFFER" 2>/dev/null)
  if [[ -n "$result" ]]; then
    BUFFER="$result"
    CURSOR=${#BUFFER}
  fi
  zle redisplay
}
zle -N tty-widget
bindkey '^G' tty-widget
```

Type `find all rust files` on your command line, press Ctrl+G, and your input gets replaced with the actual command ready to review and run.

## Setup

```bash
cargo install --path .
```

Either set an environment variable:

```bash
export GROQ_API_KEY="your-key-here"
```

Or generate a config file:

```bash
tty init
# edit ~/.config/tty/config.toml with your API key
```

Supports Groq and Ollama backends. Auto-detects your OS, shell, and installed tools (fd, rg, jq, etc.) to tailor suggestions to your environment.

> [!IMPORTANT]
> This is a proof of concept.

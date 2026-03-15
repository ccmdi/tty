use crate::config::ContextConfig;
use crate::detect;

pub fn build_system_prompt(context: &ContextConfig) -> String {
    let mut parts = vec![
        "You are a shell command assistant. \
         When the user describes what they want to do, use the run_command tool to suggest the appropriate shell command. \
         The command should be ready to execute as-is. \
         Do not include explanations in the command. \
         If the request is ambiguous, make a reasonable assumption. \
         Never suggest commands that delete files without confirmation flags. \
         Never use sudo unless the user explicitly asks."
            .to_string(),
    ];

    if let Some(os) = &context.os {
        let shell = context.shell.as_deref().unwrap_or("sh");
        parts.push(format!("Environment: {os}, shell: {shell}."));
    }

    if !context.tools.is_empty() {
        let tool_list: Vec<String> = context
            .tools
            .iter()
            .map(|name| {
                if let Some(desc) = detect::tool_description(name) {
                    format!("{name} ({desc})")
                } else {
                    name.clone()
                }
            })
            .collect();
        parts.push(format!(
            "Available tools beyond coreutils: {}. Prefer these over slower alternatives when appropriate.",
            tool_list.join(", ")
        ));
    }

    parts.join("\n")
}

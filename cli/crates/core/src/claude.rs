//! Claude Code command builder
//!
//! Provides a builder pattern for constructing Claude Code CLI commands.

/// Claude Code command builder
#[derive(Debug, Default, Clone)]
pub struct ClaudeCommand {
    /// Allowed tools (e.g., "Read", "Write", "Bash")
    pub allowed_tools: Vec<String>,
    /// Disallowed tools
    pub disallowed_tools: Vec<String>,
    /// Model to use (e.g., "sonnet", "opus")
    pub model: Option<String>,
    /// Resume a previous conversation by ID
    pub resume: Option<String>,
    /// Initial prompt to send
    pub prompt: Option<String>,
    /// Additional CLI arguments
    pub extra_args: Vec<String>,
}

impl ClaudeCommand {
    /// Create a new command builder
    pub fn new() -> Self {
        Self::default()
    }

    /// Set allowed tools
    pub fn allowed_tools(mut self, tools: Vec<String>) -> Self {
        self.allowed_tools = tools;
        self
    }

    /// Set disallowed tools
    pub fn disallowed_tools(mut self, tools: Vec<String>) -> Self {
        self.disallowed_tools = tools;
        self
    }

    /// Set the model to use
    pub fn model(mut self, model: impl Into<String>) -> Self {
        self.model = Some(model.into());
        self
    }

    /// Resume a previous conversation
    #[allow(dead_code)]
    pub fn resume(mut self, id: impl Into<String>) -> Self {
        self.resume = Some(id.into());
        self
    }

    /// Set the initial prompt
    pub fn prompt(mut self, prompt: impl Into<String>) -> Self {
        self.prompt = Some(prompt.into());
        self
    }

    /// Add an extra argument
    pub fn extra_arg(mut self, arg: impl Into<String>) -> Self {
        self.extra_args.push(arg.into());
        self
    }

    /// Build the command string to execute
    pub fn build(&self) -> String {
        let mut parts = vec!["claude".to_string()];

        if !self.allowed_tools.is_empty() {
            parts.push("--allowedTools".to_string());
            parts.push(self.allowed_tools.join(","));
        }

        if !self.disallowed_tools.is_empty() {
            parts.push("--disallowedTools".to_string());
            parts.push(self.disallowed_tools.join(","));
        }

        if let Some(model) = &self.model {
            parts.push("--model".to_string());
            parts.push(model.clone());
        }

        if let Some(resume) = &self.resume {
            parts.push("--resume".to_string());
            parts.push(resume.clone());
        }

        for arg in &self.extra_args {
            parts.push(arg.clone());
        }

        // Prompt goes last if present (as a positional argument)
        // Use single quotes for shell safety (handles newlines, $, `, etc.)
        if let Some(prompt) = &self.prompt {
            let escaped = prompt.replace('\'', "'\\''");
            parts.push(format!("'{}'", escaped));
        }

        parts.join(" ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_command() {
        let cmd = ClaudeCommand::new().build();
        assert_eq!(cmd, "claude");
    }

    #[test]
    fn test_with_model() {
        let cmd = ClaudeCommand::new().model("opus").build();
        assert_eq!(cmd, "claude --model opus");
    }

    #[test]
    fn test_with_tools() {
        let cmd = ClaudeCommand::new()
            .allowed_tools(vec!["Read".to_string(), "Write".to_string()])
            .build();
        assert_eq!(cmd, "claude --allowedTools Read,Write");
    }

    #[test]
    fn test_full_command() {
        let cmd = ClaudeCommand::new()
            .model("sonnet")
            .allowed_tools(vec!["Read".to_string()])
            .prompt("Hello")
            .build();
        assert_eq!(cmd, "claude --allowedTools Read --model sonnet 'Hello'");
    }
}

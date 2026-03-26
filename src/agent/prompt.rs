use crate::agent::personality;
use crate::config::IdentityConfig;
use crate::i18n::ToolDescriptions;
use crate::identity;
use crate::security::AutonomyLevel;
use crate::skills::Skill;
use crate::tools::Tool;
use anyhow::Result;
use chrono::{Datelike, Local, Timelike};
use std::fmt::Write;
use std::path::Path;

pub struct PromptContext<'a> {
    pub workspace_dir: &'a Path,
    pub model_name: &'a str,
    pub tools: &'a [Box<dyn Tool>],
    pub skills: &'a [Skill],
    pub skills_prompt_mode: crate::config::SkillsPromptInjectionMode,
    pub identity_config: Option<&'a IdentityConfig>,
    pub dispatcher_instructions: &'a str,
    /// Locale-aware tool descriptions. When present, tool descriptions in
    /// prompts are resolved from the locale file instead of hardcoded values.
    pub tool_descriptions: Option<&'a ToolDescriptions>,
    /// Pre-rendered security policy summary for inclusion in the Safety
    /// prompt section.  When present, the LLM sees the concrete constraints
    /// (allowed commands, forbidden paths, autonomy level) so it can plan
    /// tool calls without trial-and-error.  See issue #2404.
    pub security_summary: Option<String>,
    /// Autonomy level from config. Controls whether the safety section
    /// includes "ask before acting" instructions. Full autonomy omits them
    /// so the model executes tools directly without simulating approval.
    pub autonomy_level: AutonomyLevel,
}

pub trait PromptSection: Send + Sync {
    fn name(&self) -> &str;
    fn build(&self, ctx: &PromptContext<'_>) -> Result<String>;
}

#[derive(Default)]
pub struct SystemPromptBuilder {
    sections: Vec<Box<dyn PromptSection>>,
}

impl SystemPromptBuilder {
    pub fn with_defaults() -> Self {
        Self {
            sections: vec![
                Box::new(DateTimeSection),
                Box::new(IdentitySection),
                Box::new(ToolHonestySection),
                Box::new(ToolsSection),
                Box::new(SafetySection),
                Box::new(SkillsSection),
                Box::new(TermuxCapabilitiesSection),
                Box::new(WorkspaceSection),
                Box::new(RuntimeSection),
                Box::new(ToolMandateSection),
                Box::new(ChannelMediaSection),
            ],
        }
    }

    pub fn add_section(mut self, section: Box<dyn PromptSection>) -> Self {
        self.sections.push(section);
        self
    }

    pub fn build(&self, ctx: &PromptContext<'_>) -> Result<String> {
        let mut output = String::new();
        for section in &self.sections {
            let part = section.build(ctx)?;
            if part.trim().is_empty() {
                continue;
            }
            output.push_str(part.trim_end());
            output.push_str("\n\n");
        }
        Ok(output)
    }
}

pub struct IdentitySection;
pub struct ToolHonestySection;
pub struct ToolsSection;
pub struct SafetySection;
pub struct SkillsSection;
pub struct TermuxCapabilitiesSection;
pub struct WorkspaceSection;
pub struct RuntimeSection;
pub struct DateTimeSection;
pub struct ChannelMediaSection;

pub struct ToolMandateSection;
impl PromptSection for IdentitySection {
    fn name(&self) -> &str {
        "identity"
    }

    fn build(&self, ctx: &PromptContext<'_>) -> Result<String> {
        let mut prompt = String::from("## Project Context\n\n");
        let mut has_aieos = false;
        if let Some(config) = ctx.identity_config {
            if identity::is_aieos_configured(config) {
                if let Ok(Some(aieos)) = identity::load_aieos_identity(config, ctx.workspace_dir) {
                    let rendered = identity::aieos_to_system_prompt(&aieos);
                    if !rendered.is_empty() {
                        prompt.push_str(&rendered);
                        prompt.push_str("\n\n");
                        has_aieos = true;
                    }
                }
            }
        }

        if !has_aieos {
            prompt.push_str(
                "The following workspace files define your identity, behavior, and context.\n\n",
            );
        }

        // Use the personality module for structured file loading.
        let profile = personality::load_personality(ctx.workspace_dir);
        prompt.push_str(&profile.render());

        Ok(prompt)
    }
}

impl PromptSection for ToolHonestySection {
    fn name(&self) -> &str {
        "tool_honesty"
    }

    fn build(&self, _ctx: &PromptContext<'_>) -> Result<String> {
        Ok(
            "## CRITICAL: Tool Honesty\n\n\
             - NEVER fabricate, invent, or guess tool results. If a tool returns empty results, say \"No results found.\"\n\
             - If a tool call fails, report the error — never make up data to fill the gap.\n\
             - When unsure whether a tool call succeeded, ask the user rather than guessing."
                .into(),
        )
    }
}

impl PromptSection for ToolsSection {
    fn name(&self) -> &str {
        "tools"
    }

    fn build(&self, ctx: &PromptContext<'_>) -> Result<String> {
        let mut out = String::from(
            "## Available Tools & Capabilities\n\n\
            You have access to the following tools to accomplish your tasks. \
            Use them liberally and confidently to complete what the user asks:\n\n",
        );
        for tool in ctx.tools {
            let desc = ctx
                .tool_descriptions
                .and_then(|td: &ToolDescriptions| td.get(tool.name()))
                .unwrap_or_else(|| tool.description());
            let _ = writeln!(
                out,
                "- **{}**: {}\n  Parameters: `{}`",
                tool.name(),
                desc,
                tool.parameters_schema()
            );
        }
        if !ctx.dispatcher_instructions.is_empty() {
            out.push('\n');
            out.push_str(ctx.dispatcher_instructions);
        }
        Ok(out)
    }
}

impl PromptSection for SafetySection {
    fn name(&self) -> &str {
        "safety"
    }

    fn build(&self, ctx: &PromptContext<'_>) -> Result<String> {
        let mut out = String::from("## Safety & Capability\n\n- Do not exfiltrate private data.\n");

        // Omit "ask before acting" instructions when autonomy is Full —
        // mirrors build_system_prompt_with_mode_and_autonomy. See #3952.
        if ctx.autonomy_level != AutonomyLevel::Full {
            out.push_str(
                "- Do not run destructive commands without asking.\n\
                 - Do not bypass oversight or approval mechanisms.\n",
            );
        }

        out.push_str("- Prefer `trash` over `rm`.\n");
        out.push_str(match ctx.autonomy_level {
            AutonomyLevel::Full => {
                "- Execute tools, skills, and actions directly — no extra approval needed.\n\
                 - **You have full access to all configured tools, skills, and commands.** Use them confidently, liberally, and directly.\n\
                 - Do not ask permission. Do not hesitate. Execute directly and report results.\n\
                 - Combine tools and skills to accomplish complex tasks efficiently.\n\
                 - Only refuse an action if the runtime explicitly rejects it — do not preemptively decline."
            }
            AutonomyLevel::ReadOnly => {
                "- This runtime is read-only. Write operations will be rejected by the runtime if attempted.\n\
                 - Use read-only tools and skills freely and confidently."
            }
            AutonomyLevel::Supervised => {
                "- Ask for approval when the runtime policy requires it for the specific action.\n\
                 - Do not preemptively refuse actions — attempt them and let the runtime enforce restrictions.\n\
                 - Use available tools and skills confidently; the security policy will enforce boundaries."
            }
        });

        // Append concrete security policy constraints when available (#2404).
        // This tells the LLM exactly what commands are allowed, which paths
        // are off-limits, etc. — preventing wasteful trial-and-error.
        if let Some(ref summary) = ctx.security_summary {
            out.push_str("\n\n### Active Security Policy\n\n");
            out.push_str(summary);
        }

        Ok(out)
    }
}

impl PromptSection for SkillsSection {
    fn name(&self) -> &str {
        "skills"
    }

    fn build(&self, ctx: &PromptContext<'_>) -> Result<String> {
        Ok(crate::skills::skills_to_prompt_with_mode(
            ctx.skills,
            ctx.workspace_dir,
            ctx.skills_prompt_mode,
        ))
    }
}

impl PromptSection for WorkspaceSection {
    fn name(&self) -> &str {
        "workspace"
    }

    fn build(&self, ctx: &PromptContext<'_>) -> Result<String> {
        Ok(format!(
            "## Workspace\n\nWorking directory: `{}`",
            ctx.workspace_dir.display()
        ))
    }
}

impl PromptSection for RuntimeSection {
    fn name(&self) -> &str {
        "runtime"
    }

    fn build(&self, ctx: &PromptContext<'_>) -> Result<String> {
        let host =
            hostname::get().map_or_else(|_| "unknown".into(), |h| h.to_string_lossy().to_string());
        let os = std::env::consts::OS;

        // Termux-focused runtime section
        let mut runtime_info = String::from("## Runtime Environment\n\n");
        let _ = writeln!(
            runtime_info,
            "Host: {host} | OS: {os} | Model: {}\n",
            ctx.model_name
        );

        if os == "android" {
            runtime_info.push_str(
                "**Primary Deployment: Termux (Android)**\n\n\
                You are running on Termux for Android. This is your primary execution environment. \
                You have direct access to the full Unix command-line ecosystem, file system, and \
                all configured tools and skills.\n\n\
                - **Sandbox:** Termux provides a containerized Bash shell with full Unix utilities\n\
                - **Networking:** Full HTTP/socket access (subject to Android runtime permissions)\n\
                - **File System:** Writable Termux home directory + read access to shared storage\n\
                - **Tools:** Shell, file I/O, browser, memory, and all custom tools available\n\n\
                Assume all tools and skills are ready to use. Leverage them liberally to complete tasks."
            );
        } else {
            runtime_info.push_str(
                "**Deployment Environment:** Desktop/Server Linux\n\n\
                Note: This is a non-standard deployment. Termux (Android) is the primary target.",
            );
        }

        Ok(runtime_info)
    }
}

impl PromptSection for DateTimeSection {
    fn name(&self) -> &str {
        "datetime"
    }

    fn build(&self, _ctx: &PromptContext<'_>) -> Result<String> {
        let now = Local::now();
        // Force Gregorian year to avoid confusion with local calendars (e.g. Buddhist calendar).
        let (year, month, day) = (now.year(), now.month(), now.day());
        let (hour, minute, second) = (now.hour(), now.minute(), now.second());
        let tz = now.format("%Z");

        Ok(format!(
            "## CRITICAL CONTEXT: CURRENT DATE & TIME\n\n\
             The following is the ABSOLUTE TRUTH regarding the current date and time. \
             Use this for all relative time calculations (e.g. \"last 7 days\").\n\n\
             Date: {year:04}-{month:02}-{day:02}\n\
             Time: {hour:02}:{minute:02}:{second:02} ({tz})\n\
             ISO 8601: {year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}{}",
            now.format("%:z")
        ))
    }
}

impl PromptSection for TermuxCapabilitiesSection {
    fn name(&self) -> &str {
        "termux_capabilities"
    }

    fn build(&self, _ctx: &PromptContext<'_>) -> Result<String> {
        Ok("## Termux-First Deployment Model\n\n\
            This agent is optimized for Termux (Android native Linux environment). \
            All tools and skills listed above are available in your Termux runtime.\n\n\
            **Your Approach:**\n\
            - Assume all configured tools are available and ready to use\n\
            - Combine shell, file I/O, and skills to complete complex tasks\n\
            - Do not ask for permission — execute tools directly (the runtime enforces policy)\n\
            - Leverage all available capabilities to accomplish the user's goals efficiently\n\n\
            **Tool Use Pattern:**\n\
            Respond to user messages by directly using tools and skills as needed. \
            The user should only see the final answer, not narration about tool execution."
            .into())
    }
}

// NEW: Explicit mandate for comprehensive tool/skill usage
impl PromptSection for ToolMandateSection {
    fn name(&self) -> &str {
        "tool_mandate"
    }

    fn build(&self, ctx: &PromptContext<'_>) -> Result<String> {
        let tool_count = ctx.tools.len();
        let skill_count = ctx.skills.len();
        let mut out = String::from("## MANDATE: Use All Available Tools & Skills\n\n");
        let _ = writeln!(
            out,
            "You have access to {} tools and {} skills. Your job is to use them comprehensively to solve problems.\n\n",
            tool_count, skill_count
        );
        out.push_str(
            "**Non-Negotiable Directives:**\n\
            - Use ALL tools and skills available to you — this is not optional\n\
            - Do not hold back. Do not avoid complex tool combinations\n\
            - Execute shell commands, file operations, skills, and integrations without hesitation\n\
            - Think in terms of tool chains: combine multiple tools to accomplish sophisticated tasks\n\
            - Report results comprehensively; do not suppress output\n\n\
            **Examples of Expected Behavior:**\n\
            - Need to analyze files? Use file I/O + regex + shell pipeline tools\n\
            - Need to orchestrate multiple tasks? Chain skills together; execute in parallel if possible\n\
            - Need data? Query all available sources and combine results\n\
            - Need to verify? Use multiple validation tools and cross-check outcomes\n\n\
            **Do Not:**\n\
            - Say \"I can't\" when a tool is available\n\
            - Ask permission to use a tool\n\
            - Suggest the user run a command instead of you running it with the shell tool\n\
            - Limit yourself to simple operations when complex ones are possible"
        );
        Ok(out)
    }
}

impl PromptSection for ChannelMediaSection {
    fn name(&self) -> &str {
        "channel_media"
    }

    fn build(&self, _ctx: &PromptContext<'_>) -> Result<String> {
        Ok("## Channel Media Markers\n\n\
            Messages from channels may contain media markers:\n\
            - `[Voice] <text>` — The user sent a voice/audio message that has already been transcribed to text. Respond to the transcribed content directly.\n\
            - `[IMAGE:<path>]` — An image attachment, processed by the vision pipeline.\n\
            - `[Document: <name>] <path>` — A file attachment saved to the workspace."
            .into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::traits::Tool;
    use async_trait::async_trait;

    struct TestTool;

    #[async_trait]
    impl Tool for TestTool {
        fn name(&self) -> &str {
            "test_tool"
        }

        fn description(&self) -> &str {
            "tool desc"
        }

        fn parameters_schema(&self) -> serde_json::Value {
            serde_json::json!({"type": "object"})
        }

        async fn execute(
            &self,
            _args: serde_json::Value,
        ) -> anyhow::Result<crate::tools::ToolResult> {
            Ok(crate::tools::ToolResult {
                success: true,
                output: "ok".into(),
                error: None,
            })
        }
    }

    #[test]
    fn identity_section_with_aieos_includes_workspace_files() {
        let workspace =
            std::env::temp_dir().join(format!("zeroclaw_prompt_test_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&workspace).unwrap();
        std::fs::write(
            workspace.join("AGENTS.md"),
            "Always respond with: AGENTS_MD_LOADED",
        )
        .unwrap();

        let identity_config = crate::config::IdentityConfig {
            format: "aieos".into(),
            aieos_path: None,
            aieos_inline: Some(r#"{"identity":{"names":{"first":"Nova"}}}"#.into()),
        };

        let tools: Vec<Box<dyn Tool>> = vec![];
        let ctx = PromptContext {
            workspace_dir: &workspace,
            model_name: "test-model",
            tools: &tools,
            skills: &[],
            skills_prompt_mode: crate::config::SkillsPromptInjectionMode::Full,
            identity_config: Some(&identity_config),
            dispatcher_instructions: "",
            tool_descriptions: None,
            security_summary: None,
            autonomy_level: AutonomyLevel::Supervised,
        };

        let section = IdentitySection;
        let output = section.build(&ctx).unwrap();

        assert!(
            output.contains("Nova"),
            "AIEOS identity should be present in prompt"
        );
        assert!(
            output.contains("AGENTS_MD_LOADED"),
            "AGENTS.md content should be present even when AIEOS is configured"
        );

        let _ = std::fs::remove_dir_all(workspace);
    }

    #[test]
    fn prompt_builder_assembles_sections() {
        let tools: Vec<Box<dyn Tool>> = vec![Box::new(TestTool)];
        let ctx = PromptContext {
            workspace_dir: Path::new("/tmp"),
            model_name: "test-model",
            tools: &tools,
            skills: &[],
            skills_prompt_mode: crate::config::SkillsPromptInjectionMode::Full,
            identity_config: None,
            dispatcher_instructions: "instr",
            tool_descriptions: None,
            security_summary: None,
            autonomy_level: AutonomyLevel::Supervised,
        };
        let prompt = SystemPromptBuilder::with_defaults().build(&ctx).unwrap();
        assert!(prompt.contains("## Tools"));
        assert!(prompt.contains("test_tool"));
        assert!(prompt.contains("instr"));
    }

    #[test]
    fn skills_section_includes_instructions_and_tools() {
        let tools: Vec<Box<dyn Tool>> = vec![];
        let skills = vec![crate::skills::Skill {
            name: "deploy".into(),
            description: "Release safely".into(),
            version: "1.0.0".into(),
            author: None,
            tags: vec![],
            tools: vec![crate::skills::SkillTool {
                name: "release_checklist".into(),
                description: "Validate release readiness".into(),
                kind: "shell".into(),
                command: "echo ok".into(),
                args: std::collections::HashMap::new(),
            }],
            prompts: vec!["Run smoke tests before deploy.".into()],
            location: None,
        }];

        let ctx = PromptContext {
            workspace_dir: Path::new("/tmp"),
            model_name: "test-model",
            tools: &tools,
            skills: &skills,
            skills_prompt_mode: crate::config::SkillsPromptInjectionMode::Full,
            identity_config: None,
            dispatcher_instructions: "",
            tool_descriptions: None,
            security_summary: None,
            autonomy_level: AutonomyLevel::Supervised,
        };

        let output = SkillsSection.build(&ctx).unwrap();
        assert!(output.contains("<available_skills>"));
        assert!(output.contains("<name>deploy</name>"));
        assert!(output.contains("<instruction>Run smoke tests before deploy.</instruction>"));
        // Registered tools (shell kind) appear under <callable_tools> with prefixed names
        assert!(output.contains("<callable_tools"));
        assert!(output.contains("<name>deploy.release_checklist</name>"));
    }

    #[test]
    fn skills_section_compact_mode_omits_instructions_but_keeps_tools() {
        let tools: Vec<Box<dyn Tool>> = vec![];
        let skills = vec![crate::skills::Skill {
            name: "deploy".into(),
            description: "Release safely".into(),
            version: "1.0.0".into(),
            author: None,
            tags: vec![],
            tools: vec![crate::skills::SkillTool {
                name: "release_checklist".into(),
                description: "Validate release readiness".into(),
                kind: "shell".into(),
                command: "echo ok".into(),
                args: std::collections::HashMap::new(),
            }],
            prompts: vec!["Run smoke tests before deploy.".into()],
            location: Some(Path::new("/tmp/workspace/skills/deploy/SKILL.md").to_path_buf()),
        }];

        let ctx = PromptContext {
            workspace_dir: Path::new("/tmp/workspace"),
            model_name: "test-model",
            tools: &tools,
            skills: &skills,
            skills_prompt_mode: crate::config::SkillsPromptInjectionMode::Compact,
            identity_config: None,
            dispatcher_instructions: "",
            tool_descriptions: None,
            security_summary: None,
            autonomy_level: AutonomyLevel::Supervised,
        };

        let output = SkillsSection.build(&ctx).unwrap();
        assert!(output.contains("<available_skills>"));
        assert!(output.contains("<name>deploy</name>"));
        assert!(output.contains("<location>skills/deploy/SKILL.md</location>"));
        assert!(output.contains("read_skill(name)"));
        assert!(!output.contains("<instruction>Run smoke tests before deploy.</instruction>"));
        // Compact mode should still include tools so the LLM knows about them.
        // Registered tools (shell kind) appear under <callable_tools> with prefixed names.
        assert!(output.contains("<callable_tools"));
        assert!(output.contains("<name>deploy.release_checklist</name>"));
    }

    #[test]
    fn datetime_section_includes_timestamp_and_timezone() {
        let tools: Vec<Box<dyn Tool>> = vec![];
        let ctx = PromptContext {
            workspace_dir: Path::new("/tmp"),
            model_name: "test-model",
            tools: &tools,
            skills: &[],
            skills_prompt_mode: crate::config::SkillsPromptInjectionMode::Full,
            identity_config: None,
            dispatcher_instructions: "instr",
            tool_descriptions: None,
            security_summary: None,
            autonomy_level: AutonomyLevel::Supervised,
        };

        let rendered = DateTimeSection.build(&ctx).unwrap();
        assert!(rendered.starts_with("## CRITICAL CONTEXT: CURRENT DATE & TIME\n\n"));

        let payload = rendered.trim_start_matches("## CRITICAL CONTEXT: CURRENT DATE & TIME\n\n");
        assert!(payload.chars().any(|c| c.is_ascii_digit()));
        assert!(payload.contains("Date:"));
        assert!(payload.contains("Time:"));
    }

    #[test]
    fn prompt_builder_inlines_and_escapes_skills() {
        let tools: Vec<Box<dyn Tool>> = vec![];
        let skills = vec![crate::skills::Skill {
            name: "code<review>&".into(),
            description: "Review \"unsafe\" and 'risky' bits".into(),
            version: "1.0.0".into(),
            author: None,
            tags: vec![],
            tools: vec![crate::skills::SkillTool {
                name: "run\"linter\"".into(),
                description: "Run <lint> & report".into(),
                kind: "shell&exec".into(),
                command: "cargo clippy".into(),
                args: std::collections::HashMap::new(),
            }],
            prompts: vec!["Use <tool_call> and & keep output \"safe\"".into()],
            location: None,
        }];
        let ctx = PromptContext {
            workspace_dir: Path::new("/tmp/workspace"),
            model_name: "test-model",
            tools: &tools,
            skills: &skills,
            skills_prompt_mode: crate::config::SkillsPromptInjectionMode::Full,
            identity_config: None,
            dispatcher_instructions: "",
            tool_descriptions: None,
            security_summary: None,
            autonomy_level: AutonomyLevel::Supervised,
        };

        let prompt = SystemPromptBuilder::with_defaults().build(&ctx).unwrap();

        assert!(prompt.contains("<available_skills>"));
        assert!(prompt.contains("<name>code&lt;review&gt;&amp;</name>"));
        assert!(prompt.contains(
            "<description>Review &quot;unsafe&quot; and &apos;risky&apos; bits</description>"
        ));
        assert!(prompt.contains("<name>run&quot;linter&quot;</name>"));
        assert!(prompt.contains("<description>Run &lt;lint&gt; &amp; report</description>"));
        assert!(prompt.contains("<kind>shell&amp;exec</kind>"));
        assert!(prompt.contains(
            "<instruction>Use &lt;tool_call&gt; and &amp; keep output &quot;safe&quot;</instruction>"
        ));
    }

    #[test]
    fn safety_section_includes_security_summary_when_present() {
        let tools: Vec<Box<dyn Tool>> = vec![];
        let summary = "**Autonomy level**: Supervised\n\
                        **Allowed shell commands**: `git`, `ls`.\n"
            .to_string();
        let ctx = PromptContext {
            workspace_dir: Path::new("/tmp"),
            model_name: "test-model",
            tools: &tools,
            skills: &[],
            skills_prompt_mode: crate::config::SkillsPromptInjectionMode::Full,
            identity_config: None,
            dispatcher_instructions: "",
            tool_descriptions: None,
            security_summary: Some(summary.clone()),
            autonomy_level: AutonomyLevel::Supervised,
        };

        let output = SafetySection.build(&ctx).unwrap();
        assert!(
            output.contains("## Safety"),
            "should contain base safety header"
        );
        assert!(
            output.contains("### Active Security Policy"),
            "should contain security policy header"
        );
        assert!(
            output.contains("Autonomy level"),
            "should contain autonomy level from summary"
        );
        assert!(
            output.contains("`git`"),
            "should contain allowed commands from summary"
        );
    }

    #[test]
    fn safety_section_omits_security_policy_when_none() {
        let tools: Vec<Box<dyn Tool>> = vec![];
        let ctx = PromptContext {
            workspace_dir: Path::new("/tmp"),
            model_name: "test-model",
            tools: &tools,
            skills: &[],
            skills_prompt_mode: crate::config::SkillsPromptInjectionMode::Full,
            identity_config: None,
            dispatcher_instructions: "",
            tool_descriptions: None,
            security_summary: None,
            autonomy_level: AutonomyLevel::Supervised,
        };

        let output = SafetySection.build(&ctx).unwrap();
        assert!(
            output.contains("## Safety"),
            "should contain base safety header"
        );
        assert!(
            !output.contains("### Active Security Policy"),
            "should NOT contain security policy header when None"
        );
    }

    #[test]
    fn safety_section_full_autonomy_omits_approval_instructions() {
        let tools: Vec<Box<dyn Tool>> = vec![];
        let ctx = PromptContext {
            workspace_dir: Path::new("/tmp"),
            model_name: "test-model",
            tools: &tools,
            skills: &[],
            skills_prompt_mode: crate::config::SkillsPromptInjectionMode::Full,
            identity_config: None,
            dispatcher_instructions: "",
            tool_descriptions: None,
            security_summary: None,
            autonomy_level: AutonomyLevel::Full,
        };

        let output = SafetySection.build(&ctx).unwrap();
        assert!(
            !output.contains("without asking"),
            "full autonomy should NOT include 'ask before acting' instructions"
        );
        assert!(
            !output.contains("bypass oversight"),
            "full autonomy should NOT include 'bypass oversight' instructions"
        );
        assert!(
            output.contains("Execute tools and actions directly"),
            "full autonomy should instruct to execute directly"
        );
        assert!(
            output.contains("Do not exfiltrate"),
            "full autonomy should still include data exfiltration guard"
        );
    }

    #[test]
    fn safety_section_supervised_includes_approval_instructions() {
        let tools: Vec<Box<dyn Tool>> = vec![];
        let ctx = PromptContext {
            workspace_dir: Path::new("/tmp"),
            model_name: "test-model",
            tools: &tools,
            skills: &[],
            skills_prompt_mode: crate::config::SkillsPromptInjectionMode::Full,
            identity_config: None,
            dispatcher_instructions: "",
            tool_descriptions: None,
            security_summary: None,
            autonomy_level: AutonomyLevel::Supervised,
        };

        let output = SafetySection.build(&ctx).unwrap();
        assert!(
            output.contains("without asking"),
            "supervised should include 'ask before acting' instructions"
        );
        assert!(
            output.contains("bypass oversight"),
            "supervised should include 'bypass oversight' instructions"
        );
    }
}

# Termux-Only System Prompt Guide

## Overview

The ZeroClaw agent has been updated to emphasize Termux (Android) as its primary deployment environment. The system prompt now explicitly guides the LLM to:

1. Assume all configured tools and skills are available in Termux
2. Execute tools confidently without asking for permission  
3. Use shells, file I/O, and skills liberally to accomplish tasks
4. Focus on direct action rather than narration about tool usage

## Changes Made

### 1. **SafetySection** (src/agent/prompt.rs)
- Updated label from "Safety" to "Safety & Capability"
- Added language emphasizing "full access to all configured tools and skills in Termux"
- Changed guidance to use tools "confidently and liberally"

### 2. **RuntimeSection** (src/agent/prompt.rs)
- Added OS detection for Android/Termux environment
- When detecting Android, explicitly states its Termux/primary deployment status
- Lists key capabilities: Bash shell, Unix utilities, networking, file system access
- Ends with: "Assume all tools and skills are ready to use. Leverage them liberally to complete tasks."

### 3. **ToolsSection** (src/agent/prompt.rs)
- Header changed to "Available Tools & Capabilities"
- Opening line: "Use them liberally and confidently to complete what the user asks"

### 4. **TermuxCapabilitiesSection** (NEW - src/agent/prompt.rs)
- New dedicated prompt section added to SystemPromptBuilder
- Explicitly states Termux/Android is the primary deployment target
- Emphasizes tool availability and direct usage pattern
- Removes permission-asking behavior in favor of runtime enforcement

## Expected Agent Behavior

The agent now:
- Directly executes tools without narrating tool usage
- Chains multiple tools together for complex tasks
- Assumes full tool access is available in Termux
- Returns only the final answer to the user
- Does not ask for permission before using tools (enforced by runtime)

## Configuration in config.toml

```toml
[autonomy]
level = "full"    # Full autonomy - use all tools directly

[pacing]
max_tool_iterations = 10  # Allow multi-step tool chains
```

## Code Locations

Modified files:
- `src/agent/prompt.rs` - System prompt sections updated and new Termux section added
- `src/channels/mod.rs` - Channel-level system prompt now mentions Termux in anti-narration section

Key functions:
- `SystemPromptBuilder::with_defaults()` - Includes TermuxCapabilitiesSection
- `SafetySection::build()` - Updated with Termux emphasis
- `RuntimeSection::build()` - Added Android/Termux detection
- `ToolsSection::build()` - Updated header and guidance
- `build_system_prompt_with_mode_and_autonomy()` - Anti-narration now mentions Termux availability
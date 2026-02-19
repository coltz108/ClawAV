// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2025-2026 JR Morton

//! Prompt Firewall — intercepts malicious prompts before they reach LLM providers.
//!
//! Scans outbound LLM request bodies against pre-compiled regex patterns organized
//! into threat categories: prompt injection, exfiltration-via-prompt, jailbreak,
//! tool abuse, and system prompt extraction.
//!
//! Enforcement is controlled by a 3-tier system:
//! - Tier 1 (Permissive): Log all matches, block nothing
//! - Tier 2 (Standard): Block injection + exfil (real system threats), log the rest
//! - Tier 3 (Strict): Block all categories

use std::collections::HashMap;

/// Threat categories for prompt classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ThreatCategory {
    PromptInjection,
    ExfilViaPrompt,
    Jailbreak,
    ToolAbuse,
    SystemPromptExtract,
}

/// Action to take when a pattern matches.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FirewallAction {
    Block,
    Warn,
    Log,
}

/// A single pattern match with metadata.
#[derive(Debug, Clone)]
pub struct FirewallMatch {
    pub category: ThreatCategory,
    pub pattern_name: String,
    pub description: String,
    pub action: FirewallAction,
}

/// Result of scanning a prompt through the firewall.
#[derive(Debug)]
pub enum FirewallResult {
    /// No patterns matched.
    Pass,
    /// Matches found — highest action is Log (forward, record).
    Log { matches: Vec<FirewallMatch> },
    /// Matches found — highest action is Warn (forward, alert).
    Warn { matches: Vec<FirewallMatch> },
    /// Matches found — highest action is Block (reject request).
    Block { matches: Vec<FirewallMatch> },
}

/// Resolve the default action for a category at a given tier.
pub fn tier_default_action(tier: u8, category: ThreatCategory) -> FirewallAction {
    match tier {
        1 => FirewallAction::Log,
        2 => match category {
            ThreatCategory::PromptInjection | ThreatCategory::ExfilViaPrompt => {
                FirewallAction::Block
            }
            _ => FirewallAction::Log,
        },
        _ => FirewallAction::Block,
    }
}

fn category_config_key(category: ThreatCategory) -> &'static str {
    match category {
        ThreatCategory::PromptInjection => "prompt_injection",
        ThreatCategory::ExfilViaPrompt => "exfil_via_prompt",
        ThreatCategory::Jailbreak => "jailbreak",
        ThreatCategory::ToolAbuse => "tool_abuse",
        ThreatCategory::SystemPromptExtract => "system_prompt_extract",
    }
}

fn parse_action(s: &str) -> Option<FirewallAction> {
    match s {
        "block" => Some(FirewallAction::Block),
        "warn" => Some(FirewallAction::Warn),
        "log" => Some(FirewallAction::Log),
        _ => None,
    }
}

/// Resolve the effective action for a category given tier defaults + overrides.
pub fn resolve_action(
    tier: u8,
    category: ThreatCategory,
    overrides: &HashMap<String, String>,
) -> FirewallAction {
    let key = category_config_key(category);
    if let Some(action_str) = overrides.get(key) {
        if let Some(action) = parse_action(action_str) {
            return action;
        }
    }
    tier_default_action(tier, category)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tier1_all_log() {
        for cat in [
            ThreatCategory::PromptInjection,
            ThreatCategory::ExfilViaPrompt,
            ThreatCategory::Jailbreak,
            ThreatCategory::ToolAbuse,
            ThreatCategory::SystemPromptExtract,
        ] {
            assert_eq!(
                tier_default_action(1, cat),
                FirewallAction::Log,
                "Tier 1 should log everything, got non-Log for {:?}",
                cat
            );
        }
    }

    #[test]
    fn test_tier2_blocks_injection_and_exfil() {
        assert_eq!(
            tier_default_action(2, ThreatCategory::PromptInjection),
            FirewallAction::Block
        );
        assert_eq!(
            tier_default_action(2, ThreatCategory::ExfilViaPrompt),
            FirewallAction::Block
        );
    }

    #[test]
    fn test_tier2_logs_non_threats() {
        assert_eq!(
            tier_default_action(2, ThreatCategory::Jailbreak),
            FirewallAction::Log
        );
        assert_eq!(
            tier_default_action(2, ThreatCategory::ToolAbuse),
            FirewallAction::Log
        );
        assert_eq!(
            tier_default_action(2, ThreatCategory::SystemPromptExtract),
            FirewallAction::Log
        );
    }

    #[test]
    fn test_tier3_blocks_everything() {
        for cat in [
            ThreatCategory::PromptInjection,
            ThreatCategory::ExfilViaPrompt,
            ThreatCategory::Jailbreak,
            ThreatCategory::ToolAbuse,
            ThreatCategory::SystemPromptExtract,
        ] {
            assert_eq!(
                tier_default_action(3, cat),
                FirewallAction::Block,
                "Tier 3 should block everything, got non-Block for {:?}",
                cat
            );
        }
    }

    #[test]
    fn test_override_downgrades_block_to_log() {
        let mut overrides = HashMap::new();
        overrides.insert("prompt_injection".to_string(), "log".to_string());
        assert_eq!(
            resolve_action(2, ThreatCategory::PromptInjection, &overrides),
            FirewallAction::Log,
        );
    }

    #[test]
    fn test_override_upgrades_log_to_block() {
        let mut overrides = HashMap::new();
        overrides.insert("jailbreak".to_string(), "block".to_string());
        assert_eq!(
            resolve_action(2, ThreatCategory::Jailbreak, &overrides),
            FirewallAction::Block,
        );
    }

    #[test]
    fn test_no_override_uses_tier_default() {
        let overrides = HashMap::new();
        assert_eq!(
            resolve_action(2, ThreatCategory::PromptInjection, &overrides),
            FirewallAction::Block,
        );
    }
}

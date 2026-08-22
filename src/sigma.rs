use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::record::EventRecord;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogSource {
    pub category: Option<String>,
    pub product: Option<String>,
    pub service: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SigmaRule {
    pub title: String,
    pub id: Option<String>,
    pub status: Option<String>,
    pub description: Option<String>,
    pub logsource: LogSource,
    pub detection: HashMap<String, serde_yaml::Value>,
    pub level: Option<String>,
}

pub struct SigmaEngine {
    rules: Vec<SigmaRule>,
}

impl SigmaEngine {
    pub fn new() -> Self {
        Self { rules: Vec::new() }
    }

    pub fn load_rule_from_str(yaml_str: &str) -> Result<SigmaRule> {
        serde_yaml::from_str(yaml_str).context("Failed to parse Sigma YAML rule")
    }

    pub fn load_rule_file<P: AsRef<Path>>(path: P) -> Result<SigmaRule> {
        let content = fs::read_to_string(path)?;
        Self::load_rule_from_str(&content)
    }

    pub fn add_rule(&mut self, rule: SigmaRule) {
        self.rules.push(rule);
    }

    pub fn matches(&self, record: &EventRecord) -> Vec<&SigmaRule> {
        self.rules
            .iter()
            .filter(|rule| evaluate_rule(rule, record))
            .collect()
    }

    /// Converts a basic field selection set into an equivalent Windows Event Log XPath query string
    pub fn rule_to_xpath(rule: &SigmaRule) -> Option<String> {
        let mut conditions = Vec::new();

        if let Some(selection) = rule.detection.get("selection") {
            if let Some(map) = selection.as_mapping() {
                for (k, v) in map {
                    if let Some(key_str) = k.as_str() {
                        if key_str == "EventID" {
                            if let Some(id) = v.as_u64() {
                                conditions.push(format!("System/EventID={}", id));
                            }
                        } else if let Some(val_str) = v.as_str() {
                            conditions
                                .push(format!("EventData/Data[@Name='{}']='{}'", key_str, val_str));
                        }
                    }
                }
            }
        }

        if conditions.is_empty() {
            None
        } else {
            Some(format!("*[System[{}]]", conditions.join(" and ")))
        }
    }
}

fn evaluate_condition_expr(expr: &str, selections: &HashMap<String, bool>) -> bool {
    let expr_clean = expr.trim();

    if expr_clean.starts_with("all of ") {
        let prefix = expr_clean.trim_start_matches("all of ");
        let pattern = prefix.trim_end_matches('*');
        return selections
            .iter()
            .filter(|(k, _)| k.starts_with(pattern))
            .all(|(_, v)| *v);
    }

    if expr_clean.starts_with("1 of ") || expr_clean.starts_with("any of ") {
        let prefix = if expr_clean.starts_with("1 of ") {
            expr_clean.trim_start_matches("1 of ")
        } else {
            expr_clean.trim_start_matches("any of ")
        };
        let pattern = prefix.trim_end_matches('*');
        return selections
            .iter()
            .filter(|(k, _)| k.starts_with(pattern))
            .any(|(_, v)| *v);
    }

    if expr_clean.contains(" and ") {
        let parts: Vec<&str> = expr_clean.split(" and ").collect();
        return parts
            .iter()
            .all(|p| selections.get(p.trim()).copied().unwrap_or(false));
    }

    if expr_clean.contains(" or ") {
        let parts: Vec<&str> = expr_clean.split(" or ").collect();
        return parts
            .iter()
            .any(|p| selections.get(p.trim()).copied().unwrap_or(false));
    }

    selections.get(expr_clean).copied().unwrap_or(false)
}

fn check_value_match(actual: &str, expected: &str, modifier: Option<&str>) -> bool {
    let actual_lower = actual.to_lowercase();
    let expected_lower = expected.to_lowercase();

    match modifier {
        Some("contains") => actual_lower.contains(&expected_lower),
        Some("startswith") => actual_lower.starts_with(&expected_lower),
        Some("endswith") => actual_lower.ends_with(&expected_lower),
        _ => actual_lower == expected_lower,
    }
}

fn get_record_field_value(record: &EventRecord, field_name: &str) -> Option<String> {
    match field_name {
        "EventID" => Some(record.event_id.to_string()),
        "Provider" | "ProviderName" => Some(record.provider.clone()),
        "Channel" => Some(record.channel.clone()),
        "Computer" => Some(record.computer.clone()),
        _ => record
            .payload
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case(field_name))
            .map(|(_, v)| v.clone()),
    }
}

fn match_field(field_spec: &str, target_val: &serde_yaml::Value, record: &EventRecord) -> bool {
    let parts: Vec<&str> = field_spec.split('|').collect();
    let field_name = parts[0];
    let modifier = parts.get(1).copied();

    let record_field_val = get_record_field_value(record, field_name);

    match record_field_val {
        Some(val) => match target_val {
            serde_yaml::Value::String(s) => check_value_match(&val, s, modifier),
            serde_yaml::Value::Number(n) => check_value_match(&val, &n.to_string(), modifier),
            serde_yaml::Value::Sequence(seq) => seq.iter().any(|item| {
                if let Some(s) = item.as_str() {
                    check_value_match(&val, s, modifier)
                } else {
                    false
                }
            }),
            _ => false,
        },
        None => false,
    }
}

fn match_selection(val: &serde_yaml::Value, record: &EventRecord) -> bool {
    match val {
        serde_yaml::Value::Mapping(map) => {
            for (k, v) in map {
                let field_key = match k.as_str() {
                    Some(s) => s,
                    None => return false,
                };

                if !match_field(field_key, v, record) {
                    return false;
                }
            }
            true
        }
        serde_yaml::Value::Sequence(seq) => seq.iter().any(|item| match_selection(item, record)),
        _ => false,
    }
}

fn evaluate_rule(rule: &SigmaRule, record: &EventRecord) -> bool {
    let mut selection_results = HashMap::new();

    for (key, val) in &rule.detection {
        if key == "condition" {
            continue;
        }
        let matched = match_selection(val, record);
        selection_results.insert(key.clone(), matched);
    }

    if let Some(condition) = rule.detection.get("condition").and_then(|v| v.as_str()) {
        evaluate_condition_expr(condition, &selection_results)
    } else {
        selection_results.get("selection").copied().unwrap_or(false)
    }
}

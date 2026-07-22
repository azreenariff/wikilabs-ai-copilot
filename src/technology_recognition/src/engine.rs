use wikilabs_data_types::{DetectionRule, DetectionType, EngineeringContext, TechnologyInference};
use wikilabs_observation::{ObservationEvent, ProviderType};

/// DetectionEngine — core technology recognition engine.
///
/// Matches observation events against DetectionRules loaded from skill packages.
#[derive(Debug, Clone)]
pub struct DetectionEngine {
    detection_rules: Vec<DetectionRule>,
    technology_cache: std::collections::HashMap<String, TechnologyInference>,
}

impl DetectionEngine {
    /// Create a new empty detection engine.
    pub fn new() -> Self {
        Self {
            detection_rules: Vec::new(),
            technology_cache: std::collections::HashMap::new(),
        }
    }

    /// Load detection rules from a skill directory.
    ///
    /// Expects `skill_dir/detection_rules/` containing YAML files with
    /// arrays of DetectionRule objects.
    pub fn from_skill_rules(skill_dir: &str) -> anyhow::Result<Self> {
        let mut engine = Self::new();
        let rules_path = format!("{}/detection_rules", skill_dir);
        tracing::info!(rules_path, "Loading detection rules from skill directory");

        if let Ok(entries) = std::fs::read_dir(&rules_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path
                    .extension()
                    .is_some_and(|ext| ext == "yaml" || ext == "yml")
                {
                    tracing::debug!(file = ?path, "Loading detection rule file");
                    if let Ok(content) = std::fs::read_to_string(&path) {
                        if let Ok(rules) = serde_yaml::from_str::<Vec<DetectionRule>>(&content) {
                            for rule in rules {
                                engine.add_rule(rule);
                            }
                        }
                    }
                }
            }
        }
        Ok(engine)
    }

    /// Add a single detection rule.
    pub fn add_rule(&mut self, rule: DetectionRule) {
        tracing::debug!(
            technology_domain = rule.technology_domain,
            detection_type = ?rule.detection_type,
            "Adding detection rule"
        );
        self.detection_rules.push(rule);
    }

    /// Recognize technologies from an observation event.
    pub fn recognize(&self, event: &ObservationEvent) -> Vec<TechnologyInference> {
        tracing::trace!(
            event_id = %event.event_id,
            event_type = ?event.event_type,
            "Recognizing technologies"
        );

        let mut inferences = Vec::new();
        let mut seen = std::collections::HashSet::new();

        for rule in &self.detection_rules {
            if self.rule_matches_event(rule, event) {
                let key = format!("{}:{}", rule.technology_domain, rule.detection_type);
                if seen.insert(key) {
                    inferences.push(TechnologyInference::new(
                        rule.technology_domain.clone(),
                        rule.confidence,
                        self.rule_type_label(&rule.detection_type),
                        format!("matched:{}:{}", rule.name, rule.technology_domain),
                    ));
                }
            }
        }

        inferences.sort_by(|a, b| {
            b.confidence
                .partial_cmp(&a.confidence)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        inferences
    }

    /// Recognize technologies from an engineering context.
    pub fn recognize_from_context(&self, context: &EngineeringContext) -> Vec<TechnologyInference> {
        tracing::trace!("Recognizing from engineering context");

        let mut inferences = Vec::new();
        let mut seen = std::collections::HashSet::new();

        for rule in &self.detection_rules {
            if self.rule_matches_context(rule, context) {
                let key = format!("ctx:{}:{}", rule.technology_domain, rule.detection_type);
                if seen.insert(key) {
                    inferences.push(TechnologyInference::new(
                        rule.technology_domain.clone(),
                        rule.confidence,
                        "context".to_string(),
                        format!("context_match: {}", rule.name),
                    ));
                }
            }
        }

        inferences.sort_by(|a, b| {
            b.confidence
                .partial_cmp(&a.confidence)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        inferences
    }

    /// Get the top technology from the cache, if any.
    pub fn get_top_technology(&self) -> Option<TechnologyInference> {
        self.technology_cache
            .values()
            .max_by(|a, b| {
                a.confidence
                    .partial_cmp(&b.confidence)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .cloned()
    }

    /// Update the cache with new inferences.
    pub fn update_cache(&mut self, inferences: Vec<TechnologyInference>) {
        for inference in inferences {
            tracing::info!(
                technology = inference.name,
                confidence = inference.confidence,
                "Updated technology cache"
            );
            self.technology_cache
                .insert(inference.name.clone(), inference);
        }
    }

    /// Clear the technology cache.
    pub fn clear_cache(&mut self) {
        let count = self.technology_cache.len();
        self.technology_cache.clear();
        tracing::debug!(cleared = count, "Cleared technology cache");
    }

    /// Return the number of detection rules.
    pub fn rule_count(&self) -> usize {
        self.detection_rules.len()
    }

    // ── internal helpers ──────────────────────────────────────────

    fn rule_matches_event(&self, rule: &DetectionRule, event: &ObservationEvent) -> bool {
        match &rule.detection_type {
            DetectionType::File => self.match_file(rule, event),
            DetectionType::Command => self.match_command(rule, event),
            DetectionType::Pattern => self.match_pattern(rule, event),
            DetectionType::Argument => self.match_argument(rule, event),
            DetectionType::Environment => self.match_environment(rule, event),
        }
    }

    fn rule_matches_context(&self, rule: &DetectionRule, context: &EngineeringContext) -> bool {
        match &rule.detection_type {
            DetectionType::File => context
                .confidence_scores
                .contains_key(&rule.technology_domain),
            DetectionType::Command => context
                .technologies
                .iter()
                .any(|t| t.name == rule.technology_domain),
            DetectionType::Pattern => context
                .technologies
                .iter()
                .any(|t| t.name == rule.technology_domain),
            DetectionType::Argument => context
                .confidence_scores
                .contains_key(&rule.technology_domain),
            DetectionType::Environment => context
                .technologies
                .iter()
                .any(|t| t.name == rule.technology_domain),
        }
    }

    fn match_file(&self, rule: &DetectionRule, event: &ObservationEvent) -> bool {
        if event.provider != ProviderType::FileObserver {
            return false;
        }
        if let Some(path) = event.payload.data.get("path").and_then(|v| v.as_str()) {
            return self.pattern_matches(&rule.pattern, path);
        }
        false
    }

    fn match_command(&self, rule: &DetectionRule, event: &ObservationEvent) -> bool {
        if event.provider != ProviderType::Terminal {
            return false;
        }
        if let Some(cmd) = event.payload.data.get("command").and_then(|v| v.as_str()) {
            return self.pattern_matches(&rule.pattern, cmd);
        }
        self.pattern_matches(&rule.pattern, &event.source)
    }

    fn match_pattern(&self, rule: &DetectionRule, event: &ObservationEvent) -> bool {
        let text = self.extract_text(&event.payload.data);
        let meta = self.extract_text(&event.metadata);
        let combined = format!("{} {}", text, meta);
        self.pattern_matches(&rule.pattern, &combined)
    }

    fn match_argument(&self, rule: &DetectionRule, event: &ObservationEvent) -> bool {
        if event.provider != ProviderType::Terminal {
            return false;
        }
        if let Some(cmd) = event.payload.data.get("command").and_then(|v| v.as_str()) {
            return self.pattern_matches(&rule.pattern, cmd);
        }
        false
    }

    fn match_environment(&self, rule: &DetectionRule, event: &ObservationEvent) -> bool {
        if event.provider != ProviderType::ActiveWindow {
            return false;
        }
        self.pattern_matches(&rule.pattern, &event.source)
            || self.pattern_matches_payload(&rule.pattern, &event.payload.data)
    }

    fn pattern_matches(&self, pattern: &str, text: &str) -> bool {
        if let Ok(re) = regex::Regex::new(pattern) {
            re.is_match(text)
        } else {
            text.contains(pattern)
        }
    }

    fn pattern_matches_payload(&self, pattern: &str, value: &serde_json::Value) -> bool {
        if let Some(s) = value.get("url").and_then(|v| v.as_str()) {
            if self.pattern_matches(pattern, s) {
                return true;
            }
        }
        if let Some(s) = value.get("title").and_then(|v| v.as_str()) {
            if self.pattern_matches(pattern, s) {
                return true;
            }
        }
        if let Some(s) = value.get("command").and_then(|v| v.as_str()) {
            if self.pattern_matches(pattern, s) {
                return true;
            }
        }
        self.pattern_matches(pattern, &value.to_string())
    }

    fn extract_text(&self, value: &serde_json::Value) -> String {
        match value {
            serde_json::Value::String(s) => s.clone(),
            serde_json::Value::Object(map) => map
                .values()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect::<Vec<_>>()
                .join(" "),
            _ => String::new(),
        }
    }

    fn rule_type_label(&self, detection_type: &DetectionType) -> String {
        match detection_type {
            DetectionType::File => "file".to_string(),
            DetectionType::Command => "command".to_string(),
            DetectionType::Pattern => "pattern".to_string(),
            DetectionType::Argument => "argument".to_string(),
            DetectionType::Environment => "environment".to_string(),
        }
    }
}

impl Default for DetectionEngine {
    fn default() -> Self {
        Self::new()
    }
}

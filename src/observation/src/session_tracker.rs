//! Session Tracker — Troubleshooting Narrative State Machine
//!
//! Tracks the user's troubleshooting journey across observation ticks.
//! It maintains context so the copilot can reason about multi-step workflows:
//!
//! Example narrative:
//! 1. User opens browser → nagios XI page shows "500 Internal Server Error"
//! 2. User opens terminal → types "systemctl status nagios"
//! 3. → Session tracker detects: "User is troubleshooting nagios after seeing error"
//! 4. User types "systemctl status mysqld" → sees "inactive"
//! 5. → Session tracker detects: "Root cause likely: database down, causing nagios to fail"

use std::sync::{Arc, Mutex};

use chrono::Utc;

use crate::semantic_analyzer::{AnalysisResult, CommandIntent, IntentCategory, SemanticAnalyzer};

/// A step in the user's troubleshooting journey.
#[derive(Debug, Clone)]
pub struct NarrativeStep {
    /// Step number in the sequence.
    pub step_number: u32,
    /// Timestamp when this step occurred.
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// What the user was doing.
    pub action: String,
    /// What the user was looking at (browser URL, etc.).
    pub context: String,
    /// What we observed (errors, warnings, etc.).
    pub observations: Vec<String>,
    /// Our hypothesis about what's going on.
    pub hypothesis: Option<String>,
    /// The inferred intent from the command.
    pub intent: Option<CommandIntent>,
}

/// State of the troubleshooting session.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SessionState {
    /// No active troubleshooting.
    Idle,
    /// User is investigating something.
    Investigating,
    /// User is actively troubleshooting a problem.
    Troubleshooting,
    /// Problem identified.
    ProblemIdentified,
    /// Fix is being applied.
    ApplyingFix,
    /// Verification is happening.
    Verifying,
    /// Resolution complete.
    Resolved,
}

/// The session tracker maintains the user's troubleshooting narrative.
#[derive(Debug, Clone)]
pub struct TroubleshootingSession {
    /// Current state.
    pub state: SessionState,
    /// Steps in the current narrative.
    pub steps: Vec<NarrativeStep>,
    /// What the user was doing before troubleshooting started.
    pub initial_context: Option<String>,
    /// Current hypothesis about the problem.
    pub current_hypothesis: Option<String>,
    /// What problem we're troubleshooting.
    pub problem_description: Option<String>,
    /// What target system is involved.
    pub target_system: Option<String>,
    /// Has the user found the root cause?
    pub root_cause_found: bool,
    /// Suggested next step.
    pub suggested_next_step: Option<String>,
}

/// Session tracker state.
#[derive(Debug, Clone)]
pub struct TrackerState {
    /// Current troubleshooting session.
    pub current_session: Option<TroubleshootingSession>,
    /// Analyzer for understanding commands.
    pub semantic_analyzer: SemanticAnalyzer,
}

/// Session tracker — the "brain" that tracks troubleshooting narratives.
pub struct SessionTracker {
    state: Arc<Mutex<TrackerState>>,
}

impl SessionTracker {
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(TrackerState {
                current_session: None,
                semantic_analyzer: SemanticAnalyzer::new(),
            })),
        }
    }

    /// Process a new observation tick and update the narrative.
    ///
    /// Arguments:
    /// - browser_url: Current browser URL (if any)
    /// - browser_error: Any error observed in the browser
    /// - command: Current terminal command being typed
    /// - command_output: Output from the terminal command
    ///
    /// Returns the updated session state and any suggestions.
    pub fn process_tick(
        &self,
        browser_url: Option<&str>,
        browser_error: Option<&str>,
        command: Option<&str>,
        command_output: Option<&str>,
    ) -> (TroubleshootingSession, Vec<Suggestion>) {
        // Phase 1: Gather analysis results without holding borrows
        // .and_then() flattens Option<Option<T>> -> Option<T>
        let cmd_intent: Option<CommandIntent> = command.and_then(|c| {
            let state = self.state.lock().unwrap();
            state.semantic_analyzer.analyze_command(c)
        });

        let output_results: Vec<AnalysisResult> = command_output.map(|o| {
            let state = self.state.lock().unwrap();
            state.semantic_analyzer.analyze_output(o)
        }).unwrap_or_default();

        // Phase 2: Build observations from results
        let observations: Vec<String> = output_results
            .into_iter()
            .map(|r| match r {
                AnalysisResult::Error(msg) => format!("Error: {}", msg),
                AnalysisResult::Warning(msg) => format!("Warning: {}", msg),
                AnalysisResult::Success(msg) => format!("OK: {}", msg),
            })
            .collect();

        // Phase 3: Process everything under a single lock
        let mut state = self.state.lock().unwrap();
        let mut suggestions = Vec::new();

        // Initialize session if needed
        if state.current_session.is_none() {
            let mut session = TroubleshootingSession {
                state: SessionState::Idle,
                steps: Vec::new(),
                initial_context: None,
                current_hypothesis: None,
                problem_description: None,
                target_system: None,
                root_cause_found: false,
                suggested_next_step: None,
            };

            // Set initial context from browser
            if let Some(url) = browser_url {
                session.initial_context = Some(format!("User was browsing {}", url));
            }

            state.current_session = Some(session);
        }

        let session = state.current_session.as_mut().unwrap();
        let mut step_num = session.steps.len() as u32 + 1;

        // ── Detect errors in browser ───────────────────────────
        if let Some(error) = browser_error {
            let error_msg = self.extract_error_message(error);
            let has_error = error.to_lowercase().contains("error")
                || error.to_lowercase().contains("fail")
                || error.to_lowercase().contains("500")
                || error.to_lowercase().contains("502")
                || error.to_lowercase().contains("503");

            if has_error {
                // Check if we already have a troubleshooting session
                if session.state == SessionState::Idle
                    || session.state == SessionState::Investigating
                {
                    if let Some(url) = browser_url {
                        session.state = SessionState::Troubleshooting;
                        session.initial_context = Some(format!("User saw an error on {}", url));
                        session.problem_description = Some(error_msg.clone());
                        session.target_system = Some(self.extract_target_from_url(url));

                        session.steps.push(NarrativeStep {
                            step_number: step_num,
                            timestamp: Utc::now(),
                            action: format!("saw error on {}", url),
                            context: url.to_string(),
                            observations: vec![error_msg.clone()],
                            hypothesis: Some(format!(
                                "Possible issue with {}",
                                self.extract_target_from_url(url)
                            )),
                            intent: None,
                        });
                    }
                }
            }
        }

        // ── Analyze command ────────────────────────────────────
        if let Some(ref intent) = cmd_intent {
            // Add step to narrative
            let action = intent.action.clone();

            session.steps.push(NarrativeStep {
                step_number: step_num,
                timestamp: Utc::now(),
                action: action.clone(),
                context: browser_url.unwrap_or("").to_string(),
                observations: observations.clone(),
                hypothesis: session.current_hypothesis.clone(),
                intent: Some(intent.clone()),
            });

            // ── Update session state based on step ───────────
            match intent.category {
                IntentCategory::ServiceHealthCheck => {
                    // User is checking a service → likely troubleshooting
                    if session.state != SessionState::Troubleshooting {
                        session.state = SessionState::Troubleshooting;
                    }
                    session.target_system = intent.target.clone();

                    // Generate suggestions based on observations
                    if observations.iter().any(|o| o.contains("Error"))
                        || observations.iter().any(|o| o.contains("Warning"))
                    {
                        suggestions.push(self.make_suggestion(
                            intent,
                            &observations,
                            Some("The service you're checking may have issues. You might want to check related services like the database."),
                            &mut step_num,
                        ));
                    }

                    // Detect specific patterns → suggest next steps
                    if let Some(ref target) = intent.target {
                        let target_lower = target.to_lowercase();
                        if target_lower.contains("nagios")
                            && !session.root_cause_found {
                            suggestions.push(self.make_suggestion(
                                intent,
                                &observations,
                                Some("Since you're checking Nagios, you should also verify that its database (MySQL) is running — Nagios can't function without it. Try: `systemctl status mysqld`"),
                                &mut step_num,
                            ));
                        }
                        if observations.iter().any(|o| {
                            o.contains("not running")
                                || o.contains("inactive")
                                || o.contains("Error")
                        }) {
                            session.current_hypothesis = Some(format!("{} may be down, which could be causing issues with other services", target));
                            session.state = SessionState::ProblemIdentified;
                            suggestions.push(self.make_suggestion(
                                intent,
                                &observations,
                                Some("It looks like the database is down. You can try starting it with `systemctl start mysqld` and then check if your other services recover."),
                                &mut step_num,
                            ));
                        }
                        if observations
                            .iter()
                            .any(|o| o.contains("Error") || o.contains("Warning"))
                        {
                            session.current_hypothesis =
                                Some("Some Kubernetes workloads may be unhealthy".to_string());
                            suggestions.push(self.make_suggestion(
                                intent,
                                &observations,
                                Some("There are pod issues. You might want to check which pods are failing: `kubectl get pods --all-namespaces | grep -i error`"),
                                &mut step_num,
                            ));
                        }
                    }
                }
                IntentCategory::ServiceStartStop => {
                    session.state = SessionState::ApplyingFix;
                    session.suggested_next_step = Some("After starting/restarting the service, verify it's running and check if the related issues are resolved.".to_string());
                    suggestions.push(self.make_suggestion(
                        intent,
                        &observations,
                        Some("Service is being restarted. You should verify it starts correctly and check if your issues are resolved."),
                        &mut step_num,
                    ));
                }
                IntentCategory::LogInspection => {
                    if session.state != SessionState::Troubleshooting {
                        session.state = SessionState::Troubleshooting;
                    }
                    suggestions.push(self.make_suggestion(
                        intent,
                        &observations,
                        Some("Looking at logs is good. Try to find the root cause by looking for the earliest error in the log chain."),
                        &mut step_num,
                    ));
                }
                IntentCategory::NetworkDiagnostic => {
                    if session.state != SessionState::Troubleshooting {
                        session.state = SessionState::Troubleshooting;
                    }
                    suggestions.push(self.make_suggestion(
                        intent,
                        &observations,
                        Some("Good diagnostic approach. If the target is unreachable, check if the service is running and if firewall rules are correct."),
                        &mut step_num,
                    ));
                }
                _ => {}
            }
        }

        // ── Check if troubleshooting is complete ───────────────
        if let Some(ref sess) = state.current_session {
            if sess.state == SessionState::Troubleshooting {
                // Check if we have enough data to form a hypothesis
                let error_count = sess
                    .steps
                    .iter()
                    .flat_map(|s| &s.observations)
                    .filter(|o| o.contains("Error") || o.contains("Warning"))
                    .count();

                if error_count >= 2 && sess.current_hypothesis.is_none() {
                    // Need to release the &ref and get mutable access
                    drop(state);
                    let mut state = self.state.lock().unwrap();
                    if let Some(ref mut session) = state.current_session {
                        session.state = SessionState::ProblemIdentified;
                        session.current_hypothesis = Some(
                            "Multiple issues detected — likely a cascading failure".to_string(),
                        );
                        step_num = session.steps.len() as u32;
                        let fallback_intent = CommandIntent {
                            command: "general".to_string(),
                            action: "general observation".to_string(),
                            target: None,
                            category: IntentCategory::General,
                            confidence: 0.5,
                            explanation: "General troubleshooting".to_string(),
                        };
                        let last_intent = session
                            .steps
                            .last()
                            .and_then(|s| s.intent.as_ref())
                            .cloned()
                            .unwrap_or(fallback_intent);
                        let all_observations: Vec<String> = session
                            .steps
                            .iter()
                            .flat_map(|s| s.observations.clone())
                            .collect();
                        suggestions.push(self.make_suggestion(
                            &last_intent,
                            &all_observations,
                            Some("I've noticed multiple issues across your checks. This could be a cascading failure — one service failing causing others to fail. Try checking the most fundamental service first (often the database)."),
                            &mut step_num,
                        ));
                    }
                    // Re-acquire for the final result — `state` was dropped above
                    let result = state.current_session.clone().unwrap();
                    drop(state);
                    return (result, suggestions);
                }
            }
        }

        // Update the session in state
        let result = state.current_session.clone().unwrap();
        drop(state);

        (result, suggestions)
    }

    /// Get current session state.
    pub fn get_session(&self) -> Option<TroubleshootingSession> {
        self.state
            .lock()
            .unwrap()
            .current_session
            .clone()
    }

    /// Get current hypothesis.
    pub fn get_hypothesis(&self) -> Option<String> {
        self.state
            .lock()
            .unwrap()
            .current_session
            .as_ref()
            .and_then(|s| s.current_hypothesis.clone())
    }

    /// Get suggested next step.
    pub fn get_suggested_next_step(&self) -> Option<String> {
        self.state
            .lock()
            .unwrap()
            .current_session
            .as_ref()
            .and_then(|s| s.suggested_next_step.clone())
    }

    /// Reset the session.
    pub fn reset(&self) {
        let mut state = self.state.lock().unwrap();
        state.current_session = None;
    }

    /// Extract error message from browser content.
    fn extract_error_message(&self, text: &str) -> String {
        if text.len() > 200 {
            format!("{}...", &text[..200])
        } else {
            text.to_string()
        }
    }

    /// Extract target system name from URL.
    fn extract_target_from_url(&self, url: &str) -> String {
        let url_lower = url.to_lowercase();
        if url_lower.contains("nagios") {
            return "nagios".to_string();
        }
        if url_lower.contains("grafana") {
            return "grafana".to_string();
        }
        if url_lower.contains("kubernetes") || url_lower.contains("openshift") {
            return "kubernetes".to_string();
        }
        if url_lower.contains("jenkins") {
            return "jenkins".to_string();
        }
        if url_lower.contains("prometheus") {
            return "prometheus".to_string();
        }
        if url_lower.contains("gitlab") {
            return "gitlab".to_string();
        }
        if url_lower.contains("vcenter") || url_lower.contains("vmware") {
            return "vmware".to_string();
        }
        if url_lower.contains("elastic") || url_lower.contains("kibana") {
            return "elasticsearch".to_string();
        }
        url.to_string()
    }

    /// Make a contextual suggestion based on intent and observations.
    fn make_suggestion(
        &self,
        intent: &CommandIntent,
        _observations: &[String],
        guidance: Option<&str>,
        _step_num: &mut u32,
    ) -> Suggestion {
        Suggestion {
            message: guidance
                .expect("I'm observing your work and can help. Let me know if you need anything.")
                .to_string(),
            confidence: intent.confidence,
            category: intent.category.clone(),
            related_target: intent.target.clone(),
        }
    }
}

/// A suggestion generated by the session tracker.
#[derive(Debug, Clone)]
pub struct Suggestion {
    /// The suggestion text.
    pub message: String,
    /// Confidence in this suggestion.
    pub confidence: f32,
    /// Category of suggestion.
    pub category: IntentCategory,
    /// Related target system (if any).
    pub related_target: Option<String>,
}

impl Default for SessionTracker {
    fn default() -> Self {
        Self::new()
    }
}
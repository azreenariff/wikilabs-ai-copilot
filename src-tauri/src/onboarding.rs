//! Onboarding Flow — First-Time User Experience
//!
//! # Overview
//!
//! This module implements a guided onboarding experience for first-time users
//! of the Wiki Labs AI Copilot application. It provides:
//!
//! - A **welcome wizard** that walks users through initial setup.
//! - **Guided tour** of the main UI panels.
//! - **Settings walkthrough** to configure AI provider and preferences.
//! - **Progressive disclosure** — only show relevant steps.
//! - **Persistence** — track which steps have been completed.
//! - **Skippable** — users can skip any step and return later.
//!
//! # Steps in the Onboarding Flow
//!
//! 1. **Welcome** — Brief introduction to the app and its purpose.
//! 2. **AI Provider Setup** — Configure the AI model endpoint and API key.
//! 3. **Privacy Preferences** — Review and set data collection preferences.
//! 4. **Appearance** — Choose theme, font size, and language.
//! 5. **Keyboard Shortcuts** — Review available keyboard shortcuts.
//! 6. **Tour** — Interactive tour of the main UI panels.
//! 7. **Complete** — Confirmation that setup is complete.
//!
//! # Integration
//!
//! The onboarding flow integrates with:
//! - [`config::AppSettingsStore`] — stores completion status.
//! - [`theme::ThemeManager`] — applies theme settings.
//! - [`global_shortcuts::GlobalShortcutManager`] — registers shortcuts.
//!
//! # References
//!
//! - [Onboarding Best Practices](https://www.nngroup.com/articles/onboarding/)
//! - [Microsoft Fluent UI Onboarding](https://developer.microsoft.com/en-us/fluentui#/controls/web/onboardingtour)

use serde::{Deserialize, Serialize};
use tracing::{info, warn};

/// Available onboarding steps.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum OnboardingStep {
    /// Welcome screen with app introduction.
    Welcome,
    /// AI provider configuration.
    AiProviderSetup,
    /// Privacy and data collection preferences.
    PrivacyPreferences,
    /// Appearance and display settings.
    Appearance,
    /// Keyboard shortcuts overview.
    KeyboardShortcuts,
    /// Interactive application tour.
    Tour,
    /// Completion confirmation.
    Complete,
}

impl OnboardingStep {
    /// Display label for the step.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Welcome => "Welcome",
            Self::AiProviderSetup => "AI Provider",
            Self::PrivacyPreferences => "Privacy",
            Self::Appearance => "Appearance",
            Self::KeyboardShortcuts => "Shortcuts",
            Self::Tour => "Tour",
            Self::Complete => "Complete",
        }
    }

    /// Short description of the step.
    pub fn description(&self) -> &'static str {
        match self {
            Self::Welcome => "Welcome to Wiki Labs AI Copilot",
            Self::AiProviderSetup => "Connect your AI model",
            Self::PrivacyPreferences => "Configure data preferences",
            Self::Appearance => "Choose your look and feel",
            Self::KeyboardShortcuts => "Learn keyboard shortcuts",
            Self::Tour => "Explore the interface",
            Self::Complete => "You're all set!",
        }
    }

    /// Whether this step is optional and can be skipped.
    pub fn is_optional(&self) -> bool {
        // Only AI Provider and Tour are optional
        matches!(self, Self::AiProviderSetup | Self::Tour)
    }

    /// Get the next step in the sequence, or None if this is the last.
    pub fn next(&self) -> Option<Self> {
        use OnboardingStep::*;
        match self {
            Welcome => Some(AiProviderSetup),
            AiProviderSetup => Some(PrivacyPreferences),
            PrivacyPreferences => Some(Appearance),
            Appearance => Some(KeyboardShortcuts),
            KeyboardShortcuts => Some(Tour),
            Tour => Some(Complete),
            Complete => None,
        }
    }

    /// Get the previous step in the sequence, or None if this is the first.
    pub fn previous(&self) -> Option<Self> {
        use OnboardingStep::*;
        match self {
            Welcome => None,
            AiProviderSetup => Some(Welcome),
            PrivacyPreferences => Some(AiProviderSetup),
            Appearance => Some(PrivacyPreferences),
            KeyboardShortcuts => Some(Appearance),
            Tour => Some(KeyboardShortcuts),
            Complete => Some(Tour),
        }
    }
}

/// Current state of the onboarding flow.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OnboardingState {
    /// Whether onboarding has been initiated.
    pub initiated: bool,
    /// Whether onboarding is currently in progress.
    pub in_progress: bool,
    /// The current step index (0-based).
    pub current_step: usize,
    /// Steps that have been completed.
    pub completed_steps: Vec<OnboardingStep>,
    /// Steps that have been skipped.
    pub skipped_steps: Vec<OnboardingStep>,
    /// Whether onboarding was completed successfully.
    pub is_complete: bool,
    /// Number of times onboarding has been shown.
    pub view_count: usize,
}

impl Default for OnboardingState {
    fn default() -> Self {
        Self {
            initiated: false,
            in_progress: false,
            current_step: 0,
            completed_steps: Vec::new(),
            skipped_steps: Vec::new(),
            is_complete: false,
            view_count: 0,
        }
    }
}

/// All available onboarding steps in order.
pub fn all_steps() -> Vec<OnboardingStep> {
    vec![
        OnboardingStep::Welcome,
        OnboardingStep::AiProviderSetup,
        OnboardingStep::PrivacyPreferences,
        OnboardingStep::Appearance,
        OnboardingStep::KeyboardShortcuts,
        OnboardingStep::Tour,
        OnboardingStep::Complete,
    ]
}

/// The onboarding flow manager — tracks progress and provides navigation.
pub struct OnboardingManager {
    state: std::sync::Mutex<OnboardingState>,
    total_steps: usize,
}

impl Clone for OnboardingManager {
    fn clone(&self) -> Self {
        Self {
            state: std::sync::Mutex::new(self.state.lock().unwrap().clone()),
            total_steps: self.total_steps,
        }
    }
}

impl OnboardingManager {
    /// Create a new onboarding manager.
    pub fn new() -> Self {
        let total_steps = all_steps().len();
        Self {
            state: std::sync::Mutex::new(OnboardingState::default()),
            total_steps,
        }
    }

    /// Create from existing onboarding state.
    pub fn from_state(state: OnboardingState) -> Self {
        let total_steps = all_steps().len();
        Self {
            state: std::sync::Mutex::new(state),
            total_steps,
        }
    }

    /// Check if onboarding should be shown for a new user.
    ///
    /// Returns `true` if the user has never completed onboarding.
    pub fn should_show(&self) -> bool {
        let state = self.state.lock().unwrap();
        !state.is_complete
    }

    /// Initialize the onboarding flow.
    ///
    /// Called when the app starts for the first time.
    pub fn initiate(&self) {
        let mut state = self.state.lock().unwrap();
        state.initiated = true;
        state.in_progress = true;
        state.current_step = 0;
        state.view_count += 1;
        info!("Onboarding flow initiated");
    }

    /// Get the current step.
    pub fn current_step(&self) -> OnboardingStep {
        let state = self.state.lock().unwrap();
        all_steps()[state.current_step].clone()
    }

    /// Get the step at the given index.
    #[allow(dead_code)]
    pub fn step_at(&self, index: usize) -> Option<OnboardingStep> {
        all_steps().get(index).cloned()
    }

    /// Advance to the next step.
    ///
    /// Returns `true` if there is a next step, `false` if we're done.
    pub fn next_step(&self) -> bool {
        let mut state = self.state.lock().unwrap();
        if let Some(current) = all_steps().get(state.current_step).cloned() {
            if !state.completed_steps.contains(&current) {
                state.completed_steps.push(current.clone());
            }
        }

        state.current_step += 1;

        if state.current_step >= self.total_steps {
            state.in_progress = false;
            state.is_complete = true;
            info!("Onboarding completed successfully");
            return false;
        }

        info!(
            current_step = state.current_step,
            "Onboarding: advanced to next step"
        );
        true
    }

    /// Skip the current step.
    ///
    /// Returns `true` if the step can be skipped (it must be optional).
    pub fn skip_step(&self) -> bool {
        let mut state = self.state.lock().unwrap();
        let current = all_steps().get(state.current_step).cloned();

        if let Some(step) = current {
            if step.is_optional() {
                state.skipped_steps.push(step);
                info!("Onboarding: skipped step {:?}", step);
                true
            } else {
                warn!(
                    "Cannot skip non-optional step: {:?}",
                    step
                );
                false
            }
        } else {
            false
        }
    }

    /// Go back to the previous step.
    ///
    /// Returns `true` if there is a previous step.
    pub fn previous_step(&self) -> bool {
        let mut state = self.state.lock().unwrap();
        if state.current_step > 0 {
            state.current_step -= 1;
            info!(
                current_step = state.current_step,
                "Onboarding: went back to previous step"
            );
            true
        } else {
            false
        }
    }

    /// Mark the current step as completed.
    pub fn complete_current_step(&self) {
        let mut state = self.state.lock().unwrap();
        if let Some(step) = all_steps().get(state.current_step).cloned() {
            if !state.completed_steps.contains(&step) {
                state.completed_steps.push(step);
            }
            info!("Onboarding: completed step {:?}", step);
        }
    }

    /// Check if all required steps are completed.
    #[allow(dead_code)]
    pub fn all_required_completed(&self) -> bool {
        let state = self.state.lock().unwrap();
        let required_steps: Vec<OnboardingStep> =
            all_steps().into_iter().filter(|s| !s.is_optional()).collect();

        required_steps
            .iter()
            .all(|step| state.completed_steps.contains(step) || state.skipped_steps.contains(step))
    }

    /// Get progress information.
    #[allow(dead_code)]
    pub fn progress(&self) -> OnboardingProgress {
        let state = self.state.lock().unwrap();
        OnboardingProgress {
            current_step: state.current_step,
            total_steps: self.total_steps,
            completed: state.completed_steps.len(),
            skipped: state.skipped_steps.len(),
            in_progress: state.in_progress,
            is_complete: state.is_complete,
            percentage: if self.total_steps > 0 {
                (state.current_step as f64 / self.total_steps as f64) * 100.0
            } else {
                100.0
            },
        }
    }

    /// Reset onboarding (for testing or re-showing).
    #[allow(dead_code)]
    pub fn reset(&self) {
        let mut state = self.state.lock().unwrap();
        *state = OnboardingState::default();
        info!("Onboarding flow reset");
    }

    /// Mark onboarding as fully complete (for first-run flow).
    pub fn mark_complete(&self) {
        let mut state = self.state.lock().unwrap();
        state.is_complete = true;
        state.in_progress = false;
        state.completed_steps = all_steps();
        info!("Onboarding marked as complete");
    }

    /// Get the onboarding state for serialization.
    pub fn get_state(&self) -> OnboardingState {
        self.state.lock().unwrap().clone()
    }

    /// Set the onboarding state (used for loading from config).
    #[allow(dead_code)]
    pub fn set_state(&self, state: OnboardingState) {
        let mut current = self.state.lock().unwrap();
        *current = state;
    }
}

impl Default for OnboardingManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Progress information for the onboarding flow.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OnboardingProgress {
    /// The current step index (0-based).
    pub current_step: usize,
    /// Total number of steps.
    pub total_steps: usize,
    /// Number of completed steps.
    pub completed: usize,
    /// Number of skipped steps.
    pub skipped: usize,
    /// Whether onboarding is currently in progress.
    pub in_progress: bool,
    /// Whether onboarding is fully complete.
    pub is_complete: bool,
    /// Progress percentage (0.0–100.0).
    pub percentage: f64,
}

/// Create an onboarding manager with defaults.
pub fn create_onboarding_manager() -> OnboardingManager {
    OnboardingManager::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_steps_count() {
        let steps = all_steps();
        assert_eq!(steps.len(), 7);
    }

    #[test]
    fn test_step_sequence() {
        assert_eq!(
            OnboardingStep::Welcome.next(),
            Some(OnboardingStep::AiProviderSetup)
        );
        assert_eq!(
            OnboardingStep::PrivacyPreferences.next(),
            Some(OnboardingStep::Appearance)
        );
        assert_eq!(OnboardingStep::Complete.next(), None);
        assert_eq!(OnboardingStep::Welcome.previous(), None);
        assert_eq!(
            OnboardingStep::AiProviderSetup.previous(),
            Some(OnboardingStep::Welcome)
        );
    }

    #[test]
    fn test_optional_steps() {
        assert!(OnboardingStep::AiProviderSetup.is_optional());
        assert!(OnboardingStep::Tour.is_optional());
        assert!(!OnboardingStep::Welcome.is_optional());
        assert!(!OnboardingStep::PrivacyPreferences.is_optional());
    }

    #[test]
    fn test_onboarding_flow() {
        let mgr = OnboardingManager::new();
        assert!(!mgr.should_show()); // Not initiated yet
        assert!(!mgr.is_complete());

        mgr.initiate();
        assert!(mgr.should_show());
        assert_eq!(mgr.current_step(), OnboardingStep::Welcome);

        // Complete first step
        mgr.complete_current_step();

        // Advance
        assert!(mgr.next_step());
        assert_eq!(mgr.current_step(), OnboardingStep::AiProviderSetup);

        // Skip optional step
        assert!(mgr.skip_step());

        // Advance
        assert!(mgr.next_step());
        assert_eq!(mgr.current_step(), OnboardingStep::PrivacyPreferences);
    }

    #[test]
    fn test_progress() {
        let mgr = OnboardingManager::new();
        let progress = mgr.progress();
        assert_eq!(progress.current_step, 0);
        assert_eq!(progress.total_steps, 7);
        assert_eq!(progress.completed, 0);
        assert_eq!(progress.percentage, 0.0);
    }
}
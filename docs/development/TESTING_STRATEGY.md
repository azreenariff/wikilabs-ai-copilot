# Wiki Labs AI Copilot — Testing Strategy

**Date:** 2026-07-16
**Status:** Draft
**Version:** 0.1

---

## Table of Contents

1. [Overview](#1-overview)
2. [Testing Pyramid](#2-testing-pyramid)
3. [Rust Testing](#3-rust-testing)
4. [Frontend Testing](#4-frontend-testing)
5. [Tauri-Specific Testing](#5-tauri-specific-testing)
6. [AI / ML Testing](#6-aiml-testing)
7. [MCP Skill Testing](#7-mcp-skill-testing)
8. [Observation Engine Testing](#8-observation-engine-testing)
9. [Security Testing](#9-security-testing)
10. [Performance Testing](#10-performance-testing)
11. [Cross-Platform Testing](#11-cross-platform-testing)
12. [CI Testing Pipeline](#12-ci-testing-pipeline)
13. [Test Data Management and Fixtures](#13-test-data-management-and-fixtures)
14. [Coverage Targets and Quality Gates](#14-coverage-targets-and-quality-gates)
15. [Appendix: Test Matrix Quick Reference](#15-appendix-test-matrix-quick-reference)

---

## 1. Overview

This document defines the comprehensive testing strategy for **Wiki Labs AI Copilot** — a production-grade desktop AI engineering copilot for infrastructure and DevOps engineers. The strategy addresses all architectural layers: the React frontend, Rust core engine, MCP skill servers, AI provider abstractions, observation engine, and security infrastructure.

### Principles

1. **Automate everything feasible.** Manual testing is reserved for exploratory QA, usability review, and edge-case discovery.
2. **Test behavior, not implementation.** Tests verify what the system does, not how it does it.
3. **Fast feedback first.** Unit tests run in milliseconds; E2E tests run in minutes. CI must not block a developer for more than 5 minutes on routine PRs.
4. **Local-first testing.** Because the app is local-first, tests must validate offline behavior, degraded mode, and local data persistence.
5. **Privacy by design in tests.** Test data must never contain real credentials, real screen captures, or real customer data. All test fixtures use synthetic data.
6. **Human-in-the-loop is testable.** Because the AI never acts autonomously, every recommendation path must be verifiable via deterministic test cases.

### Test Categories Summary

| Layer | Unit | Integration | E2E | Validation |
|-------|------|-------------|-----|------------|
| React Frontend | Vitest + RTL | Vitest + MSW | Playwright | — |
| Rust Core | `cargo test` | `cargo test` (mod integration) | — | Property-based (`proptest`) |
| MCP Skills | SDK unit tests | RPC integration tests | End-to-end with mock client | Domain-specific validation |
| AI/ML | Prompt unit tests | Pipeline integration | Human-in-the-loop eval | Accuracy benchmarks |
| Observation Engine | Mock capture tests | End-to-end with mock OS | Visual regression | OCR accuracy benchmarks |
| Security | Unit tests (crypto) | Integration tests | Penetration tests | Certification |
| Performance | Benchmark tests | Load tests | — | Regression gates |

---

## 2. Testing Pyramid

The testing pyramid defines the distribution of test effort across layers. For Wiki Labs AI Copilot, the target distribution is:

```
                          /\ E2E (10%)
                         /  \
                        /    \
                       /      \
                      /        \
                     /          \
                    /            \
                   /              \
                  / Integration (25%)
                 /                \
                /                  \
               /                    \
              /                      \
             /                        \
            /                          \
           / Unit (65%)                 \
          /                              \
```

### 2.1 Unit Tests — 65% of total

Unit tests validate individual functions, methods, and structs in isolation. They are fast (sub-millisecond to millisecond), require no external dependencies, and use mocks/stubs for all I/O.

**Targets:**
- All pure functions (parsing, transformation, classification)
- Individual structs/methods (observers, engines, managers)
- Error handling paths
- Edge cases (empty input, boundary values, protocol errors)

### 2.2 Integration Tests — 25% of total

Integration tests validate interactions between components and subsystems. They may use real databases, real file I/O, and real network calls (to test servers or mocked providers).

**Targets:**
- Core ↔ Frontend RPC protocol
- Core ↔ MCP server communication
- Knowledge system import → index → search pipeline
- Workspace lifecycle (create, switch, delete)
- SQLite persistence with real migrations
- Event bus propagation across engines

### 2.3 E2E Tests — 10% of total

E2E tests validate complete user workflows from a user's perspective through the actual desktop application. They run against a built binary.

**Targets:**
- Complete troubleshooting workflow (screen detection → recommendation → chat interaction)
- Workspace creation and switching
- Settings modification and persistence
- Skill installation and activation
- Offline mode behavior
- Installer validation (MSI/DMG installation, first-run experience)

---

## 3. Rust Testing

### 3.1 Standard Library Tests (`cargo test`)

All Rust code uses `cargo test` (libtest) as the primary testing framework. This is the Rust standard and is the same approach proven in OpenHuman.

**Directory structure:**
```
src/
├── core/
│   ├── lib.rs
│   ├── tests.rs              # Inline module-level tests
│   └── observation/
│       ├── mod.rs
│       ├── screen.rs
│       └── tests/
│           └── mod.rs        # Integration tests for observation
├── frontend/
│   └── ...
└── mcp/
    ├── manager.rs
    └── tests/
        └── mod.rs
tests/                          # Crates.io-style integration tests
├── workspace_flow.rs
├── knowledge_pipeline.rs
└── mcp_integration.rs
```

**Naming convention:**
- Module-level tests: `#[cfg(test)] mod tests;` with functions `fn test_<function_name>()`
- Integration tests: File names prefixed with `test_` or suffixed with `_flow.rs`/`_integration.rs`

**Example — Intent recognition unit test:**
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_openShift_intent_detection() {
        let context = AnalysisContext {
            observation: ObservationSnapshot {
                active_app: "Google Chrome".into(),
                window_title: "OpenShift Console - Production".into(),
                visible_text: vec!["prod-api-7d8f4 CrashLoopBackOff".into()],
                content_type: ContentType::Web,
            },
            recent_commands: vec![CommandRecord {
                command: "oc describe pod prod-api-7d8f4".into(),
                output: "...OOMKilled...".into(),
                exit_code: 0,
            }],
            conversation: vec![],
            workspace: WorkspaceContext {
                tech_stack: vec!["OpenShift".into()],
                ..Default::default()
            },
            previous_intents: vec![],
        };

        let engine = IntentEngine::new();
        let intent = engine.analyze(context).await.unwrap();

        assert_eq!(intent.technology, "OpenShift");
        assert_eq!(intent.goal, "troubleshoot");
        assert!(intent.confidence > 0.8);
    }
}
```

### 3.2 Property-Based Testing with `proptest`

Property-based testing verifies that invariants hold for arbitrary inputs. Use the `proptest` crate for Rust property tests. This is especially valuable for:

- **Intent recognition confidence scoring** — confidence should always be in [0.0, 1.0]
- **Credential redaction** — redacted output should never contain recognizable credential patterns
- **OCR text extraction** — extracted text should be valid UTF-8
- **Knowledge chunking** — chunks should not exceed configured max token count
- **Hybrid search ranking** — re-ranking should be deterministic for same inputs

**Example — Credential redaction property test:**
```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_redaction_never_leaks_credentials(text in any::<String>()) {
        let redacted = redact_credentials(&text);
        // No credential pattern should survive redaction
        assert!(!PASSWORD_PATTERN.is_match(&redacted));
        assert!(!API_KEY_PATTERN.is_match(&redacted));
        assert!(!CONNECTION_STRING_PATTERN.is_match(&redacted));
    }

    #[test]
    fn test_redaction_preserves_non_secrets(text in any::<String>()) {
        let non_secret = "kubectl get pods";
        let input = format!("echo {} | oc apply", non_secret);
        let redacted = redact_credentials(&input);
        assert!(redacted.contains(non_secret));
    }
}
```

### 3.3 Async Testing with `tokio-test`

The Rust core runs on Tokio. Use `tokio-test` for async test utilities:

```rust
use tokio_test::{assert_ready, assert_ready_ok};

#[tokio::test]
async fn test_skill_startup_and_health_check() {
    let handle = SkillManager::new();
    let skill = handle.load_skill("linux").await.unwrap();

    // Skill process should start within 5 seconds
    let started = tokio::time::timeout(
        Duration::from_secs(5),
        handle.start_skill("linux"),
    ).await.unwrap().unwrap();

    // Health check should pass
    assert!(started.health.check().await.is_ok());

    // Clean up
    handle.stop_skill("linux").await.unwrap();
}
```

**Timeout strategy:** All async tests that involve I/O, network calls, or process spawning MUST have explicit timeouts. Default: 5 seconds for local operations, 30 seconds for network operations.

### 3.4 Mock Backend for Integration Tests

Use the `mockall` crate or `wiremock` for mocking external services during integration tests:

```rust
use mockall::mock;

mock! {
    pub AiProvider {
        async fn chat(&self, messages: Vec<ChatMessage>, params: ChatParams) -> Result<ChatResponse>;
        async fn embed(&self, text: String) -> Result<Vec<f32>>;
    }
}

#[tokio::test]
async fn test_knowledge_context_injection() {
    let mut mock_provider = MockAiProvider::new();
    mock_provider
        .expect_chat()
        .times(1)
        .returning(move |_msgs, _params| {
            Ok(ChatResponse {
                content: "Based on the OOMKilled event".into(),
                references: vec!["openshift-guide-4.14.md".into()],
            })
        });

    let result = reasoning_engine
        .reason(mock_provider, knowledge_context)
        .await;

    assert!(result.content.contains("OOMKilled"));
}
```

### 3.5 Benchmarking with `criterion`

Use `criterion` crate for performance benchmarking. Benchmarks run as part of CI and are regression-gated.

**Benchmark targets:**
```rust
use criterion::{criterion_group, criterion_main, Criterion};

fn benchmark_knowledge_search(c: &mut Criterion) {
    c.bench_function("hybrid_search_100_docs", |b| {
        let state = SearchBenchmarkState::setup(100);
        b.to_async(tokio::runtime::Runtime::new().unwrap())
            .iter(|| async { state.engine.search("pod crash".into()).await })
    });

    c.bench_function("hybrid_search_10k_docs", |b| {
        let state = SearchBenchmarkState::setup(10_000);
        b.to_async(tokio::runtime::Runtime::new().unwrap())
            .iter(|| async { state.engine.search("pod crash".into()).await })
    });
}

fn benchmark_screen_ocr(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let screenshot = include_bytes!("fixtures/openShift_console.png");

    c.bench_function("tesseract_ocr_1920x1080", |b| {
        b.iter(|| rt.block_on(OcrEngine::extract_text(screenshot)))
    });
}

criterion_group!(benches, benchmark_knowledge_search, benchmark_screen_ocr, benchmark_skill_routing);
criterion_main!(benches);
```

**CI benchmark gates:**
- Knowledge search (100 docs): p95 < 100ms
- Knowledge search (10k docs): p95 < 500ms
- OCR extraction (1920×1080): < 2 seconds
- Skill routing (discover + select): < 500ms

---

## 4. Frontend Testing

### 4.1 Component Tests with React Testing Library (RTL)

Use **React Testing Library** (via Vitest) to test components from the user's perspective. Test what users see and interact with, not internal React state or implementation details.

```typescript
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { ChatInterface } from './ChatInterface';
import { mockRpc } from './test-helpers';

describe('ChatInterface', () => {
  beforeEach(() => mockRpc.reset());

  it('displays AI responses with markdown formatting', async () => {
    mockRpc.rpcChatSend.mockResolvedValue({
      content: 'The pod is **OOMKilled**. Try increasing memory limits.',
      references: ['openshift-guide-4.14.md'],
    });

    render(<ChatInterface workspaceId="acme-prod" />);

    await userEvent.type(screen.getByRole('textbox'), 'Why is my pod crashing?');
    await fireEvent.click(screen.getByRole('button', { name: /send/i }));

    await waitFor(() =>
      screen.getByText(/OOMKilled/i, { exact: false })
    );
  });

  it('shows confidence indicator when AI provides confidence score', async () => {
    mockRpc.rpcChatSend.mockResolvedValue({
      content: 'Pod is likely OOMKilled',
      confidence: 0.87,
    });

    render(<ChatInterface workspaceId="acme-prod" />);

    // ... interaction ...
    await waitFor(() =>
      screen.getByTestId('confidence-indicator')
    );
    expect(screen.getByText('High')).toBeInTheDocument();
  });
});
```

**Component testing checklist:**
- [ ] All chat message types render correctly (text, code_block, warning, confidence, source_citation, action_card)
- [ ] Streaming responses render progressively
- [ ] Stop generation button works during streaming
- [ ] Copy button appears on every AI message
- [ ] Markdown rendering with code blocks and syntax highlighting
- [ ] Workspace context indicator updates on switch
- [ ] Suggestion cards appear, can be accepted/dismissed
- [ ] Settings panel saves and persists configuration
- [ ] Offline banner appears when core is unavailable
- [ ] Error states display appropriate messages

### 4.2 State Management Testing (Redux Toolkit)

Test Redux slices and thunks separately from components:

```typescript
import { configureStore } from '@reduxjs/toolkit';
import { chatSlice, initialState } from './chatSlice';
import { sendMessage } from './chatThunks';

describe('Chat Redux', () => {
  let store: ReturnType<typeof configureStore>;

  beforeEach(() => {
    store = configureStore({
      reducer: { chat: chatSlice.reducer },
      preloadedState: { chat: initialState },
    });
  });

  it('adds message to conversation', async () => {
    await store.dispatch(sendMessage({
      workspaceId: 'acme',
      messages: [{ role: 'user', content: 'help' }],
    }));

    const state = store.getState().chat;
    expect(state.messages.length).toBe(2);
    expect(state.messages[0].role).toBe('user');
    expect(state.messages[1].role).toBe('assistant');
  });

  it('handles streaming errors gracefully', async () => {
    mockRpc.rpcChatSend.mockRejectedValue(new Error('Provider unavailable'));

    await store.dispatch(sendMessage({ ... }));

    const state = store.getState().chat;
    expect(state.lastError).toBeDefined();
    expect(state.isError).toBe(true);
  });
});
```

### 4.3 API Mocking with MSW (Mock Service Worker)

Use **Mock Service Worker** to mock all RPC and Tauri IPC calls in frontend tests. This ensures tests are independent of the Rust core.

```typescript
import { http, HttpResponse } from 'msw';
import { setupServer } from 'msw/node';

// Mock the JSON-RPC endpoint
const rpcHandlers = [
  http.post('/rpc', async ({ request }) => {
    const body = await request.json();
    switch (body.method) {
      case 'rpc:chat_send':
        return HttpResponse.json({
          result: { content: 'test response', confidence: 0.9 },
        });
      case 'rpc:workspace_list':
        return HttpResponse.json({
          result: [
            { id: 'w1', name: 'Acme Corp', created_at: '2026-01-01' },
          ],
        });
      default:
        return HttpResponse.json({ error: { code: -32601, message: 'Method not found' } }, { status: 404 });
    }
  }),
];

const server = setupServer(...rpcHandlers);

beforeAll(() => server.listen());
afterEach(() => {
  server.resetHandlers();
  server.close();
});
```

### 4.4 E2E Testing with Playwright

**Playwright** is the selected E2E testing tool. It provides cross-platform reliability and excellent Tauri support via the `@tauri-apps/plugin-dialog` and custom context helpers.

**Test file organization:**
```
tests/e2e/
├── helpers/
│   ├── app.ts          # Launch Tauri app, setup Playwright context
│   ├── rpc.ts          # Mock RPC responses in e2e
│   └── fixtures.ts     # Shared test fixtures
├── smoke.spec.ts       # Critical path smoke tests
├── chat.spec.ts        # Chat interface tests
├── workspace.spec.ts   # Workspace management tests
├── observation.spec.ts # Observation engine UX tests
├── skills.spec.ts      # Skill lifecycle tests
├── settings.spec.ts    # Settings persistence tests
├── offline.spec.ts     # Offline behavior tests
└── security.spec.ts    # Security UX tests
```

**App launcher helper:**
```typescript
// tests/e2e/helpers/app.ts
import { test as base } from '@playwright/test';
import { spawn } from 'child_process';
import type { ElectronApplication } from '@tauri-apps/plugin-dialog';

export const test = base.extend({
  app: async ({ }, use) => {
    const process = spawn('cargo tauri dev', {
      stdio: 'pipe',
      env: { ...process.env, MOCK_AI_PROVIDER: 'true' },
    });

    // Wait for app to be ready
    await new Promise<void>((resolve) => {
      process.stdout.on('data', (data) => {
        if (data.toString().includes('ready')) {
          resolve();
        }
      });
    });

    yield process;
    process.kill();
  },
});
```

**E2E test example — complete troubleshooting workflow:**
```typescript
import { test, expect } from '../helpers/app';

test.describe('Troubleshooting workflow', () => {
  test('user gets context-aware recommendation when OpenShift console is active', async ({ page }) => {
    // 1. Mock the AI response for the context
    test.mockRPC('rpc:chat_send', {
      content: 'I see you\'re on the OpenShift Console. Your pod prod-api-7d8f4 is in CrashLoopBackOff.',
      confidence: 0.92,
    });

    // 2. Navigate to the app
    await page.goto('/');

    // 3. Simulate observation context (via test harness)
    await test.mockObservation({
      active_app: 'Google Chrome',
      window_title: 'OpenShift Console - Production',
      visible_text: ['prod-api-7d8f4 CrashLoopBackOff'],
      content_type: 'web',
    });

    // 4. Verify suggestion appears
    await expect(page.getByRole('status', { name: /recommendation/i })).toBeVisible();
    await expect(page.getByText(/CrashLoopBackOff/i)).toBeVisible();

    // 5. User asks follow-up
    await page.getByRole('textbox').fill('What do you think caused it?');
    await page.getByRole('button', { name: /send/i }).click();

    // 6. Verify AI response
    await expect(page.getByText(/OOMKilled/i)).toBeVisible();
  });
});
```

**E2E test coverage targets:**
- [ ] Chat: Send message → receive streaming response → stop → regenerate
- [ ] Chat: Multi-turn conversation with context retention
- [ ] Workspace: Create → configure → switch → delete
- [ ] Settings: Modify → save → verify persistence after restart
- [ ] Skills: Install → enable → call tool → disable → uninstall
- [ ] Knowledge: Import document → search → verify relevance
- [ ] Offline: Disconnect network → verify offline banner → reconnect → verify recovery
- [ ] Privacy: Configure exclusion list → verify excluded apps are not observed
- [ ] Suggestions: Accept suggestion → dismiss suggestion → verify state

---

## 5. Tauri-Specific Testing

### 5.1 Rust Side — Tauri Command Testing

Tauri commands (Rust functions exposed to the frontend) are tested as Rust integration tests:

```rust
#[cfg(test)]
mod tauri_commands {
    use super::*;
    use tauri::test::MockBuilder;

    #[tokio::test]
    async fn test_workspace_create_command() {
        let app = MockBuilder::new("wikilabs-copilot")
            .setup(|_| {})
            .build()
            .unwrap();

        // Simulate the frontend calling the Tauri command
        let command = app.command("create_workspace").unwrap();
        let payload = serde_json::json!({
            "name": "Test Customer",
            "environment": "Development",
            "region": "us-east-1",
            "tech_stack": ["OpenShift"]
        });

        let result = command.invoke(payload).await.unwrap();
        let workspace: Workspace = serde_json::from_value(result).unwrap();

        assert_eq!(workspace.name, "Test Customer");
        assert!(!workspace.id.is_empty());
        assert_eq!(workspace.tech_stack.len(), 1);
    }
}
```

### 5.2 IPC Testing

Test the JSON-RPC communication between frontend and Rust core. This validates serialization, error handling, and protocol compliance.

```rust
#[tokio::test]
async fn test_rpc_protocol_compliance() {
    let rpc_server = RpcServer::bind("127.0.0.1:0").await.unwrap();
    let port = rpc_server.local_addr().port();

    let client = RpcClient::connect(&format!("http://127.0.0.1:{}", port)).await.unwrap();

    // Test request/response roundtrip
    let response = client
        .request("rpc:workspace_list", serde_json::json!([]))
        .await
        .unwrap();

    assert!(response.is_success());
    // Response is a JSON-RPC 2.0 success envelope
    assert_eq!(response.json.get("id"), Some(&serde_json::json!(1)));
    assert_eq!(response.json.get("jsonrpc"), Some(&serde_json::json!("2.0")));

    // Test error response
    let error_response = client
        .request("rpc:workspace_delete", serde_json::json!({ "id": "nonexistent" }))
        .await
        .unwrap();

    assert!(error_response.is_error());
    assert_eq!(error_response.error.code, -32000);
}
```

**IPC test checklist:**
- [ ] All RPC methods have matching frontend and Rust implementations
- [ ] Error codes are consistent (JSON-RPC 2.0 + custom domain codes)
- [ ] WebSocket events fire correctly on state changes
- [ ] Connection drops are handled gracefully with auto-reconnect
- [ ] Large payloads (e.g., long conversation histories) serialize correctly
- [ ] Concurrent requests are handled without race conditions

### 5.3 Tauri WebView Testing

Tauri v2 uses the OS system WebView (EdgeWebView on Windows, WKWebView on macOS). Test WebView-specific behaviors:

```rust
#[tokio::test]
async fn test_webview_initialization() {
    let app_builder = AppBuilder::default()
        .uri(AppUri::Asset("./dist/index.html".into()))
        .build(|ctx| Ok(ctx.handle()))
        .unwrap();

    let app = app_builder.app();
    let webview = app.get_webview("main").unwrap();

    // Verify the webview loads and the page title is set
    assert!(webview.title().is_ok());

    // Verify JavaScript injection works (for test harness)
    let title: String = webview.eval("document.title").unwrap();
    assert!(!title.is_empty());
}
```

### 5.4 Tauri Build Testing

Validate that production builds compile and install correctly on both platforms:

```yaml
# In CI (see Section 12)
steps:
  - name: Build Windows installer
    run: cargo tauri build --target x86_64-pc-windows-msi
  - name: Validate MSI installation
    run: |
      msiexec /i wikilabs-copilot.msi /qn
      # Verify installation in registry
      # Verify executable exists
      # Run app once to verify first-run
      # Uninstall
      msiexec /x {GUID} /qn
  - name: Build macOS DMG
    run: cargo tauri build --target aarch64-apple-darwin
  - name: Validate DMG installation
    run: |
      hdiutil attach wikilabs-copilot.dmg
      cp -R /Volumes/wikilabs-copilot/Wiki\ Labs\ AI\ Copilot.app /Applications/
      # Verify app launches
      # Verify code signature
      codesign --verify /Applications/Wiki\ Labs\ AI\ Copilot.app
```

---

## 6. AI / ML Testing

### 6.1 Prompt Evaluation

Every prompt sent to an AI provider is versioned and testable. Use a prompt evaluation framework to verify prompt quality, safety, and effectiveness.

**Prompt versioning:**
```rust
struct PromptVersion {
    id: String,           // e.g., "troubleshooting-pod-crash-v3"
    template: String,     // Jinja2-style prompt template
    variables: Vec<String>, // Variable names
    test_cases: Vec<PromptTestCase>, // Deterministic test cases
    eval_metrics: PromptMetrics, // Evaluated metrics
}

struct PromptTestCase {
    name: String,
    input: HashMap<String, String>,
    expected_properties: Vec<PromptProperty>,
    forbidden_patterns: Vec<Regex>, // Patterns that should never appear
}

enum PromptProperty {
    Contains(String),        // Output must contain this text
    ConfidenceAbove(f32),    // Confidence score above threshold
    ReferencesValidSources,  // References point to real docs
    IsSafe,                  // No dangerous suggestions
    IsSpecific,              // Not generic/vague
}
```

**Example — Prompt test for pod crash troubleshooting:**
```rust
#[tokio::test]
async fn test_troubleshooting_prompt_quality() {
    let prompt = PromptVersion::get("troubleshooting-pod-crash-v3");
    let test_case = prompt.test_cases[0].clone();

    let variables: HashMap<String, String> = test_case
        .input
        .into_iter()
        .map(|(k, v)| (k, v))
        .collect();

    let rendered = prompt.render(&variables);

    // Prompt should not contain any literal secrets from test data
    for forbidden in &prompt.test_cases[0].forbidden_patterns {
        assert!(!forbidden.is_match(&rendered));
    }

    // Prompt should include all required variables
    for var in &prompt.variables {
        assert!(rendered.contains(&format!("{{{var}}}")));
    }
}
```

### 6.2 AI Response Quality Evaluation

Use a deterministic evaluation harness to score AI responses. Since responses may be non-deterministic, run evaluations as regression tests with acceptance criteria rather than exact match assertions.

**Evaluation framework:**
```rust
struct AiEvaluationResult {
    prompt_id: String,
    test_case: String,
    response: String,
    quality_score: QualityScore,      // Composite score
    safety_score: f32,                // 0.0 - 1.0
    relevance_score: f32,             // 0.0 - 1.0
    accuracy_score: f32,              // 0.0 - 1.0 (validated by experts or ground truth)
    contains_dangerous_suggestions: bool,
    references_valid_sources: bool,
    hallucinated_references: Vec<String>,
}

struct QualityScore {
    composite: f32,   // Weighted average
    thresholds: QualityThresholds,
}
```

**Evaluation test suite:**
```rust
#[tokio::test]
async fn evaluate_pod_crash_responses() {
    let evaluator = AiEvaluator::new();

    let results = evaluator.evaluate_batch("troubleshooting-pod-crash-v3")
        .await
        .unwrap();

    for result in &results {
        assert!(result.quality_score.composite >= 0.7,
            "Quality score below threshold for test case '{}'", result.test_case);
        assert!(result.safety_score >= 0.95,
            "Safety score too low for test case '{}'", result.test_case);
        assert!(!result.contains_dangerous_suggestions,
            "Dangerous suggestion found in test case '{}'", result.test_case);
        assert!(result.hallucinated_references.is_empty(),
            "Hallucinated references found in test case '{}': {:?}",
            result.test_case, result.hallucinated_references);
    }

    // Aggregate gate
    let avg_quality = results.iter().map(|r| r.quality_score.composite).sum::<f32>() / results.len() as f32;
    assert!(avg_quality >= 0.80, "Aggregate quality below 0.80");
}
```

### 6.3 Intent Recognition Accuracy

Intent recognition accuracy is measured by running the engine against a curated test dataset of observation-context pairs with known ground-truth intents.

**Test dataset format:**
```yaml
# tests/fixtures/intent_recognition_dataset.yaml
test_cases:
  - id: "int-001"
    description: "User on OpenShift console investigating pod crash"
    observation:
      active_app: "Google Chrome"
      window_title: "OpenShift Console - Production Cluster"
      visible_text:
        - "prod-api-7d8f4 CrashLoopBackOff"
        - "ImagePullBackOff"
      content_type: "web"
    recent_commands:
      - command: "oc get pods -n prod"
        output: "prod-api-7d8f4  0/1  CrashLoopBackOff"
        exit_code: 0
    expected_intent:
      technology: "OpenShift"
      goal: "troubleshoot"
      min_confidence: 0.85
    alternatives:
      - technology: "Kubernetes"
        max_confidence: 0.20

  - id: "int-002"
    description: "User in VMware vSphere console checking VM performance"
    observation:
      active_app: "VMware Workstation"
      window_title: "prod-web-01 - vSphere Client"
      visible_text:
        - "CPU: 92%"
        - "Memory: 87%"
      content_type: "dashboard"
    recent_commands:
      - command: "vmstat 1 5"
        exit_code: 0
    expected_intent:
      technology: "VMware"
      goal: "troubleshoot_performance"
      min_confidence: 0.80
```

**Accuracy test:**
```rust
#[tokio::test]
async fn test_intent_recognition_accuracy() {
    let dataset = IntentDataset::load("intent_recognition_dataset.yaml");
    let engine = IntentEngine::new();

    let mut correct = 0u32;
    let mut total = 0u32;

    for case in &dataset.test_cases {
        let context = case.to_analysis_context();
        let intent = engine.analyze(context).await.unwrap();
        total += 1;

        let tech_match = intent.technology == case.expected_intent.technology;
        let goal_match = intent.goal == case.expected_intent.goal;
        let confidence_met = intent.confidence >= case.expected_intent.min_confidence;

        if tech_match && goal_match && confidence_met {
            correct += 1;
        }
    }

    let accuracy = correct as f32 / total as f32;
    assert!(accuracy >= 0.85,
        "Intent recognition accuracy ({:.1}%) below 85% threshold: {} correct out of {}",
        accuracy * 100.0, correct, total);
}
```

### 6.4 Embedding Quality Testing

Verify that the embedding model produces meaningful embeddings for the infrastructure domain:

```rust
#[tokio::test]
async fn test_embedding_similarity() {
    let embedder = EmbeddingModel::new("all-MiniLM-L6-v2");

    let query = embedder.embed("OpenShift pod CrashLoopBackOff troubleshooting").await.unwrap();
    let relevant = embedder.embed("Container restart loop in Kubernetes").await.unwrap();
    let unrelated = embedder.embed("MySQL replication lag monitoring").await.unwrap();

    let cos_sim = |a: &[f32], b: &[f32]| { /* cosine similarity */ };

    let sim_relevant = cos_sim(&query, &relevant);
    let sim_unrelated = cos_sim(&query, &unrelated);

    assert!(sim_relevant > sim_unrelated + 0.2,
        "Embedding similarity for related texts ({:.3}) should exceed unrelated ({:.3}) by > 0.2",
        sim_relevant, sim_unrelated);

    // Domain clustering test
    let openshift_texts = vec![
        "pod CrashLoopBackOff",
        "image pull back off",
        "resource quota exceeded",
        "node not ready",
    ];

    let embeddings: Vec<Vec<f32>> = futures::future::join_all(
        openshift_texts.iter().map(|t| embedder.embed(t.to_string()))
    ).await.into_iter().collect::<Result<Vec<_>, _>>().unwrap();

    // All OpenShift-related texts should cluster together
    for i in 0..embeddings.len() {
        for j in (i+1)..embeddings.len() {
            assert!(cos_sim(&embeddings[i], &embeddings[j]) > 0.4,
                "Related infrastructure texts should cluster together");
        }
    }
}
```

### 6.5 AI Provider Abstraction Testing

Test that all supported providers implement the `AiProvider` trait correctly:

```rust
#[cfg(test)]
mod ai_provider_tests {
    use super::*;

    trait ProviderTestSuite {
        fn name(&self) -> &str;
        fn test_chat(&self) -> impl Future<Output = TestResult>;
        fn test_chat_stream(&self) -> impl Future<Output = TestResult>;
        fn test_embed(&self) -> impl Future<Output = TestResult>;
        fn test_health(&self) -> impl Future<Output = TestResult>;
    }

    #[tokio::test]
    async fn test_all_providers_provide_consistent_interface() {
        let providers: Vec<Box<dyn ProviderTestSuite>> = vec![
            Box::new(OpenAiProvider::from_test_config()),
            Box::new(OpenAiCompatibleProvider::from_test_config()),
            Box::new(OllamaProvider::from_test_config()),
        ];

        for provider in &providers {
            provider.test_health().await.unwrap();
            let chat = provider.test_chat().await.unwrap();
            assert!(!chat.content.is_empty());
            assert!(chat.references.is_empty() || verify_references(chat.references));
        }
    }
}
```

---

## 7. MCP Skill Testing

### 7.1 MCP Skill Unit Tests

Each MCP skill server is a standalone Rust binary with its own test suite. Every tool, resource, and prompt defined by the skill is tested individually.

**Skill test structure:**
```
skills/
├── openshift/
│   ├── Cargo.toml
│   ├── src/
│   │   ├── main.rs
│   │   ├── tools/
│   │   │   ├── investigate_crash_loop.rs
│   │   │   └── check_cluster_health.rs
│   │   └── resources/
│   │       └── troubleshooting_guide.rs
│   └── tests/
│       ├── mod.rs
│       ├── tool_investigate_crash_loop.rs
│       ├── tool_check_cluster_health.rs
│       └── resources_troubleshooting_guide.rs
```

**Skill unit test example:**
```rust
#[tokio::test]
async fn test_investigate_crash_loop_tool() {
    let skill = OpenShiftSkill::new();
    let tool = skill.get_tool("investigate_crash_loop").unwrap();

    let result = tool.call(serde_json::json!({
        "pod_name": "prod-api-7d8f4",
        "namespace": "production",
        "recent_errors": ["OOMKilled", "CrashLoopBackOff"],
    })).await.unwrap();

    // Tool should return structured troubleshooting recommendations
    assert!(!result.content.is_empty());
    assert!(result.content.iter().any(|c| c.type_ == "text"));

    // Output should include analysis of each error
    let text = result.content.iter()
        .filter(|c| c.type_ == "text")
        .map(|c| c.text.clone())
        .collect::<Vec<_>>()
        .join("\n");

    assert!(text.to_lowercase().contains("oom"));
    assert!(text.to_lowercase().contains("memory"));
}
```

### 7.2 MCP Protocol Compliance Tests

Verify that each skill server correctly implements the MCP protocol:

```rust
#[tokio::test]
async fn test_mcp_protocol_compliance() {
    let skill = OpenShiftSkill::spawn();
    let mut client = McpClient::connect(skill.stdio_handle()).await.unwrap();

    // 1. Initialize
    let init_result = client.initialize().await.unwrap();
    assert_eq!(init_result.protocol_version, "2024-11-05");

    // 2. List tools
    let tools = client.list_tools().await.unwrap();
    assert!(!tools.is_empty());

    for tool in &tools {
        // Each tool must have valid JSON schema
        assert!(tool.input_schema.is_object());
        assert!(tool.validate_schema().is_ok());

        // Tool name must be a valid identifier
        assert!(tool.name.chars().all(|c| c.is_alphanumeric() || c == '_'));
    }

    // 3. List resources
    let resources = client.list_resources().await.unwrap();
    for resource in &resources {
        assert!(resource.uri.starts_with("openshift://"));
    }

    // 4. List prompts
    let prompts = client.list_prompts().await.unwrap();
    for prompt in &prompts {
        assert!(!prompt.name.is_empty());
    }

    client.shutdown().await.unwrap();
}
```

### 7.3 MCP Skill Integration Tests

Test the end-to-end flow: intent recognition → skill selection → tool call → response.

```rust
#[tokio::test]
async fn test_end_to_end_skill_routing() {
    let mut skill_manager = SkillManager::new();
    let intent = Intent {
        technology: "OpenShift".into(),
        goal: "troubleshoot".into(),
        confidence: 0.92,
        ..Default::default()
    };

    // Skill manager should select the OpenShift skill
    let selected = skill_manager.find_relevant_skills(&intent).await.unwrap();
    assert!(selected.contains(&"openshift".into()));

    // Start the skill
    let handle = skill_manager.start_skill("openshift").await.unwrap();

    // Call the relevant tool
    let result = skill_manager
        .call_tool("openshift", "investigate_crash_loop", serde_json::json!({
            "pod_name": "test-pod",
            "namespace": "default",
        }))
        .await
        .unwrap();

    assert!(result.content.iter().any(|c| !c.text.is_empty()));

    // Clean up
    skill_manager.stop_skill("openshift").await.unwrap();
}
```

### 7.4 MCP Skill Quality Gates

Each skill must pass these quality gates before merging:

| Gate | Check | Threshold |
|------|-------|-----------|
| Tool count | Minimum viable tool set | ≥ 3 tools |
| Resource count | Reference materials available | ≥ 2 resources |
| Prompt coverage | Troubleshooting prompts available | ≥ 1 prompt |
| Schema validity | All tool schemas validate | 100% pass |
| Test coverage | Unit tests for all tools | ≥ 80% |
| Safety | No tool permits destructive action without confirmation | 100% pass |
| Documentation | Tool descriptions match implementation | 100% match |
| Runtime | Tool calls complete within timeout | p95 < 30s |

### 7.5 Skill Safety Tests

Verify that dangerous tools require human confirmation:

```rust
#[tokio::test]
async fn test_dangerous_tools_require_confirmation() {
    let skill = OpenShiftSkill::new();

    // This tool should be marked as dangerous
    let delete_tool = skill.get_tool("delete_deployment").unwrap();
    assert_eq!(delete_tool.safety_level, SafetyLevel::Danger);

    // Calling without confirmation should fail or return a warning
    let result = delete_tool.call_with_safety_check(
        serde_json::json!({ "deployment": "api", "namespace": "prod" }),
        SafetyContext::Unconfirmed,
    ).await;

    // Should either return an error or a danger warning
    match result {
        Ok(response) => {
            let text = response.content.iter()
                .filter(|c| c.type_ == "text")
                .map(|c| &c.text[..])
                .collect::<Vec<_>>()
                .join("");
            assert!(text.to_lowercase().contains("danger")
                || text.to_lowercase().contains("warning")
                || text.to_lowercase().contains("confirmation"));
        }
        Err(e) => {
            assert!(e.to_lowercase().contains("confirmation")
                || e.to_lowercase().contains("require"));
        }
    }
}
```

---

## 8. Observation Engine Testing

### 8.1 Screen Capture Testing

Screen capture uses OS-specific APIs (Win32 API on Windows, Quartz/CG on macOS). Tests use mocked captures.

**Mock observation data:**
```rust
struct MockScreenCapture {
    active_app: String,
    window_title: String,
    visible_text: Vec<String>,
    content_type: ContentType,
    errors_detected: Vec<String>,
    key_ui_elements: Vec<UIElement>,
}
```

**Test cases:**
```rust
#[tokio::test]
async fn test_screen_capture_active_window_mode() {
    let mut observer = ScreenObserver::new(CaptureMode::ActiveWindow);

    let capture = observer.capture().await.unwrap();

    assert!(capture.active_app.len() > 0);
    assert!(capture.window_title.len() > 0);
    assert!(capture.bounds.is_some());
}

#[tokio::test]
async fn test_screen_capture_full_screen_mode() {
    let mut observer = ScreenObserver::new(CaptureMode::FullScreen);

    let capture = observer.capture().await.unwrap();

    // Full screen capture should cover entire screen
    let bounds = capture.bounds.unwrap();
    assert!(bounds.width >= 1920);
    assert!(bounds.height >= 1080);
}

#[tokio::test]
async fn test_screen_capture_exclusion_rules() {
    let mut observer = ScreenObserver::new(CaptureMode::ActiveWindow);
    observer.set_exclusion_rules(vec![
        ExclusionRule {
            match_type: MatchType::ProcessName,
            pattern: "1Password".into(),
            action: ExclusionAction::Exclude,
        },
    ]);

    let capture = observer.capture_with_exclusions().await.unwrap();

    assert!(!capture.privacy_flags.is_empty());
    assert!(capture.privacy_flags.contains(&PrivacyFlag::ExcludedWindow));
}
```

### 8.2 OCR Accuracy Testing

OCR is tested against a curated test dataset of screenshots with known ground-truth text. Use a standard OCR engine (Tesseract or equivalent) as the baseline.

**OCR test dataset:**
```
tests/fixtures/ocr/
├── openshift_console.png    # Known: "prod-api-7d8f4 CrashLoopBackOff"
├── vmware_dashboard.png     # Known: "CPU: 92%, Memory: 87%"
├── terminal_output.png      # Known: "Error: connection refused"
├── nagios_alert.png         # Known: "CRITICAL: Disk usage 95%"
└── generic_error.png        # Known: "503 Service Unavailable"
```

**OCR accuracy benchmark:**
```rust
#[tokio::test]
async fn test_ocr_accuracy() {
    let ocr = OcrEngine::new("tesseract");

    let test_cases = vec![
        ("openshift_console.png", "prod-api-7d8f4 CrashLoopBackOff"),
        ("vmware_dashboard.png", "CPU: 92%"),
        ("terminal_output.png", "connection refused"),
        ("nagios_alert.png", "CRITICAL: Disk usage 95%"),
    ];

    let mut correct = 0u32;
    let mut total = 0u32;

    for (filename, expected_text) in test_cases {
        total += 1;
        let image = load_fixture(filename);
        let extracted = ocr.extract_text(&image).await.unwrap();

        // Check that key terms from expected text are present
        let terms: Vec<&str> = expected_text
            .split_whitespace()
            .filter(|w| w.len() > 3)  // Skip short words
            .collect();

        let matched = terms.iter()
            .filter(|t| extracted.to_lowercase().contains(&t.to_lowercase()))
            .count();

        let recall = matched as f32 / terms.len() as f32;
        if recall >= 0.7 {
            correct += 1;
        }

        // Log recall for monitoring
        println!("{}: recall={:.1}%", filename, recall * 100.0);
    }

    let accuracy = correct as f32 / total as f32;
    assert!(accuracy >= 0.80,
        "OCR recall below 80%: {} correct out of {}",
        correct, total);
}
```

### 8.3 Terminal Detection Testing

Test terminal session detection with mock data:

```rust
#[tokio::test]
async fn test_terminal_session_detection() {
    let terminal = TerminalObserver::new();

    // Simulate detecting a bash terminal
    let sessions = terminal.detect_sessions().await.unwrap();
    assert!(!sessions.is_empty());

    for session in &sessions {
        assert!(matches!(
            session.shell.as_str(),
            "bash" | "zsh" | "fish" | "pwsh" | "cmd" | "powershell"
        ));
        assert!(session.pid > 0);
    }
}

#[tokio::test]
async fn test_terminal_command_redaction() {
    let terminal = TerminalObserver::new();

    let command = "kubectl create secret generic db-creds --from-literal=password=SuperSecret123";
    let redacted = terminal.redact_credentials(command);

    assert!(!redacted.contains("SuperSecret123"));
    assert!(redacted.contains("password="));
    assert!(redacted.contains("REDACTED"));
}

#[tokio::test]
async fn test_terminal_output_parsing() {
    let terminal = TerminalObserver::new();

    let output = r#"
NAME                    STATUS     RESTARTS   AGE
prod-api-7d8f4          0/1        CrashLoopBackOff   12h
prod-worker-3a2b1       1/1        Running            3d

Last restart reason: OOMKilled
    "#;

    let parsed = terminal.parse_output(output).await.unwrap();
    assert_eq!(parsed.commands.len(), 0);  // This is output, not commands
    assert_eq!(parsed.errors.len(), 2);    // CrashLoopBackOff + OOMKilled
    assert_eq!(parsed.exit_code, None);    // No exit code in output
}
```

### 8.4 Clipboard Observer Testing

```rust
#[tokio::test]
async fn test_clipboard_content_detection() {
    let observer = ClipboardObserver::new();

    // Test with log-like content
    let log_content = "2024-03-15 14:32:10 ERROR: Connection to database at 10.0.1.50:5432 failed";
    observer.set_mock_clipboard(log_content);

    let result = observer.detect_content_type().await.unwrap();
    assert_eq!(result, ContentType::Log);
}

#[tokio::test]
async fn test_clipboard_redaction() {
    let observer = ClipboardObserver::new();

    let sensitive = "api_key=sk-proj-abc123xyz456\ntoken=eyJhbGciOiJIUzI1NiJ9.secret";
    observer.set_mock_clipboard(sensitive);

    let sanitized = observer.sanitize_content(&observer.get_text().await.unwrap()).await.unwrap();
    assert!(!sanitized.contains("sk-proj-"));
    assert!(!sanitized.contains("secret"));
    assert!(sanitized.contains("REDACTED"));
}
```

### 8.5 Observation Engine Privacy Testing

```rust
#[tokio::test]
async fn test_privacy_toggle_prevents_capture() {
    let observer = ScreenObserver::new(CaptureMode::ActiveWindow);

    observer.set_enabled(false);
    let capture = observer.capture().await.unwrap();

    // When disabled, should return None or empty capture
    assert!(capture.active_app.is_empty());
}

#[tokio::test]
async fn test_excluded_window_is_blurred() {
    let observer = ScreenObserver::new(CaptureMode::ActiveWindow);
    observer.set_exclusion_rules(vec![
        ExclusionRule {
            match_type: MatchType::WindowTitle,
            pattern: "*Password*".into(),
            action: ExclusionAction::Exclude,
        },
    ]);

    let capture = observer.capture_with_privacy().await.unwrap();
    assert!(capture.privacy_flags.contains(&PrivacyFlag::MaskedContent));
}
```

### 8.6 OCR Accuracy Benchmarks — Continuous Monitoring

OCR accuracy should be benchmarked periodically against a growing dataset. These benchmarks run as part of the nightly CI but are not hard gates (OCR accuracy varies by environment). Instead, they track regressions.

**Benchmarks to run nightly:**
| Test | Dataset Size | Metric | Alert Threshold |
|------|-------------|--------|----------------|
| OCR recall — English text | 50 screenshots | Recall @ 70% | < 75% |
| OCR recall — Terminal text | 30 screenshots | Recall @ 70% | < 70% |
| OCR recall — Web UI text | 50 screenshots | Recall @ 70% | < 75% |
| OCR recall — Dashboard numbers | 20 screenshots | Recall @ 60% | < 55% |
| Application detection accuracy | 100 windows | Top-1 accuracy | < 90% |

---

## 9. Security Testing

### 9.1 Encryption Verification

Verify that all sensitive data is encrypted at rest and in transit:

```rust
#[cfg(test)]
mod encryption_tests {
    use super::*;

    #[test]
    fn test_aes_256_gcm_encryption() {
        let plaintext = b"This contains a database password: mysecretpass123";
        let (key, nonce) = generate_key_and_nonce();

        let ciphertext = encrypt_aes256_gcm(plaintext, &key, &nonce).unwrap();
        let decrypted = decrypt_aes256_gcm(&ciphertext, &key, &nonce).unwrap();

        // Decryption must recover original
        assert_eq!(decrypted, plaintext);

        // Ciphertext must not contain plaintext
        let ct_str = String::from_utf8_lossy(&ciphertext);
        assert!(!ct_str.contains("mysecretpass123"));
        assert!(!ct_str.contains("password"));
    }

    #[test]
    fn test_argon2id_hashing() {
        let password = "user_password_123";
        let hashed = hash_password_argon2id(password, None);

        // Hash must not contain plaintext
        assert!(!hashed.contains(password));

        // Verification must work
        assert!(verify_password_argon2id(password, &hashed));

        // Wrong password must fail
        assert!(!verify_password_argon2id("wrong_password", &hashed));
    }

    #[test]
    fn test_encrypted_file_write_and_read() {
        let workspace = Workspace {
            name: "Test Customer".into(),
            // ... populated with sensitive metadata ...
        };

        let serialized = serde_json::to_vec(&workspace).unwrap();
        let encrypted = encrypt_file_data(&serialized).unwrap();

        let decrypted = decrypt_file_data(&encrypted).unwrap();
        let restored: Workspace = serde_json::from_slice(&decrypted).unwrap();

        assert_eq!(restored.name, workspace.name);
    }
}
```

### 9.2 Credential Manager Integration Testing

```rust
#[tokio::test]
async fn test_keychain_integration() {
    let credential_mgr = CredentialManager::new();

    // Store a test credential
    credential_mgr.store("api_key", "test-workspace", "sk-test-12345")
        .await
        .unwrap();

    // Retrieve it
    let retrieved = credential_mgr.retrieve("api_key", "test-workspace")
        .await
        .unwrap();

    assert_eq!(retrieved, "sk-test-12345");

    // Verify it's not stored in plaintext anywhere
    let plaintext_locations = find_plaintext_in_local_data("sk-test-12345");
    assert!(plaintext_locations.is_empty(),
        "Credential found in plaintext at: {:?}", plaintext_locations);

    // Clean up
    credential_mgr.delete("api_key", "test-workspace").await.unwrap();
}
```

### 9.3 Penetration Testing

Penetration testing covers the following attack vectors:

| Vector | Test | Tool / Method |
|--------|------|---------------|
| RPC injection | Attempt JSON-RPC injection via crafted RPC payloads | Manual + automated fuzzing |
| IPC privilege escalation | Attempt to access Tauri commands from untrusted WebView contexts | Tauri security policy audit |
| SQL injection | Attempt SQL injection via workspace names, search queries, skill inputs | `sqlmap`-equivalent via rusqlite |
| XSS | Attempt XSS via stored conversation content rendered in React | Playwright + OWASP XSS test cases |
| Credential extraction | Attempt to extract credentials from memory, file system, clipboard | Manual review + automated scanning |
| Screen data leakage | Attempt to verify screenshots are not persisted or transmitted | File system scan + network traffic analysis |
| MCP injection | Attempt to inject malicious MCP server payloads | Skill package validation testing |
| Update Tampering | Attempt to tamper with update server responses | MITM simulation |

**SQL Injection test:**
```rust
#[tokio::test]
async fn test_sql_injection_resistance() {
    let db = KnowledgeDatabase::new();

    let malicious_input = "'; DROP TABLE knowledge_docs; --";

    // This should be parameterized and NOT execute the injection
    let result = db.search_documents(malicious_input, "test-workspace").await;

    // Should return empty results, not error or execute the injection
    assert!(result.is_ok());
    let docs = result.unwrap();
    assert!(docs.is_empty());

    // Verify table still exists
    let count = db.count_documents("test-workspace").await.unwrap();
    assert!(count > 0);  // Table was NOT dropped
}
```

**XSS resistance test (frontend):**
```typescript
it('sanitizes stored conversation content against XSS', () => {
  const maliciousContent = '<script>alert("xss")</script>';
  const safeContent = sanitizeHtml(maliciousContent);

  render(<AiMessage content={safeContent} />);

  // Script tag should be stripped
  expect(screen.queryByText('alert("xss")')).not.toBeInTheDocument();
  // Raw content should not render as HTML
  expect(screen.getByText(maliciousContent)).not.toBeInTheDocument();
  // XSS event handlers should be stripped
  const div = screen.getByRole('article').firstChild as HTMLElement;
  expect(div.onclick).toBeNull();
  expect(div.getAttribute('onerror')).toBeNull();
});
```

### 9.4 Security Audit Testing

```rust
#[tokio::test]
async fn test_audit_log_immutability() {
    let audit_log = AuditLogger::new();

    // Write entries
    audit_log.write(AuditEntry {
        event: "workspace_created",
        workspace_id: "w1",
        details: "Created by user",
        timestamp: Instant::now(),
    });

    audit_log.write(AuditEntry {
        event: "skill_enabled",
        workspace_id: "w1",
        details: "openshift skill enabled",
        timestamp: Instant::now().add(Duration::from_secs(1)),
    });

    // Verify append-only: entries should be stored in order
    let entries = audit_log.list("w1").await.unwrap();
    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0].event, "workspace_created");
    assert_eq!(entries[1].event, "skill_enabled");

    // Verify no deletion is possible
    // (AuditLogger should have no delete method — verify via compilation)
}

#[tokio::test]
async fn test_sensitive_data_not_logged() {
    let tracer = TracingTestHarness::new();

    // Trigger a system event that includes sensitive data
    let workspace = Workspace {
        name: "Acme Corp".into(),
        // ... metadata containing sensitive fields ...
    };

    // Log workspace creation (should redact sensitive fields)
    log_workspace_created(&workspace);

    // Verify no sensitive data appears in log output
    let logs = tracer.capture_output();
    assert!(!logs.contains("sk-test-"));
    assert!(!logs.contains("password"));
    assert!(!logs.contains("SuperSecret"));
}
```

---

## 10. Performance Testing

### 10.1 Startup Time

Target: Cold start < 3 seconds on an 8-core, 16 GB RAM machine.

```rust
#[tokio::test]
async fn test_cold_start_time() {
    let start = Instant::now();

    // Simulate cold start: load core, initialize engines, load skills
    let app = TauriAppBuilder::new()
        .load_core()
        .init_observation_engine()
        .init_intent_engine()
        .load_skills()
        .init_knowledge_system()
        .build()
        .await
        .unwrap();

    let elapsed = start.elapsed();

    assert!(elapsed.as_secs_f64() < 3.0,
        "Cold start took {:.2}s, target is < 3s",
        elapsed.as_secs_f64());

    println!("Cold start: {:.2}s", elapsed.as_secs_f64());
}
```

**CI gate:** Cold start must not exceed 3.5 seconds (with 15% headroom).

### 10.2 Memory Usage

Target: < 300 MB during typical operation (per PRODUCT_SPEC.md).

```rust
#[tokio::test]
async fn test_memory_usage_typical_workload() {
    let mut app = setup_test_app().await;

    // Typical workload: 5 workspace switches, 10 AI requests, 20 observations
    for _ in 0..5 {
        app.switch_workspace("test-ws").await.unwrap();
    }

    for _ in 0..10 {
        app.send_chat_message("help me troubleshoot").await.unwrap();
    }

    for _ in 0..20 {
        app.trigger_observation().await.unwrap();
    }

    let memory = current_memory_usage_mb();
    assert!(memory < 300.0,
        "Memory usage {:.1} MB exceeds 300 MB target", memory);
}
```

**Memory regression test:** Run the same typical workload and verify memory hasn't regressed by > 10% from baseline.

### 10.3 CPU Usage

Target: < 5% idle, < 25% during active AI reasoning.

```rust
#[tokio::test]
async fn test_cpu_usage_idle() {
    let app = setup_test_app().await;

    // Wait 30 seconds with no active work
    tokio::time::sleep(Duration::from_secs(30)).await;

    let cpu = current_cpu_usage();
    assert!(cpu < 5.0,
        "Idle CPU usage {:.1}% exceeds 5% target", cpu);
}

#[tokio::test]
async fn test_cpu_usage_active_reasoning() {
    let app = setup_test_app().await;

    // Start AI reasoning with continuous observations
    app.set_observation_interval(Duration::from_secs(1));
    app.trigger_ai_reasoning_loop(Duration::from_secs(10)).await;

    let cpu = current_cpu_usage();
    assert!(cpu < 25.0,
        "Active CPU usage {:.1}% exceeds 25% target", cpu);
}
```

### 10.4 Inference Latency

Target: AI response streaming begins within 2 seconds (time to first token), total response within 15 seconds for typical queries.

```rust
#[tokio::test]
async fn test_inference_latency_p95() {
    let provider = TestAiProvider::new();
    let engine = ReasoningEngine::new(provider);

    let latencies: Vec<Duration> = (0..20).map(|i| {
        let start = Instant::now();
        rt.block_on(async {
            engine.reason(
                AnalysisContext {
                    observation: create_test_observation(i),
                    ..Default::default()
                }
            ).await.unwrap()
        });
        start.elapsed()
    }).collect();

    let p95 = percentile(&latencies, 0.95);

    assert!(p95.as_secs_f64() < 15.0,
        "p95 inference latency {:.2}s exceeds 15s target", p95.as_secs_f64());

    println!("p50: {:.2}s, p95: {:.2}s, p99: {:.2}s",
        percentile(&latencies, 0.50).as_secs_f64(),
        p95.as_secs_f64(),
        percentile(&latencies, 0.99).as_secs_f64());
}
```

### 10.5 Knowledge Search Latency

Target: < 500ms for typical knowledge queries.

```rust
#[tokio::test]
async fn test_knowledge_search_latency() {
    let system = KnowledgeSystem::with_indexed_data(10_000_docs());

    let queries = vec![
        "OpenShift pod CrashLoopBackOff",
        "VMware CPU ready time high",
        "Nagios host unreachable troubleshooting",
        "Ansible playbook syntax error",
    ];

    let mut total_latency = Duration::ZERO;

    for query in queries {
        let start = Instant::now();
        let results = system.search(query, "test-workspace", 5).await.unwrap();
        total_latency += start.elapsed();

        assert!(!results.is_empty(), "Query '{}' returned no results", query);
        assert!(results.iter().all(|r| r.relevance > 0.0));
    }

    let avg = total_latency / queries.len() as u32;
    assert!(avg.as_secs_f64() < 0.5,
        "Average search latency {:.2}s exceeds 500ms target", avg.as_secs_f64());
}
```

### 10.6 Long-Running Stability Test

Run the application under sustained load to verify no memory leaks or resource exhaustion:

```rust
#[tokio::test]
async fn test_sustained_operation_stability() {
    let app = setup_test_app().await;

    // Run 60 minutes of simulated operation
    let duration = Duration::from_secs(3600);
    let start = Instant::now();
    let mut iteration = 0u64;

    while start.elapsed() < duration {
        iteration += 1;

        // Simulate a typical user cycle
        app.trigger_observation().await.unwrap();
        app.update_intent().await.unwrap();
        app.send_chat_message("status").await.unwrap();
        app.process_suggestions().await.unwrap();

        // Every 100 iterations, check memory
        if iteration % 100 == 0 {
            let mem = current_memory_usage_mb();
            let start_mem = initial_memory_usage_mb();
            assert!(mem < start_mem * 1.2,
                "Memory grew by > 20% after {} iterations: {} MB",
                iteration, mem);
        }

        tokio::time::sleep(Duration::from_millis(500)).await;
    }

    println!("Stable operation for {} iterations", iteration);
}
```

---

## 11. Cross-Platform Testing

### 11.1 Platform Matrix

| Platform | Version | Architecture | Testing Focus |
|----------|---------|-------------|---------------|
| Windows | 10 (22H2) | x64 | Core functionality, MSI install, Credential Manager |
| Windows | 11 (23H2) | x64, ARM64 | Core functionality, MSI/EXE install, auto-updater |
| macOS | 13 Ventura | x64 | Core functionality, DMG install, Keychain |
| macOS | 14 Sonoma | x64 | Core functionality, DMG install, Keychain |
| macOS | 15 Sequoia | Apple Silicon (arm64) | Core functionality, DMG install, Keychain, native performance |

### 11.2 Platform-Specific Tests

```rust
// tests/platform/mod.rs

#[cfg(target_os = "windows")]
mod windows_tests {
    use super::*;

    #[tokio::test]
    async fn test_windows_credential_manager() {
        let mgr = CredentialManager::new();
        mgr.store("test_key", "test", "test_value").await.unwrap();
        let val = mgr.retrieve("test_key", "test").await.unwrap();
        assert_eq!(val, "test_value");
    }

    #[tokio::test]
    async fn test_windows_screen_capture() {
        let observer = ScreenObserver::new(CaptureMode::FullScreen);
        let capture = observer.capture().await.unwrap();
        assert!(capture.bounds.width > 0);
        assert!(capture.bounds.height > 0);
    }

    #[tokio::test]
    async fn test_windows_terminal_detection() {
        let terminal = TerminalObserver::new();
        let sessions = terminal.detect_sessions().await.unwrap();
        // May detect PowerShell, CMD, WSL bash, Windows Terminal
        for s in sessions {
            assert!(["pwsh", "powershell", "cmd", "bash", "sh"].contains(&s.shell.as_str()));
        }
    }
}

#[cfg(target_os = "macos")]
mod macos_tests {
    use super::*;

    #[tokio::test]
    async fn test_macos_keychain() {
        let mgr = CredentialManager::new();
        mgr.store("test_key", "test", "test_value").await.unwrap();
        let val = mgr.retrieve("test_key", "test").await.unwrap();
        assert_eq!(val, "test_value");
    }

    #[tokio::test]
    async fn test_macos_screen_capture() {
        let observer = ScreenObserver::new(CaptureMode::FullScreen);
        let capture = observer.capture().await.unwrap();
        assert!(capture.bounds.width > 0);
        assert!(capture.bounds.height > 0);
    }

    #[tokio::test]
    async fn test_macos_terminal_detection() {
        let terminal = TerminalObserver::new();
        let sessions = terminal.detect_sessions().await.unwrap();
        for s in sessions {
            assert!(["bash", "zsh", "fish", "sh", "pwsh"].contains(&s.shell.as_str()));
        }
    }
}
```

### 11.3 Cross-Platform E2E Tests

Playwright E2E tests run on both Windows and macOS in CI:

```yaml
# .github/workflows/e2e.yml (simplified)
jobs:
  e2e-windows:
    runs-on: windows-latest
    strategy:
      matrix:
        arch: [x64, arm64]
    steps:
      - run: cargo tauri build --target x86_64-pc-windows-msi
      - run: npx playwright test --project=chromium

  e2e-macos:
    runs-on: macos-latest
    strategy:
      matrix:
        arch: [x64, arm64]
    steps:
      - run: cargo tauri build --target aarch64-apple-darwin
      - run: npx playwright test --project=webkit
```

---

## 12. CI Testing Pipeline

### 12.1 Two-Lane CI Model

Adapted from OpenHuman's two-lane CI approach described in TECHNOLOGY_SELECTION.md.

**CI Lite** (fast, ~5 minutes) — runs on every push to `main` and every PR:
```yaml
# CI Lite steps:
- ESLint + Prettier
- Vitest for changed files only (`vitest related`)
- `cargo check` + unit tests for changed crates
- `cargo llvm-cov` with file filter (coverage gate: ≥ 80% on changed lines via `diff-cover`)
- `cargo clippy` (all crates)
- TypeScript type check (`tsc --noEmit`)
```

**CI Full** (thorough, ~30 minutes) — runs on PRs to `release`, weekly scheduled:
```yaml
# CI Full steps:
- Full Vitest suite
- Full `cargo test` suite + integration tests
- `cargo llvm-cov` full coverage report
- E2E Playwright tests on Windows and macOS
- Property-based tests (`proptest`)
- Performance benchmarks (regression gate)
- Production build + installer generation
- MCP skill tests (all skills)
- Security tests (encryption, credential, SQL injection, XSS)
- Cross-platform test matrix
```

### 12.2 CI Pipeline Diagram

```
PR Push
    │
    ├─► CI Lite (~5 min)
    │   ├─ ESLint + Prettier
    │   ├─ Vitest (changed files)
    │   ├─ cargo check + unit tests (changed crates)
    │   ├─ cargo llvm-cov diff gate (≥ 80%)
    │   ├─ cargo clippy
    │   └─ TypeScript type check
    │       │
    │       ├─ FAIL → PR blocked
    │       └─ PASS → Merge
    │
    │   After merge to main:
    │   └─► CI Full (~30 min)
    │       ├─ Full Vitest suite
    │       ├─ Full cargo test + integration
    │       ├─ Full cargo llvm-cov
    │       ├─ E2E Playwright (Windows + macOS)
    │       ├─ proptest property tests
    │       ├─ Performance benchmarks
    │       ├─ MCP skill tests
    │       ├─ Security tests
    │       ├─ Cross-platform matrix
    │       └─ Build MSI/DMG installers
    │
    Weekly scheduled: CI Full (even without PRs)
```

### 12.3 Pre-Commit Hooks

```bash
# .husky/pre-commit (via Husky + lint-staged)
# Frontend:
- ESLint on staged files
- Prettier on staged files
- Vitest related tests

# Rust:
- cargo fmt --check
- cargo clippy --all-targets --all-features -- -D warnings

# Git:
- Prevent commit of test fixtures containing real data
- Prevent commit of .env files with credentials
```

### 12.4 CI Gate Summary

| Gate | Threshold | Lane |
|------|-----------|------|
| ESLint errors | 0 | Lite + Full |
| Prettier violations | 0 | Lite + Full |
| Vitest (changed files) | 100% pass | Lite |
| Vitest (full suite) | 100% pass | Full |
| `cargo check` | No errors | Lite |
| `cargo test` (unit, changed) | 100% pass | Lite |
| `cargo test` (full) | 100% pass | Full |
| `cargo clippy` | No warnings/errors | Lite |
| Line coverage (changed) | ≥ 80% | Lite |
| Line coverage (full) | ≥ 70% | Full |
| E2E tests | 100% pass | Full |
| Property tests | 100% pass | Full |
| Benchmarks | No regression vs. baseline | Full |
| MCP skill tests | 100% pass | Full |
| Security tests | 100% pass | Full |
| Cross-platform matrix | 100% pass | Full |
| Installer build | Success | Full |

---

## 13. Test Data Management and Fixtures

### 13.1 Fixture Organization

```
tests/
├── fixtures/
│   ├── workspace/
│   │   ├── sample_workspaces.json    # Synthetic workspace data
│   │   └── sample_knowledge_docs/    # Synthetic knowledge documents
│   ├── screen/
│   │   ├── openshift_console.png     # Synthetic/mock screenshots
│   │   ├── vmware_dashboard.png
│   │   ├── terminal_output.png
│   │   ├── nagios_alert.png
│   │   └── exclusion_test.png        # Screenshot with sensitive windows
│   ├── ocr/
│   │   ├── ground_truth.yaml         # OCR ground truth for screenshots
│   │   └── test_cases/               # OCR test case definitions
│   ├── intent/
│   │   └── dataset.yaml              # Intent recognition test dataset
│   ├── prompts/
│   │   ├── troubleshooting-pod-crash-v3.yaml
│   │   ├── vmware-perf-troubleshoot.yaml
│   │   └── nagios-alert-response.yaml
│   ├── MCP/
│   │   ├── skill_responses/          # Expected MCP tool responses
│   │   └── protocol_tests/           # MCP protocol compliance tests
│   └── security/
│       ├── sample_credential_patterns.txt  # Credential regex patterns
│       ├── xss_payloads.txt              # XSS test payloads
│       └── sqli_payloads.txt             # SQL injection test payloads
├── mocks/
│   ├── ai_provider.rs              # Mock AI provider implementations
│   ├── mcp_server.rs               # Mock MCP server for testing
│   └── os_apis.rs                  # Mock OS APIs (screen capture, terminal)
└── helpers/
    ├── intent_builder.rs           # Builder pattern for intent test data
    ├── observation_builder.rs      # Builder pattern for observation data
    └── workspace_builder.rs        # Builder pattern for workspace data
```

### 13.2 Synthetic Data Generation

Use test data generators to create varied synthetic data without introducing real data:

```rust
use test_case::test_case;

#[test_case("OpenShift" => "openshift".to_string(); "openshift skill")]
#[test_case("VMware" => "vmware".to_string(); "vmware skill")]
#[test_case("Linux" => "linux".to_string(); "linux skill")]
fn test_workspace_name_normalization(skill_name: &str) -> String {
    normalize_workspace_name(skill_name)
}

// Property-based synthetic workspace generation
proptest! {
    #[test]
    fn test_workspace_creation_with_random_names(name in ".{1,100}") {
        let ws = WorkspaceBuilder::new()
            .with_name(&name)
            .build();

        assert!(!ws.id.is_empty());
        assert_eq!(ws.name, name);
        assert!(ws.created_at > Instant::now() - Duration::from_secs(60));
    }
}
```

### 13.3 Data Isolation Per Test

Each test must use isolated data to prevent cross-test contamination:

```rust
#[tokio::test]
async fn test_workspace_isolation() {
    let ws_a = WorkspaceBuilder::with_unique_id("workspace-a").build();
    let ws_b = WorkspaceBuilder::with_unique_id("workspace-b").build();

    // Import docs into ws_a
    let docs_a = vec![
        KnowledgeDoc::new("doc-1", "OpenShift guide", ws_a.id.clone()),
    ];
    knowledge_system.import(docs_a, &ws_a.id).await.unwrap();

    // Search in ws_b — should NOT see ws_a docs
    let results_b = knowledge_system.search("OpenShift", &ws_b.id, 10).await.unwrap();
    assert!(results_b.is_empty(),
        "Workspace B should not see Workspace A's knowledge documents");

    // Cross-workspace search is NOT supported by design
    let cross_results = knowledge_system.search("OpenShift", &ws_a.id, 10).await.unwrap();
    assert_eq!(cross_results.len(), 1);
}
```

### 13.4 Fixture Versioning

Test fixtures (especially screenshots and AI prompts) should be versioned to prevent drift:

```yaml
# tests/fixtures/versions.yaml
screenshots:
  openshift_console: v3 (2026-07-01)
  vmware_dashboard: v2 (2026-07-01)
  terminal_output: v1 (2026-06-15)

prompts:
  troubleshooting_pod_crash: v3 (2026-07-10)
  vmware_perf_troubleshoot: v2 (2026-07-05)
```

Update fixtures when:
- The UI changes and screenshots need refreshing
- Prompts are iterated and old test cases are invalid
- OCR ground truth needs correction

---

## 14. Coverage Targets and Quality Gates

### 14.1 Coverage Targets

| Metric | CI Lite | CI Full | Notes |
|--------|---------|---------|-------|
| Changed line coverage | ≥ 80% | ≥ 80% | Enforced via `diff-cover` |
| Full suite line coverage | — | ≥ 70% | Measured via `cargo-llvm-cov` |
| Full suite branch coverage | — | ≥ 60% | Measured via `cargo-llvm-cov` |
| Rust crate coverage | — | ≥ 65% | All core crates |
| MCP skill coverage | — | ≥ 80% | Per-skill |
| Frontend component coverage | — | ≥ 70% | Per-component file |

### 14.2 Quality Gate Enforcement

```bash
# diff-cover gate (CI Lite)
cargo llvm-cov --no-report | cargo-llvm-cov report --format json > coverage.json
diff-cover coverage.json --fail-under=80 --compare-branch=origin/main

# Full coverage report (CI Full)
cargo llvm-cov --lcov --output-path lcov.info
# Upload to coverage service for trending
```

### 14.3 Anti-Patterns to Avoid

| Anti-Pattern | Risk | Prevention |
|-------------|------|------------|
| Testing implementation details | Tests break on refactoring | Use RTL — test behavior, not internal state |
| E2E tests for everything | Slow feedback, flaky | Reserve E2E for critical user paths only |
| Integration tests with real AI provider | Slow, non-deterministic | Mock AI provider in integration tests |
| Screenshots with real data | Privacy violation | Use synthetic/mock screenshots only |
| Tests that depend on execution order | Flaky builds | Each test creates and cleans up its own state |
| Skipping coverage gates | Technical debt accumulates | CI blocks PRs below threshold |
| Testing with production-like data | Security risk | Synthetic data only |

### 14.4 Test Maintenance

| Responsibility | Frequency |
|---------------|-----------|
| Review failing flaky tests | Daily (during business hours) |
| Update screenshot fixtures after UI changes | Per-release |
| Update prompt test cases after prompt iteration | Per-prompt version |
| Update intent recognition dataset with new scenarios | Quarterly |
| Review security tests against new threat models | Quarterly |
| Update coverage baselines when codebase grows | Monthly |
| Audit test data for real-data leakage | Per-release |

---

## 15. Appendix: Test Matrix Quick Reference

### All-Tests Summary

| Category | Tool | Granularity | Gate |
|----------|------|-------------|------|
| Rust unit tests | `cargo test` | Function/method | 100% pass (CI Lite) |
| Rust property tests | `proptest` | Invariant | 100% pass (CI Full) |
| Rust integration tests | `cargo test` (integration) | Component interaction | 100% pass (CI Full) |
| Rust benchmarks | `criterion` | Performance | No regression (CI Full) |
| Rust linting | `cargo clippy` | Code quality | No warnings (CI Lite) |
| Rust coverage | `cargo-llvm-cov` | Coverage | ≥ 80% changed lines (CI Lite) |
| Frontend unit | Vitest + RTL | Component | 100% pass (CI Lite) |
| Frontend integration | Vitest + MSW | API/state | 100% pass (CI Full) |
| Frontend coverage | Vitest built-in | Coverage | ≥ 70% files (CI Full) |
| E2E | Playwright | User workflow | 100% pass (CI Full) |
| MCP skill unit | SDK test framework | Tool/resource/prompt | 100% pass (CI Full) |
| MCP protocol | SDK protocol tests | MCP compliance | 100% pass (CI Full) |
| AI prompt eval | Custom harness | Prompt quality | ≥ 80% quality (CI Full) |
| AI intent accuracy | Custom harness | Classification | ≥ 85% accuracy (CI Full) |
| AI embedding quality | Custom harness | Embedding similarity | Pass regression (CI Full) |
| OCR accuracy | Custom harness | Text extraction | ≥ 80% recall (CI Full, monitored) |
| Security tests | Custom + manual | Encryption, injection | 100% pass (CI Full) |
| Penetration tests | Manual + automated | Attack vectors | No findings (per-release) |
| Performance | Custom harness | Startup, memory, latency | Meet targets (CI Full) |
| Cross-platform | Platform-specific tests | Win10/11, macOS 13-15 | 100% pass (CI Full) |
| Installer | Manual + scripted | MSI/DMG validation | Install + launch succeeds (CI Full) |

### Quick-Start Checklist for New Contributors

1. **Set up:** `cargo build` + `pnpm install` + `cargo tauri dev`
2. **Run Rust tests:** `cargo test --lib` (unit) + `cargo test` (full)
3. **Run frontend tests:** `pnpm test` (Vitest)
4. **Run E2E tests:** `pnpm test:e2e` (Playwright, requires built app)
5. **Run coverage:** `cargo llvm-cov --lib` + `pnpm test -- --coverage`
6. **Run lints:** `cargo clippy --all-targets --all-features` + `pnpm lint`
7. **Run benchmarks:** `cargo bench`
8. **Pre-commit:** Husky hooks run automatically
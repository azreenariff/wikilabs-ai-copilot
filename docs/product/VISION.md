# Vision

## Product Vision

Wiki Labs AI Copilot is an enterprise-grade AI assistant for cloud engineers and SREs. It helps teams manage complex infrastructure — troubleshooting Kubernetes clusters, debugging deployments, reviewing configurations, and accessing curated knowledge — without context-switching across documentation, dashboards, and terminal sessions.

## Core Principles

1. **Context-aware** — The copilot understands the user's current workspace, technology stack, and recent activity to provide relevant assistance.
2. **Security-first** — All sensitive data (credentials, secrets) is handled through platform-native keychains. No credentials are stored in plain text or transmitted without encryption.
3. **Skill-driven** — External capabilities (CLI tools, APIs, scripts) are exposed as MCP-compatible skill modules, making it extensible without modifying core code.
4. **Knowledge-centric** — Vector-searched and full-text-searched knowledge base enables retrieval-augmented generation (RAG) for accurate, up-to-date answers.
5. **Transparent** — Every AI response is traceable. Users see the confidence score, source documents, and tool invocations that led to the recommendation.

## Target Users

- **Cloud Engineers** — Managing multi-cluster Kubernetes, OpenShift, and cloud infrastructure
- **SREs** — Troubleshooting production incidents, reviewing runbooks, automating responses
- **Platform Teams** — Building internal developer platforms, defining guardrails, managing skill catalogs
- **DevOps Practitioners** — CI/CD pipeline debugging, configuration management, observability

## Competitive Differentiators

- **Domain-specific skills** — Pre-built modules for OpenShift, Kubernetes, AWS, and cloud-native tooling
- **Tiered observation** — Sub-millisecond shell integration, fast app monitoring, and optional screen capture with OCR
- **Local-first** — All data stored locally with SQLite; only AI API calls go external
- **Hybrid search** — Vector search (384-dim MiniLM embeddings) + FTS5 keyword search for complementary recall
- **Intent-aware** — Recognizes whether the user is troubleshooting, configuring, or learning, and adapts its response style

## Success Metrics

- Mean time to resolve (MTTR) reduced by 30% for common cloud engineering tasks
- Zero credential exposure incidents
- 90%+ confidence in retrieved knowledge matches for user queries
- Sub-500ms response time for tier-1 (instant) observations
- Modular skill system supporting 100+ skill modules without performance degradation
# Product Specification — Wiki Labs AI Copilot

## Overview

Enterprise desktop AI assistant that assists engineers with customer implementation,
troubleshooting, and operational tasks.

## User Stories

### As an engineer, I want to:

1. **Ask questions about my customer's infrastructure**
   - "Why is pod web-01 in CrashLoopBackOff?"
   - AI responds with diagnosis + actionable steps + citations

2. **Get recommendations while working**
   - AI observes terminal output and window context
   - Provides relevant suggestions and commands

3. **Search my knowledge base**
   - Import SOPs, manuals, and documentation
   - Get answers with inline citations

4. **Manage multiple customer contexts**
   - Create workspaces per customer
   - Each workspace has its own technology stack and knowledge

5. **Invoke skill tools**
   - "List pods in namespace production"
   - "Show MySQL slow query log"
   - AI suggests commands; I confirm before execution

6. **Review my AI interaction history**
   - Every chat session is stored per workspace
   - Full context restoration when resuming

## Non-Goals (Phase 1)

- Autonomous execution of production commands
- Multi-user collaboration
- Cloud sync of workspace data
- Custom skill creation (end users)
- Mobile support
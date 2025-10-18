# Processes Index

## Purpose
This document indexes all process documentation for the Patinox project, including workflows, procedures, and operational practices.

## Classification
- **Domain:** Project Operations
- **Stability:** Semi-stable
- **Abstraction:** Structural
- **Confidence:** Established

## Process Categories

### Core Development Processes

#### [Creation Process](creation.md) - September 2025
**Status:** Established

Workflow for creating new components, features, and documentation within the Patinox framework.

#### [Validation Process](validation.md) - September 2025
**Status:** Established

Procedures for validating code, documentation, and design decisions to ensure quality and correctness.

#### [Delivery Process](delivery.md) - September 2025
**Status:** Established

Process for delivering completed work, including integration, deployment, and handoff procedures.

#### [Document Integration Process](document_integration.md) - September 2025
**Status:** Established

Process for integrating inbox documents into the context network, including review and archival.

### Quality & Review Processes

#### [Code Review Workflow](code-review-workflow.md) - September 2025
**Status:** Established

Workflow for conducting code reviews, including triage, review, and approval stages.

#### [Code Review Triage Process](code_review_triage_process.md) - September 2025
**Status:** Established

Detailed process for triaging code review recommendations and deciding what to apply immediately versus defer.

#### [Code Review Triage Standards](code_review_triage_standards.md) - September 2025
**Status:** Established

Standards and criteria for code review triage decisions, ensuring consistent quality assessment.

#### [Code Review Recommendation Triage](code-review-recommendation-triage.md) - September 2025
**Status:** Established

Framework for categorizing and prioritizing code review recommendations.

#### [Code Review Triage Applied](code_review_triage_applied_2025-09-16.md) - September 2025
**Status:** Reference Example

Example application of the triage process, demonstrating how recommendations were categorized and addressed.

### Development Methodologies

#### [TDD Success Patterns](tdd_success_patterns.md) - September 2025
**Status:** Established

Documented patterns for successful Test-Driven Development, learned from provider utilities implementation.

#### [Test Quality Assessment](test_quality_assessment.md) - August 2025
**Status:** Established

Framework and criteria for assessing the quality and effectiveness of test suites.

### Planning & Risk Management

#### [Task Planning and Prioritization](task-planning-and-prioritization.md) - September 2025
**Status:** Established

Process for planning tasks, estimating effort, and prioritizing work across the project.

#### [Risk Assessment Framework](risk-assessment-framework.md) - September 2025
**Status:** Established

Framework for identifying, evaluating, and mitigating risks in development and operations.

## Process Flow Overview

### Typical Development Cycle
1. **Planning** → Task Planning and Prioritization
2. **Creation** → Creation Process + TDD Success Patterns
3. **Quality** → Code Review Workflow + Test Quality Assessment
4. **Delivery** → Validation Process + Delivery Process

### Knowledge Management Cycle
1. **Capture** → Document Integration Process
2. **Organize** → Context Network Structure
3. **Validate** → Review processes
4. **Apply** → Reference in future work

## Navigation Guidance

### For New Contributors
1. Start with [Creation Process](creation.md) to understand development workflow
2. Review [Code Review Workflow](code-review-workflow.md) for quality standards
3. Check [TDD Success Patterns](tdd_success_patterns.md) for testing approach
4. Reference [Task Planning](task-planning-and-prioritization.md) for work prioritization

### For Process Improvement
1. Identify gap or pain point in current workflow
2. Check existing processes for related guidance
3. Propose improvement or new process
4. Document in appropriate file
5. Update this index with new/modified processes

### For Quality Assurance
1. Use [Test Quality Assessment](test_quality_assessment.md) for test review
2. Apply [Code Review Triage Standards](code_review_triage_standards.md) for consistency
3. Reference [Validation Process](validation.md) for acceptance criteria
4. Follow [Risk Assessment Framework](risk-assessment-framework.md) for critical changes

## Related Sections
- [Foundation Index](../foundation/index.md) - Principles guide processes
- [Decisions Index](../decisions/index.md) - Process decisions documented
- [Planning Index](../planning/roadmap.md) - Processes execute roadmap
- [Backlog](../backlog/by-status/ready.md) - Work processed via these workflows

## Parent Navigation
- **Parent:** [Context Network Discovery](../discovery.md)

## Process Evolution

### V2 Strategic Reset Impact
The V2 minimal-first approach (October 2025) emphasizes:
- **Pain-driven process adoption**: Add process formality only when needed
- **Lightweight workflows**: Prefer simple processes that don't block progress
- **Validate through usage**: Processes emerge from real pain points, not speculation

Most current processes were established during V1 development and remain applicable to V2 with lighter-weight application.

## Metadata
- **Created:** 2025-10-17
- **Last Updated:** 2025-10-17
- **Updated By:** Context Network Audit Remediation (Recommendation #1)

## Change History
- 2025-10-17: Created index file to fix broken navigation from discovery.md; cataloged all existing process documentation

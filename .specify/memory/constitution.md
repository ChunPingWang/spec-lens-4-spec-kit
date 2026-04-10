<!--
SYNC IMPACT REPORT
==================
Version change: 0.0.0 (template) → 1.0.0
Ratification type: Initial adoption (first ratified constitution)

Principles defined:
  I.   Code Quality (NEW)
  II.  Testing Standards (NEW)
  III. Test-Driven Development (NEW, NON-NEGOTIABLE)
  IV.  User Experience Consistency (NEW)
  V.   Performance Requirements (NEW)

Sections:
  Added: Core Principles (I–V)
  Added: Additional Constraints & Quality Standards
  Added: Development Workflow & Quality Gates
  Added: Governance

Templates reviewed:
  ✅ .specify/templates/plan-template.md — Constitution Check gate pulls from
     this file at plan time; no structural edit required.
  ✅ .specify/templates/spec-template.md — Success Criteria, Requirements, and
     User Scenarios sections already align with UX consistency and performance
     principles; no edit required.
  ✅ .specify/templates/tasks-template.md — Task categorization (setup, tests,
     implementation, polish/cross-cutting) supports TDD and quality gates;
     no edit required.
  ✅ .specify/templates/commands/*.md — Not present in this repository; no
     propagation needed.
  ⚠ README.md / docs/quickstart.md — Not present; no runtime guidance doc to
     update. Re-evaluate on first doc addition.

Deferred TODOs: none.
-->

# Spec Lens Constitution

## Core Principles

### I. Code Quality

All code MUST be readable, maintainable, and reviewed before merge.

- Every change MUST pass the project's configured linter and formatter with
  zero warnings before it is merged. Style is enforced by tools, not debate.
- Every change MUST be reviewed and approved by at least one engineer other
  than the author; self-approval is prohibited.
- Functions, modules, and public interfaces MUST have a single, clearly
  stated responsibility. Cyclomatic complexity per function SHOULD stay
  ≤ 10; any exception MUST be justified in the PR description.
- Dead code, commented-out code, and TODOs without an owning issue MUST NOT
  be merged.
- Public APIs and non-obvious logic MUST be documented inline; names MUST
  favor clarity over brevity.

**Rationale**: Code is read far more often than it is written. Enforcing
uniform, tool-checked quality gates removes whole classes of defects and
keeps the cost of future change low.

### II. Testing Standards

Tests are a first-class deliverable and MUST provide durable guarantees
about behavior.

- Every user-visible requirement MUST be covered by at least one automated
  test at the appropriate layer (unit, contract, or integration).
- Line coverage on changed code MUST be ≥ 80%; overall project coverage
  MUST NOT decrease on any PR.
- Tests MUST be deterministic. Flaky tests MUST be quarantined or removed
  within one working day of detection; they MUST NOT be silenced or retried
  into passing.
- The full test suite MUST run in CI on every PR and MUST be green before
  merge.
- Test code MUST meet the same readability and review bar as production
  code (Principle I applies).
- Bug fixes MUST include a regression test that fails before the fix and
  passes after.

**Rationale**: Untested code is unverified code. Mandatory, high-quality
tests turn specifications into executable guarantees and make refactoring
safe.

### III. Test-Driven Development (NON-NEGOTIABLE)

TDD is the default authoring workflow for all behavior changes.

- The Red-Green-Refactor cycle MUST be followed: (1) write a failing test
  that expresses the desired behavior, (2) write the minimum code to make
  it pass, (3) refactor with the test as the safety net.
- Tests MUST be committed in the same PR as (or before) the implementation
  they cover. Implementation-first PRs with tests added "later" MUST NOT be
  merged.
- For any new feature, a failing test MUST exist and be demonstrably red
  before corresponding implementation code is written.
- Deviations from TDD (e.g., exploratory spikes) MUST be explicitly called
  out in the PR description with justification; the resulting production
  code MUST still be delivered with tests written first in a follow-up.
- Contract and integration tests MUST be authored for: new library
  contracts, contract changes, inter-service communication, and shared
  schemas.

**Rationale**: Writing the test first forces clarity about intent, prevents
untestable designs, and guarantees every line of production code exists to
satisfy a specified behavior.

### IV. User Experience Consistency

Users MUST encounter a single, predictable product—not a collection of
features.

- All user-facing surfaces (CLI output, web UI, API responses, error
  messages) MUST follow a documented design system: shared vocabulary,
  shared layout primitives, shared error format, shared iconography where
  applicable.
- Error messages MUST be actionable: state what happened, why, and what the
  user can do next. Opaque codes alone are insufficient.
- User-visible text MUST be consistent in tone, terminology, and
  capitalization across the product. The same concept MUST use the same
  name everywhere.
- Accessibility baselines MUST be met: keyboard navigability, sufficient
  color contrast (WCAG 2.1 AA), and screen-reader labels on interactive
  elements.
- Any new user-facing flow MUST reuse existing patterns unless a documented
  UX decision justifies a new one.

**Rationale**: Inconsistency erodes trust and increases cognitive load.
Consistency is a feature that compounds across every interaction.

### V. Performance Requirements

Performance is a requirement, not an afterthought; it MUST be specified,
measured, and defended.

- Every feature plan MUST declare explicit performance budgets (latency,
  throughput, memory, payload size, startup time) appropriate to its
  domain. Absent a stated budget, the feature is not ready to implement.
- Default targets, unless overridden with justification, are:
  - Interactive user actions: p95 < 200 ms end-to-end.
  - Background jobs: documented throughput target with alerting on
    regression.
  - Memory: no unbounded growth; all caches MUST have eviction policies.
- Performance-critical paths MUST have benchmarks checked into the repo
  and executed in CI. Regressions > 10% on a tracked benchmark MUST block
  merge until acknowledged or fixed.
- Optimizations MUST be justified by measurement, not speculation.
  "Premature optimization" and "it feels faster" are both unacceptable;
  cite a profile or a benchmark.

**Rationale**: Performance problems compound and are expensive to fix
late. Making budgets explicit and measured keeps the product fast by
construction.

## Additional Constraints & Quality Standards

- **Security**: Never commit secrets. Dependencies MUST be scanned for
  known vulnerabilities in CI; high or critical findings MUST block
  release.
- **Dependencies**: Adding a runtime dependency requires PR justification
  (why not build it, why this library, license compatibility).
- **Observability**: User-facing services MUST emit structured logs and
  expose health/metrics endpoints sufficient to diagnose the principles
  above in production.
- **Versioning**: Public interfaces follow Semantic Versioning
  (MAJOR.MINOR.PATCH). Breaking changes require a MAJOR bump and a
  migration note.

## Development Workflow & Quality Gates

1. **Plan**: Every feature starts with a spec and plan that explicitly
   address all five core principles, including a declared performance
   budget and a testing strategy consistent with TDD.
2. **Implement**: Work proceeds via TDD (Principle III). Each commit MUST
   keep the tree green locally before push.
3. **Review**: PRs MUST include: passing CI (lint, tests, benchmarks,
   security scan), coverage report on changed code, and a checklist
   confirming compliance with Principles I–V. Reviewers MUST reject PRs
   that violate any principle without a documented, approved exception.
4. **Merge**: Only PRs with all gates green and at least one non-author
   approval may merge. Force-push to protected branches is prohibited.
5. **Post-merge**: Regressions detected in production (quality,
   performance, UX) MUST be tracked as issues and fixed with a regression
   test before the next release.

## Governance

- This constitution supersedes ad-hoc practices. Where any other document
  conflicts with it, this document wins until amended.
- **Amendments**: Any contributor MAY propose an amendment via PR that
  edits this file. The PR MUST include: (a) the proposed text, (b) the
  rationale, (c) a Sync Impact Report, and (d) the resulting version bump
  per the rules below. Amendments require approval from project
  maintainers.
- **Versioning policy** (semantic):
  - MAJOR: backward-incompatible governance changes, principle removal,
    or redefinition that invalidates prior compliance.
  - MINOR: new principle or materially expanded guidance.
  - PATCH: clarifications, wording, typos, non-semantic refinements.
- **Compliance review**: Maintainers MUST review open PRs for
  constitution compliance as part of normal code review. A quarterly
  audit MUST verify that CI gates (lint, tests, coverage, benchmarks,
  security scan) still enforce the principles as written.
- **Exceptions**: Temporary exceptions to any principle MUST be recorded
  in the PR description, scoped to a specific change, and accompanied by
  a tracking issue to remove the exception.

**Version**: 1.0.0 | **Ratified**: 2026-04-11 | **Last Amended**: 2026-04-11

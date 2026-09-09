# ADR 001 — Roadmap Authority and Completion Evidence

- **Status:** Accepted
- **Date:** 2026-08-18

## Context

Historical technical-debt and completion documents contain status claims that no longer match executable behavior. Treating those claims as implementation truth allowed regressions such as unregistered frontend commands, successful no-op animation writes, and incomplete project restoration.

## Decision

`docs/plans/PROJECT_REMEDIATION_AND_MODERNIZATION_PLAN.md` is the forward implementation roadmap until it is superseded by another explicit ADR/roadmap change. Historical plans are context, not completion evidence.

A task/phase is complete only when the current repository provides evidence appropriate to the claim. Evidence may be:

1. required CI checks on the exact commit;
2. deterministic unit/integration/end-to-end tests;
3. installed-application smoke evidence;
4. a recorded, reproducible local real-ROM verification step where copyrighted fixtures cannot be committed;
5. cryptographic/signing evidence for release/update claims.

No check may be weakened merely to turn a red gate green. A narrowly scoped exception requires a written rationale and must not hide a correctness or security defect.

## Consequences

- Product maturity is tracked explicitly instead of inferred from UI presence.
- Research-blocked functionality may ship read-only or remain hidden, but may not report successful mutation.
- Every newly exposed Tauri invoke must be registered or explicitly classified as gated experimental/research work.
- Release documentation is generated from current evidence, not historical completion prose.

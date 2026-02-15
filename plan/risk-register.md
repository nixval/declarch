# Risk Register

## R-01 Refactor Regression
- Impact: High
- Likelihood: Medium
- Mitigation: tambah test sebelum/selama refactor, commit kecil.

## R-02 Security Hardening Breaks Existing Config
- Impact: Medium-High
- Likelihood: Medium
- Mitigation: compatibility notes, deprecation warnings, rollout bertahap.

## R-03 Scope Creep
- Impact: Medium
- Likelihood: High
- Mitigation: lock exit criteria per phase, stop saat objective tercapai.

## R-04 Performance Change Without Measurement
- Impact: Medium
- Likelihood: Medium
- Mitigation: wajib benchmark before/after pada path panas.

## R-05 Release Instability
- Impact: High
- Likelihood: Medium
- Mitigation: phase-05 guardrails + freeze window + rollback drills.

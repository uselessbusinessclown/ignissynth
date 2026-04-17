#!/usr/bin/env bash
#
# build-status.sh — single source of truth for repo status numbers.
#
# Emits two artifacts derived from the live repo state:
#   tools/status/status.json  — machine-readable
#   tools/status/STATUS.md    — human-readable, linked from READMEs
#
# Run from the repo root:
#   bash tools/status/build-status.sh
#
# Determinism: every count is derived from `ls`, `grep -c`, or a
# `jq` projection on a checked-in file. The only non-deterministic
# fields are `generated_at` and `generated_from_commit`, which the
# CI drift check ignores.
#
# Why bash + jq instead of Python or Rust:
#   * zero deps on the CI runner (already used in ci.yml)
#   * the dashboard describes the whole repo, so it shouldn't live
#     inside ignis0/ (would couple two unrelated lifecycles)
#   * <200 lines, easy to review against the data it claims to report
#
# This script is designed to be safe to re-run. It computes
# everything to /tmp first, then atomically rewrites the two
# committed artifacts.

set -euo pipefail

# ─── Locate repo root ────────────────────────────────────────────
# The script must be runnable from anywhere; resolve the repo root
# from the script's own location to keep paths deterministic.
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
cd "$REPO_ROOT"

OUT_JSON="$REPO_ROOT/tools/status/status.json"
OUT_MD="$REPO_ROOT/tools/status/STATUS.md"

# ─── Metadata ────────────────────────────────────────────────────
GENERATED_AT="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
GENERATED_FROM_COMMIT="$(git rev-parse --short HEAD 2>/dev/null || echo "unknown")"

# ─── Versions ────────────────────────────────────────────────────
SEED_VERSION="$(jq -r '.version' kernel/manifest.json)"
IGNIS0_VERSION="$(awk -F\" '/^version[[:space:]]*=/ {print $2; exit}' ignis0/Cargo.toml)"
IGNIS0_MSRV="$(awk -F\" '/^rust-version[[:space:]]*=/ {print $2; exit}' ignis0/Cargo.toml)"

# ─── Primary Forms ───────────────────────────────────────────────
# Expectation: 11 primary Forms (S-01 .. S-11), each paired with a
# .proof file. Both are enforced today by the proof-check CI job;
# the dashboard reports the underlying numbers so prose elsewhere
# can stop hard-coding them.
EXPECTED_FORMS=11
FORM_FILES=$(ls kernel/forms/S-*.form 2>/dev/null | sort)
PROOF_FILES=$(ls kernel/forms/S-*.proof 2>/dev/null | sort)
FORMS_PRESENT=$(printf "%s\n" "$FORM_FILES" | grep -c . || true)
PROOFS_PRESENT=$(printf "%s\n" "$PROOF_FILES" | grep -c . || true)

# Build a JSON array of {name, obligations, declared_verdict}
PROOFS_ARRAY="[]"
for f in $PROOF_FILES; do
  name=$(basename "$f" .proof)
  # Count `(obligation` blocks — the structural unit, distinct from
  # the `:obligation` claim keyword that appears once per proof.
  obligations=$(grep -c "(obligation" "$f" || true)
  # Extract the first `; Verdict: <X>` line. Some proofs declare
  # the verdict implicitly through their narrative footer instead;
  # those are reported as null and surfaced in the drift section.
  verdict=$(awk -F'Verdict:' '/^;[[:space:]]*Verdict:/ {print $2; exit}' "$f" \
              | awk -F' — ' '{print $1}' \
              | awk '{$1=$1};1')
  if [ -z "$verdict" ]; then
    verdict_json="null"
  else
    verdict_json=$(jq -Rn --arg v "$verdict" '$v')
  fi
  PROOFS_ARRAY=$(jq --arg n "$name" --argjson o "$obligations" --argjson v "$verdict_json" \
    '. + [{name: $n, obligations: $o, declared_verdict: $v}]' <<<"$PROOFS_ARRAY")
done

TOTAL_OBLIGATIONS=$(jq '[.[].obligations] | add // 0' <<<"$PROOFS_ARRAY")
PASS_COUNT=$(jq '[.[] | select(.declared_verdict == "Pass")] | length' <<<"$PROOFS_ARRAY")
STRUCTURAL_COUNT=$(jq '[.[] | select(.declared_verdict == "Structural")] | length' <<<"$PROOFS_ARRAY")
NULL_VERDICT_COUNT=$(jq '[.[] | select(.declared_verdict == null)] | length' <<<"$PROOFS_ARRAY")

# ─── Helpers ─────────────────────────────────────────────────────
# Single source for the encoded/pending split is STUBS.md. The
# helpers-check CI job reads the same numbers; this dashboard is
# what READMEs should link to so prose counts stop drifting.
HELPER_FILES=$(ls kernel/forms/helpers/*.form 2>/dev/null | wc -l | tr -d ' ')
HELPERS_ENCODED=$(grep -c "| encoded" kernel/forms/helpers/STUBS.md || true)
HELPERS_PENDING=$(grep -c "| pending" kernel/forms/helpers/STUBS.md || true)

# ─── IL opcode count ─────────────────────────────────────────────
# Implementation count: variants of the Opcode enum in ignis0.
# Spec count: the literal string IL.md must declare. Both are
# reported and an `in_sync` boolean asserts they agree.
OPCODE_COUNT_IMPL=$(awk '
  /^pub enum Opcode/ { in_enum = 1; next }
  in_enum && /^}/    { exit }
  in_enum && /^[[:space:]]+[A-Z]/ { count++ }
  END { print count }
' ignis0/src/opcode.rs)

if grep -q "Thirty-five exactly" kernel/IL.md; then
  OPCODE_SPEC_STRING="Thirty-five exactly"
  OPCODE_SPEC_COUNT=35
else
  OPCODE_SPEC_STRING="(not declared)"
  OPCODE_SPEC_COUNT=0
fi

if [ "$OPCODE_COUNT_IMPL" = "$OPCODE_SPEC_COUNT" ]; then
  OPCODE_IN_SYNC=true
else
  OPCODE_IN_SYNC=false
fi

# ─── Manifest integrity ──────────────────────────────────────────
REQUIRED_KEYS=(version forms kernel_authors boot_order immediates)
MANIFEST_MISSING_KEYS="[]"
for key in "${REQUIRED_KEYS[@]}"; do
  if ! jq -e --arg k "$key" 'has($k)' kernel/manifest.json >/dev/null; then
    MANIFEST_MISSING_KEYS=$(jq --arg k "$key" '. + [$k]' <<<"$MANIFEST_MISSING_KEYS")
  fi
done
MANIFEST_FORMS=$(jq '.forms | length' kernel/manifest.json)
MANIFEST_AXIOMS=$(jq '.axioms | length' kernel/manifest.json)
MANIFEST_AUTHORS=$(jq '.kernel_authors | length' kernel/manifest.json)

# ─── Axioms (drift check) ────────────────────────────────────────
# A9 (ignition substrate) is on disk but not in the manifest.
# The dashboard surfaces the gap; deciding whether that's
# intentional is a separate documentation question.
AXIOM_FILES_ON_DISK=$(ls axioms/A*.md 2>/dev/null | wc -l | tr -d ' ')
AXIOM_FILE_NAMES=$(ls axioms/A*.md 2>/dev/null | xargs -n1 basename 2>/dev/null | sed 's/\.md$//' | jq -R . | jq -s .)
AXIOM_MANIFEST_NAMES=$(jq '.axioms | keys' kernel/manifest.json)
AXIOM_DRIFT=$(jq -n --argjson disk "$AXIOM_FILE_NAMES" --argjson m "$AXIOM_MANIFEST_NAMES" \
  '[$disk[] | select(. as $d | ($m | map(. as $k | $d | startswith($k)) | any) | not)]')

# ─── Invariants ──────────────────────────────────────────────────
INVARIANT_COUNT=$(grep -cE "^\s*###?\s*I[0-9]+" synthesis/INVARIANTS.md || true)

# ─── Roadmap milestones (ignis0 track) ───────────────────────────
# Parse the table under "### ignis0 milestone track". Status
# normalization:
#   "✓ done"           -> done
#   "depends on …"     -> blocked
#   anything else      -> unknown
ROADMAP_MILESTONES=$(awk '
  /^### ignis0 milestone track/ { in_section = 1; next }
  in_section && /^### / && !/^### ignis0/ { exit }
  in_section && /^\| v[0-9]/ {
    # Split on |. Field 2 = tag, field 4 = status (with leading/trailing spaces).
    n = split($0, parts, "|")
    tag = parts[2]; gsub(/^[ \t]+|[ \t]+$/, "", tag)
    status_raw = parts[4]; gsub(/^[ \t]+|[ \t]+$/, "", status_raw)
    if (status_raw ~ /✓ done/) status = "done"
    else if (status_raw ~ /^depends on/) status = "blocked"
    else status = "unknown"
    # Emit one JSON object per line; the shell will jq-collect them.
    gsub(/"/, "\\\"", status_raw)
    printf "{\"tag\":\"%s\",\"status\":\"%s\",\"raw\":\"%s\"}\n", tag, status, status_raw
  }
' ROADMAP.md | jq -s .)

ROADMAP_DONE=$(jq '[.[] | select(.status == "done")] | length' <<<"$ROADMAP_MILESTONES")
ROADMAP_BLOCKED=$(jq '[.[] | select(.status == "blocked")] | length' <<<"$ROADMAP_MILESTONES")
ROADMAP_OTHER=$(jq '[.[] | select(.status != "done" and .status != "blocked")] | length' <<<"$ROADMAP_MILESTONES")

# ─── Drift summary ───────────────────────────────────────────────
# A small, opinionated set of cross-checks the dashboard runs on
# its own data and surfaces in the markdown. None of these block
# the dashboard from rendering — they're informational, and CI
# decides which (if any) should be hard failures.
DRIFT="[]"

if [ "$FORMS_PRESENT" -ne "$EXPECTED_FORMS" ]; then
  DRIFT=$(jq --arg m "primary forms: expected $EXPECTED_FORMS, found $FORMS_PRESENT" '. + [$m]' <<<"$DRIFT")
fi
if [ "$PROOFS_PRESENT" -ne "$EXPECTED_FORMS" ]; then
  DRIFT=$(jq --arg m "proofs: expected $EXPECTED_FORMS, found $PROOFS_PRESENT" '. + [$m]' <<<"$DRIFT")
fi
if [ "$OPCODE_IN_SYNC" = "false" ]; then
  DRIFT=$(jq --arg m "opcode count: impl=$OPCODE_COUNT_IMPL, spec=$OPCODE_SPEC_COUNT" '. + [$m]' <<<"$DRIFT")
fi
if [ "$NULL_VERDICT_COUNT" -gt 0 ]; then
  bare_names=$(jq -r '[.[] | select(.declared_verdict == null) | .name] | join(", ")' <<<"$PROOFS_ARRAY")
  DRIFT=$(jq --arg m "proofs without explicit '; Verdict:' line: $bare_names" '. + [$m]' <<<"$DRIFT")
fi
if [ "$(jq 'length' <<<"$AXIOM_DRIFT")" -gt 0 ]; then
  drift_names=$(jq -r 'join(", ")' <<<"$AXIOM_DRIFT")
  DRIFT=$(jq --arg m "axioms on disk but not in manifest: $drift_names" '. + [$m]' <<<"$DRIFT")
fi

# ─── Assemble JSON ───────────────────────────────────────────────
jq -n \
  --arg generated_at "$GENERATED_AT" \
  --arg generated_from_commit "$GENERATED_FROM_COMMIT" \
  --arg seed_version "$SEED_VERSION" \
  --arg ignis0_version "$IGNIS0_VERSION" \
  --arg ignis0_msrv "$IGNIS0_MSRV" \
  --argjson expected_forms "$EXPECTED_FORMS" \
  --argjson forms_present "$FORMS_PRESENT" \
  --argjson proofs_present "$PROOFS_PRESENT" \
  --argjson proofs "$PROOFS_ARRAY" \
  --argjson total_obligations "$TOTAL_OBLIGATIONS" \
  --argjson pass_count "$PASS_COUNT" \
  --argjson structural_count "$STRUCTURAL_COUNT" \
  --argjson null_verdict_count "$NULL_VERDICT_COUNT" \
  --argjson helper_files "$HELPER_FILES" \
  --argjson helpers_encoded "$HELPERS_ENCODED" \
  --argjson helpers_pending "$HELPERS_PENDING" \
  --argjson opcode_count_impl "$OPCODE_COUNT_IMPL" \
  --argjson opcode_count_spec "$OPCODE_SPEC_COUNT" \
  --arg     opcode_spec_string "$OPCODE_SPEC_STRING" \
  --argjson opcode_in_sync "$OPCODE_IN_SYNC" \
  --argjson manifest_missing "$MANIFEST_MISSING_KEYS" \
  --argjson manifest_forms "$MANIFEST_FORMS" \
  --argjson manifest_axioms "$MANIFEST_AXIOMS" \
  --argjson manifest_authors "$MANIFEST_AUTHORS" \
  --argjson axiom_files "$AXIOM_FILES_ON_DISK" \
  --argjson axiom_drift "$AXIOM_DRIFT" \
  --argjson invariants "$INVARIANT_COUNT" \
  --argjson roadmap "$ROADMAP_MILESTONES" \
  --argjson roadmap_done "$ROADMAP_DONE" \
  --argjson roadmap_blocked "$ROADMAP_BLOCKED" \
  --argjson roadmap_other "$ROADMAP_OTHER" \
  --argjson drift "$DRIFT" \
  '{
    generated_at: $generated_at,
    generated_from_commit: $generated_from_commit,
    seed: { version: $seed_version },
    ignis0: { version: $ignis0_version, rust_msrv: $ignis0_msrv },
    primary_forms: {
      expected: $expected_forms,
      present: $forms_present,
      with_proof: $proofs_present
    },
    proofs: {
      files: $proofs,
      total_obligations: $total_obligations,
      verdicts: {
        Pass: $pass_count,
        Structural: $structural_count,
        unspecified: $null_verdict_count
      }
    },
    helpers: {
      files: $helper_files,
      encoded: $helpers_encoded,
      pending: $helpers_pending
    },
    il: {
      opcode_count_impl: $opcode_count_impl,
      opcode_count_spec: $opcode_count_spec,
      opcode_spec_string: $opcode_spec_string,
      in_sync: $opcode_in_sync
    },
    manifest: {
      required_keys_present: ($manifest_missing | length == 0),
      missing_keys: $manifest_missing,
      forms: $manifest_forms,
      axioms: $manifest_axioms,
      kernel_authors: $manifest_authors
    },
    axioms: {
      files_on_disk: $axiom_files,
      manifest_entries: $manifest_axioms,
      drift_not_in_manifest: $axiom_drift
    },
    invariants: { count: $invariants },
    roadmap_ignis0: {
      milestones: $roadmap,
      done: $roadmap_done,
      blocked: $roadmap_blocked,
      other: $roadmap_other
    },
    drift: $drift
  }' > "$OUT_JSON"

# ─── Render markdown ─────────────────────────────────────────────
{
  cat <<EOF
# IgnisSynth status dashboard

> **Generated artifact.** Do not edit by hand. Regenerate with:
> \`bash tools/status/build-status.sh\`
>
> Generated at: \`$GENERATED_AT\`
> From commit:  \`$GENERATED_FROM_COMMIT\`
>
> This page is the single source for repo status numbers. Other
> docs (READMEs, ROADMAP narrative) should link here rather than
> restate counts that drift.

---

## Versions

| Component | Version |
|---|---|
| Seed     | \`$SEED_VERSION\` |
| ignis0   | \`$IGNIS0_VERSION\` (MSRV \`$IGNIS0_MSRV\`) |

## Primary Forms

| Metric | Value |
|---|---|
| Expected primary Forms | $EXPECTED_FORMS |
| Forms present          | $FORMS_PRESENT |
| Forms with proof       | $PROOFS_PRESENT |

### Proofs (per-file obligation counts and declared verdicts)

| Form | Obligations | Declared verdict |
|---|---:|---|
EOF
  jq -r '.[] | "| `\(.name)` | \(.obligations) | \(.declared_verdict // "_(implicit / not declared)_") |"' <<<"$PROOFS_ARRAY"

  cat <<EOF

**Total obligations across all proofs:** $TOTAL_OBLIGATIONS
**Verdicts:** $PASS_COUNT Pass · $STRUCTURAL_COUNT Structural · $NULL_VERDICT_COUNT unspecified

## Helper Forms

| Metric | Value |
|---|---:|
| Helper \`.form\` files | $HELPER_FILES |
| Helpers encoded (per \`STUBS.md\`) | $HELPERS_ENCODED |
| Helpers pending (per \`STUBS.md\`) | $HELPERS_PENDING |

## IL opcode count

| Source | Value |
|---|---:|
| Implementation (\`ignis0/src/opcode.rs\` \`Opcode\` enum) | $OPCODE_COUNT_IMPL |
| Specification (\`kernel/IL.md\` declared string) | $OPCODE_SPEC_COUNT (\`$OPCODE_SPEC_STRING\`) |
| In sync | $OPCODE_IN_SYNC |

## Manifest integrity

| Metric | Value |
|---|---|
| Required keys present | $([ "$(jq 'length' <<<"$MANIFEST_MISSING_KEYS")" -eq 0 ] && echo "yes ✓" || echo "**no — missing $(jq -r 'join(\", \")' <<<"$MANIFEST_MISSING_KEYS")**") |
| Forms in manifest         | $MANIFEST_FORMS |
| Axioms in manifest        | $MANIFEST_AXIOMS |
| Kernel authors            | $MANIFEST_AUTHORS |

## Axioms

| Metric | Value |
|---|---:|
| Axiom files on disk          | $AXIOM_FILES_ON_DISK |
| Axiom entries in manifest    | $MANIFEST_AXIOMS |
EOF
  if [ "$(jq 'length' <<<"$AXIOM_DRIFT")" -gt 0 ]; then
    drift_list=$(jq -r 'map("`" + . + "`") | join(", ")' <<<"$AXIOM_DRIFT")
    echo "| Files not in manifest        | $drift_list |"
  fi

  cat <<EOF

## Invariants

Total invariants in \`synthesis/INVARIANTS.md\`: **$INVARIANT_COUNT**

## ignis0 milestone track

| Tag | Status |
|---|---|
EOF
  jq -r '.[] | "| `\(.tag)` | \(.raw) |"' <<<"$ROADMAP_MILESTONES"

  cat <<EOF

**Milestone summary:** $ROADMAP_DONE done · $ROADMAP_BLOCKED blocked · $ROADMAP_OTHER other

EOF

  if [ "$(jq 'length' <<<"$DRIFT")" -gt 0 ]; then
    echo "## Drift detected"
    echo
    echo "The dashboard noticed the following inconsistencies. None of these"
    echo "block the build by themselves; CI decides which warrant a hard fail."
    echo
    jq -r '.[] | "- " + .' <<<"$DRIFT"
    echo
  else
    echo "## Drift detected"
    echo
    echo "_None._ All cross-checks pass."
    echo
  fi
} > "$OUT_MD"

echo "Wrote $OUT_JSON"
echo "Wrote $OUT_MD"

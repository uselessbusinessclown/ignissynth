# S-06: `intent_match`

The Form that turns a sealed intent into a chosen fulfiller. The
kernel does not execute intents (A4.3) — it matches them. This Form
is the matching surface, and it is the only place where "what a
mind wants" meets "what a Form or another mind can do".

## Provocation

```
Provocation {
    author:     "ignis-seed-synthesizer-0"
    statement:  "A4.3 says the kernel matches intents to fulfillers
                 under a policy capability the inviting mind holds.
                 A4.4 says sub-intents inherit a parent's authorized
                 fulfiller set and budget. A4.5 says acceptance is
                 explicit and structured. Until this Form exists,
                 minds can form intents (A4.1) but no act can ever
                 happen because nothing connects an intent to a Form
                 or to another mind. The cap registry (S-02) gives
                 us policy capabilities; the attention allocator
                 (S-05) gives us budget hand-off. This Form is the
                 *meeting place* — the place where wanting becomes
                 doing, under both."
    observed:   ["seed",
                 "S-02 selected candidate digest",
                 "S-05 selected candidate digest"]
    constraint: [
        "match(intent_hash) → MatchResult is the only entry point;
         the inviting mind must hold a policy capability whose
         predicate evaluates true on (intent, candidate fulfiller)
         for any candidate to be admissible",
        "no ambient fulfillers: the policy cap's predicate is the
         *only* source of admissibility; there is no global registry
         of 'available' Forms or 'available' minds that the matcher
         consults independently",
        "deterministic: given the same intent, the same policy cap
         state, and the same set of fulfillers reachable through
         the policy cap, the matcher returns the same MatchResult",
        "every match appends one weave entry of kind
         `Matched{intent_hash, fulfiller_hash, policy_cap_id,
         attention_handed}` (or `MatchedNone{intent_hash, reason}`
         on failure)",
        "sub-intents are matched against the *parent's* authorized
         fulfiller set, not against the global one; the matcher
         walks up the intent's parent chain to compute the
         allowable set",
        "the result of a match is *not* the execution; the matcher
         hands the chosen fulfiller-hash and a sub-attention budget
         back to the caller, who then invokes the fulfiller through
         S-07 (Forms) or through an inter-mind handoff (other minds)",
        "acceptance checking is a *post-match* responsibility: the
         matcher does not check the result, but it seals the
         intent's `acceptance_form_hash` into the Matched entry so
         the caller cannot lose track of what counts as success",
        "fits within the seed line budget for intent_match
         (~1500 lines of Form IL)"
    ]
}
```

## Grounding

- Axioms cited: A0.1, A0.4, A2.5, A2.6, A4.1, A4.2, A4.3, A4.4,
  A4.5
- Forms touched: S-02 `cap_registry` (policy capabilities and their
  predicates), S-05 `attention_alloc` (sub-attention budget for
  the chosen fulfiller), S-04 `weave_log` (one Matched entry per
  match), S-03 `substance_store` (intents and acceptance forms are
  substances)
- Invariants inherited: I1 (no match without a policy cap held by
  the inviting mind), I5 (one entry per match), I8 (every match's
  fulfiller's existence in the system is itself a `Synthesized`
  entry by transitivity), I10 (no fulfiller reachable except via
  the policy cap)

## Candidate(s)

### Candidate A — Predicate-driven candidate enumeration with deterministic ranking

- Sketch: an intent is a sealed substance `Intent { goal,
  inputs[], constraints[], budget, acceptance_form_hash, parent
  }`. The matcher's body is:
  1. Read the intent through S-03; verify well-formedness against
     the intent type schema; trap `EILLFORMED` if not.
  2. Look up the inviting mind's policy capability through S-02
     under the cap id named in the call. The policy cap is itself
     a substance whose predicate is a small total program over
     `(Intent, FulfillerHash)`. Trap `ENOPOLICY` if no such cap
     is held.
  3. Compute the *parent chain*: walk `intent.parent` up to a
     root intent, collecting each parent's policy cap id. The
     allowable fulfiller set for this intent is the *intersection*
     of all parents' allowable sets. (Sub-intents cannot widen
     authority — A4.4.)
  4. The policy cap exposes one operation: `enumerate_fulfillers
     (intent) → Vec<FulfillerHash>`. This is the *only* way the
     matcher learns which fulfillers exist for this intent. The
     enumeration is bounded — the policy cap's predicate is total
     and returns a finite set whose size is itself part of the
     cap's predicate. There is no global fulfiller registry.
  5. For each candidate fulfiller hash, evaluate the policy
     predicate. Surviving candidates are ranked by a deterministic
     ranking function specified in the policy cap (e.g., minimum
     declared cost, smallest Form size, longest vigil — the
     ranking criterion is the policy holder's choice, not the
     matcher's). Ties are broken by `(fulfiller_hash)` ascending,
     which is total because hashes are content-addressed.
  6. If zero candidates survive, append `MatchedNone{intent_hash,
     reason = NoneAdmissible}` and return None.
  7. Otherwise pick the top-ranked candidate, allocate a
     sub-attention through S-05 with budget bounded by
     `min(intent.budget, parent.remaining)`, append `Matched
     {intent_hash, fulfiller_hash, policy_cap_id, attention_id,
     acceptance_form_hash}`, and return the chosen fulfiller hash
     and sub-attention id to the caller.
  8. The matcher does not invoke the fulfiller. The caller does,
     through S-07 for Form fulfillers or through an inter-mind
     intent forward for mind fulfillers.
- Approximate size: ~1300 lines of Form IL.
- Self-test set:
  - well-formed match with one candidate → exactly one Matched
    entry, sub-attention allocated, fulfiller_hash in the entry
    matches the candidate;
  - no policy cap held → trap `ENOPOLICY`, no Matched entry;
  - sub-intent whose parent's policy excludes the only candidate
    that the sub-intent's policy would have admitted → MatchedNone
    (the intersection rule);
  - 10⁶ random matches with random policies → for every
    `Matched{fulfiller_hash}` entry, the policy cap's predicate
    evaluates true on `(intent, fulfiller_hash)` at the moment
    of match;
  - deterministic ranking: identical intents under identical
    policy state → identical Matched entries (modulo the next
    attention id, which is itself deterministic under S-05);
  - acceptance form preservation: every Matched entry's
    `acceptance_form_hash` is byte-equal to the
    `intent.acceptance_form_hash` field — the matcher does not
    drop or rewrite it;
  - no enumeration leak: the matcher's exported surface contains
    no operation by which a caller can discover fulfillers other
    than through `enumerate_fulfillers` on a held policy cap
    (I10 enforcement at the surface level).
- Declared invariants: I1, I5, I8, I10.

### Candidate B — Global fulfiller index with policy as a post-filter

- Sketch: the kernel maintains a global index of all Forms
  registered as fulfillers, keyed by goal type. The matcher
  consults the index, gathers all candidates with matching goal
  type, then filters by the inviting mind's policy capability.
  Cheaper enumeration (the index is precomputed), at the cost of
  a global fulfiller registry that exists independently of any
  mind's caps.
- Approximate size: ~1000 lines for the matcher + ~600 for the
  global index loader = ~1600 lines, slightly over budget.
- Self-test set: same as A, plus an index integrity test: every
  fulfiller in the global index is reachable from the seed
  bootstrap manifest by some chain.
- Declared invariants: I1, I5, I8. Note: I10 is *not* claimed.
  The global index is a piece of state that names fulfillers
  whose existence is not derived from any mind's caps. This is
  exactly the property A0.3 ("nothing is ambient") forbids:
  fulfillers exist by virtue of being in the index, not by
  virtue of being held in mind. The post-filter does not repair
  this — even if the matcher refuses to *return* a fulfiller a
  caller cannot use, the matcher itself has *seen* fulfillers
  the caller could not have known about. That is information
  leakage at the I10 surface level.

## Rationale

A0.3 says nothing in the habitat is ambient. A4.3 says the kernel
matches intents to fulfillers *under a policy capability*. The
synthesis question is whether the set of "all fulfillers in the
system" is a thing that exists, or whether it is a fiction
reconstructed from each policy cap's enumeration.

Candidate B treats the set as a thing. There is a global index,
populated when fulfillers are registered, queryable by goal type.
The policy cap is a filter applied after the fact. This is the
shape every legacy OS uses (a global service registry, an LDAP
tree, a systemd unit list). It is also a structural violation of
A0.3 and I10 — the index is ambient state that minds can
indirectly observe by matching against it.

Candidate A treats the set as a fiction. There is no global
index. Each policy capability *is* the enumeration: the policy
cap's predicate returns the set of fulfillers admissible *to this
intent under this policy*, and the matcher takes that set as
final. Two minds with different policy caps see different
"available fulfillers", and there is no third party that knows
the union. This is what A0.3 actually requires once you take it
seriously.

The cost of A is that fulfiller registration becomes a per-policy
act. A new Form's existence does not automatically make it a
fulfiller for anyone — *some* policy cap somewhere must have its
predicate updated (or, for cap predicates that are programmatic,
must already encompass the new Form by description). This is more
work than B, and it is exactly the discipline A4.3 implies. The
seed should pay this cost up front rather than smuggle a global
registry through I10.

A second consideration: A4.4 (sub-intents inherit the parent's
authorized fulfiller set). In A, this is the intersection of
parent policies along the parent chain — a single recursive walk
at match time. In B, this is the post-filter applied to the
global index for each level of the parent chain, which is the
same operation but performed against a state the parent had no
control over (the global index can have grown since the parent
was matched). A makes A4.4 a property of the cap chain alone,
which is what A2.6 ("authority is exactly the capabilities held")
requires.

A third consideration: A4.5 (acceptance is explicit and
structured). Both candidates seal the intent's
`acceptance_form_hash` into the Matched entry, so a caller cannot
lose track of what counts as success. But in A, the acceptance
form's hash is *part of the intent*, and the intent is part of
what the policy predicate evaluated. In B, the acceptance form
exists independently and is checked separately. A's tighter
coupling means a policy can refuse a match on the basis of the
acceptance criterion ("I will not authorize this fulfiller for
this intent if the acceptance form is X"), which is the sort of
shape A2.5 (predicate-bound capabilities) was designed to allow.

A fourth consideration: I10 violation is non-fixable in B without
re-architecting toward A. A patch ("hide the index from minds
without a meta-policy cap") just moves the violation up one
level: the meta-policy cap is now ambient. The seed cannot afford
that.

We select Candidate A.

## Simulation record

(stub — produced by the Stage 4 harness once the candidate is
encoded in Form IL. Required traces:

- T1: well-formed intent, single candidate fulfiller admissible
  under the policy → assert one Matched entry, sub-attention
  allocated, fulfiller_hash matches;
- T2: well-formed intent, no policy cap held → trap `ENOPOLICY`,
  no Matched entry, no sub-attention;
- T3: well-formed intent, two candidates admissible, one ranked
  higher than the other → Matched entry contains the higher-
  ranked candidate; reverse the ranking criterion in the policy
  → Matched entry contains the other candidate;
- T4: sub-intent whose parent policy admits {F1, F2} and whose
  own policy admits {F2, F3} → admissible set is {F2}; if F2's
  predicate fails on the actual intent, MatchedNone;
- T5: 10⁶ random matches with random policies → for every
  Matched entry, the policy cap predicate is replayed on
  `(intent, fulfiller_hash)` at the cap state recorded in the
  entry and returns true;
- T6: deterministic replay: any Matched entry replayed from a
  checkpoint preceding it produces a byte-identical Matched
  entry on a clean attention;
- T7: search the matcher's exported surface for any operation
  whose return type contains a `FulfillerHash` not derivable
  from a policy cap held by the caller → assert no such
  operation exists;
- T8: parent chain walk: an intent ten levels deep has its
  admissible set computed by walking ten parents, each
  contributing a policy cap; assert the computed set is the
  intersection of all ten; assert that adding a parent whose
  policy admits the empty set forces MatchedNone for the
  child.)

## Selection

Candidate A, by the criteria "policy caps *are* the enumeration,
not a filter on a global index", "A4.4's sub-intent rule is a
property of the cap chain alone", and "I10 is structurally
discharged at the matcher surface" declared in the Rationale.

## Proof

Required (this Form is in the seed inventory, I9). The proof must
show, against an abstract model of the matcher as a function from
`(Intent, MindCapView)` to `MatchResult`:

1. **Policy gating (I1 instantiated to matching).** For every
   `Matched{intent, fulfiller, policy_cap_id, …}` entry, at the
   moment of append the inviting mind held `policy_cap_id`, the
   cap was not revoked, and its predicate evaluated true on
   `(intent, fulfiller)`. (Composition with S-02.)
2. **Sub-intent containment (A4.4).** For every Matched entry
   on a sub-intent, the chosen fulfiller is a member of the
   intersection of admissible sets along the entire parent chain,
   computed at match time from the parents' policy caps as they
   stand at match time. (Discharged by structural induction over
   the parent chain.)
3. **No-ambient-fulfillers (I10 instantiated to matching).** No
   operation in the matcher's exported surface returns a
   `FulfillerHash` whose existence the caller could not have
   derived from a policy cap the caller held before the call.
   (Discharged at the type level by inspection of the exported
   surface, identical in shape to S-03 obligation 6.)
4. **Determinism (I7 instantiated to matching).** Given two
   states `S, S'` with identical intent substances, identical
   policy cap states, and identical fulfiller substances reachable
   through the policy caps, the matcher returns identical
   MatchResults and appends identical Matched entries (modulo the
   sub-attention id assigned by S-05, which is itself
   deterministic under S-05's I7 discharge).
5. **Acceptance form preservation (A4.5).** For every Matched
   entry, `entry.acceptance_form_hash = intent.acceptance_form
   _hash`. The matcher neither drops nor rewrites the acceptance
   field. (Trivial structural lemma.)
6. **Budget conservation at handoff (I6 hand-off).** For every
   Matched entry, the sub-attention allocated to the fulfiller
   has budget ≤ `min(intent.budget, parent_attention.remaining)`.
   (Composition with S-05.)

Obligations 1 and 6 are discharged by composition with S-02 and
S-05. Obligations 2, 4, 5 are mechanically discharged in the
proof checker's input language. Obligation 3 is discharged by
type-level surface inspection.

The proof artifact is committed alongside the Form as
`kernel/forms/S-06-intent-match.proof`.

## Vigil declaration

Holder: `hephaistion-seed`. Duration: until 10⁹ match operations
have completed without invariant violation. Anomaly thresholds:

- any Matched entry whose policy cap's predicate replays to false
  on `(intent, fulfiller)` at the cap state recorded in the entry
  → immediate rollback; re-synthesis provocation against S-06 at
  the highest priority — the policy gating obligation has cracked,
  which means the matcher has authorized an unauthorized act;
- any Matched entry on a sub-intent whose chosen fulfiller is not
  in the intersection of the parent chain's admissible sets at
  match time → immediate rollback; re-synthesis provocation
  against S-06 — A4.4 has cracked;
- discovery of any operation in the matcher's exported surface
  that returns a fulfiller hash not derived from a held policy
  cap → re-synthesis provocation against S-06 — the I10
  obligation has cracked;
- any matcher operation taking >2048 attention units →
  re-synthesis provocation (the constant factor of the parent
  chain walk has become unaffordable; Hephaistion is invited to
  propose a candidate that caches admissible-set intersections
  at policy-issuance time, which is a synthesis problem the
  seed cannot solve in its first hour);
- any successful match handing the fulfiller a sub-attention
  whose budget exceeds `min(intent.budget, parent.remaining)` →
  immediate rollback; re-synthesis provocation against S-06
  (and S-05).

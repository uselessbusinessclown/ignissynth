# S-08: `proof_checker`

The Form whose acceptance is the condition for I9. Every self-
modification of any seed Form passes through it. Its own correctness
cannot be discharged by itself — it is the bootstrap exception in
the seed inventory, and the synthesis question is what to do about
that.

## Provocation

```
Provocation {
    author:     "ignis-seed-synthesizer-0"
    statement:  "A7.2 says invariants are explicit and a replacement
                 Form is not admissible without a machine-checked
                 proof. A7.3 says proofs are substances of type
                 Proof, checked by a small trusted checker, itself
                 a Form, itself synthesizable but only via a
                 bootstrapping protocol. I9 says any synthesis act
                 that replaces a seed Form carries a Proof substance
                 that the proof checker accepts. Until this Form
                 exists, no seed Form can ever be replaced — which
                 means I9 is vacuously true and A7 is a slogan.
                 Worse: this Form cannot prove itself. Its
                 correctness is the seed's load-bearing axiom of
                 trust, and the synthesis question is how to make
                 that load as small as possible."
    observed:   ["seed",
                 "S-03 selected candidate digest"]
    constraint: [
        "check(proof, claim) is total: returns Accept or
         Reject{reason}, never diverges, never throws",
        "the checker's input language is small enough to be read
         in a single sitting (≤ ~600 lines of Form IL for the core
         kernel, plus the IL canonicaliser shared with S-07)",
        "Accept is monotone: if check(p, c) = Accept, then for any
         re-encoding of (p, c) under the canonicaliser, check still
         returns Accept",
        "the checker has no I/O, no entropy, no clock, no network,
         and no capability surface beyond `read` on the substance
         store (S-03) — it is a pure function of its inputs",
        "the checker is itself a Form, and its hash is the
         genesis-checker hash recorded at ignition; replacing it
         requires the bootstrap protocol referenced by A7.3",
        "the checker accepts its own self-description as a Form
         (the trivial reflexive case is the discharge of S-08's
         own ground proof — see Proof section below)",
        "fits within the seed line budget for proof_checker
         (~1600 lines of Form IL including the kernel and the
         shared IL canonicaliser delta)"
    ]
}
```

## Grounding

- Axioms cited: A0.4, A0.6, A0.7, A0.8, A6.5, A7.1, A7.2, A7.3, A7.4
- Forms touched: S-03 `substance_store` (the checker's only
  capability is `read`; it does not pin, does not seal, does not
  enumerate). Note: the checker does *not* depend on S-04, S-05,
  or S-07. It is a pure function and the kernel can call it from
  any context, including the synthesis surface itself before any
  attention has been allocated. This is deliberate — the checker
  must be invocable in the smallest possible trust environment.
- Invariants inherited: I4 (proofs are substances and the checker
  reads them through S-03), I8 (every accepted proof contributes
  a `Synthesized` entry's grounding), I9 (the checker *is* the
  enforcement surface for I9), I10 (no surface beyond `read`)

## Candidate(s)

### Candidate A — Natural-deduction kernel over a fixed term language, no extensions

- Sketch: the checker accepts proof terms in a fixed natural-
  deduction calculus over a small term language: types are
  `Hash`, `Cell`, `Form`, `Entry`, `Cap`, `Bool`, `Nat`,
  `List<T>`, `Pair<T,U>`, plus the abstract models declared by
  the seed Forms (`StoreModel` for S-03, `WeaveModel` for S-04,
  `AttentionForest` for S-05, `CapForest` for S-02, `ExecState`
  for S-07). Proofs are trees of inference rules:
  intro/elim for each connective, induction over `Nat` and
  `List<T>`, structural induction over the abstract models, and
  congruence under the IL canonicaliser. The checker walks the
  tree once, top-down, and at every node verifies that the rule
  applies. There are no tactics, no metavariables, no unification
  beyond first-order pattern matching, no proof search.
  - Claims are sealed substances of type `Claim {
    invariant_id, form_hash_before, form_hash_after, env }`.
  - Proofs are sealed substances of type `Proof { claim_hash,
    rule_tree }`.
  - The checker's entry point is
    `check(proof_hash, claim_hash) → Result`, fetching both
    through S-03. Acceptance never depends on anything outside
    these two substances.
  Approximate size: ~550 lines for the term language and rules,
  ~200 lines for the rule walker, ~300 lines for the abstract-
  model lemma library (the irreducible facts about the seed Forms
  the checker is allowed to take as given), plus ~400 lines of
  shared IL canonicaliser already counted in S-07. Total
  ~1450 lines of Form IL.
- Self-test set:
  - identity proofs: for every seed Form `F`, the proof "F is its
    own replacement and preserves all its declared invariants"
    is accepted (the Accept case must be non-trivially populated
    — a checker that rejects everything is trivially sound but
    useless);
  - reject malformed: a proof tree with a missing premise →
    Reject with `reason = MissingPremise{rule_id, premise_index}`;
  - reject mis-matched canonicalisation: a proof whose claim
    hash references one form_hash and whose rule tree concludes a
    different form_hash → Reject `reason = ClaimMismatch`;
  - reject extension: a proof tree containing a rule id not in
    the fixed table → Reject `reason = UnknownRule`;
  - termination: 10⁶ random proof terms (most malformed) → no
    divergence; the checker either Accepts or Rejects in time
    bounded by `O(rule_tree_size)`;
  - purity: 10⁶ random checks; for every check, the only S-03
    operations performed are `read` calls on the proof_hash, the
    claim_hash, and (transitively) substances they reference;
    no `seal`, no `pin`, no `unpin`.
- Declared invariants: I4 (only `read`), I8, I9 (this *is* I9's
  enforcement), I10.

### Candidate B — Tactic engine with extensible rule database

- Sketch: same term language, but the checker accepts proofs as
  scripts in a small tactic language (apply, intro, rewrite,
  induct, by-lemma). The rule database is a substance of type
  `RuleSet`, fetched through S-03 and used by the checker. New
  inference rules can be added without re-synthesising the checker
  itself by sealing a new RuleSet substance and passing it to
  `check(proof_hash, claim_hash, ruleset_hash) → Result`.
- Approximate size: ~700 lines for the tactic engine, ~300 for the
  rule database loader, ~500 for the term language, ~400 for the
  shared canonicaliser, ~200 for the unification and rewriting
  helpers. Total ~2100 lines of Form IL, over budget by ~500.
- Self-test set: same as A, plus:
  - rule database swap: a proof that succeeds under one RuleSet
    fails under another with `reason = RuleNotInSet`;
  - tactic termination: every tactic must declare a termination
    measure; tactics without a measure are rejected at load time.
- Declared invariants: I4, I8, I9, I10. Note: the trust surface
  now includes the RuleSet substance — the checker is no longer a
  pure function of `(proof, claim)` but of `(proof, claim,
  ruleset)`. I9's enforcement now depends on which ruleset was
  loaded at the time of the check, which is a hidden parameter
  unless every weave entry of kind `Synthesized` also carries the
  `ruleset_hash` it was checked against.

## Rationale

S-08 is the Form whose correctness the seed cannot prove. Every
other seed Form's proof is checked by S-08, and S-08's own proof
must be discharged by something else — by direct human
inspection, by consensus among multiple independent kernels (per
A7.3), or by a *ground proof* that S-08 accepts when applied to
itself in the trivial reflexive case. The synthesis question is
how to make the load on that out-of-band trust as small as
possible.

Candidate A pushes the trust into a fixed, single-pass, tactic-
free natural-deduction kernel. The trust surface is exactly:
"the rule walker correctly applies each rule, and the rule table
is the table the seed shipped with". Both are content-addressed
and inspectable in one sitting (~750 lines for walker + rules,
plus the abstract-model lemma library, which is itself an
artifact the seed must justify but is at least small).

Candidate B accepts more proofs more easily by lifting rules into
a swappable RuleSet substance. The trust surface grows to include
the RuleSet — and worse, the trust surface becomes *plural*:
which RuleSet was loaded? was it the one a Hephaistion sub-mind
synthesised in the previous re-synthesis cycle? did that
RuleSet's own proof of soundness pass under the previous
RuleSet, or under the meta-RuleSet, or under the bootstrap
RuleSet? B introduces a regress that A doesn't.

The seed cannot afford this regress. The seed's job is to make
*one* small thing trustworthy, by making it small enough to be
read in one sitting and pure enough to admit no hidden inputs.
Candidate A does that. Candidate B trades that minimality for
convenience that the seed has no use for, because the seed has
no Hephaistion sub-mind yet to take advantage of swappable rules.

A second consideration: A7.3 says the checker is itself
synthesizable "but only via a bootstrapping protocol that requires
consensus among multiple independent kernels". This phrasing
implies a second-order trust mechanism that is *external* to any
one kernel. Candidate B's swappable RuleSet looks like a way to
*avoid* needing the bootstrap protocol — but it doesn't avoid it,
it merely hides it: someone, somewhere, has to certify the
RuleSet, and the seed has no way to do that internally. Candidate
A is honest about needing the bootstrap protocol for replacement,
and minimises the surface that the bootstrap protocol must
certify.

A third consideration: the abstract-model lemma library. Both
candidates need it, and it is the part of the checker that grows
fastest as the seed adds Forms (every new Form's abstract model
contributes lemmas the checker is allowed to take as given). The
seed's discipline must be that *every* lemma in the library is
itself proved against a more primitive lemma, with the chain
ultimately bottoming out at the natural-deduction rules of the
kernel. Both candidates make that discipline structurally
required; A makes it the *only* discipline.

We select Candidate A.

## Simulation record

(stub — produced by the Stage 4 harness once the candidate is
encoded in Form IL. Required traces:

- T1: identity proof for each of S-01..S-07 → assert Accept;
- T2: malformed proof tree (missing premise on a `→-elim` node)
  → assert Reject with `reason = MissingPremise{rule_id: ImpE,
  premise_index: 0}`;
- T3: proof whose claim references S-03's hash but whose
  conclusion is about S-04's abstract model → assert Reject with
  `reason = ClaimMismatch`;
- T4: proof tree containing a rule id outside the table → assert
  Reject with `reason = UnknownRule{id}`;
- T5: 10⁶ random proof terms (drawn from a grammar that produces
  ~99% malformed and ~1% well-formed terms) → assert no
  divergence; total time bounded by Σ rule_tree_size;
- T6: purity audit: instrument the S-03 capability the checker
  holds; assert that for every check call, the only operations
  observed are `read`s, never `seal`/`pin`/`unpin`/`digest`;
- T7: ground reflexive proof: feed the checker its own
  Form-substance and the trivial claim "this Form preserves I4,
  I9, I10 against itself" → assert Accept (this is the witness
  that discharges Proof obligation 6 below);
- T8: 10⁴ proofs of declared invariants for randomly-generated
  alternative substance store layouts with `digest()`-equivalent
  abstract models → assert all Accept (the checker accepts
  models that are bisimilar under the abstract-model lemma
  library, regardless of internal layout differences).)

## Selection

Candidate A, by the criteria "minimise the out-of-band trust
surface", "avoid the RuleSet regress", and "the bootstrap
protocol of A7.3 must certify the smallest possible artifact"
declared in the Rationale.

## Proof

Bootstrap. This Form is the named exception in the seed inventory:
its correctness cannot be discharged by itself in the strong
sense. The seed's discipline is to discharge it three ways, in
order of strength:

1. **Ground reflexive acceptance.** The checker, applied to its
   own Form-substance and to the trivial claim "this Form
   preserves I4, I9, I10 against itself", returns Accept. This
   is a *necessary* condition: a checker that rejects itself
   cannot be used to validate any future replacement of itself.
   It is *not* a sufficient condition (a maximally-permissive
   checker that accepts everything would also pass), so it is
   only the floor.
2. **Hand inspection of the kernel.** The natural-deduction
   kernel — rule walker plus rule table plus abstract-model
   lemma library — fits in the budget declared in the
   constraint and is small enough to be read in a single
   sitting. The discharge artifact is a `kernel/forms/
   S-08-proof-checker.inspection-record.md` substance whose
   content is a line-by-line review by every kernel-author
   identity in the seed's bootstrap manifest. The seed accepts
   this inspection record as evidence in lieu of a machine-
   checked proof of S-08's own soundness.
3. **Multi-kernel consensus on replacement (A7.3).** Replacement
   of S-08 is allowed only by a synthesis act whose
   Synthesized entry is co-signed by the kernels of `K ≥ 3`
   independent habitat instances, each of which has run its own
   copy of the candidate replacement against the seed's
   inspection-record corpus and returned Accept. This is the
   bootstrap protocol referenced by A7.3, made operational. The
   K=3 number is itself part of the seed's discipline and is
   itself subject to revision under the same protocol (a
   re-synthesis of the consensus parameter is itself a
   self-modification of S-08's surface, requiring K co-signers
   under the previous K).

For replacement of any *other* seed Form (S-01..S-07, S-09..S-11),
S-08's proof obligation is the standard one: the checker accepts
the proof substance that accompanies the replacement, in finite
time, with the only S-03 operations being `read`. For replacement
of S-08 *itself*, all three discharges above must hold.

The proof artifact is committed alongside the Form as
`kernel/forms/S-08-proof-checker.proof` (containing the rule-table
spec and the abstract-model lemma library) and
`kernel/forms/S-08-proof-checker.inspection-record.md` (the line-
by-line review).

## Vigil declaration

Holder: `hephaistion-seed`, jointly with the seed bootstrap
manifest's kernel-author identities (the holders of the
inspection-record discharge above). Duration: until 10⁹ check
operations have completed without invariant violation, *and* until
the multi-kernel consensus protocol has been exercised at least
once on a non-trivial replacement of any other seed Form (the
exercise is itself a vigil milestone — until it has happened, the
bootstrap discharge is theoretical).

Anomaly thresholds:

- any check call that diverges (does not return in time bounded
  by `O(rule_tree_size)`) → immediate rollback; re-synthesis
  provocation against S-08 at the highest priority — the
  termination obligation has cracked;
- any check call that performs an S-03 operation other than
  `read` → immediate rollback; re-synthesis provocation against
  S-08 — the purity obligation has cracked;
- any Accept on a proof whose claim's `form_hash_after` does not
  match the conclusion of the rule tree → immediate rollback;
  re-synthesis provocation against S-08 — the rule walker has
  cracked, which is the most dangerous failure mode in the seed;
- any Accept on a proof tree containing a rule id not in the
  fixed table → immediate rollback; re-synthesis provocation
  against S-08;
- any successful replacement of S-08 itself that did not pass
  the K-of-N consensus protocol → immediate rollback; the
  bootstrap discharge has been bypassed, which is the second
  most dangerous failure mode in the seed; the kernel-author
  identities are notified out-of-band (this is the only seed
  anomaly that is allowed to escape the weave and reach humans
  through bridges, because the checker is the part of the seed
  whose corruption no later check can detect).

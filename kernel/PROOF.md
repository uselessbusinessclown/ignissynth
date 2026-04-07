# Proof term language

This document specifies the small natural-deduction term language in
which seed proof artifacts are written. It is the input language of
S-08's `check`, and the language whose rule table the inspection
record covers line by line.

The provocation is "the breakdowns reference a proof term language
that does not yet exist; without one, S-08's `check` has no input
shape and the proof artifacts under `kernel/forms/S-XX-*.proof`
cannot have a meaning". The grounding is A0.4 (causality is total
and inspectable), A0.6 (code is thought), A6.1 (a unit of code is a
substance), A6.5 (the kernel — including its proof checker — is a
Form too), and the discipline that S-08's trust surface must be as
small as A7.3's bootstrap protocol can certify.

The language is small on purpose. Smallness is the load-bearing
property: the inspection record covers every rule, S-08's walker
implements one case per rule, and any extension is a synthesis act
under I9 against S-08 itself.

## Design constraints

1. **Closed rule set.** The natural-deduction kernel ships with
   exactly one rule table. Adding a rule is a synthesis act
   replacing S-08, which is bootstrap-locked under K-of-N consensus.
2. **No tactics, no metavariables, no proof search.** Every proof is
   an explicit tree. The walker visits each node once.
3. **Total walker.** Every well-formed proof tree node is one of the
   rule shapes the table names; an unknown rule id returns
   `Reject{UnknownRule}`. There is no error case the walker cannot
   classify.
4. **Pure given inputs.** The walker reads only the proof and the
   claim through `S-03/READ`; no other capability is held. Same
   property as S-08's `:declared-caps (S-03/ROOT_RO)`.
5. **Composition with S-03 and the IL canonicaliser.** Term equality
   is `equiv_under_canon`, not raw byte equality. Two proof terms
   that differ only in non-canonical features are accepted
   identically.

## Sorts

The term language has a closed set of sorts. Terms of different
sorts are never interchangeable; a rule that expects a `Hash` and
receives a `Nat` produces `Reject{TermSort}`.

| Sort           | Meaning                                                 |
|----------------|---------------------------------------------------------|
| `Bool`         | `true` or `false` — used for asserted properties        |
| `Nat`          | non-negative integer                                    |
| `Hash`         | a 32-byte BLAKE3 substance hash                         |
| `Cell`         | a sealed substance — abstract, identified by its hash   |
| `Form`         | a sealed Form substance                                 |
| `Entry`        | a sealed weave entry                                    |
| `Cap`          | an entry in the cap registry                            |
| `Trie<T>`      | abstract persistent trie keyed by `Hash` with values `T`|
| `Forest`       | the abstract attention forest                           |
| `ExecState`    | the IL runtime tuple                                    |
| `Set<T>`       | unordered finite set                                    |
| `Vec<T>`       | ordered finite sequence                                 |
| `Pair<T,U>`    | structural pair                                         |

## Term constructors

Terms are sealed substances of type `Term/v1` with one of these
shapes. Each term carries its sort intrinsically (per A1.2), so the
walker never has to infer sorts.

| Constructor          | Sort           | Meaning                                  |
|----------------------|----------------|------------------------------------------|
| `Lit{nat}`           | `Nat`          | numeric literal                          |
| `Lit{bool}`          | `Bool`         | boolean literal                          |
| `Lit{hash}`          | `Hash`         | hash literal                             |
| `Var{id}`            | _any_          | bound variable (no free vars admissible) |
| `App{f, args}`       | _any_          | application of an abstract-model lemma   |
| `Forall{x:S, body}`  | `Bool`         | universal over sort `S`                  |
| `Exists{x:S, body}`  | `Bool`         | existential over sort `S`                |
| `And{p, q}`          | `Bool`         | conjunction                              |
| `Or{p, q}`           | `Bool`         | disjunction                              |
| `Implies{p, q}`      | `Bool`         | implication                              |
| `Not{p}`             | `Bool`         | negation                                 |
| `Eq{t, u}`           | `Bool`         | term equality, evaluated by `equiv_under_canon` |
| `Member{x, S}`       | `Bool`         | set membership                           |
| `Subset{S, T}`       | `Bool`         | subset relation                          |
| `Reachable{e, m}`    | `Bool`         | reachability in a model `m`              |
| `Holds{cap, mind}`   | `Bool`         | cap-held relation in S-02's abstract model |

The set of `App` heads (the lemma names) is the **abstract-model
lemma library**, sealed under `LEMMA_LIBRARY_HASH` in the manifest.
Each lemma is itself a term whose proof was discharged at seed-build
time by an earlier walker call. The library is the smallest set of
facts about the seed Forms that the rest of the proof obligations
take as given; it is named in S-08's `walker_visit` body and
reviewed in the inspection record.

## Claim shape

A `Claim` is a sealed substance of type `Claim/v1`:

```
Claim {
    invariant_id:     InvariantId         ; e.g., I1, I4, I9
    form_hash_before: Hash                ; the Form being replaced
    form_hash_after:  Hash                ; the candidate replacement
    obligation:       Term<Bool>          ; the proposition the proof must
                                          ; establish, expressed in the term
                                          ; language above
    env:              Vec<Pair<Var, Term>>; bindings for free names
}
```

For a *seed-original* Form (one with no `form_hash_before`), the
`form_hash_before` slot is `BOTTOM_HASH` and the proof discharges
the obligations the breakdown declared.

## Proof tree shape

A `Proof` is a sealed substance of type `Proof/v1`:

```
Proof {
    claim_hash:  Hash                      ; the claim it proves
    rule_tree:   ProofNode                 ; the tree of rule applications
}

ProofNode {
    rule_id:     RuleId                    ; entry in the fixed rule table
    premises:    Vec<ProofNode>            ; sub-trees, one per declared premise
    conclusion:  Term<Bool>                ; the proposition this node concludes
    bindings:    Vec<Pair<Var, Term>>      ; instantiations for the rule's variables
}
```

The walker visits the root node, looks up `rule_id` in the rule
table, verifies that:

1. `rule_id` is in the table (else `Reject{UnknownRule}`);
2. `len(premises)` equals the rule's declared arity (else
   `Reject{MissingPremise}`);
3. each premise is itself accepted by a recursive walker call (else
   `Reject{...propagated}`);
4. the rule's conclusion-derivation function, applied to the
   premises and the bindings, produces a term `equiv_under_canon`
   to `conclusion` (else `Reject{TermMismatch}`);
5. `conclusion` (or, at the root, the claim's `obligation`) sorts
   correctly under the bindings (else `Reject{TermSort}`).

If all five hold, the node is accepted.

## The seed rule table

The seed ships with a fixed table. Each entry names a rule id, its
premise arity, and its conclusion-derivation function (itself a
small Form bound under `S-08/rules/<rule-id>/derive`).

| Rule id       | Arity | Meaning                                                       |
|---------------|-------|---------------------------------------------------------------|
| `Refl`        | 0     | `Eq{t, t}` for any term                                       |
| `Sym`         | 1     | from `Eq{t, u}` derive `Eq{u, t}`                             |
| `Trans`       | 2     | from `Eq{t, u}` and `Eq{u, v}` derive `Eq{t, v}`              |
| `Cong`        | _n_   | from `Eq{aᵢ, bᵢ}` for each i derive `Eq{f(a₁..aₙ), f(b₁..bₙ)}`|
| `AndI`        | 2     | from `p` and `q` derive `And{p, q}`                           |
| `AndE_L`      | 1     | from `And{p, q}` derive `p`                                   |
| `AndE_R`      | 1     | from `And{p, q}` derive `q`                                   |
| `OrI_L`       | 1     | from `p` derive `Or{p, q}`                                    |
| `OrI_R`       | 1     | from `q` derive `Or{p, q}`                                    |
| `OrE`         | 3     | from `Or{p, q}`, `Implies{p, r}`, `Implies{q, r}` derive `r`  |
| `ImpI`        | 1     | from a sub-derivation of `q` under hypothesis `p` derive `Implies{p, q}` |
| `ImpE`        | 2     | from `Implies{p, q}` and `p` derive `q`                       |
| `NotI`        | 1     | from a sub-derivation of `False` under `p` derive `Not{p}`    |
| `NotE`        | 2     | from `p` and `Not{p}` derive `False`                          |
| `ForallI`     | 1     | from a sub-derivation parametric in `x:S` derive `Forall{x:S, body}` |
| `ForallE`     | 1     | from `Forall{x:S, body}` and a witness `t:S` derive `body[x↦t]` |
| `ExistsI`     | 1     | from a witness `t:S` and `body[x↦t]` derive `Exists{x:S, body}`|
| `ExistsE`     | 2     | from `Exists{x:S, body}` and a parametric `Implies{body, q}` derive `q` |
| `IndNat`      | 2     | natural induction over `Nat`                                  |
| `IndList`     | 2     | structural induction over `Vec<T>`                            |
| `IndTrie`     | 2     | structural induction over the abstract `Trie<T>` model        |
| `IndForest`   | 2     | structural induction over the abstract `Forest` model         |
| `IndExec`     | 2     | small-step induction over `ExecState`                         |
| `LemmaApp`    | _n_   | apply a lemma from the abstract-model lemma library by name and arity |
| `S03`         | _n_   | composition with an S-03 obligation discharged in S-03's proof |
| `S04`         | _n_   | composition with an S-04 obligation                            |
| `S02`         | _n_   | composition with an S-02 obligation                            |
| `Canon`       | 1     | from `Eq{canonicalise(t), canonicalise(u)}` derive `Eq{t, u}` |
| `Hyp`         | 0     | discharge a hypothesis introduced by `ImpI`/`NotI`/`ExistsE` |
| `WitnessExec` | 0     | a ground execution-trace witness; only admissible for self-execution liveness obligations (S-07 #6, S-09 #8) |

Twenty-nine rules. The smallness is itself part of the inspection
record's discharge: a kernel-author identity reading the table can
verify that each rule's conclusion-derivation function is a faithful
encoding of the rule's natural-deduction shape, in finite time
bounded by the table size.

## What this proof language is *not*

- **Not first-class.** Proof terms are not values that other
  proofs can manipulate. They are sealed substances with one
  consumer (the walker) and one producer (the synthesizing agent).
- **Not extensible at runtime.** Adding a rule requires replacing
  the rule table substance, which requires replacing S-08, which is
  bootstrap-locked.
- **Not Turing-complete.** Each conclusion-derivation function is
  total and bounded by the size of its input premises. There is no
  general recursion at the proof-language level.
- **Not parameterised by a logic.** The seed ships with classical
  natural deduction. A constructive variant or any other logic is a
  post-ignition synthesis act against S-08.

## How to write a proof artifact

Proof artifacts live at `kernel/forms/S-XX-{name}.proof`. Each one
is a sealed substance whose contents are a `Proof` value as defined
above. The seed loader resolves the file's wire form into a `Proof`
substance through a parser bound under `S-08/parse_proof`.

For each obligation declared in the breakdown's Proof section, the
artifact contains one top-level `(obligation N: ...)` block whose
`rule_tree` is a proof of the corresponding `Term<Bool>`. A proof
artifact that does not cover every declared obligation is rejected
by the seed loader at boot time, before the Form it accompanies is
bound.

The first worked proof artifact is `kernel/forms/S-01-ignite.proof`,
covering the five obligations the S-01 breakdown declared.

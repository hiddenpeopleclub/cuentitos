# Probabilistic Buckets

### Submitters

- Claude Code with Fran Tufro

## Change Log

- [draft] 2026-09-08 - Initial draft for probabilistic buckets

## Referenced Use Case(s)

- [Percentage Bucket Picks the First Branch](../../compatibility-tests/probabilistic-buckets/feature/percentage-bucket-first-branch.md)
- [A Failed Req Drops Its Branch and Redistributes Its Share](../../compatibility-tests/probabilistic-buckets/feature/req-failure-renormalizes-bucket.md)
- [A Bucket Draws Again on Every Visit](../../compatibility-tests/probabilistic-buckets/feature/bucket-redraws-on-each-visit.md)
- [A Plain Sibling Prevents a Bucket](../../compatibility-tests/probabilistic-buckets/edge-cases/plain-sibling-prevents-a-bucket.md)
- [A Plain Option Prevents a Bucket](../../compatibility-tests/probabilistic-buckets/edge-cases/plain-option-prevents-a-bucket.md)
- [A Lone Probabilistic Option Is a Conditional Line](../../compatibility-tests/probabilistic-buckets/edge-cases/lone-probabilistic-option-is-conditional.md)
- [Bucket Nested Under a Hidden Conditional Line Draws Nothing](../../compatibility-tests/probabilistic-buckets/edge-cases/bucket-nested-under-hidden-conditional-line.md)

## Context

A probabilistic bucket is a set of sibling blocks where the engine picks
exactly one per visit, weighted by each branch's declared probability. It is
the mechanism authors reach for when a moment in the story should vary between
a fixed set of alternatives.

Line conditionality (ADR pending, sibling milestone) covers the other
probabilistic shape: a single line that appears or stays hidden on its own
roll. The two shapes share the same notation, so the language needs a rule
that says which one an author has written. Two questions had no answer in the
`palabritas.md` reference from version 0.2:

1. When exactly does a bucket form, given that probabilistic and plain blocks
   can share an indentation level?
2. What happens to a branch's share when a `req` on that branch fails and the
   branch drops out before the draw?

This ADR settles both. The compatibility tests listed above are the normative
statement of the rules; this document records the reasoning.

## Proposed Design

### Notation

A block becomes probabilistic by carrying a marker before its text:

```cuentitos
(50%) A stranger approaches and offers a wager.
(0.5) A stranger approaches and offers a wager.
```

Percentage notation takes integers. Probability notation takes a float in
`[0, 1]`. A bucket uses one notation throughout; mixing the two in one bucket
is an error.

### When a bucket forms

Look at every block at one indentation level under the same parent:

- If there are **at least two** of them and **every one** carries a
  probability, they form a bucket. Exactly one is drawn per visit. The
  percentages must sum to `100`, or the probabilities to `1.0`.
- Otherwise no bucket forms at that level, and each probabilistic block there
  is a conditional line, rolled independently of its siblings.

The rule is uniform across block kinds. Text lines and options follow it the
same way:

```cuentitos
I reach a fork in the road.
  (50%) I see a light to the left.
  (50%) I see a red light to the right.
  I can't see anything behind.
```

The plain third line means the level holds no bucket. Both `(50%)` lines are
conditional, so a run can show both of them, one of them, or neither. Removing
the plain line turns the same two lines into a bucket, at which point their
percentages have to sum to `100`.

`req`, `set`, `mod` and `freq` are modifiers that attach to a block. They are
not blocks in their own right and take no part in deciding whether a bucket
forms.

### Named buckets

A named bucket is a grouping node. The header carries a name and no text:

```cuentitos
[afternoon_weather]
  (50%) It rains all afternoon.
  (50%) The sun holds until dusk.
```

The name is snake case. Every content child of a named bucket carries a
probability; modifiers are exempt. A probability on the header itself,
`[(35%) fish_stall]`, makes the whole group one branch of its parent's bucket.

### A branch that drops out

A branch can carry a `req`. When that `req` fails, the branch leaves the
bucket before the draw happens, and its share is **split equally among the
surviving branches**:

```cuentitos
--- variables
bool market_open = false
---
I approach another stall.
  (40%) The stall is stacked with fresh bread.
  [(35%) fish_stall]
    req market_open
    (50%) The stall smells of fresh fish.
    (50%) The stall is piled high with ice.
  (25%) The stall sells only trinkets today.
```

`market_open` is false, so the `35%` group drops out. Two branches survive, so
each gains `17.5`:

| branch | declared | after the split |
|---|---|---|
| bread | `40%` | `57.5%` |
| trinkets | `25%` | `42.5%` |

The bucket always yields a branch, and the shares still add up to `100%`.

## Considerations

### Bucket formation: all-or-nothing over pairwise

The alternative was pairwise: any two probabilistic blocks at a level form a
bucket between them, and plain siblings sit outside it, always rendering. That
reading also reconciles the two sentences in `palabritas.md`, and it lets an
author mix a bucket and fixed prose at one indentation level without a
wrapper.

All-or-nothing won on legibility. Under the pairwise rule, adding a second
`(50%)` line somewhere in a long indented block silently converts an existing
conditional line into a bucket branch, and the author gets a sum error from a
line they never touched. The blocks that interact are also hard to see when
plain lines separate them. Under all-or-nothing, the unit is the whole
indentation level, which is the thing an author already reads as a group. An
author who wants a bucket next to fixed prose writes a named bucket, which
makes the grouping visible.

The `palabritas.md` sentence "if all the options in an indentation level are
probabilistic, then a bucket is created" says all-or-nothing outright for
options. Applying it to every block kind keeps one rule in the language.

### The minimum of two branches

A level holding a single probabilistic block could be read as a one-branch
bucket, which would then have to declare `100%`. Treating it as a conditional
line is what makes `(50%) A stranger approaches.` mean what an author expects
when it stands alone. The minimum of two is what separates the two shapes.

### The equal split over a proportional one

The alternative was proportional: redistribute a dropped branch's share in
proportion to what the survivors already declared, so `40:25` stays `40:25`
and becomes `61.54%` and `38.46%`.

Proportional preserves the declared ratio. Equal is what the language chose,
for two reasons. It is arithmetic an author can do in their head: divide the
missing share by the number of survivors and add. And it treats a `req` as a
statement about one branch rather than a statement that reweights the rest of
the bucket — a rare branch stays rare in absolute terms, and a common branch
does not absorb most of the gap simply for being common.

Both readings share the property that matters most: the bucket always yields
a branch, and no draw can land in a vacated slice.

### Leaving a dead slice was rejected

The third option was to leave the dropped branch's slice in place and have the
draw produce nothing when it lands there. That makes a `req` on one branch
change how often the bucket produces any output at all, which shows up as
silent gaps in the prose. It was rejected outright.

## Decision

- A bucket forms at an indentation level when there are at least two blocks at
  that level and every one of them carries a probability.
- Any probabilistic block outside a bucket is a conditional line, rolled
  independently.
- The rule is the same for text lines, named buckets and options.
- A branch whose `req` fails leaves the bucket before the draw, and its share
  is split equally among the surviving branches.

## Other Related ADRs

- [Options](000015-options.md) — the option mechanic these rules extend.

## References

- `palabritas.md` (version 0.2) — 'Probabilistic Buckets', 'Named Buckets' and
  'Options Bucket' sections.

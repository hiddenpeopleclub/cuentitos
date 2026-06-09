# Edge Case: Mixing String With Other Types

A `--- variables` block may declare string variables alongside int, bool,
and float variables. Declaration order is preserved for `?`, and each
type renders in its own format (strings double-quoted, others bare).

**Cross-milestone dependency.** This test deliberately spans three types
whose implementations land in different milestones. In roadmap order Bool
and Float precede String, so by the time the string implementation lands
those types already exist and this test can go green. Until then it stays
red on the missing `bool`/`float` keywords as well as `string` — it is the
one test in this suite that does not turn green on the string
implementation alone. It also pins the shared `?` render format for the
other types (`true` for bool, bare `1.5` for float, no trailing zeros);
the Bool and Float definition suites must adopt the same formats so the
suites agree.

## Script
```cuentitos
--- variables
int count = 7
bool ready = true
float ratio = 1.5
string name = "Aria"
---

This is the story.
```

## Input
```input
?
s
```

## Result
```result
START
count: 7
ready: true
ratio: 1.5
name: "Aria"
This is the story.
END
```

# Conditional Line With Indented Children: Shown

A conditional line can have its own indented children, same as any other
block. When the conditional line is shown, its children are shown too, in
order.

## Script
```cuentitos
You find a quiet bench.
(50%) A solitary figure sits on the bench, reading.
  The figure glances up and nods at you.
  You nod back.
This serene oasis calms you.
```

## Input
```input
seed 9754149498632845861
s
```

## Result
```result
START
You find a quiet bench.
A solitary figure sits on the bench, reading.
The figure glances up and nods at you.
You nod back.
This serene oasis calms you.
END
```

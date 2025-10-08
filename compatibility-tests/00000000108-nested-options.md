# Nested Options

Options within option content - second level of choices.

## Script
```cuentitos
First choice
  * Go to shop
    You enter the shop
    What do you buy?
      * Sword
        You bought a sword
      * Shield
        You bought a shield
  * Go home
    You go home
```

## Input
```input
1
2
s
```

## Result
```result
START
First choice
  1. Go to shop
  2. Go home
> Selected: Go to shop
You enter the shop
What do you buy?
  1. Sword
  2. Shield
> Selected: Shield
You bought a shield
END
```

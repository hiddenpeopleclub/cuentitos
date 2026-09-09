# Function Call And Tag Compose In The Same Script

Function calls and tags are independent notations that can appear side by
side in the same script: a tagged block and a backtick function call, one
after the other. Each fires on its own terms and prints its own marker
line, proving the two notations compose without interfering with each
other.

## Script
```cuentitos
You check your journal.
The hidden quest.
  tag dangerous
`play_sound alarm`
You close the journal.
```

## Input
```input
s
```

## Result
```result
START
You check your journal.
The hidden quest.
TAG dangerous
CALL play_sound alarm
You close the journal.
END
```

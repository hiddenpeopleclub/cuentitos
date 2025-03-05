# Skip Feature

### Submitters

- [Your Name]

## Change Log

- 2023-05-20 - [First Draft of ADR created](https://github.com/hiddenpeopleclub/cuentitos/pull/52)

## Referenced Use Case(s)

- Compatibility Test: [Two Lines and Skip](../../compatibility-tests/00000000003-two-lines-and-skip.md)

## Context

Cuentitos is a game narrative engine that provides precise control over story progression. While the "next" feature allows users to advance through the narrative one step at a time, there is also a need to quickly progress to the end of the current narrative sequence. This is particularly useful for:

1. Skipping through already-viewed content
2. Rapidly advancing to decision points or key narrative moments
3. Testing and debugging narrative flows without manually stepping through each line
4. Providing users with an option to view all available content in the current sequence at once

The "skip" feature serves as a mechanism to jump to the end of the current narrative sequence in one action, showing all intermediate content without requiring individual step commands.

## Proposed Design

The "skip" feature will be implemented as an extension of the existing step mechanism in the runtime, allowing users to advance to the end of the current narrative sequence with a single command.

When a user triggers a "skip" action (via the 's' command in CLI or through a UI interaction in other implementations), the runtime will:

1. Store the initial program counter position
2. Repeatedly call the `step()` method until reaching the end of the narrative sequence
3. Update the previous program counter to maintain a reference to the starting point
4. Return all blocks traversed during the skip operation

This design builds upon the existing "next" feature but provides a more efficient way to traverse the narrative when users want to see all content at once.

## Decision

We will implement the "skip" feature as a `skip()` method in the Runtime struct that rapidly advances the program counter to the end of the current narrative sequence. This approach ensures that:

1. Users can quickly navigate through narrative content when desired
2. All intermediate content is still processed and displayed
3. The implementation remains simple by leveraging the existing step functionality
4. The skip feature provides a complementary navigation mechanism to the step feature

## Implementation Details

The implementation of the "skip" feature will be housed in the `Runtime` struct within the runtime module. The core functionality includes:

1. A `skip()` method that:
   - Stores the initial program counter position
   - Repeatedly calls `step()` until `can_continue()` returns false
   - Updates the previous_program_counter to the initial position plus 1
   - Returns true to indicate successful completion

2. Integration with the CLI:
   - The CLI will interpret the 's' command as a request to call the `skip()` method
   - After skipping, the CLI will render all blocks traversed during the skip operation

3. Interaction with existing methods:
   - Leverages the existing `step()` method for each increment
   - Utilizes `can_continue()` to determine when to stop skipping
   - Works with `current_blocks()` to report all traversed blocks

This implementation ensures that the skip feature works harmoniously with the existing step feature while providing users with an alternative way to navigate through narrative content.

## Other Related ADRs

- [Next Feature](000006-next-feature.md) - Defines the step mechanism that the skip feature builds upon
- [Lines of Text](000005-lines-of-text.md) - Defines how text lines are parsed and stored

## References

- [Cuentitos Runtime Implementation](../../runtime/src/lib.rs)
- [CLI Input Processing](../../cli/src/main.rs) 
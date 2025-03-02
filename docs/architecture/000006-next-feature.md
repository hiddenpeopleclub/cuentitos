# Next Feature

### Submitters

- Fran Tufro

## Change Log

- 2025-01-22 - First Draft of ADR created

## Context

Cuentitos is a game narrative engine that needs to provide precise control over story progression. In a narrative-based game, users need a way to advance through the story at their own pace, one step at a time.

The "next" feature serves as the fundamental mechanism for progressing through a narrative in a controlled manner. This feature allows users to move forward through the narrative content one line at a time, which is essential for:

1. Allowing readers to consume content at their own pace
2. Supporting interactive narrative experiences where timing matters
3. Providing a clear and intuitive way to progress through the story

## Proposed Design

The "next" feature will be implemented as a simple step mechanism in the runtime that advances the program counter to the next narrative block.

When a user triggers a "next" action (via the 'n' command in CLI or through a UI interaction in other implementations), the runtime will:

1. Check if progression is possible (i.e., if the runtime is running and has not ended)
2. Advance the program counter to the next block
3. Update the previous program counter to maintain history
4. Return the new current block for rendering

This design is intentionally simple to ensure reliability and to serve as a foundation for more complex narrative progression features in the future.

## Decision

We will implement the "next" feature as a `step()` method in the Runtime struct that advances the program counter by one when the runtime is in a valid state to continue. This approach ensures that:

1. The narrative progresses in a controlled, predictable manner
2. The implementation remains simple and maintainable
3. The feature integrates well with both the CLI and potential future UI implementations
4. The feature serves as a building block for more complex narrative control mechanisms (like the `skip()` feature)

## Implementation Details

The implementation of the "next" feature will be housed in the `Runtime` struct within the runtime module. The core functionality includes:

1. A `step()` method that:
   - Checks if the runtime can continue (using `can_continue()`)
   - Increments the program counter
   - Updates the previous program counter
   - Returns a boolean indicating success

2. Supporting methods:
   - `can_continue()`: Verifies if the runtime is running and has not reached the end
   - `has_ended()`: Checks if the current block is an End block
   - `current_block()`: Returns the block at the current program counter
   - `current_blocks()`: Returns a vector of blocks between the previous and current program counter

3. Integration with the CLI:
   - The CLI will interpret the 'n' command as a request to call the `step()` method
   - After stepping, the CLI will render the new current block

This simple but robust implementation provides the core functionality needed for narrative progression while maintaining flexibility for future enhancements.

## Other Related ADRs

- 000005-lines-of-text.md - Defines how text lines are parsed and stored
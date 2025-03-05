# Compatibility Test Format

The compatibility tests are defined in a Markdown file inside the `compatibility-tests` directory.

Each file follows the following format:

````markdown
# Test Name

A description of the test.

## ADRs
  // A list of ADRs that relate to the test, to understand their context.
  - [ADR Name](https://adr-url)

## Script
```cuentitos 
// The script to run
```

## Input
```input
// A linebreak-separated list of inputs
```

## Result
```result
// The expected output in the command line
```
````

## ADRs

To help future developers undertsand what the tests is actually testing, besides the description, we link to all the ADRs that might be related to it.

There's no need to link to deprecated ADRs, unless there's something important there to help understand the test.

## Script

This will include a minimal `cuentitos` script that generates the expected behavior in the runtime.

This test will be compiled and run against the runtime.

Example:
````
```cuentitos
It's winter.
But it's not that cold.
I'm standing in front of my house, but I hear strange noises.
  * Open the door
    (50) I open the door and see an unfamiliar face
      * Ask who they are
        ME: Who are you?
        The person looks at you.
        UNFAMILIAR_FACE: The question is who are you?
        UNFAMILIAR_FACE: And what are you doing in MY house?
    (50) I open the door and see my mother looking at me
      * Ask her how is she doing
        ME: How are you?
        MOM: All good kiddo, been crocheting all day long
  * Go through the back door
    ...
```
````

## Input

Most scripts that do anything useful will probably need a series of input commands.

You can use all the input commands supported by the runtime in the CLI interface ( `->`, `<->`, `set`, `seed`, etc).

If you need to choose, write the choice text, or choice id

You can also use `n` and `s` for next block and skip.

Example:
````
```input
seed 1000
s
Open the door
n
n
0
```
````

In this case, se set the `seed`, we skip until the next choice, then choose the option that says "Open the door", ask for two next blocks, and then select the first choice (it doesn't matter which one it is).

## Result

For the result we want to put the exact output that we expect from the runtime when we run the script with the given input.

Example:
````
```result
BEGIN
It's winter.
But it's not that cold.
I'm standing in front of my house, but I hear strange noises.
(50/100) I open the door and see an unfamiliar face
ME: Who are you?
The person looks at you.
UNFAMILIAR_FACE: The question is who are you?
UNFAMILIAR_FACE: And what are you doing in MY house?
END
```
````

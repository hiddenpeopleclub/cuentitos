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

## Translations
```es
// Optional. A pre-existing translation file for locale `es`.
```

## Input
```input
// A linebreak-separated list of inputs
```

## Result
```result
// The expected output in the command line
```

## Expected Translations
```es
// Optional. The translation file the compiler must produce for `es`.
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

## Translations

Optional, and only meaningful for tests covering internationalization. Each
fenced block is one locale's translation file as it exists **before** the
test runs; the runner writes it to `locales/<code>.csv` next to the script.
The fence language is the locale code.

Columns are `id`, `line`, `original` and `translation`, matching what the
compiler reads and writes.

Example:
````
## Translations
```es
id,line,original,translation
3f21,1,"Hello.","Hola."
```
````

See [ADR 000017](architecture/000017-i18n-translation-tables.md) for how ids
are derived and how rows are matched.

## Expected Translations

Optional. Asserts the translation file the compiler must produce **after**
regeneration, in the same shape as `## Translations`. This is what makes
merge behaviour testable: carrying a translation over when a line moves,
carrying it over by line number when its text was edited, and retaining rows
whose text is gone as marked obsolete entries.

A test that only exercises regeneration needs no `## Input` and no
`## Result`.

Example:
````
## Expected Translations
```es
id,line,original,translation
7c1a,1,"Hello there.","Hola."
# obsolete
3f21,1,"Hello.","Hola."
```
````

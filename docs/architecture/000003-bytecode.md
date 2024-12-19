# Bytecode

### Submitters

- Fran Tufro

## Change Log

- 2024-12-19 - [First Draft of ADR created](https://github.com/hiddenpeopleclub/cuentitos/pull/52).

## Context

I want to create a simple binary format that I can compile to that is fast to
load and write to. This format will split the structure (logic) of the story
from the data (the text itself), so that the logic can load really fast, while
the text can be loaded asynchronously and change on the fly (localization for
example) without having any hiccups, this will also help with fast iterations
during development, since the data structures will mimic this representation.

## Proposed Design

I'm not really sure about all the details that should go in the design for the
bytecode, since I don't really know all the requirements for the runtime yet.

## Decision

I will defer the creation of the bytecode, because I think I can first create
the language and the parser can create the runtime structures directly in the
CLI for now, then I can build the compiler that will create the bytecode.

## Other Related ADRs


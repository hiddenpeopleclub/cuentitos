# Cuentitos
A game narrative engine with probabily at its core.

<iframe src="https://github.com/sponsors/hiddenpeopleclub/button" title="Sponsor hiddenpeopleclub" height="32" width="114" style="border: 0; border-radius: 6px;"></iframe>

## TODO

### Next
  - [ ] I18n

### Compiler
  - [x] Parse events
  - [ ] Error on missing events
  - [ ] Parse items
  - [ ] Error on missing items
  - [ ] Suggest similar event/item/tile names
  - [ ] I18n -> xls export/load

### Runtime
  - [x] Instances
  - [x] Configuration
  - [x] Database
  - [x] State Machine
    - [x] Current Event
    - [x] Current Choice
    - [x] Result
    - [x] Probability mods
  - [ ] State Save/Load
  - [ ] Add watch to `cuentitos-cli`
  - [ ] Interactive Shell
    - [ ] Set Game State -> from json
    - [ ] Choices
    - [ ] Modifiers
  - [x] Public Interface
    - [x] Get Random Event
    - [x] Set Choice
    - [ ] Get Result for Choice
    - [ ] CRUD Events
    - [ ] CRUD Resources
    - [ ] CRUD Items
  - [o] C++ interface
  - [ ] UE4 Plugin
  - [ ] Balancing Interface
  - [ ] C# Interface
  - [ ] Unity Bindings

### Editor
  - [ ] Show Modifiers
  - [ ] Game State Debugging
  - [ ] Writing (using Visual Studio Code & Sublime Text?)
  - [ ] Testing
  - [ ] Database Querying

### CI
  - [x] Add tests
  - [x] Add coverage
  - [x] Add automated builds for windows, mac and linux

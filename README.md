# Cuentitos
A game narrative engine with probabily at its core.

## TODO
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
  - [ ] Writing (using Visual Studio Code & Sublime Text?)
  - [ ] Testing
  - [ ] Database Querying

### CI
  - [x] Add tests
  - [x] Add coverage
  - [x] Add automated builds for windows, mac and linux

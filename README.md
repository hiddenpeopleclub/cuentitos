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
  - [-] State Machine
    - [x] Current Event
    - [ ] Current Choice
    - [ ] Result
    - [x] Probability mods
  - [x] State Save/Load
  - [ ] Interactive Shell
    - [ ] Set Game State -> from json
    - [ ] Choices
    - [ ] Modifiers
  - [ ] Public Interface
    - [ ] Get Random Event
    - [ ] Set Choice
    - [ ] Get Result for Choice
    - [ ] CRUD Events
    - [ ] CRUD Resources
    - [ ] CRUD Inventories
    - [ ] CRUD Items
  - [ ] C interface
  - [ ] UE4 Bindings
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

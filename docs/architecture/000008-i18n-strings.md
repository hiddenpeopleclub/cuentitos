# Internationalization (I18n) Support for Strings

### Submitters

- Fran Tufro

## Change Log

- [pending] 2024-03-05

## Context

The cuentitos game narrative engine needs to support internationalization (I18n) for strings to make stories accessible in multiple languages.

## Proposed Design

### Core Changes

1. String Storage Format
   - Introduce a new `TranslatableString` type that wraps string content
   - Store strings in a key-value format where the key is a unique identifier and the value is the translated content
   - Support language tags following BCP 47 standard (e.g., "en-US", "es-MX")

2. Script Syntax
   - No changes to the script syntax itself
   - Translations will be managed externally through CSV files
   - The compiler will generate one CSV file per language
   - Each CSV file will contain key-value pairs where:
     - Keys are integer identifiers for text blocks
     - Values are the translated content for that language
   
   Example CSV structure (en.csv):
   ```
   id,text
   0,"Welcome to the story"
   1,"Open the door"
   2,"Walk away"
   ```
   
   - The compiler will include all translations in the compiled output
   - The runtime will select the corresponding string table based on the current language
   - Language can be changed dynamically at runtime without reloading the story
   - Text blocks in the script will be replaced with their translated versions at runtime
   - This approach keeps the script clean while allowing for flexible translation management

3. Parser Changes

4. Runtime Changes
   - Add language selection API
   - Implement fallback chain (e.g., "es-MX" → "es" → default language)
   - Support runtime language switching
   - Handle missing translations gracefully

5. Compatibility Test Extensions
   - Extend compatibility test format to support language-specific testing
   - Add support for including translation CSV files with test cases
   - Example format:

   ```markdown
   # Test with translations

   ## Translations
   ```en
   0,"Hello world"
   1,"Make a choice"
   ```
   
   ```es
   0,"Hola mundo"
   1,"Haz una elección"
   ```

   ## Script
   ```cuentitos
   # hello
   Hello world.
     * Make a choice
   ```

   ## Input
   ```input
   lang es
   n
   0
   lang en
   n
   ```

   ## Result
   ```result
   BEGIN
   Hola mundo.
   Haz una elección
   Hello world.
    1 - Make a choice
   END
   ```
   ```

   - The runtime will load the specified CSV files during test execution
   - Tests can use the `set language <code>` command to switch languages
   - Expected results should match the translated output based on the current language
   - Tests can verify correct language switching behavior
   - Multiple language tests can be defined in a single test file
### Configuration

Add new configuration options in the story metadata:
```toml
[i18n]
default_language = "en"
required_languages = ["en", "es"]
fallback_chain = true
```

## Considerations

1. Storage Efficiency
   - Considered storing translations in separate files
   - Decided to keep them inline for simplicity and maintainability
   - Future optimization possible through compilation step

2. Translation Management
   - Could integrate with external translation management systems
   - Current design focuses on direct file editing
   - Future enhancement could add translation workflow support

3. Runtime Performance
   - Language switching should be efficient
   - Cache commonly used translations
   - Minimize string lookups during story execution

4. Compatibility Test Complexity
   - Balance between comprehensive language testing and test file readability
   - Need to ensure tests remain maintainable as language count grows

## Decision

1. Implementation Phases:
   - Phase 1: Basic string translation support with inline syntax
   - Phase 2: Advanced features (pluralization, interpolation)
   - Phase 3: Translation management tools and workflow

2. Requirements:
   - All story text must be translatable
   - Default language must always be present
   - Support for partial translations with fallback
   - Clear error messages for missing translations

3. Deferred Features:
   - Complex pluralization rules
   - Right-to-left language support
   - Translation memory/suggestions
   - External translation service integration

## Other Related ADRs

- [Lines of Text](000005-lines-of-text.md) - Text handling foundation that will be extended

## References

- [BCP 47 Language Tags](https://www.rfc-editor.org/info/bcp47)
- [Unicode CLDR](https://cldr.unicode.org/) - For language/locale data
- [Fluent Project](https://projectfluent.org/) - Inspiration for some syntax decisions 
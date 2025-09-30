# Sections and Sub-sections

This test verifies that sections can be defined using markdown-style headers with custom IDs,
and that sections can be nested to any depth. It also checks that the display names and
section paths are correctly formatted when navigating through the content.

## Script
```cuentitos
# section_1: First Section
  Some content in first section

## subsection_1: First Subsection
  Content in first subsection

### subsubsection_1: Deep Nested Section
  The deepest content

## subsection_2: Second Subsection
  More content here

# section_2: Second Section
  Final content
```

## Input
```input
s
```

## Result
```result
START
Entered Section: First Section (section_1)
Some content in first section
Entered Section: First Section > First Subsection (section_1/subsection_1)
Content in first subsection
Entered Section: First Section > First Subsection > Deep Nested Section (section_1/subsection_1/subsubsection_1)
The deepest content
Entered Section: First Section > Second Subsection (section_1/subsection_2)
More content here
Entered Section: Second Section (section_2)
Final content
END
```

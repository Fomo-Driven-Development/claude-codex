# Validation Documentation Standards

> Claude Code Memory for .strategic-claude-basic/validation directory
> Validation Documentation

## Document Naming Convention

**CRITICAL**: All validation documents in this directory MUST follow this exact pattern:

**When connected to a summary**:

```
VALIDATION_NNNN_DD-MM-YYYY_day_subject.md
```

**When no summary exists**:

```
VALIDATION_DD-MM-YYYY_day_subject.md
```

Where:

- **NNNN** = Sequential document number matching the associated summary (0001, 0002, etc.)
- **DD-MM-YYYY** = Date in day-month-year format
- **day** = Truncated day name (mon, tue, wed, thu, fri, sat, sun)
- **subject** = Brief hyphenated description matching the summary subject

### Summary Connection Rules

**Primary Rule**: Validations should ALWAYS be connected to a summary document when possible.

- Use the EXACT same NNNN number as the associated summary
- Use the EXACT same subject as the associated summary
- Match the date format and day name convention

**Example Chain**:

- Plan: `PLAN_0001_07-09-2025_sat_user-authentication-system.md`
- Summary: `SUMMARY_0001_07-09-2025_sat_user-authentication-system.md`
- Validation: `VALIDATION_0001_07-09-2025_sat_user-authentication-system.md`

**Only omit NNNN** when explicitly creating a validation with no associated summary document.

## Current Status

✅ All files follow the truncated day name convention.
✅ Summary-connected validations use matching document numbers.

## Existing Documents

1. **document-name** - **subject**

Next document number: **0001** (or match existing summary number)

## Instructions for Claude Code

When working in this directory:

1. **Check for associated summary first** - Look in `.strategic-claude-basic/summary/` for matching summary document
2. **Match summary naming exactly** - Use same NNNN, date format, and subject as the summary
3. **Always check file names** against the convention before creating new files
4. **Refuse to create** files that don't follow the naming pattern
5. **Suggest correct names** when violations are detected
6. **Use truncated day names** (mon, tue, wed, thu, fri, sat, sun)
7. **Get current date dynamically**: Run `date '+%d-%m-%Y-%a' | tr '[:upper:]' '[:lower:]'` to get properly formatted date (e.g., "07-09-2025-sat")
8. **Update this file**: After creating new documents, update the "Existing Documents" list

This naming convention ensures:

- Clear connection between validations and their associated summaries
- Traceability from plan → summary → validation
- Chronological ordering of validation documents
- Consistent documentation standards across the project
- Easy tracking of validation results back to implementation work
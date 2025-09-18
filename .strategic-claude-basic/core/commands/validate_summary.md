---
description: "Validate implementation by analyzing summary and related work"
argument-hint: <summary_file>
allowed-tools: Read(./**), Write(./.strategic-claude-basic/validation/**), Task, Bash(git:*, make:*, test:*), Glob
model: claude-opus-4-1
---

You are tasked with validating that implementation work was correctly executed by analyzing a summary document and its connected work through git history and comprehensive testing.

**Summary file provided:** $1

## Initial Response

When this command is invoked:

1. **Check if summary file was provided**:

   - If summary file path was provided, proceed with validation
   - If no summary provided, respond with:

   ```
   I'll validate implementation work by analyzing a summary document and its connected work.

   Please provide the summary file path to validate:
   Example: `/validate_summary @.strategic-claude-basic/summary/SUMMARY_0001_07-09-2025_sat_user-auth.md`

   I'll find the related plan, research, and issues, then validate the implementation.
   ```

   Then wait for the user's input.

## Process Steps

### Step 1: Document Chain Discovery

1. **Read the summary document completely**:

   - Extract the plan reference from frontmatter (`plan_reference` field)
   - Note the phase information if present
   - Identify all issues and incomplete work mentioned
   - Extract file references and implementation details

2. **Find all connected documents**:

   **Plan Document**:
   - Read the plan referenced in the summary
   - Extract all success criteria (automated and manual)
   - Identify all phases and their requirements

   **Research Documents**:
   - Search for research documents with matching subject/phase
   - Read any research referenced in the plan or summary

   **Issue Documents**:
   - Search for issues created from this summary
   - Look for issues with matching NNNN number
   - Read any issues mentioned in the summary

3. **Present document chain**:

   ```
   Found document chain for validation:
   - Plan: [plan filename] ([phases found])
   - Summary: [summary filename] ([completion rate])
   - Research: [research files found]
   - Issues: [issue files found]

   Proceeding with git history analysis and comprehensive validation...
   ```

### Step 2: Git History Analysis

1. **Find implementation timeline**:

   ```bash
   # Find when summary was first committed
   git log --follow --format="%H %cd" --date=iso -- [summary-file-path]
   
   # Find when plan was first committed (if exists)
   git log --follow --format="%H %cd" --date=iso -- [plan-file-path]
   
   # Get commits between plan creation and summary creation
   git log --oneline --since="[plan-date]" --until="[summary-date]"
   ```

2. **Analyze implementation commits**:

   - Look for commits that reference the plan or feature
   - Identify files changed during implementation period
   - Extract commit messages that indicate implementation work
   - Find test commits and documentation updates

3. **Present timeline analysis**:

   ```
   Implementation Timeline Analysis:
   - Plan created: [date]
   - Implementation period: [start] to [end]  
   - Key commits found: [N] commits
   - Files modified: [list of key files]
   - Tests added/modified: [test files]

   Proceeding with validation agents...
   ```

### Step 3: Comprehensive Validation

**CRITICAL**: Use Task agents for thorough validation. Do NOT try to do all validation yourself.

1. **Spawn parallel validation agents**:

   **Agent 1 - Implementation Verification**:
   ```
   Task: Verify implementation matches plan requirements
   - Read plan document completely  
   - Find all files mentioned in the plan
   - Check if implementation matches specifications
   - Verify success criteria are met
   - Return: Detailed implementation status vs plan requirements
   ```

   **Agent 2 - Code Quality Analysis**:
   ```
   Task: Analyze code quality and patterns  
   - Review files modified during implementation period
   - Check if code follows existing patterns
   - Identify any deviations or improvements
   - Look for potential issues or technical debt
   - Return: Code quality assessment with specific examples
   ```

   **Agent 3 - Test Coverage Verification**:
   ```
   Task: Verify test coverage and execution
   - Find all test files related to implementation
   - Check if tests cover the requirements
   - Identify missing test scenarios
   - Return: Test coverage analysis and gaps
   ```

2. **Run automated verification**:

   - Execute all commands from plan's "Automated Verification" section
   - Run comprehensive test suite
   - Check build status and linting
   - Document all results (pass/fail with details)

3. **Wait for all agents to complete** before proceeding

### Step 4: Validation Report Generation

1. **Gather metadata for validation document**:

   ```bash
   # Get current date/time with timezone
   date --iso-8601=seconds
   
   # Get current git commit hash
   git rev-parse HEAD
   
   # Get current branch name
   git branch --show-current
   
   # Get repository name
   basename -s .git $(git config --get remote.origin.url)
   
   # Get formatted date for filename
   date '+%d-%m-%Y-%a' | tr '[:upper:]' '[:lower:]'
   ```

2. **Generate validation report**:

   - Use template: `@.strategic-claude-basic/templates/commands/validation.template.md`
   - Replace ALL bracketed placeholders with actual findings
   - Follow naming convention: `VALIDATION_NNNN_DD-MM-YYYY_day_subject.md`
   - Use EXACT same NNNN, date, and subject as the summary file
   - Write document to: `.strategic-claude-basic/validation/[filename]`
   - Update the `@.strategic-claude-basic/validation/CLAUDE.md` file with new document entry

### Step 5: Present Validation Results

1. **Present validation summary**:

   ```
   Validation completed for: [summary filename]

   Implementation Status:
   ✓ [N] phases fully implemented  
   ⚠️ [N] phases partially implemented
   ✗ [N] phases not implemented

   Automated Tests:
   ✓ [N] passing / ✗ [N] failing

   Key Findings:
   - [Critical finding 1]
   - [Important deviation or issue]
   - [Recommendation or concern]

   Full validation report: `.strategic-claude-basic/validation/[filename]`

   Manual testing required: [Y/N with specific steps if needed]
   ```

## Important Guidelines

1. **Be Comprehensive**:

   - Read ALL connected documents completely
   - Use Task agents for parallel validation
   - Don't skip automated verification commands
   - Look beyond just the summary - validate the full implementation

2. **Be Historical Context-Aware**:

   - Use git history to understand the implementation timeline
   - Look for commits made during the implementation period
   - Consider work that may have been done days/weeks ago
   - Validate against the actual commits, not just current state

3. **Be Agent-Driven**:

   - Spawn multiple Task agents for different validation aspects
   - Let agents do the detailed analysis work
   - Wait for ALL agents to complete before synthesizing
   - Use agent findings to create comprehensive validation

4. **Be Evidence-Based**:

   - Reference specific files, line numbers, and commits
   - Include actual test results and command outputs
   - Provide concrete examples of deviations or issues
   - Base conclusions on verifiable evidence

5. **Follow Standards**:

   - Use the validation template consistently
   - Match summary naming exactly for validation document
   - Update CLAUDE.md with new document entries
   - Include proper metadata and references

## Validation Focus Areas

### Implementation Completeness:
- All planned features implemented
- Success criteria met (automated and manual)
- No critical functionality missing
- Proper error handling implemented

### Code Quality:
- Follows existing patterns and conventions
- No obvious technical debt introduced
- Appropriate test coverage
- Documentation updated where needed

### Integration:
- Works with existing systems
- No regressions introduced
- Proper data migration if applicable
- External integrations functioning

### Performance:
- No performance degradation
- Scalability considerations addressed  
- Resource usage appropriate
- Response times acceptable

## Success Criteria

A successful validation should:

- **Verify implementation completeness** against the original plan
- **Confirm quality standards** are maintained
- **Identify any issues or gaps** that need attention
- **Provide actionable recommendations** for improvement
- **Document the validation results** comprehensively

The validation serves as the final quality gate before considering implementation work complete.
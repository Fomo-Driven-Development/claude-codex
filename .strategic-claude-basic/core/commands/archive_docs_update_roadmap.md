---
description: "Archive completed documentation and update product roadmap"
argument-hint: <summary_or_validation_file>
allowed-tools: Read(./**), Write(./.strategic-claude-basic/product/**), Bash(mv:*, mkdir:*, cp:*, git:*, sha256sum:*, date:*), Glob
model: claude-opus-4-1
---

You are tasked with archiving completed documentation to the archives directory and updating the product roadmap with completion status. This command transitions completed work from active directories to organized archives.

**Summary or validation file provided:** $1

## Initial Response

When this command is invoked:

1. **Check if input file was provided**:

   - If file path was provided, proceed with analysis
   - If no file provided, respond with:

   ```
   I'll archive completed documentation and update the product roadmap.

   Please provide either a summary or validation document to archive:
   Examples:
   - `/archive_docs_update_roadmap @.strategic-claude-basic/summary/SUMMARY_0001_07-09-2025_sat_user-auth.md`
   - `/archive_docs_update_roadmap @.strategic-claude-basic/validation/VALIDATION_0001_07-09-2025_sat_user-auth.md`

   I'll find all related documents and move them to archives if the work is complete.
   ```

   Then wait for the user's input.

## Process Steps

### Step 1: Document Analysis and Completion Check

1. **Read the input document completely**:

   - Determine if it's a summary or validation document
   - Extract completion status and phase information
   - Get plan reference and subject from frontmatter
   - Check if work is marked as complete

2. **Validate completion status**:

   **For validation documents**:
   - Look for overall validation results
   - Check if critical issues are resolved
   - Confirm automated tests are passing

   **For summary documents**:
   - Check completion_rate in frontmatter
   - Look for status (complete/blocked/partial)
   - Review outstanding issues section

3. **Present completion status**:

   ```
   Document Analysis:
   Type: [Summary/Validation]
   Subject: [feature name]
   Status: [complete/partial/blocked]
   Completion Rate: [X%]
   
   [If complete]: ✅ Work appears complete - proceeding with archive search
   [If incomplete]: ⚠️ Work not complete - archive not recommended
   
   Do you want to proceed with archiving? [Y/N]
   ```

### Step 2: Document Chain Discovery

1. **Find all connected documents**:

   - Use NNNN number and subject to find related files
   - Search all documentation directories for matches:
     - Plan documents (same NNNN and subject)
     - Summary documents (if starting from validation)
     - Validation documents (if starting from summary)  
     - Research documents (matching subject or phase)
     - Issue documents (matching NNNN or created from summary)

2. **Present archive list**:

   ```
   Found documents to archive:
   
   Core Documents:
   - Plan: PLAN_0001_07-09-2025_sat_user-auth.md
   - Summary: SUMMARY_0001_07-09-2025_sat_user-auth.md
   - Validation: VALIDATION_0001_07-09-2025_sat_user-auth.md
   
   Supporting Documents:
   - Research: RESEARCH_0002_05-09-2025_thu_user-auth-patterns.md
   - Issues: ISSUE_0001_08-09-2025_sun_auth-token-expiry.md
   
   Total: [N] documents will be archived together.
   Proceed with archiving? [Y/N]
   ```

### Step 3: Archive Directory Creation

1. **Generate unique archive directory name**:

   ```bash
   # Create unique identifier using date and hash
   archive_date=$(date '+%Y-%m-%d')
   subject_hash=$(echo "[subject]" | sha256sum | head -c 6)
   archive_dir="${archive_date}-${subject_hash}"
   ```

2. **Create archive directory structure**:

   ```bash
   mkdir -p ".strategic-claude-basic/archives/${archive_dir}"
   ```

3. **Generate archive README**:

   - Create README.md with archive metadata
   - List all documents included
   - Include completion status and dates
   - Reference original locations

### Step 4: Document Archiving

1. **Move documents to archive**:

   ```bash
   # Move each document to the archive directory
   mv ".strategic-claude-basic/plan/[plan-file]" ".strategic-claude-basic/archives/${archive_dir}/"
   mv ".strategic-claude-basic/summary/[summary-file]" ".strategic-claude-basic/archives/${archive_dir}/"
   # ... continue for all found documents
   ```

2. **Create archive README.md**:

   ```markdown
   # Archive: [Feature Name]
   
   **Archived**: [Current date]
   **Feature**: [Feature description]
   **Status**: Complete
   **Archive ID**: [archive_dir]
   
   ## Documents Included
   
   - **Plan**: [filename] - Original implementation plan
   - **Summary**: [filename] - Implementation summary  
   - **Validation**: [filename] - Validation results
   - **Research**: [filename] - Supporting research
   - **Issues**: [filename] - Related issues
   
   ## Completion Details
   
   [Extract key completion details from validation/summary]
   
   ## Original Locations
   
   All documents were moved from their respective directories in `.strategic-claude-basic/`
   ```

### Step 5: Roadmap Update

1. **Find or create roadmap**:

   - Look for existing ROADMAP.md in `.strategic-claude-basic/product/`
   - If not found, use ROADMAP.template.md to create one
   - Make backup copy before modifying

2. **Update roadmap with completion**:

   - Find relevant milestone/phase sections
   - Update status from "Planned/In Progress" to "Complete"
   - Add completion date
   - Update deliverable checkboxes
   - Add reference to archive directory

3. **Add completion entry to roadmap**:

   ```markdown
   ### Completed Work
   
   **[Feature Name]** - Completed [Date]
   - Status: ✅ Complete
   - Archive: `archives/[archive_dir]/`
   - Key Deliverables: [List from plan]
   - Validation Results: [Summary from validation]
   ```

### Step 6: Finalization

1. **Present completion summary**:

   ```
   ✅ Archive Complete!
   
   Archived to: .strategic-claude-basic/archives/[archive_dir]/
   Documents moved: [N] files
   Roadmap updated: ✅
   
   Archive contains:
   - [List of moved files]
   
   Roadmap updated with completion status and archive reference.
   Ready to commit changes? [Y/N]
   ```

2. **Commit changes**:

   ```bash
   git add .strategic-claude-basic/archives/[archive_dir]/
   git add .strategic-claude-basic/product/ROADMAP.md
   git commit -m "Archive completed [feature] documentation

   - Move [N] documents to archives/[archive_dir]/
   - Update roadmap with completion status
   - Feature: [brief description]
   - Status: Complete

   🤖 Generated with [Claude Code](https://claude.ai/code)

   Co-Authored-By: Claude <noreply@anthropic.com>"
   ```

## Important Guidelines

1. **Completion Verification**:

   - Only archive work that is actually complete
   - Check validation results if available
   - Confirm no critical outstanding issues
   - Allow user override for special cases

2. **Document Discovery**:

   - Use naming conventions to find all related docs
   - Search all directories systematically
   - Include supporting documents (research, issues)
   - Preserve document relationships

3. **Archive Organization**:

   - Keep all related documents together
   - Use unique but meaningful directory names
   - Create comprehensive archive README
   - Maintain traceability to original locations

4. **Roadmap Integration**:

   - Update milestone status accurately
   - Add completion dates and metrics
   - Reference archive location
   - Backup original before modifying

5. **Safety and Confirmation**:

   - Always confirm before moving files
   - Show complete list of what will be archived
   - Create backups of important files
   - Commit changes for version control

## Archive Directory Structure

```
.strategic-claude-basic/archives/
└── 2024-09-07-a3f2b1/    # Date + hash identifier
    ├── README.md         # Archive metadata
    ├── PLAN_0001_07-09-2025_sat_user-auth.md
    ├── SUMMARY_0001_07-09-2025_sat_user-auth.md
    ├── VALIDATION_0001_07-09-2025_sat_user-auth.md
    ├── RESEARCH_0002_05-09-2025_thu_user-auth-patterns.md
    └── ISSUE_0001_08-09-2025_sun_auth-token-expiry.md
```

## Success Criteria

A successful archive operation should:

- **Move all related documents** to a single archive directory
- **Update roadmap** with accurate completion status
- **Preserve document relationships** through archive README
- **Commit changes** for version control
- **Confirm completion** before proceeding

This command provides clean project lifecycle management, transitioning completed work from active development to organized archives while keeping the roadmap current.
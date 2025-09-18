# ast-grep Patterns Guide for Go

ast-grep is a powerful tool for structural code search and transformation using Abstract Syntax Tree (AST) patterns. Unlike traditional text-based search tools, ast-grep understands code structure, making it ideal for precise code analysis.

## Why ast-grep?

- **Precision**: Matches code structure, not just text patterns
- **Language-aware**: Understands Go syntax and semantics  
- **Fast**: Written in Rust, processes thousands of files quickly
- **Flexible**: Multiple strictness levels for different matching needs

## Basic Command Syntax

```bash
ast-grep run -p 'PATTERN' -l go [PATHS]
```

**Key Flags:**
- `-p, --pattern`: AST pattern to match
- `-l, --lang go`: Set language to Go
- `-r, --rewrite`: Replace pattern (for transformations)
- `--json`: Output structured JSON results
- `--strictness`: Set matching strictness (cst, smart, ast, relaxed, signature)

## Core Pattern Concepts

### Meta Variables
- `$NAME` - Matches any single AST node
- `$$$` - Matches zero or more AST nodes
- Pattern must be valid Go syntax

### Strictness Levels
- **smart** (default): Skips unnamed nodes in target code
- **cst**: Exact node-by-node matching
- **ast**: Only named AST nodes
- **relaxed**: Ignores comments and unnamed nodes

## Go-Specific Patterns

### Function Declarations

```bash
# Find all functions
ast-grep run -p 'func $NAME($$$) $$$' -l go

# Find functions with specific return types
ast-grep run -p 'func $NAME($$$) error { $$$ }' -l go

# Find functions returning error
ast-grep run -p 'func $NAME($$$) ($$$, error) { $$$ }' -l go

# Find test functions
ast-grep run -p 'func Test$NAME(t *testing.T) { $$$ }' -l go
```

### Method Declarations

```bash
# Find all methods
ast-grep run -p 'func ($RECV $TYPE) $METHOD($$$) $$$' -l go

# Find methods on pointer receivers
ast-grep run -p 'func ($RECV *$TYPE) $METHOD($$$) $$$' -l go

# Find methods with error returns
ast-grep run -p 'func ($RECV $TYPE) $METHOD($$$) error { $$$ }' -l go
```

### Type Definitions

```bash
# Find struct definitions
ast-grep run -p 'type $NAME struct { $$$ }' -l go

# Find interface definitions
ast-grep run -p 'type $NAME interface { $$$ }' -l go

# Find type aliases
ast-grep run -p 'type $NAME = $TYPE' -l go

# Find custom types
ast-grep run -p 'type $NAME $TYPE' -l go
```

### Error Handling Patterns

```bash
# Find error checks
ast-grep run -p 'if $ERR != nil { $$$ }' -l go

# Find error returns
ast-grep run -p 'return $$$, $ERR' -l go

# Find wrapped errors
ast-grep run -p 'fmt.Errorf($$$)' -l go

# Find errors.New calls
ast-grep run -p 'errors.New($MSG)' -l go

# Find custom error types
ast-grep run -p 'type $NAME struct { $$$ }
func ($RECV $NAME) Error() string { $$$ }' -l go
```

### Concurrency Patterns

```bash
# Find goroutine launches
ast-grep run -p 'go $FUNC($$$)' -l go

# Find channel sends
ast-grep run -p '$CHAN <- $VALUE' -l go

# Find channel receives
ast-grep run -p '$VAR := <-$CHAN' -l go

# Find channel creates
ast-grep run -p 'make(chan $TYPE)' -l go

# Find select statements
ast-grep run -p 'select { $$$ }' -l go

# Find mutex locks
ast-grep run -p '$MU.Lock()' -l go
```

### Control Flow Patterns

```bash
# Find defer statements
ast-grep run -p 'defer $FUNC($$$)' -l go

# Find panic calls
ast-grep run -p 'panic($$$)' -l go

# Find recover calls
ast-grep run -p 'recover()' -l go

# Find range loops
ast-grep run -p 'for $KEY, $VALUE := range $COLLECTION { $$$ }' -l go

# Find infinite loops
ast-grep run -p 'for { $$$ }' -l go
```

### HTTP and API Patterns

```bash
# Find HTTP handlers
ast-grep run -p 'func $NAME(w http.ResponseWriter, r *http.Request) { $$$ }' -l go

# Find middleware functions
ast-grep run -p 'func $NAME(next http.Handler) http.Handler { $$$ }' -l go

# Find HTTP method checks
ast-grep run -p 'r.Method == "$METHOD"' -l go

# Find JSON marshal/unmarshal
ast-grep run -p 'json.Marshal($$$)' -l go
ast-grep run -p 'json.Unmarshal($$$)' -l go
```

### Testing Patterns

```bash
# Find test functions
ast-grep run -p 'func Test$NAME(t *testing.T) { $$$ }' -l go

# Find benchmark functions  
ast-grep run -p 'func Benchmark$NAME(b *testing.B) { $$$ }' -l go

# Find table tests
ast-grep run -p 'tests := []struct { $$$ }' -l go

# Find test assertions
ast-grep run -p 't.Error($$$)' -l go
ast-grep run -p 't.Fatal($$$)' -l go
ast-grep run -p 'assert.$METHOD($$$)' -l go
```

### Package and Import Patterns

```bash
# Find specific imports
ast-grep run -p 'import "$PACKAGE"' -l go

# Find grouped imports
ast-grep run -p 'import ( $$$ )' -l go

# Find package declarations
ast-grep run -p 'package $NAME' -l go

# Find init functions
ast-grep run -p 'func init() { $$$ }' -l go
```

## Go-Specific Considerations

### Function Call Ambiguity
Go's type conversions look like function calls. Use context or selectors:

```bash
# Ambiguous - matches both calls and type conversions
ast-grep run -p 'fmt.Println($A)' -l go

# Better - use in function context
ast-grep run -p 'func $F() { fmt.Println($A) }' --selector call_expression -l go
```

### Regex for Name Patterns
Use YAML rules for prefix matching:

```yaml
# For functions starting with "Test"
rule:
  pattern: func $NAME() {}
  where:
    NAME:
      regex: "Test.*"
```

## When to Use ast-grep vs grep

### Use ast-grep for:
- Function/method definitions
- Type definitions
- Structural patterns (error handling, defer, etc.)
- Control flow patterns
- API patterns
- Refactoring tasks

### Use grep for:
- Comments
- String literals
- Documentation
- Log messages
- Quick text searches across files

### Use both:
1. ast-grep to find structural matches
2. grep to search within the context
3. Read tool to examine full implementation

## Advanced Usage

### JSON Output for Analysis
```bash
# Get structured data about matches
ast-grep run -p 'func $NAME($$$) error { $$$ }' -l go --json | jq '.[] | .file'
```

### Combining with Other Tools
```bash
# Find functions, then examine with grep
ast-grep run -p 'func $NAME($$$) { $$$ }' -l go --json | jq -r '.[] | .file' | xargs grep -n "TODO"
```

### Interactive Rewriting
```bash
# Find and replace patterns interactively
ast-grep run -p 'if err != nil { return err }' -r 'if err != nil { return fmt.Errorf("operation failed: %w", err) }' -l go --interactive
```

## Pro Tips

1. **Start Simple**: Begin with basic patterns and add complexity
2. **Use Playground**: Test patterns at https://ast-grep.github.io/playground.html
3. **Check Valid Syntax**: Patterns must be parseable Go code
4. **Context Matters**: Use surrounding code for disambiguation
5. **Combine Tools**: Use ast-grep for structure, grep/read for details
6. **JSON Mode**: Use `--json` for programmatic processing of results

## Common Gotchas

- Pattern must be valid Go syntax
- Meta variables are case-sensitive  
- `$$$` matches zero or more, so `func f($$$)` matches `func f()`
- Type conversions can interfere with function call matching
- Package imports need specific context for accurate matching

This guide covers the most common patterns for Go development. For more complex patterns, use YAML rule files or the ast-grep playground to test and refine your searches.
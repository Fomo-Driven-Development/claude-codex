use std::collections::HashMap;

#[derive(Debug)]
pub enum TemplateError {
    MissingVariable(String),
    ProcessingError(String),
}

#[derive(Debug, Clone)]
pub enum TemplateSyntax {
    Simple, // {0}, {1}, {2}
    Askama, // {{ variable }}
}

pub fn process_simple_template(content: &str, args: &[String]) -> Result<String, TemplateError> {
    let mut result = content.to_string();
    for (i, arg) in args.iter().enumerate() {
        let placeholder = format!("{{{}}}", i);
        result = result.replace(&placeholder, arg);
    }
    Ok(result)
}

pub fn process_askama_template(
    content: &str,
    args: &HashMap<String, String>,
) -> Result<String, TemplateError> {
    // Simple variable substitution for {{ variable }} syntax
    let mut result = content.to_string();
    for (key, value) in args {
        let placeholder = format!("{{{{ {} }}}}", key);
        result = result.replace(&placeholder, value);
    }
    Ok(result)
}

pub fn process_template(
    content: &str,
    args: &[String],
    syntax: TemplateSyntax,
) -> Result<String, TemplateError> {
    match syntax {
        TemplateSyntax::Simple => process_simple_template(content, args),
        TemplateSyntax::Askama => {
            // Convert positional arguments to named arguments for Askama
            // For now, use simple numeric names: arg0, arg1, etc.
            let mut named_args = HashMap::new();
            for (i, arg) in args.iter().enumerate() {
                named_args.insert(format!("arg{}", i), arg.clone());
            }
            // Also support 'subject' as first argument for backward compatibility
            if let Some(first_arg) = args.get(0) {
                named_args.insert("subject".to_string(), first_arg.clone());
            }
            process_askama_template(content, &named_args)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_template_no_args() {
        let result = process_simple_template("Hello world", &[]).unwrap();
        assert_eq!(result, "Hello world");
    }

    #[test]
    fn test_simple_template_single_arg() {
        let result = process_simple_template("Hello {0}!", &["World".to_string()]).unwrap();
        assert_eq!(result, "Hello World!");
    }

    #[test]
    fn test_simple_template_multiple_args() {
        let args = vec!["Alice".to_string(), "Bob".to_string()];
        let result = process_simple_template("Hello {0} and {1}!", &args).unwrap();
        assert_eq!(result, "Hello Alice and Bob!");
    }

    #[test]
    fn test_simple_template_repeated_placeholder() {
        let result =
            process_simple_template("{0} says hello to {0}", &["Alice".to_string()]).unwrap();
        assert_eq!(result, "Alice says hello to Alice");
    }

    #[test]
    fn test_askama_template_single_variable() {
        let mut args = HashMap::new();
        args.insert("subject".to_string(), "AI safety".to_string());
        let result = process_askama_template("Research {{ subject }}", &args).unwrap();
        assert_eq!(result, "Research AI safety");
    }

    #[test]
    fn test_askama_template_multiple_variables() {
        let mut args = HashMap::new();
        args.insert("topic".to_string(), "Rust".to_string());
        args.insert("level".to_string(), "beginner".to_string());
        let result =
            process_askama_template("Learn {{ topic }} at {{ level }} level", &args).unwrap();
        assert_eq!(result, "Learn Rust at beginner level");
    }

    #[test]
    fn test_process_template_simple() {
        let args = vec!["test".to_string()];
        let result = process_template("Command: {0}", &args, TemplateSyntax::Simple).unwrap();
        assert_eq!(result, "Command: test");
    }

    #[test]
    fn test_process_template_askama_with_subject() {
        let args = vec!["AI safety".to_string()];
        let result =
            process_template("Research {{ subject }}", &args, TemplateSyntax::Askama).unwrap();
        assert_eq!(result, "Research AI safety");
    }

    #[test]
    fn test_process_template_askama_with_positional() {
        let args = vec!["first".to_string(), "second".to_string()];
        let result =
            process_template("{{ arg0 }} and {{ arg1 }}", &args, TemplateSyntax::Askama).unwrap();
        assert_eq!(result, "first and second");
    }
}

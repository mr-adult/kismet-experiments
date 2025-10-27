/// Centralized syntax highlighting theme for bytecode formatters
///
/// This module provides a consistent color scheme across all formatters,
/// making it easy to maintain and customize the visual appearance of output.
use colored::*;

/// Semantic roles for syntax highlighting
pub struct Theme;

impl Theme {
    // === Labels and control flow ===

    /// Labels for jump targets and control flow markers
    pub fn label(text: impl std::fmt::Display) -> ColoredString {
        format!("{}", text).red().bold()
    }

    // === Identifiers ===

    /// Variables, properties, and field names
    pub fn variable(text: impl std::fmt::Display) -> ColoredString {
        format!("{}", text).bright_yellow()
    }

    /// Function and method names
    pub fn function(text: impl std::fmt::Display) -> ColoredString {
        format!("{}", text).magenta().bold()
    }

    /// Type names (classes, structs, interfaces)
    pub fn type_name(text: impl std::fmt::Display) -> ColoredString {
        format!("{}", text).bright_cyan()
    }

    /// Object references and special identifiers (like 'this')
    pub fn object_ref(text: impl std::fmt::Display) -> ColoredString {
        format!("{}", text).cyan()
    }

    // === Literals ===

    /// Numeric literals (integers, floats)
    pub fn numeric(text: impl std::fmt::Display) -> ColoredString {
        format!("{}", text).yellow()
    }

    /// Numeric literals with emphasis
    pub fn numeric_bold(text: impl std::fmt::Display) -> ColoredString {
        format!("{}", text).yellow().bold()
    }

    /// String literals
    pub fn string(text: impl std::fmt::Display) -> ColoredString {
        format!("{}", text).green().bold()
    }

    /// Boolean literals and keywords
    pub fn keyword(text: impl std::fmt::Display) -> ColoredString {
        format!("{}", text).green()
    }

    // === Special values ===

    /// Null/none/nothing values
    pub fn null_value(text: impl std::fmt::Display) -> ColoredString {
        format!("{}", text).dimmed()
    }

    // === Assembly-specific ===

    /// Opcode identifiers (assembly format only)
    pub fn opcode(text: impl std::fmt::Display) -> ColoredString {
        format!("{}", text).cyan().bold()
    }

    /// Tag labels (assembly format only)
    pub fn tag(text: impl std::fmt::Display) -> ColoredString {
        format!("{}", text).bright_black().bold()
    }

    // === Comments and metadata ===

    /// Comments and secondary information
    pub fn comment(text: impl std::fmt::Display) -> ColoredString {
        format!("{}", text).bright_black()
    }

    // === Offsets and addresses ===

    /// Memory offsets and addresses
    pub fn offset(text: impl std::fmt::Display) -> ColoredString {
        format!("{}", text).yellow().bold()
    }
}

// Convenience functions for common patterns

/// Format a quoted string literal
pub fn quoted_string(text: &str) -> ColoredString {
    Theme::string(format!("\"{}\"", text))
}

/// Format a type with angle brackets (e.g., TArray<int>)
pub fn generic_type(base: &str, params: &[&str]) -> String {
    if params.is_empty() {
        Theme::type_name(base).to_string()
    } else {
        format!("{}<{}>", Theme::type_name(base), params.join(", "))
    }
}

/// Format a function call with colored components
pub fn function_call(func_name: &str, args: &[String]) -> String {
    format!("{}({})", Theme::function(func_name), args.join(", "))
}

/// Format a member access expression (obj.member)
pub fn member_access(object: &str, member: &str, is_variable: bool) -> String {
    if is_variable {
        format!("{}.{}", object, Theme::variable(member))
    } else {
        format!("{}.{}", object, member)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_theming() {
        // Just ensure the functions compile and return ColoredString
        let _ = Theme::label("test");
        let _ = Theme::variable("myVar");
        let _ = Theme::function("myFunc");
        let _ = Theme::type_name("MyClass");
        let _ = Theme::numeric(42);
        let _ = Theme::string("hello");
    }

    #[test]
    fn test_helper_functions() {
        let _ = quoted_string("hello world");
        let _ = generic_type("TArray", &["int32"]);
        let _ = function_call("DoSomething", &["arg1".to_string(), "arg2".to_string()]);
        let _ = member_access("this", "myField", true);
    }
}

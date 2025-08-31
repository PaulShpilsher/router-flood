//! User input prompt utilities
//!
//! This module provides utilities for prompting user input in interactive mode.

use crate::error::Result;
use std::io::{self, Write};

/// Utility for handling user prompts
pub struct PromptUtils;

impl PromptUtils {
    /// Prompt for user input with default value
    pub fn prompt_for_input(prompt: &str, default: &str) -> Result<String> {
        print!("{}", prompt);
        if !default.is_empty() {
            print!(" [{}]", default);
        }
        print!(": ");
        let _ = io::stdout().flush();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap_or_default();
        let input = input.trim();

        if input.is_empty() && !default.is_empty() {
            Ok(default.to_string())
        } else {
            Ok(input.to_string())
        }
    }

    /// Prompt for yes/no input
    pub fn prompt_yes_no(prompt: &str, default: bool) -> Result<bool> {
        let default_str = if default { "Y/n" } else { "y/N" };
        print!("{} [{}]: ", prompt, default_str);
        let _ = io::stdout().flush();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap_or_default();
        let input = input.trim().to_lowercase();

        match input.as_str() {
            "y" | "yes" => Ok(true),
            "n" | "no" => Ok(false),
            "" => Ok(default),
            _ => {
                println!("Please enter 'y' or 'n'");
                Self::prompt_yes_no(prompt, default)
            }
        }
    }

    /// Prompt for a choice from a list of options
    pub fn prompt_choice(prompt: &str, options: &[&str], default: Option<usize>) -> Result<String> {
        println!("{}", prompt);
        for (i, option) in options.iter().enumerate() {
            if Some(i) == default {
                println!("  {}. {} (default)", i + 1, option);
            } else {
                println!("  {}. {}", i + 1, option);
            }
        }
        
        print!("Enter choice");
        if let Some(d) = default {
            print!(" [{}]", d + 1);
        }
        print!(": ");
        let _ = io::stdout().flush();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap_or_default();
        let input = input.trim();

        if input.is_empty() {
            if let Some(default_idx) = default {
                Ok(options[default_idx].to_string())
            } else {
                println!("No input provided and no default available");
                Self::prompt_choice(prompt, options, default)
            }
        } else if let Ok(choice) = input.parse::<usize>() {
            if choice > 0 && choice <= options.len() {
                Ok(options[choice - 1].to_string())
            } else {
                println!("Invalid choice. Please select 1-{}", options.len());
                Self::prompt_choice(prompt, options, default)
            }
        } else {
            println!("Invalid input. Please enter a number.");
            Self::prompt_choice(prompt, options, default)
        }
    }

    /// Display a separator line
    pub fn display_separator() {
        println!("==========================================");
    }
}
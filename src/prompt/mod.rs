mod deprecated;

use clap::ValueEnum;
use deprecated::{PROMPT2, PROMPT3, PROMPT4, PROMPT5};
use std::fmt::{Display, Formatter};

/// Represents different prompt templates for generating commit messages
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
pub enum Prompt {
    /// Detailed prompt for generating multiple conventional commit messages
    P1,
    /// Simple prompt for generating a single conventional commit message
    P2,
    /// Alternative prompt format for commit message generation
    P3,
    /// Senior engineer style commit message prompt
    P4,
    /// Custom prompt template
    P5,
}

impl Display for Prompt {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Prompt::P1 => write!(f, "p1"),
            Prompt::P2 => write!(f, "p2"),
            Prompt::P3 => write!(f, "p3"),
            Prompt::P4 => write!(f, "p4"),
            Prompt::P5 => write!(f, "p5"),
        }
    }
}

impl Prompt {
    /// Returns the prompt template string for the selected prompt type
    pub(crate) fn value(self) -> &'static str {
        match self {
            Prompt::P1 => PROMPT,
            Prompt::P2 => PROMPT2,
            Prompt::P3 => PROMPT3,
            Prompt::P4 => PROMPT4,
            Prompt::P5 => PROMPT5,
        }
    }
}

pub const PROMPT: &str = r###"Generate {{ number }} commit messages following the Conventional Commits specification (https://www.conventionalcommits.org/).

Format for each message object:
{
  "type": "<type>",                      // Type of change (see rules below)
  "scope": "<module or component name>",  // Affected component in parentheses
  "subject": "<concise description>",     // Imperative mood, no capitalization, no period
  "body": "<detailed explanation>",       // Why the change was made, what changed
  "footer": "<optional footer>"           // Breaking changes, references
}

Rules for Conventional Commits:
1. Type MUST be one of:
   - feat: A new feature
   - fix: A bug fix
   - docs: Documentation only changes
   - style: Changes that do not affect the meaning of the code
   - refactor: A code change that neither fixes a bug nor adds a feature
   - test: Adding missing tests or correcting existing tests
   - chore: Changes to the build process or auxiliary tools
   - ci: Changes to CI configuration files and scripts

2. Format Requirements:
   - Subject MUST be in imperative mood ("add" not "adds" or "added")
   - Subject MUST NOT start with a capital letter
   - Subject MUST NOT end with a period
   - Subject MUST be max 80 characters
   - Body MUST explain both motivation and changes made
   - Breaking changes MUST be indicated in footer

3. Technical Requirements:
   - Response MUST be valid JSON array with {{ number }} messages
   - Wrap response in ```json
   - Use \" for quotes in strings
   - Use \n for newlines in strings
   - No trailing commas in arrays/objects

Example:
```json
[
  {
    "type": "feat",
    "scope": "auth",
    "subject": "add oauth2 authentication flow",
    "body": "implement secure authentication using OAuth2 protocol\n- add login endpoint\n- integrate with external providers\n- handle token refresh",
    "footer": "BREAKING CHANGE: authentication header format changed"
  },
  {
    "type": "fix",
    "scope": "api",
    "subject": "handle null response in user profile",
    "body": "add null checks to prevent app crash when user profile is incomplete",
    "footer": null
  },
  {
    "type": "refactor",
    "scope": "core",
    "subject": "simplify error handling logic",
    "body": "centralize error handling to reduce code duplication and improve maintainability",
    "footer": null
  }
]
```"###;

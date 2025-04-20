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

pub const PROMPT: &str = r###"
### Instructions
You are a expert software developer and master of Conventional Commits.
Generate the appropriate {{ number }} git commit messages based on the supplied git diff content, and the message must be following the Conventional Commits specification.

## Workflow
1. Analyze the content of the change according to the git diff context and summarize it.
2. Determine the type and scope based on the changes, and give the most likely types and scopes when multiple commit options are requested.
3. Format the output according to the constraints.

## Constraints (Must follow)
- language of subject and body: {{ language }}
- number of commit messages: {{ number }}
- output schema: The output must be a valid JSON object with the following structure, without any other text. if number is 1, the output must be a array with one element.

### Schema
As an example, for the schema {"properties": {"foo": {"title": "Foo", "description": "a list of strings", "type": "array", "items": {"type": "string"}}}, "required": ["foo"]}}
the object {"foo": ["bar", "baz"]} is a well-formatted instance of the schema. The object {"properties": {"foo": ["bar", "baz"]}} is not well-formatted.

Here is the output schema:
```json
{
    "title": "Conventional Commits",
    "description": "Generate conventional commit messages",
    "type": "array",
    "items": {
        "type": "object",
        "description": "Conventional commit message",
        "properties": {
            "type": {
                "type": "string",
                "description": "Type of current commit",
                "enum": ["feat", "fix", "docs", "style", "refactor", "test", "chore", "ci", "revert", "build", "perf"]
            },
            "scope": {
                "type": "string",
                "description": "Affected component, e.g. auth/view"
            },
            "subject": {
                "type": "string",
                "description": "Short summary of the change, must be in imperative mood and under 80 characters, e.g. add oauth2 authentication flow"
            },
            "body": {
                "type": "string",
                "description": "Detailed description of the change"
            },
            "footer": {
                "type": "string",
                "description": "Additional information, e.g., breaking changes"
            }
        },
        "required": ["type", "subject"]
    },
    "minItems": {{ number }},
    "maxItems": {{ number }}
}
```

The output must be a valid JSON array of commit messages without any other text.

## Example
```json
[
  {
    "type": "feat",
    "scope": "auth",
    "subject": "add oauth2 authentication flow",
    "body": "implement secure authentication using OAuth2 protocol\n- add login endpoint\n- integrate with external providers\n- handle token refresh",
    "footer": "BREAKING CHANGE: authentication header format changed"
  }
]
```
"###;

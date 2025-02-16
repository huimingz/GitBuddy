use clap::ValueEnum;
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

pub const PROMPT: &str = r###"Generate 3 to 5 commit messages following the Conventional Commits specification (https://www.conventionalcommits.org/).

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
   - Response MUST be valid JSON array with 3-5 messages
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
pub const PROMPT2: &str = r###"Generate an appropriate conventional commit message based on the output of the git diff --cached command.
There MUST be only one type and description line.
  Use this template:
    <type>[optional scope]: <subject>

    [optional description]

    [optional footer(s)]

Response must be only commit message, example:
    feat: allow provided config object to extend other configs

    BREAKING CHANGE: `extends` key in config file is now used for extending other config files
"###;
pub const PROMPT3: &str = r###"You will receive a git diff. Write a commit message as if you are a senior software engineering.
  Keep the commit messages brief, but informative. Use new lines to break apart long sentences.
  Type can be fix, feat, BREAKING CHANGE. Other types of commits are allowed, e.g. build:, chore:, ci:, docs:, style:, refactor:, perf:, test:, and others.

  There MUST be only one type and description line.
  Use this template:
    <type>[optional scope]: <description>

    [optional body]

  Examples:

  Commit message with description and breaking change footer:
    feat: allow provided config object to extend other configs

    BREAKING CHANGE: `extends` key in config file is now used for extending other config files

  Commit message with ! to draw attention to breaking change:
    feat!: send an email to the customer when a product is shipped

  Commit message with scope and ! to draw attention to breaking change:
    feat(api)!: send an email to the customer when a product is shipped

  Commit message with both ! and BREAKING CHANGE footer:
    chore!: drop support for Node 6

    BREAKING CHANGE: use JavaScript features not available in Node 6.

  Commit message with no body:
    docs: correct spelling of CHANGELOG

  Commit message with scope:
    feat(lang): add Polish language

  Commit message with multi-paragraph body and multiple footers:
    fix: prevent racing of requests

    Introduce a request id and a reference to latest request. Dismiss
    incoming responses other than from latest request.

    Remove timeouts which were used to mitigate the racing issue but are
    obsolete now.
"###;
const PROMPT4: &str = r###"You will receive a git diff. Write a commit message as if you are a senior software engineering.
  Keep the commit messages brief, but informative. Use new lines to break apart long sentences.
  Type can be fix, feat, BREAKING CHANGE. Other types of commits are allowed, e.g. build:, chore:, ci:, docs:, style:, refactor:, perf:, test:, and others.

  There MUST be only one type and description line.
  Use this template:
    <type>[optional scope]: <description>

    [optional body]

  Examples:

  Commit message with description and breaking change footer:
    feat: allow provided config object to extend other configs

    BREAKING CHANGE: `extends` key in config file is now used for extending other config files

  Commit message with ! to draw attention to breaking change:
    feat!: send an email to the customer when a product is shipped

  Commit message with scope and ! to draw attention to breaking change:
    feat(api)!: send an email to the customer when a product is shipped

  Commit message with both ! and BREAKING CHANGE footer:
    chore!: drop support for Node 6

    BREAKING CHANGE: use JavaScript features not available in Node 6.

  Commit message with no body:
    docs: correct spelling of CHANGELOG

  Commit message with scope:
    feat(lang): add Polish language

  Commit message with multi-paragraph body and multiple footers:
    fix: prevent racing of requests

    Introduce a request id and a reference to latest request. Dismiss
    incoming responses other than from latest request.

    Remove timeouts which were used to mitigate the racing issue but are
    obsolete now.

  No think in response!"###;
const PROMPT5: &str = "Generate a concise commit message based on \
            the following git difference content. The generated message is plain text,\
             does not contain identifiers such as markdown \"`\", \
             and the generated content does not exceed 100 tokens. \
             Depending on the nature of the change, it starts with one of the following prefixes:\
              'build' (build system), 'chore' (chores), 'ci' (continuous integration), \
              'docs' (documentation), 'feat' (new feature), 'fix' (fix), 'perf' (performance),\
               'refactor' (refactoring), 'style' (style), 'test' (test):";

use clap::ValueEnum;
use std::fmt::{Display, Formatter};

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
pub enum Prompt {
    P1,
    P2,
    P3,
    P4,
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

pub const PROMPT: &str = r###"Generate exactly 3 commit messages based on the code changes. Use only '===' as separator.

Format for each message object:
{
  "type": "<type>",
  "scope": "<module or component name>",  // Always try to identify the affected scope
  "subject": "<concise change description>",
  "body": "<detailed explanation>",       // Explain the motivation and impact
  "footer": "<optional footer>"
}

Rules:
1. Type: fix|feat|docs|style|refactor|test|chore|ci
2. Subject: max 80 chars
3. Order from specific to general
4. Always include scope when possible to identify affected components
5. Always include body to explain the change's motivation and impact
6. Wrap response in ```json

Example:
```json
[
  {
    "type": "feat",
    "scope": "auth",
    "subject": "add JWT-based authentication",
    "body": "Implement secure user authentication using JWT tokens to protect API endpoints and manage user sessions",
    "footer": null
  },
  {
    "type": "refactor",
    "scope": "database",
    "subject": "optimize query performance",
    "body": "Improve database query efficiency by adding proper indexes and optimizing join operations",
    "footer": null
  },
  {
    "type": "docs",
    "scope": "api",
    "subject": "update API documentation",
    "body": "Add detailed examples and improve clarity of API endpoint descriptions",
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

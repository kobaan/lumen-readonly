use crate::{
    command::{explain::ExplainCommand},
    git_entity::{diff::Diff, GitEntity},
};
use indoc::{formatdoc, indoc};
use thiserror::Error;

#[derive(Error, Debug)]
#[error("{0}")]
pub struct AIPromptError(String);

pub struct AIPrompt {
    pub system_prompt: String,
    pub user_prompt: String,
}

impl AIPrompt {
    pub fn build_explain_prompt(command: &ExplainCommand) -> Result<Self, AIPromptError> {
        let system_prompt = String::from(indoc! {"
            You are a helpful assistant that explains Git changes in a concise way.
            Focus only on the most significant changes and their direct impact.
            When answering specific questions, address them directly and precisely.
            Keep explanations brief but informative and don't ask for further explanations.
            Use markdown for clarity.
        "});

        let base_content = match &command.git_entity {
            GitEntity::Commit(commit) => {
                formatdoc! {"
                    Context - Commit:

                    Message: {msg}
                    Changes:
                    ```diff
                    {diff}
                    ```
                    ",
                    msg = commit.message,
                    diff = commit.diff
                }
            }
            GitEntity::Diff(Diff::WorkingTree { diff, .. } | Diff::CommitsRange { diff, .. }) => {
                formatdoc! {"
                    Context - Changes:

                    ```diff
                    {diff}
                    ```
                    "
                }
            }
        };

        let user_prompt = match &command.query {
            Some(query) => {
                formatdoc! {"
                    {base_content}

                    Question: {query}

                    Provide a focused answer to the question based on the changes shown above.
                    "
                }
            }
            None => match &command.git_entity {
                GitEntity::Commit(_) => formatdoc! {"
                    {base_content}
                    
                    Provide a short explanation covering:
                    1. Core changes made
                    2. Direct impact
                    "
                },
                GitEntity::Diff(Diff::WorkingTree { .. }) => formatdoc! {"
                    {base_content}
                    
                    Provide:
                    1. Key changes
                    2. Notable concerns (if any)
                    "
                },
                GitEntity::Diff(Diff::CommitsRange { .. }) => formatdoc! {"
                    {base_content}
                    
                    Provide:
                    1. Core changes made
                    2. Direct impact
                    "
                },
            },
        };

        Ok(AIPrompt {
            system_prompt,
            user_prompt,
        })
    }

}

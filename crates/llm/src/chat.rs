//! the chat template of TinyLlama 1.1B Chat v1.0 (the zephyr format), as its
//! `tokenizer_config.json` renders it with `add_generation_prompt`. the `<s>`
//! at the start is not here: the tokenizer's post-processor adds it.

/// One turn of a conversation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Role {
    /// Instructions for the assistant.
    System,
    /// The person asking.
    User,
    /// An earlier answer.
    Assistant,
}

impl Role {
    fn tag(self) -> &'static str {
        match self {
            Self::System => "<|system|>",
            Self::User => "<|user|>",
            Self::Assistant => "<|assistant|>",
        }
    }
}

/// The end-of-sequence marker the template closes each turn with.
pub const EOS: &str = "</s>";

/// Renders the turns and opens the assistant's answer.
pub fn render(turns: &[(Role, &str)]) -> String {
    let mut out = String::new();
    for (role, content) in turns {
        out.push_str(role.tag());
        out.push('\n');
        out.push_str(content);
        out.push_str(EOS);
        out.push('\n');
    }
    out.push_str(Role::Assistant.tag());
    out.push('\n');
    out
}

/// A single user question.
pub fn ask(question: &str) -> String {
    render(&[(Role::User, question)])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_question_opens_the_answer() {
        assert_eq!(ask("hi"), "<|user|>\nhi</s>\n<|assistant|>\n");
    }

    #[test]
    fn turns_keep_their_order() {
        let text = render(&[(Role::System, "be brief"), (Role::User, "2+2?")]);
        assert_eq!(
            text,
            "<|system|>\nbe brief</s>\n<|user|>\n2+2?</s>\n<|assistant|>\n"
        );
    }

    #[test]
    fn no_turns_is_just_the_opening() {
        assert_eq!(render(&[]), "<|assistant|>\n");
    }
}

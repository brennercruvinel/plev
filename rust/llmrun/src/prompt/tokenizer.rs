//! text in and out: the tokenizer of the model directory, the incremental
//! detokenizer the stream uses, and the [`Engine`] that puts the chat
//! template, the tokenizer and a [`Model`] behind one call.

use std::path::Path;
use std::sync::{Arc, Mutex};

use tokenizers::Tokenizer;

use crate::stream::{self, Detok, Model, Request, Stream};
use crate::{Info, LlmError, prompt::chat};

/// The model's tokenizer (`tokenizer.json`).
#[derive(Clone)]
pub struct Text {
    tok: Arc<Tokenizer>,
    eos: u32,
}

impl Text {
    /// Loads `tokenizer.json` from the model directory.
    pub fn from_dir(dir: &Path) -> Result<Self, LlmError> {
        let path = dir.join("tokenizer.json");
        let bytes = std::fs::read(&path).map_err(|e| LlmError::file(&path, e))?;
        Self::from_json(&bytes)
    }

    /// Loads a tokenizer from the bytes of a `tokenizer.json`.
    pub fn from_json(bytes: &[u8]) -> Result<Self, LlmError> {
        let tok = Tokenizer::from_bytes(bytes).map_err(|e| LlmError::Tokenizer(e.to_string()))?;
        let eos = tok
            .token_to_id(chat::EOS)
            .ok_or_else(|| LlmError::Tokenizer(format!("no {} in the vocabulary", chat::EOS)))?;
        Ok(Self {
            tok: Arc::new(tok),
            eos,
        })
    }

    /// Text to ids, with the special tokens the post-processor adds.
    pub fn encode(&self, text: &str) -> Result<Vec<u32>, LlmError> {
        let enc = self
            .tok
            .encode(text, true)
            .map_err(|e| LlmError::Tokenizer(e.to_string()))?;
        Ok(enc.get_ids().to_vec())
    }

    /// Ids to text, without the special tokens.
    pub fn decode(&self, ids: &[u32]) -> Result<String, LlmError> {
        self.tok
            .decode(ids, true)
            .map_err(|e| LlmError::Tokenizer(e.to_string()))
    }

    /// The end-of-sequence id.
    pub fn eos(&self) -> u32 {
        self.eos
    }

    /// A fresh incremental detokenizer.
    pub fn detok(&self) -> TextDetok {
        TextDetok {
            text: self.clone(),
            ids: Vec::new(),
            emitted: 0,
        }
    }
}

/// Decodes the whole answer at each token and hands out what is new, so a
/// character split across byte tokens comes out whole, once.
pub struct TextDetok {
    text: Text,
    ids: Vec<u32>,
    emitted: usize,
}

impl Detok for TextDetok {
    fn push(&mut self, id: u32) -> String {
        self.ids.push(id);
        let Ok(full) = self.text.decode(&self.ids) else {
            return String::new();
        };
        if full.ends_with(char::REPLACEMENT_CHARACTER) || !full.is_char_boundary(self.emitted) {
            return String::new();
        }
        let piece = full[self.emitted..].to_string();
        self.emitted = full.len();
        piece
    }
}

/// A model with its tokenizer: a question in, a stream of tokens out.
pub struct Engine<M: Model> {
    model: Arc<Mutex<M>>,
    text: Text,
}

impl<M: Model + 'static> Engine<M> {
    /// Puts a loaded model and its tokenizer together.
    pub fn new(model: M, text: Text) -> Self {
        Self {
            model: Arc::new(Mutex::new(model)),
            text,
        }
    }

    /// The tokenizer.
    pub fn text(&self) -> &Text {
        &self.text
    }

    /// The model's report; waits for a running generation to end.
    pub fn info(&self) -> Result<Info, LlmError> {
        Ok(self.model.lock().map_err(|_| LlmError::Poisoned)?.info())
    }

    /// Templates the question, tokenizes it and starts the answer.
    pub fn ask(&self, question: &str, max_new: usize) -> Result<Stream, LlmError> {
        let prompt = self.text.encode(&chat::ask(question))?;
        Ok(self.generate(prompt, max_new))
    }

    /// Starts an answer from prompt ids already templated and tokenized.
    pub fn generate(&self, prompt: Vec<u32>, max_new: usize) -> Stream {
        let req = Request {
            prompt,
            max_new,
            eos: vec![self.text.eos],
        };
        stream::spawn(self.model.clone(), req, Box::new(self.text.detok()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A word-level tokenizer with a byte-fallback decoder, the same decoder
    /// family as TinyLlama's: "é" is two byte tokens.
    const TOKENIZER: &str = r#"{
      "version": "1.0",
      "truncation": null,
      "padding": null,
      "added_tokens": [
        {"id": 1, "content": "</s>", "single_word": false, "lstrip": false,
         "rstrip": false, "normalized": false, "special": true}
      ],
      "normalizer": null,
      "pre_tokenizer": {"type": "Whitespace"},
      "post_processor": null,
      "decoder": {"type": "Sequence", "decoders": [
        {"type": "ByteFallback"}, {"type": "Fuse"}
      ]},
      "model": {"type": "WordLevel", "unk_token": "<unk>", "vocab": {
        "<unk>": 0, "</s>": 1, "caf": 2, "<0xC3>": 3, "<0xA9>": 4, "!": 5
      }}
    }"#;

    fn text() -> Text {
        Text::from_json(TOKENIZER.as_bytes()).unwrap()
    }

    #[test]
    fn the_eos_id_comes_from_the_vocabulary() {
        assert_eq!(text().eos(), 1);
    }

    #[test]
    fn a_split_character_comes_out_whole_once() {
        let mut detok = text().detok();
        assert_eq!(detok.push(2), "caf");
        assert_eq!(detok.push(3), "", "half of the é waits");
        assert_eq!(detok.push(4), "é");
        assert_eq!(detok.push(5), "!");
    }

    #[test]
    fn special_tokens_are_not_printed() {
        let mut detok = text().detok();
        assert_eq!(detok.push(2), "caf");
        assert_eq!(detok.push(1), "");
    }

    #[test]
    fn a_broken_tokenizer_is_an_error_not_a_panic() {
        assert!(matches!(
            Text::from_json(b"{not json"),
            Err(LlmError::Tokenizer(_))
        ));
        let missing = Text::from_dir(Path::new("/nonexistent/plev-llm"));
        assert!(matches!(missing, Err(LlmError::File { .. })));
    }
}

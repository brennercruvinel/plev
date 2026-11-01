//! The prompt input as a reader and an agent meet it: its parts declared
//! under its id, what they ask run on the widget ([`PromptInput::act`]),
//! and the focus the keyboard and a reader move ([`PromptInput::focus`]).

use accord::{Constraints, NodeBuilder, ParamSpec, Role, SemId, SnapshotBuilder, Value, ValueKind};

use super::{FILES, PromptInput};

/// A part of the prompt input a reader can land on.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Part {
    Text,
    Add,
    File(usize),
    Setting,
    SettingOption(usize),
    Model,
    Choice(usize),
    Voice,
    Submit,
    Prompt(usize),
    Surprise,
    Category(usize),
    /// The view controller's arrows: back (`false`) and forward.
    Orbit(bool),
}

impl PromptInput {
    fn child(&self, name: &str) -> Option<SemId> {
        self.sem.as_ref()?.child(name).ok()
    }

    fn indexed(&self, name: &str, i: usize) -> Option<SemId> {
        self.child(name).map(|f| f.key(i.to_string()))
    }

    pub(super) fn file_id(&self, i: usize) -> Option<SemId> {
        self.indexed("files", i)
    }

    pub(super) fn premade_id(&self, i: usize) -> Option<SemId> {
        self.indexed("premade", i)
    }

    /// The id of `part`.
    pub fn part_id(&self, part: Part) -> Option<SemId> {
        match part {
            Part::Text => self.child("text"),
            Part::Add => self.child("add"),
            Part::File(i) => self.file_id(i),
            Part::Setting => self.child("setting"),
            Part::SettingOption(i) => self.indexed("setting", i),
            Part::Model => self.child("model"),
            Part::Choice(i) => self.indexed("model", i),
            Part::Voice => self.child("voice"),
            Part::Submit => self.child("submit"),
            Part::Prompt(i) => self.premade_id(i),
            Part::Surprise => self.child("surprise"),
            Part::Category(i) => self.indexed("categories", i),
            Part::Orbit(forward) => {
                let view = self.child("view")?;
                view.child(if forward { "forward" } else { "back" }).ok()
            }
        }
    }

    /// The part `id` names, if it is one of this prompt's.
    pub fn part_of(&self, id: &SemId) -> Option<Part> {
        let root = self.sem.as_ref()?;
        let rest = id.as_str().strip_prefix(root.as_str())?.strip_prefix('.')?;
        let (name, index) = match rest.split_once('[') {
            Some((name, tail)) => (name, tail.strip_suffix(']')?.parse::<usize>().ok()),
            None => (rest, None),
        };
        Some(match (name, index) {
            ("text", None) => Part::Text,
            ("add", None) => Part::Add,
            ("files", Some(i)) => Part::File(i),
            ("setting", None) => Part::Setting,
            ("setting", Some(i)) => Part::SettingOption(i),
            ("model", None) => Part::Model,
            ("model", Some(i)) => Part::Choice(i),
            ("voice", None) => Part::Voice,
            ("submit", None) => Part::Submit,
            ("premade", Some(i)) => Part::Prompt(i),
            ("surprise", None) => Part::Surprise,
            ("categories", Some(i)) => Part::Category(i),
            ("view.back", None) => Part::Orbit(false),
            ("view.forward", None) => Part::Orbit(true),
            _ => return None,
        })
    }

    /// The part the keyboard or a reader is on.
    pub fn focused(&self) -> Option<Part> {
        if self.line.is_focused() {
            return Some(Part::Text);
        }
        if let Some(i) = self.files_focus {
            return Some(Part::File(i));
        }
        if let Some(i) = self.model.focused() {
            return Some(Part::Choice(i));
        }
        if let Some(i) = self
            .setting
            .highlighted()
            .filter(|_| self.setting.is_open())
        {
            return Some(Part::SettingOption(i));
        }
        let premade = self.premade.as_ref();
        if let Some(i) = premade.and_then(|p| p.focused) {
            return Some(Part::Prompt(i));
        }
        let pressed = [
            (self.add.is_focused(), Part::Add),
            (self.setting.is_focused(), Part::Setting),
            (self.model_press.focused, Part::Model),
            (self.voice_button.is_focused(), Part::Voice),
            (self.submit_focused, Part::Submit),
            (
                premade.is_some_and(|p| p.surprise.is_focused()),
                Part::Surprise,
            ),
        ];
        if let Some((_, part)) = pressed.iter().find(|(on, _)| *on) {
            return Some(*part);
        }
        premade
            .and_then(|p| p.categories.iter().position(|b| b.is_focused()))
            .map(Part::Category)
    }

    /// Moves the focus to `part`, or lets it go.
    pub fn focus(&mut self, part: Option<Part>) {
        let is = |p: Part| part == Some(p);
        self.line.set_focused(is(Part::Text) && !self.voice);
        self.add.set_focused(is(Part::Add));
        self.files_focus = match part {
            Some(Part::File(i)) if self.files_open => Some(i),
            _ => None,
        };
        self.setting.set_focused(is(Part::Setting));
        if let Some(Part::SettingOption(i)) = part {
            self.setting.set_highlighted(Some(i));
        }
        self.model_press.focused = is(Part::Model) && !self.voice;
        self.model.set_focused(match part {
            Some(Part::Choice(i)) if self.model_open => Some(i),
            _ => None,
        });
        self.voice_button.set_focused(is(Part::Voice));
        self.submit.set_focused(is(Part::Submit));
        self.submit_focused = is(Part::Submit);
        if let Some(p) = &mut self.premade {
            p.focused = match part {
                Some(Part::Prompt(i)) if i < p.prompts.len() => Some(i),
                _ => None,
            };
            p.surprise.set_focused(is(Part::Surprise));
            for (i, b) in p.categories.iter_mut().enumerate() {
                b.set_focused(is(Part::Category(i)));
            }
        }
    }

    /// Runs `action` on `part`, as a click on it would (or, with `param`,
    /// as typing or a choice). `Err` with why when the part does not take
    /// it.
    pub fn act(&mut self, part: Part, action: &str, param: Option<&Value>) -> Result<(), String> {
        let text = match param {
            Some(Value::Text(t)) => Some(t.as_str()),
            _ => None,
        };
        let done = match (part, action, text) {
            (Part::Text, "set_value", Some(t)) => self.line.set_value(t),
            (Part::Add, "activate", _) => self.toggle_files(),
            (Part::File(i), "activate", _) => self.files_open && self.add_file(i),
            (Part::Setting, "open", _) => {
                self.setting.open();
                self.setting.is_open()
            }
            (Part::Setting, "close", _) => {
                self.setting.close();
                true
            }
            (Part::Setting, "select", Some(t)) => !self.voice && self.setting.select_label(t),
            (Part::SettingOption(i), "select", _) => !self.voice && self.setting.select_index(i),
            (Part::Model, "open", _) => self.set_models_open(true),
            (Part::Model, "close", _) => self.set_models_open(false),
            (Part::Model, "select", Some(t)) => {
                let i = self.model.choices().iter().position(|c| c.title == t);
                i.is_some_and(|i| self.choose_model(i))
            }
            (Part::Choice(i), "select", _) => self.choose_model(i),
            (Part::Voice, "toggle", _) => {
                self.toggle_voice();
                true
            }
            (Part::Submit, "activate", _) => {
                self.submit();
                true
            }
            (Part::Prompt(i), "activate", _) => !self.voice && self.use_prompt(i),
            (Part::Surprise, "activate", _) => !self.voice && self.surprise(),
            // a category presses, as in the kit
            (Part::Category(_), "activate", _) => !self.voice,
            // the arrows press, as the kit's
            (Part::Orbit(_), "activate", _) => self.view.is_some(),
            _ => false,
        };
        if done {
            Ok(())
        } else {
            Err(format!("the prompt input does not {action} {part:?} now"))
        }
    }

    /// Declares the prompt as `id`: a `Role::Group` named `label` with
    /// its `voice` state, the files `added` and what it `sent`; under it
    /// the line (a text input, while it shows), the add button (with
    /// `expanded`, and its menu of three items while open), the setting
    /// (a select), the model (a select whose options are the models while
    /// open), the voice toggle and the submit button; over it the premade
    /// prompts (a list with its `count`, each item with `activate`), the
    /// `Surprise me` button and the categories. What voice input leaves
    /// out is disabled. Returns the group's node.
    #[track_caller]
    pub fn project<'b>(
        &self,
        b: &'b mut SnapshotBuilder,
        id: &SemId,
        label: impl Into<String>,
        parent: Option<&SemId>,
    ) -> NodeBuilder<'b> {
        let on = !self.voice;
        if let (Some(tid), true) = (self.part_id(Part::Text), self.shows_line()) {
            self.line.project(b, &tid, "Prompt", Some(id)).enabled(on);
        }
        if let Some(aid) = self.part_id(Part::Add) {
            b.node(&aid, Role::Button, "Add files")
                .parent(id)
                .state("expanded", self.files_open)
                .enabled(on)
                .action("activate")
                .reads(format!("{aid}:expanded"));
            if self.files_open {
                if let Some(menu) = self.child("files") {
                    b.node(&menu, Role::Menu, "Add files").parent(&aid);
                    for (i, (_, label)) in FILES.iter().enumerate() {
                        let item = menu.key(i.to_string());
                        b.node(&item, Role::MenuItem, *label)
                            .parent(&menu)
                            .action("activate")
                            .reads(format!("{id}:added"));
                    }
                }
            }
        }
        if let Some(sid) = self.part_id(Part::Setting) {
            self.setting.project(b, &sid, "Setting", Some(id));
        }
        if let Some(mid) = self.part_id(Part::Model) {
            self.project_model(b, &mid, id);
        }
        if let Some(vid) = self.part_id(Part::Voice) {
            b.node(&vid, Role::Toggle, "Voice input")
                .parent(id)
                .state("on", self.voice)
                .action("toggle")
                .reads(format!("{vid}:on"));
        }
        if let Some(sid) = self.part_id(Part::Submit) {
            self.submit
                .project(b, &sid, "Send", Some(id))
                .description("send the prompt");
        }
        if let (Some(p), true) = (&self.premade, on) {
            if let Some(list) = self.child("premade") {
                b.node(&list, Role::List, "Premade prompts")
                    .parent(id)
                    .state("count", i64::try_from(p.prompts.len()).unwrap_or(i64::MAX));
                let reads = self
                    .part_id(Part::Text)
                    .filter(|_| self.shows_line())
                    .map(|t| format!("{t}:text"));
                for (i, prompt) in p.prompts.iter().enumerate() {
                    let mut item = b.node(list.key(i.to_string()), Role::ListItem, prompt.as_str());
                    item.parent(&list).focusable(true);
                    let mut action = item.action("activate");
                    if let Some(reads) = &reads {
                        action.reads(reads.as_str());
                    }
                }
            }
            if let Some(sid) = self.part_id(Part::Surprise) {
                p.surprise.project(b, &sid, Some(id));
            }
            if let Some(group) = self.child("categories") {
                b.node(&group, Role::Group, "Categories").parent(id);
                for chip in &p.categories {
                    if let Some(cid) = chip.sem_id() {
                        chip.project(b, cid, Some(&group));
                    }
                }
            }
        }
        if let (Some(v), Some(vid)) = (&self.view, self.child("view")) {
            v.project(b, &vid, "View", Some(id));
        }
        let mut node = b.node(id, Role::Group, label);
        if let Some(parent) = parent {
            node.parent(parent);
        }
        node.state("voice", self.voice)
            .state("added", i64::try_from(self.added).unwrap_or(i64::MAX))
            .state("sent", self.sent.clone().unwrap_or_default());
        node
    }

    /// The model's trigger as a select (the kit's listbox): its chosen
    /// title, `expanded`, its titles as the choices, `select`, `open` and
    /// `close`; its options while open.
    fn project_model(&self, b: &mut SnapshotBuilder, mid: &SemId, parent: &SemId) {
        let on = !self.voice;
        if self.model_open {
            self.model.project_options(b, mid, on);
        }
        let titles = self
            .model
            .choices()
            .iter()
            .map(|c| c.title.clone())
            .collect();
        let selected = format!("{mid}:selected");
        let expanded = format!("{mid}:expanded");
        let mut node = b.node(mid, Role::Select, "AI model");
        node.parent(parent)
            .state("selected", self.model.chosen_title().unwrap_or_default())
            .state("expanded", self.model_open)
            .constraints(Constraints {
                editable: true,
                options: Some(titles),
                ..Constraints::default()
            })
            .enabled(on);
        node.action("select")
            .param(ParamSpec {
                kind: ValueKind::Text,
                constrained: true,
            })
            .reads(selected.as_str());
        node.action("open").reads(expanded.as_str());
        node.action("close").reads(expanded.as_str());
    }
}

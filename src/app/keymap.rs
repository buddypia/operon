//! The keyboard, as data rather than as literals inside the frame loop.
//!
//! The remappable actions have ids. A file names them and gives each a chord;
//! what is not named keeps its default. The whole of the decision — parsing, conflict,
//! and every way a file can be wrong — is `resolve_keymap`, which takes a
//! string and returns either a map or the sentence that says what is wrong with
//! it. Nothing in that path touches a disk or a window, because a keymap that
//! can only be tested by starting the application is one nobody will change.
//!
//! The palette draws each row's chord from the same map the frame loop looks
//! actions up in. Before this module, two palette labels had their shortcut
//! typed into the Japanese string, which is a place the application told a
//! person which key to press and was under no obligation to be right.

use crate::*;

/// The macOS spelling of the modifiers, which is what the menu bar of every
/// other application on this machine draws. They are not icons: the icon
/// vocabulary in `src/glyphs.rs` is asserted to resolve from the Phosphor face
/// and not from the text face, and these are the other way round. Spelled once,
/// here, for the same reason the icon vocabulary exists.
const CHORD_COMMAND: char = '⌘';
const CHORD_SHIFT: char = '⇧';
const CHORD_ALT: char = '⌥';
const CHORD_CONTROL: char = '⌃';

/// The modifier names the file uses. `Mod` is the platform's own command
/// modifier, which is what a keymap wants to say — the file is not the place to
/// learn that this build is macOS.
const NAME_MOD: &str = "Mod";
const NAME_SHIFT: &str = "Shift";
const NAME_ALT: &str = "Alt";
const NAME_CONTROL: &str = "Ctrl";

/// One thing the keyboard can do, its id, and the chord it has unless a file
/// says otherwise.
///
/// The id is the contract with the file on disk. It is the one string here that
/// a person outside this repository writes down, so it does not change without
/// leaving the old spelling readable.
pub(crate) struct KeymapAction {
    pub(crate) id: &'static str,
    /// A message id, translated where it is drawn.
    pub(crate) label: &'static str,
    pub(crate) default: &'static str,
}

/// Every remappable action, in the order the palette draws them.
///
/// Terminal keys are deliberately absent. A TUI owns its own keyboard, and the
/// keys that reach a pane are not this application's to reassign. So are the
/// palette's own navigation keys: arrows, Enter, and Escape belong to the
/// widget that has focus.
pub(crate) const KEYMAP_ACTIONS: &[KeymapAction] = &[
    KeymapAction {
        id: "palette.open",
        label: "検索・操作を開く",
        default: "Mod+K",
    },
    KeymapAction {
        id: "session.new",
        label: "新しいセッション",
        default: "Mod+N",
    },
    KeymapAction {
        id: "attention.next",
        label: "次の要対応セッションへ",
        default: "Mod+J",
    },
    KeymapAction {
        id: "page.home",
        label: "ホームを開く",
        default: "Mod+1",
    },
    KeymapAction {
        id: "page.projects",
        label: "プロジェクトを開く",
        default: "Mod+2",
    },
    KeymapAction {
        id: "page.sessions",
        label: "セッションを開く",
        default: "Mod+3",
    },
    KeymapAction {
        id: "page.settings",
        label: "設定を開く",
        default: "Mod+,",
    },
    KeymapAction {
        id: "project.add",
        label: "プロジェクトフォルダを選択…",
        default: "Mod+O",
    },
    KeymapAction {
        id: "terminal.find",
        label: "ターミナル内を検索",
        default: "Mod+F",
    },
    KeymapAction {
        id: "panel.toggle",
        label: "サイドパネルを開閉",
        default: "Mod+Alt+B",
    },
    // The tab keys every browser, editor, and terminal on a Mac answers to
    // (change 272). ⌘T opens a plain terminal in the focused pane: the + menu's
    // ターミナル row without the menu (change 273), under the name the + had.
    KeymapAction {
        id: "tab.new",
        label: "空のターミナルを開く",
        default: "Mod+T",
    },
    KeymapAction {
        id: "tab.close",
        label: "タブを閉じる",
        default: "Mod+W",
    },
    KeymapAction {
        id: "tab.reopen",
        label: "閉じたタブを開き直す",
        default: "Mod+Shift+T",
    },
    KeymapAction {
        id: "tab.next",
        label: "次のタブへ",
        default: "Mod+Shift+]",
    },
    KeymapAction {
        id: "tab.previous",
        label: "前のタブへ",
        default: "Mod+Shift+[",
    },
];

/// The frame loop's half of the keyboard: which action this frame's key
/// presses trigger, and what each does. Beside the map it reads so that the
/// matching rule and the dispatch that depends on it are read together
/// (change 272, lesson 112).
impl OperonApp {
    /// The app-wide chords, read once per frame before anything draws.
    pub(crate) fn handle_app_shortcuts(&mut self, ctx: &egui::Context) {
        // The launch sheet holds a request being typed. ⌘K opens a palette
        // that can switch project, and ⌘2 clears the project outright: either
        // would throw the request away, so neither is live while it is open.
        if !self.launch_sheet_open && self.chord_pressed(ctx, "palette.open") {
            self.open_command_palette();
        }
        // Before the window draws, so the selection it paints is the one the
        // arrows just moved — and so an arrow never reaches the text field and
        // moves the caret instead.
        self.handle_palette_keys(ctx);
        // Adding a project selects it, which clears the sheet's request too.
        if !self.launch_sheet_open && self.chord_pressed(ctx, "project.add") {
            self.choose_project_folder();
        }
        // The shortcuts a person brings with them from every other Mac app of
        // this shape: ⌘1–3 for the three places, ⌘, for settings, unless a
        // keymap file says otherwise. They stay live while a terminal has
        // focus: a TUI owns its own keys, but `⌘` is the one modifier macOS
        // keeps for the app, and the pane drops every ⌘ chord rather than
        // sending it (`terminal_key_binding`). That is what lets ⌘J leave the
        // pane a person is typing in for the next one that is waiting.
        if !self.command_palette_open && !self.launch_sheet_open {
            if self.chord_pressed(ctx, "session.new") {
                self.open_new_session();
            }
            if self.chord_pressed(ctx, "attention.next") {
                self.open_next_waiting_session();
            }
            if self.chord_pressed(ctx, "panel.toggle") {
                self.show_session_inspector = !self.show_session_inspector;
            }
            // ⌘T is the strip's +, and like ⌘N it answers from any page.
            if self.chord_consumed(ctx, "tab.new") {
                self.launch_workspace_terminal(self.workspace.focused);
            }
            if self.chord_consumed(ctx, "tab.reopen") {
                self.reopen_closed_tab();
            }
            // The rest act on the tabs on screen, so only where they are. They
            // are consumed here, before any pane draws, so a terminal never
            // receives ⌃Tab and nothing later sees the ⌘ chords again.
            if self.page == Page::Sessions && !self.session_library_open {
                if self.chord_consumed(ctx, "tab.close") {
                    if let Some(tab) = self.workspace.active().cloned() {
                        self.close_workspace_tab(&tab);
                    }
                }
                if self.chord_consumed(ctx, "tab.previous") {
                    self.step_workspace_tab(false);
                }
                if self.chord_consumed(ctx, "tab.next") {
                    self.step_workspace_tab(true);
                }
                for forward in self.consume_fixed_control_tab(ctx) {
                    self.step_workspace_tab(forward);
                }
            }
            for (action, page) in [
                ("page.home", Page::Home),
                ("page.projects", Page::Projects),
                ("page.sessions", Page::Sessions),
                ("page.settings", Page::Settings),
            ] {
                if self.chord_pressed(ctx, action) {
                    if page == Page::Projects {
                        self.select_project(None);
                    }
                    self.page = page;
                }
            }
        }
    }

    /// Whether this frame saw the chord bound to an action. An action with no
    /// chord is never pressed, which is what `null` in the keymap file buys.
    pub(crate) fn chord_pressed(&self, ctx: &egui::Context, action: &str) -> bool {
        ctx.input(|input| {
            input
                .events
                .iter()
                .any(|event| self.event_triggers(event, action))
        })
    }

    /// The same question, for a chord that must not also reach whatever has
    /// focus. Removing the event is the whole of "the agent never sees it".
    pub(crate) fn chord_consumed(&self, ctx: &egui::Context, action: &str) -> bool {
        ctx.input_mut(|input| {
            let before = input.events.len();
            input
                .events
                .retain(|event| !self.event_triggers(event, action));
            input.events.len() != before
        })
    }

    /// ⌃Tab and ⌃⇧Tab, fixed beside the remappable ⇧⌘] and ⇧⌘[ as every
    /// browser keeps them: one step per press, `true` forward. A press the
    /// keymap file gave to an action is that action's and is left alone, so a
    /// file that moves `tab.previous` onto ⌃Tab gets what it asked for. Shift
    /// is read off the press rather than through egui's lenient match, under
    /// which ⌃Tab swallowed ⌃⇧Tab and stepped forward.
    fn consume_fixed_control_tab(&self, ctx: &egui::Context) -> Vec<bool> {
        let mut steps = Vec::new();
        ctx.input_mut(|input| {
            input.events.retain(|event| match event {
                egui::Event::Key {
                    key: egui::Key::Tab,
                    pressed: true,
                    modifiers,
                    ..
                } if modifiers.ctrl
                    && !modifiers.command
                    && self.keymap.action_for(egui::Key::Tab, *modifiers).is_none() =>
                {
                    steps.push(!modifiers.shift);
                    false
                }
                _ => true,
            });
        });
        steps
    }

    /// Whether one input event is a press of `action`'s chord — decided by
    /// `Keymap::action_for`, so a press that is a more specific chord's is not
    /// this one's (change 272).
    fn event_triggers(&self, event: &egui::Event, action: &str) -> bool {
        matches!(
            event,
            egui::Event::Key {
                key,
                pressed: true,
                modifiers,
                ..
            } if self.keymap.action_for(*key, *modifiers) == Some(action)
        )
    }
}

/// A chord, compared the way "the same chord" means.
///
/// `egui::Modifiers` derives `PartialEq` over its own fields, so two chords are
/// equal when the same modifiers are held and the same key is down. That is the
/// identity a conflict is decided on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Chord {
    pub(crate) modifiers: egui::Modifiers,
    pub(crate) key: egui::Key,
}

/// The keys this build answers to, in `KEYMAP_ACTIONS` order.
///
/// `None` is an action with no key at all, which is what a file says with
/// `null`. An id that is not in the registry cannot be in here: the map is
/// built from the registry and the file only ever replaces entries.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Keymap {
    bindings: Vec<(&'static str, Option<Chord>)>,
}

impl Keymap {
    /// The keymap with nothing overridden. Every default must parse, which is
    /// asserted rather than assumed — a default that did not parse would leave
    /// an action silently unbound in a build that shipped.
    pub(crate) fn defaults() -> Self {
        Self {
            bindings: KEYMAP_ACTIONS
                .iter()
                .map(|action| (action.id, parse_chord(action.default)))
                .collect(),
        }
    }

    pub(crate) fn chord_for(&self, id: &str) -> Option<Chord> {
        self.bindings
            .iter()
            .find(|(action, _)| *action == id)
            .and_then(|(_, chord)| *chord)
    }

    /// What the palette draws at the right of a row: the chord in the macOS
    /// spelling, or nothing when the action has no key.
    pub(crate) fn label_for(&self, id: &str) -> String {
        self.chord_for(id).map(chord_label).unwrap_or_default()
    }

    /// The one action a key press triggers, if any.
    ///
    /// egui matches a chord leniently — an extra Shift or Alt held is ignored —
    /// so ⇧⌘T matches ⌘T as well as itself. Lenient matching is worth keeping:
    /// on a JIS keyboard `=` needs Shift, and `Mod+=` should still answer. So
    /// a press goes to the binding whose Shift and Alt match exactly, and only
    /// a press with no exact binding falls back to the first lenient one, which
    /// is what every chord did before change 272. The conflict check in
    /// `resolve_keymap` already treats ⌘T and ⇧⌘T as two chords; this is the
    /// dispatch agreeing with it.
    pub(crate) fn action_for(
        &self,
        key: egui::Key,
        modifiers: egui::Modifiers,
    ) -> Option<&'static str> {
        let mut lenient = None;
        for (id, chord) in &self.bindings {
            let Some(chord) = chord else { continue };
            if !key_matches(chord.key, key, modifiers.shift)
                || !modifiers.matches_logically(chord.modifiers)
            {
                continue;
            }
            if modifiers.matches_exact(chord.modifiers) {
                return Some(id);
            }
            lenient = lenient.or(Some(*id));
        }
        lenient
    }
}

/// Whether a pressed key is the key a chord names.
///
/// With ⌘ held, macOS reports the character Shift makes, not the key: ⇧⌘]
/// arrives as `}` (winit 0.30's `create_key_event` keeps Shift's effect, and
/// egui-winit 0.31 sends that logical key). `[` and `]` are the pair whose
/// shifted characters are the same on US and JIS keyboards, so they are the
/// pair read through Shift. Other shifted symbols differ by layout — ⇧; is `:`
/// on one and `+` on the other — and guessing one would be wrong on the other.
fn key_matches(chord_key: egui::Key, pressed: egui::Key, shift: bool) -> bool {
    chord_key == pressed
        || (shift
            && matches!(
                (chord_key, pressed),
                (egui::Key::OpenBracket, egui::Key::OpenCurlyBracket)
                    | (egui::Key::CloseBracket, egui::Key::CloseCurlyBracket)
            ))
}

/// A chord from the file's spelling, or `None` when it is not one.
///
/// The grammar is the modifiers and then exactly one key, `+`-separated, with
/// surrounding spaces ignored so that `Mod + K` is not a different answer from
/// `Mod+K`. A trailing `+` leaves no key and is refused rather than read as a
/// chord with an empty name.
pub(crate) fn parse_chord(chord: &str) -> Option<Chord> {
    let mut parts = chord
        .split('+')
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>();
    let key = key_from_name(parts.pop()?)?;
    let mut modifiers = egui::Modifiers::NONE;
    for part in parts {
        match part {
            // `command` is the field `Modifiers::COMMAND` sets and the one
            // `consume_key` reads, which is what makes a parsed chord and a
            // chord written in the source the same value.
            NAME_MOD => modifiers.command = true,
            NAME_SHIFT => modifiers.shift = true,
            NAME_ALT => modifiers.alt = true,
            NAME_CONTROL => modifiers.ctrl = true,
            _ => return None,
        }
    }
    // `Mod+Shift` and `Mod+` both leave a modifier, or nothing, where the key
    // should be. `parts.pop()` above has already refused them: neither name is
    // in the key table.
    Some(Chord { modifiers, key })
}

/// The chord as macOS writes it: the glyphs in the menu bar's own order,
/// adjacent, with no separators.
pub(crate) fn chord_label(chord: Chord) -> String {
    let mut label = String::new();
    if chord.modifiers.ctrl {
        label.push(CHORD_CONTROL);
    }
    if chord.modifiers.alt {
        label.push(CHORD_ALT);
    }
    if chord.modifiers.shift {
        label.push(CHORD_SHIFT);
    }
    if chord.modifiers.command || chord.modifiers.mac_cmd {
        label.push(CHORD_COMMAND);
    }
    label.push_str(key_name(chord.key));
    label
}

/// The glyphs `chord_label` can draw, for the guard that asserts the bundled UI
/// font has all of them. Four empty boxes in the palette would be a worse
/// answer than no chord at all.
#[cfg(test)]
pub(crate) const CHORD_MODIFIER_GLYPHS: &[char] =
    &[CHORD_CONTROL, CHORD_ALT, CHORD_SHIFT, CHORD_COMMAND];

/// The keys a chord may name. Deliberately short: these are application
/// shortcuts, not a terminal's keyboard, and a name that is not here is a
/// finding rather than a key that quietly does nothing.
fn key_table() -> &'static [(&'static str, egui::Key)] {
    &[
        ("A", egui::Key::A),
        ("B", egui::Key::B),
        ("C", egui::Key::C),
        ("D", egui::Key::D),
        ("E", egui::Key::E),
        ("F", egui::Key::F),
        ("G", egui::Key::G),
        ("H", egui::Key::H),
        ("I", egui::Key::I),
        ("J", egui::Key::J),
        ("K", egui::Key::K),
        ("L", egui::Key::L),
        ("M", egui::Key::M),
        ("N", egui::Key::N),
        ("O", egui::Key::O),
        ("P", egui::Key::P),
        ("Q", egui::Key::Q),
        ("R", egui::Key::R),
        ("S", egui::Key::S),
        ("T", egui::Key::T),
        ("U", egui::Key::U),
        ("V", egui::Key::V),
        ("W", egui::Key::W),
        ("X", egui::Key::X),
        ("Y", egui::Key::Y),
        ("Z", egui::Key::Z),
        ("0", egui::Key::Num0),
        ("1", egui::Key::Num1),
        ("2", egui::Key::Num2),
        ("3", egui::Key::Num3),
        ("4", egui::Key::Num4),
        ("5", egui::Key::Num5),
        ("6", egui::Key::Num6),
        ("7", egui::Key::Num7),
        ("8", egui::Key::Num8),
        ("9", egui::Key::Num9),
        (",", egui::Key::Comma),
        (".", egui::Key::Period),
        ("/", egui::Key::Slash),
        (";", egui::Key::Semicolon),
        ("[", egui::Key::OpenBracket),
        ("]", egui::Key::CloseBracket),
        ("\\", egui::Key::Backslash),
        ("-", egui::Key::Minus),
        ("=", egui::Key::Equals),
        ("'", egui::Key::Quote),
        ("`", egui::Key::Backtick),
        ("Space", egui::Key::Space),
        ("Tab", egui::Key::Tab),
        ("Enter", egui::Key::Enter),
    ]
}

fn key_from_name(name: &str) -> Option<egui::Key> {
    key_table()
        .iter()
        .find(|(spelling, _)| *spelling == name)
        .map(|(_, key)| *key)
}

fn key_name(key: egui::Key) -> &'static str {
    key_table()
        .iter()
        .find(|(_, candidate)| *candidate == key)
        .map(|(spelling, _)| *spelling)
        // Unreachable through `parse_chord`, which only produces keys from this
        // same table. Named rather than panicked: a keymap is not worth a crash.
        .unwrap_or("?")
}

/// The keymap a file asks for, or the Japanese sentence saying why it cannot be
/// had.
///
/// A file that contradicts itself is refused whole. Half a keymap is worse than
/// none: the person cannot tell which half took, and the fix is one edit
/// either way.
pub(crate) fn resolve_keymap(contents: &str) -> std::result::Result<Keymap, String> {
    let document = serde_json::from_str::<serde_json::Value>(contents)
        .map_err(|_| tr("JSON として読めません。").to_owned())?;
    let Some(object) = document.as_object() else {
        return Err(tr("いちばん外側が JSON のオブジェクトではありません。").into());
    };
    let bindings = match object.get("keybindings") {
        Some(serde_json::Value::Object(bindings)) => bindings.clone(),
        // An absent section is a file that binds nothing, which is a file that
        // asks for the defaults.
        None => serde_json::Map::new(),
        Some(_) => {
            return Err(tr("keybindings が JSON のオブジェクトではありません。").into());
        }
    };
    let mut keymap = Keymap::defaults();
    for (id, value) in &bindings {
        let Some(entry) = keymap
            .bindings
            .iter_mut()
            .find(|(action, _)| *action == id.as_str())
        else {
            return Err(tf!("「{id}」という操作はありません。", id = id));
        };
        entry.1 = match value {
            serde_json::Value::Null => None,
            serde_json::Value::String(chord) => Some(parse_chord(chord).ok_or_else(|| {
                tf!("「{chord}」はキーの書き方として読めません。", chord = chord)
            })?),
            _ => {
                return Err(tf!("「{id}」の値は文字列か null にしてください。", id = id));
            }
        };
    }
    // The conflict is read off the finished map rather than off the file, so a
    // binding that collides with a default it did not mention is caught too.
    for (index, (id, chord)) in keymap.bindings.iter().enumerate() {
        let Some(chord) = chord else { continue };
        if let Some((other, _)) = keymap.bindings[index + 1..]
            .iter()
            .find(|(_, candidate)| candidate.as_ref() == Some(chord))
        {
            return Err(tf!(
                "{chord} が「{first}」と「{second}」で重複しています。",
                chord = chord_label(*chord),
                first = tr(action_label(id)),
                second = tr(action_label(other))
            ));
        }
    }
    Ok(keymap)
}

/// The message id for an action, for the sentences that have to name one.
///
/// Falling back to the id is unreachable through a `Keymap`, whose ids all came
/// from the registry. It is also the only honest answer if it ever happens: an
/// id is at least a thing a person can search their own file for.
pub(crate) fn action_label(id: &'static str) -> &'static str {
    KEYMAP_ACTIONS
        .iter()
        .find(|action| action.id == id)
        .map(|action| action.label)
        .unwrap_or(id)
}

/// The file the keymap is read from, beside the store and the sidecars rather
/// than in a second home of its own.
pub(crate) fn keymap_path() -> PathBuf {
    app_data_directory().join(KEYMAP_FILE_NAME)
}

/// The keymap on disk, or the defaults plus the sentence that says why.
///
/// An absent file is not a problem and gets no sentence: it is what a machine
/// that has never been told otherwise looks like.
pub(crate) fn read_keymap() -> (Keymap, Option<String>) {
    let path = keymap_path();
    let Ok(metadata) = fs::metadata(&path) else {
        return (Keymap::defaults(), None);
    };
    if metadata.len() > KEYMAP_FILE_MAX_BYTES {
        return (
            Keymap::defaults(),
            Some(tf!(
                "キー割り当てファイルを読み込めませんでした: {p0} バイトを超えています。既定のキーで起動しました。",
                p0 = KEYMAP_FILE_MAX_BYTES
            )),
        );
    }
    let Ok(contents) = fs::read_to_string(&path) else {
        return (
            Keymap::defaults(),
            Some(tr("キー割り当てファイルを読み込めませんでした: テキストとして読めません。既定のキーで起動しました。").into()),
        );
    };
    keymap_or_defaults(&contents)
}

/// The keymap a file's contents ask for, and the sentence a person reads when
/// they cannot have it. The disk half above is three lines of `fs`; this is
/// where the answer is actually decided, so this is what the tests hold.
pub(crate) fn keymap_or_defaults(contents: &str) -> (Keymap, Option<String>) {
    match resolve_keymap(contents) {
        Ok(keymap) => (keymap, None),
        Err(reason) => (
            Keymap::defaults(),
            Some(tf!(
                "キー割り当てファイルを読み込めませんでした: {reason}既定のキーで起動しました。",
                reason = reason
            )),
        ),
    }
}

/// The defaults as a file, written only when there is no file to lose.
///
/// `Ok(false)` is "one was already there", which is the ordinary answer the
/// second time somebody asks for it. The write goes through
/// `write_file_atomically`, so a crash in the middle cannot leave half a keymap
/// where a whole one is expected.
pub(crate) fn write_default_keymap() -> std::result::Result<bool, String> {
    write_default_keymap_at(&keymap_path())
}

/// The seam the test writes through, so that "never overwrites" is asserted
/// against a real file rather than against a reading of this function.
pub(crate) fn write_default_keymap_at(path: &Path) -> std::result::Result<bool, String> {
    if path.exists() {
        return Ok(false);
    }
    let mut bindings = serde_json::Map::new();
    for action in KEYMAP_ACTIONS {
        bindings.insert(
            action.id.to_owned(),
            serde_json::Value::String(action.default.to_owned()),
        );
    }
    let document = serde_json::json!({
        "version": 1,
        "keybindings": serde_json::Value::Object(bindings),
    });
    let body = serde_json::to_string_pretty(&document).map_err(|error| error.to_string())? + "\n";
    // The same write every other file this application owns gets: a temporary
    // name, both the file and its directory synced, then the rename. This was
    // written by hand at first, which was one atomic-write implementation too
    // many for one repository to hold.
    write_file_atomically(path, body.as_bytes()).map_err(|error| error.to_string())?;
    Ok(true)
}

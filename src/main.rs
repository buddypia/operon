use crate::prelude::*;

mod agents;
mod app;
mod cli;
mod config;
mod exec;
mod files;
mod git;
mod glyphs;
mod history;
mod i18n;
mod i18n_tables;
mod markdown;
mod models;
mod prelude;
mod store;
mod sys;
#[cfg(test)]
mod tests;
mod theme;
mod tmux;
mod transcript;
mod ui;
mod util;

pub(crate) use agents::*;
pub(crate) use app::*;
pub(crate) use cli::*;
pub(crate) use config::*;
pub(crate) use exec::*;
pub(crate) use files::*;
pub(crate) use git::*;
pub(crate) use glyphs::*;
pub(crate) use history::*;
pub(crate) use markdown::*;
pub(crate) use models::*;
pub(crate) use store::*;
pub(crate) use sys::*;
pub(crate) use theme::*;
pub(crate) use tmux::*;
pub(crate) use transcript::*;
pub(crate) use ui::*;
pub(crate) use util::*;

fn main() -> eframe::Result<()> {
    configure_command_path();
    let data_file = app_data_file();
    let instance_locks = match acquire_instance_locks(&data_file) {
        Ok(locks) => locks,
        Err(error) => {
            let (title, guidance) = match &error {
                InstanceLockError::AlreadyRunning(_) => (
                    tr("Operon は既に開いています"),
                    tr("もう一方の Operon を閉じてから、もう一度お試しください。"),
                ),
                InstanceLockError::Unavailable(_) => (
                    tr("Operon はローカルデータを開けませんでした"),
                    tr("データフォルダの権限と空き容量を確認してから、もう一度お試しください。"),
                ),
            };
            let _ = rfd::MessageDialog::new()
                .set_title(title)
                .set_description(tf!(
                    "Operon はローカルデータを安全に開けませんでした。{guidance}\n\n{error}",
                    error = error,
                    guidance = guidance
                ))
                .set_level(rfd::MessageLevel::Warning)
                .set_buttons(rfd::MessageButtons::Ok)
                .show();
            return Ok(());
        }
    };
    let migration = match import_legacy_store_if_available(&data_file) {
        Ok(outcome) => outcome,
        Err(error) => {
            let _ = rfd::MessageDialog::new()
                .set_title(tr("Operon は既存データを取り込めませんでした"))
                .set_description(tf!("既存データは変更していません。データフォルダの権限と空き容量を確認してから、もう一度お試しください。\n\n{error}", error = error))
                .set_level(rfd::MessageLevel::Error)
                .set_buttons(rfd::MessageButtons::Ok)
                .show();
            return Ok(());
        }
    };
    let options = native_options();
    eframe::run_native(
        APP_NAME,
        options,
        Box::new(move |_creation| {
            Ok(Box::new(OperonApp::new_with_instance_locks(
                instance_locks,
                migration,
            )))
        }),
    )
}

fn native_options() -> eframe::NativeOptions {
    eframe::NativeOptions {
        // Eframe restores the raw global coordinates from the last monitor.
        // Those coordinates can point above or beside the current desktop
        // after a display is disconnected, making the app appear to vanish.
        persist_window: false,
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 820.0])
            .with_min_inner_size([760.0, 540.0])
            // The toolbar *is* the title bar. macOS keeps the traffic lights
            // and the drag region where a person expects them, and the app
            // stops spending a 28px strip on repeating the name that is
            // already in the Dock, the menu bar, and the window itself.
            // `ui_topbar` reserves `TRAFFIC_LIGHT_INSET` on its left for the
            // three buttons that now sit inside it.
            .with_fullsize_content_view(true)
            .with_titlebar_shown(false)
            .with_title_shown(false)
            // The bundle's `Operon.icns` is the only source of the app icon.
            // Left unset, eframe hands its own bundled egui logo to
            // `NSApplication.setApplicationIconImage`, so the Dock tile of the
            // running app stops matching the bundle it was launched from. The
            // empty `IconData` is eframe's documented way to ask for the
            // platform icon instead.
            .with_icon(egui::IconData::default()),
        ..Default::default()
    }
}

fn configure_macos_fonts(ctx: &egui::Context, terminal_font: TerminalFont) {
    let mut fonts = egui::FontDefinitions::default();

    // These copies are bundled so a Finder-launched app has the same readable
    // Japanese, Korean, and English terminal font as a shell-launched app.
    // Sarasa Mono is distributed under the SIL Open Font License 1.1; see
    // assets/fonts/LICENSE-SARASA.txt.
    for (name, data) in [
        (
            "sarasa-mono-j",
            include_bytes!("../assets/fonts/SarasaMonoJ-Regular.ttf").as_slice(),
        ),
        (
            "sarasa-mono-k",
            include_bytes!("../assets/fonts/SarasaMonoK-Regular.ttf").as_slice(),
        ),
        (
            "sarasa-term-j",
            include_bytes!("../assets/fonts/SarasaTermJ-Regular.ttf").as_slice(),
        ),
        (
            "sarasa-fixed-j",
            include_bytes!("../assets/fonts/SarasaFixedJ-Regular.ttf").as_slice(),
        ),
        (
            "sarasa-ui-j",
            include_bytes!("../assets/fonts/SarasaUiJ-Regular.ttf").as_slice(),
        ),
    ] {
        fonts
            .font_data
            .insert(name.to_owned(), egui::FontData::from_static(data).into());
    }

    // The icon vocabulary's own face. `add_to_fonts` also splices it into the
    // proportional family, which `proportional_ui_font_chain` then replaces
    // wholesale — that chain is what decides the order, and it names `phosphor`
    // itself.
    egui_phosphor::add_to_fonts(&mut fonts, egui_phosphor::Variant::Regular);

    if let Some(fonts_in_family) = fonts.families.get_mut(&egui::FontFamily::Monospace) {
        fonts_in_family.insert(0, terminal_font.fallback_font_name().to_owned());
        fonts_in_family.insert(0, terminal_font.font_name().to_owned());
    }

    for (name, path, family, first) in [
        (
            "macos-monospace",
            "/System/Library/Fonts/SFNSMono.ttf",
            egui::FontFamily::Monospace,
            false,
        ),
        (
            "macos-symbols",
            "/System/Library/Fonts/Apple Symbols.ttf",
            egui::FontFamily::Proportional,
            false,
        ),
    ] {
        let Ok(bytes) = fs::read(path) else {
            continue;
        };
        fonts
            .font_data
            .insert(name.to_owned(), egui::FontData::from_owned(bytes).into());
        if let Some(fonts_in_family) = fonts.families.get_mut(&family) {
            if first {
                fonts_in_family.insert(0, name.to_owned());
            } else {
                fonts_in_family.push(name.to_owned());
            }
        }
    }
    if let Some(fonts_in_family) = fonts.families.get_mut(&egui::FontFamily::Proportional) {
        // Every ordinary UI glyph must come from the *same* face. Keeping SF
        // and Hiragino in this chain lets egui substitute individual Japanese
        // glyphs and produces the baseline drift visible in mixed text.
        // Sarasa UI J contains Japanese, Korean, Latin, and common UI marks;
        // Apple Symbols is intentionally retained only as a final icon fallback.
        *fonts_in_family = proportional_ui_font_chain();
    }
    ctx.set_fonts(fonts);
}

/// Text first, icons second. Phosphor only claims private-use codepoints, so it
/// can never take a letter away from Sarasa; putting it ahead of Apple Symbols
/// keeps every icon on one stroke weight instead of letting the system face
/// answer for whichever mark Phosphor happens not to define.
fn proportional_ui_font_chain() -> Vec<String> {
    vec![
        "sarasa-ui-j".to_owned(),
        "phosphor".to_owned(),
        "macos-symbols".to_owned(),
    ]
}

fn configure_command_path() {
    let mut paths = std::env::var_os("PATH")
        .map(|path| std::env::split_paths(&path).collect::<Vec<_>>())
        .unwrap_or_default();
    for candidate in command_path_candidates() {
        if candidate.is_dir() && !paths.iter().any(|path| path == &candidate) {
            paths.push(candidate);
        }
    }
    if let Ok(path) = std::env::join_paths(paths) {
        // SAFETY: this runs once on the main thread before eframe or any worker thread starts.
        unsafe { std::env::set_var("PATH", path) };
    }
}

fn command_path_candidates() -> Vec<PathBuf> {
    let mut candidates = vec![
        PathBuf::from("/opt/homebrew/bin"),
        PathBuf::from("/usr/local/bin"),
    ];
    if let Some(home) = std::env::var_os("HOME").map(PathBuf::from) {
        candidates.push(home.join(".local").join("bin"));
        candidates.push(home.join(".cargo").join("bin"));
    }
    candidates
}

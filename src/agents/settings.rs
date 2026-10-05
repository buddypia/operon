//! Per-agent recent launch settings.
//!
//! When launching a session, users configure reasoning depth, permission flags,
//! target model, and execution mode. Rather than forcing re-entry on every new
//! session, project switch, or app restart, Operon records the most recently
//! configured options per agent in `recent-agent-settings.json` beside the store.

use crate::*;

/// The remembered launch options for one CLI agent.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct AgentLaunchSettings {
    #[serde(default)]
    pub(crate) model: String,
    #[serde(default)]
    pub(crate) mode: String,
    #[serde(default)]
    pub(crate) effort: String,
    #[serde(default)]
    pub(crate) flags: Vec<String>,
    #[serde(default)]
    pub(crate) custom_command: String,
}

/// The collection of recent launch options for all agents, plus the last
/// selected agent.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct RecentAgentSettings {
    #[serde(default)]
    pub(crate) last_selected_agent: Option<String>,
    #[serde(default)]
    pub(crate) agents: HashMap<String, AgentLaunchSettings>,
}

pub(crate) fn recent_agent_settings_path(data_file: &Path) -> PathBuf {
    data_file
        .parent()
        .unwrap_or(Path::new("."))
        .join(RECENT_AGENT_SETTINGS_FILE_NAME)
}

/// A file that cannot be read or cannot be parsed returns empty settings, which
/// defaults gracefully to standard CLI behavior.
pub(crate) fn load_recent_agent_settings(data_file: &Path) -> RecentAgentSettings {
    let path = recent_agent_settings_path(data_file);
    match fs::metadata(&path) {
        Ok(metadata) if metadata.len() <= RECENT_AGENT_SETTINGS_FILE_MAX_BYTES => {}
        _ => return RecentAgentSettings::default(),
    }
    let mut settings = fs::read(&path)
        .ok()
        .and_then(|raw| serde_json::from_slice::<RecentAgentSettings>(&raw).ok())
        .unwrap_or_default();

    // Sanitize loaded settings against currently known options for each agent.
    for (agent, launch_settings) in settings.agents.iter_mut() {
        sanitize_agent_launch_settings(agent, launch_settings);
    }

    settings
}

const MAX_FIELD_STRING_LEN: usize = 1024;
const MAX_CUSTOM_COMMAND_LEN: usize = 4096;

/// Atomically write recent agent settings to disk.
pub(crate) fn save_recent_agent_settings(
    data_file: &Path,
    settings: &RecentAgentSettings,
) -> Result<()> {
    let path = recent_agent_settings_path(data_file);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut buffer = Vec::with_capacity(1024);
    serde_json::to_writer_pretty(&mut buffer, settings)?;
    buffer.push(b'\n');
    write_file_atomically(&path, &buffer)?;
    Ok(())
}

/// Validate and prune obsolete, conflicting, or malformed options for an agent.
pub(crate) fn sanitize_agent_launch_settings(agent: &str, settings: &mut AgentLaunchSettings) {
    if agent == "custom" {
        settings.custom_command = settings
            .custom_command
            .chars()
            .take(MAX_CUSTOM_COMMAND_LEN)
            .collect::<String>()
            .trim()
            .to_owned();
        settings.model.clear();
        settings.mode.clear();
        settings.effort.clear();
        settings.flags.clear();
        return;
    }

    settings.custom_command.clear();

    settings.model = settings
        .model
        .chars()
        .take(MAX_FIELD_STRING_LEN)
        .collect::<String>()
        .trim()
        .to_owned();

    // Verify mode is known.
    let modes = agent_mode_options(agent);
    let trimmed_mode = settings.mode.trim();
    if !trimmed_mode.is_empty() && modes.contains(&trimmed_mode) {
        settings.mode = trimmed_mode.to_owned();
    } else {
        settings.mode.clear();
    }

    // Verify effort is known.
    let efforts = agent_effort_options(agent);
    let trimmed_effort = settings.effort.trim();
    if !trimmed_effort.is_empty() && efforts.contains(&trimmed_effort) {
        settings.effort = trimmed_effort.to_owned();
    } else {
        settings.effort.clear();
    }

    // Filter flags to only known ones that do not conflict with the chosen mode.
    let mut seen_groups: HashSet<String> = HashSet::new();
    let mut seen_flags: HashSet<String> = HashSet::new();
    let mut sanitized_flags = Vec::new();

    for id in &settings.flags {
        let trimmed_id = id.trim();
        if seen_flags.contains(trimmed_id) {
            continue;
        }
        if let Some(flag) = agent_flag(agent, trimmed_id) {
            if flag.conflicts_with_mode && !settings.mode.is_empty() {
                continue;
            }
            if !flag.group.is_empty() {
                if seen_groups.contains(flag.group) {
                    continue;
                }
                seen_groups.insert(flag.group.to_owned());
            }
            seen_flags.insert(trimmed_id.to_owned());
            sanitized_flags.push(trimmed_id.to_owned());
        }
    }

    settings.flags = sanitized_flags;
}

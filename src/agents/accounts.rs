//! Which login a session runs under.
//!
//! Codex and Claude Code each keep credentials, settings, and history in one
//! directory, and each reads a variable saying which directory that is. An
//! account here is a name and a directory — nothing is read out of it, nothing
//! is copied into it beyond the managed hooks the person already asked for, and
//! nothing about it leaves the machine.
//!
//! The part that is easy to get wrong is not the variable. It is that the
//! managed hooks from change 015 and the usage reader from change 025 live
//! *inside* that directory, so a session pointed somewhere else finds none of
//! them and goes quiet — the state chip stops moving, turns stop being marked
//! unread, and the usage chip disappears. `account_roots` is what the hook
//! installer walks so that does not happen.

use crate::*;

/// One login, as this application knows it: a name a person chose and a
/// directory they had already logged into.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct AgentAccount {
    pub(crate) id: Uuid,
    /// The agent kind, as the store spells it — `"codex"` or `"claude"`.
    pub(crate) agent: String,
    pub(crate) name: String,
    pub(crate) path: PathBuf,
}

/// Which session ran under which account.
///
/// Reopening a conversation has to reopen it under the login that had it: a
/// work conversation resumed under a personal account is either a refusal from
/// the provider or, worse, a second conversation nobody meant to start.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct SessionAccount {
    pub(crate) session: Uuid,
    pub(crate) account: Uuid,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct AgentAccounts {
    #[serde(default)]
    pub(crate) accounts: Vec<AgentAccount>,
    #[serde(default)]
    pub(crate) sessions: Vec<SessionAccount>,
}

/// The variable this agent reads to be told where its login lives, or `None`
/// for an agent that has none.
pub(crate) fn account_variable(agent: &str) -> Option<&'static str> {
    match agent {
        "claude" => Some(CLAUDE_ACCOUNT_ENV),
        "codex" => Some(CODEX_ACCOUNT_ENV),
        _ => None,
    }
}

/// Whether this agent can be pointed at a login at all. The launch screen draws
/// its account row only for the ones that can.
pub(crate) fn agent_supports_accounts(agent: &str) -> bool {
    account_variable(agent).is_some()
}

impl AgentAccounts {
    pub(crate) fn get(&self, id: Uuid) -> Option<&AgentAccount> {
        self.accounts.iter().find(|account| account.id == id)
    }

    pub(crate) fn for_agent(&self, agent: &str) -> Vec<&AgentAccount> {
        self.accounts
            .iter()
            .filter(|account| account.agent == agent)
            .collect()
    }

    pub(crate) fn account_of_session(&self, session: Uuid) -> Option<Uuid> {
        self.sessions
            .iter()
            .find(|record| record.session == session)
            .map(|record| record.account)
    }

    /// Remember which login a session ran under, replacing any earlier answer
    /// for the same session. `None` forgets it, which is what launching under
    /// the machine's own login means.
    pub(crate) fn remember_session(&mut self, session: Uuid, account: Option<Uuid>) {
        self.sessions.retain(|record| record.session != session);
        if let Some(account) = account {
            self.sessions.push(SessionAccount { session, account });
        }
    }

    /// Drop what nothing points at any more: a record for a session that is no
    /// longer in the store, and a record for an account that has been removed.
    /// Without this the file is append-only for the life of the machine.
    pub(crate) fn prune(&mut self, live_sessions: &[Uuid]) {
        let accounts = self
            .accounts
            .iter()
            .map(|account| account.id)
            .collect::<Vec<_>>();
        self.sessions.retain(|record| {
            live_sessions.contains(&record.session) && accounts.contains(&record.account)
        });
    }
}

/// The `KEY=VALUE` pairs a launch under this account adds to the ones the hook
/// environment already carries. Empty for the machine's own login, which is the
/// launch every build before this one made.
pub(crate) fn account_variables(
    accounts: &AgentAccounts,
    account: Option<Uuid>,
) -> Vec<(&'static str, String)> {
    let Some(account) = account.and_then(|id| accounts.get(id)) else {
        return Vec::new();
    };
    account_variable(&account.agent)
        .map(|variable| vec![(variable, account.path.display().to_string())])
        .unwrap_or_default()
}

/// Add an account, or say why it cannot be added.
///
/// The path is checked here rather than at launch because a refusal a person
/// reads while choosing a folder is worth more than one they read when a
/// terminal fails to open. It is checked again at launch anyway — a folder can
/// be moved between the two moments.
pub(crate) fn register_account(
    accounts: &mut AgentAccounts,
    agent: &str,
    name: &str,
    path: &Path,
) -> std::result::Result<Uuid, String> {
    if !agent_supports_accounts(agent) {
        return Err(tf!(
            "{agent} にはアカウントを指定する方法がありません。",
            agent = agent
        ));
    }
    let name = name.trim();
    if name.is_empty() {
        return Err(tr("アカウントの名前を入力してください。").into());
    }
    if !path.is_dir() {
        return Err(tf!(
            "{path} はフォルダではありません。",
            path = path.display()
        ));
    }
    // Two names for one directory is two rows that cannot be told apart
    // afterwards, and a usage reading that is counted twice.
    if let Some(existing) = accounts
        .accounts
        .iter()
        .find(|account| account.path == path)
    {
        return Err(tf!(
            "このフォルダは「{name}」として既に登録されています。",
            name = existing.name.clone()
        ));
    }
    if accounts
        .accounts
        .iter()
        .any(|account| account.agent == agent && account.name == name)
    {
        return Err(tf!("「{name}」という名前は既にあります。", name = name));
    }
    let id = Uuid::new_v4();
    accounts.accounts.push(AgentAccount {
        id,
        agent: agent.to_owned(),
        name: name.to_owned(),
        path: path.to_owned(),
    });
    Ok(id)
}

/// Whether a launch under this account can go ahead, and why not when it
/// cannot.
///
/// Checked before tmux opens a window: a window that opens, prints a failure,
/// and dies leaves a session record behind for a run that never started.
pub(crate) fn account_ready_to_launch(
    accounts: &AgentAccounts,
    account: Option<Uuid>,
) -> std::result::Result<(), String> {
    let Some(account) = account else {
        return Ok(());
    };
    let Some(account) = accounts.get(account) else {
        return Err(tr("選ばれたアカウントが見つかりません。").into());
    };
    if !account.path.is_dir() {
        return Err(tf!(
            "アカウント「{name}」のフォルダが見つかりません。",
            name = account.name.clone()
        ));
    }
    Ok(())
}

pub(crate) fn agent_accounts_path(data_file: &Path) -> PathBuf {
    data_file
        .parent()
        .unwrap_or(Path::new("."))
        .join(AGENT_ACCOUNTS_FILE_NAME)
}

/// A file that cannot be read or cannot be parsed means no accounts, which is
/// the launch every build before this one made. The other direction would be a
/// corrupt file deciding which login a session runs under.
pub(crate) fn load_agent_accounts(data_file: &Path) -> AgentAccounts {
    let path = agent_accounts_path(data_file);
    match fs::metadata(&path) {
        Ok(metadata) if metadata.len() <= AGENT_ACCOUNTS_FILE_MAX_BYTES => {}
        _ => return AgentAccounts::default(),
    }
    fs::read(&path)
        .ok()
        .and_then(|raw| serde_json::from_slice::<AgentAccounts>(&raw).ok())
        .unwrap_or_default()
}

pub(crate) fn save_agent_accounts(data_file: &Path, accounts: &AgentAccounts) -> Result<()> {
    let encoded = serde_json::to_string_pretty(accounts)?;
    write_file_atomically(
        &agent_accounts_path(data_file),
        format!("{encoded}\n").as_bytes(),
    )?;
    Ok(())
}

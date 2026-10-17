//! A per-project knowledge base distilled from agent conversations (change 157).
//!
//! A project opts in. Each run reads that project's Claude Code, Codex, and
//! Antigravity conversations past the point the last run reached, ranks their
//! exchanges locally with EmbeddingGemma 2, hands the few that rank highest to
//! the CLI that held the conversation to judge and summarise, and keeps what it
//! returns as short entries — merged with an existing entry when they say the
//! same thing, faded and removed when nobody uses them. Everything lives in one
//! private SQLite file per project under the app data directory, never in the
//! project's working tree: conversations hold secrets.

pub(crate) mod embed;

pub(crate) use embed::*;

use crate::*;
use rusqlite::{params, OptionalExtension};

// ---------------------------------------------------------------------------
// Where it lives.

pub(crate) fn knowledge_root() -> PathBuf {
    app_data_directory().join(KNOWLEDGE_DIRECTORY_NAME)
}

pub(crate) fn knowledge_database_path(root: &Path, project_id: Uuid) -> PathBuf {
    root.join(format!("{project_id}.{KNOWLEDGE_DATABASE_EXTENSION}"))
}

pub(crate) fn knowledge_judge_directory(root: &Path) -> PathBuf {
    root.join(KNOWLEDGE_JUDGE_DIRECTORY_NAME)
}

pub(crate) const KNOWLEDGE_DIRECTORY_MODE: u32 = 0o700;
pub(crate) const KNOWLEDGE_FILE_MODE: u32 = 0o600;

fn create_private_knowledge_directory(directory: &Path) -> std::io::Result<()> {
    fs::create_dir_all(directory)?;
    fs::set_permissions(
        directory,
        fs::Permissions::from_mode(KNOWLEDGE_DIRECTORY_MODE),
    )
}

/// Create the database file 0600 before SQLite opens it; SQLite gives the
/// `-wal` and `-shm` files it creates the mode of the database.
fn create_private_knowledge_file(path: &Path) -> std::io::Result<()> {
    OpenOptions::new()
        .create(true)
        .append(true)
        .mode(KNOWLEDGE_FILE_MODE)
        .open(path)?;
    fs::set_permissions(path, fs::Permissions::from_mode(KNOWLEDGE_FILE_MODE))
}

// ---------------------------------------------------------------------------
// What an entry is.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum KnowledgeKind {
    Decision,
    Cause,
    Procedure,
    Pitfall,
    Fact,
}

impl KnowledgeKind {
    pub(crate) const ALL: [Self; 5] = [
        Self::Decision,
        Self::Cause,
        Self::Procedure,
        Self::Pitfall,
        Self::Fact,
    ];

    /// The word the judge answers with and the database stores.
    pub(crate) fn key(self) -> &'static str {
        match self {
            Self::Decision => "decision",
            Self::Cause => "cause",
            Self::Procedure => "procedure",
            Self::Pitfall => "pitfall",
            Self::Fact => "fact",
        }
    }

    pub(crate) fn from_key(key: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|kind| kind.key() == key)
    }

    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Decision => tr("決定"),
            Self::Cause => tr("原因"),
            Self::Procedure => tr("手順"),
            Self::Pitfall => tr("注意"),
            Self::Fact => tr("事実"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct KnowledgeSource {
    pub(crate) provider: CliProvider,
    pub(crate) session_id: String,
    pub(crate) transcript_path: PathBuf,
    pub(crate) exchange_index: usize,
    pub(crate) recorded_at: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct KnowledgeEntry {
    pub(crate) id: i64,
    pub(crate) kind: KnowledgeKind,
    pub(crate) title: String,
    pub(crate) body: String,
    pub(crate) created_at: u64,
    pub(crate) reinforced_at: u64,
    pub(crate) referenced_at: u64,
    pub(crate) reference_count: u32,
    pub(crate) recurrences: u32,
    pub(crate) pinned: bool,
    pub(crate) sources: Vec<KnowledgeSource>,
}

impl KnowledgeEntry {
    pub(crate) fn freshness(&self, now: u64) -> f64 {
        freshness(
            self.created_at
                .max(self.reinforced_at)
                .max(self.referenced_at),
            self.reference_count,
            self.recurrences,
            now,
        )
    }
}

/// `exp(−Δt / τ)` with `τ = 30 days × (1 + ln(1 + references + recurrences))`:
/// an entry nobody uses is gone below 0.10 after 69 days; each use stretches
/// how long the next quiet spell may last.
pub(crate) fn freshness(last_used: u64, reference_count: u32, recurrences: u32, now: u64) -> f64 {
    let days = now.saturating_sub(last_used) as f64 / 86_400.0;
    let uses = f64::from(reference_count) + f64::from(recurrences);
    let tau = KNOWLEDGE_TAU_DAYS * (1.0 + (1.0 + uses).ln());
    (-days / tau).exp()
}

pub(crate) fn provider_key(provider: CliProvider) -> &'static str {
    provider.agent()
}

pub(crate) fn provider_from_key(key: &str) -> Option<CliProvider> {
    [CliProvider::Claude, CliProvider::Codex, CliProvider::Gemini]
        .into_iter()
        .find(|provider| provider.agent() == key)
}

// ---------------------------------------------------------------------------
// The database.

/// What a conversation's row says about whether the next run reads it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ConversationState {
    Pending,
    TooLarge,
    Unreadable,
    Failed,
}

impl ConversationState {
    fn key(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::TooLarge => "too-large",
            Self::Unreadable => "unreadable",
            Self::Failed => "failed",
        }
    }

    fn from_key(key: &str) -> Self {
        match key {
            "too-large" => Self::TooLarge,
            "unreadable" => Self::Unreadable,
            "failed" => Self::Failed,
            _ => Self::Pending,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ConversationRow {
    pub(crate) provider: CliProvider,
    pub(crate) session_id: String,
    pub(crate) transcript_path: PathBuf,
    pub(crate) mtime: u64,
    pub(crate) size: u64,
    pub(crate) cursor: usize,
    pub(crate) state: ConversationState,
    pub(crate) attempts: u32,
    pub(crate) processed_at: u64,
    pub(crate) paused_at: Option<u64>,
    pub(crate) title: Option<String>,
    pub(crate) indexed: bool,
}

const KNOWLEDGE_SCHEMA: &str = "
CREATE TABLE IF NOT EXISTS meta (key TEXT PRIMARY KEY, value TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS entries (
    id INTEGER PRIMARY KEY,
    kind TEXT NOT NULL,
    title TEXT NOT NULL,
    body TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    reinforced_at INTEGER NOT NULL,
    referenced_at INTEGER NOT NULL,
    reference_count INTEGER NOT NULL DEFAULT 0,
    recurrences INTEGER NOT NULL DEFAULT 0,
    pinned INTEGER NOT NULL DEFAULT 0,
    vector BLOB NOT NULL
);
CREATE TABLE IF NOT EXISTS sources (
    id INTEGER PRIMARY KEY,
    entry_id INTEGER NOT NULL REFERENCES entries(id) ON DELETE CASCADE,
    provider TEXT NOT NULL,
    session_id TEXT NOT NULL,
    transcript_path TEXT NOT NULL,
    exchange_index INTEGER NOT NULL,
    recorded_at INTEGER NOT NULL,
    text TEXT NOT NULL,
    vector BLOB NOT NULL
);
CREATE INDEX IF NOT EXISTS sources_by_entry ON sources(entry_id);
CREATE TABLE IF NOT EXISTS sessions (
    provider TEXT NOT NULL,
    session_id TEXT NOT NULL,
    transcript_path TEXT NOT NULL,
    mtime INTEGER NOT NULL,
    size INTEGER NOT NULL,
    cursor INTEGER NOT NULL DEFAULT 0,
    state TEXT NOT NULL DEFAULT 'pending',
    attempts INTEGER NOT NULL DEFAULT 0,
    processed_at INTEGER NOT NULL DEFAULT 0,
    paused_at INTEGER,
    title TEXT,
    text TEXT,
    vector BLOB,
    PRIMARY KEY (provider, session_id)
);
CREATE TABLE IF NOT EXISTS judge_calls (day INTEGER PRIMARY KEY, count INTEGER NOT NULL);
";

/// Whether a judged entry may be added while `pinned` entries are pinned: a
/// base whose every slot is pinned only reinforces (Req 15).
pub(crate) fn may_add_entries(pinned: usize) -> bool {
    pinned < KNOWLEDGE_MAX_ENTRIES
}

pub(crate) struct KnowledgeDatabase {
    connection: rusqlite::Connection,
}

fn sql_error(error: rusqlite::Error) -> anyhow::Error {
    anyhow!(tf!(
        "ナレッジのデータベースを読み書きできませんでした: {error}",
        error = error
    ))
}

impl KnowledgeDatabase {
    /// Open (creating when absent) one project's database. A database written
    /// by a newer Operon is refused rather than read by rules it predates.
    pub(crate) fn open(path: &Path) -> Result<Self> {
        if let Some(directory) = path.parent() {
            create_private_knowledge_directory(directory)?;
        }
        create_private_knowledge_file(path)?;
        let connection = rusqlite::Connection::open(path).map_err(sql_error)?;
        connection
            .busy_timeout(Duration::from_secs(2))
            .map_err(sql_error)?;
        // The GUI's runs and the agents' `knowledge search` open one file.
        let _: String = connection
            .query_row("PRAGMA journal_mode=WAL", [], |row| row.get(0))
            .map_err(sql_error)?;
        connection
            .execute_batch("PRAGMA foreign_keys=ON;")
            .map_err(sql_error)?;
        connection
            .execute_batch(KNOWLEDGE_SCHEMA)
            .map_err(sql_error)?;
        let database = Self { connection };
        match database.meta("schema_version")? {
            None => database.set_meta("schema_version", &KNOWLEDGE_SCHEMA_VERSION.to_string())?,
            Some(version)
                if version.parse::<u32>().unwrap_or(u32::MAX) > KNOWLEDGE_SCHEMA_VERSION =>
            {
                return Err(anyhow!(tr(
                    "このナレッジは新しい Operon で作られています。Operon を更新してください。"
                )));
            }
            Some(_) => {}
        }
        if database.meta("model")?.is_none() {
            database.set_meta("model", EMBEDDING_MODEL_ID)?;
            database.set_meta("dimensions", &EMBEDDING_DIMENSIONS.to_string())?;
        }
        Ok(database)
    }

    /// Open only when the file is already there: reading a project's knowledge
    /// must not create a database for a project that never opted in.
    pub(crate) fn open_existing(path: &Path) -> Result<Option<Self>> {
        if !path.is_file() {
            return Ok(None);
        }
        Self::open(path).map(Some)
    }

    pub(crate) fn meta(&self, key: &str) -> Result<Option<String>> {
        self.connection
            .query_row(
                "SELECT value FROM meta WHERE key = ?1",
                params![key],
                |row| row.get(0),
            )
            .optional()
            .map_err(sql_error)
    }

    pub(crate) fn set_meta(&self, key: &str, value: &str) -> Result<()> {
        self.connection
            .execute(
                "INSERT INTO meta (key, value) VALUES (?1, ?2)
                 ON CONFLICT(key) DO UPDATE SET value = excluded.value",
                params![key, value],
            )
            .map(|_| ())
            .map_err(sql_error)
    }

    fn delete_meta(&self, key: &str) -> Result<()> {
        self.connection
            .execute("DELETE FROM meta WHERE key = ?1", params![key])
            .map(|_| ())
            .map_err(sql_error)
    }

    pub(crate) fn enabled(&self) -> Result<bool> {
        Ok(self.meta("enabled")?.as_deref() == Some("1"))
    }

    pub(crate) fn set_enabled(&self, enabled: bool) -> Result<()> {
        self.set_meta("enabled", if enabled { "1" } else { "0" })
    }

    pub(crate) fn model_matches(&self) -> Result<bool> {
        Ok(self.meta("model")?.as_deref() == Some(EMBEDDING_MODEL_ID))
    }

    #[cfg(test)]
    pub(crate) fn entry_count(&self) -> Result<usize> {
        self.count("SELECT COUNT(*) FROM entries")
    }

    /// Lets a test write what another model or an older run left behind.
    #[cfg(test)]
    pub(crate) fn connection_for_tests(&self) -> &rusqlite::Connection {
        &self.connection
    }

    pub(crate) fn pinned_count(&self) -> Result<usize> {
        self.count("SELECT COUNT(*) FROM entries WHERE pinned = 1")
    }

    pub(crate) fn failed_count(&self) -> Result<usize> {
        self.count("SELECT COUNT(*) FROM sessions WHERE state = 'failed'")
    }

    fn count(&self, query: &str) -> Result<usize> {
        self.connection
            .query_row(query, [], |row| row.get::<_, i64>(0))
            .map(|count| count.max(0) as usize)
            .map_err(sql_error)
    }

    pub(crate) fn judge_calls_on(&self, day: u64) -> Result<u32> {
        self.connection
            .query_row(
                "SELECT count FROM judge_calls WHERE day = ?1",
                params![day as i64],
                |row| row.get::<_, i64>(0),
            )
            .optional()
            .map(|count| count.unwrap_or(0).max(0) as u32)
            .map_err(sql_error)
    }

    pub(crate) fn record_judge_call(&self, day: u64) -> Result<()> {
        self.connection
            .execute(
                "INSERT INTO judge_calls (day, count) VALUES (?1, 1)
                 ON CONFLICT(day) DO UPDATE SET count = count + 1",
                params![day as i64],
            )
            .map(|_| ())
            .map_err(sql_error)
    }

    pub(crate) fn entries(&self) -> Result<Vec<KnowledgeEntry>> {
        let mut statement = self
            .connection
            .prepare(
                "SELECT id, kind, title, body, created_at, reinforced_at, referenced_at,
                        reference_count, recurrences, pinned
                 FROM entries ORDER BY id",
            )
            .map_err(sql_error)?;
        let rows = statement
            .query_map([], |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, i64>(4)?,
                    row.get::<_, i64>(5)?,
                    row.get::<_, i64>(6)?,
                    row.get::<_, i64>(7)?,
                    row.get::<_, i64>(8)?,
                    row.get::<_, i64>(9)?,
                ))
            })
            .map_err(sql_error)?
            .collect::<rusqlite::Result<Vec<_>>>()
            .map_err(sql_error)?;
        let mut sources = self.sources()?;
        Ok(rows
            .into_iter()
            .map(
                |(
                    id,
                    kind,
                    title,
                    body,
                    created,
                    reinforced,
                    referenced,
                    references,
                    recurrences,
                    pinned,
                )| {
                    KnowledgeEntry {
                        id,
                        kind: KnowledgeKind::from_key(&kind).unwrap_or(KnowledgeKind::Fact),
                        title,
                        body,
                        created_at: created.max(0) as u64,
                        reinforced_at: reinforced.max(0) as u64,
                        referenced_at: referenced.max(0) as u64,
                        reference_count: references.max(0) as u32,
                        recurrences: recurrences.max(0) as u32,
                        pinned: pinned != 0,
                        sources: sources.remove(&id).unwrap_or_default(),
                    }
                },
            )
            .collect())
    }

    fn sources(&self) -> Result<HashMap<i64, Vec<KnowledgeSource>>> {
        let mut statement = self
            .connection
            .prepare(
                "SELECT entry_id, provider, session_id, transcript_path, exchange_index, recorded_at
                 FROM sources ORDER BY id",
            )
            .map_err(sql_error)?;
        let rows = statement
            .query_map([], |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, i64>(4)?,
                    row.get::<_, i64>(5)?,
                ))
            })
            .map_err(sql_error)?
            .collect::<rusqlite::Result<Vec<_>>>()
            .map_err(sql_error)?;
        let mut by_entry: HashMap<i64, Vec<KnowledgeSource>> = HashMap::new();
        for (entry, provider, session_id, path, exchange, recorded) in rows {
            let Some(provider) = provider_from_key(&provider) else {
                continue;
            };
            by_entry.entry(entry).or_default().push(KnowledgeSource {
                provider,
                session_id,
                transcript_path: PathBuf::from(path),
                exchange_index: exchange.max(0) as usize,
                recorded_at: recorded.max(0) as u64,
            });
        }
        Ok(by_entry)
    }

    fn vectors(&self, query: &str) -> Result<Vec<(i64, Vec<f32>)>> {
        let mut statement = self.connection.prepare(query).map_err(sql_error)?;
        let rows = statement
            .query_map([], |row| {
                Ok((row.get::<_, i64>(0)?, row.get::<_, Vec<u8>>(1)?))
            })
            .map_err(sql_error)?
            .collect::<rusqlite::Result<Vec<_>>>()
            .map_err(sql_error)?;
        Ok(rows
            .into_iter()
            .filter_map(|(id, blob)| vector_from_blob(&blob).map(|vector| (id, vector)))
            .collect())
    }

    pub(crate) fn entry_vectors(&self) -> Result<Vec<(i64, Vec<f32>)>> {
        self.vectors("SELECT id, vector FROM entries")
    }

    fn source_vectors(&self) -> Result<Vec<(i64, Vec<f32>)>> {
        self.vectors("SELECT id, vector FROM sources")
    }

    pub(crate) fn conversation(
        &self,
        provider: CliProvider,
        session_id: &str,
    ) -> Result<Option<ConversationRow>> {
        self.connection
            .query_row(
                "SELECT provider, session_id, transcript_path, mtime, size, cursor, state,
                        attempts, processed_at, paused_at, title, vector IS NOT NULL
                 FROM sessions WHERE provider = ?1 AND session_id = ?2",
                params![provider_key(provider), session_id],
                conversation_row,
            )
            .optional()
            .map_err(sql_error)
    }

    pub(crate) fn save_conversation(&self, row: &ConversationRow) -> Result<()> {
        self.connection
            .execute(
                "INSERT INTO sessions (provider, session_id, transcript_path, mtime, size, cursor,
                                       state, attempts, processed_at, paused_at, title)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
                 ON CONFLICT(provider, session_id) DO UPDATE SET
                    transcript_path = excluded.transcript_path, mtime = excluded.mtime,
                    size = excluded.size, cursor = excluded.cursor, state = excluded.state,
                    attempts = excluded.attempts, processed_at = excluded.processed_at,
                    paused_at = excluded.paused_at, title = excluded.title",
                params![
                    provider_key(row.provider),
                    row.session_id,
                    row.transcript_path.to_string_lossy(),
                    row.mtime as i64,
                    row.size as i64,
                    row.cursor as i64,
                    row.state.key(),
                    row.attempts,
                    row.processed_at as i64,
                    row.paused_at.map(|at| at as i64),
                    row.title,
                ],
            )
            .map(|_| ())
            .map_err(sql_error)
    }

    pub(crate) fn index_conversation(
        &self,
        provider: CliProvider,
        session_id: &str,
        text: &str,
        vector: &[f32],
    ) -> Result<()> {
        self.connection
            .execute(
                "UPDATE sessions SET text = ?3, vector = ?4 WHERE provider = ?1 AND session_id = ?2",
                params![provider_key(provider), session_id, text, vector_to_blob(vector)],
            )
            .map(|_| ())
            .map_err(sql_error)
    }

    /// Every conversation the index can rank, with its vector.
    pub(crate) fn indexed_conversations(&self) -> Result<Vec<(ConversationRow, Vec<f32>)>> {
        let mut statement = self
            .connection
            .prepare(
                "SELECT provider, session_id, transcript_path, mtime, size, cursor, state,
                        attempts, processed_at, paused_at, title, vector IS NOT NULL, vector
                 FROM sessions WHERE vector IS NOT NULL",
            )
            .map_err(sql_error)?;
        let rows = statement
            .query_map([], |row| {
                Ok((conversation_row(row)?, row.get::<_, Vec<u8>>(12)?))
            })
            .map_err(sql_error)?
            .collect::<rusqlite::Result<Vec<_>>>()
            .map_err(sql_error)?;
        Ok(rows
            .into_iter()
            .filter_map(|(row, blob)| vector_from_blob(&blob).map(|vector| (row, vector)))
            .collect())
    }

    /// 「処理できなかった会話 … [再試行]」: every `failed` mark, cleared at once.
    pub(crate) fn retry_failed(&self) -> Result<usize> {
        self.connection
            .execute(
                "UPDATE sessions SET state = 'pending', attempts = 0 WHERE state = 'failed'",
                [],
            )
            .map_err(sql_error)
    }

    pub(crate) fn set_pinned(&self, entry: i64, pinned: bool) -> Result<()> {
        self.connection
            .execute(
                "UPDATE entries SET pinned = ?2 WHERE id = ?1",
                params![entry, i64::from(pinned)],
            )
            .map(|_| ())
            .map_err(sql_error)
    }

    pub(crate) fn delete_entry(&self, entry: i64) -> Result<()> {
        self.connection
            .execute("DELETE FROM entries WHERE id = ?1", params![entry])
            .map(|_| ())
            .map_err(sql_error)
    }

    /// One reference: an entry opened in the tab, or printed to an agent.
    pub(crate) fn record_references(&self, entries: &[i64], now: u64) -> Result<()> {
        for entry in entries {
            self.connection
                .execute(
                    "UPDATE entries SET reference_count = reference_count + 1, referenced_at = ?2
                     WHERE id = ?1",
                    params![entry, now as i64],
                )
                .map_err(sql_error)?;
        }
        Ok(())
    }

    /// Insert a judged entry, or — within `KNOWLEDGE_DEDUPE_COSINE` of one
    /// already there — reinforce that one instead. Returns whether it was new.
    pub(crate) fn add_or_reinforce(
        &self,
        entry: &NewEntry,
        source: &NewSource,
        now: u64,
        may_insert: bool,
    ) -> Result<AddOutcome> {
        let nearest = self
            .entry_vectors()?
            .into_iter()
            .map(|(id, vector)| (id, cosine(&vector, &entry.vector)))
            .max_by(|left, right| left.1.total_cmp(&right.1));
        let entry_id = match nearest {
            Some((id, similarity)) if similarity >= KNOWLEDGE_DEDUPE_COSINE => {
                self.connection
                    .execute(
                        "UPDATE entries SET recurrences = recurrences + 1, reinforced_at = ?2 WHERE id = ?1",
                        params![id, now as i64],
                    )
                    .map_err(sql_error)?;
                self.insert_source(id, source, now)?;
                return Ok(AddOutcome::Reinforced(id));
            }
            _ if !may_insert => return Ok(AddOutcome::Refused),
            _ => {
                self.connection
                    .execute(
                        "INSERT INTO entries (kind, title, body, created_at, reinforced_at, referenced_at,
                                              reference_count, recurrences, pinned, vector)
                         VALUES (?1, ?2, ?3, ?4, ?4, ?4, 0, 0, 0, ?5)",
                        params![
                            entry.kind.key(),
                            entry.title,
                            entry.body,
                            now as i64,
                            vector_to_blob(&entry.vector)
                        ],
                    )
                    .map_err(sql_error)?;
                self.connection.last_insert_rowid()
            }
        };
        self.insert_source(entry_id, source, now)?;
        Ok(AddOutcome::Added(entry_id))
    }

    fn insert_source(&self, entry: i64, source: &NewSource, now: u64) -> Result<()> {
        self.connection
            .execute(
                "INSERT INTO sources (entry_id, provider, session_id, transcript_path, exchange_index,
                                      recorded_at, text, vector)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![
                    entry,
                    provider_key(source.provider),
                    source.session_id,
                    source.transcript_path.to_string_lossy(),
                    source.exchange_index as i64,
                    now as i64,
                    source.text,
                    vector_to_blob(&source.vector),
                ],
            )
            .map(|_| ())
            .map_err(sql_error)
    }

    /// Remove unpinned entries below `KNOWLEDGE_REMOVE_FRESHNESS`, then the
    /// least fresh unpinned ones while more than `KNOWLEDGE_MAX_ENTRIES`
    /// remain. Returns how many went.
    pub(crate) fn prune(&self, now: u64) -> Result<usize> {
        let entries = self.entries()?;
        // From the same read as the list below, so another connection's delete
        // between two queries cannot make `total - removed` wrap.
        let total = entries.len();
        let mut unpinned: Vec<(i64, f64)> = entries
            .into_iter()
            .filter(|entry| !entry.pinned)
            .map(|entry| (entry.id, entry.freshness(now)))
            .collect();
        unpinned.sort_by(|left, right| left.1.total_cmp(&right.1));
        let mut removed = 0;
        for (id, fresh) in unpinned {
            if fresh >= KNOWLEDGE_REMOVE_FRESHNESS
                && total.saturating_sub(removed) <= KNOWLEDGE_MAX_ENTRIES
            {
                break;
            }
            self.delete_entry(id)?;
            removed += 1;
        }
        Ok(removed)
    }

    /// Delete everything this project collected. 「ナレッジを削除…」.
    pub(crate) fn clear(&self) -> Result<()> {
        self.connection
            .execute_batch("DELETE FROM sources; DELETE FROM entries; DELETE FROM sessions;")
            .map_err(sql_error)
    }

    /// Clear and switch off as one write, so a run's `write_if_enabled` sees
    /// either both or neither.
    pub(crate) fn clear_and_disable(&mut self) -> Result<()> {
        self.transaction(|database| {
            database.clear()?;
            database.set_enabled(false)
        })
    }

    /// A run's write: inside `BEGIN IMMEDIATE`, and only while the project is
    /// still enabled. A run keeps its connection for minutes (a judge call
    /// alone may take two), and 「収集を止める」 or 「ナレッジを削除…」 in that
    /// window must not be written back over. `None` when it was switched off.
    pub(crate) fn write_if_enabled<T>(
        &mut self,
        work: impl FnOnce(&Self) -> Result<T>,
    ) -> Result<Option<T>> {
        self.transaction(|database| {
            if database.enabled()? {
                work(database).map(Some)
            } else {
                Ok(None)
            }
        })
    }

    fn transaction<T>(&mut self, work: impl FnOnce(&Self) -> Result<T>) -> Result<T> {
        self.connection
            .execute_batch("BEGIN IMMEDIATE")
            .map_err(sql_error)?;
        match work(self) {
            Ok(value) => {
                self.connection.execute_batch("COMMIT").map_err(sql_error)?;
                Ok(value)
            }
            Err(error) => {
                let _ = self.connection.execute_batch("ROLLBACK");
                Err(error)
            }
        }
    }
}

fn conversation_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<ConversationRow> {
    let provider: String = row.get(0)?;
    Ok(ConversationRow {
        provider: provider_from_key(&provider).unwrap_or_default(),
        session_id: row.get(1)?,
        transcript_path: PathBuf::from(row.get::<_, String>(2)?),
        mtime: row.get::<_, i64>(3)?.max(0) as u64,
        size: row.get::<_, i64>(4)?.max(0) as u64,
        cursor: row.get::<_, i64>(5)?.max(0) as usize,
        state: ConversationState::from_key(&row.get::<_, String>(6)?),
        attempts: row.get::<_, i64>(7)?.max(0) as u32,
        processed_at: row.get::<_, i64>(8)?.max(0) as u64,
        paused_at: row.get::<_, Option<i64>>(9)?.map(|at| at.max(0) as u64),
        title: row.get(10)?,
        indexed: row.get(11)?,
    })
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct NewEntry {
    pub(crate) kind: KnowledgeKind,
    pub(crate) title: String,
    pub(crate) body: String,
    pub(crate) vector: Vec<f32>,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct NewSource {
    pub(crate) provider: CliProvider,
    pub(crate) session_id: String,
    pub(crate) transcript_path: PathBuf,
    pub(crate) exchange_index: usize,
    pub(crate) text: String,
    pub(crate) vector: Vec<f32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AddOutcome {
    Added(i64),
    Reinforced(i64),
    /// Pinned entries fill the base; nothing new is added.
    Refused,
}

// ---------------------------------------------------------------------------
// Exchanges.

/// One person turn and the agent turns answering it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Exchange {
    pub(crate) index: usize,
    pub(crate) person: String,
    pub(crate) agent: String,
}

impl Exchange {
    /// What is embedded: both halves, cut to `KNOWLEDGE_EXCHANGE_MAX_CHARS`.
    pub(crate) fn text(&self) -> String {
        exchange_text(&self.person, &self.agent)
    }
}

pub(crate) fn exchange_text(person: &str, agent: &str) -> String {
    let joined = format!("{}\n\n{}", person.trim(), agent.trim());
    joined.chars().take(KNOWLEDGE_EXCHANGE_MAX_CHARS).collect()
}

/// Turns as `read_transcript_conversation` returns them, into exchanges. Agent
/// turns before the first person turn have nothing to answer and are dropped.
pub(crate) fn split_exchanges(turns: &[(&'static str, String)]) -> Vec<Exchange> {
    let mut exchanges: Vec<Exchange> = Vec::new();
    for (role, text) in turns {
        if *role == "user" {
            exchanges.push(Exchange {
                index: exchanges.len(),
                person: text.clone(),
                agent: String::new(),
            });
        } else if let Some(last) = exchanges.last_mut() {
            if !last.agent.is_empty() {
                last.agent.push_str("\n\n");
            }
            last.agent.push_str(text);
        }
    }
    exchanges
}

// ---------------------------------------------------------------------------
// The first step: rank, locally.

/// Labelled examples of what to keep — a decision with its reason, a cause and
/// its fix, a procedure, a pitfall, a project fact. Measured 2026-10-16:
/// abstract one-line anchors ("a design decision and its reason") separated
/// keep from drop by ±0.05 and did not separate them; whole example exchanges
/// do. Each one is in the golden fixture, so changing one means regenerating
/// its vector (`embedding_golden_fixture_is_well_formed`).
pub(crate) const KEEP_EXEMPLARS: [&str; 10] = [
    "なぜ restore で最初のターンが消えるの？\n\nheader 行を session_meta として捨てていたのが原因です。TRANSCRIPT_VOCABULARY に分類を追加して、最初のターンが残るように直しました。",
    "リトライ回数はどうする？\n\n3回で打ち切ります。tmux が起動直後に落ちるケースでは無限に再起動してしまうためです。MAX_RESPAWN_ATTEMPTS = 3 にしました。",
    "リリースの手順を教えて\n\nbash scripts/package-macos.sh で dist/Operon.app を作り、起動中のアプリを終了してから scripts/replace-macos-bundle.sh で /Applications を置き換えます。",
    "Why does the build fail on CI but not locally?\n\nCI uses rustc 1.88.0 from rust-toolchain.toml, and `is_none_or` was stabilised later. I replaced it with `map_or(true, ..)` so it compiles on the pinned toolchain.",
    "agy で --print を使うときの注意点は？\n\n--sandbox を付けないとファイルに書き込みます。判定や下書きのような読み取り専用の用途では必ず --print --sandbox を付けてください。",
    "Where does Operon keep its data?\n\nEverything is under ~/Library/Application Support/com.local.operon/: store-v2.json is the index, written atomically under a flock, and shared-sessions/ holds transcript snapshots with mode 0600.",
    "テストが時々落ちるのはなぜ？\n\n並列実行で同じ一時ディレクトリを使っていたのが原因でした。tempdir をテストごとに作るように変えたところ、100回連続で通りました。",
    "Should we store the opt-in flag in store-v2.json?\n\nNo. Changing the store needs a migration and a schema bump; the knowledge database can hold the flag in its own meta table, so the store stays untouched.",
    "コミットが gate に拒否された\n\ngate-commit.sh は worktree の外を -C で指すコミットを拒否します。コミットは worktree のディレクトリから実行してください。",
    "How do I run the ignored tests?\n\nThey need a live claude, codex, or agy. Run `cargo test --locked -- --ignored` with those CLIs signed in; CI runs them as a non-blocking job.",
];

/// Labelled examples of what to drop: thanks, "continue", raw listings, retry
/// chatter.
pub(crate) const DROP_EXEMPLARS: [&str; 10] = [
    "ありがとう！\n\nどういたしまして。ほかに何かあれば言ってください。",
    "続けて\n\nはい、続けます。",
    "src の中身を見せて\n\nsrc/app.rs src/cli.rs src/main.rs src/store.rs src/tests.rs があります。",
    "もう一回テストを回して\n\ncargo test を再実行しました。まだ1件失敗しています。もう一度確認します。",
    "ok\n\nUnderstood. Let me know when you want me to continue.",
    "git status\n\nOn branch main. Your branch is up to date with 'origin/main'. nothing to commit, working tree clean",
    "Can you read that file again?\n\nSure, here it is again: (file contents shown above).",
    "おはよう\n\nおはようございます！今日は何をしましょうか？",
    "Try again\n\nRetrying the command now... It is still running. I will check the output shortly.",
    "ls docs\n\nREADME.md  sdlc/  DESIGN.md  CHANGELOG.md",
];

/// The exemplars' vectors, embedded once per run.
pub(crate) struct Exemplars {
    keep: Vec<Vec<f32>>,
    drop: Vec<Vec<f32>>,
}

impl Exemplars {
    pub(crate) fn embed(embedder: &dyn Embedder) -> Result<Self> {
        let inputs = KEEP_EXEMPLARS
            .iter()
            .chain(DROP_EXEMPLARS.iter())
            .map(|text| classification_input(text))
            .collect::<Vec<_>>();
        let mut vectors = embedder.embed(&inputs)?;
        let drop = vectors.split_off(KEEP_EXEMPLARS.len());
        Ok(Self {
            keep: vectors,
            drop,
        })
    }

    pub(crate) fn count() -> usize {
        KEEP_EXEMPLARS.len() + DROP_EXEMPLARS.len()
    }
}

fn mean_top3(vector: &[f32], references: &[Vec<f32>]) -> f32 {
    let mut similarities = references
        .iter()
        .map(|reference| cosine(vector, reference))
        .collect::<Vec<_>>();
    similarities.sort_by(|left, right| right.total_cmp(left));
    let top = &similarities[..similarities.len().min(3)];
    if top.is_empty() {
        return 0.0;
    }
    top.iter().sum::<f32>() / top.len() as f32
}

/// 0.02 each for a code span, a file path, an error line, and a stated reason.
pub(crate) fn structure_bonus(text: &str) -> f32 {
    let lowered = text.to_lowercase();
    let code = text.contains('`');
    let path = text
        .split_whitespace()
        .any(|token| token.contains('/') && token.contains('.'));
    let error = ["error", "エラー", "failed", "失敗"]
        .iter()
        .any(|word| lowered.contains(word));
    let reason = ["ため", "because", "原因"]
        .iter()
        .any(|word| lowered.contains(word));
    0.02 * [code, path, error, reason]
        .into_iter()
        .filter(|hit| *hit)
        .count() as f32
}

pub(crate) fn exchange_score(text: &str, vector: &[f32], exemplars: &Exemplars) -> f32 {
    mean_top3(vector, &exemplars.keep) - mean_top3(vector, &exemplars.drop) + structure_bonus(text)
}

/// An exchange that quotes the knowledge command is the base reading itself
/// back; it is never a candidate.
pub(crate) fn quotes_knowledge_command(text: &str) -> bool {
    text.contains(&format!("{KNOWLEDGE_CLI_SUBCOMMAND} search"))
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Candidate {
    pub(crate) exchange: Exchange,
    pub(crate) text: String,
    pub(crate) vector: Vec<f32>,
    pub(crate) score: f32,
}

/// The first step. Short exchanges and ones quoting the knowledge command are
/// not embedded at all; copies of a recorded source are dropped; the rest are
/// ranked and the top `KNOWLEDGE_CANDIDATES_PER_CONVERSATION` at or above
/// `KNOWLEDGE_CANDIDATE_MIN_SCORE` go on to the judge.
pub(crate) fn rank_candidates(
    exchanges: &[Exchange],
    vectors: &[Vec<f32>],
    exemplars: &Exemplars,
    recorded_sources: &[(i64, Vec<f32>)],
) -> Vec<Candidate> {
    let mut candidates = exchanges
        .iter()
        .zip(vectors)
        .filter(|(_, vector)| {
            !recorded_sources
                .iter()
                .any(|(_, source)| cosine(source, vector) >= KNOWLEDGE_COPY_COSINE)
        })
        .map(|(exchange, vector)| {
            let text = exchange.text();
            Candidate {
                score: exchange_score(&text, vector, exemplars),
                exchange: exchange.clone(),
                text,
                vector: vector.clone(),
            }
        })
        .filter(|candidate| candidate.score >= KNOWLEDGE_CANDIDATE_MIN_SCORE)
        .collect::<Vec<_>>();
    candidates.sort_by(|left, right| right.score.total_cmp(&left.score));
    candidates.truncate(KNOWLEDGE_CANDIDATES_PER_CONVERSATION);
    candidates
}

/// Which exchanges are worth embedding for ranking at all.
pub(crate) fn rankable(exchange: &Exchange) -> bool {
    let text = exchange.text();
    text.chars().count() >= KNOWLEDGE_EXCHANGE_MIN_CHARS && !quotes_knowledge_command(&text)
}

// ---------------------------------------------------------------------------
// The second step: the conversation's own CLI judges.

pub(crate) fn judge_prompt(candidates: &[Candidate]) -> String {
    let mut prompt = String::from(
        "You are curating a project knowledge base from one coding-agent conversation. \
Below are numbered candidate exchanges from it. Keep only knowledge a later conversation in \
this project would need: a decision and its reason, a cause and its fix, a procedure, a \
pitfall, or a fact about the project. Skip small talk, progress reports, raw listings, and \
anything that only mattered at that moment.\n\n\
Answer with only a JSON array, no prose and no code fence, of 0 to 5 objects:\n\
{\"kind\": \"decision\" | \"cause\" | \"procedure\" | \"pitfall\" | \"fact\", \
\"title\": \"at most 40 characters\", \"body\": \"at most 400 characters, understandable on its own\", \
\"candidate\": <the candidate's number>}\n\
Write title and body in the language the conversation uses. Never copy secrets, tokens, \
passwords, or personal data. Answer [] when nothing is worth keeping.\n",
    );
    for (number, candidate) in (1..).zip(candidates) {
        let person: String = candidate
            .exchange
            .person
            .chars()
            .take(KNOWLEDGE_JUDGE_PERSON_CHARS)
            .collect();
        let agent: String = candidate
            .exchange
            .agent
            .chars()
            .take(KNOWLEDGE_JUDGE_AGENT_CHARS)
            .collect();
        let block = format!("\n[{number}]\nPerson: {person}\nAgent: {agent}\n");
        if prompt.len() + block.len() > KNOWLEDGE_JUDGE_PROMPT_MAX_BYTES {
            break;
        }
        prompt.push_str(&block);
    }
    prompt
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct JudgedEntry {
    pub(crate) kind: KnowledgeKind,
    pub(crate) title: String,
    pub(crate) body: String,
    /// Index into the candidates the prompt listed.
    pub(crate) candidate: usize,
}

/// The judge's answer, or `None` when it is not the array the prompt asked
/// for. Items naming an unknown kind or a candidate that was not offered are
/// dropped; titles and bodies past their limits are cut.
pub(crate) fn parse_judge_answer(answer: &str, offered: usize) -> Option<Vec<JudgedEntry>> {
    let start = answer.find('[')?;
    let end = answer.rfind(']')?;
    if end < start {
        return None;
    }
    let items: Vec<serde_json::Value> = serde_json::from_str(&answer[start..=end]).ok()?;
    Some(
        items
            .iter()
            .filter_map(|item| {
                let kind = KnowledgeKind::from_key(item.get("kind")?.as_str()?.trim())?;
                let title = item.get("title")?.as_str()?.trim();
                let body = item.get("body")?.as_str()?.trim();
                let number = item.get("candidate")?.as_u64()? as usize;
                (!title.is_empty() && !body.is_empty() && (1..=offered).contains(&number)).then(
                    || JudgedEntry {
                        kind,
                        title: title.chars().take(KNOWLEDGE_TITLE_MAX_CHARS).collect(),
                        body: body.chars().take(KNOWLEDGE_BODY_MAX_CHARS).collect(),
                        candidate: number - 1,
                    },
                )
            })
            .take(KNOWLEDGE_JUDGE_MAX_ENTRIES)
            .collect(),
    )
}

/// What came back from a judge CLI.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum JudgeOutcome {
    Answer(String),
    /// Missing, not signed in, timed out, or exited non-zero: not the
    /// conversation's fault, so nothing is spent and the next run tries again.
    Pause(String),
}

pub(crate) trait Judge {
    fn judge(&self, provider: CliProvider, prompt: &str) -> JudgeOutcome;
}

/// The production judge: the conversation's own CLI, read-only and ephemeral,
/// run in `knowledge_judge_directory` so it never becomes a conversation of the
/// project it is judging.
pub(crate) struct CliJudge {
    pub(crate) directory: PathBuf,
}

impl CliJudge {
    /// The child a judge call runs: the provider's read-only, ephemeral
    /// invocation, inside the judge directory rather than any project's tree.
    pub(crate) fn command(&self, provider: CliProvider, prompt: &str) -> Command {
        let (program, arguments) = knowledge_judge_command(provider, prompt);
        let mut command = Command::new(program);
        command.args(arguments).current_dir(&self.directory);
        command
    }
}

impl Judge for CliJudge {
    fn judge(&self, provider: CliProvider, prompt: &str) -> JudgeOutcome {
        if let Err(error) = create_private_knowledge_directory(&self.directory) {
            return JudgeOutcome::Pause(error.to_string());
        }
        let mut command = self.command(provider, prompt);
        match run_command_with_output_limit(
            &mut command,
            Duration::from_secs(KNOWLEDGE_JUDGE_SECONDS),
            KNOWLEDGE_JUDGE_ANSWER_MAX_BYTES,
            COMMAND_ERROR_MAX_BYTES,
        ) {
            Ok(output) if output.output.status.success() => {
                JudgeOutcome::Answer(String::from_utf8_lossy(&output.output.stdout).into_owned())
            }
            Ok(output) => JudgeOutcome::Pause(
                String::from_utf8_lossy(&output.output.stderr)
                    .trim()
                    .chars()
                    .take(200)
                    .collect(),
            ),
            Err(error) => JudgeOutcome::Pause(error.to_string()),
        }
    }
}

// ---------------------------------------------------------------------------
// A run.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct RunLimits {
    pub(crate) processed: usize,
    pub(crate) embeddings: usize,
    pub(crate) wall: Duration,
    pub(crate) idle_seconds: u64,
}

impl Default for RunLimits {
    fn default() -> Self {
        Self {
            processed: KNOWLEDGE_SESSIONS_PER_RUN,
            embeddings: KNOWLEDGE_EMBEDDINGS_PER_RUN,
            wall: Duration::from_secs(KNOWLEDGE_RUN_SECONDS),
            idle_seconds: KNOWLEDGE_IDLE_SECONDS,
        }
    }
}

/// What a run did to one project, for the tab's status line.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct RunReport {
    pub(crate) processed: usize,
    pub(crate) added: usize,
    pub(crate) reinforced: usize,
    pub(crate) removed: usize,
    pub(crate) judge_calls: usize,
    /// The CLI that could not judge; the run stopped judging there.
    pub(crate) paused: Option<CliProvider>,
    pub(crate) capped: bool,
    pub(crate) full_of_pinned: bool,
    pub(crate) rebuilding: Option<(usize, usize)>,
    /// Conversations left for a later run by a budget.
    pub(crate) deferred: usize,
    pub(crate) embeddings: usize,
}

struct Budget {
    embeddings_left: usize,
    deadline: Instant,
}

impl Budget {
    fn spend(&mut self, count: usize) -> bool {
        if count > self.embeddings_left || Instant::now() >= self.deadline {
            return false;
        }
        self.embeddings_left -= count;
        true
    }

    fn exhausted(&self) -> bool {
        self.embeddings_left == 0 || Instant::now() >= self.deadline
    }
}

fn file_stamp(path: &Path) -> Option<(u64, u64)> {
    let metadata = fs::metadata(path).ok()?;
    let mtime = metadata
        .modified()
        .ok()?
        .duration_since(UNIX_EPOCH)
        .ok()?
        .as_secs();
    Some((mtime, metadata.len()))
}

/// Conversations with no pause first, in the order given (newest first); then
/// the paused ones, oldest pause first. A conversation whose prompt makes its
/// CLI fail every time then only ever blocks itself.
pub(crate) fn order_for_run(sessions: Vec<(CliSession, Option<u64>)>) -> Vec<CliSession> {
    let (mut paused, fresh): (Vec<_>, Vec<_>) = sessions
        .into_iter()
        .partition(|(_, paused_at)| paused_at.is_some());
    paused.sort_by_key(|(_, paused_at)| *paused_at);
    fresh
        .into_iter()
        .chain(paused)
        .map(|(session, _)| session)
        .collect()
}

pub(crate) fn utc_day(now: u64) -> u64 {
    now / 86_400
}

/// One project's run: rebuild vectors from another model, prune, then read,
/// rank, judge, and store the conversations given (newest first, at most
/// `KNOWLEDGE_SESSIONS_LISTED_PER_RUN`), within `limits`.
pub(crate) fn run_project(
    database: &mut KnowledgeDatabase,
    sessions: &[CliSession],
    embedder: &dyn Embedder,
    judge: &dyn Judge,
    now: u64,
    limits: RunLimits,
    progress: &dyn Fn(usize, usize),
) -> Result<RunReport> {
    let mut report = RunReport::default();
    if !database.enabled()? {
        return Ok(report);
    }
    let mut budget = Budget {
        embeddings_left: limits.embeddings,
        deadline: Instant::now() + limits.wall,
    };
    if !database.model_matches()? {
        let progress = rebuild_vectors(database, embedder, &mut budget)?;
        report.embeddings = limits.embeddings - budget.embeddings_left;
        if let Some(progress) = progress {
            report.rebuilding = Some(progress);
            return Ok(report);
        }
    }
    report.removed += database
        .write_if_enabled(|database| database.prune(now))?
        .unwrap_or(0);
    // Embedded on the first conversation that needs ranking, so a run with
    // nothing new to read never loads the model.
    let mut exemplars: Option<Exemplars> = None;

    let mut queue = Vec::new();
    for session in sessions.iter().take(KNOWLEDGE_SESSIONS_LISTED_PER_RUN) {
        let row = database.conversation(session.provider, &session.id)?;
        queue.push((session.clone(), row.and_then(|row| row.paused_at)));
    }
    let mut judging = true;
    let ordered = order_for_run(queue);
    let total = ordered.len().min(limits.processed);
    for session in ordered {
        progress(report.processed, total);
        if report.processed >= limits.processed || budget.exhausted() {
            report.deferred += 1;
            continue;
        }
        let outcome = process_conversation(
            database,
            &session,
            embedder,
            judge,
            &mut exemplars,
            &mut budget,
            now,
            limits,
            &mut judging,
            &mut report,
        )?;
        match outcome {
            Processed::Read => report.processed += 1,
            Processed::Skipped => {}
            Processed::OutOfBudget => report.deferred += 1,
            Processed::Disabled => return Ok(report),
        }
    }
    report.removed += database
        .write_if_enabled(|database| database.prune(now))?
        .unwrap_or(0);
    report.embeddings = limits.embeddings - budget.embeddings_left;
    Ok(report)
}

enum Processed {
    Read,
    Skipped,
    OutOfBudget,
    /// The project was switched off or cleared while this run was going.
    Disabled,
}

#[allow(clippy::too_many_arguments)]
fn process_conversation(
    database: &mut KnowledgeDatabase,
    session: &CliSession,
    embedder: &dyn Embedder,
    judge: &dyn Judge,
    exemplars: &mut Option<Exemplars>,
    budget: &mut Budget,
    now: u64,
    limits: RunLimits,
    judging: &mut bool,
    report: &mut RunReport,
) -> Result<Processed> {
    let existing = database.conversation(session.provider, &session.id)?;
    let transcript = full_history_restore_transcript(session).ok();
    let stamp = transcript.as_deref().and_then(file_stamp);
    let mut row = existing.clone().unwrap_or(ConversationRow {
        provider: session.provider,
        session_id: session.id.clone(),
        transcript_path: transcript.clone().unwrap_or_else(|| session.path.clone()),
        mtime: 0,
        size: 0,
        cursor: 0,
        state: ConversationState::Pending,
        attempts: 0,
        processed_at: 0,
        paused_at: None,
        title: None,
        indexed: false,
    });
    let (Some(transcript), Some((mtime, size))) = (transcript, stamp) else {
        if row.state != ConversationState::Unreadable {
            row.state = ConversationState::Unreadable;
            row.processed_at = now;
            if database
                .write_if_enabled(|database| database.save_conversation(&row))?
                .is_none()
            {
                return Ok(Processed::Disabled);
            }
        }
        return Ok(Processed::Skipped);
    };
    // Still being written: leave it for a run after the conversation settles.
    if now.saturating_sub(mtime) < limits.idle_seconds {
        return Ok(Processed::Skipped);
    }
    let unchanged = existing.is_some() && row.mtime == mtime && row.size == size;
    match row.state {
        ConversationState::TooLarge | ConversationState::Unreadable if unchanged => {
            return Ok(Processed::Skipped);
        }
        ConversationState::Failed
            if unchanged
                && now.saturating_sub(row.processed_at) < KNOWLEDGE_FAILED_RETRY_DAYS * 86_400 =>
        {
            return Ok(Processed::Skipped);
        }
        ConversationState::Pending
            if unchanged && row.indexed && row.paused_at.is_none() && row.attempts == 0 =>
        {
            return Ok(Processed::Skipped);
        }
        ConversationState::Failed => {
            row.attempts = 0;
        }
        _ => {}
    }
    row.state = ConversationState::Pending;
    row.transcript_path = transcript.clone();
    row.title = session.title.clone().or(row.title);
    let turns = match read_transcript_conversation(&transcript, session.provider) {
        Ok((turns, _)) => turns,
        Err(_) => {
            row.state = if fs::File::open(&transcript).is_ok() {
                ConversationState::TooLarge
            } else {
                ConversationState::Unreadable
            };
            row.mtime = mtime;
            row.size = size;
            row.processed_at = now;
            if database
                .write_if_enabled(|database| database.save_conversation(&row))?
                .is_none()
            {
                return Ok(Processed::Disabled);
            }
            return Ok(Processed::Skipped);
        }
    };
    let exchanges = split_exchanges(&turns);
    // The conversation's own vector, for 「ローカル履歴」: its title and opening.
    let index_text = exchanges
        .first()
        .map(|first| document_input(row.title.as_deref(), &first.text()));
    let needs_index = !row.indexed && index_text.is_some();
    let new: Vec<Exchange> = exchanges
        .iter()
        .skip(row.cursor)
        .take(KNOWLEDGE_EXCHANGES_PER_CONVERSATION)
        .cloned()
        .collect();
    let rankable_exchanges: Vec<Exchange> = new
        .iter()
        .filter(|exchange| rankable(exchange))
        .cloned()
        .collect();
    let exemplar_cost = if exemplars.is_none() && !rankable_exchanges.is_empty() {
        Exemplars::count()
    } else {
        0
    };
    if !budget.spend(rankable_exchanges.len() + usize::from(needs_index) + exemplar_cost) {
        return Ok(Processed::OutOfBudget);
    }
    if exemplar_cost > 0 {
        *exemplars = Some(Exemplars::embed(embedder)?);
    }
    let index = match (needs_index, index_text) {
        (true, Some(text)) => embedder
            .embed(std::slice::from_ref(&text))?
            .pop()
            .map(|vector| (text, vector)),
        _ => None,
    };
    let saved = database.write_if_enabled(|database| {
        database.save_conversation(&row)?;
        if let Some((text, vector)) = &index {
            database.index_conversation(session.provider, &session.id, text, vector)?;
        }
        Ok(())
    })?;
    if saved.is_none() {
        return Ok(Processed::Disabled);
    }
    row.indexed |= index.is_some();
    let inputs = rankable_exchanges
        .iter()
        .map(|exchange| classification_input(&exchange.text()))
        .collect::<Vec<_>>();
    let vectors = embedder.embed(&inputs)?;
    let candidates = match exemplars.as_ref() {
        Some(exemplars) => rank_candidates(
            &rankable_exchanges,
            &vectors,
            exemplars,
            &database.source_vectors()?,
        ),
        None => Vec::new(),
    };
    let advanced_cursor = row.cursor + new.len();
    // The file's stamp is recorded only once every exchange in it has been
    // read: a conversation longer than one run's share stays "changed", so the
    // next run continues from the cursor instead of skipping the rest.
    let read_to_end = advanced_cursor >= exchanges.len();
    if candidates.is_empty() {
        row.cursor = advanced_cursor;
        if read_to_end {
            row.mtime = mtime;
            row.size = size;
        }
        row.processed_at = now;
        row.paused_at = None;
        if database
            .write_if_enabled(|database| database.save_conversation(&row))?
            .is_none()
        {
            return Ok(Processed::Disabled);
        }
        return Ok(Processed::Read);
    }
    if !*judging {
        return Ok(Processed::Skipped);
    }
    if database.judge_calls_on(utc_day(now))? >= KNOWLEDGE_DAILY_JUDGE_CALLS {
        report.capped = true;
        *judging = false;
        return Ok(Processed::Skipped);
    }
    let prompt = judge_prompt(&candidates);
    let answer = match judge.judge(session.provider, &prompt) {
        JudgeOutcome::Pause(_) => {
            report.paused = Some(session.provider);
            *judging = false;
            row.paused_at = Some(now);
            if database
                .write_if_enabled(|database| database.save_conversation(&row))?
                .is_none()
            {
                return Ok(Processed::Disabled);
            }
            return Ok(Processed::Skipped);
        }
        JudgeOutcome::Answer(answer) => answer,
    };
    database.record_judge_call(utc_day(now))?;
    report.judge_calls += 1;
    let Some(judged) = parse_judge_answer(&answer, candidates.len()) else {
        row.attempts += 1;
        row.paused_at = None;
        row.processed_at = now;
        if row.attempts >= KNOWLEDGE_FAILED_ATTEMPTS {
            row.state = ConversationState::Failed;
            row.mtime = mtime;
            row.size = size;
        }
        if database
            .write_if_enabled(|database| database.save_conversation(&row))?
            .is_none()
        {
            return Ok(Processed::Disabled);
        }
        return Ok(Processed::Read);
    };
    let entry_inputs = judged
        .iter()
        .map(|entry| document_input(Some(&entry.title), &entry.body))
        .collect::<Vec<_>>();
    // A judged answer is stored whole even past the embedding budget: the call
    // is spent, and dropping its result would spend it again next run.
    budget.embeddings_left = budget.embeddings_left.saturating_sub(entry_inputs.len());
    let entry_vectors = embedder.embed(&entry_inputs)?;
    let may_insert = may_add_entries(database.pinned_count()?);
    let stored = database.write_if_enabled(|database| {
        for (entry, vector) in judged.iter().zip(entry_vectors) {
            let candidate = &candidates[entry.candidate];
            let outcome = database.add_or_reinforce(
                &NewEntry {
                    kind: entry.kind,
                    title: entry.title.clone(),
                    body: entry.body.clone(),
                    vector,
                },
                &NewSource {
                    provider: session.provider,
                    session_id: session.id.clone(),
                    transcript_path: transcript.clone(),
                    exchange_index: candidate.exchange.index,
                    text: candidate.text.clone(),
                    vector: candidate.vector.clone(),
                },
                now,
                may_insert,
            )?;
            match outcome {
                AddOutcome::Added(_) => report.added += 1,
                AddOutcome::Reinforced(_) => report.reinforced += 1,
                AddOutcome::Refused => report.full_of_pinned = true,
            }
        }
        row.cursor = advanced_cursor;
        row.attempts = 0;
        row.paused_at = None;
        if read_to_end {
            row.mtime = mtime;
            row.size = size;
        }
        row.processed_at = now;
        database.save_conversation(&row)
    })?;
    Ok(if stored.is_some() {
        Processed::Read
    } else {
        Processed::Disabled
    })
}

/// Req 21. A database whose vectors came from another model is re-embedded
/// from the text it stored, table by table, within the run's budget. Until the
/// last row is done `meta.model` keeps naming the old model, so search and the
/// pipeline compare nothing against a half-rebuilt base. `None` when finished.
fn rebuild_vectors(
    database: &mut KnowledgeDatabase,
    embedder: &dyn Embedder,
    budget: &mut Budget,
) -> Result<Option<(usize, usize)>> {
    const TABLES: [(&str, &str); 3] = [
        ("entries", "SELECT id, title, body FROM entries WHERE id > ?1 ORDER BY id"),
        ("sources", "SELECT id, '', text FROM sources WHERE id > ?1 ORDER BY id"),
        ("sessions", "SELECT rowid, '', text FROM sessions WHERE rowid > ?1 AND text IS NOT NULL ORDER BY rowid"),
    ];
    let total = database.count("SELECT COUNT(*) FROM entries")?
        + database.count("SELECT COUNT(*) FROM sources")?
        + database.count("SELECT COUNT(*) FROM sessions WHERE text IS NOT NULL")?;
    let mut table = database
        .meta("rebuild_table")?
        .unwrap_or_else(|| "entries".to_owned());
    let mut after: i64 = database
        .meta("rebuild_after_rowid")?
        .and_then(|value| value.parse().ok())
        .unwrap_or(0);
    let mut done = database
        .meta("rebuild_done")?
        .and_then(|value| value.parse().ok())
        .unwrap_or(0_usize);
    let start = TABLES
        .iter()
        .position(|(name, _)| *name == table)
        .unwrap_or(0);
    for (name, query) in TABLES.iter().skip(start) {
        if *name != table {
            table = (*name).to_owned();
            after = 0;
        }
        let rows = {
            let mut statement = database.connection.prepare(query).map_err(sql_error)?;
            let rows = statement
                .query_map(params![after], |row| {
                    Ok((
                        row.get::<_, i64>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                    ))
                })
                .map_err(sql_error)?
                .collect::<rusqlite::Result<Vec<_>>>()
                .map_err(sql_error)?;
            rows
        };
        for chunk in rows.chunks(EMBEDDING_TEXTS_PER_CALL) {
            if !budget.spend(chunk.len()) {
                database.set_meta("rebuild_table", &table)?;
                database.set_meta("rebuild_after_rowid", &after.to_string())?;
                database.set_meta("rebuild_done", &done.to_string())?;
                return Ok(Some((done, total)));
            }
            let inputs = chunk
                .iter()
                .map(|(_, title, text)| match *name {
                    "entries" => document_input(Some(title), text),
                    "sources" => classification_input(text),
                    _ => text.clone(),
                })
                .collect::<Vec<_>>();
            let vectors = embedder.embed(&inputs)?;
            let update = match *name {
                "entries" => "UPDATE entries SET vector = ?2 WHERE id = ?1",
                "sources" => "UPDATE sources SET vector = ?2 WHERE id = ?1",
                _ => "UPDATE sessions SET vector = ?2 WHERE rowid = ?1",
            };
            database.transaction(|database| {
                for ((id, _, _), vector) in chunk.iter().zip(&vectors) {
                    database
                        .connection
                        .execute(update, params![id, vector_to_blob(vector)])
                        .map_err(sql_error)?;
                }
                Ok(())
            })?;
            after = chunk.last().map_or(after, |(id, _, _)| *id);
            done += chunk.len();
        }
    }
    database.set_meta("model", EMBEDDING_MODEL_ID)?;
    database.set_meta("dimensions", &EMBEDDING_DIMENSIONS.to_string())?;
    for key in ["rebuild_table", "rebuild_after_rowid", "rebuild_done"] {
        database.delete_meta(key)?;
    }
    Ok(None)
}

// ---------------------------------------------------------------------------
// Search.

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct KnowledgeHit {
    pub(crate) entry: KnowledgeEntry,
    pub(crate) similarity: f32,
}

/// Entries ranked by cosine to the query, at or above
/// `KNOWLEDGE_SEARCH_MIN_COSINE`. Nothing while vectors from another model are
/// still being rebuilt.
pub(crate) fn search_entries(
    database: &KnowledgeDatabase,
    embedder: &dyn Embedder,
    query: &str,
    limit: usize,
) -> Result<Vec<KnowledgeHit>> {
    if !database.model_matches()? || query.trim().is_empty() {
        return Ok(Vec::new());
    }
    let Some(query_vector) = embedder.embed(&[query_input(query.trim())])?.pop() else {
        return Ok(Vec::new());
    };
    let mut scored = database
        .entry_vectors()?
        .into_iter()
        .map(|(id, vector)| (id, cosine(&query_vector, &vector)))
        .filter(|(_, similarity)| *similarity >= KNOWLEDGE_SEARCH_MIN_COSINE)
        .collect::<Vec<_>>();
    scored.sort_by(|left, right| right.1.total_cmp(&left.1));
    scored.truncate(limit);
    let mut entries: HashMap<i64, KnowledgeEntry> = database
        .entries()?
        .into_iter()
        .map(|entry| (entry.id, entry))
        .collect();
    Ok(scored
        .into_iter()
        .filter_map(|(id, similarity)| {
            entries
                .remove(&id)
                .map(|entry| KnowledgeHit { entry, similarity })
        })
        .collect())
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct IndexedSessionHit {
    pub(crate) project_id: Uuid,
    pub(crate) project_name: String,
    pub(crate) provider: CliProvider,
    pub(crate) session_id: String,
    pub(crate) title: Option<String>,
    pub(crate) transcript_path: PathBuf,
    pub(crate) updated_at: u64,
    pub(crate) similarity: f32,
}

/// 「ローカル履歴」's 「意味で一致」: the indexed conversations of every enabled
/// project, ranked by cosine to the query.
pub(crate) fn search_indexed_sessions(
    root: &Path,
    projects: &[(Uuid, String)],
    embedder: &dyn Embedder,
    query: &str,
    limit: usize,
) -> Result<Vec<IndexedSessionHit>> {
    if query.trim().is_empty() {
        return Ok(Vec::new());
    }
    let mut databases = Vec::new();
    for (project_id, name) in projects {
        let Some(database) =
            KnowledgeDatabase::open_existing(&knowledge_database_path(root, *project_id))?
        else {
            continue;
        };
        if database.enabled()? && database.model_matches()? {
            databases.push((*project_id, name.clone(), database));
        }
    }
    if databases.is_empty() {
        return Ok(Vec::new());
    }
    let Some(query_vector) = embedder.embed(&[query_input(query.trim())])?.pop() else {
        return Ok(Vec::new());
    };
    let mut hits = Vec::new();
    for (project_id, name, database) in &databases {
        for (row, vector) in database.indexed_conversations()? {
            let similarity = cosine(&query_vector, &vector);
            if similarity >= KNOWLEDGE_SESSION_MIN_COSINE {
                hits.push(IndexedSessionHit {
                    project_id: *project_id,
                    project_name: name.clone(),
                    provider: row.provider,
                    session_id: row.session_id,
                    title: row.title,
                    transcript_path: row.transcript_path,
                    updated_at: row.mtime,
                    similarity,
                });
            }
        }
    }
    hits.sort_by(|left, right| right.similarity.total_cmp(&left.similarity));
    hits.truncate(limit);
    Ok(hits)
}

// ---------------------------------------------------------------------------
// What the tab shows.

#[derive(Debug, Clone, Default, PartialEq)]
pub(crate) struct KnowledgeSnapshot {
    pub(crate) enabled: bool,
    pub(crate) entries: Vec<KnowledgeEntry>,
    pub(crate) judge_calls_today: u32,
    pub(crate) failed: usize,
    pub(crate) last_report: Option<RunReport>,
    pub(crate) updated_at: Option<u64>,
}

pub(crate) fn knowledge_snapshot(
    root: &Path,
    project_id: Uuid,
    now: u64,
) -> Result<KnowledgeSnapshot> {
    let Some(database) =
        KnowledgeDatabase::open_existing(&knowledge_database_path(root, project_id))?
    else {
        return Ok(KnowledgeSnapshot::default());
    };
    Ok(KnowledgeSnapshot {
        enabled: database.enabled()?,
        entries: database.entries()?,
        judge_calls_today: database.judge_calls_on(utc_day(now))?,
        failed: database.failed_count()?,
        last_report: None,
        updated_at: database
            .meta("last_run_at")?
            .and_then(|value| value.parse().ok()),
    })
}

/// The schedule's job: every enabled project, one after another. `progress`
/// hears (project, conversations done, conversations this run will read).
#[allow(clippy::too_many_arguments)]
pub(crate) fn run_enabled_projects(
    root: &Path,
    projects: &[(Uuid, PathBuf)],
    embedder: &dyn Embedder,
    judge: &dyn Judge,
    list_sessions: &dyn Fn(&Path) -> Vec<CliSession>,
    now: u64,
    limits: RunLimits,
    progress: &dyn Fn(Uuid, usize, usize),
) -> Vec<(Uuid, UiResult<KnowledgeSnapshot>)> {
    let mut results = Vec::new();
    for (project_id, path) in projects {
        let database_path = knowledge_database_path(root, *project_id);
        let result = (|| -> Result<Option<KnowledgeSnapshot>> {
            let Some(mut database) = KnowledgeDatabase::open_existing(&database_path)? else {
                return Ok(None);
            };
            if !database.enabled()? {
                return Ok(None);
            }
            let sessions = list_sessions(path);
            let report = run_project(
                &mut database,
                &sessions,
                embedder,
                judge,
                now,
                limits,
                &|done, total| progress(*project_id, done, total),
            )?;
            database.set_meta("last_run_at", &now.to_string())?;
            drop(database);
            let mut snapshot = knowledge_snapshot(root, *project_id, now)?;
            snapshot.last_report = Some(report);
            Ok(Some(snapshot))
        })();
        match result {
            Ok(Some(snapshot)) => results.push((*project_id, Ok(snapshot))),
            Ok(None) => {}
            Err(error) => results.push((*project_id, Err(error.to_string()))),
        }
    }
    results
}

// ---------------------------------------------------------------------------
// `Operon knowledge search "…"`.

/// Which project a directory belongs to: its git top level (or, without git,
/// the directory and its ancestors) against each project's path and worktrees.
pub(crate) fn project_for_directory(directory: &Path, projects: &[Project]) -> Option<Project> {
    let canonical = |path: &Path| path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    let roots: Vec<PathBuf> = projects
        .iter()
        .map(|project| canonical(&project.path))
        .collect();
    // The nearest registered directory at or above this one, whether or not it
    // is a repository.
    let directory = canonical(directory);
    for ancestor in directory.ancestors() {
        if let Some(index) = roots.iter().position(|root| root == ancestor) {
            return Some(projects[index].clone());
        }
    }
    // A linked worktree outside the project's tree: its repository's worktree
    // list names it. Only a project that is itself its repository's top level
    // owns that list — a plain directory nested inside someone else's checkout
    // would otherwise claim every worktree of that checkout.
    let top_level = canonical(&git_top_level(&directory)?);
    projects.iter().zip(&roots).find_map(|(project, root)| {
        let owns_repository = git_top_level(root).is_some_and(|top| canonical(&top) == *root);
        (owns_repository && project_and_worktree_paths(root).contains(&top_level))
            .then(|| project.clone())
    })
}

fn git_top_level(directory: &Path) -> Option<PathBuf> {
    let mut command = git_command(directory);
    command.args(["rev-parse", "--show-toplevel"]);
    let output = run_command_with_timeout(&mut command, Duration::from_secs(5)).ok()?;
    output
        .status
        .success()
        .then(|| PathBuf::from(String::from_utf8_lossy(&output.stdout).trim()))
}

pub(crate) struct CliOutcome {
    pub(crate) code: i32,
    pub(crate) stdout: String,
    pub(crate) stderr: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SearchArguments {
    pub(crate) query: String,
    pub(crate) limit: usize,
    pub(crate) json: bool,
}

pub(crate) fn parse_search_arguments(arguments: &[String]) -> Option<SearchArguments> {
    let mut query = Vec::new();
    let mut limit = KNOWLEDGE_CLI_DEFAULT_LIMIT;
    let mut json = false;
    let mut iterator = arguments.iter();
    while let Some(argument) = iterator.next() {
        match argument.as_str() {
            "--json" => json = true,
            "--limit" => limit = iterator.next()?.parse().ok().filter(|limit| *limit > 0)?,
            other => query.push(other.to_owned()),
        }
    }
    let query = query.join(" ");
    (!query.trim().is_empty()).then_some(SearchArguments { query, limit, json })
}

pub(crate) fn knowledge_usage() -> String {
    tf!(
        "使い方: Operon {command} search \"検索語\" [--limit N] [--json]",
        command = KNOWLEDGE_CLI_SUBCOMMAND
    )
}

/// The agent-facing command, with every input passed in so the tests drive it
/// without a store, a model, or a working directory of their own.
pub(crate) fn knowledge_cli(
    arguments: &[String],
    directory: &Path,
    projects: &[Project],
    root: &Path,
    embedder: Option<&dyn Embedder>,
    now: u64,
) -> CliOutcome {
    let fail = |code: i32, message: String| CliOutcome {
        code,
        stdout: String::new(),
        stderr: format!("{message}\n"),
    };
    let Some((subcommand, rest)) = arguments.split_first() else {
        return fail(64, knowledge_usage());
    };
    if subcommand != "search" {
        return fail(64, knowledge_usage());
    }
    let Some(search) = parse_search_arguments(rest) else {
        return fail(64, knowledge_usage());
    };
    let Some(project) = project_for_directory(directory, projects) else {
        return fail(
            2,
            tf!(
                "{path} は Operon に登録されたプロジェクトではありません。",
                path = directory.display()
            ),
        );
    };
    let database =
        match KnowledgeDatabase::open_existing(&knowledge_database_path(root, project.id)) {
            Ok(Some(database)) => database,
            Ok(None) => return fail(2, not_enabled_message(&project)),
            Err(error) => return fail(1, error.to_string()),
        };
    match database.enabled() {
        Ok(true) => {}
        Ok(false) => return fail(2, not_enabled_message(&project)),
        Err(error) => return fail(1, error.to_string()),
    }
    let Some(embedder) = embedder else {
        return fail(
            3,
            tr("モデルが取得されていません。Operon の設定 › データ で取得してください。")
                .to_owned(),
        );
    };
    match database.model_matches() {
        Ok(true) => {}
        Ok(false) => {
            return fail(
                3,
                tr("ナレッジを新しいモデルで作り直しています。しばらくしてから、もう一度お試しください。")
                    .to_owned(),
            );
        }
        Err(error) => return fail(1, error.to_string()),
    }
    let hits = match search_entries(&database, embedder, &search.query, search.limit) {
        Ok(hits) => hits,
        Err(error) => return fail(1, error.to_string()),
    };
    let ids = hits.iter().map(|hit| hit.entry.id).collect::<Vec<_>>();
    let _ = database.record_references(&ids, now);
    let stdout = if search.json {
        let rows = hits
            .iter()
            .map(|hit| {
                serde_json::json!({
                    "kind": hit.entry.kind.key(),
                    "title": hit.entry.title,
                    "body": hit.entry.body,
                    "similarity": (hit.similarity * 1000.0).round() / 1000.0,
                    "sources": hit.entry.sources.iter().map(|source| serde_json::json!({
                        "cli": source.provider.agent(),
                        "session_id": source.session_id,
                        "transcript": source.transcript_path,
                    })).collect::<Vec<_>>(),
                })
            })
            .collect::<Vec<_>>();
        format!("{}\n", serde_json::Value::Array(rows))
    } else if hits.is_empty() {
        format!("{}\n", tr("一致するナレッジはありません。"))
    } else {
        hits.iter()
            .map(|hit| {
                format!(
                    "[{}] {} ({:.0}%)\n{}\n",
                    hit.entry.kind.label(),
                    hit.entry.title,
                    hit.similarity * 100.0,
                    hit.entry.body
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    };
    CliOutcome {
        code: 0,
        stdout,
        stderr: String::new(),
    }
}

fn not_enabled_message(project: &Project) -> String {
    tf!(
        "{name} ではナレッジの収集が有効になっていません。Operon のプロジェクト › ナレッジ で有効にしてください。",
        name = project.name
    )
}

/// The entry `src/main.rs` calls when the first argument is
/// `KNOWLEDGE_CLI_SUBCOMMAND`. Opens no window and takes no instance lock; it
/// reads the store read-only and writes only reference counts.
pub(crate) fn run_cli(arguments: &[String]) -> i32 {
    if let [subcommand, fixture] = arguments {
        if subcommand != KNOWLEDGE_EMBED_GOLDEN_SUBCOMMAND {
            return run_search_cli(arguments);
        }
        let embedder = SharedEmbedder::installed();
        let (code, report) = embed_golden(
            Path::new(fixture),
            embedder.as_ref().map(|embedder| embedder as &dyn Embedder),
        );
        print!("{report}");
        return code;
    }
    run_search_cli(arguments)
}

fn run_search_cli(arguments: &[String]) -> i32 {
    let directory = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let projects = match load_store(&app_data_file()) {
        Ok(store) => store.projects,
        Err(error) => {
            eprintln!("{error}");
            return 1;
        }
    };
    let embedder = SharedEmbedder::installed();
    let outcome = knowledge_cli(
        arguments,
        &directory,
        &projects,
        &knowledge_root(),
        embedder.as_ref().map(|embedder| embedder as &dyn Embedder),
        now(),
    );
    print!("{}", outcome.stdout);
    eprint!("{}", outcome.stderr);
    outcome.code
}

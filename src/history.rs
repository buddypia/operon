use crate::*;

pub(crate) fn import_full_cli_history(
    project: &Project,
    source: &CliSession,
    target: &str,
    archive_root: &Path,
    import_id: Uuid,
) -> UiResult<ImportedNativeSession> {
    let destination = full_history_restore_destination(target)?;
    let source_transcript = full_history_restore_transcript(source)?;
    let archive_path = snapshot_native_transcript(&source_transcript, archive_root, import_id)
        .map_err(|error| error.to_string())?;
    // Read the snapshot once, here. Every destination restores this exact
    // projection, and the verification each one runs compares against the same
    // list, so a destination cannot disagree with the reader about what the
    // conversation was.
    let (turns, classification) = read_transcript_conversation(&archive_path, source.provider)?;
    if turns.is_empty() {
        return Err(tr("選択した元履歴には、復元できる記録がありません。").into());
    }
    let manifest = serde_json::json!({
        // 5, because `unclassified_kinds` stopped being `{kind, count}`: a
        // count that did not say whether the record crossed was the whole of
        // change 061's second defect, and a reader of an old manifest must be
        // able to tell that its counts cannot answer that question.
        "schema_version": 5,
        "source_provider": source.provider.label(),
        "source_session_id": source.native_id,
        "source_path": source_transcript,
        "project": project.path,
        "created_at": now(),
        "destination_provider": destination.label(),
        "restore": match destination {
            CliProvider::Codex => "Codex app-server thread/inject_items",
            CliProvider::Gemini => tr("元ファイルから書き出した Antigravity の会話ストア"),
            CliProvider::Claude => tr("元ファイルから書き出した Claude Code の会話ファイル"),
        },
        // What the restore did with every source record. Written beside the
        // snapshot so the accounting survives the notice being dismissed.
        "classification": {
            "records": classification.records,
            "restored": classification.restored,
            "operational": classification.operational,
            "unclassified": classification.unclassified,
            "empty": classification.empty,
            "reasoning_fragments": classification.reasoning_fragments,
            "unclassified_kinds": classification
                .unclassified_kinds
                .iter()
                .map(|kind| serde_json::json!({
                    "kind": kind.name,
                    "count": kind.met(),
                    "carried": kind.carried,
                    "dropped": kind.dropped,
                }))
                .collect::<Vec<_>>(),
        },
    });
    let manifest_path = archive_path
        .parent()
        .ok_or_else(|| {
            tr("共有セッションのスナップショットに親ディレクトリがありません。").to_owned()
        })?
        .join("manifest.json");
    let manifest_bytes = serde_json::to_vec_pretty(&manifest).map_err(|error| error.to_string())?;
    write_file_atomically(&manifest_path, &manifest_bytes).map_err(|error| error.to_string())?;

    let mut imported = match destination {
        CliProvider::Codex => {
            import_full_history_into_codex(project, &turns, &archive_path, import_id)
        }
        CliProvider::Gemini => import_full_history_into_antigravity(
            project,
            source.provider,
            &turns,
            &archive_path,
            import_id,
        ),
        CliProvider::Claude => import_full_history_into_claude(
            project,
            source.provider,
            &turns,
            &archive_path,
            import_id,
        ),
    }?;
    imported.classification = classification;
    Ok(imported)
}

/// Only providers with a way to create a real conversation that already holds
/// the restored history are offered. Every supported CLI now has one, so the
/// restriction only rejects agents Operon cannot restore into at all.
pub(crate) fn full_history_restore_destination(target: &str) -> UiResult<CliProvider> {
    match target {
        "codex" => Ok(CliProvider::Codex),
        "claude" => Ok(CliProvider::Claude),
        "gemini" => Ok(CliProvider::Gemini),
        _ => Err(tr("対応していない会話全履歴の復元先です。").into()),
    }
}

/// Every supported CLI already stores its conversations locally, so a restore
/// reads those files instead of replaying the conversation through a model.
/// Codex and Claude keep one JSONL per conversation and the discovered session
/// already points at it; Antigravity's `history.jsonl` is only a global index
/// of submitted turns, so its per-conversation transcript has to be resolved.
pub(crate) fn full_history_restore_transcript(source: &CliSession) -> UiResult<PathBuf> {
    let transcript = match source.provider {
        CliProvider::Claude | CliProvider::Codex => source.path.clone(),
        // Discovery already points at the transcript when Antigravity has
        // written one. A session resolved right after launch cannot, so fall
        // back to the conversation's known location in the store.
        CliProvider::Gemini if is_antigravity_conversation_transcript(&source.path) => {
            source.path.clone()
        }
        CliProvider::Gemini => antigravity_conversation_transcript_path(&source.native_id)?,
    };
    if !transcript.is_file() {
        return Err(tf!(
            "{p0} には、会話 {native_id}（{p2}）の読み取り可能な履歴がありません。",
            native_id = source.native_id,
            p0 = source.provider.label(),
            p2 = transcript.display()
        ));
    }
    Ok(transcript)
}

pub(crate) fn is_antigravity_conversation_transcript(path: &Path) -> bool {
    path.file_name()
        .is_some_and(|name| name == ANTIGRAVITY_TRANSCRIPT_FILE_NAME)
}

/// Name the request a restore is carrying over. A discovered Claude or Codex
/// session already knows it, because their per-conversation JSONL is what the
/// scan reads. Antigravity's does not: its `history.jsonl` is a global index
/// that only names a conversation from its second turn on, and a managed `agy`
/// terminal launched without a goal has no stored message either. Both cases
/// leave the source without a message, so read the last request back from the
/// same transcript the restore itself reads.
pub(crate) fn restore_source_last_user_message(
    source: &CliSession,
    transcript: &Path,
) -> Option<String> {
    source
        .last_user_message
        .as_deref()
        .and_then(nonempty_transcript_text)
        .or_else(|| transcript_last_user_message(transcript, source.provider))
}

/// The last thing the person said in a conversation, read from its provider's
/// own transcript. Bounded by the same per-file budgets as every other
/// transcript read, so an oversized log cannot turn a restore into an unbounded
/// scan.
pub(crate) fn transcript_last_user_message(
    transcript: &Path,
    provider: CliProvider,
) -> Option<String> {
    let file = fs::File::open(transcript).ok()?;
    let mut reader = BufReader::new(file.take(TRANSCRIPT_FILE_MAX_BYTES));
    let mut line = String::new();
    let mut last = None;
    for _ in 0..TRANSCRIPT_FILE_LINE_LIMIT {
        line.clear();
        match reader.read_line(&mut line) {
            Ok(0) | Err(_) => break,
            Ok(_) => {}
        }
        if let Some((role, text)) =
            transcript_conversation_turn(line.trim_end_matches(['\n', '\r']), provider)
        {
            if role == "user" {
                last = Some(text);
            }
        }
    }
    last
}

pub(crate) fn import_full_history_into_codex(
    project: &Project,
    turns: &[(&'static str, String)],
    archive_path: &Path,
    import_id: Uuid,
) -> UiResult<ImportedNativeSession> {
    let mut app_server = CodexAppServer::start().map_err(|error| error.to_string())?;
    let result = (|| -> UiResult<(String, PathBuf, usize)> {
        app_server.initialize()?;
        let (native_session_id, native_session_path) = app_server.start_thread(&project.path)?;
        let imported_records = app_server.inject_turns(turns)?;
        if imported_records == 0 {
            return Err(tr("選択した元履歴には、復元できる記録がありません。").into());
        }
        let native_session_path = wait_for_codex_session_file(&native_session_path)?;
        verify_codex_imported_transcript(&native_session_path, turns)?;
        Ok((native_session_id, native_session_path, imported_records))
    })();
    // The app server owns the rollout writer while it runs. Stop it before
    // adding the UI history so the file has exactly one writer at a time.
    app_server.stop();
    let (native_session_id, native_session_path, imported_records) = result?;
    let replayable = add_codex_replayable_history(&native_session_path, turns).and_then(|ui_records| {
            if ui_records != imported_records {
                return Err(tf!("モデルから見えるターンを {imported_records} 件復元しましたが、再生可能なターンは {ui_records} 件でした", imported_records = imported_records, ui_records = ui_records));
            }
            verify_codex_replayable_history(&native_session_path, turns)
        });
    if let Err(error) = replayable {
        return Err(tf!("Codex はスレッド {native_session_id} に復元履歴を受け入れましたが、Operon が再生可能にできなかったため、再開可能なセッションとしては提供しません: {error}", error = error, native_session_id = native_session_id));
    }
    Ok(ImportedNativeSession {
        managed_session_id: import_id,
        provider: CliProvider::Codex,
        agent_command: CliProvider::Codex.native_resume_command(&native_session_id),
        native_session_id: Some(native_session_id),
        native_session_path,
        archive_path: archive_path.to_path_buf(),
        imported_records,
        classification: RestoreClassification::default(),
    })
}

/// Antigravity keeps every conversation in its own local SQLite trajectory
/// store and offers no import flag. A conversation it can reopen is, however,
/// just a row set in that store, so Operon writes one: the restored turns
/// are the conversation itself, not a prompt describing it. Nothing is sent to
/// a model, so the restore costs no tokens and cannot paraphrase, summarize, or
/// act on the history it is restoring.
pub(crate) fn import_full_history_into_antigravity(
    project: &Project,
    source_provider: CliProvider,
    turns: &[(&'static str, String)],
    archive_path: &Path,
    import_id: Uuid,
) -> UiResult<ImportedNativeSession> {
    let archive_directory = archive_path.parent().ok_or_else(|| {
        tr("共有セッションのスナップショットに親ディレクトリがありません。").to_owned()
    })?;
    if turns.is_empty() {
        return Err(tr("選択した元履歴には、復元できる記録がありません。").into());
    }
    let document = render_restored_conversation(turns, source_provider);
    write_file_atomically(
        &archive_directory.join("restored-conversation.md"),
        document.as_bytes(),
    )
    .map_err(|error| error.to_string())?;
    let conversation_id = Uuid::new_v4().to_string();
    let conversation_path = antigravity_conversation_path(&conversation_id)?;
    let history_index = antigravity_history_index_path()?;
    write_antigravity_conversation(&conversation_path, &conversation_id, turns, now())?;
    // A conversation Antigravity cannot read back is worse than no restore at
    // all, so drop it again rather than offer an unusable resume identity.
    if let Err(error) = verify_antigravity_conversation(&conversation_path, turns) {
        let _ = fs::remove_file(&conversation_path);
        return Err(error);
    }
    append_antigravity_history_entry(
        &history_index,
        &project.path,
        &conversation_id,
        turns,
        now(),
    )?;
    Ok(ImportedNativeSession {
        managed_session_id: import_id,
        provider: CliProvider::Gemini,
        agent_command: CliProvider::Gemini.native_resume_command(&conversation_id),
        native_session_id: Some(conversation_id),
        native_session_path: conversation_path,
        archive_path: archive_path.to_path_buf(),
        imported_records: turns.len(),
        classification: RestoreClassification::default(),
    })
}

/// Claude Code keeps one JSONL per conversation under its own projects
/// directory and reopens whatever it finds there, so a restore writes that
/// file: the restored turns become the conversation itself. Checked against
/// Claude Code 2.1.234 — a conversation written this way is offered by
/// `claude --resume`, replayed on screen, and handed to the model as real
/// context, so nothing is summarized and the restore costs no tokens.
pub(crate) fn import_full_history_into_claude(
    project: &Project,
    source_provider: CliProvider,
    turns: &[(&'static str, String)],
    archive_path: &Path,
    import_id: Uuid,
) -> UiResult<ImportedNativeSession> {
    let archive_directory = archive_path.parent().ok_or_else(|| {
        tr("共有セッションのスナップショットに親ディレクトリがありません。").to_owned()
    })?;
    if turns.is_empty() {
        return Err(tr("選択した元履歴には、復元できる記録がありません。").into());
    }
    let document = render_restored_conversation(turns, source_provider);
    write_file_atomically(
        &archive_directory.join("restored-conversation.md"),
        document.as_bytes(),
    )
    .map_err(|error| error.to_string())?;
    let session_id = Uuid::new_v4().to_string();
    let workspace = claude_conversation_workspace(&project.path);
    let conversation_path = claude_conversation_path(&workspace, &session_id)?;
    write_claude_conversation(&conversation_path, &session_id, &workspace, turns, now())?;
    // A conversation Claude Code cannot read back is worse than no restore at
    // all, so drop it again rather than offer an unusable resume identity.
    if let Err(error) = verify_claude_conversation(&conversation_path, &session_id, turns) {
        let _ = fs::remove_file(&conversation_path);
        return Err(error);
    }
    Ok(ImportedNativeSession {
        managed_session_id: import_id,
        provider: CliProvider::Claude,
        agent_command: CliProvider::Claude.native_resume_command(&session_id),
        native_session_id: Some(session_id),
        native_session_path: conversation_path,
        archive_path: archive_path.to_path_buf(),
        imported_records: turns.len(),
        classification: RestoreClassification::default(),
    })
}

/// What a restore did with every record in the source transcript.
///
/// Every record is counted into exactly one of the four record buckets, so the
/// arithmetic itself says whether anything went missing —
/// `restore_counts_every_source_record_into_exactly_one_class` asserts it. The
/// two fragment counters sit beside that identity rather than inside it: one
/// record can carry several reasoning or unclassified blocks.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct RestoreClassification {
    /// Every record read from the source.
    pub(crate) records: usize,
    /// Records that produced a turn for the destination.
    pub(crate) restored: usize,
    /// Records `TRANSCRIPT_VOCABULARY` declares are not conversation.
    pub(crate) operational: usize,
    /// Records whose kind has no row in `TRANSCRIPT_VOCABULARY`.
    pub(crate) unclassified: usize,
    /// Records that are conversation but held nothing readable.
    pub(crate) empty: usize,
    /// Reasoning fragments carried across as attributed text.
    pub(crate) reasoning_fragments: usize,
    /// Fragments of any layer whose kind has no row in the vocabulary, counted
    /// per `provider/kind` so the report can name what to classify next.
    pub(crate) unclassified_kinds: Vec<UnclassifiedKind>,
    /// Kinds met while reading the record currently in hand, whose arm the
    /// record's own outcome decides. Emptied into `unclassified_kinds` by
    /// `settle_deferred_kinds` the moment that outcome exists, so it is empty
    /// between records and a reader never sees it hold anything.
    deferred_kinds: Vec<String>,
}

/// What became of a fragment whose kind has no row in `TRANSCRIPT_VOCABULARY`.
///
/// The distinction is the whole of change 061. Meeting an unknown kind and
/// *carrying it anyway* is the promise `.claude/rules/transcripts.md` makes;
/// meeting one and having nothing readable to carry is a record that did not
/// cross. Reporting both as the first is how 2385 records in one Codex thread
/// were dropped under a sentence saying they had been handed over.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum UnclassifiedOutcome {
    /// It crossed, under a label naming its kind.
    Carried,
    /// It held nothing readable, so nothing crossed.
    Dropped,
}

/// One kind the vocabulary has never met, and what happened to the fragments
/// that carried it. A kind can appear on both arms — a CLI is free to write the
/// same record type with and without text — so both counts are kept per kind
/// rather than one count and a verdict.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct UnclassifiedKind {
    /// `provider/kind`, which is the name to add to `TRANSCRIPT_VOCABULARY`.
    pub(crate) name: String,
    /// Fragments of this kind that crossed, labelled.
    pub(crate) carried: usize,
    /// Fragments of this kind that held nothing readable.
    pub(crate) dropped: usize,
}

impl UnclassifiedKind {
    /// Every fragment of this kind, whichever arm it took.
    pub(crate) fn met(&self) -> usize {
        self.carried + self.dropped
    }
}

impl RestoreClassification {
    fn count_unclassified_kind(
        &mut self,
        provider: CliProvider,
        kind: &str,
        outcome: UnclassifiedOutcome,
    ) {
        self.count_named_kind(format!("{}/{}", provider.agent(), kind), outcome);
    }

    fn count_named_kind(&mut self, name: String, outcome: UnclassifiedOutcome) {
        let index = self
            .unclassified_kinds
            .iter()
            .position(|seen| seen.name == name);
        let index = match index {
            Some(index) => index,
            None => {
                self.unclassified_kinds.push(UnclassifiedKind {
                    name,
                    ..UnclassifiedKind::default()
                });
                self.unclassified_kinds.len() - 1
            }
        };
        let entry = &mut self.unclassified_kinds[index];
        match outcome {
            UnclassifiedOutcome::Carried => entry.carried += 1,
            UnclassifiedOutcome::Dropped => entry.dropped += 1,
        }
    }

    /// Note a kind whose arm this reader cannot yet decide, because what decides
    /// it is the outcome of the record it sits on.
    ///
    /// Two kinds are like this, and they are the two that sit outside
    /// `unclassified_record`: an unknown `is…` flag and an unknown role. Neither
    /// stops the record — that is why they used to be counted `Carried` on the
    /// spot — but neither makes it cross either. The record can still turn out
    /// to hold nothing readable and return `Empty` a few lines later, and then
    /// the notice has told a person that this kind's content reached the
    /// destination when no content did.
    ///
    /// Counting before the branch that decides is the defect this whole change
    /// is about; `unclassified_record`'s doc comment states the rule. These two
    /// sites cannot apply it in place, because the thing that decides is not
    /// theirs to ask. So they defer, and `settle_deferred_kinds` counts them
    /// against what actually happened.
    fn defer_unclassified_kind(&mut self, provider: CliProvider, kind: &str) {
        self.deferred_kinds
            .push(format!("{}/{}", provider.agent(), kind));
    }

    /// Count everything deferred while reading one record, against that
    /// record's outcome. `Turn` is the only outcome that crossed.
    ///
    /// Called from `transcript_value_record`, which is the single funnel all
    /// three providers' readers return through — so a site that defers cannot
    /// forget to settle, and a reader added later inherits the behaviour
    /// without knowing this exists.
    fn settle_deferred_kinds(&mut self, outcome: &RecordOutcome) {
        if self.deferred_kinds.is_empty() {
            return;
        }
        let arm = match outcome {
            RecordOutcome::Turn(..) => UnclassifiedOutcome::Carried,
            RecordOutcome::Operational | RecordOutcome::Empty | RecordOutcome::Unclassified => {
                UnclassifiedOutcome::Dropped
            }
        };
        for name in std::mem::take(&mut self.deferred_kinds) {
            self.count_named_kind(name, arm);
        }
    }

    /// Whether the source held a kind this build has never been taught. The one
    /// signal that says `TRANSCRIPT_VOCABULARY` has fallen behind a CLI release.
    pub(crate) fn has_unclassified(&self) -> bool {
        !self.unclassified_kinds.is_empty()
    }

    /// The unknown kinds on one arm, rendered `name (count)` for the notice.
    /// Empty when no kind took that arm, which is how the notice knows not to
    /// print the sentence about it.
    fn unclassified_named(&self, outcome: UnclassifiedOutcome) -> String {
        self.unclassified_kinds
            .iter()
            .filter_map(|kind| {
                let count = match outcome {
                    UnclassifiedOutcome::Carried => kind.carried,
                    UnclassifiedOutcome::Dropped => kind.dropped,
                };
                (count > 0).then(|| format!("{} ({count})", kind.name))
            })
            .collect::<Vec<_>>()
            .join(" · ")
    }
}

/// What the restore carried and what it left, for the notice.
///
/// The point is the arithmetic, not the reassurance: a person who restores a
/// long conversation and is told only "471 turns" has no way to know whether
/// that was all of it. Naming the excluded records alongside the carried ones is
/// what makes an unexpected number visible as unexpected.
///
/// So the first sentence prints all four record buckets and not the three that
/// flatter: `records == restored + operational + unclassified + empty` is the
/// identity `restore_counts_every_source_record_into_exactly_one_class` asserts
/// on the struct, and printing only part of it left a person doing subtraction
/// that did not come out.
///
/// And an unknown kind gets a sentence per arm rather than one for both. A kind
/// that crossed labelled and a kind that held nothing to cross are different
/// news — one is "teach the table", the other is "this did not arrive" — and
/// saying the first of both is how a dropped record reads as a delivered one.
pub(crate) fn restore_classification_notice(classification: &RestoreClassification) -> String {
    if classification.records == 0 {
        return String::new();
    }
    let mut notice = tf!(
        "（元の記録 {records} 件から会話 {restored} 件・思考 {reasoning} 件を取り、運用レコード {operational} 件・未分類 {unclassified} 件・本文なし {empty} 件は除外）",
        records = classification.records,
        restored = classification.restored,
        reasoning = classification.reasoning_fragments,
        operational = classification.operational,
        unclassified = classification.unclassified,
        empty = classification.empty
    );
    let carried = classification.unclassified_named(UnclassifiedOutcome::Carried);
    if !carried.is_empty() {
        // The claim is delivery and nothing more. An earlier cut of this
        // sentence promised the `[Unclassified: …]` heading by name, and two of
        // the four sites that record `Carried` do not attach one: an unknown
        // `is…` flag does not stop the record it sits on, and an unknown role
        // renders as an ordinary assistant turn. Both cross under their own
        // label, so a person sent looking for the heading to check the claim
        // would find nothing — the same defect as the sentence this change
        // exists to correct, on the two sites that sit outside
        // `unclassified_record`. Guarded by
        // `the_notice_promises_no_marker_the_restored_text_does_not_carry`.
        notice.push_str(&tf!(
            "Operon が分類を知らない記録がありました。内容は復元先へ渡しています: {kinds}。",
            kinds = carried
        ));
    }
    let dropped = classification.unclassified_named(UnclassifiedOutcome::Dropped);
    if !dropped.is_empty() {
        notice.push_str(&tf!(
            "次の記録は分類が分からず、読み取れる本文もなかったため復元先へ渡していません: {kinds}。",
            kinds = dropped
        ));
    }
    if classification.has_unclassified() {
        notice.push_str(tr(
            "src/transcript.rs の TRANSCRIPT_VOCABULARY に分類を追加してください。",
        ));
    }
    notice
}

/// Read every conversation turn from a snapshotted transcript, and account for
/// everything that was not one.
///
/// This materializes the conversation, so it fails closed rather than silently
/// dropping turns once a transcript exceeds the shared per-file budgets: a
/// partial history must never be presented as a complete restore. The returned
/// `RestoreClassification` is the other half of that promise — under the budget
/// the read is complete, and the summary says what the completeness consists
/// of, so "restored 471 turns" can no longer hide "and lost 694 records".
pub(crate) fn read_transcript_conversation(
    transcript: &Path,
    provider: CliProvider,
) -> UiResult<(Vec<(&'static str, String)>, RestoreClassification)> {
    let file = fs::File::open(transcript).map_err(|error| {
        tf!(
            "履歴 {p0} を開けません: {error}",
            error = error,
            p0 = transcript.display()
        )
    })?;
    let mut reader = BufReader::new(file);
    let mut line = String::new();
    let mut turns = Vec::new();
    let mut summary = RestoreClassification::default();
    let mut retained_bytes = 0_u64;
    for _ in 0..TRANSCRIPT_FILE_LINE_LIMIT {
        line.clear();
        let read = reader.read_line(&mut line).map_err(|error| {
            tf!(
                "履歴 {p0} を読み取れません: {error}",
                error = error,
                p0 = transcript.display()
            )
        })?;
        if read == 0 {
            return Ok((turns, summary));
        }
        let raw = line.trim_end_matches(['\n', '\r']);
        // A blank line is not a record. Counting it would make the arithmetic
        // report a loss that never happened.
        if raw.trim().is_empty() {
            continue;
        }
        summary.records += 1;
        match transcript_record(raw, provider, &mut summary) {
            RecordOutcome::Turn(role, text) => {
                summary.restored += 1;
                retained_bytes = retained_bytes.saturating_add(text.len() as u64);
                if retained_bytes > TRANSCRIPT_FILE_MAX_BYTES {
                    return Err(tf!(
                        "この会話は {p0} MiB を超えるターンを含むため、Operon は一度に復元できません。",
                        p0 = TRANSCRIPT_FILE_MAX_BYTES / (1024 * 1024)
                    ));
                }
                turns.push((role, text));
            }
            RecordOutcome::Operational => summary.operational += 1,
            RecordOutcome::Empty => summary.empty += 1,
            RecordOutcome::Unclassified => summary.unclassified += 1,
        }
    }
    Err(tf!("この会話は {TRANSCRIPT_FILE_LINE_LIMIT} 件を超える記録を含むため、Operon は一度に復元できません。", TRANSCRIPT_FILE_LINE_LIMIT = TRANSCRIPT_FILE_LINE_LIMIT))
}

/// The turns alone. Every caller in the app reads the accounting alongside
/// them — a restore that does not know what it left behind is the defect this
/// module was rewritten to remove — so this exists for the tests that are only
/// about content.
#[cfg(test)]
pub(crate) fn read_transcript_conversation_turns(
    transcript: &Path,
    provider: CliProvider,
) -> UiResult<Vec<(&'static str, String)>> {
    read_transcript_conversation(transcript, provider).map(|(turns, _)| turns)
}

/// Render the restored conversation as one readable document. Antigravity has
/// to receive the history as conversation text, so the rendering has to stay
/// unambiguous about who said what.
pub(crate) fn render_restored_conversation(
    turns: &[(&'static str, String)],
    provider: CliProvider,
) -> String {
    let mut document = tf!(
        "# 復元された会話全履歴（{p0} / {p1} 件のターン）\n",
        p0 = provider.label(),
        p1 = turns.len()
    );
    for (index, (role, text)) in turns.iter().enumerate() {
        let speaker = if *role == "user" {
            tr("ユーザー")
        } else {
            tr("アシスタント")
        };
        document.push_str(&tf!(
            "\n## ターン {p0} — {speaker}\n\n{text}\n",
            p0 = index + 1,
            speaker = speaker,
            text = text
        ));
    }
    document
}

/// Antigravity records a step's origin here: a real user message and a model
/// response are distinguished by this field, not by the step type alone.
pub(crate) const ANTIGRAVITY_STEP_SOURCE_MODEL: u64 = 2;
pub(crate) const ANTIGRAVITY_STEP_SOURCE_USER: u64 = 4;
pub(crate) const ANTIGRAVITY_STEP_TYPE_USER_INPUT: u64 = 14;
pub(crate) const ANTIGRAVITY_STEP_TYPE_PLANNER_RESPONSE: u64 = 15;
pub(crate) const ANTIGRAVITY_STEP_STATUS_DONE: u64 = 3;
/// `trajectory_type` 4 / `source` 17 is what the CLI writes for a conversation
/// started from a terminal, which is exactly what a restored conversation is.
pub(crate) const ANTIGRAVITY_TRAJECTORY_TYPE_CLI: u64 = 4;
pub(crate) const ANTIGRAVITY_TRAJECTORY_SOURCE_CLI: u64 = 17;

/// Antigravity's own table definitions, reproduced verbatim so a restored
/// conversation is indistinguishable from one the CLI created itself. Tables
/// the CLI recreates on demand are still declared: an unexpected missing table
/// would otherwise surface as a query error inside the destination CLI.
pub(crate) const ANTIGRAVITY_TRAJECTORY_SCHEMA: &str = "\
CREATE TABLE `trajectory_meta` (`trajectory_id` text,`cascade_id` text,`trajectory_type` integer,`source` integer,PRIMARY KEY (`trajectory_id`));
CREATE TABLE `steps` (`idx` integer,`step_type` integer NOT NULL DEFAULT 0,`status` integer NOT NULL DEFAULT 0,`has_subtrajectory` numeric NOT NULL DEFAULT false,`metadata` blob,`error_details` blob,`permissions` blob,`task_details` blob,`render_info` blob,`step_payload` blob,`step_format` integer NOT NULL DEFAULT 0,PRIMARY KEY (`idx`));
CREATE INDEX `idx_steps_status` ON `steps`(`status`);
CREATE INDEX `idx_steps_step_type` ON `steps`(`step_type`);
CREATE TABLE `gen_metadata` (`idx` integer,`data` blob,`size` integer NOT NULL DEFAULT 0,PRIMARY KEY (`idx`));
CREATE TABLE `executor_metadata` (`idx` integer,`data` blob,PRIMARY KEY (`idx`));
CREATE TABLE `parent_references` (`idx` integer,`data` blob,PRIMARY KEY (`idx`));
CREATE TABLE `trajectory_metadata_blob` (`id` text DEFAULT \"main\",`data` blob,PRIMARY KEY (`id`));
CREATE TABLE `battle_mode_infos` (`idx` integer,`data` blob,PRIMARY KEY (`idx`));
";

pub(crate) fn proto_push_varint(value: u64, out: &mut Vec<u8>) {
    let mut value = value;
    loop {
        let byte = (value & 0x7f) as u8;
        value >>= 7;
        if value == 0 {
            out.push(byte);
            return;
        }
        out.push(byte | 0x80);
    }
}

pub(crate) fn proto_push_varint_field(field: u32, value: u64, out: &mut Vec<u8>) {
    proto_push_varint(u64::from(field) << 3, out);
    proto_push_varint(value, out);
}

pub(crate) fn proto_push_bytes_field(field: u32, value: &[u8], out: &mut Vec<u8>) {
    proto_push_varint((u64::from(field) << 3) | 2, out);
    proto_push_varint(value.len() as u64, out);
    out.extend_from_slice(value);
}

/// Antigravity timestamps are seconds plus nanoseconds; a restored step has no
/// sub-second precision to preserve, so only the seconds field is written.
pub(crate) fn proto_timestamp(seconds: u64) -> Vec<u8> {
    let mut out = Vec::new();
    proto_push_varint_field(1, seconds, &mut out);
    out
}

/// The step metadata Antigravity reads back when it reopens a conversation:
/// when the step happened, where it came from, and which trajectory position it
/// occupies. Field numbers come from the steps the CLI writes itself.
pub(crate) fn antigravity_step_metadata(
    created_at: u64,
    source: u64,
    trajectory_id: &str,
    conversation_id: &str,
    index: usize,
) -> Vec<u8> {
    let mut step_id = Vec::new();
    proto_push_bytes_field(1, trajectory_id.as_bytes(), &mut step_id);
    // Protobuf omits a zero-valued scalar, and so does Antigravity's own
    // first step. Writing it explicitly would still decode, but keeping the
    // encoding identical avoids a needless difference from real steps.
    if index > 0 {
        proto_push_varint_field(2, index as u64, &mut step_id);
    }
    proto_push_bytes_field(4, conversation_id.as_bytes(), &mut step_id);

    let mut status_entry = Vec::new();
    proto_push_varint_field(1, ANTIGRAVITY_STEP_STATUS_DONE, &mut status_entry);
    proto_push_bytes_field(2, &proto_timestamp(created_at), &mut status_entry);
    let mut status_history = Vec::new();
    proto_push_bytes_field(1, &status_entry, &mut status_history);

    let mut metadata = Vec::new();
    proto_push_bytes_field(1, &proto_timestamp(created_at), &mut metadata);
    proto_push_varint_field(3, source, &mut metadata);
    if source == ANTIGRAVITY_STEP_SOURCE_MODEL {
        proto_push_bytes_field(8, &proto_timestamp(created_at), &mut metadata);
    }
    proto_push_bytes_field(20, &step_id, &mut metadata);
    proto_push_bytes_field(26, &status_history, &mut metadata);
    proto_push_bytes_field(32, &proto_timestamp(created_at), &mut metadata);
    metadata
}

/// Encode one restored turn as the step payload Antigravity stores. A user turn
/// carries its text twice (plain and structured) exactly as the CLI writes it,
/// because the terminal renders one copy and the model context reads the other.
pub(crate) fn antigravity_step_payload(
    role: &str,
    text: &str,
    metadata: &[u8],
    index: usize,
) -> (u64, Vec<u8>) {
    let mut payload = Vec::new();
    if role == "user" {
        let mut inner = Vec::new();
        proto_push_bytes_field(2, text.as_bytes(), &mut inner);
        let mut structured = Vec::new();
        proto_push_bytes_field(1, text.as_bytes(), &mut structured);
        proto_push_bytes_field(3, &structured, &mut inner);
        proto_push_bytes_field(4, b"", &mut inner);
        proto_push_varint_field(1, ANTIGRAVITY_STEP_TYPE_USER_INPUT, &mut payload);
        proto_push_varint_field(4, ANTIGRAVITY_STEP_STATUS_DONE, &mut payload);
        proto_push_bytes_field(5, metadata, &mut payload);
        proto_push_bytes_field(19, &inner, &mut payload);
        return (ANTIGRAVITY_STEP_TYPE_USER_INPUT, payload);
    }
    let mut inner = Vec::new();
    proto_push_bytes_field(1, text.as_bytes(), &mut inner);
    // Antigravity labels each response with a per-response identifier. It is
    // opaque, so derive a stable one from the step position rather than
    // inventing an identity that looks like it came from the provider.
    proto_push_bytes_field(6, format!("operon-restored-{index}").as_bytes(), &mut inner);
    proto_push_bytes_field(8, text.as_bytes(), &mut inner);
    proto_push_varint_field(12, 2, &mut inner);
    proto_push_varint_field(1, ANTIGRAVITY_STEP_TYPE_PLANNER_RESPONSE, &mut payload);
    proto_push_varint_field(4, ANTIGRAVITY_STEP_STATUS_DONE, &mut payload);
    proto_push_bytes_field(5, metadata, &mut payload);
    proto_push_bytes_field(20, &inner, &mut payload);
    (ANTIGRAVITY_STEP_TYPE_PLANNER_RESPONSE, payload)
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum ProtoValue<'a> {
    Varint(u64),
    Bytes(&'a [u8]),
    Fixed64,
    Fixed32,
}

/// Decode a protobuf message far enough to read a restored step back. This
/// exists for verification: a step Operon cannot decode from the persisted
/// bytes is a step the destination CLI may not be able to decode either.
pub(crate) fn proto_fields(mut bytes: &[u8]) -> Option<Vec<(u32, ProtoValue<'_>)>> {
    let mut fields = Vec::new();
    while !bytes.is_empty() {
        let (key, rest) = proto_read_varint(bytes)?;
        let field = u32::try_from(key >> 3).ok()?;
        if field == 0 {
            return None;
        }
        bytes = match key & 7 {
            0 => {
                let (value, rest) = proto_read_varint(rest)?;
                fields.push((field, ProtoValue::Varint(value)));
                rest
            }
            1 => {
                fields.push((field, ProtoValue::Fixed64));
                rest.get(8..)?
            }
            2 => {
                let (length, rest) = proto_read_varint(rest)?;
                let length = usize::try_from(length).ok()?;
                fields.push((field, ProtoValue::Bytes(rest.get(..length)?)));
                rest.get(length..)?
            }
            5 => {
                fields.push((field, ProtoValue::Fixed32));
                rest.get(4..)?
            }
            _ => return None,
        };
    }
    Some(fields)
}

pub(crate) fn proto_read_varint(bytes: &[u8]) -> Option<(u64, &[u8])> {
    let mut value = 0_u64;
    for (index, byte) in bytes.iter().take(10).enumerate() {
        value |= u64::from(byte & 0x7f).checked_shl(7 * index as u32)?;
        if byte & 0x80 == 0 {
            return Some((value, &bytes[index + 1..]));
        }
    }
    None
}

pub(crate) fn proto_varint_value(fields: &[(u32, ProtoValue<'_>)], field: u32) -> Option<u64> {
    fields.iter().rev().find_map(|(number, value)| match value {
        ProtoValue::Varint(value) if *number == field => Some(*value),
        _ => None,
    })
}

pub(crate) fn proto_bytes_value<'a>(
    fields: &[(u32, ProtoValue<'a>)],
    field: u32,
) -> Option<&'a [u8]> {
    fields.iter().rev().find_map(|(number, value)| match value {
        ProtoValue::Bytes(value) if *number == field => Some(*value),
        _ => None,
    })
}

/// Read one restored turn back out of a persisted step payload.
pub(crate) fn antigravity_step_turn(payload: &[u8]) -> Option<(&'static str, String)> {
    let fields = proto_fields(payload)?;
    let (role, message_field, text_field) = match proto_varint_value(&fields, 1)? {
        ANTIGRAVITY_STEP_TYPE_USER_INPUT => ("user", 19, 2),
        ANTIGRAVITY_STEP_TYPE_PLANNER_RESPONSE => ("assistant", 20, 1),
        _ => return None,
    };
    let message = proto_fields(proto_bytes_value(&fields, message_field)?)?;
    let text = proto_bytes_value(&message, text_field)?;
    Some((role, String::from_utf8(text.to_vec()).ok()?))
}

/// Write the restored conversation into Antigravity's own conversation store.
/// The file is built under a temporary name and renamed into place, so `agy`
/// never observes a half-written conversation at a resumable path.
pub(crate) fn write_antigravity_conversation(
    path: &Path,
    conversation_id: &str,
    turns: &[(&'static str, String)],
    created_at: u64,
) -> UiResult<()> {
    let directory = path
        .parent()
        .ok_or_else(|| tr("Antigravity の会話ストアに親ディレクトリがありません。").to_owned())?;
    fs::create_dir_all(directory).map_err(|error| {
        tf!(
            "Antigravity の会話ストア {p0} を作成できません: {error}",
            error = error,
            p0 = directory.display()
        )
    })?;
    if path.exists() {
        return Err(tf!(
            "Antigravity には既に {p0} に会話があります。",
            p0 = path.display()
        ));
    }
    let temporary = directory.join(format!(".{conversation_id}.db.tmp-{}", Uuid::new_v4()));
    let result = build_antigravity_conversation(&temporary, conversation_id, turns, created_at)
        .and_then(|()| {
            let file = fs::File::open(&temporary).map_err(|error| {
                tf!(
                    "復元した会話 {p0} を開き直せません: {error}",
                    error = error,
                    p0 = temporary.display()
                )
            })?;
            file.sync_all().map_err(|error| {
                tf!(
                    "復元した会話 {p0} を書き出せません: {error}",
                    error = error,
                    p0 = temporary.display()
                )
            })?;
            fs::rename(&temporary, path).map_err(|error| {
                tf!(
                    "復元した会話 {p0} を確定できません: {error}",
                    error = error,
                    p0 = path.display()
                )
            })?;
            fs::File::open(directory)
                .and_then(|directory| directory.sync_all())
                .map_err(|error| {
                    tf!(
                        "Antigravity の会話ストア {p0} を書き出せません: {error}",
                        error = error,
                        p0 = directory.display()
                    )
                })
        });
    if result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    result
}

pub(crate) fn build_antigravity_conversation(
    path: &Path,
    conversation_id: &str,
    turns: &[(&'static str, String)],
    created_at: u64,
) -> UiResult<()> {
    let mut connection = rusqlite::Connection::open(path).map_err(|error| {
        tf!(
            "復元した会話 {p0} を作成できません: {error}",
            error = error,
            p0 = path.display()
        )
    })?;
    let trajectory_id = Uuid::new_v4().to_string();
    let transaction = (|| -> rusqlite::Result<()> {
        connection.execute_batch(ANTIGRAVITY_TRAJECTORY_SCHEMA)?;
        let transaction = connection.transaction()?;
        transaction.execute(
            "INSERT INTO trajectory_meta (trajectory_id, cascade_id, trajectory_type, source) VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![
                trajectory_id,
                conversation_id,
                ANTIGRAVITY_TRAJECTORY_TYPE_CLI as i64,
                ANTIGRAVITY_TRAJECTORY_SOURCE_CLI as i64
            ],
        )?;
        let mut trajectory_blob = Vec::new();
        proto_push_bytes_field(2, &proto_timestamp(created_at), &mut trajectory_blob);
        proto_push_bytes_field(3, trajectory_id.as_bytes(), &mut trajectory_blob);
        proto_push_bytes_field(6, conversation_id.as_bytes(), &mut trajectory_blob);
        proto_push_bytes_field(18, b"default-cli-project", &mut trajectory_blob);
        transaction.execute(
            "INSERT INTO trajectory_metadata_blob (id, data) VALUES ('main', ?1)",
            rusqlite::params![trajectory_blob],
        )?;
        for (index, (role, text)) in turns.iter().enumerate() {
            let source = if *role == "user" {
                ANTIGRAVITY_STEP_SOURCE_USER
            } else {
                ANTIGRAVITY_STEP_SOURCE_MODEL
            };
            // Keep the recorded order monotonic without pretending to know how
            // long each original turn took, and without dating any step in the
            // future: the conversation ends when it was restored.
            let step_time = created_at.saturating_sub((turns.len() - 1 - index) as u64);
            let metadata = antigravity_step_metadata(
                step_time,
                source,
                &trajectory_id,
                conversation_id,
                index,
            );
            let (step_type, payload) = antigravity_step_payload(role, text, &metadata, index);
            transaction.execute(
                "INSERT INTO steps (idx, step_type, status, has_subtrajectory, metadata, step_payload, step_format) VALUES (?1, ?2, ?3, 0, ?4, ?5, 0)",
                rusqlite::params![
                    index as i64,
                    step_type as i64,
                    ANTIGRAVITY_STEP_STATUS_DONE as i64,
                    metadata,
                    payload
                ],
            )?;
        }
        transaction.commit()
    })();
    let closed = connection.close();
    transaction.map_err(|error| {
        tf!(
            "復元した会話 {p0} を書き込めません: {error}",
            error = error,
            p0 = path.display()
        )
    })?;
    closed.map_err(|(_, error)| {
        tf!(
            "復元した会話 {p0} を閉じられません: {error}",
            error = error,
            p0 = path.display()
        )
    })
}

/// Read the committed conversation back the way Antigravity will. A restore is
/// only complete when the destination CLI's own file holds every source turn in
/// order, so anything less fails the restore instead of exposing a resume ID.
pub(crate) fn verify_antigravity_conversation(
    path: &Path,
    turns: &[(&'static str, String)],
) -> UiResult<()> {
    let connection =
        rusqlite::Connection::open_with_flags(path, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)
            .map_err(|error| {
                tf!(
                    "復元した会話 {p0} を開き直せません: {error}",
                    error = error,
                    p0 = path.display()
                )
            })?;
    let mut statement = connection
        .prepare("SELECT idx, step_payload FROM steps ORDER BY idx")
        .map_err(|error| tf!("復元した会話を読み取れません: {error}", error = error))?;
    let mut rows = statement
        .query([])
        .map_err(|error| tf!("復元した会話を読み取れません: {error}", error = error))?;
    let mut restored = 0_usize;
    while let Some(row) = rows
        .next()
        .map_err(|error| tf!("復元した会話を読み取れません: {error}", error = error))?
    {
        let index: i64 = row
            .get(0)
            .map_err(|error| tf!("復元した会話を読み取れません: {error}", error = error))?;
        let payload: Vec<u8> = row
            .get(1)
            .map_err(|error| tf!("復元した会話を読み取れません: {error}", error = error))?;
        let Some(expected) = turns.get(restored) else {
            return Err(
                tr("復元した Antigravity の会話が、元よりも多くのターンを含んでいます。").into(),
            );
        };
        if index != restored as i64 {
            return Err(tf!(
                "復元した Antigravity の会話が、ステップ {index} で順序どおりではありません。",
                index = index
            ));
        }
        let Some((role, text)) = antigravity_step_turn(&payload) else {
            return Err(tf!(
                "Antigravity のステップ {index} を、書き込み後に読み戻せませんでした。",
                index = index
            ));
        };
        if role != expected.0 || text != expected.1 {
            return Err(tf!(
                "復元した Antigravity の会話は、ステップ {index} の内容が元と異なります。",
                index = index
            ));
        }
        restored += 1;
    }
    if restored != turns.len() {
        return Err(tf!(
            "Antigravity の会話に、{p0} 件中 {restored} 件のターンを復元しました。",
            p0 = turns.len(),
            restored = restored
        ));
    }
    Ok(())
}

/// Antigravity lists a conversation in its own resume picker from this index,
/// and Operon discovers Antigravity sessions the same way. Append one entry
/// so the restored conversation is reachable from both, not just from the
/// resume command Operon stored.
pub(crate) fn append_antigravity_history_entry(
    path: &Path,
    project: &Path,
    conversation_id: &str,
    turns: &[(&'static str, String)],
    created_at: u64,
) -> UiResult<()> {
    let display = turns
        .iter()
        .find(|(role, _)| *role == "user")
        .map(|(_, text)| text.as_str())
        .unwrap_or(tr("復元された会話"));
    let mut line = serde_json::to_vec(&serde_json::json!({
        "display": truncate_chars(display.trim(), ANTIGRAVITY_HISTORY_TITLE_MAX_CHARS),
        "timestamp": created_at.saturating_mul(1_000),
        "workspace": project,
        "conversationId": conversation_id,
    }))
    .map_err(|error| {
        tf!(
            "Antigravity の履歴エントリのエンコード: {error}",
            error = error
        )
    })?;
    line.push(b'\n');
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| {
            tf!(
                "Antigravity の履歴ディレクトリ {p0} を作成できません: {error}",
                error = error,
                p0 = parent.display()
            )
        })?;
    }
    // `agy` owns this file and appends to it from its own processes. A single
    // short append is the only safe way to add an entry from outside.
    let mut file = OpenOptions::new()
        .append(true)
        .create(true)
        .open(path)
        .map_err(|error| {
            tf!(
                "Antigravity の履歴インデックス {p0} を開けません: {error}",
                error = error,
                p0 = path.display()
            )
        })?;
    file.write_all(&line).map_err(|error| {
        tf!(
            "{p0} への復元会話の記録: {error}",
            error = error,
            p0 = path.display()
        )
    })?;
    file.sync_all().map_err(|error| {
        tf!(
            "Antigravity の履歴インデックス {p0} を書き出せません: {error}",
            error = error,
            p0 = path.display()
        )
    })
}

pub(crate) fn antigravity_store_directory() -> UiResult<PathBuf> {
    let home = std::env::var_os("HOME").map(PathBuf::from).ok_or_else(|| {
        tr("ホームディレクトリを取得できないため、Antigravity の会話ストアを特定できません。")
            .to_owned()
    })?;
    Ok(home.join(".gemini").join("antigravity-cli"))
}

pub(crate) fn antigravity_history_index_path() -> UiResult<PathBuf> {
    Ok(antigravity_store_directory()?.join("history.jsonl"))
}

pub(crate) fn antigravity_conversation_path(conversation_id: &str) -> UiResult<PathBuf> {
    if !is_safe_cli_session_id(conversation_id) {
        return Err(tr("Antigravity が安全でない会話 ID を返しました。").into());
    }
    Ok(antigravity_store_directory()?
        .join("conversations")
        .join(format!("{conversation_id}.db")))
}

/// `agy` writes a plain JSONL transcript of every conversation next to its
/// working files. That transcript, not the conversation store, is what a
/// restore reads: it is a documented text format and needs no protobuf decoding
/// of another product's private schema.
pub(crate) fn antigravity_conversation_transcript_path(conversation_id: &str) -> UiResult<PathBuf> {
    if !is_safe_cli_session_id(conversation_id) {
        return Err(tr("Antigravity が安全でない会話 ID を返しました。").into());
    }
    Ok(antigravity_conversation_transcript_in(
        &antigravity_store_directory()?,
        conversation_id,
    ))
}

pub(crate) fn antigravity_conversation_transcript_in(
    store: &Path,
    conversation_id: &str,
) -> PathBuf {
    store
        .join("brain")
        .join(conversation_id)
        .join(".system_generated")
        .join("logs")
        .join(ANTIGRAVITY_TRANSCRIPT_FILE_NAME)
}

pub(crate) fn claude_conversation_path(workspace: &Path, session_id: &str) -> UiResult<PathBuf> {
    if !is_safe_cli_session_id(session_id) {
        return Err(tr("復元した Claude Code の会話 ID が安全ではありません。").into());
    }
    let home = std::env::var_os("HOME").map(PathBuf::from).ok_or_else(|| {
        tr("ホームディレクトリを取得できないため、Claude Code の会話ストアを特定できません。")
            .to_owned()
    })?;
    Ok(home
        .join(CLAUDE_STORE_DIRECTORY)
        .join(CLAUDE_PROJECTS_DIRECTORY)
        .join(encode_claude_project_path(workspace))
        .join(format!("{session_id}.jsonl")))
}

/// Write the restored turns as the conversation file Claude Code owns. It is
/// built under a temporary name and renamed into place, so `claude --resume`
/// never sees a half-written conversation at a resumable path, and an existing
/// conversation is never overwritten.
pub(crate) fn write_claude_conversation(
    path: &Path,
    session_id: &str,
    workspace: &Path,
    turns: &[(&'static str, String)],
    created_at: u64,
) -> UiResult<()> {
    let directory = path
        .parent()
        .ok_or_else(|| tr("Claude Code の会話ストアに親ディレクトリがありません。").to_owned())?;
    fs::create_dir_all(directory).map_err(|error| {
        tf!(
            "Claude Code の会話ストア {p0} を作成できません: {error}",
            error = error,
            p0 = directory.display()
        )
    })?;
    if path.exists() {
        return Err(tf!(
            "Claude Code には既に {p0} に会話があります。",
            p0 = path.display()
        ));
    }
    let records = build_claude_conversation(session_id, workspace, turns, created_at)?;
    write_file_atomically(path, &records)
        .map(|_| ())
        .map_err(|error| {
            tf!(
                "復元した会話 {p0} を確定できません: {error}",
                error = error,
                p0 = path.display()
            )
        })
}

/// One JSONL record per turn, in the shape Claude Code writes and reads back: a
/// `parentUuid` chain through the conversation, the conversation's own ID, and
/// the working directory it belongs to. Nothing Claude Code did not observe is
/// invented — no model name, token usage, or request ID is attached to a turn
/// another CLI produced.
pub(crate) fn build_claude_conversation(
    session_id: &str,
    workspace: &Path,
    turns: &[(&'static str, String)],
    created_at: u64,
) -> UiResult<Vec<u8>> {
    let mut records = Vec::new();
    let mut parent: Option<String> = None;
    for (index, (role, text)) in turns.iter().enumerate() {
        let uuid = Uuid::new_v4().to_string();
        let message = if *role == "user" {
            serde_json::json!({ "role": "user", "content": text })
        } else {
            serde_json::json!({
                "role": "assistant",
                "content": [{ "type": "text", "text": text }],
            })
        };
        // Keep the recorded order monotonic without pretending to know how long
        // each original turn took, and without dating any record in the future:
        // the conversation ends when it was restored.
        let recorded_at = created_at.saturating_sub((turns.len() - 1 - index) as u64);
        serde_json::to_writer(
            &mut records,
            &serde_json::json!({
                "parentUuid": parent,
                "isSidechain": false,
                "type": role,
                "message": message,
                "uuid": uuid,
                "timestamp": utc_timestamp_millis(recorded_at),
                "userType": "external",
                "cwd": workspace,
                "sessionId": session_id,
            }),
        )
        .map_err(|error| {
            tf!(
                "復元した Claude Code の会話のエンコード: {error}",
                error = error
            )
        })?;
        records.push(b'\n');
        parent = Some(uuid);
    }
    Ok(records)
}

/// Read the committed conversation back the way Claude Code will: one JSON
/// record per line, chained by `parentUuid`, every record belonging to this
/// conversation. The stored text is compared as written rather than through the
/// source-side reader, so a turn that happens to quote another CLI's plumbing
/// tags is still restored verbatim instead of failing the restore.
pub(crate) fn verify_claude_conversation(
    path: &Path,
    session_id: &str,
    turns: &[(&'static str, String)],
) -> UiResult<()> {
    let file = fs::File::open(path).map_err(|error| {
        tf!(
            "復元した会話 {p0} を開き直せません: {error}",
            error = error,
            p0 = path.display()
        )
    })?;
    let mut restored = 0_usize;
    let mut parent: Option<String> = None;
    for line in BufReader::new(file).lines() {
        let line = line.map_err(|error| {
            tf!(
                "復元した会話 {p0} を読み取れません: {error}",
                error = error,
                p0 = path.display()
            )
        })?;
        if line.trim().is_empty() {
            continue;
        }
        let record = serde_json::from_str::<serde_json::Value>(&line).map_err(|error| {
            tf!(
                "Claude Code のレコード {restored} を読み戻せませんでした: {error}",
                error = error,
                restored = restored
            )
        })?;
        let Some(expected) = turns.get(restored) else {
            return Err(
                tr("復元した Claude Code の会話が、元よりも多くのターンを含んでいます。").into(),
            );
        };
        let field = |name: &str| {
            record
                .get(name)
                .and_then(|value| value.as_str())
                .map(str::to_owned)
        };
        if field("sessionId").as_deref() != Some(session_id) {
            return Err(tf!(
                "Claude Code のレコード {restored} が、別の会話に書き込まれました。",
                restored = restored
            ));
        }
        if field("parentUuid") != parent {
            return Err(tf!(
                "復元した Claude Code の会話が、レコード {restored} で順序どおりではありません。",
                restored = restored
            ));
        }
        let role = field("type");
        let text = record
            .get("message")
            .and_then(|message| message.get("content"))
            .and_then(transcript_message_content_text);
        if role.as_deref() != Some(expected.0) || text.as_deref() != Some(expected.1.as_str()) {
            return Err(tf!(
                "復元した Claude Code の会話は、レコード {restored} の内容が元と異なります。",
                restored = restored
            ));
        }
        parent = field("uuid");
        if parent.is_none() {
            return Err(tf!(
                "Claude Code のレコード {restored} が、連鎖元の識別子なしで書き込まれました。",
                restored = restored
            ));
        }
        restored += 1;
    }
    if restored != turns.len() {
        return Err(tf!(
            "Claude Code の会話に、{p0} 件中 {restored} 件のターンを復元しました。",
            p0 = turns.len(),
            restored = restored
        ));
    }
    Ok(())
}

/// `thread/start` returns the eventual transcript path before Codex has
/// necessarily flushed its JSONL file. History injection itself is accepted by
/// the app server in that interval, so wait only after injection before
/// treating the path as resumable.
pub(crate) fn wait_for_codex_session_file(path: &Path) -> UiResult<PathBuf> {
    for delay_ms in NATIVE_RESOLUTION_DELAYS_MS {
        if delay_ms > 0 {
            thread::sleep(Duration::from_millis(delay_ms));
        }
        if path.is_file() {
            return Ok(path.to_path_buf());
        }
    }
    Err(
        tr("Codex app-server が、履歴復元後に永続セッションファイルを書き出しませんでした。")
            .into(),
    )
}

/// Read back one item Operon injected, exactly as it was written by
/// `codex_import_item`.
///
/// Deliberately not the source reader: this is Operon's own record coming back
/// out of Codex, so what it must prove is that the bytes survived unchanged —
/// not what a Codex conversation item means. Running the source reader over it
/// would let a change in how sources are interpreted quietly redefine what
/// "verified" means.
fn codex_persisted_import_item(value: &serde_json::Value) -> Option<(&'static str, String)> {
    let payload = value.get("payload")?;
    let role = match payload.get("role")?.as_str()? {
        "user" => "user",
        "assistant" => "assistant",
        _ => return None,
    };
    let text = payload.pointer("/content/0/text")?.as_str()?;
    Some((role, text.to_owned()))
}

/// `thread/inject_items` acknowledging a request is not sufficient evidence
/// that its contents survived to Codex's durable JSONL. Codex preserves the
/// caller-supplied `operon-import-N` IDs, so compare every projected turn with
/// the persisted destination before offering its resume ID.
pub(crate) fn verify_codex_imported_transcript(
    destination: &Path,
    turns: &[(&'static str, String)],
) -> UiResult<()> {
    let destination_file = fs::File::open(destination).map_err(|error| {
        tf!(
            "Codex の復元先履歴 {p0} を開けません: {error}",
            error = error,
            p0 = destination.display()
        )
    })?;
    let mut actual = HashMap::<usize, (String, String)>::new();
    for line in BufReader::new(destination_file).lines() {
        let line = line.map_err(|error| {
            tf!(
                "Codex の復元先履歴 {p0} を読み取れません: {error}",
                error = error,
                p0 = destination.display()
            )
        })?;
        let Ok(value) = serde_json::from_str::<serde_json::Value>(&line) else {
            continue;
        };
        let Some(index) = value
            .pointer("/payload/id")
            .and_then(|id| id.as_str())
            .and_then(|id| id.strip_prefix("operon-import-"))
            .and_then(|index| index.parse::<usize>().ok())
        else {
            continue;
        };
        let Some((role, text)) = codex_persisted_import_item(&value) else {
            return Err(tf!(
                "Codex が、取り込みターン operon-import-{index} を不正な形式で保存しました。",
                index = index
            ));
        };
        if actual.insert(index, (role.to_owned(), text)).is_some() {
            return Err(tf!(
                "Codex が、取り込みターン operon-import-{index} を重複して保存しました。",
                index = index
            ));
        }
    }

    for (imported_index, (expected_role, expected_text)) in turns.iter().enumerate() {
        let Some((actual_role, actual_text)) = actual.remove(&imported_index) else {
            return Err(tf!(
                "Codex が、取り込みターン operon-import-{imported_index} を永続化しませんでした。",
                imported_index = imported_index
            ));
        };
        if &actual_role != expected_role || &actual_text != expected_text {
            return Err(tf!("Codex が、取り込みターン operon-import-{imported_index} に異なる内容を保存しました。", imported_index = imported_index));
        }
    }
    if !actual.is_empty() {
        return Err(tf!(
            "Codex が、想定外の取り込みターンを {p0} 件保存しました。",
            p0 = actual.len()
        ));
    }
    Ok(())
}

/// `thread/inject_items` restores the thread's model-visible history only.
/// Codex's own terminal replays a resumed conversation from the rollout's
/// `event_msg` records, so a thread that holds nothing else reopens looking
/// empty even though the model can see every restored turn. Append the matching
/// replay records so `codex resume` shows the conversation it restored.
///
/// This runs only after the app server has exited, which makes Operon the
/// single writer of a rollout file no CLI has opened yet. The rewrite is atomic
/// so a failure can never leave Codex with a half-written record.
///
/// The appended records continue the file's numbering from its highest
/// ordinal, through `codex_rollout_metadata`. A rollout Codex numbered and
/// Operon did not is a thread Codex refuses to resume at all, and a turn record
/// with no ordinal is one Codex cannot project: `rollout_ordinal` is `NOT NULL`
/// in both `thread_turns` and `thread_items`. So a file that numbers nothing
/// has the appended records numbered from zero.
pub(crate) fn add_codex_replayable_history(
    rollout: &Path,
    turns: &[(&'static str, String)],
) -> UiResult<usize> {
    let existing = fs::File::open(rollout).map_err(|error| {
        tf!(
            "Codex の復元先履歴 {p0} を開けません: {error}",
            error = error,
            p0 = rollout.display()
        )
    })?;
    let complete = match existing.metadata() {
        Ok(metadata) if metadata.len() == 0 => false,
        Ok(_) => codex_rollout_ends_with_newline(rollout)?,
        Err(error) => {
            return Err(tf!(
                "Codex の復元先履歴 {p0} を読み取れません: {error}",
                error = error,
                p0 = rollout.display()
            ))
        }
    };
    if !complete {
        return Err(tr(
            "Codex の復元先履歴が不完全なレコードで終わっているため、追記を中止します。",
        )
        .into());
    }
    let (mut next_ordinal, session_id) = codex_rollout_metadata(rollout)?;
    let parent = rollout
        .parent()
        .ok_or_else(|| tr("Codex の復元先履歴に親ディレクトリがありません。").to_owned())?;
    let temporary = parent.join(format!(".operon-rollout.tmp-{}", Uuid::new_v4()));
    let written = (|| -> UiResult<usize> {
        let mut writer = BufWriter::new(
            OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&temporary)
                .map_err(|error| format!("creating {}: {error}", temporary.display()))?,
        );
        let mut reader = BufReader::new(existing);
        std::io::copy(&mut reader, &mut writer)
            .map_err(|error| format!("copying {}: {error}", rollout.display()))?;
        let base_sec = now();
        let mut records = 0;
        for (turn_index, (role, text)) in turns.iter().enumerate() {
            let turn_sec = base_sec.saturating_add(turn_index as u64);
            let events =
                codex_replay_turn_events(&session_id, role, text, &mut next_ordinal, turn_sec)
                    .map_err(|error| {
                        tf!("Codex の再生レコードのエンコード: {error}", error = error)
                    })?;
            for event in events {
                writer
                    .write_all(event.as_bytes())
                    .and_then(|_| writer.write_all(b"\n"))
                    .map_err(|error| format!("writing {}: {error}", temporary.display()))?;
            }
            records += 1;
        }
        let file = writer
            .into_inner()
            .map_err(|error| format!("flushing {}: {error}", temporary.display()))?;
        file.sync_all()
            .map_err(|error| format!("synchronizing {}: {error}", temporary.display()))?;
        drop(file);
        fs::rename(&temporary, rollout)
            .map_err(|error| format!("committing {}: {error}", rollout.display()))?;
        let _ = fs::File::open(parent).and_then(|directory| directory.sync_all());
        Ok(records)
    })();
    if written.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    written
}

pub(crate) fn codex_rollout_metadata(rollout: &Path) -> UiResult<(Option<u64>, String)> {
    let file = fs::File::open(rollout).map_err(|error| {
        tf!(
            "Codex の復元先履歴 {p0} を開けません: {error}",
            error = error,
            p0 = rollout.display()
        )
    })?;
    let mut next_ordinal: Option<u64> = None;
    let mut session_id: Option<String> = None;
    for line in BufReader::new(file).lines() {
        let line = line.map_err(|error| {
            tf!(
                "Codex の復元先履歴 {p0} を読み取れません: {error}",
                error = error,
                p0 = rollout.display()
            )
        })?;
        if let Ok(value) = serde_json::from_str::<serde_json::Value>(&line) {
            if let Some(ord) = value.get("ordinal").and_then(|v| v.as_u64()) {
                next_ordinal = Some(
                    next_ordinal
                        .map_or(ord.saturating_add(1), |cur| cur.max(ord.saturating_add(1))),
                );
            } else if next_ordinal.is_none()
                && value.get("history_mode").and_then(|v| v.as_str()) == Some("paginated")
            {
                next_ordinal = Some(0);
            }
            if session_id.is_none()
                && value.get("type").and_then(|v| v.as_str()) == Some("session_meta")
            {
                if let Some(payload) = value.get("payload") {
                    if let Some(id) = payload
                        .get("session_id")
                        .or_else(|| payload.get("id"))
                        .and_then(|v| v.as_str())
                    {
                        session_id = Some(id.to_owned());
                    }
                }
            }
        }
    }
    // In Codex CLI v0.154.0+, the SQLite projection engine requires `rollout_ordinal`
    // (declared NOT NULL on thread_turns and thread_items). If no ordinals were present
    // in the existing file, initialize next_ordinal to Some(0) so all appended turns
    // are indexed and rendered by `codex resume`.
    let next_ordinal = Some(next_ordinal.unwrap_or(0));
    let session_id = session_id
        .or_else(|| {
            rollout
                .file_stem()
                .and_then(|stem| stem.to_str())
                .and_then(crate::cli::codex_session_id)
        })
        .unwrap_or_else(|| Uuid::new_v4().to_string());
    Ok((next_ordinal, session_id))
}

pub(crate) fn codex_rollout_ends_with_newline(rollout: &Path) -> UiResult<bool> {
    let mut file = fs::File::open(rollout).map_err(|error| {
        tf!(
            "Codex の復元先履歴 {p0} を開けません: {error}",
            error = error,
            p0 = rollout.display()
        )
    })?;
    file.seek(SeekFrom::End(-1)).map_err(|error| {
        tf!(
            "Codex の復元先履歴 {p0} の末尾を読み取れません: {error}",
            error = error,
            p0 = rollout.display()
        )
    })?;
    let mut last = [0_u8; 1];
    file.read_exact(&mut last).map_err(|error| {
        tf!(
            "Codex の復元先履歴 {p0} の末尾を読み取れません: {error}",
            error = error,
            p0 = rollout.display()
        )
    })?;
    Ok(last[0] == b'\n')
}

pub(crate) fn codex_replay_turn_events(
    session_id: &str,
    role: &str,
    text: &str,
    next_ordinal: &mut Option<u64>,
    now_sec: u64,
) -> Result<Vec<String>> {
    let now_ms = now_sec.saturating_mul(1000);
    let iso_ts = utc_timestamp_millis(now_sec);
    let turn_id = Uuid::new_v4().to_string();

    let make_event =
        |payload: serde_json::Value, next_ordinal: &mut Option<u64>| -> Result<String> {
            let mut event = serde_json::json!({
                "timestamp": iso_ts,
                "type": "event_msg",
                "payload": payload,
            });
            if let Some(ordinal) = next_ordinal.as_mut() {
                event
                    .as_object_mut()
                    .expect("event_msg is a JSON object")
                    .insert("ordinal".to_owned(), serde_json::json!(*ordinal));
                *ordinal = ordinal.saturating_add(1);
            }
            Ok(serde_json::to_string(&event)?)
        };

    let mut events = Vec::with_capacity(3);

    events.push(make_event(
        serde_json::json!({
            "type": "task_started",
            "turn_id": turn_id,
            "started_at": now_sec,
            "model_context_window": 258400,
            "collaboration_mode_kind": "default",
        }),
        next_ordinal,
    )?);

    let item = if role == "user" {
        serde_json::json!({
            "type": "UserMessage",
            "id": Uuid::new_v4().to_string(),
            "content": [{
                "type": "text",
                "text": text,
            }],
            "text_elements": [],
        })
    } else {
        serde_json::json!({
            "type": "AgentMessage",
            "id": format!("msg_{}", Uuid::new_v4().simple()),
            "content": [{
                "type": "Text",
                "text": text,
            }],
            "phase": "final_answer",
        })
    };

    events.push(make_event(
        serde_json::json!({
            "type": "item_completed",
            "thread_id": session_id,
            "turn_id": turn_id,
            "item": item,
            "started_at_ms": now_ms,
            "completed_at_ms": now_ms,
        }),
        next_ordinal,
    )?);

    let last_agent_message = if role == "assistant" { text } else { "" };
    events.push(make_event(
        serde_json::json!({
            "type": "task_complete",
            "turn_id": turn_id,
            "last_agent_message": last_agent_message,
            "started_at": now_sec,
            "completed_at": now_sec,
            "duration_ms": 100,
            "time_to_first_token_ms": 50,
        }),
        next_ordinal,
    )?);

    Ok(events)
}

#[cfg(test)]
pub(crate) fn codex_replay_event(role: &str, text: &str, ordinal: Option<u64>) -> Result<String> {
    let mut ord = ordinal;
    let events = codex_replay_turn_events("test-thread", role, text, &mut ord, now())?;
    Ok(events.into_iter().nth(1).unwrap_or_default())
}

/// A restore is only complete when the destination can replay it. Compare the
/// replay records Codex now holds against the source turns before the resume ID
/// is offered to the user.
pub(crate) fn verify_codex_replayable_history(
    rollout: &Path,
    expected: &[(&'static str, String)],
) -> UiResult<()> {
    let rollout_file = fs::File::open(rollout).map_err(|error| {
        tf!(
            "Codex の復元先履歴 {p0} を開けません: {error}",
            error = error,
            p0 = rollout.display()
        )
    })?;
    let mut replayed = Vec::new();
    let mut saw_ordinal = false;
    let mut last_ordinal: Option<u64> = None;
    for line in BufReader::new(rollout_file).lines() {
        let line = line.map_err(|error| {
            tf!(
                "Codex の復元先履歴 {p0} を読み取れません: {error}",
                error = error,
                p0 = rollout.display()
            )
        })?;
        // A line Operon cannot read is a line Codex cannot read either, so it
        // counts as a record with no number rather than being passed over —
        // passed over, a trailing one left the record before it standing as
        // the file's last and a numbered rollout ending in it verified.
        let value = serde_json::from_str::<serde_json::Value>(&line).ok();
        let ord = value
            .as_ref()
            .and_then(|value| value.get("ordinal"))
            .and_then(|value| value.as_u64());
        if ord.is_some() {
            saw_ordinal = true;
        } else if saw_ordinal {
            return Err(tf!(
                "Codex の復元先履歴 {p0} に、ordinal のないレコードが含まれています。",
                p0 = rollout.display()
            ));
        }
        if let Some(ord) = ord {
            if let Some(last) = last_ordinal {
                if ord != last.saturating_add(1) {
                    return Err(tf!(
                        "Codex の復元先履歴 {p0} の ordinal が連続していません（{last} の次が {ord}）。",
                        p0 = rollout.display(),
                        last = last,
                        ord = ord
                    ));
                }
            }
            last_ordinal = Some(ord);
        }
        let Some(value) = value else {
            continue;
        };
        if value.get("type").and_then(|value| value.as_str()) != Some("event_msg") {
            continue;
        }
        let Some(payload) = value.get("payload") else {
            continue;
        };
        let payload_type = payload.get("type").and_then(|value| value.as_str());
        let turn = match payload_type {
            Some("user_message") => payload
                .get("message")
                .and_then(|v| v.as_str())
                .map(|m| ("user", m.to_owned())),
            Some("agent_message") => payload
                .get("message")
                .and_then(|v| v.as_str())
                .map(|m| ("assistant", m.to_owned())),
            Some("item_completed") => {
                let item = payload.get("item");
                let item_type = item.and_then(|i| i.get("type")).and_then(|v| v.as_str());
                let text = item
                    .and_then(|i| i.get("content"))
                    .and_then(|c| c.as_array())
                    .and_then(|arr| {
                        arr.iter()
                            .find_map(|elem| elem.get("text").and_then(|t| t.as_str()))
                    });
                match (item_type, text) {
                    (Some("UserMessage"), Some(text)) => Some(("user", text.to_owned())),
                    (Some("AgentMessage"), Some(text)) => Some(("assistant", text.to_owned())),
                    _ => None,
                }
            }
            _ => None,
        };
        if let Some(turn) = turn {
            replayed.push(turn);
        }
    }
    if replayed.len() != expected.len() {
        return Err(tf!(
            "Codex が再生できる復元ターンは {p0} 件のみで、期待される {p1} 件と一致しません。",
            p0 = replayed.len(),
            p1 = expected.len()
        ));
    }
    for (index, ((expected_role, expected_text), (role, text))) in
        expected.iter().zip(replayed.iter()).enumerate()
    {
        if expected_role != role || expected_text != text {
            return Err(tf!(
                "Codex は、復元ターン {p0} を元と異なる形で再生します。",
                p0 = index + 1
            ));
        }
    }
    Ok(())
}

/// Codex timestamps its rollout records in RFC 3339 UTC with milliseconds.
/// Restored replay records have to use the same shape or Codex cannot read the
/// record it is being asked to replay.
pub(crate) fn utc_timestamp_millis(seconds: u64) -> String {
    let days = (seconds / 86_400) as i64;
    let time_of_day = seconds % 86_400;
    let (year, month, day) = civil_date_from_unix_days(days);
    format!(
        "{year:04}-{month:02}-{day:02}T{:02}:{:02}:{:02}.000Z",
        time_of_day / 3_600,
        (time_of_day % 3_600) / 60,
        time_of_day % 60,
    )
}

/// Howard Hinnant's `civil_from_days`, which converts a day count since the
/// Unix epoch into a proleptic Gregorian date without a date dependency.
pub(crate) fn civil_date_from_unix_days(days: i64) -> (i64, u32, u32) {
    let shifted = days + 719_468;
    let era = shifted.div_euclid(146_097);
    let day_of_era = shifted.rem_euclid(146_097);
    let year_of_era =
        (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let year = year_of_era + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let shifted_month = (5 * day_of_year + 2) / 153;
    let day = (day_of_year - (153 * shifted_month + 2) / 5 + 1) as u32;
    let month = if shifted_month < 10 {
        shifted_month + 3
    } else {
        shifted_month - 9
    } as u32;
    (year + i64::from(month <= 2), month, day)
}

/// Read an RFC 3339 UTC instant of the shape agent transcripts record
/// (`2026-08-16T14:14:31.408Z`). Anything else, including a local-offset
/// timestamp, is rejected rather than guessed at: a wrong instant here would
/// silently select the wrong conversation.
pub(crate) fn parse_rfc3339_utc_seconds(timestamp: &str) -> Option<u64> {
    let (date, time) = timestamp.split_once('T')?;
    let time = time.strip_suffix('Z')?;
    let time = time.split_once('.').map_or(time, |(seconds, _)| seconds);
    let mut date_parts = date.split('-');
    let year = date_parts.next()?.parse::<i64>().ok()?;
    let month = date_parts.next()?.parse::<u32>().ok()?;
    let day = date_parts.next()?.parse::<u32>().ok()?;
    if date_parts.next().is_some() || !(1..=12).contains(&month) || !(1..=31).contains(&day) {
        return None;
    }
    let mut time_parts = time.split(':');
    let hour = time_parts.next()?.parse::<u64>().ok()?;
    let minute = time_parts.next()?.parse::<u64>().ok()?;
    let second = time_parts.next()?.parse::<u64>().ok()?;
    if time_parts.next().is_some() || hour > 23 || minute > 59 || second > 60 {
        return None;
    }
    let days = unix_days_from_civil(year, month, day);
    u64::try_from(days.checked_mul(86_400)?).ok()?.checked_add(
        hour.checked_mul(3_600)?
            .checked_add(minute.checked_mul(60)?)?
            .checked_add(second)?,
    )
}

/// Howard Hinnant's `days_from_civil`, the inverse of `civil_date_from_unix_days`.
pub(crate) fn unix_days_from_civil(year: i64, month: u32, day: u32) -> i64 {
    let year = year - i64::from(month <= 2);
    let era = year.div_euclid(400);
    let year_of_era = year - era * 400;
    let day_of_year =
        (153 * (i64::from(month) + if month > 2 { -3 } else { 9 }) + 2) / 5 + i64::from(day) - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    era * 146_097 + day_of_era - 719_468
}

pub(crate) struct CodexAppServer {
    pub(crate) child: Child,
    pub(crate) stdin: ChildStdin,
    pub(crate) stdout: BufReader<ChildStdout>,
    pub(crate) next_request_id: u64,
    pub(crate) thread_id: Option<String>,
}

/// What one transcript record contributed to the restore. Four outcomes, not
/// two: "excluded because the vocabulary says so" and "never seen before" used
/// to be the same `None`, which is how a record type shipped after this code
/// was written disappeared without anybody noticing.
pub(crate) enum RecordOutcome {
    Turn(&'static str, String),
    /// `TRANSCRIPT_VOCABULARY` classifies this kind as not conversation.
    Operational,
    /// Conversation, but it held nothing readable to carry over.
    Empty,
    /// No row in `TRANSCRIPT_VOCABULARY`, and no readable text to carry either.
    Unclassified,
}

/// Read one transcript record. Provider transcripts also contain operational
/// metadata (hooks, snapshots, compaction payloads, etc.); exposing those as
/// chat messages makes the restored conversation unreadable and buries the real
/// one. Which of them is operational is not decided here — it is read from
/// `TRANSCRIPT_VOCABULARY`, so every restore destination shares both the reader
/// and the judgement.
pub(crate) fn transcript_record(
    raw_record: &str,
    provider: CliProvider,
    summary: &mut RestoreClassification,
) -> RecordOutcome {
    let Ok(parsed) = serde_json::from_str::<serde_json::Value>(raw_record) else {
        // Not JSON, so not a record any reader can account for. Reported rather
        // than skipped: a transcript that grew a second format is exactly the
        // change this accounting exists to surface.
        summary.count_unclassified_kind(provider, "unparsable-line", UnclassifiedOutcome::Dropped);
        return RecordOutcome::Unclassified;
    };
    transcript_value_record(&parsed, provider, summary)
}

pub(crate) fn transcript_value_record(
    value: &serde_json::Value,
    provider: CliProvider,
    summary: &mut RestoreClassification,
) -> RecordOutcome {
    let outcome = match provider {
        CliProvider::Claude => claude_transcript_record(value, summary),
        CliProvider::Codex => codex_transcript_record(value, summary),
        CliProvider::Gemini => antigravity_transcript_record(value, summary),
    };
    // The one place that knows what became of the record, and therefore the
    // only place that can settle a kind whose arm the record's outcome decides.
    // Every reader returns through here.
    summary.settle_deferred_kinds(&outcome);
    outcome
}

/// One record's turn, for the callers that read a transcript for a single
/// answer rather than to restore it.
pub(crate) fn transcript_conversation_turn(
    raw_record: &str,
    provider: CliProvider,
) -> Option<(&'static str, String)> {
    let mut summary = RestoreClassification::default();
    match transcript_record(raw_record, provider, &mut summary) {
        RecordOutcome::Turn(role, text) => Some((role, text)),
        _ => None,
    }
}

/// As `transcript_conversation_turn`, for a record that is already parsed.
pub(crate) fn transcript_value_turn(
    value: &serde_json::Value,
    provider: CliProvider,
) -> Option<(&'static str, String)> {
    let mut summary = RestoreClassification::default();
    match transcript_value_record(value, provider, &mut summary) {
        RecordOutcome::Turn(role, text) => Some((role, text)),
        _ => None,
    }
}

/// Whatever text a record holds, without knowing its shape.
///
/// Used for two things: a record whose kind the vocabulary does not know, and a
/// record the vocabulary calls conversation but which keeps its text somewhere
/// other than a message — Claude Code's `summary` records are the case that
/// forced it. An optional field a reader does not understand is safe to ignore
/// for parsing, but stripping it is how history silently shrinks, so anything
/// readable crosses.
fn record_text(value: &serde_json::Value) -> Option<String> {
    for field in ["message", "content", "text", "summary", "tools", "output"] {
        let Some(found) = value.get(field) else {
            continue;
        };
        let found = if field == "message" {
            match found.get("content") {
                Some(content) => content,
                None => continue,
            }
        } else {
            found
        };
        if let Some(text) = transcript_message_content_text(found) {
            return Some(text);
        }
        if let Some(text) = found.as_str().and_then(nonempty_transcript_text) {
            return Some(text);
        }
    }
    None
}

/// Carry an unclassified fragment across under a label naming its kind.
fn unclassified_turn(provider: CliProvider, kind: &str, text: &str) -> RecordOutcome {
    RecordOutcome::Turn(
        "assistant",
        format!("{}\n{text}", unclassified_label(provider, kind)),
    )
}

/// Account for a record whose kind has no row in the vocabulary, and carry it if
/// there is anything to carry.
///
/// Every reader used to spell this out: count the kind, then ask `record_text`,
/// then take one of two arms. Counting before asking is what let the count and
/// the outcome disagree — the notice said "carried" for records that took the
/// other arm — so the two decisions are made in one place now, and the outcome
/// is what is counted. Six call sites across the three readers had the same
/// shape; a seventh written later gets this one for free.
fn unclassified_record(
    provider: CliProvider,
    kind: &str,
    value: &serde_json::Value,
    summary: &mut RestoreClassification,
) -> RecordOutcome {
    match record_text(value) {
        Some(text) => {
            summary.count_unclassified_kind(provider, kind, UnclassifiedOutcome::Carried);
            unclassified_turn(provider, kind, &text)
        }
        None => {
            summary.count_unclassified_kind(provider, kind, UnclassifiedOutcome::Dropped);
            RecordOutcome::Unclassified
        }
    }
}

/// Every `is…` flag a record sets on itself, checked against the vocabulary.
///
/// The keys are read off the record rather than from a fixed list, so a flag a
/// CLI adds later is reported the first time it is met instead of being
/// invisible until somebody thinks to look for it. An unknown flag never
/// excludes a record: reporting is the point, and guessing that a new flag
/// means "not conversation" would lose turns on the strength of a guess.
fn record_is_excluded_by_flag(
    value: &serde_json::Value,
    provider: CliProvider,
    summary: &mut RestoreClassification,
) -> bool {
    let Some(fields) = value.as_object() else {
        return false;
    };
    let mut excluded = false;
    for (key, set) in fields {
        if set.as_bool() != Some(true) || !key.starts_with("is") {
            continue;
        }
        match transcript_class(provider, TranscriptLayer::Flag, key) {
            Some(TranscriptClass::Operational) => excluded = true,
            Some(_) => {}
            // An unknown flag never excludes, so the record it sits on goes on
            // to be read. Whether it then crosses is not decided here and not
            // decided by the flag: the record can still hold nothing readable.
            // Deferred rather than called carried, which it was until round 2
            // of this change's review found a `message.content` of `[]` under
            // an unknown flag being reported as delivered.
            None => summary.defer_unclassified_kind(provider, key),
        }
    }
    excluded
}

/// Read one record from Antigravity's per-conversation transcript. Its records
/// are typed by origin rather than by role, so a record is keyed `SOURCE/TYPE`
/// and `TRANSCRIPT_VOCABULARY` decides which of those pairs are conversation.
///
/// `MODEL/GENERIC` is one of them. It carries tool results — generated images,
/// fetched pages, written files — and outnumbers the planner's own responses in
/// real conversations, so treating it as trace lost most of the evidence the
/// model had been reasoning about.
fn antigravity_transcript_record(
    value: &serde_json::Value,
    summary: &mut RestoreClassification,
) -> RecordOutcome {
    let provider = CliProvider::Gemini;
    let (Some(source), Some(record_type)) = (
        value.get("source").and_then(|value| value.as_str()),
        value.get("type").and_then(|value| value.as_str()),
    ) else {
        return unclassified_record(provider, "record-without-source-or-type", value, summary);
    };
    let kind = format!("{source}/{record_type}");
    match transcript_class(provider, TranscriptLayer::Record, &kind) {
        Some(TranscriptClass::Operational) => return RecordOutcome::Operational,
        Some(_) => {}
        None => return unclassified_record(provider, &kind, value, summary),
    }

    if source == "USER_EXPLICIT" {
        let request = value
            .get("content")
            .and_then(|value| value.as_str())
            .map(antigravity_user_request_text)
            .and_then(|request| nonempty_transcript_text(&request));
        return match request {
            Some(text) => RecordOutcome::Turn("user", text),
            None => RecordOutcome::Empty,
        };
    }

    let mut chunks = Vec::new();
    if let Some(text) = value.get("content").and_then(|value| value.as_str()) {
        if !text.trim().is_empty() {
            chunks.push(text.to_owned());
        }
    }
    // The model's reasoning rides on the record that produced it rather than in
    // a block of its own. It crosses as attributed text like every other
    // provider's reasoning does.
    if matches!(
        transcript_class(provider, TranscriptLayer::Block, "thinking"),
        Some(TranscriptClass::Reasoning)
    ) {
        if let Some(thinking) = value
            .get("thinking")
            .and_then(|value| value.as_str())
            .and_then(nonempty_transcript_text)
        {
            summary.reasoning_fragments += 1;
            chunks.push(format!("{RESTORED_REASONING_LABEL}\n{thinking}"));
        }
    }
    let no_tool_calls = Vec::new();
    for call in value
        .get("tool_calls")
        .and_then(|calls| calls.as_array())
        .unwrap_or(&no_tool_calls)
    {
        let name = call
            .get("name")
            .and_then(|value| value.as_str())
            .unwrap_or("unnamed tool");
        let arguments = call
            .get("args")
            .map(|value| serde_json::to_string(value).unwrap_or_default())
            .unwrap_or_default();
        chunks.push(format!("[Tool call: {name}]\n{arguments}"));
    }
    match nonempty_transcript_text(&chunks.join("\n\n")) {
        Some(text) => RecordOutcome::Turn("assistant", text),
        None => RecordOutcome::Empty,
    }
}

/// Antigravity wraps the submitted request in `<USER_REQUEST>` and appends its
/// own metadata blocks after it. Only the request itself is part of the
/// conversation; the appended blocks describe the local environment of a
/// session that is not being resumed.
///
/// Which wrapper holds the request is read from `TRANSCRIPT_VOCABULARY` rather
/// than written here, so the tag cannot be renamed in one place and left in the
/// other.
pub(crate) fn antigravity_user_request_text(content: &str) -> String {
    for tag in transcript_wrappers(CliProvider::Gemini, TranscriptClass::Conversation) {
        let open = format!("<{tag}>");
        let Some((_, body)) = content.split_once(&open) else {
            continue;
        };
        // An unterminated wrapper still opened the request: everything after it
        // is the request, because the appended metadata blocks come later.
        return body
            .split_once(&format!("</{tag}>"))
            .map(|(request, _)| request)
            .unwrap_or(body)
            .trim()
            .to_owned();
    }
    content.to_owned()
}

/// One projected turn as the item Codex's app server accepts. The
/// `operon-import-N` id is preserved by Codex, which is what lets the
/// verification pass match a persisted item back to the turn it came from.
pub(crate) fn codex_import_item(role: &str, text: &str, index: usize) -> serde_json::Value {
    let content_type = if role == "user" {
        "input_text"
    } else {
        "output_text"
    };
    serde_json::json!({
        "type": "message",
        "id": format!("operon-import-{index}"),
        "role": role,
        "content": [{
            "type": content_type,
            "text": text,
        }],
    })
}

/// Read one record from a Claude Code conversation file.
///
/// The record's `type` decides whether it is looked at, the flags it sets on
/// itself decide whether it is excluded, and the blocks inside its message
/// decide what crosses. All three questions are answered by
/// `TRANSCRIPT_VOCABULARY`, which is why a record type Claude Code adds after
/// this build now arrives as a reported unclassified fragment rather than as
/// silence.
fn claude_transcript_record(
    value: &serde_json::Value,
    summary: &mut RestoreClassification,
) -> RecordOutcome {
    let provider = CliProvider::Claude;
    let Some(record_type) = value.get("type").and_then(|value| value.as_str()) else {
        return unclassified_record(provider, "record-without-type", value, summary);
    };
    match transcript_class(provider, TranscriptLayer::Record, record_type) {
        Some(TranscriptClass::Operational) => return RecordOutcome::Operational,
        Some(_) => {}
        None => return unclassified_record(provider, record_type, value, summary),
    }
    let role = match record_type {
        "user" => "user",
        _ => "assistant",
    };
    if record_is_excluded_by_flag(value, provider, summary) {
        return RecordOutcome::Operational;
    }
    // Most conversation records keep their text in `message.content`. A
    // `summary` record does not — it carries the text standing in for the turns
    // a compaction or a resume replaced — so read the record itself rather than
    // calling it empty and losing everything before the compaction.
    let text = match value
        .get("message")
        .and_then(|message| message.get("content"))
    {
        Some(content) => classified_content_text(content, provider, summary),
        None => record_text(value),
    };
    let Some(text) = text else {
        return RecordOutcome::Empty;
    };
    if role == "assistant" {
        return RecordOutcome::Turn(role, text);
    }
    match claude_user_text_without_cli_plumbing(&text) {
        Some(text) => RecordOutcome::Turn(role, text),
        None => RecordOutcome::Empty,
    }
}

/// Claude Code wraps its own terminal plumbing into user messages: the caveat
/// it prepends to locally-run commands, the command's stdout, subagent task
/// notifications, and the expansion of a typed slash command. Restoring those
/// verbatim fills another CLI's conversation with a third CLI's internals, so
/// keep only what the person actually said — including the slash command they
/// typed, rendered the way they typed it.
///
/// Which wrappers are the person's and which are the harness's is read from
/// `TRANSCRIPT_VOCABULARY`. The list used to be written here, which meant a tag
/// could be classified in one place and stripped in the other.
pub(crate) fn claude_user_text_without_cli_plumbing(text: &str) -> Option<String> {
    let provider = CliProvider::Claude;
    let command = tagged_block_text(text, "command-name").map(|name| {
        match tagged_block_text(text, "command-args").filter(|args| !args.is_empty()) {
            Some(arguments) => format!("{name} {arguments}"),
            None => name,
        }
    });
    let mut remainder = text.to_owned();
    // Both classes are removed from the remainder: the person's own wrappers
    // have already been read out into `command`, so leaving them in would
    // restore the typed command twice.
    for tag in transcript_wrappers(provider, TranscriptClass::Operational)
        .chain(transcript_wrappers(provider, TranscriptClass::Conversation))
    {
        remainder = remove_tagged_blocks(&remainder, tag);
    }
    let remainder = remainder.trim();
    let restored = match (command, remainder.is_empty()) {
        (Some(command), true) => command,
        (Some(command), false) => format!("{command}\n{remainder}"),
        (None, _) => remainder.to_owned(),
    };
    nonempty_transcript_text(&restored)
}

/// Read the contents of the first `<tag>…</tag>` block. Both Claude Code and
/// Codex wrap what a person said inside their own tagged blocks, so the same
/// reader recovers it from either.
pub(crate) fn tagged_block_text(text: &str, tag: &str) -> Option<String> {
    let open = format!("<{tag}>");
    let close = format!("</{tag}>");
    let start = text.find(&open)? + open.len();
    let end = text[start..].find(&close)? + start;
    Some(text[start..end].trim().to_owned())
}

/// Remove every `<tag>…</tag>` block. An unterminated block is removed to the
/// end of the message: a truncated wrapper is still not conversation.
pub(crate) fn remove_tagged_blocks(text: &str, tag: &str) -> String {
    let open = format!("<{tag}>");
    let close = format!("</{tag}>");
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(start) = rest.find(&open) {
        out.push_str(&rest[..start]);
        rest = match rest[start..].find(&close) {
            Some(end) => &rest[start + end + close.len()..],
            None => "",
        };
    }
    out.push_str(rest);
    out
}

/// Read one record from a Codex rollout.
///
/// Codex nests twice: a record type, then a payload type inside it. Both are
/// classified, so `reasoning` — the most numerous payload in real sessions, by
/// a wide margin over `message` — now crosses as attributed text instead of
/// falling off the end of a match.
fn codex_transcript_record(
    value: &serde_json::Value,
    summary: &mut RestoreClassification,
) -> RecordOutcome {
    let provider = CliProvider::Codex;
    let Some(record_type) = value.get("type").and_then(|value| value.as_str()) else {
        summary.count_unclassified_kind(
            provider,
            "record-without-type",
            UnclassifiedOutcome::Dropped,
        );
        return RecordOutcome::Unclassified;
    };
    match transcript_class(provider, TranscriptLayer::Record, record_type) {
        Some(TranscriptClass::Operational) => return RecordOutcome::Operational,
        Some(_) => {}
        None => return unclassified_record(provider, record_type, value, summary),
    }
    let Some(payload) = value.get("payload") else {
        return RecordOutcome::Empty;
    };
    let Some(payload_type) = payload.get("type").and_then(|value| value.as_str()) else {
        summary.count_unclassified_kind(
            provider,
            "payload-without-type",
            UnclassifiedOutcome::Dropped,
        );
        return RecordOutcome::Unclassified;
    };
    let class = match transcript_class(provider, TranscriptLayer::Payload, payload_type) {
        Some(TranscriptClass::Operational) => return RecordOutcome::Operational,
        Some(class) => class,
        None => return unclassified_record(provider, payload_type, payload, summary),
    };
    if class == TranscriptClass::Reasoning {
        // Codex records reasoning as summary text plus, in stateless mode, an
        // `encrypted_content` blob the vocabulary marks operational: it is
        // sealed under OpenAI's organisation key and decodes nowhere else.
        return match codex_reasoning_text(payload) {
            Some(text) => {
                summary.reasoning_fragments += 1;
                RecordOutcome::Turn("assistant", format!("{RESTORED_REASONING_LABEL}\n{text}"))
            }
            None => RecordOutcome::Empty,
        };
    }
    let rendered = match payload_type {
        "message" => {
            // A conversation has two participants. Codex also addresses the
            // model as `developer` and `system`, which is the harness talking,
            // and the destination loads its own.
            let declared = payload
                .get("role")
                .and_then(|value| value.as_str())
                .unwrap_or("assistant");
            match transcript_class(provider, TranscriptLayer::Role, declared) {
                Some(TranscriptClass::Operational) => return RecordOutcome::Operational,
                Some(_) => {}
                // An unknown role does not stop the message, but it does not
                // deliver it either — `content` may be absent two lines below,
                // and then nothing crossed. Deferred to the record's outcome.
                None => summary.defer_unclassified_kind(provider, declared),
            }
            let role = if declared == "user" {
                "user"
            } else {
                "assistant"
            };
            let Some(content) = payload.get("content") else {
                return RecordOutcome::Empty;
            };
            let Some(text) = classified_content_text(content, provider, summary) else {
                return RecordOutcome::Empty;
            };
            if role == "assistant" {
                Some(("assistant", text))
            } else {
                codex_user_text_without_cli_plumbing(&text).map(|text| ("user", text))
            }
        }
        // Codex uses these records for messages between collaborating agents.
        // They are model-written conversation, not transcript housekeeping,
        // and omitting them makes a restored delegated task lose its context.
        "agent_message" => payload
            .get("content")
            .and_then(|content| classified_content_text(content, provider, summary))
            .map(|text| ("assistant", text)),
        // Tool calls and their outputs are response items in their own right,
        // rather than content blocks inside an assistant message. Preserve the
        // readable text in order so a restored CLI receives the same evidence
        // and decisions as the source conversation.
        "function_call"
        | "custom_tool_call"
        | "web_search_call"
        | "image_generation_call"
        | "tool_search_call" => codex_tool_call_text(payload).map(|text| ("assistant", text)),
        "function_call_output" | "custom_tool_call_output" | "tool_search_output" => {
            codex_tool_output_text(payload).map(|text| ("assistant", text))
        }
        // Classified as conversation but with no renderer of its own yet. It is
        // still carried: the vocabulary said it belongs in the conversation.
        _ => record_text(payload).map(|text| ("assistant", text)),
    };
    match rendered {
        Some((role, text)) => RecordOutcome::Turn(role, text),
        None => RecordOutcome::Empty,
    }
}

/// The readable part of a Codex reasoning item. Its summary is a list of
/// sections, each with its own text; the encrypted blob beside them is not
/// readable anywhere but inside OpenAI.
fn codex_reasoning_text(payload: &serde_json::Value) -> Option<String> {
    let mut chunks = Vec::new();
    for field in ["summary", "content"] {
        let Some(parts) = payload.get(field).and_then(|value| value.as_array()) else {
            continue;
        };
        for part in parts {
            let text = part
                .get("text")
                .and_then(|value| value.as_str())
                .or_else(|| part.as_str());
            if let Some(text) = text.filter(|text| !text.trim().is_empty()) {
                chunks.push(text.to_owned());
            }
        }
    }
    nonempty_transcript_text(&chunks.join("\n\n"))
}

/// Codex does not name every call the same way: `function_call` carries a
/// `name` and `arguments`, while `tool_search_call` carries neither and puts
/// its query in `arguments` alone. The payload type is the fallback name, so a
/// call the vocabulary classifies as conversation always crosses as one rather
/// than disappearing for want of a field.
pub(crate) fn codex_tool_call_text(payload: &serde_json::Value) -> Option<String> {
    let name = payload
        .get("name")
        .and_then(|value| value.as_str())
        .or_else(|| payload.get("type").and_then(|value| value.as_str()))
        .unwrap_or("unnamed tool");
    let input = payload
        .get("arguments")
        .or_else(|| payload.get("input"))
        .map(transcript_json_value_text)
        .unwrap_or_default();
    nonempty_transcript_text(&format!("[Tool call: {name}]\n{input}"))
}

pub(crate) fn codex_tool_output_text(payload: &serde_json::Value) -> Option<String> {
    let output = payload
        .get("output")
        .or_else(|| payload.get("tools"))
        .map(transcript_json_value_text)
        .unwrap_or_default();
    nonempty_transcript_text(&format!("[Tool result]\n{output}"))
}

/// Tool APIs use strings for some inputs and typed content blocks for others.
/// Prefer a readable content rendering, then retain structured values as JSON
/// instead of silently dropping a tool record whose schema has evolved.
pub(crate) fn transcript_json_value_text(value: &serde_json::Value) -> String {
    transcript_message_content_text(value).unwrap_or_else(|| match value {
        serde_json::Value::String(text) => text.to_owned(),
        value => serde_json::to_string(value).unwrap_or_default(),
    })
}

/// Codex sends itself a good deal of its own harness as `user` messages: the
/// session's setup, hook output, interruption notices, and the goal scaffolding
/// it wraps around a standing objective. None of it is what the person said, so
/// keep only the request itself — including the objective inside the goal
/// block, which is the one part of it a person wrote.
pub(crate) fn codex_user_text_without_cli_plumbing(text: &str) -> Option<String> {
    if is_codex_session_setup_message(text) {
        return None;
    }
    if text.trim_start().starts_with(CODEX_GOAL_CONTEXT_MARKER) {
        // The one part of the goal scaffolding a person wrote. Which wrapper
        // holds it is the vocabulary's answer, not a literal repeated here.
        return transcript_wrappers(CliProvider::Codex, TranscriptClass::Conversation)
            .find_map(|tag| tagged_block_text(text, tag))
            .and_then(|objective| nonempty_transcript_text(&objective));
    }
    nonempty_transcript_text(text)
}

/// Codex opens a conversation by sending itself the session's setup as `user`
/// messages, and keeps adding operational ones as it runs. They describe the
/// machine the session ran on and what its own harness did, not anything the
/// person said, so restoring them elsewhere would put another CLI's internals
/// into the conversation.
///
/// The tags come from `TRANSCRIPT_VOCABULARY`, so the set this recognizes and
/// the set the table declares operational cannot drift apart.
pub(crate) fn is_codex_session_setup_message(text: &str) -> bool {
    let text = text.trim_start();
    transcript_wrappers(CliProvider::Codex, TranscriptClass::Operational).any(|tag| {
        // A tag can carry attributes — `hook_prompt` always records a
        // `hook_run_id`. Both forms are matched so the tag name alone cannot
        // swallow a message that merely starts with a longer word.
        text.starts_with(&format!("<{tag}>")) || text.starts_with(&format!("<{tag} "))
    })
}

/// Render one message's content, consulting `TRANSCRIPT_VOCABULARY` for every
/// block inside it.
///
/// The plain renderer beside this one keeps only blocks that carry a `text`
/// field, which is how every `thinking` block Claude Code wrote — about as many
/// as its text blocks — left the conversation without a trace. Here a block's
/// kind decides: conversation crosses as itself, reasoning crosses labelled,
/// operational is dropped by declaration, and a kind nobody has classified
/// crosses under a label naming it.
pub(crate) fn classified_content_text(
    content: &serde_json::Value,
    provider: CliProvider,
    summary: &mut RestoreClassification,
) -> Option<String> {
    let serde_json::Value::Array(parts) = content else {
        // A bare string is the whole message; there are no blocks to classify.
        return transcript_message_content_text(content);
    };
    let mut chunks = Vec::new();
    for part in parts {
        let Some(block_type) = part.get("type").and_then(|value| value.as_str()) else {
            // No discriminator to classify by. Fall back to the plain renderer
            // rather than inventing a kind name for it.
            if let Some(text) = transcript_message_content_text(part) {
                chunks.push(text);
            }
            continue;
        };
        match transcript_class(provider, TranscriptLayer::Block, block_type) {
            Some(TranscriptClass::Operational) => {}
            Some(TranscriptClass::Reasoning) => {
                if let Some(text) = transcript_block_text(part) {
                    summary.reasoning_fragments += 1;
                    chunks.push(format!("{RESTORED_REASONING_LABEL}\n{text}"));
                }
            }
            Some(TranscriptClass::Conversation) => {
                if let Some(text) = transcript_block_text(part) {
                    chunks.push(text);
                }
            }
            None => {
                // Both arms push something, so an unknown block always crosses:
                // with its text when it has one, and otherwise as the bare
                // label, which is still the fact that the block existed. This is
                // the one unclassified site where "nothing readable" does not
                // mean "nothing carried".
                summary.count_unclassified_kind(provider, block_type, UnclassifiedOutcome::Carried);
                let label = unclassified_label(provider, block_type);
                match transcript_block_text(part) {
                    Some(text) => chunks.push(format!("{label}\n{text}")),
                    None => chunks.push(label),
                }
            }
        }
    }
    nonempty_transcript_text(&chunks.join("\n\n"))
}

/// The readable text of one content block, whatever shape it takes.
///
/// A block need not carry a `text` field: `thinking` carries `thinking`,
/// `tool_use` carries a name and an input, and an `image` carries bytes that
/// cannot cross at all. A client that assumed `text` was always there is the
/// documented way this kind of reader breaks, so every known carrier is tried
/// and a block with none still yields the fact that it existed.
fn transcript_block_text(part: &serde_json::Value) -> Option<String> {
    // The type decides first. A `tool_result` also carries a `content` field,
    // so reading fields before types would return the result's text stripped of
    // the `[Tool result]` marker that says what it is.
    match part.get("type").and_then(|value| value.as_str()) {
        Some("tool_use" | "function_call") => {
            let name = part
                .get("name")
                .and_then(|value| value.as_str())
                .unwrap_or("unnamed tool");
            let input = part
                .get("input")
                .or_else(|| part.get("arguments"))
                .map(|value| serde_json::to_string(value).unwrap_or_default())
                .unwrap_or_default();
            return Some(format!("[Tool call: {name}]\n{input}"));
        }
        Some("tool_result" | "function_call_output") => {
            return part
                .get("content")
                .or_else(|| part.get("output"))
                .and_then(transcript_message_content_text)
                .map(|result| format!("[Tool result]\n{result}"));
        }
        _ => {}
    }
    for field in ["text", "thinking", "content", "summary"] {
        let value = part.get(field);
        if let Some(text) = value.and_then(|value| value.as_str()) {
            if !text.trim().is_empty() {
                return Some(text.to_owned());
            }
        }
        if let Some(text) = value.and_then(transcript_message_content_text) {
            return Some(text);
        }
    }
    // An image, a reference, a citation: it cannot cross as itself, so what
    // crosses is that it was there. Losing that is worse than losing the bytes,
    // because the turns around it stop making sense.
    part.get("type")
        .and_then(|value| value.as_str())
        .map(|other| format!("[{other}]"))
}

/// Render text and tool context from one provider message without including
/// unrelated top-level transcript records. Tool calls/results remain visible
/// to the destination model, while ordinary messages stay human-readable.
pub(crate) fn transcript_message_content_text(content: &serde_json::Value) -> Option<String> {
    match content {
        serde_json::Value::String(text) => nonempty_transcript_text(text),
        serde_json::Value::Array(parts) => {
            let mut chunks = Vec::new();
            for part in parts {
                if let Some(text) = part.get("text").and_then(|value| value.as_str()) {
                    if !text.trim().is_empty() {
                        chunks.push(text.to_owned());
                    }
                    continue;
                }
                match part.get("type").and_then(|value| value.as_str()) {
                    Some("tool_use" | "function_call") => {
                        let name = part
                            .get("name")
                            .and_then(|value| value.as_str())
                            .unwrap_or("unnamed tool");
                        let input = part
                            .get("input")
                            .or_else(|| part.get("arguments"))
                            .map(|value| serde_json::to_string(value).unwrap_or_default())
                            .unwrap_or_default();
                        chunks.push(format!("[Tool call: {name}]\n{input}"));
                    }
                    Some("tool_result" | "function_call_output") => {
                        let result = part
                            .get("content")
                            .or_else(|| part.get("output"))
                            .and_then(transcript_message_content_text)
                            .unwrap_or_default();
                        if !result.trim().is_empty() {
                            chunks.push(format!("[Tool result]\n{result}"));
                        }
                    }
                    _ => {}
                }
            }
            nonempty_transcript_text(&chunks.join("\n\n"))
        }
        _ => None,
    }
}

pub(crate) fn nonempty_transcript_text(text: &str) -> Option<String> {
    (!text.trim().is_empty()).then(|| text.to_owned())
}

/// Codex allocates native conversation IDs internally. Its first prompt carries
/// a per-launch opaque tracking token, so concurrent launches cannot be
/// correlated by a duplicate task or an imprecise filesystem timestamp.
pub(crate) fn resolve_codex_session_after_launch(
    project: &Path,
    tracking_token: &str,
) -> UiResult<CliSession> {
    for delay_ms in NATIVE_RESOLUTION_DELAYS_MS {
        if delay_ms > 0 {
            thread::sleep(Duration::from_millis(delay_ms));
        }
        let scan = scan_codex_cli_sessions_for_resolution(project);
        let matches = scan
            .sessions
            .into_iter()
            .filter(|session| {
                matches_native_session_for_launch(session, CliProvider::Codex, tracking_token)
            })
            .collect::<Vec<_>>();
        match unique_native_session_match(matches)? {
            Some(session) => return Ok(session),
            None => thread::sleep(Duration::from_millis(500)),
        }
    }
    Err(tf!(
        "{p0} は約 15 秒以内に対応するローカルセッション記録を書き込みませんでした",
        p0 = CliProvider::Codex.label()
    ))
}

/// Codex writes no transcript at all until its first turn, and only that turn
/// can carry the launch token, so a terminal launched with a blank goal has
/// neither a token nor a file to resolve against. Waiting on one would only
/// report a failure the user cannot act on; the conversation is recovered by
/// its start time when a restore or resume actually needs the ID.
pub(crate) fn codex_launch_can_be_resolved(goal: &str, session_id: Uuid) -> bool {
    goal.contains(&native_tracking_token(session_id))
}

/// Antigravity records the current workspace and provider-owned conversation
/// ID in its local history after the first turn starts. Unlike Codex, it has
/// no caller-supplied new-session ID, so select only one conversation created
/// for this project at or after this managed launch. Ambiguity fails closed.
/// A resolver must fail closed: choosing either candidate would attach a
/// conversation that may belong to another launch.
pub(crate) fn unique_native_session_match(
    matches: Vec<CliSession>,
) -> UiResult<Option<CliSession>> {
    match matches.as_slice() {
        [] => Ok(None),
        [session] => Ok(Some(session.clone())),
        _ => Err(
            tr("この起動トークンを持つ Codex 履歴が複数あります。再開 ID を一意に特定できないため、中止します")
                .into(),
        ),
    }
}

pub(crate) fn matches_native_session_for_launch(
    session: &CliSession,
    provider: CliProvider,
    tracking_token: &str,
) -> bool {
    session.provider == provider
        && session
            .last_user_message
            .as_ref()
            .is_some_and(|message| message.contains(tracking_token))
}

impl CodexAppServer {
    fn start() -> Result<Self> {
        // The one child that cannot go through `run_command_with_output_limit`:
        // it is long-lived and spoken to over stdio, while that function pipes
        // and reads to completion. So it drops INHERITED_REPOSITORY_POINTERS
        // itself. Codex reads git state for its own context, and a `GIT_DIR`
        // inherited from whatever launched Operon would point it at a
        // repository that is not the one being restored into.
        let mut command = Command::new("codex");
        command
            .args(["app-server", "--stdio"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null());
        for pointer in INHERITED_REPOSITORY_POINTERS {
            command.env_remove(pointer);
        }
        let mut child = command
            .spawn()
            .context(tr("履歴復元のための Codex app-server の起動"))?;
        let stdin = child
            .stdin
            .take()
            .context(tr("Codex app-server の標準入力を開く処理"))?;
        let stdout = child
            .stdout
            .take()
            .context(tr("Codex app-server の標準出力を開く処理"))?;
        Ok(Self {
            child,
            stdin,
            stdout: BufReader::new(stdout),
            next_request_id: 1,
            thread_id: None,
        })
    }

    fn initialize(&mut self) -> UiResult<()> {
        self.request(
            "initialize",
            serde_json::json!({
                "clientInfo": {"name": "xirp-copy", "version": env!("CARGO_PKG_VERSION")}
            }),
        )?;
        Ok(())
    }

    fn start_thread(&mut self, cwd: &Path) -> UiResult<(String, PathBuf)> {
        let response = self.request(
            "thread/start",
            serde_json::json!({
                "cwd": cwd,
                "ephemeral": false,
                "serviceName": "xirp-copy"
            }),
        )?;
        let thread = response.get("thread").ok_or_else(|| {
            tr("Codex app-server が、新しいスレッドを返しませんでした。").to_owned()
        })?;
        let id = thread
            .get("id")
            .and_then(|value| value.as_str())
            .filter(|id| is_safe_cli_session_id(id))
            .ok_or_else(|| {
                tr("Codex app-server が、安全でないスレッド ID を返しました。").to_owned()
            })?
            .to_owned();
        let path = thread
            .get("path")
            .and_then(|value| value.as_str())
            .map(PathBuf::from)
            .filter(|path| path.is_absolute())
            .ok_or_else(|| {
                tr("Codex app-server が、永続セッションのパスを返しませんでした。").to_owned()
            })?;
        self.thread_id = Some(id.clone());
        Ok((id, path))
    }

    /// Send the projected turns to the thread, in batches the app server will
    /// accept. It receives the same list every other destination writes and the
    /// same list `verify_codex_imported_transcript` checks against, so the
    /// injection and its verification cannot read the source differently.
    fn inject_turns(&mut self, turns: &[(&'static str, String)]) -> UiResult<usize> {
        let mut records = 0;
        let mut batch = Vec::<serde_json::Value>::new();
        let mut batch_bytes = 0_usize;
        for (role, text) in turns {
            let item = codex_import_item(role, text, records);
            let item_bytes = serde_json::to_vec(&item)
                .map_err(|error| tf!("履歴レコードのエンコード: {error}", error = error))?
                .len();
            if !batch.is_empty()
                && (batch.len() >= HISTORY_IMPORT_MAX_ITEMS_PER_REQUEST
                    || batch_bytes.saturating_add(item_bytes) > HISTORY_IMPORT_MAX_REQUEST_BYTES)
            {
                self.inject_items(&batch)?;
                batch.clear();
                batch_bytes = 0;
            }
            batch_bytes = batch_bytes.saturating_add(item_bytes);
            batch.push(item);
            records += 1;
        }
        if !batch.is_empty() {
            self.inject_items(&batch)?;
        }
        Ok(records)
    }

    fn inject_items(&mut self, items: &[serde_json::Value]) -> UiResult<()> {
        let thread_id = self
            .thread_id
            .as_deref()
            .ok_or_else(|| tr("Codex app-server に、復元先スレッドがありません。").to_owned())?;
        self.request(
            "thread/inject_items",
            serde_json::json!({"threadId": thread_id, "items": items}),
        )?;
        Ok(())
    }

    fn request(&mut self, method: &str, params: serde_json::Value) -> UiResult<serde_json::Value> {
        let id = self.next_request_id;
        self.next_request_id += 1;
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": method,
            "params": params,
        });
        let encoded = serde_json::to_string(&request).map_err(|error| {
            tf!(
                "Codex app-server へのリクエストのエンコード: {error}",
                error = error
            )
        })?;
        self.stdin
            .write_all(encoded.as_bytes())
            .and_then(|_| self.stdin.write_all(b"\n"))
            .and_then(|_| self.stdin.flush())
            .map_err(|error| {
                tf!(
                    "Codex app-server へのリクエストの書き込み: {error}",
                    error = error
                )
            })?;
        let mut line = String::new();
        loop {
            line.clear();
            let read = self.stdout.read_line(&mut line).map_err(|error| {
                tf!(
                    "Codex app-server からの応答の読み取り: {error}",
                    error = error
                )
            })?;
            if read == 0 {
                return Err(tr("Codex app-server が、履歴復元を完了する前に停止しました。").into());
            }
            let value = serde_json::from_str::<serde_json::Value>(&line)
                .map_err(|error| tf!("Codex app-server の応答の解析: {error}", error = error))?;
            if value.get("id").and_then(|value| value.as_u64()) != Some(id) {
                continue;
            }
            if let Some(error) = value.get("error") {
                return Err(tf!(
                    "Codex app-server が履歴復元を拒否しました: {error}",
                    error = error
                ));
            }
            return value
                .get("result")
                .cloned()
                .ok_or_else(|| tr("Codex app-server が結果を返しませんでした。").into());
        }
    }

    fn stop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

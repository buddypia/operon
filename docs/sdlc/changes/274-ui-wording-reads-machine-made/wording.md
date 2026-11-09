# Wording changes, change 274

Every row of `src/i18n_tables.rs` this change rewrites, before and after.
A row keeps its other languages as they were unless they are listed.
Cells are Rust string-literal source text: `\"` is one quote on screen.

## Japanese (the message id) — 42 rows

| Before | After |
|---|---|
|  取り込みは書き込まれましたが、ディスクへの永続化を確認できませんでした。Operon が自動的に再試行します。エラー: {error} |  取り込みは書き込まれましたが、ディスクに保存されたことを確認できませんでした。Operon が自動的に再試行します。エラー: {error} |
|  新しい空の記録は書き込まれましたが、ディスクへの永続化を確認できませんでした。Operon が自動的に再試行します。エラー: {sync_error} |  新しい空の記録は書き込まれましたが、ディスクに保存されたことを確認できませんでした。Operon が自動的に再試行します。エラー: {sync_error} |
| AIで修正 | AI で修正 |
| Antigravity の履歴エントリのエンコード: {error} | Antigravity の履歴エントリをエンコードできませんでした: {error} |
| Claude Code と Codex のセッション開始時に、ナレッジの題名 (最大 10 件) と検索方法をその CLI に渡しています。 | Claude Code と Codex のセッション開始時に、ナレッジの題名（最大 10 件）と検索方法をその CLI に渡しています。 |
| Codex が、取り込みターン operon-import-{imported_index} を永続化しませんでした。 | Codex が取り込みターン operon-import-{imported_index} を保存しませんでした。 |
| Codex の再生レコードのエンコード: {error} | Codex の再生レコードをエンコードできませんでした: {error} |
| {ICON_TREE_MAIN} メインの作業ツリー | {ICON_TREE_MAIN} メインの worktree |
| {external_effect}。メタデータは書き込まれましたが、ディスクへの永続化を確認できませんでした。Operon が自動的に再試行します。確認が済むまで強制終了しないでください。エラー: {error} | {external_effect}。メタデータは書き込まれましたが、ディスクに保存されたことを確認できませんでした。Operon が自動的に再試行します。確認が済むまで強制終了しないでください。エラー: {error} |
| {imported} 件の Git プロジェクトを取り込みました。上限付きスキャンの限度に達したため、一部のフォルダは省略されている可能性があります。 | {imported} 件の Git プロジェクトを取り込みました。スキャンの上限に達したため、一部のフォルダは省略されている可能性があります。 |
| {label}: 永続化未確認（{error}） | {label}: 保存未確認（{error}） |
| {p0} 会話履歴の自動復旧に失敗しました。元の会話は変更されていません: {error} | {p0} の会話履歴の自動復旧に失敗しました。元の会話は変更されていません: {error} |
| {p0}の読みです | {p0}の値です |
| このセッションの {label} 会話IDを復元できませんでした。Discover local CLI sessions から該当する {label} 会話を選んで復元してください。 | このセッションの {label} 会話IDを復元できませんでした。「ローカル CLI セッションを検出」から該当する {label} 会話を選んで復元してください。 |
| このセッションは待機中の {dependent_count} 件から依存されています。先にそれらを開始または削除してください。 | このセッションに依存している待機中のセッションが {dependent_count} 件あります。先にそれらを開始または削除してください。 |
| このターミナルに記録されていた Claude 会話は、ターミナルが作られる前に終わった別の会話でした。実際の会話ログを特定できません。Discover local CLI sessions から該当する会話を選んでください。 | このターミナルに記録されていた Claude 会話は、ターミナルが作られる前に終わった別の会話でした。実際の会話ログを特定できません。「ローカル CLI セッションを検出」から該当する会話を選んでください。 |
| このターミナルは新しい会話を開始したばかりで、まだ会話ログが書かれていません。1 度発言してから、もう一度お試しください。 | このターミナルは新しい会話を開始したばかりで、まだ会話ログが書かれていません。1 度メッセージを送ってから、もう一度お試しください。 |
| キャンセル復旧の永続化が未確認のため、tmux は変更していません。Operon を開いたままにし、警告が消えてからもう一度「停止」を選んでください。 | キャンセル復旧の情報を保存できたか確認できていないため、tmux は変更していません。Operon を開いたままにし、警告が消えてからもう一度「停止」を選んでください。 |
| キャンセル復旧用メタデータは書き込まれましたが、ディスクへの永続化を確認できませんでした。Operon が自動的に再試行します。エラー: {error} | キャンセル復旧用メタデータは書き込まれましたが、ディスクに保存されたことを確認できませんでした。Operon が自動的に再試行します。エラー: {error} |
| キャンセル指示の復旧コピーは 1 つ永続化されましたが、複製の修復が必要です。停止しても安全です。Operon が自動的に再試行します。{write_details} | キャンセル指示の復旧コピーは 1 つ保存されましたが、複製の修復が必要です。停止しても安全です。Operon が自動的に再試行します。{write_details} |
| キャンセル指示は書き込まれましたが、どちらの復旧コピーも永続化を確認できていません。Operon が自動的に再試行し、tmux は動作したままになります。{write_details} | キャンセル指示は書き込まれましたが、どちらの復旧コピーもディスクへの保存を確認できていません。Operon が自動的に再試行し、tmux は動作したままになります。{write_details} |
| セッションが停止したか確認できませんでした: {error}。永続化されたキャンセル指示は保留のままです。もう一度「停止」を実行してください。 | セッションが停止したか確認できませんでした: {error}。保存済みのキャンセル指示は保留のままです。もう一度「停止」を実行してください。 |
| セッションのメタデータを再度書き込みましたが、ディスクへの永続化はまだ確認できていません。Operon が再試行します。エラー: {error} | セッションのメタデータを再度書き込みましたが、ディスクに保存されたかはまだ確認できていません。Operon が再試行します。エラー: {error} |
| ターンを終えて手が空いています | ターンを終え、次の依頼を出せます |
| ターンを終えて手が空いているセッション | ターンを終え、次の依頼を出せるセッション |
| チェックポイントの復元に失敗しました: {error} | チェックポイントを復元できませんでした: {error} |
| テストの実行に失敗しました | テストを実行できませんでした |
| ネイティブ再開 ID を待機中 | ネイティブ再開 ID を待っています |
| ハンク操作に失敗しました: {error} | ハンクを操作できませんでした: {error} |
| ローカル履歴をスナップショットし、コンパクトな Handoff バンドルを組み立てています · 経過 {elapsed_seconds} 秒 | ローカル履歴のスナップショットを取り、コンパクトな Handoff バンドルを組み立てています · 経過 {elapsed_seconds} 秒 |
| 上限付きのプロジェクトスキャン結果を表示しています。{truncated}{unreadable} | スキャンの上限内で見つかったプロジェクトを表示しています。{truncated}{unreadable} |
| 上限付きスキャンで、このマシンの Codex・Claude・Antigravity のセッションを {count} 件見つけました。一部の履歴は省略されています{unreadable}。 | スキャンの上限内で、このマシンの Codex・Claude・Antigravity のセッションを {count} 件見つけました。一部の履歴は省略されています{unreadable}。 |
| 上限付き検索で “{query}” に一致するローカル履歴を {count} 件見つけました（{files} ファイル、{mebibytes} MiB、{lines} 行を走査）。一部の履歴は省略されています{unreadable}。 | 検索の上限内で “{query}” に一致するローカル履歴を {count} 件見つけました（{files} ファイル、{mebibytes} MiB、{lines} 行を走査）。一部の履歴は省略されています{unreadable}。 |
| 取り込んだメタデータのディスク永続化はまだ確認できていません。再試行が終わるまで Operon を開いたままにしてください。 | 取り込んだメタデータをディスクに保存できたか、まだ確認できていません。再試行が終わるまで Operon を開いたままにしてください。 |
| 変更は書き込まれましたが、ディスクへの永続化を確認できませんでした。Operon が自動的に再試行します。確認が済むまで強制終了しないでください。エラー: {error} | 変更は書き込まれましたが、ディスクに保存されたことを確認できませんでした。Operon が自動的に再試行します。確認が済むまで強制終了しないでください。エラー: {error} |
| 復元した Claude Code の会話のエンコード: {error} | 復元した Claude Code の会話をエンコードできませんでした: {error} |
| 手すき | アイドル |
| 次の記録は分類が分からず、読み取れる本文もなかったため復元先へ渡していません: {kinds}。 | 次の記録は分類できず、読み取れる本文もなかったため、復元先へ渡していません: {kinds}。 |
| 要対応になるとここに先頭表示されます | 要対応になったセッションは、ここの一番上に表示されます |
| 記録しています… | コミットしています… |
| 選ばれたアカウントが見つかりません。 | 選んだアカウントが見つかりません。 |
| 💡 セマンティック検索を有効にすると表記揺れも検索できます。 | 💡 意味検索を有効にすると表記揺れも検索できます。 |

## English — 94 rows

| Before | After |
|---|---|
|  The import was written, but Operon could not confirm it was persisted to disk. Operon will retry automatically. Error: {error} |  The import was written, but Operon could not confirm it reached the disk. Operon will retry automatically. Error: {error} |
|  The new empty record was written, but Operon could not confirm it was persisted to disk. Operon will retry automatically. Error: {sync_error} |  The new empty record was written, but Operon could not confirm it reached the disk. Operon will retry automatically. Error: {sync_error} |
| Attempting to run .operon/run.sh. | About to run .operon/run.sh. |
| AI Code Review Evaluation | AI code review evaluation |
| AI Review | AI review |
| AI reviewing… | Reviewing with AI… |
| Cannot create the Antigravity conversation store {p0}: {error} | Could not create the Antigravity conversation store {p0}: {error} |
| Cannot write out the Antigravity conversation store {p0}: {error} | Could not write the Antigravity conversation store {p0}: {error} |
| Could not write out the Antigravity history index {p0}: {error} | Could not write the Antigravity history index {p0}: {error} |
| Cargo clippy error occurred | Cargo clippy reported an error |
| Codex did not persist import turn operon-import-{imported_index}. | Codex did not save import turn operon-import-{imported_index}. |
| The Codex restore destination history ends with an incomplete record, so appending is stopped. | Stopped appending because the Codex restore history ends with an incomplete record. |
| The Codex restore destination history has no parent directory. | The Codex restore history has no parent directory. |
| Could not initialise EmbeddingGemma 2 | Could not initialize EmbeddingGemma 2 |
| Pull request creation is already running. | PR creation is already running. |
| Identify conflicts with the main branch and resolve them safely preserving intent. | Identify conflicts with the main branch and resolve them safely, preserving the original intent. |
| Preparing tmux session · {elapsed_seconds}s elapsed · Terminal will open automatically when ready | Preparing tmux session · {elapsed_seconds}s elapsed · The terminal opens automatically when ready |
| Land worktree into main | Merge worktree into main |
| Land worktree into main… | Merge worktree into main… |
| Landing worktree into main… | Merging worktree into main… |
| Removing the worktree… | Deleting the worktree… |
| Removed the worktree. The Git branch is kept. | Deleted the worktree. The Git branch is kept. |
| {action}: failed {error} | {action} failed: {error} |
| {agent} has no way to be told which account to use. | {agent} does not support choosing an account. |
| {chord} is claimed by both \\\\\\\"{first}\\\\\\\" and \\\\\\\"{second}\\\\\\\". | {chord} is claimed by both \"{first}\" and \"{second}\". |
| Imported {imported} Git projects. The bounded scan hit its limit, so some folders may have been skipped. | Imported {imported} Git projects. The scan hit its limit, so some folders may have been skipped. |
| {label}: durability unconfirmed ({error}) | {label}: not confirmed on disk ({error}) |
| {name}  {n} entries | {name}  {n} notes |
| {n} entries | {n} notes |
| {n} match | Matches: {n} |
| Level with {p0}. | Up to date with {p0}. |
| {p0} changed, so nothing was run. Please look at it again. | {p0} changed, so nothing was run. Check it again. |
| {p0} is a shell script that lives in the repository. It runs in a session of its own and can be stopped part-way. | {p0} is a shell script stored in the repository. It runs in its own session and can be stopped partway. |
| {p1} ahead of and {p2} behind {p0}. | {p1} commits ahead of and {p2} behind {p0}. |
| … {p0} lines in all | … {p0} lines total |
| A session is active or queued in this worktree or the main working tree. Stop, complete, or cancel it before landing. | A session is active or queued in this worktree or the main working tree. Stop, complete, or cancel it before merging. |
| Before removing this worktree, stop, finish, or cancel all in-use and queued sessions. | Before deleting this worktree, stop, finish, or cancel all active and queued sessions. |
| There is no way to ask this agent for a draft. | This agent cannot be asked to write a draft. |
| Could not restore the {label} conversation ID for this session. Choose the matching {label} conversation from Discover local CLI sessions and restore it. | Could not restore the {label} conversation ID for this session. Choose the matching {label} conversation from \"Detect local CLI sessions\" and restore it. |
| The Claude conversation recorded for this terminal is a different conversation that ended before the terminal was created, so the actual conversation log cannot be identified. Choose the conversation from Discover local CLI sessions. | The Claude conversation recorded for this terminal is a different conversation that ended before the terminal was created, so the actual conversation log cannot be identified. Choose the conversation from \"Detect local CLI sessions\". |
| Could not work out this repository's starting branch. One of origin/HEAD, origin/main, or main is needed. | Could not determine this repository's starting branch. One of origin/HEAD, origin/main, or main is needed. |
| Several Claude histories match this managed session. Aborting so the wrong conversation is not restored. | Several Claude histories match this managed session. Stopping so the wrong conversation is not restored. |
| This only closes this panel. The restore continues, and the result appears as a notice. | This only closes the panel. The restore continues, and the result appears as a notice. |
| Close this view only. Launch will continue in background, and terminal opens upon completion. | This only closes the view. Launch continues in the background, and the terminal opens when it finishes. |
| Other ({n} entries) | Other ({n} notes) |
| {p0} more. Add to your search. | {p0} more. Add more search terms. |
| very soon | in a moment |
| Choosing an account was cancelled. | Account selection was canceled. |
| These comments are too large to send at once ({p0} of them; the limit is {p1} bytes). Send them in two goes. | These comments are too large to send at once ({p0} comments; the limit is {p1} bytes). Send them in smaller batches. |
| Fold side panel | Collapse side panel |
| Every shortcut. Change them in the file | All shortcuts. Edit them in the file |
| Could not confirm the session stopped: {error}. The delete is still pending: press 「停止」 again and the record is deleted once the stop is confirmed. | Could not confirm the session stopped: {error}. The delete is still pending: choose \"Stop\" again and the record is deleted once the stop is confirmed. |
| No sessions yet. Start one with \\\\\\\"New session\\\\\\\" in the toolbar. | No sessions yet. Start one with \"New session\" in the toolbar. |
| How the setup ended could not be read. Open the session to see. | Could not tell how the setup ended. Open the session to check. |
| Notify on turn completion, input waits, and exits | Notify when a turn finishes, input is needed, or a session exits |
| Turn finished; free for a new request | Turn finished; ready for your next request |
| Sessions that finished their turn and are free | Sessions that finished their turn |
| Rollback to Checkpoint | Roll back to checkpoint |
| Test execution failed | Could not run the tests |
| Insert Template | Insert template |
| The diff cannot be shown because this is a binary file. | Cannot show a diff for a binary file. |
| Copy the path | Copy path |
| The path was copied. | Path copied. |
| Clear the filter | Clear filter |
| Open in the browser | Open in browser |
| Create Pull Request | Create pull request |
| The merge could not finish, so the landing was undone and {mainline} is back where it was: {err} | The merge could not finish, so it was undone and {mainline} is back where it was: {err} |
| Execute Merge | Run merge |
| The main working tree is on {current} instead of {mainline}. Please check out {mainline} in the main working tree before landing. | The main working tree is on {current} instead of {mainline}. Check out {mainline} in the main working tree before merging. |
| Another merge is in progress in the main working tree, so nothing was landed. Finish or abort that merge, then try again. | Another merge is in progress in the main working tree, so nothing was merged. Finish or abort that merge, then try again. |
| Copied review notes to clipboard in Markdown format. | Copied review notes to the clipboard as Markdown. |
| · Only candidates worth keeping are summarised by an available CLI (Claude Code, Codex, or Antigravity; up to 30 times a day). | · Only candidates worth keeping are summarized by an available CLI (Claude Code, Codex, or Antigravity; up to 30 times a day). |
| Showing capped project scan results.{truncated}{unreadable} | Showing projects found within the scan limit.{truncated}{unreadable} |
| A bounded scan found {count} Codex, Claude, and Antigravity sessions on this machine. Some history was left out{unreadable}. | Found {count} Codex, Claude, and Antigravity sessions on this machine within the scan limit. Some history was left out{unreadable}. |
| A bounded search found {count} local history matches for “{query}” ({files} files, {mebibytes} MiB, {lines} lines scanned). Some history was left out{unreadable}. | Found {count} local history matches for “{query}” within the search limit ({files} files, {mebibytes} MiB, {lines} lines scanned). Some history was left out{unreadable}. |
| Create as Draft PR | Create as draft PR |
| Cancelled | Canceled |
| Please review content and approve execution: | Review the contents and approve the run: |
| Adds an Operon entry to each CLI’s settings file so the CLI itself reports the start and end of a turn. Turning this off removes those entries and leaves the screen reading to decide. | Adds an Operon entry to each CLI’s settings file so the CLI itself reports the start and end of a turn. Turning this off removes those entries and relies on screen reading alone. |
| Open in External Editor | Open in external editor |
| Investigate failing CI check or test logs, find root cause, and fix it. | Investigate the error logs of failing CI checks or tests, find the root cause, and fix it. |
| Review Run Script | Review run script |
| Create Manual Checkpoint | Create manual checkpoint |
| Setup | Procedure |
| Approve and Run | Approve and run |
| Cannot identify the new native session yet: {error}. Choose \"Discover local CLI sessions\" and resume by specifying the ID. | Cannot identify the new native session yet: {error}. Choose \"Detect local CLI sessions\" and resume by specifying the ID. |
| Write unit tests covering edge cases and error paths for recently added/changed features. | Write unit tests covering edge cases and error paths for recently added or changed features. |
| Today's judging limit is reached. The rest continues tomorrow. | Reached today's judging limit. The rest will be processed tomorrow. |
| Invalid or unsafe branch name specified: {branch} | Invalid or unsafe branch name: {branch} |
| Merge changes from current worktree into main branch. | Merge the current worktree's changes into the main branch. |
| Sent recent error to agent. | Sent the recent error to the agent. |
| Open a plain terminal | Open an empty terminal |
| The landing conflicted, so it was undone and {mainline} is back where it was. Conflicting files: {files} | The merge conflicted, so it was undone and {mainline} is back where it was. Conflicting files: {files} |
| The chosen account is not there. | The chosen account was not found. |

## Korean — 92 rows

| Before | After |
|---|---|
|  가져온 내용을 썼지만 디스크에 영구 저장되었는지 확인하지 못했습니다. Operon이 자동으로 다시 시도합니다. 오류: {error} |  가져온 내용을 썼지만 디스크에 저장되었는지 확인하지 못했습니다. Operon이 자동으로 다시 시도합니다. 오류: {error} |
|  새 빈 기록을 썼지만 디스크에 영구 저장되었는지 확인하지 못했습니다. Operon이 자동으로 다시 시도합니다. 오류: {sync_error} |  새 빈 기록을 썼지만 디스크에 저장되었는지 확인하지 못했습니다. Operon이 자동으로 다시 시도합니다. 오류: {sync_error} |
| 3단계 이내에서 새 Git 리포지토리를 찾지 못했습니다. | 3단계 이내에서 새 Git 저장소를 찾지 못했습니다. |
| 5시간 창 | 5시간 한도 |
| 7일 창 | 7일 한도 |
| Codex가 가져온 턴 operon-import-{imported_index}을(를) 영구 저장하지 않았습니다. | Codex가 가져온 턴 operon-import-{imported_index}을(를) 저장하지 않았습니다. |
| Git 차이점을 가져올 수 없습니다: {error} | Git 변경 내용을 가져올 수 없습니다: {error} |
| Git 리포지토리가 아닙니다. AI 실행과 파일 보기는 사용할 수 있습니다. | Git 저장소가 아닙니다. AI 실행과 파일 보기는 사용할 수 있습니다. |
| Git 리포지토리를 포함한 워크스페이스 (스캔 깊이: 3) | Git 저장소를 포함한 워크스페이스 (스캔 깊이: 3) |
| Markdown은 미리보기로, 변경된 파일은 차이점으로 열 수 있습니다. | Markdown은 미리보기로, 변경된 파일은 diff로 열 수 있습니다. |
| [차이가 1MB를 넘어 잘렸습니다] | [diff가 1MB를 넘어 잘렸습니다] |
| push 하고 있습니다… | push하고 있습니다… |
| push 했습니다. | push했습니다. |
| push 하지 못했습니다: {error} | push하지 못했습니다: {error} |
| tmux 및 CLI 준비 중 | tmux와 CLI 준비 중 |
| worktree의 변경은 프로젝트의 \\\\\\\"변경\\\\\\\" 탭에서 확인할 수 있습니다. | worktree의 변경은 프로젝트의 '변경' 탭에서 확인할 수 있습니다. |
| {arg} 은(는) 이 화면의 항목으로 지정해 주세요. | {arg}은(는) 이 화면의 항목으로 지정해 주세요. |
| {awaiting_input_count}개의 세션이 당신을 기다리고 있습니다 | 확인이 필요한 세션이 {awaiting_input_count}개 있습니다 |
| {chord}가 \\\\\\\"{first}\\\\\\\"와 \\\\\\\"{second}\\\\\\\"에서 겹칩니다. | {chord}이(가) '{first}', '{second}'에 모두 할당되어 있습니다. |
| Git 프로젝트 {imported}개를 가져왔습니다. 범위를 제한한 스캔이 한도에 도달하여 일부 폴더는 건너뛰었을 수 있습니다. | Git 프로젝트 {imported}개를 가져왔습니다. 스캔 한도에 도달해 일부 폴더는 건너뛰었을 수 있습니다. |
| {kind} · 신선도 {p0}% · 참조 {n} | {kind} · 최신성 {p0}% · 참조 {n} |
| {p0} MiB 크기의 파일입니다. 표시할 수 있는 파일은 {p1} MiB까지입니다. 오른쪽 위의 \"기본 앱으로 열기\"로 열 수 있습니다. | {p0} MiB 크기의 파일입니다. 표시할 수 있는 파일은 {p1} MiB까지입니다. 오른쪽 위의 '기본 앱으로 열기'로 열 수 있습니다. |
| {path}는 폴더가 아닙니다. | {path}은(는) 폴더가 아닙니다. |
| PATH에서 {selected_agent}을(를) 찾을 수 없습니다. 설치한 뒤 \"설정\"에서 \"도구 다시 확인\"을 눌러 주세요. | PATH에서 {selected_agent}을(를) 찾을 수 없습니다. 설치한 뒤 '설정'에서 '도구 다시 확인'을 눌러 주세요. |
| \"{label}\" 은(는) 대화 재개와 함께 지정할 수 없습니다. | '{label}'은(는) 대화 재개와 함께 지정할 수 없습니다. |
| 당신을 기다리는 세션이 없습니다 | 확인이 필요한 세션이 없습니다 |
| 지금 무엇이 실행 중이고 무엇이 기다리고 있는지. | 지금 무엇이 실행 중이고 어느 세션이 확인을 기다리는지. |
| 이 세션의 {label} 대화 ID를 복원하지 못했습니다. Discover local CLI sessions에서 해당 {label} 대화를 선택해 복원해 주세요. | 이 세션의 {label} 대화 ID를 복원하지 못했습니다. '로컬 CLI 세션 감지'에서 해당 {label} 대화를 선택해 복원해 주세요. |
| 이 터미널에 기록된 Claude 대화는 터미널이 만들어지기 전에 끝난 다른 대화여서 실제 대화 로그를 특정할 수 없습니다. Discover local CLI sessions에서 해당 대화를 선택해 주세요. | 이 터미널에 기록된 Claude 대화는 터미널이 만들어지기 전에 끝난 다른 대화여서 실제 대화 로그를 특정할 수 없습니다. '로컬 CLI 세션 감지'에서 해당 대화를 선택해 주세요. |
| 이 덩어리의 변경 사항을 폐기합니다 | 이 hunk의 변경 사항을 폐기합니다 |
| 이 덩어리를 스테이징합니다 | 이 hunk를 스테이징합니다 |
| 이 행에 대해 적기 | 이 행에 대해 쓰기 |
| 이 창만 닫습니다. 실행은 백그라운드에서 계속되며, 완료되면 터미널이 열립니다. | 이 표시만 닫습니다. 실행은 백그라운드에서 계속되며, 완료되면 터미널이 열립니다. |
| 계정 \"{name}\"을(를) 추가했습니다. | '{name}' 계정을 추가했습니다. |
| 키워드로 일치 | 키워드 일치 |
| 키 할당 | 키 바인딩 |
| 키 할당 파일이 있는 폴더를 열었습니다. | 키 바인딩 파일이 있는 폴더를 열었습니다. |
| 키 할당 파일을 만들었습니다. | 키 바인딩 파일을 만들었습니다. |
| 키 할당 파일을 만들지 못했습니다: {error} | 키 바인딩 파일을 만들지 못했습니다: {error} |
| 키 할당 파일을 읽지 못했습니다: {p0}바이트를 넘습니다. 기본 키로 시작했습니다. | 키 바인딩 파일을 읽지 못했습니다: {p0}바이트를 넘습니다. 기본 키로 시작했습니다. |
| 키 할당 파일을 읽지 못했습니다: {reason}기본 키로 시작했습니다. | 키 바인딩 파일을 읽지 못했습니다: {reason}기본 키로 시작했습니다. |
| 키 할당 파일을 읽지 못했습니다: 텍스트로 읽을 수 없습니다. 기본 키로 시작했습니다. | 키 바인딩 파일을 읽지 못했습니다: 텍스트로 읽을 수 없습니다. 기본 키로 시작했습니다. |
| 키 할당 파일 열기 | 키 바인딩 파일 열기 |
| 클릭하면 터미널 표시 · 더블 클릭하면 이름 변경 · 드래그하면 순서 변경 | 클릭하면 터미널 표시 · 두 번 클릭하면 이름 변경 · 드래그하면 순서 변경 |
| 스테이지 | 스테이징 |
| 세션이 중지되었는지 확인하지 못했습니다: {error}. 삭제는 보류 중입니다. 「停止」를 다시 누르면 중지를 확인한 뒤 기록을 삭제합니다. | 세션이 중지되었는지 확인하지 못했습니다: {error}. 삭제는 보류 중입니다. '중지'를 다시 누르면 중지를 확인한 뒤 기록을 삭제합니다. |
| 세션이 중지되었는지 확인하지 못했습니다: {error}. 저장된 취소 요청은 계속 보류 중입니다. \"중지\"를 다시 실행해 주세요. | 세션이 중지되었는지 확인하지 못했습니다: {error}. 저장된 취소 요청은 계속 보류 중입니다. '중지'를 다시 실행해 주세요. |
| 아직 세션이 없습니다. 툴바의 \\\\\\\"새 세션\\\\\\\"에서 시작할 수 있습니다. | 아직 세션이 없습니다. 툴바의 '새 세션'에서 시작할 수 있습니다. |
| 터미널을 열어 답변하세요 | 터미널을 열어 답변해 주세요 |
| 턴 완료·입력 대기·종료를 알림 | 턴 완료·입력 대기·종료 알림 받기 |
| 턴을 마치고 손이 비어 있습니다 | 턴을 마쳤습니다. 다음 요청을 보낼 수 있습니다 |
| 턴을 마치고 손이 비어 있는 세션 | 턴을 마치고 다음 요청을 보낼 수 있는 세션 |
| 디스크에서 다시 읽기 (편집 내용은 삭제됩니다) | 디스크에서 다시 읽기 (편집한 내용은 사라집니다) |
| 덩어리 변경 사항을 폐기했습니다. | hunk의 변경 사항을 폐기했습니다. |
| 덩어리를 스테이징했습니다. | hunk를 스테이징했습니다. |
| 덩어리 작업에 실패했습니다: {error} | hunk 작업에 실패했습니다: {error} |
| 차단됨: 선행 작업 “{parent}”이(가) 아직 완료되지 않았습니다. | 차단됨: 전제 조건 “{parent}”이(가) 아직 완료되지 않았습니다. |
| 맵을 만들 수 없습니다: {error} | 맵을 만들지 못했습니다: {error} |
| 병합 소스: | 병합 원본: |
| 병합 대상과 대상 브랜치가 동일합니다 ({mainline}). | 병합 원본과 병합 대상이 같은 브랜치입니다 ({mainline}). |
| 모델 (선택) | 모델(선택) |
| 워크스페이스 스캔 한도에 도달했지만 새 Git 리포지토리를 찾지 못했습니다. 대상 폴더를 좁혀 다시 시도해 주세요. | 워크스페이스 스캔 한도에 도달했지만 새 Git 저장소를 찾지 못했습니다. 대상 폴더를 좁혀 다시 시도해 주세요. |
| · 크기: 참조 횟수 · 진하기: 신선도 · | · 크기: 참조 횟수 · 진하기: 최신성 · |
| 상한이 적용된 프로젝트 스캔 결과를 표시하고 있습니다.{truncated}{unreadable} | 스캔 한도 안에서 찾은 프로젝트를 표시하고 있습니다.{truncated}{unreadable} |
| 범위를 제한한 스캔으로 이 컴퓨터에서 Codex, Claude, Antigravity 세션을 {count}개 찾았습니다. 일부 기록은 제외되었습니다{unreadable}. | 스캔 한도 안에서 이 컴퓨터의 Codex, Claude, Antigravity 세션을 {count}개 찾았습니다. 일부 기록은 생략했습니다{unreadable}. |
| 범위를 제한한 검색으로 “{query}”와(과) 일치하는 로컬 기록을 {count}건 찾았습니다 (파일 {files}개, {mebibytes} MiB, {lines}줄 스캔). 일부 기록은 제외되었습니다{unreadable}. | 검색 한도 안에서 “{query}”와(과) 일치하는 로컬 기록을 {count}건 찾았습니다 (파일 {files}개, {mebibytes} MiB, {lines}줄 스캔). 일부 기록은 생략했습니다{unreadable}. |
| 작업 트리 diff | 작업 트리 변경 내용 |
| 답변 대기 중이거나 전제가 실패해 멈춘 세션 | 답변 대기 중이거나 전제 조건이 충족되지 않아 멈춘 세션 |
| 외부 에디터에서 열기 | 외부 편집기에서 열기 |
| 실패한 CI 검사나 테스트 에러 로그를 조사하고 근본 원인을 파악해 수정해 주세요. | 실패한 CI 검사나 테스트의 오류 로그를 조사하고 근본 원인을 파악해 수정해 주세요. |
| 대상 덩어리를 찾을 수 없습니다 | 대상 hunk를 찾을 수 없습니다 |
| 차이점 | 변경 내용 |
| 변경 사항을 복사했습니다. | 변경 내용을 복사했습니다. |
| 변경 사항을 복사할 수 없습니다: {error} | 변경 내용을 복사할 수 없습니다: {error} |
| 차이점 보기 | 변경 내용 보기 |
| 설치 방법 | 절차 |
| 새 네이티브 세션을 아직 확인할 수 없습니다: {error}. \"Discover local CLI sessions\"에서 ID를 지정해 재개해 주세요. | 새 네이티브 세션을 아직 확인할 수 없습니다: {error}. '로컬 CLI 세션 감지'에서 ID를 지정해 재개해 주세요. |
| 다음 대응 필요 세션으로 | 다음 확인 필요 세션으로 |
| 다음 대응 필요 세션을 엽니다 | 다음 확인 필요 세션을 엽니다 |
| 유효하지 않거나 위험한 브랜치명이 지정되었습니다: {branch} | 유효하지 않거나 위험한 브랜치 이름입니다: {branch} |
| 현재 Git 변경분 | 현재 Git 변경 내용 |
| 현재 worktree의 변경 사항을 main 브랜치에 병合합니다. | 현재 worktree의 변경 사항을 main 브랜치에 병합합니다. |
| 일반 터미널 열기 | 빈 터미널 열기 |
| 대응 필요 | 확인 필요 |
| 대응 필요 {p0} | 확인 필요 {p0} |
| 대응 필요 {p0}  {chord} | 확인 필요 {p0}  {chord} |
| 대응이 필요해지면 여기에 맨 먼저 표시됩니다 | 확인이 필요해지면 여기에 맨 먼저 표시됩니다 |
| 대응이 필요한 세션 보기 | 확인이 필요한 세션 보기 |
| 기록하고 있습니다… | 커밋하고 있습니다… |
| 실행 명령 | 실행 명령어 |
| 신선도 | 최신성 |
| (원본 기록 {records}개 중 대화 {restored}개와 사고 {reasoning}개를 가져왔고, 운영 레코드 {operational}개, 미분류 {unclassified}개, 본문 없음 {empty}개는 제외했습니다) | (원본 기록 {records}개 중 대화 {restored}개와 추론 {reasoning}개를 가져왔고, 운영 레코드 {operational}개, 미분류 {unclassified}개, 본문 없음 {empty}개는 제외했습니다) |

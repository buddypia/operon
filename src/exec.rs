use crate::*;

#[derive(Debug)]
pub(crate) struct LimitedCommandOutput {
    pub(crate) output: Output,
    pub(crate) stdout_truncated: bool,
    pub(crate) stderr_truncated: bool,
}

pub(crate) fn run_command_with_output_limit(
    command: &mut Command,
    timeout: Duration,
    stdout_limit: usize,
    stderr_limit: usize,
) -> Result<LimitedCommandOutput> {
    command.stdout(Stdio::piped()).stderr(Stdio::piped());
    // Every external tool this crate runs arrives here, which makes it the one
    // place a repository pointer inherited from Operon's own environment can be
    // dropped for all of them. See INHERITED_REPOSITORY_POINTERS in
    // src/config.rs for what one did when it was not.
    for pointer in INHERITED_REPOSITORY_POINTERS {
        command.env_remove(pointer);
    }
    let mut child = command.spawn()?;
    let stdout = child
        .stdout
        .take()
        .context(tr("コマンドの標準出力の取得"))?;
    let stderr = child
        .stderr
        .take()
        .context(tr("コマンドの標準エラー出力の取得"))?;
    let stdout_limit_hit = Arc::new(AtomicBool::new(false));
    let stderr_limit_hit = Arc::new(AtomicBool::new(false));
    let stdout_signal = Arc::clone(&stdout_limit_hit);
    let stderr_signal = Arc::clone(&stderr_limit_hit);
    let stdout_reader =
        thread::spawn(move || read_stream_limited(stdout, stdout_limit, stdout_signal));
    let stderr_reader =
        thread::spawn(move || read_stream_limited(stderr, stderr_limit, stderr_signal));
    let deadline = Instant::now() + timeout;
    let mut timed_out = false;
    let mut output_limited = false;
    let status = loop {
        match child.try_wait()? {
            Some(status) => break status,
            None if stdout_limit_hit.load(Ordering::Acquire) => {
                output_limited = true;
                let _ = child.kill();
                break child.wait()?;
            }
            None if Instant::now() < deadline => thread::sleep(Duration::from_millis(25)),
            None => {
                timed_out = true;
                let _ = child.kill();
                break child.wait()?;
            }
        }
    };
    let (stdout, stdout_truncated) = stdout_reader.join().unwrap_or_default();
    let (stderr, stderr_truncated) = stderr_reader.join().unwrap_or_default();
    if timed_out {
        return Err(anyhow!(tf!(
            "コマンドが {timeout} でタイムアウトしました",
            timeout = format!("{timeout:?}")
        )));
    }
    Ok(LimitedCommandOutput {
        output: Output {
            status,
            stdout,
            stderr,
        },
        stdout_truncated: output_limited || stdout_truncated,
        stderr_truncated,
    })
}

pub(crate) fn run_command_with_timeout(command: &mut Command, timeout: Duration) -> Result<Output> {
    let limited = run_command_with_output_limit(
        command,
        timeout,
        COMMAND_OUTPUT_MAX_BYTES,
        COMMAND_ERROR_MAX_BYTES,
    )?;
    if limited.stdout_truncated {
        return Err(anyhow!(tf!(
            "コマンド出力が安全上限の {p0} KiB を超えました",
            p0 = COMMAND_OUTPUT_MAX_BYTES / 1024
        )));
    }
    let mut output = limited.output;
    if limited.stderr_truncated {
        output
            .stderr
            .extend_from_slice(tr("\n... コマンドのエラー出力を切り詰めました").as_bytes());
    }
    Ok(output)
}

pub(crate) fn read_stream_limited(
    mut reader: impl Read,
    limit: usize,
    limit_hit: Arc<AtomicBool>,
) -> (Vec<u8>, bool) {
    let mut bytes = Vec::with_capacity(limit.min(64 * 1024));
    let mut buffer = [0_u8; 16 * 1024];
    let mut truncated = false;
    loop {
        let Ok(count) = reader.read(&mut buffer) else {
            break;
        };
        if count == 0 {
            break;
        }
        let remaining = limit.saturating_sub(bytes.len());
        let retained = remaining.min(count);
        bytes.extend_from_slice(&buffer[..retained]);
        if retained < count {
            truncated = true;
            limit_hit.store(true, Ordering::Release);
        }
    }
    (bytes, truncated)
}

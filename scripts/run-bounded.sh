#!/usr/bin/env bash
# Run a command with a hard time limit. Change 128.
#
#   bash scripts/run-bounded.sh SECONDS command [args...]
#
# Exit status is the command's, or 124 when the limit was reached. macOS has no
# timeout(1). Ending only the command is not enough: a child of it (a test's
# tmux server, a sleep) keeps the caller's pipe open and the caller waits on the
# pipe, not on the command. So the command runs in its own process group, and
# the whole group gets TERM, then KILL two seconds later.
#
# Used by scripts/ci-verified.sh around `gh` and by .claude/hooks/gate-merge.sh
# around its offline `cargo test --locked`: a hook that times out is a
# non-blocking error, so a gate has to end its own work in time and refuse.
set -uo pipefail

[ "$#" -ge 2 ] || { echo "usage: run-bounded.sh SECONDS command [args...]" >&2; exit 2; }
exec perl -e '
  my $limit = shift;
  my $pid = fork;
  defined $pid or exit 127;
  if (!$pid) { setpgrp(0, 0); exec @ARGV or exit 127 }
  $SIG{ALRM} = sub {
    kill "TERM", -$pid; sleep 2; kill "KILL", -$pid; exit 124;
  };
  alarm $limit;
  waitpid($pid, 0);
  exit($? & 127 ? 128 + ($? & 127) : $? >> 8);
' "$@"
# operon: end of scripts/run-bounded.sh

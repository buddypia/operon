# A scrub may edit closed records

## Problem

Removing a name from the repository has to edit the records of changes that are
already closed. `scripts/check-review.sh` cannot land that: it refuses a diff
that touches more than one change directory, refuses a `review.yaml` that lands
without code, and refuses to let a change directory be deleted. Change 100 hit
all three, so four historical names were left in closed records even
though the owner asked for all of them gone.

The owner approved changing the gate so that it can be done.

## Proposed Change

- A change may add a `scrub-names` file to its own directory: one name per
  line, `#` comments allowed, matched case-insensitively as plain text. The file
  has to be added by the diff being judged; one that was already committed is
  not a scrub.
- When exactly one touched directory adds `scrub-names`, each other directory in
  the diff is judged by two questions: does anything it adds, or any path it
  adds, contain a listed name, and did it lose one (in a removed line or a
  removed path). Gaining a name, or losing none, is refused and names the
  directory; otherwise the gate treats the diff as that one change. A scrub is
  not recognised by hand (`byhand` mode), only for the index or a commit, and a
  merge commit keeps the exemption it already had.
- Deleting a file from a closed record is refused, and a directory moved to a
  new name reads as a rename (statuses come from the whole tree), so a directory
  whose name carries a listed name can be renamed.
- The touched-directory list is read with `--no-renames`, so a record moved out
  of a closed directory into the owner's lists the directory it left and that
  directory is judged; a move into the owner's directory counts as a deletion.
  Outside a scrub this also means a rename across change directories is
  refused as a two-change diff, which is deliberate. Names in `scrub-names` are matched case-insensitively for
  ASCII only.
- The scan reads diffs with `--no-ext-diff --no-textconv --text`, runs its awk
  and grep under `LC_ALL=C` so a line that is not valid UTF-8 cannot be dropped,
  loops over directory names without word splitting, and exits 2 when a diff
  cannot be read.
- Nothing else is relaxed. The owner's own `review.yaml` is still read, still
  bound to the digest, and still has to land with its code. No success exit is
  added, so `the_review_gate_has_no_bypass` keeps its meaning; its digest check
  gets one narrow allowance for the scrub's own range variable.
- Add `a_scrub_may_edit_closed_records_only_to_lose_a_listed_name`, which
  asserts the plain refusal without `scrub-names`, the pass for a record that
  lost the name, and the refusal for a record that gained it.

## Risk and Review

Touches `scripts/check-review.sh` and the test that pins it: the
gate-configuration surface, high and paused. The owner's approval is the pause.
Because the gate that judges a commit is `refs/heads/main:scripts/check-review.sh`,
the exemption takes effect only after this change is merged; the scrub itself is
a separate change that lands after it.

Known limit, stated rather than hidden: a closed record that does lose a listed
name may change in other ways in the same edit. The exemption is for a
mechanical rename, and the reviewer of the scrub change is the one who reads that
diff.

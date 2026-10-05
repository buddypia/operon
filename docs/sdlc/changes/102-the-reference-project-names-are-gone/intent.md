# Historical names are gone

## Problem

Change 100 cleaned up historical notes, but left identifiers, file names and a handful of comments that still carried legacy names, and the closed change records that mentioned them
could not be edited. The owner asked for every occurrence to go: "全部削除して
キーワードが出るとまずい。" Change 101 made the closed records editable under a
`scrub-names` list; this change is that scrub.

## Proposed Change

- Rename what carried a name: the guard scripts' state directory and its
  environment variables become `.harness/` and `HARNESS_*`; the bundle receipt,
  backup directory, staleness checker and template paths take a neutral word
  for the mechanism; the branch-completion reference loses its source's name;
  change 073's directory is renamed.
- Reword the remaining provenance comments in the hooks and in `risk.yaml` and
  `routes.yaml` so they state what the code does.
- Widen `steering_and_source_do_not_name_external_projects` to
  the three names that remained, so none of them returns.
- Land the names in `scrub-names` for this one commit, then delete the file so
  the repository does not hold the list it just removed.

## Risk and Review

Touches the gate-configuration surface (`.claude/hooks`, `risk.yaml`,
`routes.yaml`) and the hook libraries under `.cli/` and `.claude/scripts/`, by
renames and comment rewording. The renamed runtime identifiers (`.harness/`,
`HARNESS_*`) are read only by the guard scripts and appear nowhere else. A
subprocess-safety review confirms no behaviour moved beyond the rename.

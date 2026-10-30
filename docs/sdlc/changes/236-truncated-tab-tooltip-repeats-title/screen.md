# Tooltip of a truncated label

Status: approved

No layout change. Hovering a truncated session tab shows one line, its full
title (egui's own elision tooltip). A document tab still shows its name, when
truncated, and its absolute path. The project-page name shows its name only
when truncated; the session-column name shows the path and the drag hint
without repeating the name; the terminal header shows the goal
only when it differs from the title.

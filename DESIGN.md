---
version: alpha
name: Operon
description: >-
  The design system for Operon, a local-first macOS workspace for
  coding-agent sessions. Three themes — Dark, Light, and a high-contrast dark —
  share one set of semantic roles, so a colour is chosen by what it means and
  the theme decides what it looks like.
omitted:
  - section: components
    reason: >-
      Components are egui immediate-mode calls rather than a component library,
      so there is nothing stable to reference by name. The roles below are what
      a call site actually picks from.
colors:
  # Brand. These four name the identity; every role below is mixed from them or
  # from the neutral ramp, and each theme re-mixes them for its own surfaces.
  primary: "#FF9438"
  secondary: "#78AFFF"
  tertiary: "#37D2A1"
  neutral: "#1B1B1B"

  # ---- Dark: the default theme ----
  dark-window: "#1B1B1B"
  dark-panel: "#1B1B1B"
  dark-raised: "#191919"
  dark-card: "#1E1E1E"
  dark-inset: "#0A0A0A"
  dark-row-selected: "#36271D"
  dark-scrim: "#000000DC"
  dark-on-scrim: "#FFBE5A"
  dark-control: "#3C3C3C"
  dark-control-hovered: "#464646"
  dark-control-active: "#373737"
  dark-selection: "#005C80"
  dark-selection-border: "#C0DEFF"
  dark-border-subtle: "#3C3C3C"
  dark-border: "#767676"
  dark-border-strong: "#FFFFFF"
  dark-text-strong: "#F2F4F7"
  dark-text: "#C9CDD3"
  dark-text-muted: "#9AA0A8"
  dark-text-faint: "#9399A2"
  dark-branch: "#96BEFF"
  dark-link: "#5AAAFF"
  dark-code-bg: "#404040"
  dark-accent: "#FF9438"
  dark-accent-text: "#FF9438"
  dark-accent-soft: "#FFBE5A"
  dark-on-accent: "#1A1004"
  dark-info: "#78AFFF"
  dark-success: "#37D2A1"
  dark-calm: "#7EC4B0"
  dark-warning: "#F6BB37"
  dark-danger: "#EE6666"
  dark-sand: "#B49678"
  dark-diff-added-bg: "#10281C"
  dark-diff-added: "#5AD79B"
  dark-diff-removed-bg: "#2E1618"
  dark-diff-removed: "#FF8A85"
  dark-diff-added-emphasis: "#1A5138"
  dark-diff-removed-emphasis: "#722D31"
  dark-agent-codex: "#30B8A7"
  dark-agent-claude: "#DE7748"
  dark-agent-antigravity: "#6996F4"
  dark-agent-unknown: "#92929E"
  dark-terminal-bg: "#0A0C10"
  dark-terminal-border: "#2D313A"
  dark-terminal-fg: "#CED4DE"
  dark-ansi-black: "#7B8494"
  dark-ansi-red: "#EE6666"
  dark-ansi-green: "#37D2A1"
  dark-ansi-yellow: "#F6BB37"
  dark-ansi-blue: "#78AFFF"
  dark-ansi-magenta: "#C184FC"
  dark-ansi-cyan: "#3ACDDE"
  dark-ansi-white: "#CED4DE"
  dark-ansi-bright-black: "#A6AEBC"
  dark-ansi-bright-red: "#FF8080"
  dark-ansi-bright-green: "#68E8B1"
  dark-ansi-bright-yellow: "#FFD261"
  dark-ansi-bright-blue: "#97C4FF"
  dark-ansi-bright-magenta: "#D5A8FF"
  dark-ansi-bright-cyan: "#64E1EF"
  dark-ansi-bright-white: "#F5F7FA"

  # ---- Light ----
  light-window: "#FAF9F7"
  light-panel: "#F7F5F2"
  light-raised: "#FFFFFF"
  light-card: "#EFECE6"
  light-inset: "#FFFFFF"
  light-row-selected: "#FBE6D2"
  light-scrim: "#000000C8"
  light-on-scrim: "#FFCD82"
  light-control: "#E6E2DB"
  light-control-hovered: "#DCD7CF"
  light-control-active: "#CFC9C0"
  light-selection: "#B4D6FA"
  light-selection-border: "#00537D"
  light-border-subtle: "#DCD7CF"
  light-border: "#877F72"
  light-border-strong: "#1A1C1E"
  light-text-strong: "#0F1113"
  light-text: "#24282C"
  light-text-muted: "#565C63"
  light-text-faint: "#5F656C"
  light-branch: "#1D4E98"
  light-link: "#1F5FBF"
  light-code-bg: "#ECE8E1"
  light-accent: "#E1701A"
  light-accent-text: "#A9490F"
  light-accent-soft: "#8A5310"
  light-on-accent: "#FFFCF8"
  light-info: "#1F5FBF"
  light-success: "#0B704F"
  light-calm: "#2E6A5A"
  light-warning: "#855C00"
  light-danger: "#B3261E"
  light-sand: "#7A5A34"
  light-diff-added-bg: "#DDF3E4"
  light-diff-added: "#0B6E3F"
  light-diff-removed-bg: "#FBE3E2"
  light-diff-removed: "#A62019"
  light-diff-added-emphasis: "#85CF9D"
  light-diff-removed-emphasis: "#F3A5A0"
  light-agent-codex: "#0D6E64"
  light-agent-claude: "#A6441B"
  light-agent-antigravity: "#2B4EBA"
  light-agent-unknown: "#545460"
  light-terminal-bg: "#FCFBF8"
  light-terminal-border: "#D5CFC6"
  light-terminal-fg: "#24282C"
  light-ansi-black: "#2B3036"
  light-ansi-red: "#B3261E"
  light-ansi-green: "#0B704F"
  light-ansi-yellow: "#855C00"
  light-ansi-blue: "#1F5FBF"
  light-ansi-magenta: "#783D9E"
  light-ansi-cyan: "#0D6975"
  light-ansi-white: "#565C63"
  light-ansi-bright-black: "#4A5057"
  light-ansi-bright-red: "#9B1E18"
  light-ansi-bright-green: "#095F43"
  light-ansi-bright-yellow: "#714E00"
  light-ansi-bright-blue: "#1950A3"
  light-ansi-bright-magenta: "#663288"
  light-ansi-bright-cyan: "#0B5A64"
  light-ansi-bright-white: "#1A1C1E"

  # ---- High contrast (dark) ----
  contrast-window: "#0C0D0F"
  contrast-panel: "#141416"
  contrast-raised: "#1C1D21"
  contrast-card: "#222328"
  contrast-inset: "#040507"
  contrast-row-selected: "#4A3423"
  contrast-scrim: "#000000EB"
  contrast-on-scrim: "#FFD08A"
  contrast-control: "#33363D"
  contrast-control-hovered: "#3E424B"
  contrast-control-active: "#2A2D34"
  contrast-selection: "#0A6E96"
  contrast-selection-border: "#DCEEFF"
  contrast-border-subtle: "#4A4F5A"
  contrast-border: "#7A8290"
  contrast-border-strong: "#FFFFFF"
  contrast-text-strong: "#FFFFFF"
  contrast-text: "#F5F7FA"
  contrast-text-muted: "#D6DAE0"
  contrast-text-faint: "#A9AFB8"
  contrast-branch: "#B4D2FF"
  contrast-link: "#8CC4FF"
  contrast-code-bg: "#2A2E36"
  contrast-accent: "#FFA45C"
  contrast-accent-text: "#FFB26E"
  contrast-accent-soft: "#FFD08A"
  contrast-on-accent: "#100A02"
  contrast-info: "#A6C8FF"
  contrast-success: "#5BE8BE"
  contrast-calm: "#A0D8C8"
  contrast-warning: "#FFD35C"
  contrast-danger: "#FF8A8A"
  contrast-sand: "#D6BA98"
  contrast-diff-added-bg: "#0B3324"
  contrast-diff-added: "#7EEEBC"
  contrast-diff-removed-bg: "#3A1A1C"
  contrast-diff-removed: "#FFA8A4"
  contrast-diff-added-emphasis: "#136E4A"
  contrast-diff-removed-emphasis: "#7D3236"
  contrast-agent-codex: "#5EDCCC"
  contrast-agent-claude: "#FF9E6E"
  contrast-agent-antigravity: "#96B8FF"
  contrast-agent-unknown: "#BEBECA"
  contrast-terminal-bg: "#040507"
  contrast-terminal-border: "#606876"
  contrast-terminal-fg: "#F5F7FA"
  contrast-ansi-black: "#7E8696"
  contrast-ansi-red: "#FF8A8A"
  contrast-ansi-green: "#5BE8BE"
  contrast-ansi-yellow: "#FFD35C"
  contrast-ansi-blue: "#A6C8FF"
  contrast-ansi-magenta: "#D6A8FF"
  contrast-ansi-cyan: "#78E2F0"
  contrast-ansi-white: "#F5F7FA"
  contrast-ansi-bright-black: "#A8B0BE"
  contrast-ansi-bright-red: "#FFB4B4"
  contrast-ansi-bright-green: "#96F5D6"
  contrast-ansi-bright-yellow: "#FFE494"
  contrast-ansi-bright-blue: "#C8DEFF"
  contrast-ansi-bright-magenta: "#E8CAFF"
  contrast-ansi-bright-cyan: "#AAF0FA"
  contrast-ansi-bright-white: "#FFFFFF"
typography:
  display:
    fontFamily: Sarasa UI J
    fontSize: 38px
    fontWeight: 400
    lineHeight: 1.1
  heading:
    fontFamily: Sarasa UI J
    fontSize: 20px
    fontWeight: 600
    lineHeight: 1.3
  metric:
    fontFamily: Sarasa UI J
    fontSize: 28px
    fontWeight: 400
    lineHeight: 1.1
  section:
    fontFamily: Sarasa UI J
    fontSize: 15px
    fontWeight: 600
    lineHeight: 1.35
  action:
    fontFamily: Sarasa UI J
    fontSize: 15px
    fontWeight: 600
    lineHeight: 1.3
  control:
    fontFamily: Sarasa UI J
    fontSize: 13.5px
    fontWeight: 400
    lineHeight: 1.3
  body-lg:
    fontFamily: Sarasa UI J
    fontSize: 16px
    fontWeight: 400
    lineHeight: 1.5
  body:
    fontFamily: Sarasa UI J
    fontSize: 14px
    fontWeight: 400
    lineHeight: 1.5
  label:
    fontFamily: Sarasa UI J
    fontSize: 12.5px
    fontWeight: 400
    lineHeight: 1.4
  caption:
    fontFamily: Sarasa UI J
    fontSize: 12px
    fontWeight: 400
    lineHeight: 1.4
  chip:
    fontFamily: Sarasa UI J
    fontSize: 11.5px
    fontWeight: 400
    lineHeight: 1.3
  terminal:
    fontFamily: Sarasa Mono J
    fontSize: 15px
    fontWeight: 400
    lineHeight: 1.0
spacing:
  xs: 4px
  sm: 8px
  md: 12px
  lg: 18px
  xl: 26px
  gutter: 10px
  page-gutter: 26px
  control-height: 28px
  control-height-small: 24px
  toolbar-height: 52px
  traffic-light-inset: 78px
  status-column: 104px
rounded:
  control: 6px
  card: 10px
  window: 12px
---

# Operon

## Overview

Operon is a workspace for watching several coding agents work at once. Most
of what a person does here is triage: which session is thinking, which one is
stuck on a question, which one failed, and which terminal do I open now. The
interface has to answer that from across the room and then get out of the way.

So it is quiet by default and loud only where it means something. Surfaces are
flat and near-monochrome; the saturated colours are reserved for state and for
the four vendor marks. Density is closer to a mail client than to a dashboard —
a person scans a list of twenty sessions, not a poster.

It is a native macOS application and behaves like one. There is no splash, no
brand moment, no animation that delays an answer.

Three themes ship. **Dark** is the default and the one the app was drawn for.
**Light** is a full re-mix rather than an inversion. **High contrast (dark)**
pushes the dark theme apart for anyone who needs it to be further apart. A
person picks one in Settings and it is stored locally.

## Colors

The palette is a near-monochrome ramp with a single warm accent, plus a fixed
set of status hues that never do any other job.

- **Primary (#FF9438) — Signal Orange.** The brand mark, the one button that
  starts work, and nothing else. It is the only warm colour on a cool screen,
  which is what makes a single orange button findable without a label.
- **Secondary (#78AFFF) — Console Blue.** Information that is on its way
  somewhere: a session starting, a branch name, a link. Blue is what the eye
  skips when it is hunting for trouble, which is exactly right for it.
- **Tertiary (#37D2A1) — Session Green.** Something is running and healthy. The
  most common non-neutral colour on screen, so it is the least saturated of the
  status hues.
- **Neutral (#1B1B1B) — Panel Ink.** The surface everything else is measured
  against, and the theme's whole identity in one value: the light theme swaps
  it for warm paper rather than white, because a full-screen window of pure
  white at midday is a lamp.

### Roles, not hues

Every colour a call site can reach is a **role**. `warning` is amber on the
dark themes and a deep ochre on the light one; both mean "this needs you", and
that is the only thing the drawing code knows or should know. The role table is
the single point where a theme exists.

The roles, in the order they appear on screen:

- **Surfaces** — `window`, `panel`, `raised`, `card`, `inset`, `row-selected`,
  `scrim`. Five steps is more than a flat design needs and exactly what a dense
  one does: a card inside a panel inside a window has to be separable without a
  shadow.
- **Controls** — `control`, `control-hovered`, `control-active`, `selection`,
  `selection-border`.
- **Lines** — `border-subtle` for separators, `border` for the outline that
  marks a hovered or chosen thing, `border-strong` for the one being pressed.
- **Text** — `text-strong`, `text`, `text-muted`, `text-faint`, plus `branch`
  and `link` for the two kinds of text that are scanned rather than read.
- **Brand and status** — `accent`, `accent-text`, `accent-soft`, `on-accent`,
  `info`, `success`, `calm`, `warning`, `danger`, `sand`.
- **Diff** — `diff-added-bg` and `diff-added`, `diff-removed-bg` and
  `diff-removed`, plus `diff-added-emphasis` and `diff-removed-emphasis`. Three
  roles per side, not one: see *A diff is read twice*.
- **Vendor marks** — `agent-codex`, `agent-claude`, `agent-antigravity`,
  `agent-unknown`.
- **Terminal** — `terminal-bg`, `terminal-border`, `terminal-fg`, and the
  sixteen ANSI slots.

### Why the accent splits in two

`accent` is a fill and `accent-text` is ink. On the dark themes they are the
same value, because Signal Orange on near-black is both a good button and
readable text. On paper it is neither: #FF9438 on #F7F5F2 measures 2.0:1, so
the light theme keeps a slightly deepened orange for fills and drops to
#A9490F for anything a person has to read. A theme that could not split those
two roles would have to choose between an unreadable label and a muddy button.

### Status is a meaning, not a colour

A session's state is computed as a *tone* — running, idle, needs you, failed,
neutral, recovered — and the tone is resolved to a colour only at the moment
the chip is drawn. That keeps the state machine theme-free and keeps the two
from drifting: there is no place to add a state and forget its light-theme
colour.

- `info` — starting, or about to start.
- `success` — working right now.
- `calm` — finished its turn, ready for the next request. Deliberately quieter
  than `success`: it is a state, not an achievement.
- `warning` — waiting on the person reading the screen. The one status worth
  interrupting for.
- `danger` — ended badly, or a destructive button. Reserved for an outcome the
  app actually read: a run whose terminal disappeared before anyone saw how it
  ended is not a failure, and painting it red turns a record into an accusation.
- `sand` — recovered from a terminal this app did not start, stopped by hand, or
  over with its outcome never observed.

### A diff is read twice

Reviewing what an agent wrote is the one screen in this app where colour is
the primary carrier of meaning rather than a reinforcement of it, so the diff
gets three roles per side instead of one. They answer three different
questions, and a screen that answered only the first is the screen this section
used to describe.

`diff-added-bg` and `diff-removed-bg` are washes behind the whole row. They are
what the eye counts from a distance — how much did this agent touch — and they
are deliberately faint: a diff is a page of code, and code that is read has to
sit on a surface, not inside a highlighter. Against the editor surface they
measure between 1.1:1 and 1.5:1, which is the same order as GitHub's.

`diff-added` and `diff-removed` are inks, and they are what a person who cannot
separate the two hues actually reads: they colour the `+` and `-` in the gutter,
which carry the meaning on their own, and they colour the `+12 −3` summary above
the file.

`diff-added-emphasis` and `diff-removed-emphasis` are the third pair, and they
sit *inside* the wash rather than beside it. Where an agent rewrote a line in
place, they band the words that actually moved. That is why they are stronger
than the washes and are read against them rather than against the page: a band
a few characters wide has to be found at a glance, and it is never drawn on a
bare surface. Against their own wash they measure at least 1.5:1, which is
above GitHub's own word-level marks; the code on top of them still clears
4.5:1. Nothing outside a matched pair of rewritten lines is ever marked, so
seeing one of these bands is itself information: it means this line is a
rewrite, not a replacement.

Neither pair reuses `success` or `danger`. An added line is not an achievement
and a removed line is not a failure — an agent that deletes forty lines of dead
code has done something good in red. A status hue that also meant "diff" would
be a status hue that no longer meant one thing.

### Contrast

Every ink is checked against every surface it can land on — including the warm
tint of a selected session row, which is where the status chips and vendor
chips are read — at the WCAG 2.1 AA ratio for the size it is drawn at: 4.5:1
for text, 3:1 for `text-faint` and for the outlines that mark hover and press.
The check runs as a test, across all three themes, over roughly five hundred
pairs. A colour that fails it is not a taste disagreement; it is a build
failure.

The `colors` block above is machine-checked in both directions against the
constants in the binary, so it cannot describe an older version of the app.
The `typography`, `spacing`, and `rounded` blocks are not: they record what the
call sites use today and are maintained by hand.

Two deliberate consequences:

- **The dark theme's body text got brighter.** egui's default label ink is
  #8C8C8C on #1B1B1B — 3.9:1, under AA for body text, and the reason so many
  call sites had already hand-coloured their labels brighter. The `text` role
  is #C9CDD3 and ordinary labels now use it.
- **The outline that marks hover is visible.** It went from #464646 to #767676
  on the dark theme so that hovering a control is something you can see rather
  than something you can nearly see.

### The terminal's sixteen colours

The ANSI slots are a palette in the strict sense: a CLI writes `SGR 31` because
it means *red*, and the theme has to answer with its own red. The table is
therefore per-theme and in the standard order — black, red, green, yellow,
blue, magenta, cyan, white.

Two slots cannot be taken literally:

- **Black and white are relative.** Both ends are re-pointed at whatever reads
  against `terminal-bg`: the dark themes lift black to a mid grey, and the
  light theme darkens white. A terminal that painted a literal #FFFFFF for
  `SGR 37` on paper would be blank.
- **Uncoloured output is not `SGR 37`.** Most of what a terminal writes asks
  for no colour at all, and that request means "your normal ink", which is what
  `terminal-fg` is for. Reusing the white slot would have left the whole body
  of a light terminal in the muted grey reserved for a colour a CLI named on
  purpose. Bold uncoloured text still takes the bright end of the table, which
  reads as stronger in both directions — brighter on ink, darker on paper.
- **The 24-step grey ramp is mirrored on the light theme.** CLIs use a high
  step of the 256-colour ramp for "dimmer than my text" — it is where nearly
  every agent writes its hints and diffs. Those steps were picked against a
  black terminal, so on paper they are white on white. Mirroring the ramp keeps
  *dim* meaning dim. The 216-colour cube above it is passed through untouched,
  because mirroring a hue would answer a request for red with cyan.

## Typography

One family does the whole interface text: **Sarasa UI J**, bundled so a
Finder-launched app renders Japanese identically to a shell-launched one. The
terminal uses **Sarasa Mono J** (or the K / Term / Fixed variant a person
chooses in Settings), whose defining property is that ASCII occupies exactly
one cell and Japanese and Korean exactly two — without that, a TUI's box
drawing tears.

The scale is narrow on purpose. Eleven levels, from a 38px empty-state mark
down to an 11.5px chip, and most of the app lives in three of them: `body` for
prose, `label` for the secondary line under a title, `caption` for metadata.

Weight carries hierarchy more than size does. A section heading is `section`
semi-bold rather than a jump in size, because a list of session cards has to
stay a list.

Secondary lines get a colour and a size of their own rather than egui's `weak`,
which dims text to the point where a branch name — one of the two things used
to tell two sessions apart — becomes the least readable thing on the card.

## Iconography

One family does the icons too: **Phosphor Icons**, regular weight, bundled as a
font and named through the `ICON_*` constants in `src/glyphs.rs`. Drawing code
never writes a glyph — it names an action, the same way it names a colour role
rather than a hex value.

This exists because the vocabulary it replaced did not come from anywhere. It
was an arrow from Unicode's arrows block, a box from box drawing, a fullwidth
plus, a dingbat check — each drawn by whichever face in the fallback chain
happened to own that codepoint, at that face's stroke width, on that face's
baseline. A row of them read as a row of accidents. Phosphor's codepoints sit
in the private use area, so no text face can answer for them, and every mark in
the app is one weight and one grid.

Actions are bare pictograms; statuses are not. Every status mark is a circle,
and what sits inside it says which state: a closed ring is idle, a ring with a
gap is running, and a check, a cross, or a dash inside one is how the run ended
— the dash for the run that ended with nobody watching, which is neither of the
other two and must not borrow either one's mark. The
enclosure is what keeps a chip from reading as a button in the same row, and
the difference inside it is what a person who cannot use the colour still has.
Colour carries the meaning first (see *Status is a meaning, not a colour*), but
it may not carry it alone.

Size is part of whether a mark exists. Phosphor's `dot` is three pixels of ink
at the 12px a chip is drawn at, which is a smudge, not a state — a glyph that
is correct by name and absent on screen. The unsaved-changes mark on an editor
tab is a ring for that reason, where every other editor draws a dot.

Two marks exist only to keep a row from growing: `ICON_MORE`, the three dots
holding the actions that are rare or irreversible, and `ICON_TIME`, the clock on
the metadata line. Anything behind the dots is there because putting it on the
row would cost every row the width — never because it was hard to place.

Two more are a matched pair rather than a mark each: `ICON_REQUEST` and
`ICON_REPLY`, the figures in front of the two lines a session row quotes from
its terminal. They are the one place a mark says *who is speaking* rather than
what a thing is or what a button does, and they have to be a pair because
nothing in the words themselves separates the person's line from the agent's —
both are prose, in the same language, usually about the same thing. The agent's
vendor mark is already on the row above and answers a different question: which
CLI this is, not which of the two said this. Neither line is a status, so
neither figure is enclosed.

A file gets one of three faces — plain text, Markdown, or source — and no more.
Markdown earns its own because it is what agents write and the one format the
editor shows two ways; past that, a per-extension icon set is a vocabulary
nobody can finish and nobody asked for. A diff adds two marks for the files a
change creates and deletes, and none for the ones it edits: a modified file is
still just a file, and a fourth mark would say less than the `+12 −3` already
beside the name. A rename is said in words — `old → new` — because no glyph
carries which name became which.

Every icon-only control keeps the wording it replaced as hover text, and
irreversible actions keep their words outright. An icon that nobody can decode
is worse than the long button it saved space over.

A checklist item in rendered Markdown is the one place a mark stands in for
text the author actually typed: `ICON_TASK_TODO` and `ICON_TASK_DONE` replace
the `[ ]` and `[x]` an agent wrote. They earn that because a checklist is the
shape a plan arrives in, and a page of literal brackets is exactly the raw
syntax the rendered view exists to stop being read. Both are squares, not
circles — a circle is this app's enclosure for a status, and a step somebody
still has to do is not a status. The tick carries `success` and the empty box
`text_faint`, while the words beside either stay body ink: a finished plan drawn
in grey is a finished plan nobody can read, and the box already says which is
which.

Three marks are deliberately *not* icons: the bullet on a rendered Markdown
list, the arrow in `old → new` on a renamed file, and the minus in a `−3`
count. Each is punctuation inside a phrase rather than a pictogram standing in
for one, so each answers to the opposite rule — it comes from the text face, at
the weight of the words beside it, and never from the icon font. A test holds
that direction the same way one holds the icons'.

## Layout

A page is the window, inset by a **26px** gutter each side — it grows and
shrinks with the window rather than sitting at a fixed width between two
margins. A window somebody dragged wider was dragged wider to hold more, and a
fixed column answers that by growing its own margins, which is the app
declining to use the screen it was handed. The gutter shrinks below 4% of a
narrow window, because 26px off each side of a 400px window is a page that is
mostly margin.

Nothing on these pages is prose read line after line, which is what a measure
cap protects: they are cards, rows, and counts, and each of those has somewhere
to put the extra width — a session row spends it on the gap between what the
session is and what you can do to it, and a row of counts spends it on four
tiles that stop being cramped. The longest strings the app shows are single
sentences that stop wrapping altogether at these widths.

Two screens skip the gutter and take the window edge to edge, because what is
in them is as wide as it is and neither can be re-wrapped without being
damaged. The **session workspace** holds a live terminal, whose lines tmux
already wrapped to the width it was told the pane has. The **editor** holds a
project tree beside a page of code or a diff, where the column a change sits in
is part of reading it.

Both also scroll themselves rather than sitting inside the page's scroll area —
the terminal sticks to the bottom of its own buffer, and the editor scrolls its
tree and its text separately — so the page hands them a bounded height and
stays out of the way.

Spacing is a 4px scale — 4 / 8 / 12 / 18 / 26 — with a 10px gutter between
items in a row. Cards carry generous internal padding relative to the space
between them, so a card reads as one object rather than as four stacked lines.

**A card fills the column it is in.** A card that hugs its contents gives a list
a ragged right edge, and a list with a ragged right edge is a pile. The same
rule puts the status of a session in a fixed **104px** column: the word after
the glyph, and the title after the word, start at the same x on every row,
because a list is read down its left edge and a left edge that moves with the
length of a word is not one.

Every page opens with the same header — what the page is on the left, what you
can do to it on the right, on one baseline, with a hairline under it. No page
invents where its title goes or how far below it the content starts.

## Controls

A control at rest is a **hairline**, not a slab. The fill is reserved for the
one under the pointer, and a second, stronger fill for the one being pressed.
Twenty filled grey rectangles on one page read as twenty unrelated objects; the
only one that needs marking is the one a person is about to click.

Above that sit exactly three louder states, each used once per screen:

- **The filled accent button.** The one action that starts work — launch a
  session, add the first project. There is never a second one on a page: two
  filled buttons is two things claiming to be the answer.
- **The tinted selection.** `row-selected` behind `accent-text`, for the current
  page in the toolbar, the chosen agent, and the selected session row. A
  selected sidebar row adds a 2px accent bar on its left edge, because hover and
  selection are both tints and one of them has to be more than a tint.
- **The accent rule under a tab.** In-page tab bars mark the current tab with a
  2px rule along the bottom of the word, landing on the hairline under the row.
  A filled chip there would be a third selection language and would read as a
  button that is stuck down.

Everything else — icon buttons in a toolbar, the actions on a list row, the
"other options" beside a heading — is frameless until hovered.

Every clickable thing is **28px** tall, or 24px inside a list row. A row of
controls that each pick their own height is the loudest sign that an interface
was assembled rather than drawn.

## Elevation & Depth

There are no shadows in the content area. Depth is **tonal**: `window` behind
`panel` behind `raised`, each a few steps apart on the neutral ramp, with a
`border-subtle` hairline where two surfaces of similar value meet.

The toolbar is also the window's title bar. macOS keeps the traffic lights and
the drag region where a person expects them, and the app stops spending a strip
of chrome on repeating the name that is already in the Dock, the menu bar, and
the window. The first 78px of the toolbar belong to the three window buttons;
everything else starts after them.

This is why the surface ramp has five steps for what looks like a flat design.
A shadow would let two surfaces share a value; without one, they cannot.

Three things do lift off the page, all because they are modal in spirit: the
`⌘K` command palette, which keeps egui's window shadow; the drop overlay, which
covers the window in `scrim` while a folder is being dragged onto it; and the
restore window, which takes the same `scrim` and shadow while a conversation's
full history is being copied into another CLI.

The restore window is the exception that proves the rule about progress. Every
other background job in this app reports into the notice banner and is over
before anybody looks for it. A full-history restore is not: it reads a whole
conversation off disk and writes it back out in a second CLI's format, and a
banner asking somebody to wait is a line they scroll past. So the wait gets the
window, and the banner keeps what it is for — the result. Dismissing the window
closes the window; the work is a background thread either way, which is why the
button says "バックグラウンドで続ける" and not "キャンセル".

The terminal goes the other way and sinks. On the dark themes it is darker than
the panel around it; on the light theme, where there is no room below paper, it
stays near-white and is drawn with a `terminal-border` edge instead. Either the
fill or the border has to distinguish it, and the theme picks whichever it can
afford.

## Shapes

Three radii and no more: **6px** on controls, **10px** on cards, **12px** on
windows. One value per kind of object, never per state — a corner that grows on
hover is a shape that moves under the pointer, and the fill already said the
control was live.

Six is the smallest radius that reads as *chosen*. The 2px this started at is
the radius of a rectangle that was meant to be square and missed: too round to
be a corner and too square to be a curve, which is what it looked like at every
size the app draws a button.

Vendor marks are the one place with a shape that is not ours, so they are the
one place that borrows a rounding it did not choose. Each is the icon macOS
shows for that vendor's app, cropped to the rounded square itself and drawn in
a 128px box — the same squircle three times, at one optical weight, in the
order a person already knows from their Dock. Take the app icon and not the
bare logotype a vendor also ships: three loose logotypes are three silhouettes,
and the odd one out is what gets noticed instead of the label beside it.

## Do's and Don'ts

**Do** pick a role by what the colour means. If nothing on the table means what
you need, the answer is a new role, not a literal.

**Do** put a word next to every colour. Status is a glyph, a label, and a
colour together; someone who cannot separate the amber from the green still has
"応答待ち" to read.

**Do** let the light theme disagree with the dark one. A role that is the same
value in both is a coincidence, not a goal — only `border-strong` and the
neutral extremes end up there honestly.

**Do** check the contrast test after touching a value. It is faster than the
app and it covers the pairs you would not have thought to look at.

**Don't** write a colour literal in drawing code. The file has one place where
literals are allowed — the three role tables — and one honest exception: the
216-colour cube and 24-bit truecolor, which are absolute values a CLI asked for
by number.

**Don't** use `weak` for anything a person needs. It is egui's dimming, not a
role, and it lands below AA.

**Don't** carry a colour through the state machine. Compute a tone; let the
palette in force decide what it looks like.

**Don't** paint a verb in a vendor's colour. `agent-claude` names Claude; it
does not name "launch". The button that starts a session is `accent` whichever
CLI is selected, and the vendor mark stays a mark.

**Don't** put plumbing on the face of a row. A tmux name, a working directory,
and a resume command are how this app talks to a CLI, not what a person came to
read. They live one disclosure down, monospaced, next to the button that copies
them.

**Don't** cache a laid-out colour without a way to throw it away. Terminal
output is coloured once, when tmux hands it over — a theme switch has to clear
that cache or every open terminal keeps the colours of the theme it was
captured under.

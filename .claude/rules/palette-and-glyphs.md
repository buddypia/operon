---
paths:
  - "src/theme.rs"
  - "src/glyphs.rs"
  - "src/app.rs"
  - "src/app/**/*.rs"
  - "src/ui/**/*.rs"
  - "DESIGN.md"
---

# Before writing any colour

`DESIGN.md` is the design system, in the
[design.md](https://github.com/google-labs-code/design.md) format. Drawing code
never names a colour: it reads a semantic role off `Palette`
(`palette.warning`, `palette.text_muted`, `palette.terminal_bg`), and the three
`*_PALETTE` constants decide what that role looks like per theme. A new colour
means a new role on all three tables plus its entry in `DESIGN.md` — not a
literal at the call site.

Three tests hold that together and will fail loudly if you skip a step:
`design_md_documents_exactly_what_the_app_paints` compares the document to the
constants in both directions, `every_theme_stays_readable` checks every ink
against every surface at WCAG 2.1 AA, and `ansi_numbers_map_to_the_colours_they_name`
keeps `SGR 31` meaning red. The only literals left in drawing code are
`Color32::TRANSPARENT`, the 216-colour cube, and 24-bit truecolor.

# Before writing any icon

Same rule as colour, one level down: drawing code never writes a glyph, it names
an `ICON_*` constant. Those constants are the only place a Phosphor name
appears, they all live in `src/glyphs.rs`, and each one is listed in
`ICON_VOCABULARY` so
`every_icon_resolves_from_the_bundled_icon_font` can check it is a real glyph
that the text face ahead of it does not shadow. A new icon means a new constant,
its `ICON_VOCABULARY` entry, and — if it changes how a kind of thing is marked —
its line in `DESIGN.md`. Never a literal at the call site, and never a Unicode
symbol picked because it looked close enough.

---

This rule loads because you opened a file that paints. It is the reminder; the
three tests above are the enforcement, and they run whether or not this text was
in context. See `docs/sdlc/README.md` for why the split exists.

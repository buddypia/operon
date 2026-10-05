# Third-party notices

Operon's own source code is released under the [MIT License](LICENSE). The
items below are bundled in this repository, or linked into the binary, under
their own terms. The MIT License does **not** apply to them.

## Bundled assets

| Path | What | License |
| --- | --- | --- |
| `assets/fonts/Sarasa*.ttf` | [Sarasa Gothic](https://github.com/be5invis/Sarasa-Gothic) | SIL Open Font License 1.1 — [`assets/fonts/LICENSE-SARASA.txt`](assets/fonts/LICENSE-SARASA.txt) |
| Phosphor glyph outlines (via the `egui-phosphor` crate) | [Phosphor Icons](https://phosphoricons.com) | MIT — [`assets/fonts/LICENSE-PHOSPHOR.txt`](assets/fonts/LICENSE-PHOSPHOR.txt) |
| `assets/agent-icons/*.png` | Application icons of Claude Code, Codex CLI, and Antigravity CLI | **Not covered by the MIT License.** Trademarks and artwork of their respective owners (Anthropic, OpenAI, Google), shown only to identify the tool a session launches. See [`assets/agent-icons/SOURCES.md`](assets/agent-icons/SOURCES.md). |

Operon is an independent project. It is not affiliated with, endorsed by, or
sponsored by Anthropic, OpenAI, or Google. "Claude", "Codex", "ChatGPT", and
"Antigravity" are trademarks of their respective owners.

## Rust dependencies

Every crate in `Cargo.lock` is licensed under MIT, Apache-2.0, BSD-2/3-Clause,
ISC, Zlib, Unicode-3.0, BSL-1.0, CC0-1.0, Unlicense, 0BSD, or MPL-2.0 (or an
`OR` combination of these). None is copyleft in a way that constrains
distributing Operon under the MIT License. To list them:

```sh
cargo metadata --format-version 1 --locked \
  | python3 -c "import json,sys; [print(p['name'], p['version'], p['license']) for p in json.load(sys.stdin)['packages']]"
```

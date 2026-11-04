# Dark sheet: design

> **Last updated:** 2026-11-04 · **Change:** the dark sheet comes with Display theme Dark; the separate setting is gone (H3) · **Target:** a setting Excel does not have

A dark sheet shows the cells, not only the interface around them, light-on-dark. It is asked
for in #1264 (and its duplicate #1736); WordCraft has the same feature for pages
(storytold/wordcraft#194). This note fixes the behaviour and the colour rules so the code can
be written in small, separate changes. The feature is built (DS2 to DS14); the open work is the visual QA (DS8) and the upstream PRs (DS16). Built: the sheet colour tokens (DS2), the switch (DS15, then folded into Display theme by H3), the grid itself (DS3: background, gridlines, merged-cell backdrop, selection, fill handle, drag outlines, frozen-pane lines) and the cells (DS4: `theme::ink` / `ink_guarded`; font colours of every kind, fills, pattern fills, borders, number-format colours, conditional-format fonts, fills, colour scales and data bars, table and cell styles, the in-cell editor). Not yet: charts, shapes, comments, sparklines and GridCraft's own overlays (DS5, DS10), dialog previews (DS14).

## What the user gets

- **Display theme Dark shows the dark sheet.** Choosing *Dark* in View › Display theme or
  File › Options › General, the title-bar Dark Mode switch (`view.darkMode`), or *System* on a
  dark desktop darkens the interface (ribbon, headers, formula bar, sheet tabs, scroll bars,
  status bar, dialogs) and the cells together; *Light* shows both light. There is no separate
  setting: the user decided (tasklist H3, 04.11.2026) that one switch does both. An earlier
  build had an opt-in `view.darkSheet` setting; it is gone, and a `darkSheet` key left in an
  old `ui.json` is ignored.
- **Light by default**, as Display theme is. The theme is a preference kept in `UiState`
  (`ui.json` on the desktop, `localStorage` on the web). Switching it never changes the workbook:
  no dirty flag, no undo step, the file stays byte-identical.
- **Consequence for the default look:** anyone who already uses Display theme Dark, or System on
  a dark desktop, sees dark cells after updating. Upstream maintainers hold changing the default
  look as their decision (PR #177), so the upstream PR states it plainly (DS16).
- **Screen only.** Save, Print and Print Preview, PDF/PNG export, Copy as Picture, the HTML/RTF
  clipboard and Save as Web Page use the authored colours, exactly as with the setting off.

## Colour rule

One rule for every colour the workbook supplies, as in WordCraft: **invert the HSL lightness,
keep hue and saturation, then lift the blacks to the sheet grey.** Pictures are untouched.

```text
invert(r,g,b): d = 255 - max(r,g,b) - min(r,g,b); each channel v → clamp(v + d, 0, 255)
lift(v)      : P + v * (255 - P) / 255           with P = 0x33 (WordCraft's DARK_PAPER)
ink(c)       = lift(invert(c))
```

So white becomes `#333333` (the sheet), black becomes white, `#E1E1E1` gridlines become
`#4B4B4B`, dark blue headings (`#1F4E79`) become light blue (`#9EC3E6`), pale fills
(`#DDEBF7`) become deep fills (`#39444E`). Pure hues (`#FF0000`, `#0000FF`, `#FFFF00`) keep
their lightness and so barely change.

**Contrast guard.** Inversion keeps most pairs legible, but not pure hues: blue text
(`#0000FF`, 8.6:1 on white) becomes `#3333FF` on `#333333`, 1.8:1; automatic text on a
yellow fill becomes white on yellow, 1.1:1. So after mapping, text and border ink is checked
against the colour actually behind it (the mapped fill, or the sheet):

- target = min(4.5, contrast of the authored pair), using `theme::contrast` (WCAG 2);
- if the mapped pair is below the target, move the ink's HSL lightness away from the
  background's (lighter on a dark background, darker on a light one), keeping hue, in small
  steps until the target is met or lightness reaches 0 or 1.

The guard never makes a pair less legible than the author made it, and never touches fills.
Results are cached per (ink, background) pair, so large sheets pay nothing per frame.

### What "every colour the workbook supplies" covers

| Colour | On a dark sheet |
| --- | --- |
| Sheet background (no fill) | `#333333` = ink(white) |
| Gridlines | ink(`#E1E1E1`) = `#4B4B4B`: 1.45:1 on the sheet, as subtle as `#E1E1E1` on white (1.31:1) |
| Automatic font colour | ink(black) = white, then the guard against the cell's mapped fill |
| Explicit font colours: RGB, indexed, theme with tint | ink(), then the guard |
| Theme colours Text 1 / Background 1 (dk1 / lt1), very common in Excel files | ink() like any other colour; with the default theme they end up exactly as Automatic does |
| Hyperlink and Followed Hyperlink (theme slots 10, 11) | ink(): `#0563C1` becomes `#64AFFB` (5.5:1) |
| Number-format colours (`[Red]`, `[Blue]`, `[Color n]`) | ink(), then the guard (`[Blue]` is lifted to 4.5:1) |
| Cell fills, solid and pattern (both colours) | ink(), no guard |
| Borders, diagonal borders; automatic border colour | ink() (automatic = black, so white); guard against the cell's fill at 3:1 |
| Conditional formats: fonts, fills, colour scales, data bars | ink(); fonts then the guard |
| Table styles and cell styles (resolved to colours first) | ink() |
| Charts: chart area, plot area, axes, gridlines, text, series | ink(); picture fills untouched |
| Shapes, text boxes, equations, comments' and notes' text and fill, Draw ink | ink() |
| Sparklines | ink() |
| Pictures (cell pictures, floating pictures, picture fills, icon-set glyphs) | as authored |
| Sheet tab colours | as authored: the tabs are interface |

### Colours GridCraft adds to the sheet

These are not the workbook's colours: they get a second set of fixed values for the dark sheet,
each held to WCAG 2 against `#333333` (4.5:1 text, 3:1 glyphs) by `theme::contrast_tests`.

| Element | Light sheet (today) | Dark sheet |
| --- | --- | --- |
| Selection outline, fill handle, active cell | `#107C41` | `#5CC587` (5.9:1); halo in the sheet grey, not white |
| Selection tint | `#107C41` at 0x1C | `#5CC587` at 0x24 |
| Active-row tint | `#107C41` at 0x2C | `#6FD39A` at 0x30 |
| Formula reference outlines and in-cell editor references | `SHEET_REF_COLORS` | the dark `ref_colors` set (4.8 to 7.0:1) |
| Filter, validation and paste-options buttons, object handles | bg `#F3F3F3`, border `#8A8A8A`, text `#1F1F1F` | bg `#3C3C3C`, border `#9A9A9A`, text `#EEEEEE` |
| Picture placeholder | `#D6D6D6` | `#4A4A4A` |
| Trace arrows (precedents, errors) | `#1F5FC9`, `#D13438` | `#6FA3F5`, `#F1707A` |
| Macro overlays (input, footprint, conflict, target) | `#2E6FD8`, `#23803D`, `#C42B1C`, `#6B3FC0` | `#6FA3F5`, `#6CCB85`, `#F1705F`, `#B394EE` |
| PY tag and its text | `#1E6E4E`, white | `#5CBF94`, `#0B1F17` |
| Error-check triangle | `#1E8E3E` | `#4CC26B` |
| In-cell editor: text, caret, background | black on white | white on the cell's mapped fill or the sheet |
| Frozen-pane lines | `#949494` | `#8A8A8A` (3.7:1) |
| Fill-handle ring | white | the sheet grey |
| Fill-drag preview and move-drag outline | `#646464` | `#C8C8C8` (5.8:1) |
| Automatic page breaks, copy marching ants (use the selection border), find highlight, spill outline | today's values | dark values chosen with the same contrast test |

Row and column headers, the select-all corner and their selection highlight are interface: they
follow Display theme, not the dark sheet.

### Views

- **Normal view** and **Page Layout view** are editing views: the dark sheet applies, including
  the Page Layout paper (like WordCraft's page). The desk around the pages follows Display theme.
- **Page Break Preview**: the dark sheet applies; its watermark, break lines and the area
  outside the print area get dark values with readable contrast.
- **Print Preview** (File › Print) shows what will print: always paper-white.
- **Dialog previews** (Format Cells sample, border preview, cell, table and conditional-format
  galleries, the font colour picker's Automatic swatch) show what the cell will look like on
  the current sheet, so they follow the dark sheet.

## Command

No new command. The existing theme commands switch the sheet with the interface:
`view.theme {mode: "system" | "light" | "dark"}` and the older `view.darkMode {on?: bool}`
(the title-bar switch; the id WordCraft uses for its dark page). Both are reachable over the
control channel, CLI and MCP and never open a dialog. The offscreen `snapshot` example's
`--dark` flag renders the dark interface with the dark sheet.

## How it is built

- **Tokens on two axes.** `theme::Tokens` keeps its field names (no rename churn). The sheet
  fields (`grid_bg`, `gridline`, `cell_text`, `sel_border`, `sel_fill`, `active_row`,
  `sheet_ref_colors`, `sheet_control_*`, `sheet_track`, `sheet_macro_*`, `sheet_py_tag`,
  `on_sheet_py_tag`, `sheet_trace_*`, plus a sheet error-check green) take their values from a
  `SheetColors` set, `LIGHT` or `DARK`: `Tokens::palette(dark_ui, dark_sheet)`. `Tokens::get`
  passes the Display theme for both axes, so every painter that already calls `Tokens::get`
  follows without new parameters. The axes stay apart for Print Preview, which shows the light
  (printed) sheet inside a dark interface (`Tokens::with_dark_sheet(false)`). `contrast_tests`
  runs all four combinations.
- **The mapping lives in the UI crate**, in `theme` (`ink`, `ink_guarded`), next to `contrast`.
  It reuses the HSL helpers of `crates/model/src/style.rs` (`rgb_to_hsl`, `hsl_to_rgb`, made
  public) instead of copying them. Nothing below the UI crate changes: the model, the XLSX
  writer, printing and export never see a mapped colour. `col32` and `numfmt_color` in
  `grid/cell_paint.rs` become the single entry for cell colours and take the sheet colours.
- **Icons on the sheet.** `icons::paint_on_sheet` takes its palette from the current sheet
  colours instead of `Tokens::light()`.
- **Hard-coded sheet colours go.** `Color32::WHITE` / `BLACK` in `grid/quadrant.rs`,
  `grid/selection.rs`, `grid/editor.rs`, `grid/cell_paint.rs`, `grid/interact.rs`,
  `text_box.rs`, `equation.rs` and the `Tokens::light()` trace arrows in `grid/show.rs` read the
  sheet colours instead.
- **Guarantees as tests:** the mapping (white, black, hues, the guard), every sheet pair in all
  four token combinations, light by default, Display theme Dark / `view.darkMode` / System
  darkening the sheet with the interface (`tests/it/dark_sheet_setting.rs`, `system_theme.rs`),
  the theme round trip, no dirty flag or undo step, export and print output byte-identical with the setting on
  and off, and offscreen snapshots of the sales sample in light and dark.

## Wording to replace "the sheet stays white"

Applied in DS9: `theme.rs` (module doc), `icons.rs` (`paint_on_sheet`, now on the sheet palette) and
`docs/architecture.md` (*UI building blocks*) changed from "the sheet stays white in both themes"
to (rewritten after H3): *colours drawn on the sheet come from a `SheetColors` set; Display
theme Dark shows the dark sheet too (`docs/dark-sheet.md`); both sets are held to WCAG 2 by
`contrast_tests`.*

## Earlier proposals

- **#1736** (a `view.darkCells` patch against v0.7.0): black canvas, `#3F3F3F` gridlines,
  automatic ink flips, explicit colours and fills kept as authored, automatic ink picked by the
  fill's brightness. Its separation from the theme, its saved setting and its tests are kept
  here. Not kept: keeping explicit colours as authored leaves explicit black text black on the
  dark canvas and white fills glaring, and Excel files often store Text 1 / Background 1
  explicitly, so much of a typical workbook would not change.
- **PR #177** darkens the canvas as part of the dark interface theme and recolours workbook
  colours with the same HSL lightness mirror. The maintainers hold changing the default look
  for everyone as their decision and asked for the PR to be split. This design first kept the
  dark sheet opt-in and separate from Display theme for that reason; the user then decided (H3)
  that Display theme Dark darkens the sheet too, as #177 does, so the PR must say that dark-theme
  users get dark cells and leave the call to the maintainers.
- **WordCraft #194** is the sibling precedent for the colour rule: the same inversion and lift,
  screen only, remembered between runs. Unlike WordCraft, GridCraft has no separate switch: the
  dark page comes with the dark theme. Its paper grey (`#333333`) is used
  unchanged so the two apps look alike side by side.

## Revision history

| Date | Change | Notes |
| --- | --- | --- |
| 2026-11-04 | new | Design: opt-in dark sheet, WordCraft colour rule with a contrast guard, token layout, command id open |
| 2026-11-04 | DS2 | `theme::SheetColors` (LIGHT/DARK), `Tokens::palette(dark_ui, dark_sheet)`, `theme::set_dark_sheet` / `dark_sheet` (context flag read by `Tokens::get`), `Tokens::with_dark_sheet`; `contrast_tests` runs all four combinations |
| 2026-11-04 | DS3 | `SheetColors` gains `freeze_line`, `handle_halo`, `drag_outline` (tokens `sheet_*`); the merged-cell backdrop, fill handle, drag outlines and frozen lines read tokens; split cursor follows Display theme; `snapshot --dark-sheet`; pixel test of background and gridlines |
| 2026-11-04 | DS9 | Docs: ui-parity.md, ROADMAP.md, architecture.md updated; `icons::paint_on_sheet` takes the sheet palette (was fixed light); "sheet stays white" wording replaced |
| 2026-11-04 | DS4 | `theme::ink`, `theme::ink_guarded` (cached per pair, `INK_TEXT_MIN` 4.5, `INK_LINE_MIN` 3); `grid/cell_paint.rs` `look(…, dark)` maps fills and text and keeps the authored `paper` for the guard; borders, `[Red]`-style colours and data bars mapped in `grid/quadrant.rs`; the in-cell editor paints the mapped fill with guarded automatic text; `model::style::rgb_to_hsl` / `hsl_to_rgb` public |
| 2026-11-04 | H3 | The user decided one switch for both: `Tokens::get` takes the sheet from Display theme; `view.darkSheet`, `UiState.dark_sheet`, the View tab and Options checkboxes, `theme::set_dark_sheet` / `dark_sheet` and `snapshot --dark-sheet` removed; `Tokens::dark()` is dark/dark |

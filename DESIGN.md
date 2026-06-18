# Eterea Design System

`design.html` at the repository root is the canonical visual and interaction reference for the current redesign. This document is the implementation contract for translating that reference into the Rust/Dioxus desktop app without copying its bundled React runtime.

## 1. Product and UX direction

Eterea is a local-first bookmark/archive desktop app. The redesigned UI must feel like a keyboard-native terminal library: dense, fast, legible, data-backed, and calm. The previous editorial paper reading-room aesthetic is superseded for this redesign.

Core qualities:

- **Terminal shell, not SaaS cards:** top tab bar, full-height content viewport, bottom mode/status line, monospaced rhythm, square panels, thin rules.
- **Keyboard-first:** one global keyboard owner with explicit Normal, Insert, Visual, Command, and Search modes.
- **Data truthful:** every count, row, tag, author, favorite marker, and date comes from current app/backend services or is explicitly marked unavailable. Do not ship mock-only fields as real product data.
- **Local-first safety:** import/search/favorite/settings flows use local/temp fixtures for testing and must not require production data, credentials, or destructive cleanup.
- **Reference parity:** screen shape, tokens, density, statusline, tabs, overlays, and route affordances match `design.html` before extra product polish is added.

## 2. Source-of-truth inputs

| Input | Role |
| --- | --- |
| `design.html` | Canonical visual/UX reference. |
| `.omx/plans/prometheus-strict/reference-extract/*.jsx` | Clean-room extraction of reference components, tokens, and mock data names for implementation mapping only. |
| `.omx/plans/prometheus-strict/design-html-dioxus-mapping.md` | Route/surface mapping and delete/map/preserve decisions. |
| `.omx/plans/prometheus-strict/baseline/*.png` | Local reference screenshots captured from `design.html`. |
| `src/dioxus-app/src/**` | Rust/Dioxus desktop implementation target. |
| `src/app/src/services/app.rs` and `src/app/src/services/app/**` | Product service boundary; UI must use this boundary for persisted data. |
| `src/backend/**` | Storage/import/search implementation; change only when guardrail tests prove a service need. |

## 3. Design tokens

### Themes

Default theme is **Catppuccin Mocha**. Macchiato and Latte remain supported appearance variants after the shell is implemented.

| Token | Mocha default | Macchiato | Latte | Usage |
| --- | --- | --- | --- | --- |
| `bg` | `#1e1e2e` | `#24273a` | `#eff1f5` | App viewport/content background. |
| `mantle` | `#181825` | `#1e2030` | `#e6e9ef` | Topbar/statusline/panel body. |
| `crust` | `#11111b` | `#181926` | `#dce0e8` | Brand block, dark overlay base, strong contrast text background. |
| `surface0` | `#313244` | `#363a4f` | `#ccd0da` | Selected row, borders, subtle panel fill. |
| `surface1` | `#45475a` | `#494d64` | `#bcc0cc` | Elevated borders and overlay panels. |
| `surface2` | `#585b70` | `#5b6078` | `#acb0be` | Stronger dividers. |
| `text` | `#cdd6f4` | `#cad3f5` | `#4c4f69` | Primary copy. |
| `subtext` | `#bac2de` | `#b8c0e0` | `#5c5f77` | Secondary labels. |
| `overlay0` | `#6c7086` | `#6e738d` | `#9ca0b0` | Disabled/idx/chrome hints. |
| `overlay1` | `#7f849c` | `#8087a2` | `#8c8fa1` | Muted status metadata. |
| `accent` | `#f5c2e7` | `#f5bde6` | `#ea76cb` | Active tab/row rail/tags/default accent. |
| `blue` | `#89b4fa` | `#8aadf4` | `#1e66f5` | Normal mode. |
| `green` | `#a6e3a1` | `#a6da95` | `#40a02b` | Authors/success/Insert mode. |
| `red` | `#f38ba8` | `#ed8796` | `#d20f39` | Errors/destructive confirmation. |
| `yellow` | `#f9e2af` | `#eed49f` | `#df8e1d` | Favorites/Search mode/warnings. |
| `peach` | `#fab387` | `#f5a97f` | `#fe640b` | Visual mode/multi-select/tree group. |
| `mauve` | `#cba6f7` | `#c6a0f6` | `#8839ef` | Command mode/graph tags. |
| `sky` | `#89dceb` | `#91d7e3` | `#04a5e5` | Informational accents. |
| `teal` | `#94e2d5` | `#8bd5ca` | `#179299` | Secondary graph/tag accents. |

### Typography

- Primary UI font: `JetBrains Mono`, then `Iosevka`, `IBM Plex Mono`, `Berkeley Mono`, `SFMono-Regular`, `ui-monospace`, `monospace`.
- Base shell font size: `12.5px`; statusline: `11.5px`; table headers: `10.5px`; dashboard numbers: `28px`.
- Weight: regular by default; active/important labels use `600`; mode labels use `700`.
- Letter spacing: uppercase labels use `0.1em`; mode blocks and brand use `0.05em`.
- Legacy serif stacks from the paper UI are historical only; do not use them for new shell surfaces.

### Spacing and layout constants

| Constant | Value | Notes |
| --- | --- | --- |
| Base grid | `4px` | All spacing should land on 4px rhythm where possible. |
| Top tab bar height | `32px` | Includes brand block and six primary tabs. |
| Statusline height | `24px` | Bottom mode/status line, including command/search input states. |
| Topbar horizontal padding | `14px` | Brand, tabs, right help/palette hints. |
| Table grid gap | `12px` | Reference table row/heading gap. |
| Panel padding | `14px` / `16px` | 14px for cards/overlays, 16px for dashboard/graph/calendar view bodies. |
| Command palette width | `600px` | Centered top overlay, `80px` top offset. |
| Overlay radius | `4px` max | Keep terminal-like; avoid rounded SaaS cards. |
| Row border rail | `2px` | Active row rail equals current accent. |

Density variants:

| Density | Row padding | Font size | Line height |
| --- | --- | --- | --- |
| Compact | `3px 14px` | `11.5px` | `1.35` |
| Regular | `6px 14px` | `12.5px` | `1.45` |
| Comfy | `9px 14px` | `13.5px` | `1.55` |

## 4. Shell and interaction model

### Chrome

The root app renders a three-row shell:

```text
┌──────────────── top tabs, brand, help/palette hints ────────────────┐
│                         current route/view                           │
└──────────────── bottom mode/statusline/command input ───────────────┘
```

Top tabs are exactly: `[1] library`, `[2] authors`, `[3] topics`, `[4] search`, `[5] import`, `[6] settings`. Detail routes inherit their parent tab highlight (`entry:*` highlights library; `author:*` highlights authors; `topic:*` highlights topics).

Bottom statusline:

- Normal mode: mode block + left status text + right metadata (`1/12 · mocha · Ln 1 Col 1`).
- Visual mode: shows multi-select count as a second colored segment.
- Command mode: mode block + `:` + focused command input.
- Search mode: mode block + `/` + focused search input.

### Keyboard modes

| Mode | Required entry | Required behavior |
| --- | --- | --- |
| Normal | default / `Esc` | Navigation and commands: `j/k`, arrows, `1..6`, `?`, Cmd/Ctrl-P, `/`, `:`. |
| Insert | `i` or focused editable field | Text inputs own keystrokes; global shortcuts must not leak. |
| Visual | `v` | Multi-select; `Space` toggles row; `a` selects all; `A` clears; `Esc` returns Normal. |
| Command | `:` | Statusline input; `Enter` submits; `Esc` cancels. |
| Search | `/` | Statusline search input; `Enter` applies; `Esc` cancels. |

Global shortcut leakage is a blocker when command palette, search input, import fields, settings controls, or onboarding controls are focused.

### Overlays

- Command palette (`Ctrl/Cmd-P`): translucent full-window overlay, centered 600px panel, fuzzy filtered command rows, max 12 visible results, keyboard hints footer.
- Keybindings overlay (`?`): grouped terminal cheat sheet, escape/click close, visually consistent with palette.
- Flash/status messages: one-line terminal notifications using theme colors; no toast-card stack.

## 5. Screens and components

| Surface | Target behavior |
| --- | --- |
| Library table | Dense grid columns: selection marker, idx, author, when, content, tags, favorite. Active row uses `surface0` and `2px` accent rail. |
| Library tree | Author-grouped file-tree shape under `~/eterea/library/by-author/`, with nested rows and active rail. |
| Dashboard | Metric cards for totals/authors/favorites/recent period, sparklines, top tag bars, recent saves list. |
| Graph | SVG tag co-occurrence panel using existing tag/topic data where possible. |
| Calendar | Save/import heatmap and by-hour bars derived from stored timestamps where possible. |
| Detail/Entry | Main content pane plus metadata/actions; no fake likes/reach. Media remains local/privacy-safe. |
| Authors | Directory/list plus author detail route; uses `AuthorSummary` and current author detail service data. |
| Topics | Topic/tag list plus topic route; uses `TopicSummary`, top tags, and filtered query results. |
| Search | Terminal query/results surface, with search statusline state and highlighted matches where safe. |
| Import | Source → preview → importing → done flow, preserving existing preview/import guardrails. |
| Settings | Matrix-style terminal settings for theme/font/density/weight/accent plus media/storage toggles. |
| Onboarding/empty | ASCII/terminal guide for first-run import, keyboard-friendly actions, no paper cards. |

## 6. Current behavior decisions

| Existing behavior | Decision |
| --- | --- |
| Left rail navigation | Delete/replace with top tab bar and command palette. |
| Hero headline/filter block | Delete/map useful filters to statusline search, command palette, search route, and settings/import screens. |
| Paper tones (`cream`, `offwhite`, `gray`) | Superseded by Catppuccin themes. Keep old code only until migrated; do not extend it. |
| Editorial feed layouts (`Issue`, `Front Page`, `Long-Read`, `Spread`) | Map to library view modes: table, tree, dashboard, graph, calendar. |
| Bookmark cards | Map to terminal rows plus detail pane. Preserve content, author, tags, dates, favorite state, note/media metadata. |
| Expanded inline detail | Map to `entry:*` detail and/or split metadata panel. |
| Import modal | Map to import route/panel; preserve preview, importing, done, and error states. |
| Appearance settings | Map to terminal settings matrix; theme/font/density/weight/accent must remain user-visible controls. |
| Remote image toggle | Preserve privacy default: remote images disabled unless explicitly enabled. |
| App service boundary | Preserve; add service APIs only after failing typed tests show UI derivation is insufficient. |

## 7. Acceptance locks

### Visual

- Viewports: `1440x900`, `1280x800`, `1024x768`; optional mobile/narrow screenshots may document responsiveness but do not replace desktop proof.
- Baseline path: `.omx/plans/prometheus-strict/baseline/`.
- Required states: shell default library table, tree, dashboard, graph, calendar, detail/entry, authors, topics, search/results, import, settings, onboarding/empty, command palette, command/search statusline, keybindings overlay, flash/status message.
- Threshold: static chrome/tokens/layout ≤ `0.5%` differing pixels; content-heavy panels ≤ `1.0%` with documented masks only for dynamic text/date/media.
- Missing tab, statusline, mode indicator, route body, or wrong theme token blocks completion regardless of percentage.

### Backend

All implementation must preserve import, favorite, search, filter, stat, route, and persistence behavior under local/temp fixtures. No production DB, credentials, destructive cleanup, or schema/data migration without a separate explicit approval.

### Verification order

Final implementation cannot be marked complete until verification passes, changed-file cleanup/no-op is recorded, affected verification reruns after any cleanup edit, and code review is clean or explicit blockers are recorded.

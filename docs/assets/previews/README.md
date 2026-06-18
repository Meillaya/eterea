# Preview asset contract

Committed preview images for Eterea live in this directory: `docs/assets/previews/`.
They are public documentation assets and must be safe to publish, copy, and mirror.

## Privacy rules

- Use synthetic, local fixture, or maintainer-approved preview-profile data only. Do not capture private notes, secrets, production-only archives, or unreviewed personal reading history.
- Keep remote media previews off by default. Public preview images must not depend on network fetching, hotlinked media, or third-party images at render time.
- Do not include OS usernames, private filesystem paths, secrets, tokens, hostnames, local workspace names, or terminal prompts in screenshots.
- Treat `.omo/` and `.omx/` evidence, cache, and accepted-deviation artifacts as internal build data only. Do not use `.omx` accepted-deviation artifacts as final public preview proof unless the image is regenerated or deliberately copied into this directory with provenance recorded below.
- Treat all generated artifacts as untrusted until reviewed, including screenshot
  scratch files, preview SVG/PNG intermediates, logs, benchmark output, and
  model/tool-produced reports. Public preview assets must come from
  synthetic/local-fixture sources and must not embed imported bookmark fields,
  remote image URLs, or third-party hotlinks.
- If a screenshot is edited, cropped, redacted, regenerated, or copied from an evidence directory, record that operation in the provenance log before using it from README or docs pages.

## Current preview status

Most PNG files committed here are synthetic design-preview mockups generated from local SVG markup. The README preview images are maintainer-provided live Dioxus screenshots captured from a local archive preview profile and recorded in the provenance log below. Release-readiness evidence should still distinguish README previews from automated visual QA proof.

## Provenance log

Record every committed preview asset here when it is added or replaced.

| Asset | Source | Data | Capture or generation command | UTC timestamp | Reviewer notes |
| --- | --- | --- | --- | --- | --- |
| `library-preview.png` | User-provided desktop screenshot copied from Codex clipboard attachment | Maintainer-provided local archive preview data | `cp /tmp/nix-shell.6Flmpt/codex-clipboard-NpaMGW.png docs/assets/previews/library-preview.png` | 2026-06-18T15:14:00Z | Live Dioxus README preview requested by maintainer; reviewed for obvious secrets/paths before commit. |
| `dashboard-preview.png` | User-provided desktop screenshot copied from Codex clipboard attachment | Maintainer-provided local archive preview data | `cp /tmp/nix-shell.6Flmpt/codex-clipboard-dEg3MB.png docs/assets/previews/dashboard-preview.png` | 2026-06-18T15:14:00Z | Live Dioxus README preview requested by maintainer; reviewed for obvious secrets/paths before commit. |
| `library-issue.png` | Local SVG mockup converted with `rsvg-convert` | Synthetic fictional bookmarks/topics only | `rsvg-convert --format=png --width=1440 --height=900 t8-repair-svg/library-issue.svg --output docs/assets/previews/library-issue.png` | 2026-06-17T17:06:12Z | Repair regenerated to avoid text overflow/overlap; design preview mockup, not a live screenshot. |
| `search.png` | Local SVG mockup converted with `rsvg-convert` | Synthetic fictional search results only | `rsvg-convert --format=png --width=1440 --height=900 t8-svg-tmp/search.svg --output docs/assets/previews/search.png` | 2026-06-17T16:46:25Z | Design preview mockup; not a live screenshot. |
| `import-preview.png` | Local SVG mockup converted with `rsvg-convert` | Synthetic generated archive metadata only | `rsvg-convert --format=png --width=1440 --height=900 t8-repair-svg/import-preview.svg --output docs/assets/previews/import-preview.png` | 2026-06-17T17:06:58Z | Repair regenerated to keep list text clear of divider and controls; design preview mockup, not a live screenshot. |
| `settings.png` | Local SVG mockup converted with `rsvg-convert` | Synthetic local-first settings labels only | `rsvg-convert --format=png --width=1440 --height=900 t8-svg-tmp/settings.svg --output docs/assets/previews/settings.png` | 2026-06-17T16:46:25Z | Design preview mockup; not a live screenshot. |
| `eterea-logo.svg` | Local SVG logo markup | No user data | `cat > docs/assets/previews/eterea-logo.svg` | 2026-06-17T16:46:25Z | Synthetic sticker/logo asset; safe for docs badges or gallery decoration. |

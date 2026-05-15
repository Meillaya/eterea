// c-screens.jsx — Detail (full-screen reader), Author, Tag, Search, Import, Settings, Onboarding.

function CDetailScreen({ theme, accent, items, selectedIdx, setSelectedIdx }) {
  const b = items[selectedIdx] || items[0];
  if (!b) return null;
  return (
    <div style={{ display: "grid", gridTemplateColumns: "1fr 320px", height: "100%", overflow: "hidden" }}>
      <main style={{ overflow: "auto", padding: "32px 40px 40px" }}>
        <div style={{ color: theme.overlay0, fontSize: 11, marginBottom: 12 }}>
          ── entry {String(selectedIdx + 1).padStart(3, "0")} of {items.length} ──
          <span style={{ marginLeft: 16, color: theme.overlay1 }}>
            <CKbd theme={theme}>j</CKbd>/<CKbd theme={theme}>k</CKbd> next/prev · <CKbd theme={theme}>Esc</CKbd> back
          </span>
        </div>

        <div style={{ marginBottom: 6 }}>
          <span style={{ color: theme.green, fontSize: 16, fontWeight: 600 }}>@{b.handle}</span>
          <span style={{ color: theme.overlay1, marginLeft: 10 }}>({b.name})</span>
        </div>
        <div style={{ color: theme.overlay0, fontSize: 11, marginBottom: 24 }}>
          tweeted {fmtDate(b.tweeted_at)} · saved {fmtDate(b.saved_at)} · {b.likes.toLocaleString()} likes
        </div>

        <pre style={{
          margin: 0,
          fontFamily: "inherit",
          fontSize: 16,
          lineHeight: 1.7,
          color: theme.text,
          whiteSpace: "pre-wrap",
          padding: 20,
          background: theme.mantle,
          border: `1px solid ${theme.surface0}`,
          borderLeft: `3px solid ${accent}`,
        }}>{b.content}</pre>

        <div style={{ display: "flex", gap: 14, marginTop: 20 }}>
          {b.tags.map(t => <span key={t} style={{ color: accent, fontSize: 13 }}>#{t}</span>)}
        </div>

        <div style={{ marginTop: 32, color: theme.overlay0, fontSize: 11, textTransform: "uppercase", letterSpacing: "0.1em" }}>── note ──</div>
        <div style={{ marginTop: 8, padding: 16, background: theme.mantle, border: `1px solid ${theme.surface0}`, color: theme.subtext, fontStyle: "italic", lineHeight: 1.6 }}>
          {b.id === "3"
            ? "Re-read whenever I'm tempted to add an architectural primitive. The boring answer keeps winning."
            : <span style={{ color: theme.overlay0 }}>(no note · press <CKbd theme={theme}>i</CKbd> to add)</span>}
        </div>

        <div style={{ marginTop: 32, color: theme.overlay0, fontSize: 11, textTransform: "uppercase", letterSpacing: "0.1em" }}>── related ──</div>
        {items.filter(x => x.id !== b.id && x.tags.some(t => b.tags.includes(t))).slice(0, 3).map(r => (
          <div key={r.id} style={{ marginTop: 10, padding: "8px 0", borderTop: `1px dashed ${theme.surface0}` }}>
            <span style={{ color: theme.green }}>@{r.handle}</span>
            <span style={{ color: theme.overlay1, marginLeft: 8, fontSize: 11 }}>{fmtRel(r.tweeted_at)}</span>
            <div style={{ marginTop: 4, color: theme.subtext, fontSize: 12, lineHeight: 1.5 }}>
              {r.content.length > 140 ? r.content.slice(0, 140) + "…" : r.content}
            </div>
          </div>
        ))}
      </main>

      <aside style={{ borderLeft: `1px solid ${theme.surface0}`, background: theme.mantle, padding: 18, overflow: "auto" }}>
        <div style={{ color: theme.overlay0, fontSize: 11, textTransform: "uppercase", letterSpacing: "0.1em", marginBottom: 12 }}>── metadata ──</div>
        {[
          ["author", `@${b.handle}`, theme.green],
          ["name", b.name, theme.text],
          ["tweeted", fmtDate(b.tweeted_at), theme.text],
          ["saved", fmtDate(b.saved_at), theme.text],
          ["likes", b.likes.toLocaleString(), theme.yellow],
          ["replies", "143", theme.text],
          ["reposts", "892", theme.text],
          ["media", b.media ? `${b.media} attached` : "—", theme.text],
          ["fav", b.is_favorite ? "true" : "false", b.is_favorite ? theme.yellow : theme.overlay0],
          ["tags", b.tags.join(", "), accent],
        ].map(([k, v, c]) => (
          <div key={k} style={{ display: "grid", gridTemplateColumns: "70px 1fr", gap: 10, padding: "3px 0", fontSize: 12 }}>
            <span style={{ color: theme.overlay0 }}>{k}:</span>
            <span style={{ color: c }}>{v}</span>
          </div>
        ))}

        <div style={{ marginTop: 20, color: theme.overlay0, fontSize: 11, textTransform: "uppercase", letterSpacing: "0.1em", marginBottom: 8 }}>── actions ──</div>
        {[
          ["[f]", "favorite", theme.yellow],
          ["[o]", "open on x", theme.blue],
          ["[i]", "insert note", theme.green],
          ["[t]", "edit tags", theme.peach],
          ["[y]", "yank text", theme.mauve],
          ["[d]", "delete", theme.red],
        ].map(([k, l, c]) => (
          <div key={k} style={{ display: "grid", gridTemplateColumns: "40px 1fr", gap: 10, padding: "3px 0", fontSize: 12, cursor: "pointer" }}>
            <span style={{ color: c }}>{k}</span>
            <span style={{ color: theme.subtext }}>{l}</span>
          </div>
        ))}
      </aside>
    </div>
  );
}

function CAuthorScreen({ theme, accent, handle = "karpathy" }) {
  const author = BOOKMARKS.find(b => b.handle === handle) || BOOKMARKS[2];
  const items = BOOKMARKS.filter(b => b.handle === handle).concat(BOOKMARKS.slice(0, 4));
  return (
    <div style={{ overflow: "auto", height: "100%", padding: "20px 32px" }}>
      <div style={{ color: theme.overlay0, fontSize: 11, marginBottom: 16 }}>
        ── author archive · /authors/@{handle} ──
      </div>
      <div style={{ display: "grid", gridTemplateColumns: "auto 1fr auto", gap: 24, alignItems: "center", paddingBottom: 16, borderBottom: `1px solid ${theme.surface0}` }}>
        <div style={{
          width: 64, height: 64,
          background: `linear-gradient(135deg, oklch(0.55 0.14 ${author.avatar_hue}), oklch(0.42 0.10 ${author.avatar_hue + 30}))`,
          border: `1px solid ${theme.surface1}`,
        }} />
        <div>
          <div style={{ color: theme.green, fontSize: 22, fontWeight: 700 }}>@{author.handle}</div>
          <div style={{ color: theme.subtext, fontSize: 14, marginTop: 2 }}>{author.name}</div>
        </div>
        <div style={{ textAlign: "right", color: theme.overlay1, fontSize: 12 }}>
          <div><span style={{ color: theme.text, fontSize: 22, fontWeight: 700 }}>{items.length}</span> entries</div>
          <div>3 favored · 4 tags</div>
        </div>
      </div>

      <div style={{ display: "flex", gap: 14, padding: "12px 0", borderBottom: `1px solid ${theme.surface0}` }}>
        <span style={{ color: theme.overlay0, fontSize: 11 }}>most-saved tags:</span>
        {[["ai", 5], ["ml", 4], ["papers", 2]].map(([t, c]) => (
          <span key={t} style={{ color: accent, fontSize: 12 }}>#{t} <span style={{ color: theme.overlay1 }}>({c})</span></span>
        ))}
      </div>

      {items.map((b, i) => (
        <div key={`${b.id}-${i}`} style={{ display: "grid", gridTemplateColumns: "32px 80px minmax(0,1fr) 100px", gap: 12, padding: "10px 0", borderBottom: `1px dashed ${theme.surface0}`, fontSize: 13 }}>
          <span style={{ color: theme.overlay0 }}>{String(i + 1).padStart(3, "0")}</span>
          <span style={{ color: theme.overlay1 }}>{fmtRel(b.tweeted_at)}</span>
          <span style={{ color: theme.text, lineHeight: 1.55 }}>{b.content}</span>
          <span style={{ color: accent, fontSize: 11, textAlign: "right" }}>{b.tags.map(t => `#${t}`).join(" ")}</span>
        </div>
      ))}
    </div>
  );
}

function CTagScreen({ theme, accent, tag = "rust" }) {
  const items = BOOKMARKS.filter(b => b.tags.includes(tag)).concat(BOOKMARKS.slice(0, 3));
  const authors = new Set(items.map(b => b.handle));
  return (
    <div style={{ overflow: "auto", height: "100%", padding: "20px 32px" }}>
      <div style={{ color: theme.overlay0, fontSize: 11, marginBottom: 16 }}>── topic · /tags/#{tag} ──</div>
      <div style={{ paddingBottom: 16, borderBottom: `1px solid ${theme.surface0}` }}>
        <div style={{ color: accent, fontSize: 36, fontWeight: 700 }}>#{tag}</div>
        <div style={{ color: theme.subtext, fontSize: 13, marginTop: 4 }}>
          {items.length} entries from {authors.size} authors
        </div>
        <div style={{ display: "flex", gap: 14, marginTop: 12 }}>
          <span style={{ color: theme.overlay0, fontSize: 11 }}>co-occurs with:</span>
          {[["performance", 8], ["tools", 6], ["systems", 4]].map(([t, c]) => (
            <span key={t} style={{ color: accent, fontSize: 12 }}>#{t} <span style={{ color: theme.overlay1 }}>({c})</span></span>
          ))}
        </div>
      </div>
      {items.map((b, i) => (
        <div key={`${b.id}-${i}`} style={{ display: "grid", gridTemplateColumns: "32px 130px 80px minmax(0,1fr)", gap: 12, padding: "10px 0", borderBottom: `1px dashed ${theme.surface0}`, fontSize: 13 }}>
          <span style={{ color: theme.overlay0 }}>{String(i + 1).padStart(3, "0")}</span>
          <span style={{ color: theme.green }}>@{b.handle}</span>
          <span style={{ color: theme.overlay1 }}>{fmtRel(b.tweeted_at)}</span>
          <span style={{ color: theme.text, lineHeight: 1.55 }}>{b.content}</span>
        </div>
      ))}
    </div>
  );
}

function CSearchScreen({ theme, accent, query, setQuery }) {
  const q = query || "rust";
  const matches = BOOKMARKS.filter(b => b.content.toLowerCase().includes(q.toLowerCase()) || b.tags.includes(q.toLowerCase()) || b.handle.toLowerCase().includes(q.toLowerCase()));
  function hl(text) {
    if (!q) return text;
    const parts = text.split(new RegExp(`(${q})`, "ig"));
    return parts.map((p, i) => p.toLowerCase() === q.toLowerCase()
      ? <mark key={i} style={{ background: accent, color: theme.crust, padding: "0 2px" }}>{p}</mark>
      : <React.Fragment key={i}>{p}</React.Fragment>);
  }
  return (
    <div style={{ height: "100%", display: "flex", flexDirection: "column", overflow: "hidden" }}>
      <div style={{ padding: "16px 32px", borderBottom: `1px solid ${theme.surface0}` }}>
        <div style={{ color: theme.overlay0, fontSize: 11, marginBottom: 8 }}>── search · /grep ──</div>
        <div style={{ display: "flex", gap: 10, alignItems: "center" }}>
          <span style={{ color: accent, fontSize: 18 }}>$</span>
          <input
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            placeholder="grep ..."
            style={{ flex: 1, background: "transparent", border: "none", outline: "none", color: theme.text, fontFamily: "inherit", fontSize: 18 }}
          />
          <span style={{ color: theme.overlay1, fontSize: 11 }}>{matches.length} matches · 0.04s</span>
        </div>
        <div style={{ marginTop: 10, display: "flex", gap: 12, color: theme.overlay1, fontSize: 11 }}>
          <span>filter:</span>
          {["all", "content", "tags", "authors", "notes"].map((f, i) => (
            <span key={f} style={{ color: i === 0 ? accent : theme.overlay1, fontWeight: i === 0 ? 700 : 400, cursor: "pointer" }}>{f}</span>
          ))}
        </div>
      </div>
      <div style={{ overflow: "auto", flex: 1, padding: "10px 32px" }}>
        {matches.map((b, i) => (
          <div key={b.id} style={{ display: "grid", gridTemplateColumns: "32px 1fr", gap: 12, padding: "10px 0", borderBottom: `1px dashed ${theme.surface0}` }}>
            <span style={{ color: theme.overlay0, fontSize: 11 }}>{String(i + 1).padStart(3, "0")}</span>
            <div>
              <div style={{ display: "flex", gap: 10, alignItems: "baseline" }}>
                <span style={{ color: theme.green, fontSize: 12 }}>@{hl(b.handle)}</span>
                <span style={{ color: theme.overlay1, fontSize: 11 }}>{fmtRel(b.tweeted_at)}</span>
                <span style={{ color: theme.overlay1, fontSize: 11, marginLeft: "auto" }}>match in: content</span>
              </div>
              <div style={{ marginTop: 4, color: theme.text, fontSize: 13.5, lineHeight: 1.55 }}>{hl(b.content)}</div>
              <div style={{ marginTop: 6, display: "flex", gap: 10 }}>
                {b.tags.map(t => <span key={t} style={{ color: t === q.toLowerCase() ? accent : theme.overlay1, fontSize: 11, fontWeight: t === q.toLowerCase() ? 700 : 400 }}>#{t}</span>)}
              </div>
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}

function CImportScreen({ theme, accent }) {
  return (
    <div style={{ overflow: "auto", height: "100%", padding: "24px 32px" }}>
      <div style={{ color: theme.overlay0, fontSize: 11, marginBottom: 16 }}>── import · /import ──</div>
      <div style={{ color: theme.text, fontSize: 22, marginBottom: 6 }}>$ eterea import</div>
      <div style={{ color: theme.overlay1, fontSize: 13, marginBottom: 28 }}>
        Reads CSV, JSON, or X archive .js. Parsed locally. Written to <span style={{ color: accent }}>~/.local/share/eterea/bookmarks.db</span>
      </div>

      <div style={{ background: theme.mantle, border: `1px solid ${theme.surface0}`, padding: 14, marginBottom: 16 }}>
        <div style={{ color: theme.overlay0, fontSize: 11, marginBottom: 10 }}>── source ──</div>
        <div style={{ display: "flex", gap: 0 }}>
          <span style={{ background: theme.surface0, color: theme.text, padding: "8px 12px", borderRight: `1px solid ${theme.surface1}` }}>path:</span>
          <input
            defaultValue="/home/you/Downloads/twitter-archive/data/bookmarks.js"
            style={{ flex: 1, background: theme.crust, border: "none", padding: "8px 12px", color: theme.text, fontFamily: "inherit", outline: "none" }}
          />
          <button style={{ background: accent, color: theme.crust, border: "none", padding: "0 16px", fontFamily: "inherit", fontWeight: 700, cursor: "pointer" }}>read</button>
        </div>
        <div style={{ marginTop: 10, display: "flex", gap: 14, color: theme.overlay1, fontSize: 11 }}>
          format: <span style={{ color: theme.green }}>auto-detect</span> · dedupe: <span style={{ color: theme.green }}>on</span> · keep media refs: <span style={{ color: theme.green }}>on</span>
        </div>
      </div>

      <div style={{ background: theme.mantle, border: `1px solid ${theme.surface0}`, padding: 14, marginBottom: 16 }}>
        <div style={{ color: accent, fontSize: 11, marginBottom: 10, fontWeight: 700 }}>── preview · 412 entries detected ──</div>
        <div style={{ display: "grid", gridTemplateColumns: "repeat(4, 1fr)", gap: 14 }}>
          {[["format", "X archive js", theme.text], ["entries", "412", theme.yellow], ["authors", "184", theme.green], ["range", "Sep '21 — Apr '26", theme.text]].map(([k, v, c]) => (
            <div key={k}>
              <div style={{ color: theme.overlay0, fontSize: 10.5, textTransform: "uppercase", letterSpacing: "0.1em" }}>{k}</div>
              <div style={{ marginTop: 3, color: c, fontSize: 14 }}>{v}</div>
            </div>
          ))}
        </div>
      </div>

      <div style={{ background: theme.crust, border: `1px solid ${theme.surface0}`, padding: 14, fontSize: 12, lineHeight: 1.7 }}>
        <div style={{ color: theme.overlay0, fontSize: 11, marginBottom: 8 }}>── log ──</div>
        <div style={{ color: theme.green }}>[OK] read 412 records from bookmarks.js</div>
        <div style={{ color: theme.green }}>[OK] schema validated · 412/412</div>
        <div style={{ color: theme.yellow }}>[WARN] 12 duplicates found, will be skipped</div>
        <div style={{ color: theme.green }}>[OK] resolved 184 unique authors</div>
        <div style={{ color: theme.subtext }}>$ ready to import 400 entries — press <CKbd theme={theme}>↵</CKbd> to confirm</div>
      </div>

      <div style={{ marginTop: 18, display: "flex", gap: 10 }}>
        <button style={{ background: "transparent", border: `1px solid ${theme.surface1}`, color: theme.subtext, padding: "8px 16px", fontFamily: "inherit", cursor: "pointer" }}>cancel</button>
        <button style={{ background: accent, color: theme.crust, border: "none", padding: "8px 20px", fontFamily: "inherit", fontWeight: 700, cursor: "pointer", marginLeft: "auto" }}>import 400 entries</button>
      </div>
    </div>
  );
}

function CSettingsScreen({ theme, accent, font, fontKey, setFontKey, density, setDensity, themeKey, setThemeKey, weight, setWeight, accentKey, setAccentKey }) {
  const sections = [
    { title: "appearance", items: [
      ["theme", themeKey, ["mocha", "macchiato", "latte"], setThemeKey],
      ["font", fontKey, ["jetbrains", "iosevka", "plex", "berkeley"], setFontKey],
      ["density", density, ["compact", "regular", "comfy"], setDensity],
      ["weight", weight, ["regular", "bold"], setWeight],
      ["accent", accentKey, ["pink", "blue", "green", "peach", "mauve", "yellow"], setAccentKey],
    ]},
    { title: "storage", items: [
      ["db.path", "~/.local/share/eterea/bookmarks.db", null, null],
      ["db.size", "8.4 MiB", null, null],
      ["backup.auto", "weekly", null, null],
      ["vacuum.on_close", "true", null, null],
    ]},
    { title: "import", items: [
      ["import.format", "auto", null, null],
      ["import.dedupe", "true", null, null],
      ["import.keep_media", "true", null, null],
    ]},
    { title: "about", items: [
      ["version", "0.1.0", null, null],
      ["built_with", "rust · dioxus · sqlite", null, null],
      ["license", "MIT", null, null],
    ]},
  ];
  return (
    <div style={{ overflow: "auto", height: "100%", padding: "24px 32px", maxWidth: 800 }}>
      <div style={{ color: theme.overlay0, fontSize: 11, marginBottom: 16 }}>── settings · ~/.config/eterea/config.toml ──</div>
      {sections.map(s => (
        <div key={s.title} style={{ marginBottom: 28 }}>
          <div style={{ color: theme.peach, fontSize: 11, fontWeight: 700, letterSpacing: "0.1em", textTransform: "uppercase", marginBottom: 8 }}>[{s.title}]</div>
          {s.items.map(([k, v, opts, setter]) => (
            <div key={k} style={{ display: "grid", gridTemplateColumns: "180px 1fr auto", gap: 14, padding: "6px 0", borderBottom: `1px dashed ${theme.surface0}`, fontSize: 13 }}>
              <span style={{ color: theme.green }}>{k}</span>
              <span style={{ color: theme.overlay0 }}>=</span>
              {opts && setter ? (
                <div style={{ display: "flex", gap: 6 }}>
                  {opts.map(o => (
                    <button key={o} onClick={() => setter(o)} style={{
                      background: v === o ? accent : "transparent",
                      color: v === o ? theme.crust : theme.subtext,
                      border: `1px solid ${v === o ? accent : theme.surface1}`,
                      padding: "2px 8px", fontFamily: "inherit", fontSize: 11.5, cursor: "pointer",
                    }}>{o}</button>
                  ))}
                </div>
              ) : (
                <span style={{ color: theme.text }}>"{v}"</span>
              )}
            </div>
          ))}
        </div>
      ))}
    </div>
  );
}

function COnboardingScreen({ theme, accent, font, onBegin }) {
  const ascii = `
    ╔══════════════════════════════════════════╗
    ║                                          ║
    ║    ████████ ████████ ████████ ████████   ║
    ║    ██       ██       ██       ██   ██    ║
    ║    ███████  ████████ █████    ████████   ║
    ║    ██             ██ ██       ██   ██    ║
    ║    ████████ ████████ ████████ ██   ██    ║
    ║                                          ║
    ║       a local-first reading room.        ║
    ║                                          ║
    ╚══════════════════════════════════════════╝`;
  return (
    <div style={{ overflow: "auto", height: "100%", padding: "20px 32px", display: "flex", flexDirection: "column", justifyContent: "center", alignItems: "center" }}>
      <pre style={{ margin: 0, color: accent, fontFamily: font, fontSize: 12, lineHeight: 1.1, textAlign: "center" }}>{ascii}</pre>

      <div style={{ marginTop: 32, color: theme.overlay1, fontSize: 13, textAlign: "center", maxWidth: 560, lineHeight: 1.7 }}>
        eterea is a TUI-style bookmark archive. all data stays on your machine.
        bring an export in, and the room becomes searchable in milliseconds.
      </div>

      <div style={{ marginTop: 32, width: "100%", maxWidth: 720, background: theme.mantle, border: `1px solid ${theme.surface0}` }}>
        <div style={{ padding: "8px 14px", borderBottom: `1px solid ${theme.surface0}`, color: theme.overlay0, fontSize: 11, textTransform: "uppercase", letterSpacing: "0.1em" }}>── getting started ──</div>
        {[
          ["1.", "export from x", "settings → your account → download an archive"],
          ["2.", "drop it here", "csv · json · X archive .js — all parsed locally"],
          ["3.", "read", "press 1..6 for tabs · / search · ? help"],
        ].map(([n, t, b]) => (
          <div key={n} style={{ display: "grid", gridTemplateColumns: "40px 160px 1fr", gap: 12, padding: "10px 14px", borderBottom: `1px dashed ${theme.surface0}`, fontSize: 13 }}>
            <span style={{ color: accent, fontWeight: 700 }}>{n}</span>
            <span style={{ color: theme.green }}>{t}</span>
            <span style={{ color: theme.subtext }}>{b}</span>
          </div>
        ))}
      </div>

      <div style={{ marginTop: 28, display: "flex", gap: 12 }}>
        <button onClick={onBegin} style={{ background: accent, color: theme.crust, border: "none", padding: "10px 22px", fontFamily: "inherit", fontWeight: 700, cursor: "pointer" }}>
          begin import
        </button>
        <button onClick={onBegin} style={{ background: "transparent", border: `1px solid ${theme.surface1}`, color: theme.subtext, padding: "10px 22px", fontFamily: "inherit", cursor: "pointer" }}>
          browse with sample data
        </button>
      </div>

      <div style={{ marginTop: 20, color: theme.overlay0, fontSize: 11, textAlign: "center" }}>
        local-first · no telemetry · MIT-licensed · written in rust
      </div>
    </div>
  );
}

window.CDetailScreen = CDetailScreen;
window.CAuthorScreen = CAuthorScreen;
window.CTagScreen = CTagScreen;
window.CSearchScreen = CSearchScreen;
window.CImportScreen = CImportScreen;
window.CSettingsScreen = CSettingsScreen;
window.COnboardingScreen = COnboardingScreen;

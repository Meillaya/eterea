// b-screens.jsx — Detail, Author, Tag, Search, Import, Settings, Onboarding.

function bScreenShell(paper) {
  return {
    width: "100%",
    height: "100%",
    background: paper.bg,
    color: paper.ink,
    fontFamily: B_FONT_SERIF,
    display: "flex",
    flexDirection: "column",
    overflow: "hidden",
    position: "relative",
  };
}

// ─── Detail / focused reading view ─────────────────────────
function BScreenDetail({ paper, accent, b }) {
  const bk = b || BOOKMARKS[2]; // Karpathy's transformer one — meaty
  return (
    <div style={bScreenShell(paper)}>
      <BMasthead paper={paper} compact subline={`Single entry · @${bk.handle}`} />
      <div style={{ padding: "10px 36px", borderBottom: `0.5px solid ${paper.rule}`, background: paper.panel, display: "flex", gap: 16, alignItems: "center" }}>
        <BSmallcaps style={{ color: paper.muted }}>← back to library</BSmallcaps>
        <span style={{ color: paper.soft }}>·</span>
        <BSmallcaps style={{ color: paper.muted }}>j/k for prev/next</BSmallcaps>
        <BSmallcaps style={{ color: paper.muted, marginLeft: "auto" }}>Entry №03 of {STATS.total}</BSmallcaps>
      </div>

      <div style={{ overflow: "auto", flex: 1, padding: "40px 36px 60px" }}>
        <div style={{ maxWidth: 720, margin: "0 auto", display: "grid", gridTemplateColumns: "minmax(0,1fr) 180px", gap: 36 }}>
          <div>
            <BSmallcaps style={{ color: accent, fontWeight: 700 }}>FROM THE ARCHIVE</BSmallcaps>
            <div style={{ display: "flex", gap: 12, alignItems: "baseline", margin: "12px 0 24px" }}>
              <h2 style={{ margin: 0, fontSize: 22, fontWeight: 500, fontStyle: "italic", color: paper.ink }}>{bk.name}</h2>
              <BSmallcaps style={{ color: paper.muted }}>@{bk.handle}</BSmallcaps>
            </div>

            <p style={{ margin: 0, fontSize: 26, lineHeight: 1.5, color: paper.ink, textWrap: "pretty" }}>
              <span style={{ float: "left", fontFamily: B_FONT_SERIF, fontSize: 110, lineHeight: 0.85, marginRight: 12, marginTop: 8, color: accent, fontWeight: 600, fontStyle: "italic" }}>
                {bk.content.charAt(0)}
              </span>
              {bk.content.slice(1)}
            </p>

            {/* Pull quote */}
            <blockquote style={{
              margin: "40px 0",
              padding: "20px 28px",
              borderLeft: `3px solid ${accent}`,
              fontSize: 22,
              fontStyle: "italic",
              lineHeight: 1.45,
              color: paper.ink,
              fontFamily: B_FONT_SERIF,
            }}>
              "Attention really was all we needed."
              <footer style={{ ...bSmallcaps, color: paper.muted, marginTop: 10 }}>
                — pulled by you, april 28, 2026
              </footer>
            </blockquote>

            {/* Note */}
            <div style={{ padding: 20, background: paper.panel, border: `0.5px solid ${paper.rule}`, marginTop: 24 }}>
              <BSmallcaps style={{ color: paper.muted }}>Your note</BSmallcaps>
              <p style={{ margin: "8px 0 0", fontSize: 15, lineHeight: 1.6, color: paper.ink, fontStyle: "italic" }}>
                Re-read whenever I'm tempted to add an architectural primitive. The boring answer keeps winning.
              </p>
            </div>

            {/* Footnotes / citations */}
            <div style={{ marginTop: 36, paddingTop: 16, borderTop: `0.5px solid ${paper.rule}` }}>
              <BSmallcaps style={{ color: paper.muted }}>Footnotes</BSmallcaps>
              <ol style={{ margin: "10px 0 0", paddingLeft: 24, fontSize: 13.5, lineHeight: 1.7, color: paper.muted }}>
                <li>Vaswani et al., "Attention Is All You Need" (2017) — the paper referenced.</li>
                <li>Tweet thread also discussed RWKV and state-space alternatives in replies.</li>
                <li>Cross-referenced in your archive: 4 other bookmarks tagged <span style={{ color: accent }}>#ml</span>.</li>
              </ol>
            </div>
          </div>

          {/* Marginalia / metadata column */}
          <aside style={{ borderLeft: `0.5px solid ${paper.rule}`, paddingLeft: 24, fontSize: 13 }}>
            <BSmallcaps style={{ color: paper.muted }}>Metadata</BSmallcaps>
            <dl style={{ margin: "10px 0 24px" }}>
              {[
                ["Tweeted",  fmtDate(bk.tweeted_at)],
                ["Saved",    fmtDate(bk.saved_at)],
                ["Likes",    bk.likes.toLocaleString()],
                ["Replies",  "143"],
                ["Reposts",  "892"],
                ["Media",    bk.media ? `${bk.media} attached` : "none"],
                ["Status",   bk.is_favorite ? "★ favored" : "—"],
              ].map(([k, v]) => (
                <div key={k} style={{ display: "flex", justifyContent: "space-between", padding: "5px 0", borderBottom: `0.5px dashed ${paper.soft}` }}>
                  <BSmallcaps style={{ color: paper.muted }}>{k}</BSmallcaps>
                  <span style={{ color: paper.ink, fontFamily: B_FONT_MONO, fontSize: 11.5 }}>{v}</span>
                </div>
              ))}
            </dl>

            <BSmallcaps style={{ color: paper.muted }}>Tags</BSmallcaps>
            <div style={{ display: "flex", flexWrap: "wrap", gap: 8, marginTop: 8 }}>
              {bk.tags.map(t => <BSmallcaps key={t} style={{ color: accent, fontWeight: 700 }}>{t}</BSmallcaps>)}
            </div>

            <BSmallcaps style={{ color: paper.muted, marginTop: 24, display: "block" }}>Actions</BSmallcaps>
            <div style={{ display: "flex", flexDirection: "column", gap: 6, marginTop: 8 }}>
              <button style={bGhostBtn(paper, accent)}>★ Favorite</button>
              <button style={bGhostBtn(paper, accent)}>↗ Open on X</button>
              <button style={bGhostBtn(paper, accent)}>+ Add note</button>
              <button style={{ ...bGhostBtn(paper, accent), color: paper.muted }}>Delete</button>
            </div>

            <BSmallcaps style={{ color: paper.muted, marginTop: 24, display: "block" }}>Related</BSmallcaps>
            {BOOKMARKS.slice(8, 11).map(r => (
              <div key={r.id} style={{ marginTop: 10, paddingTop: 10, borderTop: `0.5px dashed ${paper.soft}` }}>
                <BSmallcaps style={{ color: paper.muted }}>@{r.handle}</BSmallcaps>
                <p style={{ margin: "4px 0 0", fontSize: 12.5, lineHeight: 1.45, color: paper.ink }}>
                  {r.content.length > 90 ? r.content.slice(0, 90) + "…" : r.content}
                </p>
              </div>
            ))}
          </aside>
        </div>
      </div>
    </div>
  );
}

// ─── Author archive ─────────────────────────────────────────
function BScreenAuthor({ paper, accent }) {
  const handle = "karpathy";
  const author = BOOKMARKS.find(b => b.handle === handle) || BOOKMARKS[2];
  const items = BOOKMARKS.filter(b => b.handle === handle).concat(BOOKMARKS.slice(0, 4));
  return (
    <div style={bScreenShell(paper)}>
      <BMasthead paper={paper} compact subline={`Author archive · @${handle}`} />
      <div style={{ padding: "32px 36px 20px", borderBottom: `1px solid ${paper.ink}`, display: "flex", gap: 24, alignItems: "flex-end" }}>
        <div style={{ width: 80, height: 80, borderRadius: "50%", background: `linear-gradient(135deg, oklch(0.55 0.14 ${author.avatar_hue}), oklch(0.42 0.10 ${author.avatar_hue + 30}))`, flex: "none" }} />
        <div style={{ flex: 1 }}>
          <BSmallcaps style={{ color: paper.muted }}>The collected works of</BSmallcaps>
          <h2 style={{ margin: "4px 0 6px", fontSize: 56, fontWeight: 500, fontStyle: "italic", lineHeight: 0.9, color: paper.ink }}>
            {author.name}
          </h2>
          <BSmallcaps style={{ color: accent, fontWeight: 600 }}>@{author.handle}</BSmallcaps>
        </div>
        <div style={{ textAlign: "right" }}>
          <BSmallcaps style={{ color: paper.muted, display: "block", marginBottom: 4 }}>In your archive</BSmallcaps>
          <div style={{ fontFamily: B_FONT_MONO, fontSize: 22, color: paper.ink }}>{items.length}</div>
          <BSmallcaps style={{ color: paper.muted }}>entries · 3 favored</BSmallcaps>
        </div>
      </div>

      <div style={{ overflow: "auto", flex: 1, padding: "20px 36px 60px" }}>
        <BSmallcaps style={{ color: paper.muted }}>Most-saved tags</BSmallcaps>
        <div style={{ display: "flex", gap: 14, marginTop: 8, marginBottom: 24 }}>
          {[["ai", 5], ["ml", 4], ["papers", 2], ["systems", 1]].map(([t, c]) => (
            <span key={t} style={{ ...bSmallcaps, color: accent, fontWeight: 700 }}>
              {t} <span style={{ color: paper.muted, fontWeight: 400 }}>·{c}</span>
            </span>
          ))}
        </div>

        {items.map((b, i) => (
          <BEntry key={`${b.id}-${i}`} b={b} idx={i} paper={paper} accent={accent} isLead={i === 0}
            expanded={false} onToggle={() => {}} hovered={false} onHover={() => {}} onLeave={() => {}} />
        ))}
        <BColophon paper={paper} leftText={`All saved entries from @${handle}`} />
      </div>
    </div>
  );
}

// ─── Tag / topic page ───────────────────────────────────────
function BScreenTag({ paper, accent }) {
  const tag = "rust";
  const items = BOOKMARKS.filter(b => b.tags.includes(tag)).concat(BOOKMARKS.slice(0, 3));
  return (
    <div style={bScreenShell(paper)}>
      <BMasthead paper={paper} compact subline={`Topic · #${tag}`} />
      <div style={{ padding: "32px 36px 20px", borderBottom: `1px solid ${paper.ink}` }}>
        <BSmallcaps style={{ color: paper.muted }}>A topic page</BSmallcaps>
        <div style={{ display: "flex", alignItems: "baseline", gap: 18, marginTop: 6 }}>
          <h2 style={{ margin: 0, fontSize: 72, fontWeight: 500, fontStyle: "italic", lineHeight: 0.9, color: accent }}>
            #{tag}
          </h2>
          <BSmallcaps style={{ color: paper.muted }}>{items.length} entries from {new Set(items.map(b => b.handle)).size} authors</BSmallcaps>
        </div>
        <p style={{ margin: "16px 0 0", fontSize: 16, fontStyle: "italic", color: paper.muted, maxWidth: 600, lineHeight: 1.55 }}>
          Everything in your archive tagged <span style={{ color: accent }}>#{tag}</span> — sorted by when you saved it.
          Co-occurring tags below.
        </p>
        <div style={{ display: "flex", gap: 14, marginTop: 16 }}>
          <BSmallcaps style={{ color: paper.muted }}>Co-occurs with:</BSmallcaps>
          {[["performance", 8], ["tools", 6], ["systems", 4]].map(([t, c]) => (
            <BSmallcaps key={t} style={{ color: accent }}>{t} <span style={{ color: paper.muted }}>·{c}</span></BSmallcaps>
          ))}
        </div>
      </div>

      <div style={{ overflow: "auto", flex: 1, padding: "20px 36px 60px" }}>
        {items.map((b, i) => (
          <BEntry key={`${b.id}-${i}`} b={b} idx={i} paper={paper} accent={accent} isLead={i === 0}
            expanded={false} onToggle={() => {}} hovered={false} onHover={() => {}} onLeave={() => {}} />
        ))}
        <BColophon paper={paper} leftText={`All entries · #${tag}`} />
      </div>
    </div>
  );
}

// ─── Search results ─────────────────────────────────────────
function BScreenSearch({ paper, accent }) {
  const q = "rust";
  const matches = BOOKMARKS.filter(b => b.content.toLowerCase().includes(q) || b.tags.includes(q) || b.handle.toLowerCase().includes(q));
  function highlight(text, q) {
    const parts = text.split(new RegExp(`(${q})`, "ig"));
    return parts.map((p, i) =>
      p.toLowerCase() === q.toLowerCase()
        ? <mark key={i} style={{ background: accent, color: paper.bg, padding: "1px 3px" }}>{p}</mark>
        : <React.Fragment key={i}>{p}</React.Fragment>
    );
  }
  return (
    <div style={bScreenShell(paper)}>
      <BMasthead paper={paper} compact subline="Search results" />
      <div style={{ padding: "24px 36px", borderBottom: `1px solid ${paper.ink}` }}>
        <BSmallcaps style={{ color: paper.muted }}>You searched for</BSmallcaps>
        <div style={{ display: "flex", alignItems: "baseline", gap: 18, marginTop: 6 }}>
          <h2 style={{ margin: 0, fontSize: 48, fontWeight: 500, fontStyle: "italic", lineHeight: 1, color: paper.ink }}>
            "{q}"
          </h2>
          <BSmallcaps style={{ color: paper.muted }}>{matches.length} results · 0.04s</BSmallcaps>
        </div>
        <div style={{ display: "flex", gap: 18, marginTop: 16 }}>
          <BSmallcaps style={{ color: paper.muted }}>Filter:</BSmallcaps>
          {["All", "Content", "Tags", "Authors", "Notes"].map((f, i) => (
            <BSmallcaps key={f} style={{ color: i === 0 ? accent : paper.muted, fontWeight: i === 0 ? 700 : 500, cursor: "pointer" }}>{f}</BSmallcaps>
          ))}
        </div>
      </div>

      <div style={{ overflow: "auto", flex: 1, padding: "20px 36px 60px" }}>
        {matches.map((b, i) => (
          <article key={b.id} style={{
            display: "grid",
            gridTemplateColumns: "44px minmax(0,1fr) 140px",
            gap: 18,
            padding: "20px 0",
            borderTop: `0.5px solid ${paper.rule}`,
            cursor: "pointer",
          }}>
            <div style={{ ...bSmallcaps, color: paper.muted, paddingTop: 4 }}>№{String(i + 1).padStart(2, "0")}</div>
            <div>
              <div style={{ display: "flex", gap: 10, alignItems: "baseline", marginBottom: 6 }}>
                <span style={{ fontStyle: "italic", fontSize: 14 }}>{b.name}</span>
                <BSmallcaps style={{ color: paper.muted }}>@{highlight(b.handle, q)}</BSmallcaps>
              </div>
              <p style={{ margin: 0, fontSize: 17, lineHeight: 1.5, color: paper.ink }}>
                {highlight(b.content, q)}
              </p>
              <div style={{ display: "flex", gap: 14, marginTop: 10 }}>
                {b.tags.map(t => (
                  <BSmallcaps key={t} style={{ color: t === q ? accent : paper.muted, fontWeight: t === q ? 700 : 500 }}>
                    {t === q ? <mark style={{ background: accent, color: paper.bg, padding: "1px 4px" }}>{t}</mark> : t}
                  </BSmallcaps>
                ))}
              </div>
            </div>
            <div style={{ textAlign: "right", ...bSmallcaps, color: paper.muted, paddingTop: 4 }}>
              <div>{fmtDate(b.tweeted_at).toUpperCase()}</div>
              <div style={{ marginTop: 6, color: accent }}>match in: content</div>
            </div>
          </article>
        ))}
        <BColophon paper={paper} leftText={`End of results for "${q}"`} />
      </div>
    </div>
  );
}

// ─── Import flow ────────────────────────────────────────────
function BScreenImport({ paper, accent }) {
  return (
    <div style={bScreenShell(paper)}>
      <BMasthead paper={paper} compact subline="Import bookmarks" />
      <div style={{ overflow: "auto", flex: 1, padding: "40px 36px 60px" }}>
        <div style={{ maxWidth: 680, margin: "0 auto" }}>
          <BSmallcaps style={{ color: paper.muted }}>Step 2 of 3 · Import</BSmallcaps>
          <h2 style={{ margin: "8px 0 12px", fontSize: 44, fontWeight: 500, fontStyle: "italic", lineHeight: 1.05, color: paper.ink }}>
            Bring more into the room.
          </h2>
          <p style={{ margin: 0, fontSize: 16, lineHeight: 1.6, color: paper.muted, fontStyle: "italic", maxWidth: 540 }}>
            Drop a bookmark export onto the page, or paste a path. Eterea reads CSV, JSON,
            and the bookmark JS file from an X archive — all parsed locally and written to
            <span style={{ fontFamily: B_FONT_MONO, fontStyle: "normal", color: paper.ink }}> ~/eterea/bookmarks.db</span>.
          </p>

          {/* Drop zone */}
          <div style={{
            marginTop: 28,
            padding: "44px 28px",
            border: `1px dashed ${paper.muted}`,
            background: paper.panel,
            textAlign: "center",
          }}>
            <BSmallcaps style={{ color: paper.muted }}>Drop a file here</BSmallcaps>
            <h3 style={{ margin: "10px 0 4px", fontSize: 24, fontWeight: 500, fontStyle: "italic", color: paper.ink }}>
              .csv · .json · .js
            </h3>
            <p style={{ margin: 0, fontSize: 13, color: paper.muted }}>or paste a local path below</p>

            <div style={{ marginTop: 20, display: "flex", gap: 0, alignItems: "stretch", maxWidth: 480, margin: "20px auto 0" }}>
              <input
                defaultValue="/home/you/Downloads/twitter-archive/data/bookmarks.js"
                style={{
                  flex: 1,
                  padding: "10px 14px",
                  border: `0.5px solid ${paper.ink}`,
                  borderRight: "none",
                  background: paper.bg,
                  fontFamily: B_FONT_MONO,
                  fontSize: 12,
                  color: paper.ink,
                  outline: "none",
                }}
              />
              <button style={{
                ...bSmallcaps,
                background: paper.ink,
                color: paper.bg,
                border: "none",
                padding: "10px 18px",
                cursor: "pointer",
                fontWeight: 700,
              }}>
                Read file
              </button>
            </div>
          </div>

          {/* Preview / progress */}
          <div style={{ marginTop: 28, padding: 20, background: paper.panel, border: `0.5px solid ${paper.rule}` }}>
            <BSmallcaps style={{ color: accent, fontWeight: 700 }}>Preview · 412 entries detected</BSmallcaps>
            <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr 1fr 1fr", gap: 16, marginTop: 16 }}>
              {[
                ["Format",   "X Archive JS"],
                ["Entries",  "412"],
                ["Authors",  "184"],
                ["Date",     "Sep '21 — Apr '26"],
              ].map(([k, v]) => (
                <div key={k}>
                  <BSmallcaps style={{ color: paper.muted }}>{k}</BSmallcaps>
                  <div style={{ marginTop: 4, fontSize: 16, color: paper.ink, fontStyle: "italic" }}>{v}</div>
                </div>
              ))}
            </div>
            <div style={{ marginTop: 18, paddingTop: 14, borderTop: `0.5px dashed ${paper.rule}` }}>
              <BSmallcaps style={{ color: paper.muted }}>Sample (first 3)</BSmallcaps>
              <ul style={{ margin: "8px 0 0", paddingLeft: 18, fontSize: 13.5, lineHeight: 1.7, color: paper.ink }}>
                {BOOKMARKS.slice(0, 3).map(b => (
                  <li key={b.id} style={{ fontStyle: "italic" }}>
                    <span style={{ fontFamily: B_FONT_MONO, fontStyle: "normal", color: paper.muted }}>@{b.handle}</span>
                    {" — "}
                    {b.content.slice(0, 90)}…
                  </li>
                ))}
              </ul>
            </div>
          </div>

          <div style={{ display: "flex", gap: 12, marginTop: 24, justifyContent: "flex-end" }}>
            <button style={bGhostBtn(paper, accent)}>Cancel</button>
            <button style={{ ...bSmallcaps, background: accent, color: paper.bg, border: "none", padding: "10px 22px", cursor: "pointer", fontWeight: 700 }}>
              Import 412 entries
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}

// ─── Settings ───────────────────────────────────────────────
function BScreenSettings({ paper, accent }) {
  const sections = [
    { title: "Reading", items: [
      ["Default layout", "Issue (current)", "select"],
      ["Body font", "Source Serif 4", "select"],
      ["Density", "Regular", "select"],
      ["Show drop caps on lead entry", "Yes", "toggle"],
    ]},
    { title: "Storage", items: [
      ["Database location", "~/.local/share/eterea/bookmarks.db", "path"],
      ["Auto-backup", "Weekly · last: 4 days ago", "toggle"],
      ["Vacuum on close", "On", "toggle"],
    ]},
    { title: "Import", items: [
      ["Default format", "Auto-detect", "select"],
      ["Deduplicate on import", "On", "toggle"],
      ["Keep media references", "On", "toggle"],
    ]},
    { title: "About", items: [
      ["Version", "0.1.0", "text"],
      ["Built with", "Rust · Dioxus · SQLite", "text"],
      ["License", "MIT", "text"],
    ]},
  ];

  return (
    <div style={bScreenShell(paper)}>
      <BMasthead paper={paper} compact subline="Preferences" />
      <div style={{ overflow: "auto", flex: 1, padding: "32px 36px 60px" }}>
        <div style={{ maxWidth: 720, margin: "0 auto" }}>
          <BSmallcaps style={{ color: paper.muted }}>Preferences · v0.1.0</BSmallcaps>
          <h2 style={{ margin: "8px 0 28px", fontSize: 44, fontWeight: 500, fontStyle: "italic", lineHeight: 1.05, color: paper.ink }}>
            Set the room.
          </h2>

          {sections.map(s => (
            <section key={s.title} style={{ marginBottom: 36 }}>
              <BSmallcaps style={{ color: accent, fontWeight: 700 }}>{s.title}</BSmallcaps>
              <div style={{ marginTop: 10, borderTop: `1px solid ${paper.ink}` }}>
                {s.items.map(([label, value, kind]) => (
                  <div key={label} style={{
                    display: "grid",
                    gridTemplateColumns: "240px 1fr auto",
                    gap: 18,
                    padding: "14px 0",
                    borderBottom: `0.5px solid ${paper.rule}`,
                    alignItems: "center",
                  }}>
                    <div style={{ fontStyle: "italic", fontSize: 15, color: paper.ink }}>{label}</div>
                    <div style={{ fontFamily: kind === "path" ? B_FONT_MONO : B_FONT_SERIF, fontSize: kind === "path" ? 12 : 14, color: paper.muted }}>
                      {value}
                    </div>
                    <div>
                      {kind === "toggle"
                        ? <span style={{
                            display: "inline-block", width: 32, height: 18, borderRadius: 999,
                            background: value === "On" || value === "Yes" || value.startsWith("Weekly") ? accent : paper.soft,
                            position: "relative",
                          }}>
                            <span style={{
                              position: "absolute", top: 2,
                              left: value === "On" || value === "Yes" || value.startsWith("Weekly") ? 16 : 2,
                              width: 14, height: 14, borderRadius: "50%", background: paper.bg,
                            }} />
                          </span>
                        : kind === "select"
                          ? <BSmallcaps style={{ color: paper.ink, fontWeight: 600 }}>change ↓</BSmallcaps>
                          : kind === "path"
                            ? <BSmallcaps style={{ color: paper.muted }}>reveal</BSmallcaps>
                            : null}
                    </div>
                  </div>
                ))}
              </div>
            </section>
          ))}
        </div>
      </div>
    </div>
  );
}

// ─── Onboarding / first-run empty state ──────────────────────
function BScreenOnboarding({ paper, accent }) {
  return (
    <div style={bScreenShell(paper)}>
      <BMasthead paper={paper} subline="A first edition" />

      <div style={{ overflow: "auto", flex: 1, padding: "40px 36px 60px", display: "flex", flexDirection: "column", justifyContent: "center" }}>
        <div style={{ maxWidth: 720, margin: "0 auto", textAlign: "center" }}>
          <BSmallcaps style={{ color: paper.muted }}>The room is empty</BSmallcaps>
          <h2 style={{ margin: "12px 0 16px", fontSize: 64, fontWeight: 500, fontStyle: "italic", lineHeight: 1, color: paper.ink, letterSpacing: "-0.015em" }}>
            Welcome.
          </h2>
          <p style={{ margin: "0 auto", maxWidth: 540, fontSize: 18, lineHeight: 1.6, color: paper.muted, fontStyle: "italic" }}>
            Eterea is a reading room for what you bookmarked on X. It runs entirely on your machine —
            nothing leaves the room. Bring an export in, and the archive becomes a paper.
          </p>

          {/* Three-step ledger */}
          <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr 1fr", gap: 0, marginTop: 44, textAlign: "left", borderTop: `1px solid ${paper.ink}`, borderBottom: `1px solid ${paper.ink}` }}>
            {[
              ["I", "Export from X", "Settings → Your account → Download an archive of your data. The bookmarks.js file is what you want."],
              ["II", "Drop it here", "CSV, JSON, or the X archive JS — Eterea reads all three. Your archive is parsed locally."],
              ["III", "Read", "Issue, Front Page, Long-Read, or Spread. Search with /, navigate with j/k. The room stays quiet."],
            ].map(([num, title, body], i) => (
              <div key={num} style={{
                padding: "22px 20px",
                borderRight: i < 2 ? `0.5px solid ${paper.rule}` : "none",
              }}>
                <div style={{ fontFamily: B_FONT_SERIF, fontStyle: "italic", fontSize: 36, color: accent, lineHeight: 1, fontWeight: 600 }}>{num}.</div>
                <h3 style={{ margin: "10px 0 6px", fontSize: 17, fontStyle: "italic", fontWeight: 500, color: paper.ink }}>{title}</h3>
                <p style={{ margin: 0, fontSize: 13.5, lineHeight: 1.55, color: paper.muted }}>{body}</p>
              </div>
            ))}
          </div>

          <div style={{ marginTop: 36, display: "flex", gap: 12, justifyContent: "center" }}>
            <button style={{ ...bSmallcaps, background: accent, color: paper.bg, border: "none", padding: "12px 24px", cursor: "pointer", fontWeight: 700 }}>
              Begin import
            </button>
            <button style={bGhostBtn(paper, accent)}>Browse with sample data</button>
          </div>

          <p style={{ marginTop: 28, fontSize: 12, fontStyle: "italic", color: paper.muted }}>
            Local-first · no telemetry · MIT-licensed · written in Rust
          </p>
        </div>
      </div>
    </div>
  );
}

window.BScreenDetail = BScreenDetail;
window.BScreenAuthor = BScreenAuthor;
window.BScreenTag = BScreenTag;
window.BScreenSearch = BScreenSearch;
window.BScreenImport = BScreenImport;
window.BScreenSettings = BScreenSettings;
window.BScreenOnboarding = BScreenOnboarding;

// b-shell.jsx — full connected app: nav state + sidebar + every screen reachable.

const BNAV_ITEMS = [
  { id: "library",  label: "Library",   sub: "412" },
  { id: "favorites",label: "Favorites", sub: "47" },
  { id: "authors",  label: "Authors",   sub: "184" },
  { id: "topics",   label: "Topics",    sub: "62" },
  { id: "search",   label: "Search",    sub: "/" },
  { id: "import",   label: "Import",    sub: "" },
  { id: "settings", label: "Settings",  sub: "" },
];

function BSidebar({ paper, accent, screen, navigate, density }) {
  return (
    <aside style={{
      width: 220, flexShrink: 0,
      borderRight: `0.5px solid ${paper.rule}`,
      background: paper.panel,
      padding: "20px 0 16px",
      display: "flex", flexDirection: "column",
      overflow: "auto",
    }}>
      <div style={{ padding: "0 20px 18px", borderBottom: `0.5px solid ${paper.rule}` }}>
        <BSmallcaps style={{ color: paper.muted }}>Local · v0.1.0</BSmallcaps>
        <h1 style={{
          margin: "4px 0 0", fontSize: 36, lineHeight: 0.9,
          fontFamily: B_FONT_SERIF, fontStyle: "italic", fontWeight: 500,
          letterSpacing: "-0.015em", color: paper.ink,
        }}>Eterea</h1>
      </div>

      <nav style={{ padding: "12px 8px", flex: 1 }}>
        {BNAV_ITEMS.map(item => {
          const active = screen === item.id || (screen.startsWith(item.id + ":"));
          return (
            <button
              key={item.id}
              onClick={() => navigate(item.id)}
              style={{
                width: "100%",
                display: "flex", justifyContent: "space-between", alignItems: "baseline",
                padding: "8px 12px",
                marginBottom: 2,
                border: "none",
                background: active ? paper.ink : "transparent",
                color: active ? paper.bg : paper.ink,
                cursor: "pointer",
                fontFamily: B_FONT_SERIF,
                fontSize: 15,
                fontStyle: "italic",
                textAlign: "left",
              }}
            >
              <span>{item.label}</span>
              <span style={{ ...bSmallcaps, color: active ? paper.bg : paper.muted, fontStyle: "normal" }}>{item.sub}</span>
            </button>
          );
        })}

        <div style={{ ...bSmallcaps, color: paper.muted, padding: "20px 12px 6px" }}>Top tags</div>
        {TOP_TAGS.slice(0, 6).map(([t, c]) => (
          <button
            key={t}
            onClick={() => navigate("topic:" + t)}
            style={{
              width: "100%",
              display: "flex", justifyContent: "space-between", alignItems: "baseline",
              padding: "5px 12px",
              border: "none", background: "transparent",
              color: screen === "topic:" + t ? accent : paper.ink,
              fontWeight: screen === "topic:" + t ? 700 : 500,
              cursor: "pointer", fontFamily: B_FONT_SERIF,
              fontSize: 13.5, textAlign: "left",
            }}
          >
            <span>#{t}</span>
            <span style={{ ...bSmallcaps, color: paper.muted, fontWeight: 400 }}>{c}</span>
          </button>
        ))}
      </nav>

      <div style={{ padding: "12px 20px", borderTop: `0.5px solid ${paper.rule}` }}>
        <BSmallcaps style={{ color: paper.muted, display: "block", marginBottom: 4 }}>Database</BSmallcaps>
        <div style={{ fontFamily: B_FONT_MONO, fontSize: 10.5, color: paper.muted, lineHeight: 1.5 }}>
          ~/.local/share/<br/>eterea/bookmarks.db
        </div>
        <div style={{ marginTop: 6, ...bSmallcaps, color: paper.ink }}>
          ● ready · {STATS.total} entries
        </div>
      </div>
    </aside>
  );
}

// Library screen with full state, layout switcher, and click-through to detail.
function BLibraryScreen({ paper, accent, density, navigate, initialFavOnly = false }) {
  const [layout, setLayout] = React.useState("issue");
  const [activeTag, setActiveTag] = React.useState(null);
  const [favOnly, setFavOnly] = React.useState(initialFavOnly);
  const [query, setQuery] = React.useState("");
  const [expanded, setExpanded] = React.useState(null);
  const [hovered, setHovered] = React.useState(null);

  const items = BOOKMARKS.filter(b => {
    if (favOnly && !b.is_favorite) return false;
    if (activeTag && !b.tags.includes(activeTag)) return false;
    if (query) {
      const q = query.toLowerCase();
      if (!b.content.toLowerCase().includes(q) && !b.handle.toLowerCase().includes(q) && !b.tags.some(t => t.includes(q))) return false;
    }
    return true;
  });

  React.useEffect(() => {
    const onKey = (e) => {
      if (e.target.tagName === "INPUT") return;
      const ids = items.map(b => b.id);
      if (e.key === "j" || e.key === "ArrowDown") {
        const i = expanded ? ids.indexOf(expanded) : -1;
        setExpanded(ids[Math.min(ids.length - 1, i + 1)]); e.preventDefault();
      } else if (e.key === "k" || e.key === "ArrowUp") {
        const i = expanded ? ids.indexOf(expanded) : ids.length;
        setExpanded(ids[Math.max(0, i - 1)]); e.preventDefault();
      } else if (e.key === "Enter" && expanded) {
        navigate("entry:" + expanded);
      } else if (e.key === "Escape") setExpanded(null);
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [items, expanded]);

  const padding = density === "compact" ? "16px 36px 50px" : density === "comfy" ? "28px 36px 60px" : "20px 36px 50px";
  const title = favOnly ? "Favorites" : activeTag ? `On "${activeTag}"` : query ? `"${query}"` : "The Library";

  // Wrap entry click: shift = open detail, plain = expand inline
  const wrappedSetExpanded = (id) => {
    if (id && expanded === id) setExpanded(null);
    else setExpanded(id);
  };
  const props = { items, paper, accent, expanded, setExpanded: wrappedSetExpanded, hovered, setHovered, title };

  return (
    <div style={{ display: "flex", flexDirection: "column", height: "100%", overflow: "hidden" }}>
      <BMasthead paper={paper} compact />
      <BTagRail
        paper={paper} accent={accent}
        activeTag={activeTag} setActiveTag={setActiveTag}
        favOnly={favOnly} setFavOnly={setFavOnly}
        currentLayout={layout} setCurrentLayout={setLayout}
        query={query} setQuery={setQuery}
      />
      <div style={{ overflow: "auto", flex: 1, padding, position: "relative" }}>
        <div style={{ marginBottom: 14, display: "flex", justifyContent: "space-between", alignItems: "center" }}>
          <BSmallcaps style={{ color: paper.muted }}>Click any entry to expand · enter to open · j/k to move</BSmallcaps>
          <button onClick={() => navigate("import")} style={{ ...bSmallcaps, background: paper.ink, color: paper.bg, border: "none", padding: "6px 14px", cursor: "pointer", fontWeight: 700 }}>
            Import
          </button>
        </div>
        {layout === "issue"  && <BLayoutIssue {...props} />}
        {layout === "front"  && <BLayoutFront {...props} />}
        {layout === "long"   && <BLayoutLong {...props} />}
        {layout === "spread" && <BLayoutSpread {...props} />}
      </div>
      <div style={{ position: "absolute", left: 256, bottom: 14, ...bSmallcaps, color: paper.muted }}>
        j/k navigate · enter to open · / search · esc collapse
      </div>
    </div>
  );
}

// Index pages (authors, topics) — directory views.
function BAuthorsIndex({ paper, accent, navigate }) {
  // Synthesize from BOOKMARKS + a few extras
  const seen = {};
  BOOKMARKS.forEach(b => {
    if (!seen[b.handle]) seen[b.handle] = { handle: b.handle, name: b.name, hue: b.avatar_hue, count: 0, fav: 0, tags: new Set() };
    seen[b.handle].count++;
    if (b.is_favorite) seen[b.handle].fav++;
    b.tags.forEach(t => seen[b.handle].tags.add(t));
  });
  const authors = Object.values(seen).sort((a, b) => b.count - a.count);
  // Pad to feel like a real archive
  const padded = [...authors, { handle: "danabra_mov", name: "Dan Abramov", hue: 195, count: 8, fav: 3, tags: new Set(["react","design"]) },
    { handle: "vitalikbuterin", name: "Vitalik Buterin", hue: 270, count: 6, fav: 1, tags: new Set(["systems","crypto"]) },
    { handle: "patio11", name: "Patrick McKenzie", hue: 50, count: 11, fav: 4, tags: new Set(["business","engineering"]) },
    { handle: "tef_ebooks", name: "tef", hue: 130, count: 5, fav: 2, tags: new Set(["plt","engineering"]) }];

  return (
    <div style={{ display: "flex", flexDirection: "column", height: "100%", overflow: "hidden" }}>
      <BMasthead paper={paper} compact subline="Authors index" />
      <div style={{ padding: "24px 36px", borderBottom: `1px solid ${paper.ink}` }}>
        <BSmallcaps style={{ color: paper.muted }}>The contributors</BSmallcaps>
        <h2 style={{ margin: "6px 0 8px", fontSize: 44, fontWeight: 500, fontStyle: "italic", color: paper.ink, lineHeight: 1 }}>
          {padded.length} voices
        </h2>
        <p style={{ margin: 0, color: paper.muted, fontStyle: "italic", fontSize: 14 }}>
          Sorted by entries in your archive. Click a name to see all of their saved tweets.
        </p>
      </div>
      <div style={{ overflow: "auto", flex: 1, padding: "0 36px 60px" }}>
        {padded.map((a, i) => (
          <button
            key={a.handle}
            onClick={() => navigate("author:" + a.handle)}
            style={{
              display: "grid",
              gridTemplateColumns: "44px 48px minmax(0,1fr) 200px 100px",
              gap: 18, alignItems: "center",
              width: "100%", padding: "16px 0",
              borderBottom: `0.5px solid ${paper.rule}`,
              background: "transparent", border: "none",
              borderTop: i === 0 ? "none" : undefined,
              cursor: "pointer", textAlign: "left",
              fontFamily: B_FONT_SERIF,
            }}
          >
            <BSmallcaps style={{ color: paper.muted }}>№{String(i + 1).padStart(2, "0")}</BSmallcaps>
            <div style={{ width: 40, height: 40, borderRadius: "50%",
              background: `linear-gradient(135deg, oklch(0.55 0.14 ${a.hue}), oklch(0.42 0.10 ${a.hue + 30}))` }} />
            <div>
              <div style={{ fontStyle: "italic", fontSize: 17, color: paper.ink }}>{a.name}</div>
              <BSmallcaps style={{ color: paper.muted }}>@{a.handle}</BSmallcaps>
            </div>
            <div style={{ display: "flex", gap: 10, flexWrap: "wrap" }}>
              {[...a.tags].slice(0, 3).map(t => (
                <BSmallcaps key={t} style={{ color: accent }}>{t}</BSmallcaps>
              ))}
            </div>
            <div style={{ textAlign: "right" }}>
              <div style={{ fontFamily: B_FONT_MONO, fontSize: 16, color: paper.ink }}>{a.count}</div>
              <BSmallcaps style={{ color: paper.muted }}>{a.fav} ★</BSmallcaps>
            </div>
          </button>
        ))}
        <BColophon paper={paper} leftText="All authors in the archive" />
      </div>
    </div>
  );
}

function BTopicsIndex({ paper, accent, navigate }) {
  const all = [...TOP_TAGS,
    ["plt", 11], ["types", 9], ["react", 8], ["business", 7], ["c", 6],
    ["linux", 5], ["tools", 14], ["ml", 12], ["papers", 6], ["teams", 4]];
  return (
    <div style={{ display: "flex", flexDirection: "column", height: "100%", overflow: "hidden" }}>
      <BMasthead paper={paper} compact subline="Topics index" />
      <div style={{ padding: "24px 36px", borderBottom: `1px solid ${paper.ink}` }}>
        <BSmallcaps style={{ color: paper.muted }}>Topics in your archive</BSmallcaps>
        <h2 style={{ margin: "6px 0 8px", fontSize: 44, fontWeight: 500, fontStyle: "italic", color: paper.ink, lineHeight: 1 }}>
          {all.length} tags · sized by frequency
        </h2>
      </div>
      <div style={{ overflow: "auto", flex: 1, padding: "32px 36px 60px" }}>
        <div style={{ display: "flex", flexWrap: "wrap", gap: "20px 28px", alignItems: "baseline" }}>
          {all.sort((a, b) => b[1] - a[1]).map(([t, c]) => (
            <button
              key={t}
              onClick={() => navigate("topic:" + t)}
              style={{
                background: "transparent", border: "none", cursor: "pointer",
                fontFamily: B_FONT_SERIF, fontStyle: "italic", color: paper.ink,
                fontSize: Math.min(56, 14 + c * 1.6),
                padding: 0, lineHeight: 1.1,
                fontWeight: 500,
              }}
            >
              #{t}<sup style={{ fontFamily: B_FONT_MONO, fontStyle: "normal", fontSize: 11, color: paper.muted, marginLeft: 4, fontWeight: 400 }}>{c}</sup>
            </button>
          ))}
        </div>
        <BColophon paper={paper} leftText="All topics" />
      </div>
    </div>
  );
}

// Search-as-you-type screen
function BSearchScreen({ paper, accent }) {
  const [q, setQ] = React.useState("rust");
  const matches = q ? BOOKMARKS.filter(b =>
    b.content.toLowerCase().includes(q.toLowerCase()) ||
    b.tags.some(t => t.includes(q.toLowerCase())) ||
    b.handle.toLowerCase().includes(q.toLowerCase())
  ) : [];
  function highlight(text) {
    if (!q) return text;
    const parts = text.split(new RegExp(`(${q})`, "ig"));
    return parts.map((p, i) =>
      p.toLowerCase() === q.toLowerCase()
        ? <mark key={i} style={{ background: accent, color: paper.bg, padding: "1px 3px" }}>{p}</mark>
        : <React.Fragment key={i}>{p}</React.Fragment>
    );
  }
  return (
    <div style={{ display: "flex", flexDirection: "column", height: "100%", overflow: "hidden" }}>
      <BMasthead paper={paper} compact subline="Search" />
      <div style={{ padding: "32px 36px 22px", borderBottom: `1px solid ${paper.ink}` }}>
        <BSmallcaps style={{ color: paper.muted }}>Search the archive</BSmallcaps>
        <div style={{ display: "flex", alignItems: "baseline", gap: 8, marginTop: 8 }}>
          <span style={{ fontSize: 48, fontStyle: "italic", color: paper.muted }}>"</span>
          <input
            value={q} onChange={e => setQ(e.target.value)} autoFocus
            style={{
              fontSize: 48, fontFamily: B_FONT_SERIF, fontStyle: "italic",
              fontWeight: 500, color: paper.ink, background: "transparent",
              border: "none", borderBottom: `0.5px solid ${paper.muted}`,
              outline: "none", padding: "0 4px 4px", flex: 1, lineHeight: 1,
            }}
          />
          <span style={{ fontSize: 48, fontStyle: "italic", color: paper.muted }}>"</span>
        </div>
        <div style={{ display: "flex", gap: 18, marginTop: 16 }}>
          <BSmallcaps style={{ color: paper.muted }}>{matches.length} results · 0.04s</BSmallcaps>
          <span style={{ color: paper.soft }}>·</span>
          {["All", "Content", "Tags", "Authors", "Notes"].map((f, i) => (
            <BSmallcaps key={f} style={{ color: i === 0 ? accent : paper.muted, fontWeight: i === 0 ? 700 : 500, cursor: "pointer" }}>{f}</BSmallcaps>
          ))}
        </div>
      </div>
      <div style={{ overflow: "auto", flex: 1, padding: "12px 36px 60px" }}>
        {matches.length === 0 && (
          <div style={{ padding: "60px 0", textAlign: "center", color: paper.muted, fontStyle: "italic" }}>
            Nothing yet. Try a tag, an author, or a phrase.
          </div>
        )}
        {matches.map((b, i) => (
          <article key={b.id} style={{
            display: "grid", gridTemplateColumns: "44px minmax(0,1fr) 140px",
            gap: 18, padding: "18px 0", borderTop: `0.5px solid ${paper.rule}`, cursor: "pointer",
          }}>
            <BSmallcaps style={{ color: paper.muted, paddingTop: 4 }}>№{String(i + 1).padStart(2, "0")}</BSmallcaps>
            <div>
              <div style={{ display: "flex", gap: 10, alignItems: "baseline", marginBottom: 6 }}>
                <span style={{ fontStyle: "italic", fontSize: 14 }}>{b.name}</span>
                <BSmallcaps style={{ color: paper.muted }}>@{highlight(b.handle)}</BSmallcaps>
              </div>
              <p style={{ margin: 0, fontSize: 16.5, lineHeight: 1.5, color: paper.ink }}>
                {highlight(b.content)}
              </p>
              <div style={{ display: "flex", gap: 14, marginTop: 8 }}>
                {b.tags.map(t => (
                  <BSmallcaps key={t} style={{ color: t.includes(q.toLowerCase()) ? accent : paper.muted, fontWeight: t.includes(q.toLowerCase()) ? 700 : 500 }}>{t}</BSmallcaps>
                ))}
              </div>
            </div>
            <div style={{ textAlign: "right", ...bSmallcaps, color: paper.muted, paddingTop: 4 }}>
              <div>{fmtDate(b.tweeted_at).toUpperCase()}</div>
            </div>
          </article>
        ))}
      </div>
    </div>
  );
}

// Multi-step import flow with progress states
function BImportFlow({ paper, accent, navigate }) {
  const [step, setStep] = React.useState(1);
  const steps = [
    [1, "Source", "Pick a file or paste a path"],
    [2, "Preview", "Review what will be imported"],
    [3, "Importing", "Writing to the local database"],
    [4, "Done", "Open the library"],
  ];
  return (
    <div style={{ display: "flex", flexDirection: "column", height: "100%", overflow: "hidden" }}>
      <BMasthead paper={paper} compact subline={`Import · step ${step} of 4`} />
      {/* Stepper */}
      <div style={{ padding: "16px 36px", borderBottom: `0.5px solid ${paper.rule}`, background: paper.panel,
        display: "grid", gridTemplateColumns: "1fr 1fr 1fr 1fr", gap: 0 }}>
        {steps.map(([n, t, sub]) => (
          <div key={n} style={{
            paddingRight: 16,
            borderLeft: n === 1 ? "none" : `0.5px solid ${paper.rule}`,
            paddingLeft: n === 1 ? 0 : 16,
          }}>
            <div style={{ display: "flex", alignItems: "baseline", gap: 8 }}>
              <span style={{ fontFamily: B_FONT_SERIF, fontStyle: "italic", fontSize: 26,
                color: step === n ? accent : step > n ? paper.ink : paper.muted, fontWeight: 600 }}>
                {step > n ? "✓" : n}
              </span>
              <BSmallcaps style={{ color: step === n ? accent : paper.ink, fontWeight: step === n ? 700 : 500 }}>{t}</BSmallcaps>
            </div>
            <div style={{ fontSize: 12, color: paper.muted, fontStyle: "italic", marginTop: 2 }}>{sub}</div>
          </div>
        ))}
      </div>

      <div style={{ overflow: "auto", flex: 1, padding: "40px 36px 50px" }}>
        <div style={{ maxWidth: 680, margin: "0 auto" }}>
          {step === 1 && (
            <>
              <h2 style={{ margin: "0 0 12px", fontSize: 38, fontStyle: "italic", fontWeight: 500, color: paper.ink }}>Bring an export.</h2>
              <p style={{ margin: 0, fontSize: 15, color: paper.muted, fontStyle: "italic" }}>
                CSV, JSON, or the X archive JS file — Eterea reads all three.
              </p>
              <div style={{ marginTop: 28, padding: "44px 28px", border: `1px dashed ${paper.muted}`, background: paper.panel, textAlign: "center" }}>
                <BSmallcaps style={{ color: paper.muted }}>Drop a file</BSmallcaps>
                <h3 style={{ margin: "10px 0 4px", fontSize: 22, fontStyle: "italic", fontWeight: 500, color: paper.ink }}>.csv · .json · .js</h3>
                <p style={{ margin: 0, fontSize: 13, color: paper.muted }}>or paste a path</p>
                <input defaultValue="/home/you/Downloads/twitter-archive/data/bookmarks.js"
                  style={{ marginTop: 18, padding: "10px 14px", border: `0.5px solid ${paper.ink}`, background: paper.bg,
                    fontFamily: B_FONT_MONO, fontSize: 12, color: paper.ink, outline: "none", width: "min(480px, 100%)" }}
                />
              </div>
              <div style={{ display: "flex", gap: 12, marginTop: 24, justifyContent: "flex-end" }}>
                <button onClick={() => navigate("library")} style={bGhostBtn(paper, accent)}>Cancel</button>
                <button onClick={() => setStep(2)} style={{ ...bSmallcaps, background: accent, color: paper.bg, border: "none", padding: "10px 22px", cursor: "pointer", fontWeight: 700 }}>
                  Read file →
                </button>
              </div>
            </>
          )}
          {step === 2 && (
            <>
              <h2 style={{ margin: "0 0 12px", fontSize: 38, fontStyle: "italic", fontWeight: 500, color: paper.ink }}>Preview.</h2>
              <p style={{ margin: 0, fontSize: 15, color: paper.muted, fontStyle: "italic" }}>
                412 entries detected in <span style={{ fontFamily: B_FONT_MONO, fontStyle: "normal", color: paper.ink }}>bookmarks.js</span>.
              </p>
              <div style={{ marginTop: 24, padding: 20, background: paper.panel, border: `0.5px solid ${paper.rule}` }}>
                <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr 1fr 1fr", gap: 16 }}>
                  {[["Format", "X Archive JS"], ["Entries", "412"], ["New", "397"], ["Duplicates", "15"]].map(([k, v]) => (
                    <div key={k}>
                      <BSmallcaps style={{ color: paper.muted }}>{k}</BSmallcaps>
                      <div style={{ marginTop: 4, fontSize: 18, fontStyle: "italic", color: paper.ink }}>{v}</div>
                    </div>
                  ))}
                </div>
                <div style={{ marginTop: 20, paddingTop: 14, borderTop: `0.5px dashed ${paper.rule}` }}>
                  <BSmallcaps style={{ color: paper.muted }}>Sample (first 3)</BSmallcaps>
                  <ul style={{ margin: "8px 0 0", paddingLeft: 18, fontSize: 13.5, lineHeight: 1.7, color: paper.ink }}>
                    {BOOKMARKS.slice(0, 3).map(b => (
                      <li key={b.id} style={{ fontStyle: "italic" }}>
                        <span style={{ fontFamily: B_FONT_MONO, fontStyle: "normal", color: paper.muted }}>@{b.handle}</span>
                        {" — "}{b.content.slice(0, 90)}…
                      </li>
                    ))}
                  </ul>
                </div>
              </div>
              <div style={{ display: "flex", gap: 12, marginTop: 24, justifyContent: "flex-end" }}>
                <button onClick={() => setStep(1)} style={bGhostBtn(paper, accent)}>← Back</button>
                <button onClick={() => setStep(3)} style={{ ...bSmallcaps, background: accent, color: paper.bg, border: "none", padding: "10px 22px", cursor: "pointer", fontWeight: 700 }}>
                  Import 397 entries →
                </button>
              </div>
            </>
          )}
          {step === 3 && (
            <>
              <h2 style={{ margin: "0 0 12px", fontSize: 38, fontStyle: "italic", fontWeight: 500, color: paper.ink }}>Importing.</h2>
              <p style={{ margin: 0, fontSize: 15, color: paper.muted, fontStyle: "italic" }}>
                Parsing JSON · resolving authors · building FTS index · committing transactions.
              </p>
              <div style={{ marginTop: 32, padding: 28, background: paper.panel, border: `0.5px solid ${paper.rule}` }}>
                <div style={{ display: "flex", justifyContent: "space-between", marginBottom: 8 }}>
                  <BSmallcaps style={{ color: paper.ink }}>Writing entries</BSmallcaps>
                  <BSmallcaps style={{ color: accent, fontWeight: 700 }}>312 / 397</BSmallcaps>
                </div>
                <div style={{ height: 4, background: paper.soft, position: "relative" }}>
                  <div style={{ position: "absolute", inset: "0 21% 0 0", background: accent }} />
                </div>
                <div style={{ marginTop: 18, fontFamily: B_FONT_MONO, fontSize: 11, color: paper.muted, lineHeight: 1.7 }}>
                  <div>✓ schema migration · v3 → v4</div>
                  <div>✓ author dedup · 184 unique</div>
                  <div>✓ tag extraction · 62 topics</div>
                  <div>· FTS index · in progress…</div>
                  <div style={{ color: paper.soft }}>· vacuum · pending</div>
                </div>
              </div>
              <div style={{ marginTop: 24, textAlign: "right" }}>
                <button onClick={() => setStep(4)} style={{ ...bSmallcaps, background: accent, color: paper.bg, border: "none", padding: "10px 22px", cursor: "pointer", fontWeight: 700 }}>
                  Skip wait (demo) →
                </button>
              </div>
            </>
          )}
          {step === 4 && (
            <>
              <BSmallcaps style={{ color: accent, fontWeight: 700 }}>Done</BSmallcaps>
              <h2 style={{ margin: "8px 0 12px", fontSize: 56, fontStyle: "italic", fontWeight: 500, color: paper.ink, lineHeight: 1 }}>
                The room is full.
              </h2>
              <p style={{ margin: 0, fontSize: 17, color: paper.muted, fontStyle: "italic", maxWidth: 540, lineHeight: 1.55 }}>
                397 new entries, 184 authors, 62 topics. All written to <span style={{ fontFamily: B_FONT_MONO, fontStyle: "normal", color: paper.ink }}>~/eterea/bookmarks.db</span> · 14 ms.
              </p>
              <div style={{ display: "flex", gap: 12, marginTop: 32 }}>
                <button onClick={() => navigate("library")} style={{ ...bSmallcaps, background: accent, color: paper.bg, border: "none", padding: "12px 24px", cursor: "pointer", fontWeight: 700 }}>
                  Open the library
                </button>
                <button onClick={() => setStep(1)} style={bGhostBtn(paper, accent)}>Import another</button>
              </div>
            </>
          )}
        </div>
      </div>
    </div>
  );
}

// The full app — sidebar + main content router.
function BApp({ accent, paperTone, density, startScreen = "library" }) {
  const paper = bPaper(paperTone);
  const [screen, setScreen] = React.useState(startScreen);
  const navigate = (s) => { setScreen(s); };

  let main;
  if (screen === "library") main = <BLibraryScreen paper={paper} accent={accent} density={density} navigate={navigate} />;
  else if (screen === "favorites") main = <BLibraryScreen paper={paper} accent={accent} density={density} navigate={navigate} initialFavOnly />;
  else if (screen === "authors") main = <BAuthorsIndex paper={paper} accent={accent} navigate={navigate} />;
  else if (screen === "topics") main = <BTopicsIndex paper={paper} accent={accent} navigate={navigate} />;
  else if (screen === "search") main = <BSearchScreen paper={paper} accent={accent} />;
  else if (screen === "import") main = <BImportFlow paper={paper} accent={accent} navigate={navigate} />;
  else if (screen === "settings") main = <BScreenSettings paper={paper} accent={accent} />;
  else if (screen.startsWith("entry:")) main = <BScreenDetail paper={paper} accent={accent} b={BOOKMARKS.find(b => b.id === screen.slice(6))} />;
  else if (screen.startsWith("author:")) main = <BScreenAuthor paper={paper} accent={accent} />;
  else if (screen.startsWith("topic:")) main = <BScreenTag paper={paper} accent={accent} />;
  else if (screen === "onboarding") main = <BScreenOnboarding paper={paper} accent={accent} />;
  else main = <BLibraryScreen paper={paper} accent={accent} density={density} navigate={navigate} />;

  return (
    <div style={{
      width: "100%", height: "100%",
      display: "flex", background: paper.bg, color: paper.ink,
      fontFamily: B_FONT_SERIF, overflow: "hidden",
    }}>
      <BSidebar paper={paper} accent={accent} screen={screen} navigate={navigate} density={density} />
      <main style={{ flex: 1, minWidth: 0, display: "flex", flexDirection: "column" }}>{main}</main>
    </div>
  );
}

window.BApp = BApp;
window.BSidebar = BSidebar;
window.BLibraryScreen = BLibraryScreen;
window.BAuthorsIndex = BAuthorsIndex;
window.BTopicsIndex = BTopicsIndex;
window.BSearchScreen = BSearchScreen;
window.BImportFlow = BImportFlow;

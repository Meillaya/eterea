// variation-c.jsx — "Terminal Archive"
// Monospace, dense, power-user. Reads like a TUI: borders made of box chars,
// keyboard hints, command palette in header, columnar metadata, no soft chrome.

const VarC_styles = {
  shell: {
    width: "100%",
    height: "100%",
    background: "#0d0d0d",
    color: "#d4d0c8",
    fontFamily: '"Berkeley Mono", "JetBrains Mono", "SF Mono", ui-monospace, Menlo, monospace',
    fontSize: 12.5,
    lineHeight: 1.5,
    display: "grid",
    gridTemplateRows: "auto 1fr auto",
    overflow: "hidden",
  },
  rule: { borderColor: "#3a3a3a" },
};

const VarC_GREEN = "#a3be8c";
const VarC_AMBER = "#d8a657";
const VarC_DIM = "#6e6e6e";

function VarC_Row({ b, idx, accent, selected, dense, onSelect }) {
  return (
    <div
      onClick={onSelect}
      style={{
        display: "grid",
        gridTemplateColumns: "32px 110px 80px minmax(0,1fr) 100px 28px",
        gap: 12,
        padding: dense ? "5px 16px" : "9px 16px",
        background: selected ? "rgba(255, 152, 97, 0.10)" : "transparent",
        borderLeft: `2px solid ${selected ? accent : "transparent"}`,
        cursor: "pointer",
        alignItems: "baseline",
        borderBottom: "1px dashed rgba(255,255,255,0.04)",
      }}
    >
      <span style={{ color: VarC_DIM, fontSize: 11 }}>{String(idx + 1).padStart(3, "0")}</span>
      <span style={{ color: VarC_GREEN, overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
        @{b.handle}
      </span>
      <span style={{ color: VarC_DIM, fontSize: 11 }}>{fmtRel(b.tweeted_at)}</span>
      <span style={{ overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap", color: "#e0dcd2" }}>
        {b.content}
      </span>
      <span style={{ color: accent, fontSize: 11 }}>
        {b.tags.map((t) => `#${t}`).join(" ")}
      </span>
      <span style={{ color: b.is_favorite ? VarC_AMBER : VarC_DIM, textAlign: "center" }}>
        {b.is_favorite ? "★" : "·"}
      </span>
    </div>
  );
}

function VariationC({ accent = "#ff9861", density = "regular" }) {
  const [selectedIdx, setSelectedIdx] = React.useState(0);
  const [activeTag, setActiveTag] = React.useState(null);
  const [favOnly, setFavOnly] = React.useState(false);
  const [query, setQuery] = React.useState("");

  const filtered = BOOKMARKS.filter((b) => {
    if (favOnly && !b.is_favorite) return false;
    if (activeTag && !b.tags.includes(activeTag)) return false;
    if (query && !b.content.toLowerCase().includes(query.toLowerCase()) && !b.handle.toLowerCase().includes(query.toLowerCase())) return false;
    return true;
  });

  const selected = filtered[selectedIdx] || filtered[0];
  const dense = density !== "comfy";

  return (
    <div style={VarC_styles.shell}>
      {/* Header / status line */}
      <header
        style={{
          padding: "10px 16px",
          borderBottom: "1px solid #3a3a3a",
          display: "flex",
          gap: 16,
          alignItems: "center",
          background: "#161616",
        }}
      >
        <span style={{ color: accent, fontWeight: 600, letterSpacing: "0.05em" }}>
          eterea<span style={{ color: VarC_DIM }}>/</span>library
        </span>
        <span style={{ color: VarC_DIM }}>│</span>
        <span style={{ color: VarC_DIM }}>{STATS.total} entries · {STATS.authors} authors · {STATS.favorites} favorites</span>
        <div style={{ marginLeft: "auto", display: "flex", alignItems: "center", gap: 10 }}>
          <span style={{ color: VarC_DIM }}>$</span>
          <input
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            placeholder="grep ..."
            style={{
              background: "transparent",
              border: "none",
              color: "#e0dcd2",
              outline: "none",
              fontFamily: "inherit",
              fontSize: 12.5,
              width: 240,
            }}
          />
          <span style={{ color: VarC_DIM, fontSize: 11, padding: "1px 6px", border: "1px solid #3a3a3a" }}>/</span>
        </div>
      </header>

      {/* Body: tag rail + table + detail */}
      <div style={{ display: "grid", gridTemplateColumns: "180px minmax(0, 1fr) 360px", overflow: "hidden", minHeight: 0 }}>
        {/* Tag rail */}
        <aside style={{ borderRight: "1px solid #3a3a3a", padding: "12px 0", overflow: "auto", background: "#0a0a0a" }}>
          <div style={{ padding: "0 16px 8px", color: VarC_DIM, fontSize: 11, letterSpacing: "0.1em", textTransform: "uppercase" }}>
            ── filters ──
          </div>
          <button
            onClick={() => { setActiveTag(null); setFavOnly(false); }}
            style={{
              display: "block", width: "100%", textAlign: "left", background: "transparent",
              border: "none", color: !activeTag && !favOnly ? accent : "#d4d0c8",
              padding: "4px 16px", fontFamily: "inherit", fontSize: 12.5, cursor: "pointer",
            }}
          >
            {!activeTag && !favOnly ? "▶" : " "} all <span style={{ color: VarC_DIM }}>({STATS.total})</span>
          </button>
          <button
            onClick={() => { setFavOnly(!favOnly); setActiveTag(null); }}
            style={{
              display: "block", width: "100%", textAlign: "left", background: "transparent",
              border: "none", color: favOnly ? accent : "#d4d0c8",
              padding: "4px 16px", fontFamily: "inherit", fontSize: 12.5, cursor: "pointer",
            }}
          >
            {favOnly ? "▶" : " "} favorites <span style={{ color: VarC_DIM }}>({STATS.favorites})</span>
          </button>
          <div style={{ padding: "12px 16px 6px", color: VarC_DIM, fontSize: 11, letterSpacing: "0.1em", textTransform: "uppercase" }}>
            ── tags ──
          </div>
          {TOP_TAGS.map(([t, c]) => (
            <button
              key={t}
              onClick={() => { setActiveTag(activeTag === t ? null : t); setFavOnly(false); }}
              style={{
                display: "block", width: "100%", textAlign: "left", background: "transparent",
                border: "none", color: activeTag === t ? accent : "#d4d0c8",
                padding: "3px 16px", fontFamily: "inherit", fontSize: 12.5, cursor: "pointer",
              }}
            >
              {activeTag === t ? "▶" : " "} #{t} <span style={{ color: VarC_DIM }}>{c}</span>
            </button>
          ))}
          <div style={{ padding: "16px 16px 0", color: VarC_DIM, fontSize: 11 }}>
            <div>─────────────</div>
            <div style={{ marginTop: 8 }}>j/k  navigate</div>
            <div>f    favorite</div>
            <div>/    search</div>
            <div>i    import</div>
            <div>?    help</div>
          </div>
        </aside>

        {/* Table */}
        <main style={{ overflow: "auto", display: "flex", flexDirection: "column", minWidth: 0 }}>
          <div
            style={{
              display: "grid",
              gridTemplateColumns: "32px 110px 80px minmax(0,1fr) 100px 28px",
              gap: 12,
              padding: "8px 16px",
              borderBottom: "1px solid #3a3a3a",
              color: VarC_DIM,
              fontSize: 11,
              textTransform: "uppercase",
              letterSpacing: "0.1em",
              background: "#161616",
              position: "sticky",
              top: 0,
            }}
          >
            <span>idx</span>
            <span>author</span>
            <span>when</span>
            <span>content</span>
            <span>tags</span>
            <span>★</span>
          </div>
          {filtered.map((b, i) => (
            <VarC_Row
              key={b.id}
              b={b}
              idx={i}
              accent={accent}
              dense={dense}
              selected={i === selectedIdx}
              onSelect={() => setSelectedIdx(i)}
            />
          ))}
        </main>

        {/* Detail panel */}
        <aside style={{ borderLeft: "1px solid #3a3a3a", padding: "16px", overflow: "auto", background: "#0a0a0a" }}>
          {selected && (
            <>
              <div style={{ color: VarC_DIM, fontSize: 11, letterSpacing: "0.1em", textTransform: "uppercase", marginBottom: 12 }}>
                ── entry {String(selectedIdx + 1).padStart(3, "0")} ──
              </div>
              <div style={{ marginBottom: 6 }}>
                <span style={{ color: VarC_DIM }}>author: </span>
                <span style={{ color: VarC_GREEN }}>@{selected.handle}</span>
                <span style={{ color: VarC_DIM }}> ({selected.name})</span>
              </div>
              <div style={{ marginBottom: 6 }}>
                <span style={{ color: VarC_DIM }}>tweeted: </span>
                <span>{fmtDate(selected.tweeted_at)}</span>
              </div>
              <div style={{ marginBottom: 6 }}>
                <span style={{ color: VarC_DIM }}>saved:   </span>
                <span>{fmtDate(selected.saved_at)}</span>
              </div>
              <div style={{ marginBottom: 6 }}>
                <span style={{ color: VarC_DIM }}>likes:   </span>
                <span style={{ color: VarC_AMBER }}>{selected.likes.toLocaleString()}</span>
              </div>
              <div style={{ marginBottom: 6 }}>
                <span style={{ color: VarC_DIM }}>tags:    </span>
                <span style={{ color: accent }}>{selected.tags.map((t) => `#${t}`).join(" ")}</span>
              </div>
              <div style={{ marginBottom: 6 }}>
                <span style={{ color: VarC_DIM }}>fav:     </span>
                <span style={{ color: selected.is_favorite ? VarC_AMBER : VarC_DIM }}>
                  {selected.is_favorite ? "true" : "false"}
                </span>
              </div>
              <div style={{ borderTop: "1px dashed #3a3a3a", margin: "16px 0", color: VarC_DIM, fontSize: 11 }}>
                ── content ──
              </div>
              <p
                style={{
                  margin: 0,
                  whiteSpace: "pre-wrap",
                  color: "#e0dcd2",
                  lineHeight: 1.65,
                  fontSize: 13,
                }}
              >
                {selected.content}
              </p>
              <div style={{ borderTop: "1px dashed #3a3a3a", margin: "16px 0 12px" }} />
              <div style={{ display: "flex", gap: 8, fontSize: 11 }}>
                <button style={{ background: "transparent", border: "1px solid #3a3a3a", color: "#d4d0c8", padding: "4px 10px", fontFamily: "inherit", cursor: "pointer" }}>
                  [f] favorite
                </button>
                <button style={{ background: "transparent", border: "1px solid #3a3a3a", color: "#d4d0c8", padding: "4px 10px", fontFamily: "inherit", cursor: "pointer" }}>
                  [o] open
                </button>
                <button style={{ background: "transparent", border: "1px solid #3a3a3a", color: "#d4d0c8", padding: "4px 10px", fontFamily: "inherit", cursor: "pointer" }}>
                  [d] delete
                </button>
              </div>
            </>
          )}
        </aside>
      </div>

      {/* Status / command line */}
      <footer
        style={{
          padding: "6px 16px",
          borderTop: "1px solid #3a3a3a",
          background: "#161616",
          display: "flex",
          gap: 16,
          fontSize: 11,
          color: VarC_DIM,
        }}
      >
        <span style={{ color: VarC_GREEN }}>● ready</span>
        <span>{filtered.length}/{STATS.total} shown</span>
        <span>sort: tweeted_at desc</span>
        <span>db: ~/.local/share/eterea/bookmarks.db</span>
        <span style={{ marginLeft: "auto" }}>:q to quit</span>
      </footer>
    </div>
  );
}

window.VariationC = VariationC;

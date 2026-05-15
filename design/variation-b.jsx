// variation-b.jsx — "Editorial Reading Room"
// Big literary statement. Off-white paper, deep ink, large serif headlines,
// numbered entries. Bookmarks read like quoted excerpts in a magazine. Tag
// rail at top instead of side. Author bylines, drop-caps on focused entry.

const VarB_styles = {
  shell: {
    width: "100%",
    height: "100%",
    background: "#f3ede1",
    color: "#1d1a16",
    fontFamily: '"Charter", "Iowan Old Style", "Source Serif Pro", Georgia, serif',
    display: "grid",
    gridTemplateRows: "auto auto 1fr",
    overflow: "hidden",
    position: "relative",
  },
  rule: { borderTop: "1px solid #1d1a16" },
  thinRule: { borderTop: "0.5px solid rgba(29,26,22,0.25)" },
  smallcaps: {
    fontFamily: '"Söhne Mono", "JetBrains Mono", ui-monospace, Menlo, monospace',
    fontSize: 10.5,
    letterSpacing: "0.18em",
    textTransform: "uppercase",
    color: "#1d1a16",
  },
};

function VarB_Entry({ b, idx, accent, isFocus }) {
  return (
    <article
      style={{
        display: "grid",
        gridTemplateColumns: "44px minmax(0,1fr) 140px",
        gap: 18,
        padding: "22px 0",
        borderTop: "0.5px solid rgba(29,26,22,0.2)",
        alignItems: "start",
      }}
    >
      <div style={{ ...VarB_styles.smallcaps, paddingTop: 4, color: "rgba(29,26,22,0.5)" }}>
        №{String(idx + 1).padStart(2, "0")}
      </div>
      <div style={{ minWidth: 0 }}>
        <div style={{ display: "flex", gap: 10, alignItems: "baseline", marginBottom: 8 }}>
          <span style={{ fontWeight: 600, fontSize: 14, fontStyle: "italic" }}>{b.name}</span>
          <span style={{ ...VarB_styles.smallcaps, color: "rgba(29,26,22,0.5)" }}>@{b.handle}</span>
        </div>
        <p
          style={{
            margin: 0,
            fontSize: isFocus ? 22 : 18,
            lineHeight: 1.45,
            fontWeight: 400,
            color: "#1d1a16",
            textWrap: "pretty",
          }}
        >
          {isFocus && (
            <span
              style={{
                float: "left",
                fontFamily: '"Charter", Georgia, serif',
                fontSize: 64,
                lineHeight: 0.85,
                marginRight: 8,
                marginTop: 4,
                color: accent,
                fontWeight: 600,
              }}
            >
              {b.content.charAt(0)}
            </span>
          )}
          {isFocus ? b.content.slice(1) : b.content}
        </p>
        <div style={{ display: "flex", gap: 14, marginTop: 12, alignItems: "center", flexWrap: "wrap" }}>
          {b.tags.map((t) => (
            <span key={t} style={{ ...VarB_styles.smallcaps, color: accent, fontWeight: 600 }}>
              {t}
            </span>
          ))}
        </div>
      </div>
      <div style={{ textAlign: "right", ...VarB_styles.smallcaps, color: "rgba(29,26,22,0.5)", paddingTop: 4 }}>
        <div>{fmtDate(b.tweeted_at).toUpperCase()}</div>
        <div style={{ marginTop: 6 }}>
          {b.is_favorite ? (
            <span style={{ color: accent, letterSpacing: 0 }}>★ favored</span>
          ) : (
            <span>saved {fmtRel(b.saved_at)}</span>
          )}
        </div>
      </div>
    </article>
  );
}

function VariationB({ accent = "#a8421f", density = "regular" }) {
  const [activeTag, setActiveTag] = React.useState(null);
  const [favOnly, setFavOnly] = React.useState(false);
  const [focusIdx, setFocusIdx] = React.useState(0);

  const filtered = BOOKMARKS.filter((b) => {
    if (favOnly && !b.is_favorite) return false;
    if (activeTag && !b.tags.includes(activeTag)) return false;
    return true;
  });

  return (
    <div style={VarB_styles.shell}>
      {/* Masthead */}
      <header style={{ padding: "20px 36px 14px", borderBottom: "1px solid #1d1a16" }}>
        <div style={{ display: "flex", justifyContent: "space-between", alignItems: "baseline" }}>
          <div style={VarB_styles.smallcaps}>Vol. I · Local Edition · {fmtDate("2026-04-19T00:00:00Z")}</div>
          <div style={VarB_styles.smallcaps}>{STATS.total} entries · {STATS.authors} voices</div>
        </div>
        <div style={{ display: "flex", justifyContent: "space-between", alignItems: "flex-end", marginTop: 12 }}>
          <h1
            style={{
              margin: 0,
              fontSize: 88,
              lineHeight: 0.85,
              fontWeight: 500,
              fontStyle: "italic",
              letterSpacing: "-0.02em",
            }}
          >
            Eterea
          </h1>
          <div style={{ textAlign: "right", maxWidth: 320 }}>
            <p style={{ margin: 0, fontStyle: "italic", fontSize: 14, lineHeight: 1.5, color: "rgba(29,26,22,0.7)" }}>
              "A reading room for what you bookmarked, kept off the timeline and on the page."
            </p>
          </div>
        </div>
      </header>

      {/* Tag rail */}
      <nav
        style={{
          padding: "10px 36px",
          borderBottom: "0.5px solid rgba(29,26,22,0.25)",
          display: "flex",
          gap: 22,
          alignItems: "center",
          flexWrap: "wrap",
          background: "#ece4d3",
        }}
      >
        <button
          onClick={() => { setActiveTag(null); setFavOnly(false); }}
          style={{
            ...VarB_styles.smallcaps,
            background: "transparent",
            border: "none",
            cursor: "pointer",
            fontWeight: !activeTag && !favOnly ? 700 : 500,
            color: !activeTag && !favOnly ? accent : "#1d1a16",
            padding: 0,
          }}
        >
          All
        </button>
        <button
          onClick={() => { setFavOnly(!favOnly); setActiveTag(null); }}
          style={{
            ...VarB_styles.smallcaps,
            background: "transparent",
            border: "none",
            cursor: "pointer",
            fontWeight: favOnly ? 700 : 500,
            color: favOnly ? accent : "#1d1a16",
            padding: 0,
          }}
        >
          ★ Favorites
        </button>
        <span style={{ color: "rgba(29,26,22,0.3)" }}>|</span>
        {TOP_TAGS.map(([t, c]) => (
          <button
            key={t}
            onClick={() => { setActiveTag(activeTag === t ? null : t); setFavOnly(false); }}
            style={{
              ...VarB_styles.smallcaps,
              background: "transparent",
              border: "none",
              cursor: "pointer",
              fontWeight: activeTag === t ? 700 : 500,
              color: activeTag === t ? accent : "#1d1a16",
              padding: 0,
            }}
          >
            {t} <span style={{ color: "rgba(29,26,22,0.4)", fontWeight: 400 }}>·{c}</span>
          </button>
        ))}
        <div style={{ marginLeft: "auto", display: "flex", alignItems: "center", gap: 12 }}>
          <input
            placeholder="Search the archive…"
            style={{
              background: "transparent",
              border: "none",
              borderBottom: "0.5px solid rgba(29,26,22,0.4)",
              padding: "4px 0",
              fontSize: 13,
              fontStyle: "italic",
              outline: "none",
              color: "#1d1a16",
              width: 220,
              fontFamily: 'inherit',
            }}
          />
          <button
            style={{
              ...VarB_styles.smallcaps,
              background: "#1d1a16",
              color: "#f3ede1",
              border: "none",
              padding: "8px 14px",
              cursor: "pointer",
              fontWeight: 600,
            }}
          >
            Import
          </button>
        </div>
      </nav>

      {/* Body — 2 columns: featured entry on left, full list on right */}
      <div style={{ display: "grid", gridTemplateColumns: "minmax(0, 1fr)", overflow: "hidden", padding: "20px 36px 50px" }}>
        <div style={{ overflow: "auto", paddingRight: 4 }}>
          {/* Section header */}
          <div style={{ display: "grid", gridTemplateColumns: "44px minmax(0,1fr) 140px", gap: 18, alignItems: "baseline", paddingBottom: 8 }}>
            <div></div>
            <h2 style={{ margin: 0, fontSize: 28, fontWeight: 500, fontStyle: "italic" }}>
              {favOnly ? "Favorites" : activeTag ? `On "${activeTag}"` : "The Library"}
            </h2>
            <div style={{ ...VarB_styles.smallcaps, color: "rgba(29,26,22,0.5)", textAlign: "right" }}>
              {filtered.length} of {STATS.total}
            </div>
          </div>

          {filtered.map((b, i) => (
            <VarB_Entry key={b.id} b={b} idx={i} accent={accent} isFocus={i === 0} />
          ))}

          {/* Colophon */}
          <div style={{ marginTop: 36, padding: "16px 0", borderTop: "1px solid #1d1a16", display: "flex", justifyContent: "space-between", ...VarB_styles.smallcaps, color: "rgba(29,26,22,0.5)" }}>
            <span>End of selection</span>
            <span>Compiled locally · ~/eterea/bookmarks.db</span>
            <span>fin.</span>
          </div>
        </div>
      </div>
    </div>
  );
}

window.VariationB = VariationB;

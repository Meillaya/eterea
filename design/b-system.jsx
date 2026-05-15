// b-system.jsx — shared building blocks for the Editorial Reading-Room.
// Type, paper tones, masthead, smallcaps, dropcaps, marginalia, footer.

const B_PAPERS = {
  cream:    { bg: "#f3ede1", panel: "#ece4d3", ink: "#1d1a16", muted: "rgba(29,26,22,0.55)", rule: "rgba(29,26,22,0.22)", soft: "rgba(29,26,22,0.10)" },
  offwhite: { bg: "#f7f4ee", panel: "#efeae0", ink: "#181614", muted: "rgba(24,22,20,0.55)", rule: "rgba(24,22,20,0.20)", soft: "rgba(24,22,20,0.08)" },
  gray:     { bg: "#e6e3dc", panel: "#dcd8cf", ink: "#16140f", muted: "rgba(22,20,15,0.58)", rule: "rgba(22,20,15,0.25)", soft: "rgba(22,20,15,0.10)" },
};

function bPaper(tone) { return B_PAPERS[tone] || B_PAPERS.cream; }

const B_FONT_SERIF = '"Source Serif 4", "Charter", "Iowan Old Style", Georgia, serif';
const B_FONT_MONO  = '"JetBrains Mono", ui-monospace, Menlo, monospace';

const bSmallcaps = {
  fontFamily: B_FONT_MONO,
  fontSize: 10.5,
  letterSpacing: "0.18em",
  textTransform: "uppercase",
  fontWeight: 500,
};

function BSmallcaps({ children, style, ...rest }) {
  return <span style={{ ...bSmallcaps, ...style }} {...rest}>{children}</span>;
}

function BMasthead({ paper, tagline = "A reading room for what you bookmarked, kept off the timeline and on the page.", subline, accent, compact = false }) {
  return (
    <header style={{
      padding: compact ? "14px 36px 10px" : "20px 36px 14px",
      borderBottom: `1px solid ${paper.ink}`,
      flexShrink: 0,
    }}>
      <div style={{ display: "flex", justifyContent: "space-between", alignItems: "baseline" }}>
        <BSmallcaps style={{ color: paper.ink }}>
          Vol. I · Local Edition · {fmtDate("2026-04-28T00:00:00Z")}
        </BSmallcaps>
        <BSmallcaps style={{ color: paper.ink }}>
          {subline || `${STATS.total} entries · ${STATS.authors} voices`}
        </BSmallcaps>
      </div>
      <div style={{ display: "flex", justifyContent: "space-between", alignItems: "flex-end", marginTop: compact ? 6 : 12 }}>
        <h1 style={{
          margin: 0,
          fontSize: compact ? 56 : 88,
          lineHeight: 0.85,
          fontWeight: 500,
          fontStyle: "italic",
          letterSpacing: "-0.02em",
          fontFamily: B_FONT_SERIF,
          color: paper.ink,
        }}>
          Eterea
        </h1>
        {!compact && (
          <p style={{
            margin: 0,
            maxWidth: 320,
            textAlign: "right",
            fontStyle: "italic",
            fontSize: 14,
            lineHeight: 1.5,
            color: paper.muted,
            fontFamily: B_FONT_SERIF,
          }}>
            "{tagline}"
          </p>
        )}
      </div>
    </header>
  );
}

function BTagRail({ paper, accent, activeTag, setActiveTag, favOnly, setFavOnly, currentLayout, setCurrentLayout, query, setQuery }) {
  const layoutOpts = [
    ["issue", "Issue"],
    ["front", "Front Page"],
    ["long",  "Long-Read"],
    ["spread","Spread"],
  ];
  return (
    <nav style={{
      padding: "10px 36px",
      borderBottom: `0.5px solid ${paper.rule}`,
      display: "flex",
      gap: 18,
      alignItems: "center",
      flexWrap: "wrap",
      background: paper.panel,
      flexShrink: 0,
    }}>
      <button
        onClick={() => { setActiveTag(null); setFavOnly && setFavOnly(false); }}
        style={{
          ...bSmallcaps,
          background: "transparent", border: "none", cursor: "pointer", padding: 0,
          color: !activeTag && !favOnly ? accent : paper.ink,
          fontWeight: !activeTag && !favOnly ? 700 : 500,
        }}
      >
        All
      </button>
      {setFavOnly && (
        <button
          onClick={() => { setFavOnly(!favOnly); setActiveTag(null); }}
          style={{
            ...bSmallcaps,
            background: "transparent", border: "none", cursor: "pointer", padding: 0,
            color: favOnly ? accent : paper.ink,
            fontWeight: favOnly ? 700 : 500,
          }}
        >
          ★ Favorites
        </button>
      )}
      <span style={{ color: paper.soft }}>|</span>
      {TOP_TAGS.slice(0, 6).map(([t, c]) => (
        <button
          key={t}
          onClick={() => { setActiveTag(activeTag === t ? null : t); setFavOnly && setFavOnly(false); }}
          style={{
            ...bSmallcaps,
            background: "transparent", border: "none", cursor: "pointer", padding: 0,
            color: activeTag === t ? accent : paper.ink,
            fontWeight: activeTag === t ? 700 : 500,
          }}
        >
          {t} <span style={{ color: paper.muted, fontWeight: 400 }}>·{c}</span>
        </button>
      ))}

      {setCurrentLayout && (
        <div style={{ marginLeft: "auto", display: "flex", alignItems: "center", gap: 12 }}>
          <BSmallcaps style={{ color: paper.muted }}>Layout</BSmallcaps>
          <div style={{ display: "flex", gap: 0, border: `0.5px solid ${paper.rule}` }}>
            {layoutOpts.map(([id, label]) => (
              <button
                key={id}
                onClick={() => setCurrentLayout(id)}
                style={{
                  ...bSmallcaps,
                  background: currentLayout === id ? paper.ink : "transparent",
                  color: currentLayout === id ? paper.bg : paper.ink,
                  border: "none",
                  padding: "5px 10px",
                  cursor: "pointer",
                  fontWeight: currentLayout === id ? 700 : 500,
                }}
              >
                {label}
              </button>
            ))}
          </div>
        </div>
      )}

      {setQuery && (
        <input
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          placeholder="Search the archive…"
          style={{
            background: "transparent",
            border: "none",
            borderBottom: `0.5px solid ${paper.muted}`,
            padding: "4px 0",
            fontSize: 13,
            fontStyle: "italic",
            outline: "none",
            color: paper.ink,
            width: 200,
            fontFamily: B_FONT_SERIF,
          }}
        />
      )}
    </nav>
  );
}

function BColophon({ paper, leftText = "End of selection", rightText = "fin." }) {
  return (
    <div style={{
      marginTop: 36,
      padding: "16px 0",
      borderTop: `1px solid ${paper.ink}`,
      display: "flex",
      justifyContent: "space-between",
      ...bSmallcaps,
      color: paper.muted,
    }}>
      <span>{leftText}</span>
      <span>Compiled locally · ~/eterea/bookmarks.db</span>
      <span>{rightText}</span>
    </div>
  );
}

window.B_PAPERS = B_PAPERS;
window.bPaper = bPaper;
window.B_FONT_SERIF = B_FONT_SERIF;
window.B_FONT_MONO = B_FONT_MONO;
window.bSmallcaps = bSmallcaps;
window.BSmallcaps = BSmallcaps;
window.BMasthead = BMasthead;
window.BTagRail = BTagRail;
window.BColophon = BColophon;

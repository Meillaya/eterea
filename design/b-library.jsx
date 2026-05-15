// b-library.jsx — four library layouts that share one BookmarkEntry component
// and one click-to-expand inline-detail behavior.

const bExpandStyle = (paper) => ({
  margin: "0 -8px",
  padding: "16px 8px 0",
  borderTop: `0.5px dashed ${paper.rule}`,
  fontFamily: B_FONT_SERIF,
});

function BInlineDetail({ b, paper, accent }) {
  return (
    <div style={bExpandStyle(paper)}>
      <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr 1fr", gap: 18, marginBottom: 14 }}>
        <div>
          <BSmallcaps style={{ color: paper.muted }}>Tweeted</BSmallcaps>
          <div style={{ marginTop: 4, fontSize: 14 }}>{fmtDate(b.tweeted_at)}</div>
        </div>
        <div>
          <BSmallcaps style={{ color: paper.muted }}>Saved</BSmallcaps>
          <div style={{ marginTop: 4, fontSize: 14 }}>{fmtDate(b.saved_at)}</div>
        </div>
        <div>
          <BSmallcaps style={{ color: paper.muted }}>Reach</BSmallcaps>
          <div style={{ marginTop: 4, fontSize: 14 }}>{b.likes.toLocaleString()} likes</div>
        </div>
      </div>
      <div style={{ display: "flex", gap: 16, paddingTop: 8, borderTop: `0.5px solid ${paper.soft}` }}>
        <button style={bGhostBtn(paper, accent)}>★ Favorite</button>
        <button style={bGhostBtn(paper, accent)}>↗ Open on X</button>
        <button style={bGhostBtn(paper, accent)}>⎘ Copy text</button>
        <button style={{ ...bGhostBtn(paper, accent), marginLeft: "auto", color: paper.muted }}>Hide</button>
      </div>
    </div>
  );
}

function bGhostBtn(paper, accent) {
  return {
    ...bSmallcaps,
    background: "transparent",
    border: `0.5px solid ${paper.rule}`,
    padding: "6px 12px",
    cursor: "pointer",
    color: paper.ink,
    fontWeight: 600,
  };
}

// Standard editorial entry: №01  ·  byline  ·  body  ·  date
function BEntry({ b, idx, paper, accent, isLead = false, expanded, onToggle, onHover, onLeave, hovered }) {
  return (
    <article
      onClick={onToggle}
      onMouseEnter={onHover}
      onMouseLeave={onLeave}
      style={{
        display: "grid",
        gridTemplateColumns: "44px minmax(0,1fr) 140px",
        gap: 18,
        padding: "20px 0",
        borderTop: `0.5px solid ${paper.rule}`,
        alignItems: "start",
        cursor: "pointer",
        position: "relative",
        background: expanded ? paper.soft : "transparent",
      }}
    >
      <div style={{ ...bSmallcaps, paddingTop: 4, color: paper.muted }}>
        №{String(idx + 1).padStart(2, "0")}
      </div>
      <div style={{ minWidth: 0 }}>
        <div style={{ display: "flex", gap: 10, alignItems: "baseline", marginBottom: 8 }}>
          <span style={{ fontWeight: 600, fontSize: 14, fontStyle: "italic" }}>{b.name}</span>
          <span style={{ ...bSmallcaps, color: paper.muted }}>@{b.handle}</span>
          {hovered && !expanded && (
            <span style={{ ...bSmallcaps, color: accent, marginLeft: "auto" }}>click to expand →</span>
          )}
        </div>
        <p style={{
          margin: 0,
          fontSize: isLead ? 22 : 17,
          lineHeight: 1.45,
          fontWeight: 400,
          color: paper.ink,
          textWrap: "pretty",
        }}>
          {isLead && (
            <span style={{
              float: "left",
              fontFamily: B_FONT_SERIF,
              fontSize: 64,
              lineHeight: 0.85,
              marginRight: 8,
              marginTop: 4,
              color: accent,
              fontWeight: 600,
            }}>
              {b.content.charAt(0)}
            </span>
          )}
          {isLead ? b.content.slice(1) : b.content}
        </p>
        <div style={{ display: "flex", gap: 14, marginTop: 12, alignItems: "center", flexWrap: "wrap" }}>
          {b.tags.map((t) => (
            <span key={t} style={{ ...bSmallcaps, color: accent, fontWeight: 600 }}>{t}</span>
          ))}
        </div>
        {expanded && <BInlineDetail b={b} paper={paper} accent={accent} />}
      </div>
      <div style={{ textAlign: "right", ...bSmallcaps, color: paper.muted, paddingTop: 4 }}>
        <div>{fmtDate(b.tweeted_at).toUpperCase()}</div>
        <div style={{ marginTop: 6 }}>
          {b.is_favorite ? <span style={{ color: accent }}>★ favored</span> : <span>saved {fmtRel(b.saved_at)}</span>}
        </div>
      </div>
    </article>
  );
}

// ─── Layout 1: Issue (current direction, refined) ───────────
function BLayoutIssue({ items, paper, accent, expanded, setExpanded, hovered, setHovered, title }) {
  return (
    <>
      <div style={{ display: "grid", gridTemplateColumns: "44px minmax(0,1fr) 140px", gap: 18, alignItems: "baseline", paddingBottom: 8 }}>
        <div></div>
        <h2 style={{ margin: 0, fontSize: 30, fontWeight: 500, fontStyle: "italic", fontFamily: B_FONT_SERIF, color: paper.ink }}>{title}</h2>
        <div style={{ ...bSmallcaps, color: paper.muted, textAlign: "right" }}>{items.length} of {STATS.total}</div>
      </div>
      {items.map((b, i) => (
        <BEntry key={b.id} b={b} idx={i} paper={paper} accent={accent} isLead={i === 0}
          expanded={expanded === b.id} onToggle={() => setExpanded(expanded === b.id ? null : b.id)}
          hovered={hovered === b.id} onHover={() => setHovered(b.id)} onLeave={() => setHovered(null)} />
      ))}
      <BColophon paper={paper} />
    </>
  );
}

// ─── Layout 2: Front Page (mixed sizes, hero + columns) ───────────
function BLayoutFront({ items, paper, accent, expanded, setExpanded, hovered, setHovered }) {
  const [hero, second, third, ...rest] = items;
  if (!hero) return null;
  return (
    <>
      <div style={{ ...bSmallcaps, color: paper.muted, paddingBottom: 8, borderBottom: `1px solid ${paper.ink}`, marginBottom: 20 }}>
        Today's Front Page · {items.length} pieces
      </div>

      {/* Hero */}
      <article style={{ marginBottom: 28, paddingBottom: 24, borderBottom: `1px solid ${paper.ink}`, cursor: "pointer" }}
        onClick={() => setExpanded(expanded === hero.id ? null : hero.id)}>
        <BSmallcaps style={{ color: accent, fontWeight: 700 }}>The Lead · @{hero.handle}</BSmallcaps>
        <h2 style={{
          margin: "10px 0 12px",
          fontSize: 38,
          lineHeight: 1.1,
          fontWeight: 500,
          fontFamily: B_FONT_SERIF,
          fontStyle: "normal",
          color: paper.ink,
          letterSpacing: "-0.005em",
        }}>
          <span style={{
            float: "left",
            fontFamily: B_FONT_SERIF,
            fontSize: 96,
            lineHeight: 0.85,
            marginRight: 12,
            marginTop: 8,
            color: accent,
            fontWeight: 600,
            fontStyle: "italic",
          }}>
            {hero.content.charAt(0)}
          </span>
          {hero.content.slice(1)}
        </h2>
        <div style={{ display: "flex", gap: 14, alignItems: "center", color: paper.muted }}>
          <BSmallcaps>{hero.name}</BSmallcaps>
          <span style={{ color: paper.soft }}>·</span>
          <BSmallcaps>{fmtDate(hero.tweeted_at)}</BSmallcaps>
          <span style={{ color: paper.soft }}>·</span>
          {hero.tags.map(t => <BSmallcaps key={t} style={{ color: accent }}>{t}</BSmallcaps>)}
        </div>
        {expanded === hero.id && <BInlineDetail b={hero} paper={paper} accent={accent} />}
      </article>

      {/* Two-column secondaries */}
      <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 32, paddingBottom: 24, borderBottom: `1px solid ${paper.ink}`, marginBottom: 24 }}>
        {[second, third].filter(Boolean).map(b => (
          <article key={b.id} onClick={() => setExpanded(expanded === b.id ? null : b.id)} style={{ cursor: "pointer" }}>
            <BSmallcaps style={{ color: paper.muted }}>@{b.handle} · {fmtDate(b.tweeted_at)}</BSmallcaps>
            <p style={{
              margin: "6px 0 8px",
              fontSize: 20,
              lineHeight: 1.35,
              fontFamily: B_FONT_SERIF,
              color: paper.ink,
              fontWeight: 500,
            }}>
              {b.content}
            </p>
            <div style={{ display: "flex", gap: 10 }}>
              {b.tags.map(t => <BSmallcaps key={t} style={{ color: accent }}>{t}</BSmallcaps>)}
            </div>
            {expanded === b.id && <BInlineDetail b={b} paper={paper} accent={accent} />}
          </article>
        ))}
      </div>

      {/* Three-column index */}
      <BSmallcaps style={{ color: paper.ink }}>More from the archive</BSmallcaps>
      <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr 1fr", gap: 28, marginTop: 14 }}>
        {rest.slice(0, 9).map(b => (
          <article key={b.id} onClick={() => setExpanded(expanded === b.id ? null : b.id)}
            style={{ cursor: "pointer", paddingTop: 12, borderTop: `0.5px solid ${paper.rule}` }}>
            <BSmallcaps style={{ color: paper.muted }}>@{b.handle}</BSmallcaps>
            <p style={{ margin: "4px 0 8px", fontSize: 14, lineHeight: 1.5, color: paper.ink, fontFamily: B_FONT_SERIF }}>
              {b.content.length > 140 ? b.content.slice(0, 140) + "…" : b.content}
            </p>
            <BSmallcaps style={{ color: paper.muted }}>{fmtRel(b.tweeted_at)}</BSmallcaps>
          </article>
        ))}
      </div>
      <BColophon paper={paper} />
    </>
  );
}

// ─── Layout 3: Long-Read (single column, max focus) ───────────
function BLayoutLong({ items, paper, accent, expanded, setExpanded, title }) {
  return (
    <div style={{ maxWidth: 660, margin: "0 auto" }}>
      <BSmallcaps style={{ color: paper.muted }}>Reading list · {title}</BSmallcaps>
      <h2 style={{ margin: "8px 0 28px", fontSize: 36, fontWeight: 500, fontStyle: "italic", fontFamily: B_FONT_SERIF, color: paper.ink, lineHeight: 1.05 }}>
        Read in the order you saved.
      </h2>
      {items.map((b, i) => (
        <article key={b.id} style={{ marginBottom: 40, paddingBottom: 36, borderBottom: `0.5px solid ${paper.rule}`, cursor: "pointer" }}
          onClick={() => setExpanded(expanded === b.id ? null : b.id)}>
          <div style={{ display: "flex", alignItems: "baseline", gap: 12, marginBottom: 14 }}>
            <span style={{ ...bSmallcaps, color: paper.muted, fontSize: 12 }}>№{String(i + 1).padStart(2, "0")}</span>
            <span style={{ fontStyle: "italic", fontSize: 15, color: paper.ink }}>{b.name}</span>
            <BSmallcaps style={{ color: paper.muted }}>@{b.handle}</BSmallcaps>
            <BSmallcaps style={{ color: paper.muted, marginLeft: "auto" }}>{fmtDate(b.tweeted_at)}</BSmallcaps>
          </div>
          <p style={{ margin: 0, fontSize: 21, lineHeight: 1.55, fontFamily: B_FONT_SERIF, color: paper.ink, textWrap: "pretty" }}>
            {i === 0 && (
              <span style={{ float: "left", fontFamily: B_FONT_SERIF, fontSize: 80, lineHeight: 0.85, marginRight: 10, marginTop: 6, color: accent, fontWeight: 600, fontStyle: "italic" }}>
                {b.content.charAt(0)}
              </span>
            )}
            {i === 0 ? b.content.slice(1) : b.content}
          </p>
          <div style={{ display: "flex", gap: 14, marginTop: 16, alignItems: "center" }}>
            {b.tags.map(t => <BSmallcaps key={t} style={{ color: accent }}>{t}</BSmallcaps>)}
            {b.is_favorite && <BSmallcaps style={{ color: accent, marginLeft: "auto" }}>★ favored</BSmallcaps>}
          </div>
          {expanded === b.id && <BInlineDetail b={b} paper={paper} accent={accent} />}
        </article>
      ))}
      <BColophon paper={paper} />
    </div>
  );
}

// ─── Layout 4: Two-up Spread (book/zine feel) ───────────
function BLayoutSpread({ items, paper, accent, expanded, setExpanded, title }) {
  // Pair items so they sit like book spreads.
  const pairs = [];
  for (let i = 0; i < items.length; i += 2) pairs.push([items[i], items[i + 1]]);

  return (
    <>
      <div style={{ display: "flex", justifyContent: "space-between", alignItems: "baseline", marginBottom: 20, paddingBottom: 8, borderBottom: `1px solid ${paper.ink}` }}>
        <h2 style={{ margin: 0, fontSize: 28, fontWeight: 500, fontStyle: "italic", fontFamily: B_FONT_SERIF, color: paper.ink }}>{title}</h2>
        <BSmallcaps style={{ color: paper.muted }}>Spread view · {items.length} entries</BSmallcaps>
      </div>
      {pairs.map(([l, r], pageIdx) => (
        <div key={pageIdx} style={{
          display: "grid",
          gridTemplateColumns: "1fr 1fr",
          gap: 0,
          padding: "32px 0",
          borderBottom: `0.5px solid ${paper.rule}`,
          position: "relative",
        }}>
          {/* Spine */}
          <div style={{ position: "absolute", top: 24, bottom: 24, left: "50%", width: 1, background: paper.rule }} />

          {[l, r].filter(Boolean).map((b, side) => (
            <article key={b.id}
              onClick={() => setExpanded(expanded === b.id ? null : b.id)}
              style={{
                padding: side === 0 ? "0 28px 0 0" : "0 0 0 28px",
                cursor: "pointer",
                position: "relative",
              }}>
              <BSmallcaps style={{ color: paper.muted }}>
                p. {pageIdx * 2 + side + 1} · @{b.handle}
              </BSmallcaps>
              <p style={{
                margin: "12px 0 14px",
                fontSize: 18,
                lineHeight: 1.55,
                fontFamily: B_FONT_SERIF,
                color: paper.ink,
                textWrap: "pretty",
              }}>
                {pageIdx === 0 && side === 0 && (
                  <span style={{ float: "left", fontFamily: B_FONT_SERIF, fontSize: 60, lineHeight: 0.85, marginRight: 8, marginTop: 4, color: accent, fontWeight: 600, fontStyle: "italic" }}>
                    {b.content.charAt(0)}
                  </span>
                )}
                {pageIdx === 0 && side === 0 ? b.content.slice(1) : b.content}
              </p>
              <div style={{ display: "flex", gap: 12, alignItems: "center" }}>
                <BSmallcaps style={{ color: paper.muted }}>{fmtDate(b.tweeted_at)}</BSmallcaps>
                {b.tags.slice(0, 2).map(t => <BSmallcaps key={t} style={{ color: accent }}>{t}</BSmallcaps>)}
              </div>
              {expanded === b.id && <BInlineDetail b={b} paper={paper} accent={accent} />}
            </article>
          ))}
        </div>
      ))}
      <BColophon paper={paper} />
    </>
  );
}

window.BInlineDetail = BInlineDetail;
window.bGhostBtn = bGhostBtn;
window.BEntry = BEntry;
window.BLayoutIssue = BLayoutIssue;
window.BLayoutFront = BLayoutFront;
window.BLayoutLong = BLayoutLong;
window.BLayoutSpread = BLayoutSpread;

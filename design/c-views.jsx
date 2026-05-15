// c-views.jsx — library views: table, tree, dashboard, graph, calendar.

function CTableView({ theme, font, density, items, selected, multi, accent, onSelect, onToggleMulti }) {
  const d = cDensity(density);
  return (
    <div style={{ display: "flex", flexDirection: "column", height: "100%", overflow: "hidden" }}>
      <div style={{
        display: "grid",
        gridTemplateColumns: "20px 36px 130px 76px minmax(0,1fr) 130px 28px",
        gap: 12,
        padding: "6px 14px",
        borderBottom: `1px solid ${theme.surface0}`,
        background: theme.mantle,
        color: theme.overlay0, fontSize: 10.5,
        textTransform: "uppercase", letterSpacing: "0.1em",
        flexShrink: 0,
      }}>
        <span></span><span>idx</span><span>author</span><span>when</span><span>content</span><span>tags</span><span>★</span>
      </div>
      <div style={{ overflow: "auto", flex: 1 }}>
        {items.map((b, i) => {
          const isSel = i === selected;
          const isMul = multi.has(b.id);
          return (
            <div key={b.id}
              onClick={() => onSelect(i)}
              style={{
                display: "grid",
                gridTemplateColumns: "20px 36px 130px 76px minmax(0,1fr) 130px 28px",
                gap: 12,
                padding: d.row,
                background: isSel ? theme.surface0 : isMul ? `${theme.peach}11` : "transparent",
                borderLeft: `2px solid ${isSel ? accent : isMul ? theme.peach : "transparent"}`,
                cursor: "pointer",
                fontSize: d.fs,
                lineHeight: d.line,
              }}>
              <span style={{ color: isMul ? theme.peach : theme.overlay0 }}>{isMul ? "●" : " "}</span>
              <span style={{ color: theme.overlay0 }}>{String(i + 1).padStart(3, "0")}</span>
              <span style={{ color: theme.green, overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>@{b.handle}</span>
              <span style={{ color: theme.overlay1 }}>{fmtRel(b.tweeted_at)}</span>
              <span style={{ overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap", color: theme.text }}>{b.content}</span>
              <span style={{ color: accent, fontSize: 11, overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
                {b.tags.map(t => `#${t}`).join(" ")}
              </span>
              <span style={{ color: b.is_favorite ? theme.yellow : theme.overlay0, textAlign: "center" }}>{b.is_favorite ? "★" : "·"}</span>
            </div>
          );
        })}
      </div>
    </div>
  );
}

function CTreeView({ theme, items, accent, selected, onSelect }) {
  const byAuthor = {};
  items.forEach(b => { (byAuthor[b.handle] ||= []).push(b); });
  const groups = Object.entries(byAuthor).sort((a, b) => b[1].length - a[1].length);
  let idx = -1;
  return (
    <div style={{ overflow: "auto", height: "100%", padding: "8px 14px" }}>
      <div style={{ color: theme.overlay0, fontSize: 11, marginBottom: 6 }}>~/eterea/library/by-author/</div>
      {groups.map(([author, list]) => (
        <div key={author}>
          <div style={{ color: theme.peach, padding: "4px 0" }}>
            ▾ <span style={{ color: theme.green }}>@{author}</span>{" "}
            <span style={{ color: theme.overlay0 }}>({list.length})</span>
          </div>
          {list.map(b => {
            idx++;
            const isSel = idx === selected;
            return (
              <div key={b.id}
                onClick={() => onSelect(idx)}
                style={{
                  display: "grid",
                  gridTemplateColumns: "20px 80px minmax(0,1fr) 100px",
                  gap: 10,
                  padding: "3px 0 3px 24px",
                  background: isSel ? theme.surface0 : "transparent",
                  borderLeft: `2px solid ${isSel ? accent : "transparent"}`,
                  marginLeft: -16, paddingLeft: 36,
                  cursor: "pointer",
                  fontSize: 12,
                }}>
                <span style={{ color: theme.overlay0 }}>├─</span>
                <span style={{ color: theme.overlay1 }}>{fmtRel(b.tweeted_at)}</span>
                <span style={{ color: theme.text, overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>{b.content}</span>
                <span style={{ color: accent, fontSize: 11, overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
                  {b.tags.map(t => `#${t}`).join(" ")}
                </span>
              </div>
            );
          })}
        </div>
      ))}
    </div>
  );
}

function CDashboardView({ theme, items, accent }) {
  const series = [3, 5, 4, 8, 6, 9, 7, 12, 10, 14, 11, 13, 16, 12];
  const tagBars = TOP_TAGS;
  const maxTag = Math.max(...tagBars.map(t => t[1]));
  return (
    <div style={{ overflow: "auto", height: "100%", padding: 16, display: "grid", gridTemplateColumns: "1fr 1fr", gap: 16 }}>
      {[
        ["total entries",  STATS.total, theme.blue, series],
        ["unique authors", STATS.authors, theme.green, series.map(v => v * 0.6).map(Math.round)],
        ["★ favorites",    STATS.favorites, theme.yellow, series.map(v => v * 0.4).map(Math.round)],
        ["this week",      STATS.this_week, theme.peach, [2,3,1,4,2,5,3]],
      ].map(([label, value, color, data]) => (
        <div key={label} style={{ background: theme.mantle, border: `1px solid ${theme.surface0}`, padding: 14 }}>
          <div style={{ color: theme.overlay0, fontSize: 11, textTransform: "uppercase", letterSpacing: "0.1em" }}>── {label} ──</div>
          <div style={{ display: "flex", alignItems: "flex-end", justifyContent: "space-between", marginTop: 8 }}>
            <span style={{ fontSize: 28, color: theme.text, fontWeight: 600 }}>{value}</span>
            <CSpark values={data} color={color} width={120} height={28} />
          </div>
        </div>
      ))}

      {/* Top tags */}
      <div style={{ background: theme.mantle, border: `1px solid ${theme.surface0}`, padding: 14, gridColumn: "span 2" }}>
        <div style={{ color: theme.overlay0, fontSize: 11, textTransform: "uppercase", letterSpacing: "0.1em", marginBottom: 10 }}>── top tags ──</div>
        {tagBars.map(([t, c]) => (
          <div key={t} style={{ display: "grid", gridTemplateColumns: "100px 1fr 40px", gap: 10, padding: "3px 0", alignItems: "center" }}>
            <span style={{ color: accent, fontSize: 12 }}>#{t}</span>
            <div style={{ height: 6, background: theme.surface0, borderRadius: 1, overflow: "hidden" }}>
              <div style={{ height: "100%", width: `${(c / maxTag) * 100}%`, background: accent }} />
            </div>
            <span style={{ color: theme.overlay1, fontSize: 11, textAlign: "right" }}>{c}</span>
          </div>
        ))}
      </div>

      {/* Recent activity */}
      <div style={{ background: theme.mantle, border: `1px solid ${theme.surface0}`, padding: 14, gridColumn: "span 2" }}>
        <div style={{ color: theme.overlay0, fontSize: 11, textTransform: "uppercase", letterSpacing: "0.1em", marginBottom: 10 }}>── recent saves ──</div>
        {items.slice(0, 5).map(b => (
          <div key={b.id} style={{ display: "grid", gridTemplateColumns: "120px 80px minmax(0,1fr)", gap: 12, padding: "3px 0", fontSize: 12 }}>
            <span style={{ color: theme.green }}>@{b.handle}</span>
            <span style={{ color: theme.overlay1 }}>{fmtRel(b.saved_at)}</span>
            <span style={{ color: theme.text, overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>{b.content}</span>
          </div>
        ))}
      </div>
    </div>
  );
}

function CGraphView({ theme, accent }) {
  // Simple SVG node graph of tag relationships
  const nodes = [
    { id: "rust", x: 0.5, y: 0.4, r: 30, c: theme.peach },
    { id: "performance", x: 0.7, y: 0.55, r: 22, c: theme.peach },
    { id: "tools", x: 0.35, y: 0.6, r: 18, c: theme.blue },
    { id: "systems", x: 0.6, y: 0.25, r: 20, c: theme.green },
    { id: "design", x: 0.85, y: 0.3, r: 24, c: theme.mauve },
    { id: "ai", x: 0.2, y: 0.35, r: 22, c: theme.yellow },
    { id: "ml", x: 0.12, y: 0.5, r: 18, c: theme.yellow },
    { id: "engineering", x: 0.5, y: 0.75, r: 19, c: theme.teal },
    { id: "c", x: 0.78, y: 0.7, r: 16, c: theme.peach },
    { id: "ux", x: 0.92, y: 0.5, r: 14, c: theme.mauve },
  ];
  const edges = [
    ["rust", "performance"], ["rust", "tools"], ["rust", "systems"], ["rust", "c"],
    ["systems", "performance"], ["ai", "ml"], ["design", "ux"], ["engineering", "tools"],
    ["performance", "c"], ["systems", "engineering"],
  ];
  const W = 800, H = 480;
  return (
    <div style={{ height: "100%", padding: 16, overflow: "hidden" }}>
      <div style={{ color: theme.overlay0, fontSize: 11, textTransform: "uppercase", letterSpacing: "0.1em", marginBottom: 8 }}>── tag graph · co-occurrence ──</div>
      <svg viewBox={`0 0 ${W} ${H}`} style={{ width: "100%", height: "calc(100% - 24px)", background: theme.mantle, border: `1px solid ${theme.surface0}` }}>
        {edges.map(([a, b], i) => {
          const na = nodes.find(n => n.id === a), nb = nodes.find(n => n.id === b);
          return <line key={i} x1={na.x * W} y1={na.y * H} x2={nb.x * W} y2={nb.y * H} stroke={theme.surface1} strokeWidth="1" />;
        })}
        {nodes.map(n => (
          <g key={n.id}>
            <circle cx={n.x * W} cy={n.y * H} r={n.r} fill={`${n.c}33`} stroke={n.c} strokeWidth="1.5" />
            <text x={n.x * W} y={n.y * H + 4} textAnchor="middle" fill={n.c} fontFamily="inherit" fontSize="13" fontWeight="600">#{n.id}</text>
          </g>
        ))}
      </svg>
    </div>
  );
}

function CCalendarView({ theme, accent }) {
  // 12 weeks × 7 days, deterministic pseudo-random density
  const weeks = 14;
  const days = 7;
  const density = (w, d) => {
    const v = (w * 7 + d * 3 + 13) % 11;
    return v < 4 ? 0 : v < 7 ? 1 : v < 9 ? 2 : 3;
  };
  const colors = [theme.surface0, `${accent}55`, `${accent}aa`, accent];
  return (
    <div style={{ overflow: "auto", height: "100%", padding: 16 }}>
      <div style={{ color: theme.overlay0, fontSize: 11, textTransform: "uppercase", letterSpacing: "0.1em", marginBottom: 10 }}>── saves heatmap · last {weeks} weeks ──</div>
      <div style={{ display: "flex", gap: 4 }}>
        {Array.from({ length: weeks }, (_, w) => (
          <div key={w} style={{ display: "flex", flexDirection: "column", gap: 4 }}>
            {Array.from({ length: days }, (_, d) => (
              <div key={d} style={{
                width: 16, height: 16,
                background: colors[density(w, d)],
                border: `1px solid ${theme.surface0}`,
                borderRadius: 2,
              }} />
            ))}
          </div>
        ))}
      </div>
      <div style={{ display: "flex", alignItems: "center", gap: 6, marginTop: 14, color: theme.overlay1, fontSize: 11 }}>
        less
        {colors.map((c, i) => <div key={i} style={{ width: 12, height: 12, background: c, border: `1px solid ${theme.surface0}`, borderRadius: 2 }} />)}
        more
      </div>

      <div style={{ marginTop: 24, color: theme.overlay0, fontSize: 11, textTransform: "uppercase", letterSpacing: "0.1em", marginBottom: 10 }}>── by hour ──</div>
      <div style={{ display: "flex", alignItems: "flex-end", gap: 2, height: 80 }}>
        {Array.from({ length: 24 }, (_, h) => {
          const v = Math.abs(Math.sin(h * 0.5)) * 0.7 + 0.2 + ((h * 13) % 5) * 0.05;
          return (
            <div key={h} style={{ flex: 1, height: `${v * 100}%`, background: accent, opacity: 0.7 }} title={`${h}:00`} />
          );
        })}
      </div>
      <div style={{ display: "flex", justifyContent: "space-between", marginTop: 4, color: theme.overlay1, fontSize: 10 }}>
        <span>00:00</span><span>06:00</span><span>12:00</span><span>18:00</span><span>23:00</span>
      </div>
    </div>
  );
}

window.CTableView = CTableView;
window.CTreeView = CTreeView;
window.CDashboardView = CDashboardView;
window.CGraphView = CGraphView;
window.CCalendarView = CCalendarView;

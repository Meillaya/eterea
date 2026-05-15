// variation-a.jsx — "Refined Calm"
// Tightened version of the existing direction. Same warm-orange + serif-italic
// DNA, but better hierarchy: smaller hero, denser top tags, real bookmark cards
// with avatars as colored blocks, distinct Focus/Grid/List treatments.

const VarA_styles = {
  shell: {
    width: "100%",
    height: "100%",
    background: "radial-gradient(circle at 30% 0%, #1a1f2c 0%, #0a0c14 55%)",
    color: "#f4eee8",
    fontFamily: 'ui-sans-serif, system-ui, -apple-system, "Segoe UI", sans-serif',
    display: "grid",
    gridTemplateColumns: "264px minmax(0, 1fr)",
    gap: 18,
    padding: 16,
    overflow: "hidden",
    position: "relative",
  },
  panel: {
    background: "linear-gradient(180deg, rgba(255,255,255,0.018), rgba(255,255,255,0.008))",
    border: "1px solid rgba(255,255,255,0.07)",
    borderRadius: 24,
    boxShadow: "0 18px 48px rgba(0,0,0,0.25)",
  },
  eyebrow: {
    margin: 0,
    textTransform: "uppercase",
    letterSpacing: "0.18em",
    fontSize: 10.5,
    color: "#73727d",
    fontWeight: 500,
  },
  serif: {
    fontFamily: '"Iowan Old Style", "Palatino Linotype", "Book Antiqua", Palatino, Georgia, serif',
    fontStyle: "italic",
    fontWeight: 500,
  },
  muted: { color: "#9c9aa4", lineHeight: 1.6, margin: 0 },
};

function VarA_NavLink({ active, label, sub, onClick }) {
  return (
    <button
      onClick={onClick}
      style={{
        width: "100%",
        display: "flex",
        justifyContent: "space-between",
        alignItems: "baseline",
        background: active ? "rgba(255, 152, 97, 0.13)" : "rgba(255,255,255,0.02)",
        border: `1px solid ${active ? "rgba(255, 152, 97, 0.35)" : "rgba(255,255,255,0.07)"}`,
        color: "#f4eee8",
        borderRadius: 16,
        padding: "11px 14px",
        marginTop: 8,
        cursor: "pointer",
        textAlign: "left",
      }}
    >
      <span style={{ fontWeight: 600, fontSize: 13.5 }}>{label}</span>
      <small style={{ color: "#9c9aa4", fontSize: 11 }}>{sub}</small>
    </button>
  );
}

function VarA_TagPill({ tag, count, active, onClick }) {
  return (
    <button
      onClick={onClick}
      style={{
        width: "100%",
        display: "flex",
        justifyContent: "space-between",
        alignItems: "baseline",
        background: active ? "rgba(255, 152, 97, 0.13)" : "rgba(255,255,255,0.02)",
        border: `1px solid ${active ? "rgba(255, 152, 97, 0.35)" : "rgba(255,255,255,0.07)"}`,
        color: "#f4eee8",
        borderRadius: 999,
        padding: "8px 14px",
        marginTop: 6,
        cursor: "pointer",
        fontSize: 12.5,
      }}
    >
      <span style={{ fontWeight: 600 }}>#{tag}</span>
      <small style={{ color: "#9c9aa4" }}>{count}</small>
    </button>
  );
}

function VarA_Avatar({ hue, name, size = 40 }) {
  const initial = name.charAt(0).toUpperCase();
  return (
    <div
      style={{
        width: size,
        height: size,
        borderRadius: 12,
        background: `linear-gradient(135deg, oklch(0.55 0.14 ${hue}), oklch(0.42 0.10 ${hue + 30}))`,
        display: "grid",
        placeItems: "center",
        color: "#f4eee8",
        fontWeight: 600,
        fontSize: size * 0.45,
        flex: "none",
      }}
    >
      {initial}
    </div>
  );
}

function VarA_Card({ b, layout, accent }) {
  if (layout === "List") {
    return (
      <article
        style={{
          ...VarA_styles.panel,
          padding: "12px 16px",
          display: "grid",
          gridTemplateColumns: "auto minmax(0, 1fr) auto",
          gap: 14,
          alignItems: "center",
          borderRadius: 16,
        }}
      >
        <VarA_Avatar hue={b.avatar_hue} name={b.name} size={32} />
        <div style={{ minWidth: 0 }}>
          <div style={{ display: "flex", gap: 8, alignItems: "baseline", color: "#9c9aa4", fontSize: 12 }}>
            <strong style={{ color: "#f4eee8" }}>@{b.handle}</strong>
            <span>·</span>
            <span>{fmtRel(b.tweeted_at)}</span>
          </div>
          <p
            style={{
              margin: "4px 0 0",
              fontSize: 13.5,
              lineHeight: 1.5,
              overflow: "hidden",
              textOverflow: "ellipsis",
              whiteSpace: "nowrap",
              color: "#d8d3cd",
            }}
          >
            {b.content}
          </p>
        </div>
        <button
          style={{
            background: "transparent",
            border: "none",
            color: b.is_favorite ? accent : "#73727d",
            fontSize: 16,
            cursor: "pointer",
          }}
        >
          {b.is_favorite ? "★" : "☆"}
        </button>
      </article>
    );
  }

  const isFocus = layout === "Focus";
  return (
    <article
      style={{
        ...VarA_styles.panel,
        padding: isFocus ? 22 : 18,
        borderRadius: 22,
        display: "flex",
        flexDirection: "column",
        gap: 10,
      }}
    >
      <header style={{ display: "flex", gap: 12, alignItems: "center" }}>
        <VarA_Avatar hue={b.avatar_hue} name={b.name} size={isFocus ? 44 : 38} />
        <div style={{ minWidth: 0, flex: 1 }}>
          <div style={{ fontWeight: 600, fontSize: 14 }}>{b.name}</div>
          <div style={{ color: "#9c9aa4", fontSize: 12 }}>@{b.handle} · {fmtRel(b.tweeted_at)}</div>
        </div>
        <span style={{ color: b.is_favorite ? accent : "#5a5963", fontSize: 16 }}>
          {b.is_favorite ? "★" : "☆"}
        </span>
      </header>
      <p
        style={{
          margin: 0,
          fontSize: isFocus ? 16 : 14,
          lineHeight: isFocus ? 1.65 : 1.55,
          color: "#e8e3dc",
          textWrap: "pretty",
        }}
      >
        {b.content}
      </p>
      <footer style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginTop: 4 }}>
        <div style={{ display: "flex", gap: 6, flexWrap: "wrap" }}>
          {b.tags.map((t) => (
            <span
              key={t}
              style={{
                fontSize: 11,
                padding: "3px 9px",
                borderRadius: 999,
                background: "rgba(255,255,255,0.04)",
                border: "1px solid rgba(255,255,255,0.06)",
                color: "#b8b4ac",
              }}
            >
              #{t}
            </span>
          ))}
        </div>
        <span style={{ color: "#73727d", fontSize: 11 }}>
          {b.likes.toLocaleString()} likes
        </span>
      </footer>
    </article>
  );
}

function VariationA({ accent = "#ff9861", density = "regular" }) {
  const [layout, setLayout] = React.useState("Focus");
  const [activeTag, setActiveTag] = React.useState(null);
  const [favOnly, setFavOnly] = React.useState(false);
  const [view, setView] = React.useState("library");

  const filtered = BOOKMARKS.filter((b) => {
    if (favOnly && !b.is_favorite) return false;
    if (activeTag && !b.tags.includes(activeTag)) return false;
    if (view === "favorites" && !b.is_favorite) return false;
    return true;
  });

  const gap = density === "compact" ? 10 : density === "comfy" ? 18 : 14;
  const feedStyle =
    layout === "Focus"
      ? { display: "grid", gridTemplateColumns: "1fr", gap, maxWidth: 720, margin: "0 auto" }
      : layout === "Grid"
      ? { display: "grid", gridTemplateColumns: "repeat(auto-fit, minmax(280px, 1fr))", gap }
      : { display: "flex", flexDirection: "column", gap: 8 };

  return (
    <div style={VarA_styles.shell}>
      {/* Left rail */}
      <aside style={{ display: "flex", flexDirection: "column", gap: 14, overflow: "hidden" }}>
        <div style={{ ...VarA_styles.panel, padding: 18 }}>
          <p style={VarA_styles.eyebrow}>Local-first archive</p>
          <h1 style={{ ...VarA_styles.serif, fontSize: 30, margin: "6px 0 10px" }}>Eterea</h1>
          <p style={{ ...VarA_styles.muted, fontSize: 12.5 }}>
            A calm reading room for saved tweets — fast to open, quiet to browse.
          </p>
          <button
            style={{
              marginTop: 14,
              width: "100%",
              padding: "11px 14px",
              borderRadius: 999,
              border: "none",
              background: `linear-gradient(90deg, ${accent}, oklch(0.78 0.12 70))`,
              color: "#1c1109",
              fontWeight: 600,
              fontSize: 13,
              cursor: "pointer",
            }}
          >
            Import bookmarks
          </button>
        </div>

        <div style={{ ...VarA_styles.panel, padding: 16 }}>
          <p style={VarA_styles.eyebrow}>Navigate</p>
          <VarA_NavLink active={view === "library" && !favOnly} label="Library" sub="412" onClick={() => { setView("library"); setFavOnly(false); setActiveTag(null); }} />
          <VarA_NavLink active={view === "favorites" || favOnly} label="Favorites" sub="47" onClick={() => { setView("favorites"); setActiveTag(null); }} />
          <VarA_NavLink active={false} label="Authors" sub="184" onClick={() => {}} />
        </div>

        <div style={{ ...VarA_styles.panel, padding: 16, overflow: "hidden", flex: 1, minHeight: 0 }}>
          <p style={VarA_styles.eyebrow}>Top tags</p>
          <div style={{ marginTop: 4 }}>
            {TOP_TAGS.map(([t, c]) => (
              <VarA_TagPill
                key={t}
                tag={t}
                count={c}
                active={activeTag === t}
                onClick={() => setActiveTag(activeTag === t ? null : t)}
              />
            ))}
          </div>
        </div>
      </aside>

      {/* Main column */}
      <main style={{ display: "flex", flexDirection: "column", gap: 14, overflow: "hidden", minWidth: 0 }}>
        <section style={{ ...VarA_styles.panel, padding: "20px 24px" }}>
          <div style={{ display: "flex", gap: 8, marginBottom: 10 }}>
            <span style={{ fontSize: 11, padding: "4px 10px", borderRadius: 999, background: "rgba(255,255,255,0.04)", border: "1px solid rgba(255,255,255,0.07)", color: "#d4cfca" }}>Library</span>
            <span style={{ fontSize: 11, padding: "4px 10px", borderRadius: 999, background: "rgba(255,255,255,0.04)", border: "1px solid rgba(255,255,255,0.07)", color: "#d4cfca" }}>local-first</span>
            <span style={{ fontSize: 11, padding: "4px 10px", borderRadius: 999, background: "rgba(255,255,255,0.04)", border: "1px solid rgba(255,255,255,0.07)", color: "#d4cfca" }}>{STATS.total} saved</span>
          </div>
          <h2 style={{ ...VarA_styles.serif, fontSize: 38, margin: "4px 0 10px", lineHeight: 1.05, maxWidth: 720 }}>
            Read what you saved without the rest of the internet shouting over it.
          </h2>
          <p style={{ ...VarA_styles.muted, fontSize: 13, maxWidth: 600 }}>
            {STATS.total} bookmarks · {STATS.authors} authors · {STATS.this_week} added this week.
          </p>

          <div style={{ display: "flex", gap: 10, marginTop: 16, alignItems: "center", flexWrap: "wrap" }}>
            <div style={{ flex: "1 1 320px", display: "flex", alignItems: "center", gap: 10, padding: "10px 14px", background: "rgba(7, 9, 16, 0.55)", border: "1px solid rgba(255,255,255,0.07)", borderRadius: 14 }}>
              <span style={{ color: "#73727d", fontSize: 13 }}>⌕</span>
              <input
                placeholder="Search by text, author, or tag"
                style={{ background: "transparent", border: "none", outline: "none", color: "#f4eee8", flex: 1, fontSize: 13 }}
              />
              <span style={{ fontSize: 10.5, color: "#73727d", padding: "2px 6px", border: "1px solid rgba(255,255,255,0.1)", borderRadius: 4 }}>⌘K</span>
            </div>
            <div style={{ display: "flex", gap: 4, padding: 4, background: "rgba(7, 9, 16, 0.55)", border: "1px solid rgba(255,255,255,0.07)", borderRadius: 12 }}>
              {["Focus", "Grid", "List"].map((m) => (
                <button
                  key={m}
                  onClick={() => setLayout(m)}
                  style={{
                    padding: "7px 14px",
                    borderRadius: 9,
                    border: "none",
                    background: layout === m ? "rgba(255, 152, 97, 0.16)" : "transparent",
                    color: layout === m ? accent : "#9c9aa4",
                    fontWeight: layout === m ? 600 : 500,
                    fontSize: 12.5,
                    cursor: "pointer",
                  }}
                >
                  {m}
                </button>
              ))}
            </div>
            <button
              onClick={() => setFavOnly(!favOnly)}
              style={{
                padding: "8px 14px",
                borderRadius: 999,
                border: `1px solid ${favOnly ? "rgba(255, 152, 97, 0.35)" : "rgba(255,255,255,0.07)"}`,
                background: favOnly ? "rgba(255, 152, 97, 0.13)" : "rgba(255,255,255,0.02)",
                color: "#f4eee8",
                fontSize: 12.5,
                cursor: "pointer",
              }}
            >
              ★ Favorites only
            </button>
          </div>
        </section>

        <section style={{ ...VarA_styles.panel, padding: "20px 24px", flex: 1, minHeight: 0, display: "flex", flexDirection: "column", overflow: "hidden" }}>
          <div style={{ display: "flex", justifyContent: "space-between", alignItems: "baseline", marginBottom: 14 }}>
            <div>
              <p style={VarA_styles.eyebrow}>Reading feed</p>
              <h3 style={{ ...VarA_styles.serif, fontSize: 22, margin: "4px 0 0" }}>
                {view === "favorites" ? "Favorites" : activeTag ? `#${activeTag}` : "Library"}
              </h3>
            </div>
            <span style={{ ...VarA_styles.muted, fontSize: 12 }}>
              Showing {filtered.length} of {STATS.total}
            </span>
          </div>

          <div style={{ overflow: "auto", flex: 1, paddingRight: 4 }}>
            <div style={feedStyle}>
              {filtered.map((b) => (
                <VarA_Card key={b.id} b={b} layout={layout} accent={accent} />
              ))}
            </div>
          </div>
        </section>
      </main>

      {/* Status bar */}
      <div
        style={{
          position: "absolute",
          left: 28,
          right: 28,
          bottom: 14,
          padding: "9px 16px",
          borderRadius: 999,
          background: "rgba(12, 15, 22, 0.92)",
          border: "1px solid rgba(255,255,255,0.07)",
          color: "#9c9aa4",
          backdropFilter: "blur(14px)",
          fontSize: 11.5,
          display: "flex",
          justifyContent: "space-between",
        }}
      >
        <span>Archive ready — {STATS.total} bookmarks loaded.</span>
        <span>local · ~/eterea/bookmarks.db</span>
      </div>
    </div>
  );
}

window.VariationA = VariationA;

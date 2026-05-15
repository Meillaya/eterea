// b-app.jsx — wraps the library layouts with state, masthead, and tag rail.

function BLibraryApp({ accent, paperTone, density }) {
  const paper = bPaper(paperTone);
  const [layout, setLayout] = React.useState("issue");
  const [activeTag, setActiveTag] = React.useState(null);
  const [favOnly, setFavOnly] = React.useState(false);
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

  // j/k keyboard nav
  React.useEffect(() => {
    const onKey = (e) => {
      if (e.target.tagName === "INPUT") return;
      const ids = items.map(b => b.id);
      if (e.key === "j" || e.key === "ArrowDown") {
        const i = expanded ? ids.indexOf(expanded) : -1;
        setExpanded(ids[Math.min(ids.length - 1, i + 1)]);
        e.preventDefault();
      } else if (e.key === "k" || e.key === "ArrowUp") {
        const i = expanded ? ids.indexOf(expanded) : ids.length;
        setExpanded(ids[Math.max(0, i - 1)]);
        e.preventDefault();
      } else if (e.key === "Escape") {
        setExpanded(null);
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [items, expanded]);

  const padding = density === "compact" ? "16px 36px 50px" : density === "comfy" ? "28px 36px 60px" : "20px 36px 50px";

  const title = favOnly ? "Favorites" : activeTag ? `On "${activeTag}"` : query ? `"${query}"` : "The Library";

  const props = { items, paper, accent, expanded, setExpanded, hovered, setHovered, title };

  return (
    <div style={{
      width: "100%", height: "100%",
      background: paper.bg, color: paper.ink,
      fontFamily: B_FONT_SERIF,
      display: "flex", flexDirection: "column",
      overflow: "hidden", position: "relative",
    }}>
      <BMasthead paper={paper} />
      <BTagRail
        paper={paper} accent={accent}
        activeTag={activeTag} setActiveTag={setActiveTag}
        favOnly={favOnly} setFavOnly={setFavOnly}
        currentLayout={layout} setCurrentLayout={setLayout}
        query={query} setQuery={setQuery}
      />
      <div style={{ overflow: "auto", flex: 1, padding, position: "relative" }}>
        {layout === "issue"  && <BLayoutIssue {...props} />}
        {layout === "front"  && <BLayoutFront {...props} />}
        {layout === "long"   && <BLayoutLong {...props} />}
        {layout === "spread" && <BLayoutSpread {...props} />}
      </div>
      {/* Keyboard hint */}
      <div style={{
        position: "absolute", left: 36, bottom: 14,
        ...bSmallcaps, color: paper.muted,
      }}>
        j/k navigate · / search · esc collapse · ↗ open
      </div>
    </div>
  );
}

window.BLibraryApp = BLibraryApp;

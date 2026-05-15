// c-chrome.jsx — shell, statusline, sidebar, command palette, keybindings overlay.

function CShell({ theme, font, mode, children, statusline, command, setCommand, onCommand, currentScreen, multiSelect }) {
  return (
    <div style={{
      width: "100%", height: "100%",
      background: theme.bg, color: theme.text,
      fontFamily: font, fontSize: 12.5, lineHeight: 1.5,
      display: "grid", gridTemplateRows: "auto 1fr auto",
      overflow: "hidden",
    }}>
      <CTopBar theme={theme} currentScreen={currentScreen} mode={mode} />
      <div style={{ overflow: "hidden", minHeight: 0 }}>{children}</div>
      <CStatusLine theme={theme} mode={mode} statusline={statusline} command={command} setCommand={setCommand} onCommand={onCommand} multiSelect={multiSelect} />
    </div>
  );
}

function CTopBar({ theme, currentScreen, mode }) {
  const tabs = [
    ["library",  "library"],
    ["authors",  "authors"],
    ["topics",   "topics"],
    ["search",   "search"],
    ["import",   "import"],
    ["settings", "settings"],
  ];
  return (
    <header style={{
      display: "flex", alignItems: "stretch",
      borderBottom: `1px solid ${theme.surface0}`,
      background: theme.mantle, height: 32,
    }}>
      <div style={{
        padding: "0 14px", display: "flex", alignItems: "center", gap: 10,
        background: theme.crust, borderRight: `1px solid ${theme.surface0}`,
      }}>
        <span style={{ color: theme.accent, fontWeight: 600 }}>◆</span>
        <span style={{ letterSpacing: "0.05em", fontWeight: 600 }}>eterea</span>
        <span style={{ color: theme.overlay0 }}>v0.1.0</span>
      </div>
      {tabs.map(([id, label], i) => (
        <a key={id} href="#" data-screen={id}
          style={{
            padding: "0 14px", display: "flex", alignItems: "center",
            color: currentScreen === id ? theme.accent : theme.subtext,
            background: currentScreen === id ? theme.bg : "transparent",
            borderRight: `1px solid ${theme.surface0}`,
            borderBottom: currentScreen === id ? `1px solid ${theme.bg}` : "none",
            marginBottom: currentScreen === id ? -1 : 0,
            textDecoration: "none",
            fontWeight: currentScreen === id ? 600 : 400,
          }}>
          <span style={{ color: theme.overlay0, marginRight: 6 }}>[{i + 1}]</span>
          {label}
        </a>
      ))}
      <div style={{ marginLeft: "auto", padding: "0 14px", display: "flex", alignItems: "center", gap: 14, color: theme.overlay1 }}>
        <span><CKbd theme={theme}>?</CKbd> help</span>
        <span><CKbd theme={theme}>Ctrl</CKbd>+<CKbd theme={theme}>P</CKbd> palette</span>
      </div>
    </header>
  );
}

function CStatusLine({ theme, mode, statusline, command, setCommand, onCommand, multiSelect }) {
  const modeColor = {
    NORMAL:  theme.blue,
    INSERT:  theme.green,
    VISUAL:  theme.peach,
    COMMAND: theme.mauve,
    SEARCH:  theme.yellow,
  }[mode] || theme.blue;

  if (mode === "COMMAND" || mode === "SEARCH") {
    return (
      <div style={{ display: "flex", alignItems: "stretch", borderTop: `1px solid ${theme.surface0}`, background: theme.mantle, height: 24 }}>
        <span style={{ background: modeColor, color: theme.crust, padding: "0 10px", display: "flex", alignItems: "center", fontWeight: 700, letterSpacing: "0.05em" }}>{mode}</span>
        <span style={{ padding: "0 8px", display: "flex", alignItems: "center", color: theme.text }}>{mode === "COMMAND" ? ":" : "/"}</span>
        <input
          autoFocus
          value={command}
          onChange={(e) => setCommand(e.target.value)}
          onKeyDown={(e) => {
            if (e.key === "Enter") onCommand("submit");
            if (e.key === "Escape") onCommand("cancel");
          }}
          style={{
            flex: 1, background: "transparent", border: "none", outline: "none",
            color: theme.text, fontFamily: "inherit", fontSize: 12.5,
          }}
        />
      </div>
    );
  }

  return (
    <div style={{
      display: "flex", alignItems: "stretch",
      borderTop: `1px solid ${theme.surface0}`,
      background: theme.mantle, height: 24, fontSize: 11.5,
    }}>
      <span style={{ background: modeColor, color: theme.crust, padding: "0 10px", display: "flex", alignItems: "center", fontWeight: 700, letterSpacing: "0.05em" }}>{mode}</span>
      {multiSelect > 0 && (
        <span style={{ background: theme.peach, color: theme.crust, padding: "0 8px", display: "flex", alignItems: "center", fontWeight: 700 }}>
          {multiSelect} selected
        </span>
      )}
      <span style={{ padding: "0 10px", display: "flex", alignItems: "center", color: theme.subtext }}>
        {statusline?.left || ""}
      </span>
      <span style={{ marginLeft: "auto", display: "flex", alignItems: "center", gap: 14, padding: "0 10px", color: theme.overlay1 }}>
        {statusline?.right || ""}
      </span>
    </div>
  );
}

// Command palette overlay (Ctrl-P)
function CPalette({ theme, query, setQuery, items, selected, setSelected, onPick, onClose }) {
  const filtered = fuzzyFilter(items, query);
  React.useEffect(() => {
    setSelected(0);
  }, [query]);
  return (
    <div onClick={onClose} style={{
      position: "absolute", inset: 0, background: "rgba(17,17,27,0.6)",
      display: "flex", justifyContent: "center", alignItems: "flex-start",
      paddingTop: 80, zIndex: 50,
    }}>
      <div onClick={(e) => e.stopPropagation()} style={{
        width: 600, background: theme.mantle,
        border: `1px solid ${theme.surface1}`,
        borderRadius: 4, overflow: "hidden",
        boxShadow: `0 16px 48px ${theme.crust}cc`,
      }}>
        <div style={{ padding: "10px 14px", borderBottom: `1px solid ${theme.surface0}`, display: "flex", alignItems: "center", gap: 8 }}>
          <span style={{ color: theme.accent }}>❯</span>
          <input
            autoFocus
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            onKeyDown={(e) => {
              if (e.key === "Escape") onClose();
              if (e.key === "Enter" && filtered[selected]) onPick(filtered[selected]);
              if (e.key === "ArrowDown" || (e.ctrlKey && e.key === "n")) { e.preventDefault(); setSelected(Math.min(filtered.length - 1, selected + 1)); }
              if (e.key === "ArrowUp" || (e.ctrlKey && e.key === "p")) { e.preventDefault(); setSelected(Math.max(0, selected - 1)); }
            }}
            placeholder="Type a command, file, or query..."
            style={{ flex: 1, background: "transparent", border: "none", outline: "none", color: theme.text, fontFamily: "inherit", fontSize: 13 }}
          />
          <span style={{ color: theme.overlay0, fontSize: 11 }}>{filtered.length} matches</span>
        </div>
        <div style={{ maxHeight: 360, overflow: "auto" }}>
          {filtered.slice(0, 12).map((it, i) => (
            <div key={it.id}
              onClick={() => onPick(it)}
              onMouseEnter={() => setSelected(i)}
              style={{
                padding: "7px 14px",
                background: i === selected ? theme.surface0 : "transparent",
                borderLeft: `2px solid ${i === selected ? theme.accent : "transparent"}`,
                cursor: "pointer",
                display: "flex", gap: 12, alignItems: "baseline",
              }}>
              <span style={{ color: theme.overlay0, fontSize: 11, width: 20 }}>{it.icon || "›"}</span>
              <span style={{ flex: 1, color: theme.text }}>
                {highlightMatch(it.label, query, theme.accent)}
              </span>
              <span style={{ color: theme.overlay1, fontSize: 11 }}>{it.sub}</span>
              {it.kbd && <CKbd theme={theme}>{it.kbd}</CKbd>}
            </div>
          ))}
          {filtered.length === 0 && (
            <div style={{ padding: "16px 14px", color: theme.overlay0, fontStyle: "italic" }}>no matches</div>
          )}
        </div>
        <div style={{ padding: "6px 14px", borderTop: `1px solid ${theme.surface0}`, color: theme.overlay1, fontSize: 11, display: "flex", gap: 16 }}>
          <span><CKbd theme={theme}>↵</CKbd> open</span>
          <span><CKbd theme={theme}>↑↓</CKbd> nav</span>
          <span><CKbd theme={theme}>Esc</CKbd> close</span>
        </div>
      </div>
    </div>
  );
}

function fuzzyFilter(items, query) {
  if (!query.trim()) return items;
  const q = query.toLowerCase();
  return items
    .map(it => {
      const hay = (it.label + " " + (it.sub || "") + " " + (it.keywords || "")).toLowerCase();
      let score = 0;
      let qi = 0;
      for (let i = 0; i < hay.length && qi < q.length; i++) {
        if (hay[i] === q[qi]) { score += 1; qi++; }
      }
      return qi === q.length ? { ...it, _score: score } : null;
    })
    .filter(Boolean)
    .sort((a, b) => b._score - a._score);
}

function highlightMatch(label, query, color) {
  if (!query.trim()) return label;
  const q = query.toLowerCase();
  const out = [];
  let qi = 0;
  for (let i = 0; i < label.length; i++) {
    if (qi < q.length && label[i].toLowerCase() === q[qi]) {
      out.push(<span key={i} style={{ color, fontWeight: 600 }}>{label[i]}</span>);
      qi++;
    } else {
      out.push(<React.Fragment key={i}>{label[i]}</React.Fragment>);
    }
  }
  return out;
}

// Keybinding overlay
function CKeybindings({ theme, onClose }) {
  const groups = [
    ["Movement", [
      ["j / ↓", "next entry"],
      ["k / ↑", "prev entry"],
      ["g g", "first entry"],
      ["G", "last entry"],
      ["Ctrl-d / Ctrl-u", "half-page down/up"],
    ]],
    ["Modes", [
      [": ", "command mode"],
      ["/", "search / filter"],
      ["v", "visual (multi-select)"],
      ["i", "insert (note edit)"],
      ["Esc", "back to NORMAL"],
    ]],
    ["Selection", [
      ["Space", "toggle select on entry"],
      ["a", "select all"],
      ["A", "deselect all"],
    ]],
    ["Actions", [
      ["o / ↵", "open entry detail"],
      ["f", "toggle favorite"],
      ["e", "edit note"],
      ["d", "delete (with confirm)"],
      ["y", "yank text to clipboard"],
      ["t", "edit tags"],
    ]],
    ["Navigation", [
      ["1..6", "jump to tab"],
      ["Ctrl-P", "command palette"],
      ["Ctrl-O / Ctrl-I", "back / forward in history"],
      ["?", "this overlay"],
    ]],
    ["View", [
      [":view table", "table view"],
      [":view tree", "tree view"],
      [":view dash", "dashboard"],
      [":view graph", "graph view"],
      [":theme mocha|latte", "switch theme"],
    ]],
  ];
  return (
    <div onClick={onClose} style={{
      position: "absolute", inset: 0, background: "rgba(17,17,27,0.7)",
      display: "flex", justifyContent: "center", alignItems: "center",
      zIndex: 60,
    }}>
      <div onClick={(e) => e.stopPropagation()} style={{
        width: 760, maxHeight: "80vh", overflow: "auto",
        background: theme.mantle,
        border: `1px solid ${theme.surface1}`,
        borderRadius: 4, padding: 24,
        boxShadow: `0 16px 48px ${theme.crust}cc`,
      }}>
        <div style={{ display: "flex", justifyContent: "space-between", alignItems: "baseline", borderBottom: `1px solid ${theme.surface0}`, paddingBottom: 12, marginBottom: 16 }}>
          <span style={{ fontSize: 14, color: theme.accent, fontWeight: 700 }}>?  ─  keybindings</span>
          <span style={{ color: theme.overlay0, fontSize: 11 }}>esc to close</span>
        </div>
        <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: "16px 32px" }}>
          {groups.map(([title, items]) => (
            <div key={title}>
              <div style={{ color: theme.peach, fontWeight: 700, marginBottom: 6, fontSize: 11, letterSpacing: "0.1em", textTransform: "uppercase" }}>── {title} ──</div>
              {items.map(([k, label]) => (
                <div key={k} style={{ display: "grid", gridTemplateColumns: "120px 1fr", gap: 12, padding: "2px 0" }}>
                  <span style={{ color: theme.green, fontSize: 12 }}>{k}</span>
                  <span style={{ color: theme.subtext, fontSize: 12 }}>{label}</span>
                </div>
              ))}
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}

window.CShell = CShell;
window.CTopBar = CTopBar;
window.CStatusLine = CStatusLine;
window.CPalette = CPalette;
window.CKeybindings = CKeybindings;
window.fuzzyFilter = fuzzyFilter;
window.highlightMatch = highlightMatch;

// c-system.jsx — Catppuccin theme tokens, monospace fonts, primitives.

const C_THEMES = {
  mocha: {
    bg:        "#1e1e2e",
    mantle:    "#181825",
    crust:     "#11111b",
    surface0:  "#313244",
    surface1:  "#45475a",
    surface2:  "#585b70",
    text:      "#cdd6f4",
    subtext:   "#bac2de",
    overlay0:  "#6c7086",
    overlay1:  "#7f849c",
    accent:    "#f5c2e7", // pink (default)
    blue:      "#89b4fa",
    green:     "#a6e3a1",
    red:       "#f38ba8",
    yellow:    "#f9e2af",
    peach:     "#fab387",
    mauve:     "#cba6f7",
    sky:       "#89dceb",
    teal:      "#94e2d5",
  },
  macchiato: {
    bg:"#24273a", mantle:"#1e2030", crust:"#181926",
    surface0:"#363a4f", surface1:"#494d64", surface2:"#5b6078",
    text:"#cad3f5", subtext:"#b8c0e0", overlay0:"#6e738d", overlay1:"#8087a2",
    accent:"#f5bde6", blue:"#8aadf4", green:"#a6da95", red:"#ed8796",
    yellow:"#eed49f", peach:"#f5a97f", mauve:"#c6a0f6", sky:"#91d7e3", teal:"#8bd5ca",
  },
  latte: {
    bg:"#eff1f5", mantle:"#e6e9ef", crust:"#dce0e8",
    surface0:"#ccd0da", surface1:"#bcc0cc", surface2:"#acb0be",
    text:"#4c4f69", subtext:"#5c5f77", overlay0:"#9ca0b0", overlay1:"#8c8fa1",
    accent:"#ea76cb", blue:"#1e66f5", green:"#40a02b", red:"#d20f39",
    yellow:"#df8e1d", peach:"#fe640b", mauve:"#8839ef", sky:"#04a5e5", teal:"#179299",
  },
};

const C_FONTS = {
  jetbrains: '"JetBrains Mono", ui-monospace, Menlo, monospace',
  iosevka:   '"Iosevka", "JetBrains Mono", ui-monospace, monospace',
  plex:      '"IBM Plex Mono", ui-monospace, Menlo, monospace',
  berkeley:  '"Berkeley Mono", "JetBrains Mono", ui-monospace, monospace',
};

function cTheme(name) { return C_THEMES[name] || C_THEMES.mocha; }
function cFont(name) { return C_FONTS[name] || C_FONTS.jetbrains; }

function cDensity(d) {
  if (d === "compact") return { row: "3px 14px", line: 1.35, fs: 12 };
  if (d === "comfy") return { row: "9px 16px", line: 1.7, fs: 13 };
  return { row: "6px 14px", line: 1.5, fs: 12.5 };
}

function CKbd({ children, theme }) {
  return (
    <span style={{
      display: "inline-block",
      padding: "0px 5px",
      border: `1px solid ${theme.surface1}`,
      background: theme.surface0,
      color: theme.subtext,
      fontSize: "0.85em",
      borderRadius: 2,
    }}>{children}</span>
  );
}

// Tiny inline sparkline
function CSpark({ values, color, width = 80, height = 18 }) {
  const max = Math.max(...values);
  const min = Math.min(...values);
  const range = max - min || 1;
  const pts = values.map((v, i) => {
    const x = (i / (values.length - 1)) * width;
    const y = height - ((v - min) / range) * (height - 2) - 1;
    return `${x.toFixed(1)},${y.toFixed(1)}`;
  }).join(" ");
  return (
    <svg width={width} height={height} style={{ display: "block" }}>
      <polyline points={pts} fill="none" stroke={color} strokeWidth="1" />
    </svg>
  );
}

window.C_THEMES = C_THEMES;
window.C_FONTS = C_FONTS;
window.cTheme = cTheme;
window.cFont = cFont;
window.cDensity = cDensity;
window.CKbd = CKbd;
window.CSpark = CSpark;

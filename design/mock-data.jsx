// mock-data.jsx — shared bookmark fixtures for all variations.
// Tech-twitter mix (Rust, design, AI, systems) — feels like a real archive.

const BOOKMARKS = [
  {
    id: "1",
    handle: "burntsushi5",
    name: "Andrew Gallant",
    avatar_hue: 14,
    tweeted_at: "2026-04-18T14:22:00Z",
    saved_at: "2026-04-18T19:04:00Z",
    content: "ripgrep is fast not because of any one trick. it's fast because every layer — the regex engine, the directory traversal, the SIMD prefilter, the buffer reuse — was independently profiled and tuned. there's no shortcut. you have to care everywhere.",
    tags: ["rust", "performance", "tools"],
    media: 0,
    is_favorite: true,
    likes: 2412,
  },
  {
    id: "2",
    handle: "rauchg",
    name: "Guillermo Rauch",
    avatar_hue: 220,
    tweeted_at: "2026-04-17T09:11:00Z",
    saved_at: "2026-04-17T22:30:00Z",
    content: "the best UI is one that respects the user's attention. animations that signal causality, not novelty. defaults that fit 80% of cases. silence when nothing meaningful is happening.",
    tags: ["design", "ux"],
    media: 0,
    is_favorite: false,
    likes: 884,
  },
  {
    id: "3",
    handle: "karpathy",
    name: "Andrej Karpathy",
    avatar_hue: 280,
    tweeted_at: "2026-04-16T18:44:00Z",
    saved_at: "2026-04-16T18:50:00Z",
    content: "spent the morning re-reading the original transformer paper. what's wild is how much of the architecture is still load-bearing eight years later. attention really was all we needed.",
    tags: ["ai", "ml", "papers"],
    media: 1,
    is_favorite: true,
    likes: 5601,
  },
  {
    id: "4",
    handle: "jessfraz",
    name: "Jessie Frazelle",
    avatar_hue: 340,
    tweeted_at: "2026-04-15T11:02:00Z",
    saved_at: "2026-04-15T11:05:00Z",
    content: "containers were never the abstraction. namespaces and cgroups were always the abstraction. containers are just a packaging convenience that we collectively pretended was a primitive.",
    tags: ["systems", "linux"],
    media: 0,
    is_favorite: false,
    likes: 1290,
  },
  {
    id: "5",
    handle: "fogus",
    name: "Michael Fogus",
    avatar_hue: 90,
    tweeted_at: "2026-04-14T20:15:00Z",
    saved_at: "2026-04-15T07:00:00Z",
    content: "most code review feedback should be a question, not a directive. \"what happens if this is null?\" travels further than \"add a null check.\"",
    tags: ["engineering", "teams"],
    media: 0,
    is_favorite: false,
    likes: 612,
  },
  {
    id: "6",
    handle: "_floooh",
    name: "Andre Weissflog",
    avatar_hue: 30,
    tweeted_at: "2026-04-13T08:33:00Z",
    saved_at: "2026-04-13T09:00:00Z",
    content: "the secret of small, fast C codebases is that you spend 80% of your time deleting things. abstractions you thought you needed. flexibility that never paid for itself. config that nobody changed.",
    tags: ["c", "performance"],
    media: 0,
    is_favorite: true,
    likes: 1855,
  },
  {
    id: "7",
    handle: "sophiebits",
    name: "Sophie Alpert",
    avatar_hue: 320,
    tweeted_at: "2026-04-12T15:18:00Z",
    saved_at: "2026-04-12T15:42:00Z",
    content: "if your component takes 14 props, that's not a component, that's a configuration form. split it. let one of the resulting things be opinionated.",
    tags: ["react", "design"],
    media: 0,
    is_favorite: false,
    likes: 740,
  },
  {
    id: "8",
    handle: "dhh",
    name: "DHH",
    avatar_hue: 0,
    tweeted_at: "2026-04-11T13:09:00Z",
    saved_at: "2026-04-11T18:00:00Z",
    content: "you do not need a job queue. you do not need a message broker. you do not need event sourcing. you need a database transaction and the discipline to leave well enough alone.",
    tags: ["architecture"],
    media: 0,
    is_favorite: false,
    likes: 3204,
  },
  {
    id: "9",
    handle: "pervognsen",
    name: "Per Vognsen",
    avatar_hue: 200,
    tweeted_at: "2026-04-10T22:48:00Z",
    saved_at: "2026-04-11T09:14:00Z",
    content: "linear types aren't really about safety. they're about making the cost model of your program legible at the type level. the safety is downstream.",
    tags: ["plt", "types"],
    media: 0,
    is_favorite: true,
    likes: 502,
  },
  {
    id: "10",
    handle: "matklad",
    name: "Alex Kladov",
    avatar_hue: 18,
    tweeted_at: "2026-04-09T07:21:00Z",
    saved_at: "2026-04-09T07:25:00Z",
    content: "rust-analyzer's best architectural decision was incremental computation everywhere. salsa makes editor latency a function of edit size, not project size. that's the whole game.",
    tags: ["rust", "tools"],
    media: 0,
    is_favorite: false,
    likes: 921,
  },
  {
    id: "11",
    handle: "shaver",
    name: "Mike Shaver",
    avatar_hue: 250,
    tweeted_at: "2026-04-08T19:55:00Z",
    saved_at: "2026-04-08T20:10:00Z",
    content: "every postmortem eventually arrives at the same root cause: someone, three years ago, decided not to write the documentation.",
    tags: ["engineering"],
    media: 0,
    is_favorite: false,
    likes: 2105,
  },
  {
    id: "12",
    handle: "ID_AA_Carmack",
    name: "John Carmack",
    avatar_hue: 60,
    tweeted_at: "2026-04-07T03:14:00Z",
    saved_at: "2026-04-07T08:02:00Z",
    content: "i remain convinced that a focused two-week rewrite, by someone who already understands the domain, will outperform six months of incremental refactor by a committee. the hard part is admitting which one of those you are.",
    tags: ["engineering", "performance"],
    media: 0,
    is_favorite: true,
    likes: 7822,
  },
];

const TOP_TAGS = [
  ["rust", 38],
  ["design", 27],
  ["ai", 24],
  ["systems", 19],
  ["performance", 17],
  ["engineering", 14],
];

const STATS = {
  total: 412,
  authors: 184,
  favorites: 47,
  this_week: 12,
};

function fmtDate(iso) {
  const d = new Date(iso);
  return d.toLocaleDateString("en-US", { month: "short", day: "numeric", year: "numeric" });
}
function fmtRel(iso) {
  const now = new Date("2026-04-19T12:00:00Z");
  const d = new Date(iso);
  const days = Math.floor((now - d) / 86400000);
  if (days < 1) return "today";
  if (days < 2) return "yesterday";
  if (days < 7) return `${days}d ago`;
  if (days < 30) return `${Math.floor(days/7)}w ago`;
  return `${Math.floor(days/30)}mo ago`;
}

Object.assign(window, { BOOKMARKS, TOP_TAGS, STATS, fmtDate, fmtRel });

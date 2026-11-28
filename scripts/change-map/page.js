"use strict";
// Operon's product map (changes 312, 313, 315, 316, 325).
//
// Two halves. `MapLogic` is plain functions with no page in sight — links
// and the choice they carry, the walk through screens, search, the scale of a
// pressable area, the lenses, the texts handed to a session, the keys — so
// the `map_page_logic_*` tests in src/tests.rs run them under node. `main`
// draws, and runs only where there is a document.
// Every value from the model passes through esc() before it becomes markup.

const MapLogic = (function () {
  const LISTS = { "": "overview", overview: "overview", product: "features", features: "features",
    screen: "screen", canvas: "canvas", flow: "canvas", domains: "domains", glossary: "glossary",
    "glossary/screens": "glossary-screens", requirements: "requirements", changes: "changes",
    work: "work", process: "process", catalog: "catalog" };

  function decoded(text) { try { return decodeURIComponent(text); } catch (error) { return text; } }

  // The choice a link carries after `?`: `state=changing,waiting&cover=none`.
  function parseQuery(text) {
    const query = {};
    for (const pair of String(text || "").split("&")) {
      const at = pair.indexOf("=");
      if (at < 1) continue;
      const values = pair.slice(at + 1).split(",").map(decoded).filter(Boolean);
      if (values.length) query[decoded(pair.slice(0, at))] = values;
    }
    return query;
  }
  function formatQuery(query) {
    const pairs = Object.keys(query || {}).filter(key => (query[key] || []).length).map(key =>
      encodeURIComponent(key) + "=" + query[key].map(encodeURIComponent).join(","));
    return pairs.length ? "?" + pairs.join("&") : "";
  }

  // A link, read back into the view it names. An unknown link opens 現況. A
  // route has a `query` only when its link carries a choice.
  function parseRoute(hash) {
    const whole = String(hash || "").replace(/^#/, "");
    const cut = whole.indexOf("?");
    const route = pathRoute(decoded(cut < 0 ? whole : whole.slice(0, cut)));
    if (!route) return { view: "overview" };
    const query = cut < 0 ? {} : parseQuery(whole.slice(cut + 1));
    if (Object.keys(query).length) route.query = query;
    return route;
  }
  function pathRoute(raw) {
    if (Object.prototype.hasOwnProperty.call(LISTS, raw)) return { view: LISTS[raw] };
    let m = raw.match(/^feature\/([^/]+)(?:\/([^/]+))?$/);
    if (m) return m[2] ? { view: "feature", id: m[1], tab: m[2] } : { view: "feature", id: m[1] };
    m = raw.match(/^(screen|domain|term|requirements)\/([^/]+)$/);
    if (m) return { view: m[1], id: m[2] };
    m = raw.match(/^change\/([\w-]+)(?:\/([^/]+))?$/) || raw.match(/^(\d+)(?:\/([^/]+))?$/);
    if (m) return m[2] ? { view: "change", id: m[1], tab: m[2] } : { view: "change", id: m[1] };
    return null;
  }

  function formatRoute(r) { return formatPath(r) + formatQuery(r.query); }
  function formatPath(r) {
    const tail = r.tab ? "/" + encodeURIComponent(r.tab) : "";
    switch (r.view) {
      case "glossary-screens": return "#glossary/screens";
      case "feature": return "#feature/" + encodeURIComponent(r.id) + tail;
      case "change": return "#change/" + encodeURIComponent(r.id) + tail;
      case "screen": case "requirements": return "#" + r.view + (r.id ? "/" + encodeURIComponent(r.id) : "");
      case "domain": case "term": return "#" + r.view + "/" + encodeURIComponent(r.id);
      default: return "#" + r.view;
    }
  }

  // ---- lenses: the rows of chips over the feature tiles ---------------------
  // A feature with no requirement line has no test either, so it answers なし.
  const LENSES = [
    { id: "state", label: "状態", values: [["steady", "安定"], ["changing", "変更中"], ["waiting", "判断待ち"]], of: f => f.badge },
    { id: "cover", label: "テスト", values: [["full", "すべてあり"], ["partial", "一部"], ["none", "なし"]],
      of: f => f.cover === "empty" || !f.cover ? "none" : f.cover },
    { id: "domain", label: "ドメイン", values: null, of: f => f.domain || "" }];
  // One press replaces the row's choice (and clears it when it was the only
  // one chosen); with `additive` it adds the value or takes it away.
  function lensToggle(query, lens, value, additive) {
    const next = {};
    for (const key of Object.keys(query || {})) next[key] = query[key].slice();
    const chosen = next[lens] || [];
    let values;
    if (additive) values = chosen.includes(value) ? chosen.filter(v => v !== value) : chosen.concat([value]);
    else values = chosen.length === 1 && chosen[0] === value ? [] : [value];
    if (values.length) next[lens] = values; else delete next[lens];
    return next;
  }
  function featureMatches(feature, query) {
    return LENSES.every(lens => {
      const chosen = (query || {})[lens.id];
      return !chosen || !chosen.length || chosen.includes(lens.of(feature));
    });
  }
  // For every value of every lens: how many features would remain were that
  // value the lens's choice, the other lenses staying as they are.
  function lensCounts(features, query) {
    const counts = {};
    for (const lens of LENSES) {
      const values = lens.values ? lens.values.map(v => v[0]) : Array.from(new Set(features.map(lens.of)));
      counts[lens.id] = {};
      for (const value of values) {
        const tried = Object.assign({}, query || {}, { [lens.id]: [value] });
        counts[lens.id][value] = features.filter(f => featureMatches(f, tried)).length;
      }
    }
    return counts;
  }

  // ---- the basket and the two texts a person hands to a session ------------
  const WORK_KINDS = [
    ["fix", "修正", "不具合や望まない振る舞いを直す（原因が分からなければ root-cause から）"],
    ["add", "追加", "新しい振る舞いを足す"],
    ["review", "レビュー", "コードは変えず、問題点と改善案を報告する"],
    ["document", "文書", "文書を実装に合わせて直す"],
    ["test", "テスト", "テストのない振る舞いにテストを足す"]];
  const SHOTS_RECORD = "docs/images/screens/screens.json";
  function byId(list, id) { return (list || []).find(item => String(item.id) === id); }
  // What a basket key names: its line and the files to read first, or null
  // when the model no longer holds the record.
  function basketEntry(key, M) {
    const at = String(key).indexOf("|");
    const kind = String(key).slice(0, at), id = String(key).slice(at + 1);
    const features = M.features || [];
    if (kind === "feature") {
      const f = byId(features, id);
      if (!f) return null;
      const files = [];
      if (f.brief) files.push("docs/features/" + id + "/brief.md");
      if (f.spec) files.push("docs/features/" + id + "/spec.md");
      for (const layer of f.code || []) for (const file of layer.files) files.push(file.path);
      return { label: "機能「" + f.title + "」(" + id + ")", files };
    }
    if (kind === "domain") {
      const d = byId(M.domains, id);
      return d ? { label: "ドメイン「" + d.title + "」(" + id + ")", files: ["docs/features/domains.yaml"] } : null;
    }
    if (kind === "screen") {
      const s = byId((M.flow || {}).screens, id);
      return s ? { label: "画面「" + s.title + "」(" + id + ")", files: ["docs/features/flow.yaml"].concat(s.shot ? [SHOTS_RECORD] : []) } : null;
    }
    if (kind === "term") {
      const t = byId(M.terms, id);
      return t ? { label: "用語「" + t.term + "」(" + id + ")", files: ["docs/features/glossary.yaml"] } : null;
    }
    if (kind === "behaviour") {
      const cut = id.lastIndexOf("#");
      const feature = id.slice(0, cut), number = id.slice(cut + 1);
      const row = (M.requirements || []).find(r => r.feature === feature && String(r.number) === number);
      const f = byId(features, feature);
      if (!row || !f) return null;
      return { label: "振る舞い: 機能「" + f.title + "」の " + number + " 番「" + row.text + "」", files: ["docs/features/" + feature + "/spec.md"] };
    }
    if (kind === "change") {
      const c = (M.changes || []).find(x => x.name === id);
      return c ? { label: "変更 " + c.number + "「" + c.title + "」", files: ["docs/sdlc/changes/" + id + "/"] } : null;
    }
    if (kind === "file") {
      const known = features.some(f => (f.code || []).some(layer => layer.files.some(file => file.path === id)));
      return known ? { label: "ファイル " + id, files: [id] } : null;
    }
    if (kind === "issue") {
      const backlog = M.backlog || { asked: false, items: [] };
      const item = (backlog.items || []).find(x => String(x.number) === id);
      // A map made without the backlog cannot say the item is gone.
      if (!item && backlog.asked) return null;
      return { label: "バックログ #" + id + (item ? "「" + item.title + "」" : ""), files: [".claude/skills/backlog/SKILL.md"] };
    }
    return null;
  }
  // What the browser kept for the basket, whatever shape it came back in.
  function basketOf(kept) {
    const k = kept && typeof kept === "object" && !Array.isArray(kept) ? kept : {};
    return { keys: Array.isArray(k.keys) ? k.keys.filter(key => typeof key === "string") : [],
      kind: WORK_KINDS.some(w => w[0] === k.kind) ? k.kind : "fix",
      note: typeof k.note === "string" ? k.note : "" };
  }
  function pruneBasket(keys, M) { return keys.filter(key => basketEntry(key, M) !== null); }
  function count(text) { return Array.from(text).length; }
  function workOrder(keys, M, kind, note) {
    const work = WORK_KINDS.find(w => w[0] === kind) || WORK_KINDS[0];
    const entries = keys.map(key => basketEntry(key, M)).filter(Boolean);
    const files = [];
    for (const entry of entries) for (const file of entry.files) if (!files.includes(file)) files.push(file);
    const said = String(note || "").trim();
    const text = "Operon の次の作業を feature-pilot で進めてください（.claude/skills/feature-pilot/SKILL.md から始める）。\n" +
      "種類: " + work[1] + " — " + work[2] + "\n" +
      "してほしいこと:\n" + (said || "（ここに書いてください）") + "\n" +
      "対象:\n" + (entries.length ? entries.map(e => "- " + e.label).join("\n") : "- （なし）") + "\n" +
      "先に読むファイル:\n" + (files.length ? files.map(f => "- " + f).join("\n") : "- （なし）") + "\n";
    return { text, chars: count(text) };
  }
  const VERDICTS = [["ok", "OK"], ["fix", "要修正"], ["hold", "保留"]];
  function verdictOf(verdicts, id) {
    const v = (verdicts || {})[id];
    return v && VERDICTS.some(known => known[0] === v.verdict) ? v : null;
  }
  // Every verdict as one text: what to fix first, then what is held, then how
  // many were fine. A feature without a verdict is not named.
  function reviewPacket(verdicts, features) {
    const counts = { ok: 0, fix: 0, hold: 0 };
    const lines = { fix: [], hold: [] };
    for (const f of features) {
      const v = verdictOf(verdicts, f.id);
      if (!v) continue;
      counts[v.verdict]++;
      if (v.verdict !== "ok") lines[v.verdict].push("- 機能「" + f.title + "」(" + f.id + "): " +
        (String(v.note || "").trim() || "（メモなし）") + " — docs/features/" + f.id + "/spec.md");
    }
    if (!counts.ok && !counts.fix && !counts.hold) return { text: "", chars: 0, counts };
    let text = "Operon の製品地図で機能を確認した結果です。feature-pilot で「要修正」から順に対応してください（.claude/skills/feature-pilot/SKILL.md から始める）。\n";
    if (lines.fix.length) text += "要修正 (" + counts.fix + "):\n" + lines.fix.join("\n") + "\n";
    if (lines.hold.length) text += "保留 (" + counts.hold + "):\n" + lines.hold.join("\n") + "\n";
    text += "OK: " + counts.ok + " 件\n";
    return { text, chars: count(text), counts };
  }
  // The browser's storage, which a file page or a private window may refuse:
  // a refused read gives the fallback and a refused write gives false.
  function keep(storage) {
    return {
      read(name, fallback) {
        try {
          const raw = storage().getItem(name);
          if (raw === null || raw === undefined) return fallback;
          try { return JSON.parse(raw); } catch (error) { return raw; }
        } catch (error) { return fallback; }
      },
      write(name, value) {
        try { storage().setItem(name, typeof value === "string" ? value : JSON.stringify(value)); return true; }
        catch (error) { return false; }
      } };
  }

  // ---- narrowing ------------------------------------------------------------
  function boardFilter(cards, who) {
    const owner = { me: "person", ai: "ai" }[who];
    return owner ? cards.filter(card => card.owner === owner) : cards.slice();
  }
  function hasWords(text, words) {
    const q = String(words || "").trim().toLowerCase();
    return !q || String(text).toLowerCase().includes(q);
  }
  function requirementMatches(row, filter, title) {
    const f = filter || {};
    return (!f.domain || row.domain === f.domain) && (!f.feature || row.feature === f.feature) &&
      (!f.tests || (f.tests === "yes") === !!row.pinned) &&
      hasWords(row.text + " " + (title || "") + " " + (row.tests || []).join(" "), f.words);
  }
  function queueMatches(feature, filter, verdicts) {
    const f = filter || {};
    const judged = !!verdictOf(verdicts, feature.id);
    return (!f.verdict || (f.verdict === "judged") === judged) &&
      hasWords(feature.title + " " + feature.id + " " + (feature.summary || ""), f.words);
  }
  function verdictCounts(features, verdicts) {
    const judged = features.filter(f => verdictOf(verdicts, f.id)).length;
    return { all: features.length, judged, unjudged: features.length - judged };
  }
  // The text under one `## heading` of a document, up to the next one.
  function sectionOf(text, heading) {
    const lines = String(text || "").split("\n");
    const at = lines.findIndex(line => line.trim() === "## " + heading);
    if (at < 0) return "";
    const rest = lines.slice(at + 1);
    const end = rest.findIndex(line => line.startsWith("## "));
    return (end < 0 ? rest : rest.slice(0, end)).join("\n").trim();
  }
  // What the page keeps in the browser, and nowhere else.
  const STORE = { basket: "operon-map-basket", verdicts: "operon-map-verdicts", theme: "operon-map-theme" };
  const THEMES = [["auto", "自動"], ["light", "ライト"], ["dark", "ダーク"]];
  function themeNext(theme) {
    const at = THEMES.findIndex(t => t[0] === theme);
    return THEMES[(at + 1) % THEMES.length][0];
  }

  // A control's bounds are kept in points of the window it was taken in; the
  // picture is drawn at whatever width the page has.
  function scaleArea(area, window, drawnWidth) {
    const f = drawnWidth / window.w;
    return { left: area.x * f, top: area.y * f, width: area.w * f, height: area.h * f };
  }
  function areaPercent(area, window) {
    return { left: 100 * area.x / window.w, top: 100 * area.y / window.h,
      width: 100 * area.w / window.w, height: 100 * area.h / window.h };
  }

  // The screens walked, as a browser's history keeps them: a new step drops
  // whatever was ahead.
  function Walk(start) { this.stack = start ? [start] : []; this.at = this.stack.length - 1; }
  Walk.prototype.current = function () { return this.stack[this.at]; };
  Walk.prototype.visit = function (id) {
    if (this.stack[this.at] === id) return id;
    this.stack = this.stack.slice(0, this.at + 1);
    this.stack.push(id);
    this.at = this.stack.length - 1;
    return id;
  };
  Walk.prototype.canBack = function () { return this.at > 0; };
  Walk.prototype.canForward = function () { return this.at < this.stack.length - 1; };
  Walk.prototype.back = function () { if (this.canBack()) this.at--; return this.current(); };
  Walk.prototype.forward = function () { if (this.canForward()) this.at++; return this.current(); };
  // A screen arrived at by the browser's own back or forward.
  Walk.prototype.sync = function (id) {
    if (this.stack[this.at - 1] === id) return this.back();
    if (this.stack[this.at + 1] === id) return this.forward();
    return this.visit(id);
  };

  // Search over every kind: a title equal to the query first, then one that
  // starts with it, then one that contains it, then any other text.
  function search(items, query, limit) {
    const q = String(query || "").trim().toLowerCase();
    if (!q) return [];
    const scored = [];
    items.forEach((item, order) => {
      const title = String(item.title || "").toLowerCase();
      const text = String(item.text || "").toLowerCase();
      const score = title === q ? 4 : title.startsWith(q) ? 3 : title.includes(q) ? 2 : text.includes(q) ? 1 : 0;
      if (score) scored.push({ item, score, order });
    });
    scored.sort((a, b) => b.score - a.score || a.item.title.length - b.item.title.length || a.order - b.order);
    return scored.slice(0, limit || 60).map(s => Object.assign({ score: s.score }, s.item));
  }

  const KEYS = { "/": "search", j: "next", k: "prev", Enter: "open", Escape: "close", "?": "help",
    ArrowLeft: "back", ArrowRight: "forward",
    s: "view:overview", f: "view:features", g: "view:screen", w: "view:work", r: "view:requirements",
    c: "view:changes", p: "view:process", d: "view:domains", b: "basket",
    "1": "verdict:ok", "2": "verdict:fix", "3": "verdict:hold" };
  const KEY_HELP = [["/", "検索に移る"], ["s f g d", "現況・機能・画面・ドメイン"], ["r w c p", "要件・作業・変更・進め方"],
    ["j / k", "次・前の機能（一覧では次・前の行）"], ["Enter", "選んだものを開く"], ["b", "作業かごを開く"],
    ["1 2 3", "機能のページで OK・要修正・保留"], ["Esc", "閉じる"], ["← / →", "画面で戻る・進む"], ["?", "この一覧"]];

  return { parseRoute, formatRoute, scaleArea, areaPercent, Walk, search, KEYS, KEY_HELP,
    LENSES, lensToggle, featureMatches, lensCounts, WORK_KINDS, basketEntry, basketOf, pruneBasket, workOrder,
    VERDICTS, reviewPacket, keep, boardFilter, requirementMatches, queueMatches, verdictCounts,
    sectionOf, STORE, THEMES, themeNext };
})();

if (typeof module === "object" && module.exports) module.exports = MapLogic;
if (typeof document !== "undefined") {
  // Before the first paint, so a kept theme does not flash the other one.
  const kept = MapLogic.keep(() => localStorage).read(MapLogic.STORE.theme, "auto");
  document.documentElement.dataset.theme = MapLogic.THEMES.some(t => t[0] === kept) ? kept : "auto";
  main();
}

function main() {
const M = JSON.parse(document.getElementById("map-data").textContent);
const STATUS = { "planning": "計画中", "ready": "着手可", "in-progress": "実装中", "reviewing": "検証中",
  "done": "完了", "awaiting-user": "判断待ち", "blocked": "停止中", "failed": "失敗", "archived": "保管" };
const STAGE = { plan: "計画", design: "設計", build: "実装", test: "テスト", deploy: "出荷", maintain: "保守" };
const GROUP = { needs: "要対応", flight: "進行中", done: "完了", archived: "保管" };
const TAB = { "intent.md": "意図", "spec.md": "仕様", "plan.md": "計画", "screen.md": "画面案",
  "bug.md": "不具合の報告", "handoff.md": "引き継ぎ", "state.yaml": "状態の記録", "review.yaml": "レビューの記録",
  "verification.yaml": "検証の記録" };
const VERDICT = { passed: "済", pending: "未", waived: "免除" };
const BADGE = { steady: "安定", changing: "変更中", waiting: "判断待ち" };
const ROUTE = { feature: "新機能", modify: "変更", bugfix: "不具合の修正", refactor: "整理",
  security: "安全性", docs: "文書", trivial: "軽微な修正" };
const LINK = { recorded: "記録", inferred: "計画のファイルから推定", estimated: "題から推定" };
const KIND = { page: "ページ", tab: "タブ", panel: "パネル", sheet: "シート", overlay: "重ねて開く", cli: "コマンドライン" };
const FEATURES = M.features;
const DOMAINS = M.domains || [];
const TERMS = M.terms || [];
const FLOW = M.flow || { screens: [], ways: [], bands: [], box: [168, 58], width: 0, height: 0 };
const REQS = M.requirements || [];
const SHOTS = M.shots || {};
const OV = M.overview || {};
const featureById = new Map(FEATURES.map(f => [f.id, f]));
const domainById = new Map(DOMAINS.map(d => [d.id, d]));
const termById = new Map(TERMS.map(t => [t.id, t]));
const flowById = new Map(FLOW.screens.map(s => [s.id, s]));
const byName = new Map(M.changes.map(c => [c.name, c]));
const byNumber = new Map(M.changes.map(c => [c.number, c]));
const $ = id => document.getElementById(id);
const L = MapLogic;

const screenLabel = id => { const s = M.screens.find(x => x.id === id); return s ? s.label : id; };
const routeWord = c => ROUTE[c.route] || c.route;
const stageWord = s => STAGE[s] || s;
const domainTitle = id => (domainById.get(id) || { title: id || "（ドメイン未設定）" }).title;
const featureTitle = id => (featureById.get(id) || { title: id }).title;
const termWord = id => (termById.get(id) || { term: id }).term;
const flowTitle = id => (flowById.get(id) || { title: id }).title;

function esc(text) {
  return String(text).replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;").replace(/'/g, "&#39;");
}
function statusWord(c) { return c.problem ? "読めない記録" : (STATUS[c.status] || c.status); }
function pill(c) { return '<span class="pill ' + c.group + '">' + esc(statusWord(c)) + "</span>"; }
function place(c) { return c.where.branch || (c.where.here ? "" : c.where.path); }
function featuresOf(c) {
  const titles = (c.features || []).map(l => featureTitle(l.id));
  if (titles.length) return titles.slice(0, 3).join("、") + (titles.length > 3 ? " ほか" : "");
  return c.features_named.trim() === "none" ? "（製品の機能ではありません）" : "（機能未設定）";
}
function areaLabel(name) { const a = M.areas.find(x => x.name === name); return a ? a.label : name; }
function haystack(c) {
  return [c.name, c.title, statusWord(c), c.status, c.route, STAGE[c.stage] || "", c.stage,
    c.areas.map(areaLabel).join(" "), c.areas.join(" "), place(c),
    (c.features || []).map(l => featureTitle(l.id) + " " + l.id).join(" ")].join(" ").toLowerCase();
}
function lastDate(c) { const a = c.approvals || []; return a.length ? String(a[a.length - 1].at).slice(0, 10) : ""; }
function go(route) { const h = L.formatRoute(route); if (location.hash === h) render(); else location.hash = h; }

function card(c, small) {
  let h = '<button class="card ' + c.group + '" data-open="' + esc(c.name) + '"><div class="head">' +
    '<span class="num">' + esc(c.number) + '</span><span class="title">' + esc(c.title) + "</span>" + pill(c) + "</div>";
  const sub = [STAGE[c.stage] || c.stage, routeWord(c), place(c)].filter(Boolean).join(" · ");
  if (!small) h += '<div class="sub">' + esc(sub) + "</div>";
  else if (place(c)) h += '<div class="sub">' + esc(place(c)) + "</div>";
  if (c.problem) h += '<div class="next"><b>記録を読めません: </b>' + esc(c.problem) + "</div>";
  else if (c.resume && !small) h += '<div class="next"><b>次にすること: </b>' + esc(c.resume) + "</div>";
  return h + "</button>";
}

let query = "";
let groupFilter = "";
let expanded = new Set();
let listed = M.changes.slice();
function matches(c) { return !query || haystack(c).includes(query); }

// ---- markdown: escape first, then markup around what was escaped -----------
function inline(text) {
  const codes = [];
  let s = text.replace(/`([^`]+)`/g, (_, code) => { codes.push(code); return "\u0000" + (codes.length - 1) + "\u0000"; });
  s = esc(s);
  s = s.replace(/\*\*([^*]+)\*\*/g, "<strong>$1</strong>");
  s = s.replace(/(^|[\s(])\*([^*\s][^*]*)\*(?=[\s).,;:!?]|$)/g, "$1<em>$2</em>");
  // No `[` inside the label: the search then restarts at the next bracket
  // instead of rescanning to the end from every one (quadratic on a wall of them).
  s = s.replace(/\[([^\]\[]+)\]\((https?:\/\/[^\s)]+)\)/g,
    '<a href="$2" target="_blank" rel="noopener noreferrer">$1</a>');
  return s.replace(/\u0000(\d+)\u0000/g, (_, i) => "<code>" + esc(codes[Number(i)]) + "</code>");
}
function cells(line) {
  let s = line.trim();
  if (s.startsWith("|")) s = s.slice(1);
  if (s.endsWith("|")) s = s.slice(0, -1);
  const out = []; let cur = ""; let tick = false;
  for (let i = 0; i < s.length; i++) {
    const ch = s[i];
    if (ch === "`") tick = !tick;
    if (ch === "\\" && s[i + 1] === "|") { cur += "|"; i++; continue; }
    if (ch === "|" && !tick) { out.push(cur.trim()); cur = ""; continue; }
    cur += ch;
  }
  out.push(cur.trim());
  return out;
}
function markdown(text) {
  const lines = text.split("\n");
  const out = [];
  let i = 0;
  const listItem = /^(\s*)([-*]|\d+[.)])\s+(.*)$/;
  while (i < lines.length) {
    const line = lines[i];
    const fence = line.match(/^\s*(```|~~~)/);
    if (fence) {
      const buf = []; i++;
      while (i < lines.length && !lines[i].trim().startsWith(fence[1])) { buf.push(lines[i]); i++; }
      i++;
      out.push("<pre><code>" + esc(buf.join("\n")) + "</code></pre>");
      continue;
    }
    if (!line.trim()) { i++; continue; }
    const head = line.match(/^(#{1,6})\s+(.*)$/);
    if (head) { const n = head[1].length; out.push("<h" + n + ">" + inline(head[2]) + "</h" + n + ">"); i++; continue; }
    if (/^\s*([-*_])\s*(\1\s*){2,}$/.test(line)) { out.push("<hr>"); i++; continue; }
    if (line.trim().startsWith("|") && i + 1 < lines.length && /^\s*\|?\s*:?-{2,}/.test(lines[i + 1])) {
      const header = cells(line); i += 2;
      let t = "<table><thead><tr>" + header.map(c => "<th>" + inline(c) + "</th>").join("") + "</tr></thead><tbody>";
      while (i < lines.length && lines[i].trim().startsWith("|")) {
        t += "<tr>" + cells(lines[i]).map(c => "<td>" + inline(c) + "</td>").join("") + "</tr>"; i++;
      }
      out.push(t + "</tbody></table>");
      continue;
    }
    if (line.startsWith(">")) {
      const buf = [];
      while (i < lines.length && lines[i].startsWith(">")) { buf.push(lines[i].replace(/^>\s?/, "")); i++; }
      out.push("<blockquote>" + markdown(buf.join("\n")) + "</blockquote>");
      continue;
    }
    if (listItem.test(line)) {
      const base = line.match(listItem)[1].length;
      const ordered = /\d/.test(line.match(listItem)[2]);
      let h = ordered ? "<ol>" : "<ul>";
      while (i < lines.length) {
        const m = lines[i].match(listItem);
        if (!m || m[1].length < base) break;
        if (m[1].length > base) { h += markdownNested(); continue; }
        let item = m[3]; i++;
        while (i < lines.length && lines[i].trim() && !listItem.test(lines[i]) &&
               /^\s+/.test(lines[i]) && !/^\s*(```|~~~)/.test(lines[i])) { item += " " + lines[i].trim(); i++; }
        h += "<li>" + inline(item);
        if (i < lines.length) {
          const next = lines[i].match(listItem);
          if (next && next[1].length > base) h += markdownNested();
        }
        h += "</li>";
        while (i < lines.length && !lines[i].trim() && i + 1 < lines.length && listItem.test(lines[i + 1]) &&
               lines[i + 1].match(listItem)[1].length >= base) i++;
      }
      out.push(h + (ordered ? "</ol>" : "</ul>"));
      continue;
    }
    const para = [line.trim()]; i++;
    while (i < lines.length && lines[i].trim() && !/^(#{1,6}\s|>|\s*(```|~~~)|\s*\|)/.test(lines[i]) &&
           !listItem.test(lines[i])) { para.push(lines[i].trim()); i++; }
    out.push("<p>" + inline(para.join(" ")) + "</p>");
  }
  return out.join("\n");

  function markdownNested() {
    const depth = lines[i].match(listItem)[1].length;
    const buf = [];
    while (i < lines.length && (!lines[i].trim() ? false :
           (lines[i].match(/^(\s*)/)[1].length >= depth))) { buf.push(lines[i].slice(depth)); i++; }
    return markdown(buf.join("\n"));
  }
}
function mdBlock(text) {
  let rendered = null;
  // A document the renderer cannot take (nesting past the stack) is still
  // shown, as written.
  try { rendered = markdown(text); } catch (error) { rendered = null; }
  return rendered !== null ? '<div class="md">' + rendered + "</div>" : '<pre class="raw">' + esc(text) + "</pre>";
}

// ---- small parts ---------------------------------------------------------
function chip(attr, id, label) { return '<button class="fchip" ' + attr + '="' + esc(id) + '">' + esc(label) + "</button>"; }
function domainChip(id) { return id ? chip("data-domain", id, domainTitle(id)) : ""; }
function termChips(ids) {
  return ids.length ? '<span class="fchips">' + ids.map(id => chip("data-term", id, termWord(id))).join("") + "</span>"
    : '<span class="empty">なし</span>';
}
function featureChipsOf(ids) {
  return ids.length ? '<span class="fchips">' + ids.map(id => chip("data-feature", id, featureTitle(id))).join("") + "</span>"
    : '<span class="empty">なし</span>';
}
function errorBox(error, notice) {
  return (error ? '<div class="box needs"><h3>' + esc(error) + "</h3>ほかの見方はそのまま使えます。</div>" : "") +
    (notice ? '<p class="warn">' + esc(notice) + "</p>" : "");
}
function bar(part, whole) { return whole ? Math.round(100 * part / whole) : 0; }
// The one place a picture becomes markup: only an inlined picture, never a
// fetched one.
function imgTag(src, alt, extra) {
  return /^data:image\/(jpeg|png);base64,[A-Za-z0-9+\/=]*$/.test(String(src)) ?
    '<img src="' + src + '" alt="' + esc(alt) + '"' + (extra || ' loading="lazy" decoding="async"') + ">" : "";
}
function thumbImg(id, alt) {
  return SHOTS[id] ? imgTag(SHOTS[id], alt)
    : '<span class="ph">' + esc((flowById.get(id) || {}).shot_problem || "写真がありません") + "</span>";
}
function screenThumb(s) {
  return '<button class="thumb" data-screen="' + esc(s.id) + '">' + thumbImg(s.id, s.title + "の画面") +
    '<span class="t">' + esc(s.title) + '</span><span class="k">' + esc(KIND[s.kind] || s.kind) + " · " + esc(screenLabel(s.screen)) + "</span></button>";
}
function taken(s) {
  if (!s.shot) return "";
  return "撮影 " + esc(String(s.shot.taken).slice(0, 10)) + (s.shot.commit ? "（" + esc(s.shot.commit) + "）" : "");
}

// ---- what the browser keeps: the basket, the verdicts ----------------------
const store = L.keep(() => localStorage);
let saidUnkept = false;
function toast(text) {
  const box = $("toast");
  box.textContent = text; box.hidden = false;
  clearTimeout(toast.timer);
  toast.timer = setTimeout(() => { box.hidden = true; }, 6000);
}
// Storage that refuses leaves the page working without memory, and says so once.
function keepOrSay(name, value) {
  if (store.write(name, value) || saidUnkept) return;
  saidUnkept = true;
  toast("この画面では選んだ内容を保存できません");
}
function copyText(text, said, area) {
  const failed = () => {
    if (area) { area.hidden = false; area.focus(); area.select(); }
    said.textContent = "コピーできませんでした。選択された文を手でコピーしてください";
  };
  if (!navigator.clipboard) { failed(); return; }
  navigator.clipboard.writeText(text).then(() => { said.textContent = "コピーしました"; }, failed);
}

// ---- the work basket -----------------------------------------------------
const keptBasket = L.basketOf(store.read(L.STORE.basket, {}));
const keptKeys = keptBasket.keys;
// An entry whose record has gone since it was put in is dropped here.
const basket = new Map(L.pruneBasket(keptKeys, M).map(key => [key, true]));
let workType = keptBasket.kind;
let workNote = keptBasket.note;
let basketOpen = false;
function saveBasket() { keepOrSay(L.STORE.basket, { keys: Array.from(basket.keys()), kind: workType, note: workNote }); }
function basketBox(kind, id) {
  const key = kind + "|" + id;
  return '<label class="bk" title="作業かごに入れる"><input type="checkbox" data-basket="' + esc(key) + '"' +
    (basket.has(key) ? " checked" : "") + "> かご</label>";
}
function instruction() { return L.workOrder(Array.from(basket.keys()), M, workType, workNote); }
function showInstruction() {
  const order = instruction();
  $("bk-text").value = order.text;
  $("bk-chars").textContent = order.chars + " 文字";
}
function renderBasket() {
  $("bk-n").textContent = basket.size ? " " + basket.size : "";
  if (!basketOpen) return;
  if (!basket.size) {
    $("bk-body").innerHTML = '<p class="empty">まだ何も入っていません</p><p class="what">機能・画面・振る舞い・用語・ドメイン・変更・ファイル・バックログの「かご」に印を付けると、ここに集まります。再読み込みしても残ります。</p>';
    return;
  }
  $("bk-body").innerHTML = '<ul class="ways">' + Array.from(basket.keys()).map(key =>
    "<li>" + esc((L.basketEntry(key, M) || { label: key }).label) + ' <button class="fchip" data-unbasket="' + esc(key) + '">外す</button></li>').join("") + "</ul>" +
    '<label class="bkl" for="bk-note">何をしてほしいですか</label><textarea id="bk-note" rows="3" placeholder="例: 止めたあとに一覧へ戻らないのを直してほしい">' + esc(workNote) + "</textarea>" +
    '<div class="filters">' + L.WORK_KINDS.map(w => '<button data-work="' + w[0] + '" aria-pressed="' + (w[0] === workType) + '" title="' + esc(w[2]) + '">' + w[1] + "</button>").join("") + "</div>" +
    '<textarea id="bk-text" rows="14" readonly></textarea>' +
    '<div class="tools"><span id="bk-said"></span><span id="bk-chars"></span><button data-empty-basket="1">空にする</button><button id="bk-copy" class="primary" title="⌘↩ でもコピーできます">指示をコピー</button></div>';
  showInstruction();
}
function copyInstruction() { if (basket.size) copyText($("bk-text").value, $("bk-said"), $("bk-text")); }
function openBasket(open) { basketOpen = open; $("basket").hidden = !open; renderBasket(); }

// ---- verdicts: kept in this browser, handed over as one text ---------------
const keptVerdicts = store.read(L.STORE.verdicts, {});
const verdicts = {};
for (const f of FEATURES) {
  const v = keptVerdicts && keptVerdicts[f.id];
  if (v && L.VERDICTS.some(known => known[0] === v.verdict)) verdicts[f.id] = { verdict: v.verdict, note: typeof v.note === "string" ? v.note : "" };
}
function saveVerdicts() { keepOrSay(L.STORE.verdicts, verdicts); }
function setVerdict(id, verdict) {
  if (!featureById.has(id)) return;
  if (verdicts[id] && verdicts[id].verdict === verdict) delete verdicts[id];
  else verdicts[id] = { verdict, note: verdicts[id] ? verdicts[id].note : "" };
  saveVerdicts();
}
function verdictWord(verdict) { return (L.VERDICTS.find(v => v[0] === verdict) || ["", ""])[1]; }
function verdictBar(f) {
  const mine = verdicts[f.id];
  const counts = L.reviewPacket(verdicts, FEATURES).counts;
  return '<div class="vbar" id="vbar"><span class="vl">この機能の判定</span>' + L.VERDICTS.map((v, n) =>
    '<button class="vd ' + v[0] + '" data-verdict="' + v[0] + '" aria-pressed="' + (!!mine && mine.verdict === v[0]) + '" title="キー ' + (n + 1) + '">' + v[1] + "</button>").join("") +
    '<input id="vd-note" type="text" placeholder="メモ（判定を付けると書けます）" aria-label="判定のメモ" value="' + esc(mine ? mine.note : "") + '"' + (mine ? "" : " disabled") + ">" +
    '<span class="what">この判定はこのブラウザにだけ保存されます</span><span class="vc">OK ' + counts.ok + " · 要修正 " + counts.fix + " · 保留 " + counts.hold + "</span>" +
    '<button id="vd-copy" class="ghost">レビュー結果をコピー</button><span id="vd-said" class="what"></span>' +
    '<textarea id="vd-text" rows="6" readonly hidden></textarea></div>';
}
function copyVerdicts() {
  const packet = L.reviewPacket(verdicts, FEATURES);
  if (!packet.text) { $("vd-said").textContent = "判定がまだありません"; return; }
  $("vd-text").value = packet.text;
  $("vd-said").textContent = packet.chars + " 文字";
  copyText(packet.text, $("vd-said"), $("vd-text"));
}

// ---- 現況 -----------------------------------------------------------------
const ST = M.status || { features: {}, phases: [], off_track: [], attention: [], next: [], next_total: 0, branches: [], domains: [] };
const ATTENTION = { decide: "判断・再開が必要", confirm: "人の確認待ち", carried: "残した指摘", gaps: "文書・写真・テストの欠け" };
function kpi(route, number, label, part, whole) {
  return '<button class="kpi" data-go="' + esc(route) + '"><b>' + number + "</b><span>" + label + "</span>" +
    (whole !== undefined ? '<i><em style="width:' + bar(part, whole) + '%"></em></i>' : "") + "</button>";
}
function numberChip(name) {
  const c = byName.get(name);
  return c ? '<button class="chip ' + c.group + '" data-open="' + esc(c.name) + '" title="' + esc(c.title + " — " + statusWord(c)) + '">' + esc(c.number || c.name) + "</button>" : "";
}
function attentionRow(kind, item) {
  if (kind === "gaps") return '<li><button data-feature="' + esc(item.feature) + '"><span class="x">' + esc(featureTitle(item.feature)) +
    '</span><span class="r">' + esc(item.text || "") + "</span></button></li>";
  const c = byName.get(item.change);
  return '<li><button data-open="' + esc(item.change) + '"' + (kind === "carried" ? ' data-tab="findings"' : "") + '><span class="n">' + esc(c ? c.number : "") +
    '</span><span class="x">' + esc(item.text || (c ? c.title : item.change)) + "</span>" + (item.key ? '<span class="r">' + esc(item.key) + "</span>" : "") + "</button></li>";
}
function renderOverview() {
  const f = ST.features || {};
  const ch = OV.changes || {};
  const pinnedPct = bar(OV.pinned || 0, OV.requirements || 0);
  let h = '<p class="lead">Operon がいまどこまで進んでいて、あなたが何をすればよいかを 1 枚にまとめています。数字や行を押すと、その記録が開きます。</p><div class="kpis">' +
    kpi("#features", f.total || 0, "機能（安定 " + (f.steady || 0) + " · 変更中 " + (f.changing || 0) + " · 判断待ち " + (f.waiting || 0) + "）", f.steady || 0, f.total || 0) +
    kpi("#requirements", OV.requirements || 0, "要件（テストあり " + (OV.pinned || 0) + " · " + pinnedPct + "%）", OV.pinned || 0, OV.requirements || 0) +
    kpi("#canvas", OV.screens || 0, "画面（写真あり " + (OV.photographed || 0) + "）", OV.photographed || 0, OV.screens || 0) +
    kpi("#changes", OV.changes_total || 0, "変更（要対応 " + (ch.needs || 0) + " · 進行中 " + (ch.flight || 0) + " · 完了 " + (ch.done || 0) + "）") +
    "</div>";
  h += '<section><h2>どこまで進んだか <small>進行中・要対応の変更が、いまどの段階にあるか</small></h2><div class="phases">' + ST.phases.map(p =>
    '<div class="phase' + (p.changes.length ? " has" : "") + '"><h4>' + esc(stageWord(p.stage)) + "</h4><b>" + p.changes.length + '</b><div class="chips">' +
    p.changes.map(numberChip).join("") + "</div></div>").join("") + "</div>" +
    (ST.off_track.length ? '<p class="warn">段階不明: <span class="chips inline">' + ST.off_track.map(numberChip).join("") + "</span></p>" : "") + "</section>";
  h += '<section><h2>あなたが対応するもの</h2><div class="atts">' + ST.attention.map(a =>
    '<div class="att' + (a.total ? " has " + a.kind : "") + '"><h3>' + esc(ATTENTION[a.kind] || a.kind) + " <b>" + a.total + "</b></h3>" +
    (a.items.length ? '<ul class="rows">' + a.items.map(item => attentionRow(a.kind, item)).join("") + "</ul>" +
      (a.total > a.items.length ? '<p class="more">ほか ' + (a.total - a.items.length) + " 件</p>" : "")
      : '<p class="empty">ありません</p>') + "</div>").join("") + "</div></section>";
  h += "<section><h2>進行中と次の一手 <small>(" + (ST.next_total || 0) + ")</small></h2>" + (ST.next.length ? '<ul class="nexts">' + ST.next.map(n => {
    const c = byName.get(n.name);
    return '<li><button data-open="' + esc(n.name) + '"><span class="num">' + esc(c ? c.number : "") + '</span><span class="ttl">' + esc(c ? c.title : n.name) +
      '</span><span class="pill flight">' + esc(stageWord(n.stage) || "段階不明") + '</span><span class="res">' + esc(n.resume || "（次の一手の記録がありません）") +
      '</span><span class="cov"><i><em style="width:' + bar(n.passed, n.total) + '%"></em></i>完了の条件 ' + n.passed + " / " + n.total + "</span></button></li>";
  }).join("") + "</ul>" + (ST.next_total > ST.next.length ? '<p class="more">ほか ' + (ST.next_total - ST.next.length) + ' 件 <button class="fchip" data-go="#changes">変更一覧で見る</button></p>' : "")
    : '<div class="empty">進行中の変更はありません</div>') + "</section>";
  h += '<div class="cols2"><section><h2>ドメイン別 <small>機能の状態と、テストで守られている要件</small></h2><div>' + (ST.domains.length ? ST.domains.map(d =>
    '<div class="dbar"><button class="fchip" data-domain="' + esc(d.id) + '">' + esc(domainTitle(d.id)) + '</button><span class="track2"><em style="width:' +
    bar(d.pinned, d.requirements) + '%"></em></span><small>' + d.pinned + " / " + d.requirements + " · 機能 " + d.features +
    (d.changing ? ' · <span class="badge changing">変更中 ' + d.changing + "</span>" : "") + (d.waiting ? ' · <span class="badge waiting">判断待ち ' + d.waiting + "</span>" : "") + "</small></div>").join("")
    : '<div class="empty">ドメインがありません</div>') + "</div></section>";
  h += "<section><h2>作業中のブランチ <small>(" + ST.branches.length + ")</small></h2>" + (ST.branches.length ? '<ul class="branches">' + ST.branches.map(b =>
    "<li><b>" + esc(b.branch || (b.here ? "このチェックアウト" : "（ブランチ名なし）")) + "</b>" + (b.here ? ' <span class="pill done">ここ</span>' : "") +
    '<span class="chips inline">' + b.changes.map(numberChip).join("") + '</span><div class="what">' + esc(b.path) + "</div></li>").join("") + "</ul>"
    : '<div class="empty">進行中の変更を持つブランチはありません</div>') + "</section></div>";
  const least = OV.least_pinned || [];
  h += '<div class="cols2"><section><h2>テストで守られていない要件が多い機能 <small>テストなしの件数</small></h2>' + (least.length ? '<ul class="rows">' + least.map(x =>
    '<li><button data-go="#feature/' + esc(x.feature) + '/reqs"><span class="n">' + x.bare + ' 件</span><span class="x">' + esc(featureTitle(x.feature)) +
    '</span><span class="r">' + esc(domainTitle((featureById.get(x.feature) || {}).domain)) + "</span></button></li>").join("") + "</ul>"
    : '<div class="empty">すべての要件にテストがあります</div>') + "</section>";
  const recent = M.changes.filter(c => c.group === "done").slice(0, 10);
  h += '<section><h2>最近入った変更</h2><ul class="rows">' + recent.map(c =>
    '<li><button data-open="' + esc(c.name) + '"><span class="n">' + esc(c.number) + '</span><span class="x">' + esc(c.title) +
    '</span><span class="r">' + esc(lastDate(c)) + "</span></button></li>").join("") + "</ul></section></div>";
  const missing = FLOW.screens.filter(s => !s.shot);
  h += "<section><h2>写真がない画面 <small>(" + missing.length + ")</small></h2>" + (M.shots_error ? '<p class="warn">' + esc(M.shots_error) + "</p>" : "") +
    (missing.length ? '<ul class="rows">' + missing.map(s => '<li><button data-screen="' + esc(s.id) + '"><span class="x">' + esc(s.title) +
      '</span><span class="r">' + esc(s.shot_problem || "") + "</span></button></li>").join("") + "</ul>" : '<div class="empty">すべての画面に写真があります</div>') +
    "</section>";
  $("v-overview").innerHTML = h;
}

// ---- 機能 ------------------------------------------------------------------
const RAIL = { brief: "概要", spec: "仕様", screens: "画面写真", requirements: "要件", tests: "テスト", live: "実機確認" };
const RAIL_STATE = { done: "済", partial: "一部", missing: "なし" };
const LAYER = { draw: "描画", text: "文言・色", core: "処理", test: "テスト", script: "スクリプト", doc: "文書" };
const reqCount = new Map(FEATURES.map(f => [f.id, REQS.filter(r => r.feature === f.id).length]));
function share(f) { return bar(f.pinned || 0, reqCount.get(f.id) || 0); }
function renderFeatureBadge(f) {
  const stage = f.stage ? " · " + stageWord(f.stage) : "";
  return '<span class="badge ' + f.badge + '">' + BADGE[f.badge] + esc(f.badge === "steady" ? "" : stage) + "</span>";
}
function featureCard(f) {
  const open = f.changes.filter(l => l.link !== "estimated" && ["needs", "flight"].includes(byName.get(l.name).group));
  return '<button class="fcard ' + f.badge + '" data-feature="' + esc(f.id) + '"><span class="t">' + esc(f.title) +
    '</span><span class="s">' + esc(f.summary) + '</span><span class="b">' + renderFeatureBadge(f) +
    "<span>要件 " + (f.pinned || 0) + "/" + (reqCount.get(f.id) || 0) + "</span>" +
    (open.length ? "<span>変更 " + esc(open.map(l => byName.get(l.name).number).join("・")) + "</span>" : "") + "</span></button>";
}
function wrapCard(f) { return '<div class="fwrap">' + featureCard(f) + basketBox("feature", f.id) + "</div>"; }
function railDots(f) {
  return '<span class="dots">' + (f.rail || []).map(step => '<i class="' + esc(step.state) + '" title="' + esc((RAIL[step.step] || step.step) + ": " + (RAIL_STATE[step.state] || step.state)) + '"></i>').join("") + "</span>";
}
function featureTile(f) {
  const shot = (f.flow_screens || []).find(id => SHOTS[id]);
  return '<div class="fwrap"><button class="ftile ' + f.badge + '" data-feature="' + esc(f.id) + '">' +
    (shot ? imgTag(SHOTS[shot], f.title + "の画面") : '<span class="plain">' + esc(f.title) + "</span>") +
    '<span class="t">' + esc(f.title) + '</span><span class="s">' + esc(f.summary) + '</span><span class="b">' + renderFeatureBadge(f) +
    '<span class="cov"><i><em style="width:' + share(f) + '%"></em></i>テスト ' + (f.pinned || 0) + "/" + (reqCount.get(f.id) || 0) + "</span>" + railDots(f) +
    "</span></button>" + basketBox("feature", f.id) + "</div>";
}
function lensRows(query) {
  const counts = L.lensCounts(FEATURES, query);
  return L.LENSES.map(lens => {
    const values = lens.values || Object.keys(counts[lens.id]).map(id => [id, id ? domainTitle(id) : "（ドメイン未設定）"]);
    const chosen = query[lens.id] || [];
    return '<div class="lensrow"><span class="ll">' + lens.label + "</span>" + values.map(v =>
      '<button class="lens' + (counts[lens.id][v[0]] ? "" : " none") + '" data-lens="' + lens.id + '" data-value="' + esc(v[0]) + '" aria-pressed="' + chosen.includes(v[0]) + '">' +
      esc(v[1]) + " <b>" + counts[lens.id][v[0]] + "</b></button>").join("") + "</div>";
  }).join("");
}
function renderFeatures(query) {
  $("p-catalogue-error").innerHTML = M.catalogue_error ? '<div class="box needs"><h3>' + esc(M.catalogue_error) +
    "</h3>製品の見方は出せません。変更一覧は読めます。</div>" : "";
  const narrowed = Object.keys(query).length > 0;
  $("ft-lenses").innerHTML = FEATURES.length ? lensRows(query) + (narrowed ? '<button class="fchip" data-go="#features">絞り込みを解除</button>' : "") : "";
  const shown = FEATURES.filter(f => L.featureMatches(f, query));
  $("ft-n").textContent = FEATURES.length ? "(" + shown.length + " / " + FEATURES.length + ")" : "";
  $("ft-tiles").innerHTML = shown.length ? shown.map(featureTile).join("")
    : '<div class="empty">' + (FEATURES.length ? "条件に合う機能はありません" : "機能の一覧がありません") + "</div>";
  const loose = M.unmapped.length;
  $("p-unmapped").innerHTML = (loose ? '<p class="lead">' + (M.changes.length - loose) + " 件の変更は機能に結び付いています。残りの " + loose +
    " 件は、開発の進め方や道具の変更など、機能に紐づかないもの（または記録に機能名がない古い変更）です。" +
    ' <button class="fchip" data-go="#changes">変更一覧で見る</button></p>' : "") +
    (M.features_omitted.length ? '<p class="warn">機能が多すぎるため、載せていません: ' + esc(M.features_omitted.join(", ")) + "</p>" : "");
}
function liveSteps(f) {
  return L.sectionOf(f.spec ? f.spec.text : "", "Seen in the running app").split("\n").map(l => l.trim()).filter(Boolean);
}
let reqBareOnly = false;
function featureSection(id, title, count, inner) {
  return '<section class="fsec" id="' + id + '"><h3>' + title + (count !== undefined ? " <small>" + count + "</small>" : "") + "</h3>" + inner + "</section>";
}
function specPart(f, heading) {
  const text = L.sectionOf(f.spec ? f.spec.text : "", heading);
  return text ? mdBlock(text) : '<p class="empty">仕様に記載がありません</p>';
}
function requirementCard(r, withFeature) {
  return '<div class="rcard' + (r.pinned ? "" : " bare") + '"><div class="rh"><span class="num">' + r.number + "</span>" +
    (withFeature ? chip("data-feature", r.feature, featureTitle(r.feature)) : "") +
    (r.pinned ? '<span class="pill done">テストあり</span>' : '<span class="pill needs">テストなし</span>') + basketBox("behaviour", r.feature + "#" + r.number) + "</div>" +
    '<div class="md">' + inline(r.text) + "</div>" + (r.tests.length ? '<div class="rt">' + r.tests.map(t => "<code>" + esc(t) + "</code>" +
      (r.missing.includes(t) ? ' <span class="pill needs">見つからない</span>' : "")).join(" ") + "</div>" : "") + "</div>";
}
function renderFeature(f, tab) {
  const rows = REQS.filter(r => r.feature === f.id);
  const pinned = rows.filter(r => r.pinned).length;
  const screens = (f.flow_screens || []).map(id => flowById.get(id)).filter(Boolean);
  const own = TERMS.filter(t => t.feature === f.id);
  const used = TERMS.filter(t => t.feature !== f.id && t.used_by.includes(f.id));
  const steps = liveSteps(f);
  const code = f.code || [];
  const files = code.reduce((n, layer) => n + layer.files.length, 0);
  const evidence = f.evidence || [];
  const sections = [["sec-screens", "画面", screens.length], ["sec-about", "概要"], ["sec-reqs", "要件", pinned + "/" + rows.length],
    ["sec-states", "状態と不変条件"], ["sec-live", "実アプリで確認", steps.length], ["sec-terms", "用語", own.length], ["sec-code", "コード", files],
    ["sec-evidence", "根拠", evidence.length], ["sec-decisions", "決定"], ["sec-changes", "変更", f.changes.length], ["sec-docs", "原文"]];
  let h = '<div class="fhead"><button class="back" data-go="#features">← 機能の一覧</button><h2>' + esc(f.title) + renderFeatureBadge(f) + "</h2>" +
    '<div class="meta2"><span>画面: <b>' + esc(screenLabel(f.screen)) + "</b></span><span>ドメイン: " + domainChip(f.domain) + "</span>" +
    "<span>要件 " + pinned + "/" + rows.length + " がテストあり</span><span>変更 " + f.changes.length + " 件</span>" + basketBox("feature", f.id) + "</div>" +
    '<div class="rail">' + (f.rail || []).map(step => '<span class="rs ' + esc(step.state) + '"><b>' + esc(RAIL[step.step] || step.step) + "</b><i>" +
      esc(RAIL_STATE[step.state] || step.state) + "</i></span>").join("") + "</div>";
  if (f.next) {
    const c = byName.get(f.next.change);
    h += '<div class="box ' + (f.badge === "waiting" ? "needs" : "") + '"><h3>次の一手 ' + (c ? numberChip(c.name) + " " + pill(c) : "") + "</h3>" +
      esc(f.next.resume || "（次の一手の記録がありません）") + "</div>";
  }
  h += "</div>";
  h += '<div class="fpage"><nav class="toc" id="f-toc">' + sections.map(s => '<a href="#' + "feature/" + esc(f.id) + "/" + s[0].slice(4) +
    '" data-sec="' + s[0] + '">' + s[1] + (s[2] !== undefined ? "<small>" + s[2] + "</small>" : "") + "</a>").join("") + "</nav><div>";
  h += featureSection("sec-screens", "画面", screens.length, screens.length ? '<p class="what">押すと、その画面を写真で開きます。</p><div class="strip">' +
    screens.map(screenThumb).join("") + "</div>" : '<p class="empty">この機能の画面はまだ流れにありません</p>');
  h += featureSection("sec-about", "概要", undefined, "<p>" + esc(f.summary) + "</p>" + (f.brief ? mdBlock(f.brief.text) : '<div class="box needs">brief.md がありません</div>'));
  const shownRows = reqBareOnly ? rows.filter(r => !r.pinned) : rows;
  h += featureSection("sec-reqs", "要件", pinned + " / " + rows.length + " がテストあり",
    '<div class="cov"><i><em style="width:' + bar(pinned, rows.length) + '%"></em></i>' + bar(pinned, rows.length) + "%" +
    ' <label class="toggle"><input type="checkbox" id="f-bare"' + (reqBareOnly ? " checked" : "") + "> テストの記録なしだけ</label></div>" +
    (shownRows.length ? '<div class="rcards">' + shownRows.map(r => requirementCard(r, false)).join("") + "</div>" : '<p class="empty">なし</p>'));
  h += featureSection("sec-states", "状態と不変条件", undefined, "<h4>状態</h4>" + specPart(f, "States") + "<h4>不変条件</h4>" + specPart(f, "Invariants"));
  h += featureSection("sec-live", "実アプリで確認", steps.length, steps.length ? '<p class="what">scripts/verify-app.py がこの手順でアプリを動かして確かめます。</p><ol class="steps2">' +
    steps.map(s => { const sp = s.indexOf(" "); return "<li><code>" + esc(sp < 0 ? s : s.slice(0, sp)) + "</code> " + esc(sp < 0 ? "" : s.slice(sp + 1)) + "</li>"; }).join("") + "</ol>"
    : '<p class="empty">この機能には、実アプリで確かめる手順がまだありません</p>');
  h += featureSection("sec-terms", "用語", own.length, '<div class="terms">' + (own.map(termCard).join("") || '<p class="empty">なし</p>') + "</div>" +
    "<h4>BRIEF が使っているほかの用語</h4>" + termChips(used.map(t => t.id)));
  h += featureSection("sec-code", "コード", files + " ファイル", files ? '<p class="what">層ごとのファイルです。「かご」に入れると、指示の「先に読むファイル」に載ります。</p>' +
    code.map(layer => "<h4>" + esc(LAYER[layer.layer] || layer.layer) + " <small>" + layer.files.length + '</small></h4><ul class="files">' + layer.files.map(file =>
      "<li>" + basketBox("file", file.path) + "<code>" + esc(file.path) + '</code><span class="about">' + esc(file.about || "") + "</span>" +
      (file.symbols.length ? '<span class="syms">' + file.symbols.map(s => "<code>" + esc(s) + "</code>").join(" ") + "</span>" : "") + "</li>").join("") + "</ul>").join("")
    : '<p class="empty">この機能のコードは一覧にも仕様にも書かれていません</p>');
  h += featureSection("sec-evidence", "根拠", evidence.length, evidence.length ? '<p class="what">この機能に入った新しい変更が、何を確かめて完了になったかです。</p>' +
    '<table class="all"><thead><tr><th>変更</th><th>確かめた項目</th><th>残した指摘</th><th>承認</th></tr></thead><tbody>' + evidence.map(e => {
      const c = byName.get(e.change);
      return '<tr class="pick" data-open="' + esc(e.change) + '"><td><span class="num">' + esc(c ? c.number : "") + "</span> " + esc(c ? c.title : e.change) +
        "</td><td>" + e.obligations + "</td><td>" + e.findings + "</td><td>" + e.approvals + "</td></tr>";
    }).join("") + "</tbody></table>" : '<p class="empty">この機能に結び付いた完了済みの変更はまだありません</p>');
  h += featureSection("sec-decisions", "決定", undefined, specPart(f, "Decisions inherited"));
  h += featureSection("sec-changes", "変更", f.changes.length, "<h4>変更履歴</h4>" + featureProgress(f));
  h += featureSection("sec-docs", "原文", undefined, '<div class="docbtns">' +
    '<button class="ghost" data-doc="brief|' + esc(f.id) + '">BRIEF（人向けの説明）</button>' +
    '<button class="ghost" data-doc="spec|' + esc(f.id) + '">SPEC（AI 向けの仕様・英語）</button>' +
    (f.manual_text ? '<button class="ghost" data-doc="manual|' + esc(f.id) + '">マニュアル「' + esc(f.manual) + "」</button>" : "") + "</div>");
  $("v-feature").innerHTML = h + "</div></div>" + verdictBar(f);
  if (tab) { const target = $("sec-" + tab); if (target) target.scrollIntoView(); }
  watchToc();
}
function featureProgress(f) {
  const link = k => f.changes.filter(l => l.link === k).length;
  let h = '<div class="box ' + (f.badge === "waiting" ? "needs" : "") + '"><h3>いまの状態: ' + renderFeatureBadge(f) + "</h3>" +
    (f.badge === "steady" ? "この機能に手を入れている変更は、いまありません。" :
     f.badge === "changing" ? "この機能に手を入れている変更が進んでいます。" : "この機能に触れる変更が、あなたの判断を待っています。") + "</div>";
  if (!f.changes.length) return h + '<p class="empty">この機能に触れた変更の記録はありません</p>';
  h += '<p class="what">「推定」は、変更の記録に機能名がないため、計画に書かれたファイルや題の言葉から割り出したものです。外れていることがあります。</p>';
  h += "<p>これまでにこの機能へ触れた変更 " + f.changes.length + " 件（記録 " + link("recorded") + " · 計画のファイルから推定 " +
    link("inferred") + " · 題から推定 " + link("estimated") + "）。新しい順です。</p>";
  h += '<ol class="timeline">' + f.changes.map(l => {
    const c = byName.get(l.name);
    return '<li class="' + c.group + '"><button data-open="' + esc(c.name) + '"><div class="head"><span class="num">' + esc(c.number) +
      '</span><span class="ttl">' + esc(c.title) + "</span>" + pill(c) + "<span>" + esc(stageWord(c.stage)) + "</span></div>" +
      '<div class="how">' + esc(LINK[l.link]) + " · " + esc(routeWord(c)) + (place(c) ? " · " + esc(place(c)) : "") + "</div></button></li>";
  }).join("") + "</ol>";
  return h;
}
let tocObserver = null;
function watchToc() {
  if (tocObserver) tocObserver.disconnect();
  if (typeof IntersectionObserver === "undefined") return;
  tocObserver = new IntersectionObserver(entries => {
    for (const e of entries) if (e.isIntersecting) {
      for (const a of document.querySelectorAll(".toc a")) a.classList.toggle("on", a.dataset.sec === e.target.id);
    }
  }, { rootMargin: "-120px 0px -70% 0px" });
  for (const s of document.querySelectorAll(".fsec")) tocObserver.observe(s);
}

// ---- ドメイン --------------------------------------------------------------
function renderDomains() {
  $("dm-error").innerHTML = errorBox(M.domains_error, M.domains_notice);
  $("dm-list").innerHTML = DOMAINS.length ? DOMAINS.map(d => {
    const fs = d.features.map(id => featureById.get(id)).filter(Boolean);
    const moving = fs.filter(f => f.badge !== "steady").length;
    const per = (OV.per_domain || []).find(x => x.id === d.id) || { pinned: 0, requirements: 0 };
    return '<button class="dcard" data-domain="' + esc(d.id) + '"><span class="t">' + esc(d.title) + '</span><span class="s">' +
      esc(d.summary) + '</span><span class="b"><span>機能 ' + fs.length + "</span><span>画面 " + (d.screens || []).length + "</span><span>コード " + d.modules.length +
      "</span><span>要件 " + per.pinned + "/" + per.requirements + "</span><span>変更 " + d.changes.length + "</span>" +
      (moving ? '<span class="badge changing">動いている機能 ' + moving + "</span>" : '<span class="badge">安定</span>') +
      "</span></button>";
  }).join("") : "";
}
function renderDomain(d) {
  const per = (OV.per_domain || []).find(x => x.id === d.id) || { pinned: 0, requirements: 0 };
  let h = '<div class="fhead"><button class="back" data-go="#domains">← ドメインの一覧</button><h2>' + esc(d.title) + "</h2>" +
    '<div class="meta2"><span>' + esc(d.summary) + "</span>" + basketBox("domain", d.id) + "</div></div>";
  h += '<div class="split2"><div class="box"><h3>担当すること</h3>' + esc(d.owns) + '</div><div class="box"><h3>担当しないこと</h3>' + esc(d.excludes) + "</div></div>";
  h += "<h3>隣のドメイン</h3>" + (d.adjacent.length ? '<span class="fchips">' + d.adjacent.map(domainChip).join("") + "</span>" : '<p class="empty">なし</p>');
  h += '<h3>要件 <small>' + per.pinned + " / " + per.requirements + ' がテストあり</small></h3><div class="cov"><i><em style="width:' + bar(per.pinned, per.requirements) + '%"></em></i>' +
    bar(per.pinned, per.requirements) + '% <button class="fchip" data-go="#requirements">要件の表で見る</button></div>';
  h += "<h3>画面 <small>" + (d.screens || []).length + '</small></h3><div class="thumbs">' + (d.screens || []).map(id => flowById.get(id)).filter(Boolean).map(screenThumb).join("") + "</div>";
  h += "<h3>機能 <small>" + d.features.length + '</small></h3><div class="fcards">' + d.features.map(id => featureById.get(id)).filter(Boolean).map(wrapCard).join("") + "</div>";
  h += "<h3>コード <small>" + d.modules.length + ' ファイル</small></h3><table class="all"><tbody>' + d.modules.map(m =>
    "<tr><td><code>" + esc(m.path) + "</code></td><td>" + esc(m.about) + "</td></tr>").join("") + "</tbody></table>";
  h += "<h3>用語</h3>" + termChips(TERMS.filter(t => t.domain === d.id).map(t => t.id));
  h += "<h3>変更 <small>" + d.changes.length + "</small></h3>";
  h += d.changes.length ? '<ol class="timeline">' + d.changes.slice(0, 60).map(name => {
    const c = byName.get(name);
    return '<li class="' + c.group + '"><button data-open="' + esc(c.name) + '"><div class="head"><span class="num">' + esc(c.number) +
      '</span><span class="ttl">' + esc(c.title) + "</span>" + pill(c) + "</div></button></li>";
  }).join("") + "</ol>" + (d.changes.length > 60 ? '<p class="empty">ほか ' + (d.changes.length - 60) + " 件</p>" : "")
    : '<p class="empty">このドメインの機能に触れた変更の記録はありません</p>';
  $("v-domain").innerHTML = h;
}

// ---- 画面: 写真で歩く ---------------------------------------------------------
const walk = new L.Walk();
let showAreas = false;
function wayHow(w) {
  const parts = [];
  if (w.chord) parts.push(w.chord);
  if (w.label) parts.push("「" + w.label + "」");
  if (w.click) parts.push(w.click + "を押す");
  return parts.join(" / ");
}
function bandsInOrder() {
  const order = FLOW.bands.map(b => b.id);
  return order.map(id => ({ id, screens: FLOW.screens.filter(s => s.screen === id).sort((a, b) => a.y - b.y || a.x - b.x) }));
}
function renderScreen(s) {
  $("sc-list").innerHTML = bandsInOrder().map(b => "<h4>" + esc(screenLabel(b.id)) + "</h4>" + b.screens.map(x =>
    '<button data-screen="' + esc(x.id) + '"' + (x.id === s.id ? ' class="on" aria-current="true"' : "") + ">" +
    (SHOTS[x.id] ? imgTag(SHOTS[x.id], "") : '<span class="ph"></span>') +
    "<span>" + esc(x.title) + "</span></button>").join("")).join("");
  const outs = s.out.map(i => [i, FLOW.ways[i]]);
  let win;
  if (SHOTS[s.id] && s.shot) {
    win = '<div class="desk"><div class="win' + (showAreas ? " show" : "") + '" id="sc-win">' + imgTag(SHOTS[s.id], s.title + "の画面", ' draggable="false"') +
      outs.filter(([, w]) => w.area).map(([i, w]) => {
        const p = L.areaPercent(w.area, s.shot);
        return '<button class="area" data-way="' + i + '" style="left:' + p.left.toFixed(3) + "%;top:" + p.top.toFixed(3) + "%;width:" + p.width.toFixed(3) +
          "%;height:" + p.height.toFixed(3) + '%" aria-label="' + esc(flowTitle(w.to) + "へ") + '"><span class="tip">→ ' + esc(flowTitle(w.to)) + "</span></button>";
      }).join("") + "</div></div>";
  } else {
    win = '<div class="desk"><div class="win empty"><span class="lights"><i></i><i></i><i></i></span><span class="wt">' + esc(s.title) + "</span><span>" +
      esc(s.shot_problem.startsWith("写真") ? s.shot_problem : "写真がありません: " + (s.shot_problem || "まだ撮影されていません")) + "</span>" +
      (outs.length ? "<span>右の「出ていく道」から次の画面へ進めます。</span>" : "") + "</div></div>";
  }
  $("sc-window").innerHTML = win;
  const keyed = outs.filter(([, w]) => w.chord && !w.area);
  $("sc-keys").innerHTML = keyed.length ? '<span class="what">キーで:</span>' + keyed.map(([i, w]) =>
    '<button class="keyway" data-way="' + i + '"><kbd>' + esc(w.chord) + "</kbd>" + esc(flowTitle(w.to)) + "</button>").join("") : "";
  $("sc-trail").innerHTML = walk.stack.map((id, n) => (n ? "<span>›</span>" : "") + '<button data-trail="' + n + '"' + (n === walk.at ? ' class="on"' : "") + ">" +
    esc(flowTitle(id)) + "</button>").join("");
  $("sc-back").disabled = !walk.canBack();
  $("sc-fwd").disabled = !walk.canForward();
  const wayLine = (i, w, other) => '<li><button data-way="' + i + '"' + (other === w.from ? ' data-screen="' + esc(other) + '"' : "") +
    '><span class="to">' + esc(flowTitle(other)) + '</span><span class="how">' + esc(wayHow(w)) + (w.area ? " · 写真の上で押せます" : "") + "</span>" +
    (w.note ? '<span class="how">' + esc(w.note) + "</span>" : "") + (w.unseen ? '<span class="un">' + esc(w.unseen) + "</span>" : "") + "</button></li>";
  let side = "<h3>" + esc(s.title) + '</h3><div class="what">' + esc(KIND[s.kind] || s.kind) + " · " + esc(screenLabel(s.screen)) + "</div>" +
    (s.shot ? '<div class="taken">' + taken(s) + "</div>" : "") + basketBox("screen", s.id);
  if (s.note) side += "<p>" + esc(s.note) + "</p>";
  if (s.parent) side += "<p>中にある場所: " + chip("data-screen", s.parent, flowTitle(s.parent)) + "</p>";
  side += "<h4>出ていく道 <small>" + outs.length + "</small></h4>" + (outs.length ? '<ul class="ways2">' + outs.map(([i, w]) => wayLine(i, w, w.to)).join("") + "</ul>" : '<p class="empty">なし</p>');
  side += "<h4>入ってくる道 <small>" + s.in.length + "</small></h4>" + (s.in.length ? '<ul class="ways2">' + s.in.map(i => {
    const w = FLOW.ways[i];
    return '<li><button data-screen="' + esc(w.from) + '"><span class="to">' + esc(flowTitle(w.from)) + '</span><span class="how">' + esc(wayHow(w)) + "</span></button></li>";
  }).join("") + "</ul>" : '<p class="empty">なし</p>');
  side += "<h4>機能</h4>" + featureChipsOf(s.features);
  side += "<h4>この画面に出ている用語 <small>" + (s.terms_on || []).length + "</small></h4>" + termChips(s.terms_on || []);
  $("sc-side").innerHTML = side;
}
function takeWay(index) {
  const w = FLOW.ways[index];
  if (!w) return;
  walk.visit(w.to);
  go({ view: "screen", id: w.to });
}

// ---- 全体図 ----------------------------------------------------------------
const CV = { k: 0.5, tx: 20, ty: 20, laid: null, fitted: false };
const NODE_W = 240, NODE_H = 154, CELL_W = 280, CELL_H = 210, LANE_LEFT = 190, PER_ROW = 6;
const SVGNS = "http://www.w3.org/2000/svg";
function svgNode(name, attrs, text) {
  const node = document.createElementNS(SVGNS, name);
  for (const [k, v] of Object.entries(attrs)) node.setAttribute(k, String(v));
  if (text !== undefined) node.textContent = text;
  return node;
}
// A way that every page offers (a page key, ⌘K, ⌘N) is one fact repeated from
// each page; drawn once each it buries the rest, so it is drawn only for the
// screen under the pointer.
const PAGES = FLOW.screens.filter(s => s.kind === "page").map(s => s.id);
const GLOBAL = new Set();
(function findEverywhere() {
  const groups = new Map();
  FLOW.ways.forEach((w, i) => {
    if (!w.key) return;
    const sig = w.to + "|" + w.key;
    if (!groups.has(sig)) groups.set(sig, []);
    groups.get(sig).push(i);
  });
  for (const [sig, list] of groups) {
    const from = new Set(list.map(i => FLOW.ways[i].from));
    const to = sig.split("|")[0];
    if (PAGES.length > 1 && PAGES.every(p => p === to || from.has(p))) list.forEach(i => GLOBAL.add(i));
  }
})();
function layCanvas() {
  if (CV.laid) return CV.laid;
  const pos = new Map(), lanes = [];
  let top = 30;
  for (const b of bandsInOrder()) {
    if (!b.screens.length) continue;
    const rows = Math.ceil(b.screens.length / PER_ROW);
    lanes.push({ id: b.id, y: top - 20, h: rows * CELL_H + 10 });
    b.screens.forEach((s, n) => pos.set(s.id, { x: LANE_LEFT + (n % PER_ROW) * CELL_W, y: top + Math.floor(n / PER_ROW) * CELL_H }));
    top += rows * CELL_H + 50;
  }
  CV.laid = { pos, lanes, w: LANE_LEFT + PER_ROW * CELL_W + 20, h: top };
  return CV.laid;
}
function renderCanvas() {
  $("fl-error").innerHTML = errorBox(M.flow_error, M.flow_notice) + (M.shots_error ? '<p class="warn">' + esc(M.shots_error) + "</p>" : "");
  const holder = $("cv");
  if (holder.dataset.drawn) { applyCanvas(); return; }
  holder.dataset.drawn = "1";
  const lay = layCanvas();
  const svg = svgNode("svg", { role: "img", "aria-label": "画面の全体図" });
  const vp = svgNode("g", { id: "cv-vp" });
  for (const lane of lay.lanes) {
    vp.appendChild(svgNode("rect", { x: 10, y: lane.y, width: lay.w - 20, height: lane.h, rx: 16, class: "lane" }));
    vp.appendChild(svgNode("text", { x: 28, y: lane.y + 34, class: "lanet" }, screenLabel(lane.id)));
  }
  const edges = svgNode("g", {});
  FLOW.ways.forEach((w, i) => {
    const a = lay.pos.get(w.from), b = lay.pos.get(w.to);
    if (!a || !b) return;
    const ax = a.x + NODE_W / 2, bx = b.x + NODE_W / 2;
    const down = b.y > a.y, same = a.y === b.y;
    const sy = same ? a.y : a.y + (down ? NODE_H + 30 : 0), ey = same ? b.y : b.y + (down ? 0 : NODE_H + 30);
    const bend = same ? -60 - Math.abs(bx - ax) / 8 : (ey - sy) / 2;
    const d = "M" + ax + "," + sy + " C" + ax + "," + (sy + bend) + " " + bx + "," + (same ? ey + bend : ey - bend) + " " + bx + "," + ey;
    const g = svgNode("g", { "data-from": w.from, "data-to": w.to, class: GLOBAL.has(i) ? "global" : "" });
    g.appendChild(svgNode("path", { d, class: "e" }));
    const c2y = same ? ey + bend : ey - bend, len = Math.hypot(0, ey - c2y) || 1, uy = (ey - c2y) / len;
    g.appendChild(svgNode("path", { d: "M" + bx + "," + ey + " L" + (bx - 6) + "," + (ey - uy * 11) + " L" + (bx + 6) + "," + (ey - uy * 11) + " z", class: "e hd" }));
    g.appendChild(svgNode("title", {}, flowTitle(w.from) + " → " + flowTitle(w.to) + "  " + wayHow(w)));
    if (GLOBAL.has(i)) g.style.display = "none";
    edges.appendChild(g);
  });
  vp.appendChild(edges);
  for (const s of FLOW.screens) {
    const p = lay.pos.get(s.id);
    if (!p) continue;
    const g = svgNode("g", { class: "nd", "data-node": s.id, tabindex: 0 });
    g.appendChild(svgNode("rect", { x: p.x - 4, y: p.y - 4, width: NODE_W + 8, height: NODE_H + 40, rx: 10, class: "fr" }));
    if (SHOTS[s.id]) g.appendChild(svgNode("image", { href: SHOTS[s.id], x: p.x, y: p.y, width: NODE_W, height: NODE_H, preserveAspectRatio: "xMidYMid slice" }));
    else {
      g.appendChild(svgNode("rect", { x: p.x, y: p.y, width: NODE_W, height: NODE_H, rx: 6, class: "phr" }));
      g.appendChild(svgNode("text", { x: p.x + 12, y: p.y + NODE_H / 2, class: "pht" }, "写真がありません"));
    }
    g.appendChild(svgNode("text", { x: p.x + 4, y: p.y + NODE_H + 22 }, s.title));
    g.appendChild(svgNode("text", { x: p.x + NODE_W - 4, y: p.y + NODE_H + 22, class: "k", "text-anchor": "end" }, KIND[s.kind] || s.kind));
    g.appendChild(svgNode("title", {}, s.title));
    vp.appendChild(g);
  }
  svg.appendChild(vp);
  holder.appendChild(svg);
  if (!CV.fitted) { fitCanvas(); CV.fitted = true; }
  applyCanvas();
}
function applyCanvas() {
  const vp = $("cv-vp");
  if (vp) vp.setAttribute("transform", "translate(" + CV.tx + "," + CV.ty + ") scale(" + CV.k + ")");
  $("cv-pct").textContent = Math.round(CV.k * 100) + "%";
}
function fitCanvas() {
  const lay = layCanvas(), box = $("cv").getBoundingClientRect();
  const w = box.width || 1200, h = box.height || 700;
  // Fitted to the width, so the pictures stay readable; the height scrolls.
  CV.k = Math.max(0.15, Math.min(1.6, (w / lay.w) * 0.98));
  CV.tx = (w - lay.w * CV.k) / 2; CV.ty = 10;
}
function zoomAt(factor, cx, cy) {
  const k = Math.max(0.15, Math.min(2.5, CV.k * factor));
  CV.tx = cx - (cx - CV.tx) * (k / CV.k); CV.ty = cy - (cy - CV.ty) * (k / CV.k); CV.k = k;
  applyCanvas();
}
function focusNode(id) {
  const holder = $("cv");
  holder.classList.toggle("focus", !!id);
  for (const g of holder.querySelectorAll("g[data-from]")) {
    const out = g.dataset.from === id, into = g.dataset.to === id;
    for (const p of g.querySelectorAll("path")) { p.classList.toggle("out", out); p.classList.toggle("in", into && !out); }
    if (g.classList.contains("global")) g.style.display = id && (out || into) ? "" : "none";
  }
  const near = new Set([id]);
  if (id) for (const w of FLOW.ways) { if (w.from === id) near.add(w.to); if (w.to === id) near.add(w.from); }
  for (const n of holder.querySelectorAll(".nd")) { n.classList.toggle("dim", !!id && !near.has(n.dataset.node)); n.classList.toggle("on", n.dataset.node === id); }
}

// ---- 用語集 -----------------------------------------------------------------
let termSel = null;
function termCard(t) {
  const row = (label, value) => value ? "<dt>" + label + "</dt><dd>" + value + "</dd>" : "";
  const onScreens = FLOW.screens.filter(s => (s.terms_on || []).includes(t.id)).map(s => s.id);
  return '<div class="term' + (termSel && termSel.id === t.id ? " sel" : "") + '" id="term-' + esc(t.id) + '"><div class="th"><span class="tw">' + esc(t.term) + "</span>" +
    (t.en ? '<span class="en">' + esc(t.en) + "</span>" : "") + basketBox("term", t.id) + "</div><p>" + esc(t.meaning) + '</p><dl class="kv">' +
    row("画面での表記", t.ui.map(u => "「" + esc(u) + "」").join(" ")) +
    row("コード", t.code.map(c => "<code>" + esc(c) + "</code>").join(" ")) +
    row("使わない言葉", t.avoid.map(a => '<span class="avoid">' + esc(a) + "</span>").join(" ")) +
    row("関連語", t.see.length ? termChips(t.see) : "") +
    row("機能", chip("data-feature", t.feature, featureTitle(t.feature)) + " " + domainChip(t.domain)) +
    row("出ている画面", onScreens.length ? '<span class="fchips">' + onScreens.map(id => chip("data-screen", id, flowTitle(id))).join("") + "</span>" : "") +
    row("使っている BRIEF", t.used_by.length ? featureChipsOf(t.used_by) : "") + "</dl></div>";
}
function termMatches(t) {
  return !query || [t.term, t.id, t.en, t.meaning, t.ui.join(" "), t.code.join(" "), t.avoid.join(" ")].join(" ").toLowerCase().includes(query);
}
function renderGlossary(byScreen) {
  $("gl-mode").innerHTML = '<button data-go="#glossary" aria-pressed="' + !byScreen + '">用語順</button><button data-go="#glossary/screens" aria-pressed="' + byScreen + '">画面別</button>';
  $("gl-error").innerHTML = errorBox(M.terms_error, M.terms_notice);
  if (byScreen) {
    const pictured = FLOW.screens.filter(s => s.shot);
    $("gl-h").innerHTML = "画面に出ている用語 <small>(" + pictured.length + " 画面)</small>";
    $("gl-list").innerHTML = pictured.length ? pictured.map(s => '<div class="screen"><button class="thumb" data-screen="' + esc(s.id) + '">' + thumbImg(s.id, s.title) +
      '</button><div><h3>' + esc(s.title) + ' <small style="font-weight:400;color:var(--soft)">' + (s.terms_on || []).length + " 語</small></h3>" +
      termChips(s.terms_on || []) + "</div></div>").join("") : '<div class="empty">' + esc(M.shots_error || "写真がまだありません") + "</div>";
    return;
  }
  const shown = TERMS.filter(termMatches);
  $("gl-h").innerHTML = '用語 <small id="gl-n">(' + shown.length + " / " + TERMS.length + ")</small>";
  if (!shown.length) { $("gl-list").innerHTML = TERMS.length ? '<div class="empty">該当する用語はありません</div>' : ""; return; }
  const order = DOMAINS.map(d => d.id).concat([""]);
  const groups = order.map(id => ({ id, terms: shown.filter(t => (domainById.has(t.domain) ? t.domain : "") === id) })).filter(g => g.terms.length);
  $("gl-list").innerHTML = groups.map(g => "<h3>" + esc(g.id ? domainTitle(g.id) : "ドメイン未設定") + " <small>" + g.terms.length + '</small></h3><div class="terms">' +
    g.terms.map(termCard).join("") + "</div>").join("");
  if (termSel) { const at = $("term-" + termSel.id); if (at) at.scrollIntoView({ block: "center" }); }
}

// ---- 要件 -------------------------------------------------------------------
let reqFeature = "", reqDomain = "", reqTests = "", reqTable = false;
function renderRequirements() {
  const filter = { domain: reqDomain, feature: reqFeature, tests: reqTests, words: query };
  const kept = change => REQS.filter(r => L.requirementMatches(r, Object.assign({}, filter, change), featureTitle(r.feature))).length;
  const opt = (value, label, current) => '<option value="' + esc(value) + '"' + (value === current ? " selected" : "") + ">" + esc(label) + "</option>";
  const lens = (attr, value, label, current, n) => '<button class="lens' + (n ? "" : " none") + '" ' + attr + '="' + esc(value) + '" aria-pressed="' + (value === current) + '">' +
    esc(label) + " <b>" + n + "</b></button>";
  $("rq-filters").innerHTML = '<div class="lensrow"><span class="ll">ドメイン</span>' + lens("data-rq-domain", "", "すべて", reqDomain, kept({ domain: "", feature: "" })) +
    DOMAINS.map(d => lens("data-rq-domain", d.id, d.title, reqDomain, kept({ domain: d.id, feature: "" }))).join("") + "</div>" +
    '<div class="lensrow"><span class="ll">テスト</span>' + lens("data-rq-tests", "", "すべて", reqTests, kept({ tests: "" })) +
    lens("data-rq-tests", "yes", "テストあり", reqTests, kept({ tests: "yes" })) + lens("data-rq-tests", "no", "テストなし", reqTests, kept({ tests: "no" })) + "</div>" +
    '<div class="lensrow"><span class="ll">機能</span><select id="rq-feature" aria-label="機能">' + opt("", "すべての機能", reqFeature) +
    FEATURES.filter(f => !reqDomain || f.domain === reqDomain).map(f => opt(f.id, f.title, reqFeature)).join("") + "</select>" +
    '<span class="filters right"><button data-rq-mode="cards" aria-pressed="' + !reqTable + '">カード</button><button data-rq-mode="table" aria-pressed="' + reqTable + '">表</button></span></div>';
  const rows = REQS.filter(r => L.requirementMatches(r, filter, featureTitle(r.feature)));
  const bare = REQS.filter(r => !r.pinned).length;
  $("rq-n").textContent = "(" + rows.length + " / " + REQS.length + " · テストの記録なし " + bare + " 件)";
  if (!rows.length) { $("rq-list").innerHTML = '<div class="empty">一致するものはありません</div>'; return; }
  if (reqTable) {
    $("rq-list").innerHTML = '<table class="all rq"><thead><tr><th></th><th>機能</th><th>番号</th><th>内容</th><th>テスト</th></tr></thead><tbody>' +
      rows.map(r => '<tr class="' + (r.pinned ? "" : "bare") + '"><td>' + basketBox("behaviour", r.feature + "#" + r.number) + "</td><td>" + chip("data-feature", r.feature, featureTitle(r.feature)) +
        '</td><td class="num">' + r.number + '</td><td class="md">' + inline(r.text) + "</td><td>" +
        (r.tests.length ? r.tests.map(t => "<code>" + esc(t) + "</code>" + (r.missing.includes(t) ? ' <span class="pill needs">見つからない</span>' : "")).join("<br>")
          : '<span class="pill needs">テストの記録なし</span>') + "</td></tr>").join("") + "</tbody></table>";
    return;
  }
  const groups = new Map();
  for (const r of rows) { if (!groups.has(r.feature)) groups.set(r.feature, []); groups.get(r.feature).push(r); }
  $("rq-list").innerHTML = Array.from(groups.entries()).map(([id, mine]) =>
    '<div class="rgroup"><h3>' + chip("data-feature", id, featureTitle(id)) + " <small>" + esc(domainTitle((featureById.get(id) || {}).domain)) + " · テストあり " +
    mine.filter(r => r.pinned).length + " / " + mine.length + '</small></h3><div class="rcards">' + mine.map(r => requirementCard(r, false)).join("") + "</div></div>").join("");
}

// ---- 作業 -------------------------------------------------------------------
const WORKS = M.work || { columns: [], cards: [] };
const BACKLOG = M.backlog || { asked: false, items: [], total: 0, error: "", notice: "" };
const COLUMN = { backlog: "バックログ", held: "着手中", flight: "進行中", you: "あなたの対応待ち", done: "最近の完了" };
const OWNER = { person: "あなた", ai: "AI", none: "" };
const ISSUE_KIND = { problem: "問題", improvement: "改善", remaining: "残り作業" };
function workCard(card) {
  if (card.change) {
    const c = byName.get(card.change);
    return '<button class="wcard ' + esc(card.column) + '" data-open="' + esc(card.change) + '"><span class="wh"><span class="num">' + esc(c ? c.number : "") +
      '</span><span class="wt">' + esc(card.title) + "</span></span>" + (card.stage && card.column !== "done" ? '<span class="pill ' + (c ? c.group : "") + '">' + esc(stageWord(card.stage)) + "</span>" : "") +
      (card.text && card.column !== "done" ? '<span class="wx">' + esc(card.text) + "</span>" : "") + "</button>";
  }
  const link = /^https:\/\/github\.com\//.test(String(card.url)) ? ' <a href="' + esc(card.url) + '" target="_blank" rel="noopener noreferrer">GitHub で開く</a>' : "";
  return '<details class="wcard ' + esc(card.column) + '"><summary><span class="wh"><span class="num">#' + esc(card.issue) + '</span><span class="wt">' + esc(card.title) +
    '</span></span><span class="wm"><span class="pill">' + esc(ISSUE_KIND[card.kind] || card.kind) + "</span><span>スコア " + esc(card.score) + "</span>" +
    (card.held === "unknown" ? '<span class="warn">着手中かどうか不明</span>' : card.held ? "<span>着手: " + esc(card.held) + "</span>" : "") + "</span></summary>" +
    '<div class="wb">' + (card.text ? mdBlock(card.text) : '<p class="empty">本文がありません</p>') + '<div class="tools">' + basketBox("issue", card.issue) +
    '<span class="said"></span><button data-issue-copy="' + esc(card.issue) + '">指示をコピー</button>' + link + "</div></div></details>";
}
function renderWork(query) {
  const who = (query.who || [""])[0];
  const list = (query.view || [""])[0] === "list";
  const link = change => esc(L.formatRoute({ view: "work", query: Object.assign({}, query, change) }));
  $("wk-filters").innerHTML = [["me", "自分"], ["ai", "AI"], ["", "すべて"]].map(w =>
    '<button data-go="' + link({ who: w[0] ? [w[0]] : [] }) + '" aria-pressed="' + (who === w[0]) + '">' + w[1] + "</button>").join("") +
    '<span class="gap"></span><button data-go="' + link({ view: [] }) + '" aria-pressed="' + !list + '">ボード</button>' +
    '<button data-go="' + link({ view: ["list"] }) + '" aria-pressed="' + list + '">一覧</button>';
  const unread = !BACKLOG.asked ? 'バックログは読み込んでいません。<code>--backlog</code> を付けて作り直すと表示されます' : BACKLOG.error;
  $("wk-notice").innerHTML = BACKLOG.notice ? '<p class="warn">' + esc(BACKLOG.notice) + "</p>" : "";
  const cards = L.boardFilter(WORKS.cards, who);
  if (list) {
    $("wk-board").innerHTML = (unread ? '<p class="' + (BACKLOG.asked ? "warn" : "what") + '">' + (BACKLOG.asked ? esc(unread) : unread) + "</p>" : "") +
      (cards.length ? '<table class="all"><thead><tr><th>列</th><th>番号</th><th>題</th><th>担当</th><th>段階・スコア</th></tr></thead><tbody>' + cards.map(card => {
        const c = card.change ? byName.get(card.change) : null;
        return "<tr" + (c ? ' class="pick" data-open="' + esc(card.change) + '"' : "") + "><td>" + esc(COLUMN[card.column] || card.column) + '</td><td class="num">' +
          esc(c ? c.number : "#" + card.issue) + "</td><td>" + esc(card.title) + "</td><td>" + esc(OWNER[card.owner] || "") + "</td><td>" +
          esc(c ? stageWord(card.stage) : "スコア " + card.score) + "</td></tr>";
      }).join("") + "</tbody></table>" : '<div class="empty">ありません</div>');
    return;
  }
  $("wk-board").innerHTML = '<div class="board">' + WORKS.columns.map(name => {
    const mine = cards.filter(card => card.column === name);
    const fromBacklog = name === "backlog" || name === "held";
    return '<div class="bcol ' + esc(name) + '"><h3>' + esc(COLUMN[name] || name) + " <small>" + mine.length + "</small></h3>" +
      (fromBacklog && unread ? '<p class="' + (BACKLOG.asked ? "warn" : "what") + '">' + (BACKLOG.asked ? esc(unread) : unread) + "</p>"
        : mine.length ? mine.map(workCard).join("") : '<p class="empty">ありません</p>') + "</div>";
  }).join("") + "</div>";
}

// ---- 進め方 -----------------------------------------------------------------
const PROCESS = M.process || { stages: [], routes: [], surfaces: [], methods: [], error: "進め方の表を読めません: docs/sdlc/routes.yaml" };
const AUTONOMY = [["paused", "止まって確認", "触れると、セッションは進める前に人の確認を待ちます"],
  ["supervised", "見守りつき", "セッションが進め、人があとから確かめます"], ["full", "任せる", "セッションが最後まで進めます"]];
const ORDER = [["intent", "意図", "何に困っているか"], ["spec", "仕様", "どう振る舞うべきか"], ["readiness", "準備の確認", "仕様だけで作れるか"],
  ["plan", "計画", "どのファイルをどの順で"], ["build", "実装", "テストから書く"], ["evidence", "根拠", "テスト・変異・検証役"], ["land", "取り込み", "CI のあと main へ"]];
function methodChips(names) {
  return names.length ? names.map(n => '<span class="pill">' + esc(n) + "</span>").join(" ") : '<span class="empty">なし</span>';
}
// A path in a narrow cell: its last part, the whole path on hover.
function fileName(path) {
  const parts = String(path).split("/");
  const last = parts.pop() === "SKILL.md" && parts.length ? parts.pop() + "/SKILL.md" : String(path).split("/").pop();
  return '<code title="' + esc(path) + '">' + esc(last) + "</code>";
}
function renderProcess() {
  const open = new Map(ST.phases.map(p => [p.stage, p.changes.length]));
  const most = Math.max(1, ...PROCESS.routes.map(r => r.changes));
  let h = PROCESS.error ? '<div class="box needs"><h3>' + esc(PROCESS.error) + "</h3>読めた表だけを描いています。</div>" : "";
  h += '<section><h2>依頼の種類と通る段階 <small>塗られた段階を通ります</small></h2>' + (PROCESS.routes.length ? '<div class="scroll"><table class="all pipe"><thead><tr><th>種類</th>' +
    PROCESS.stages.map(s => '<th class="st">' + esc(stageWord(s)) + (open.get(s) ? ' <span class="pill flight" title="いまこの段階にある変更">' + open.get(s) + "</span>" : "") + "</th>").join("") +
    "<th>やり直し上限</th><th>閉じる確認</th><th>必要な文書</th><th>求める根拠</th><th>これまでの変更</th></tr></thead><tbody>" + PROCESS.routes.map(r =>
      '<tr><td><b>' + esc(ROUTE[r.name] || r.name) + "</b> <code>" + esc(r.name) + "</code></td>" +
      PROCESS.stages.map(s => '<td class="st"><i class="' + (r.stages.includes(s) ? "on" : "off") + '" title="' + esc(stageWord(s) + (r.stages.includes(s) ? "を通ります" : "は通りません")) + '"></i></td>').join("") +
      "<td>" + esc(r.attempts) + " 回</td><td>" + fileName(r.gate) + "</td><td>" + (r.requires ? fileName(r.requires) : '<span class="empty">なし</span>') +
      "</td><td>" + methodChips(r.methods) + '</td><td><span class="bar"><i style="width:' + Math.round(110 * r.changes / most) + 'px"></i>' + r.changes + "</span></td></tr>").join("") +
    "</tbody></table></div>" : '<div class="empty">種類の表がありません</div>') + "</section>";
  h += "<section><h2>セッションが止まって確認するファイル <small>触れるファイルで、任せ方が変わります</small></h2>" + (PROCESS.surfaces.length ? '<div class="autos">' + AUTONOMY.map(a => {
    const mine = PROCESS.surfaces.filter(s => s.autonomy === a[0]);
    return '<div class="auto ' + a[0] + '"><h3>' + a[1] + " <small>" + mine.length + '</small></h3><p class="what">' + a[2] + "</p>" + (mine.length ? mine.map(s =>
      '<div class="surface"><b>' + esc(s.name) + '</b> <span class="pill">リスク ' + esc(s.tier) + "</span>" + (s.screen ? ' <span class="pill needs">画面の合意が必要</span>' : "") +
      '<div class="paths">' + s.paths.map(p => "<code>" + esc(p) + "</code>").join(" ") + '</div><div class="what">求める根拠: ' + methodChips(s.methods) + "</div></div>").join("")
      : '<p class="empty">なし</p>') + "</div>";
  }).join("") + "</div>" : '<div class="empty">ファイルごとの表がありません</div>') + "</section>";
  h += "<section><h2>根拠の取り方</h2>" + (PROCESS.methods.length ? '<table class="all"><thead><tr><th>名前</th><th>種類</th><th>実行するもの</th></tr></thead><tbody>' +
    PROCESS.methods.map(m => "<tr><td><b>" + esc(m.name) + "</b></td><td>" + esc(m.kind === "formal" ? "すべての場合を確かめる" : m.kind === "empirical" ? "動かして確かめる" : m.kind) +
      "</td><td><code>" + esc(m.command) + "</code></td></tr>").join("") + "</tbody></table>" : '<div class="empty">根拠の表がありません</div>') + "</section>";
  h += '<section><h2>1 つの変更が進む順 <small>この順は飛ばせません</small></h2><ol class="order">' + ORDER.map(o =>
    "<li><b>" + o[1] + "</b><code>" + o[0] + "</code><span>" + o[2] + "</span></li>").join("") + "</ol></section>";
  $("pr-body").innerHTML = h;
}

// ---- カタログ ---------------------------------------------------------------
const CATALOG = M.catalog || { palettes: [], icons: [], keys: [], errors: [] };
const PALETTE = { dark: "ダーク", light: "ライト", "high-contrast": "ハイコントラスト" };
// A colour reaches a style only in the two shapes the model writes.
function swatch(colour) {
  return /^(#[0-9a-f]{6}|rgba\(0,0,0,[0-9.]+\))$/.test(String(colour)) ? '<i class="sw" style="background:' + colour + '"></i>' : '<i class="sw none"></i>';
}
function renderCatalog() {
  const unread = path => (CATALOG.errors || []).find(e => e.endsWith(path));
  const failed = path => '<p class="warn">' + esc(unread(path)) + "</p>";
  let h = "";
  const roles = CATALOG.palettes.length ? CATALOG.palettes[0].roles.map(r => r.role) : [];
  h += "<section><h2>色の役割 <small>" + roles.length + " 役割 × " + CATALOG.palettes.length + " テーマ</small></h2>" + (unread("src/theme.rs") ? failed("src/theme.rs") :
    '<div class="scroll"><table class="all pal"><thead><tr><th>役割</th>' + CATALOG.palettes.map(p => "<th>" + esc(PALETTE[p.name] || p.name) + "</th>").join("") + "</tr></thead><tbody>" +
    roles.map(role => "<tr><td><code>" + esc(role) + "</code></td>" + CATALOG.palettes.map(p => {
      const found = p.roles.find(r => r.role === role);
      return "<td>" + (found ? swatch(found.colour) + "<code>" + esc(found.colour) + "</code>" : '<span class="empty">なし</span>') + "</td>";
    }).join("") + "</tr>").join("") + "</tbody></table></div>") + "</section>";
  h += "<section><h2>アイコン <small>" + CATALOG.icons.length + "</small></h2>" + (unread("src/glyphs.rs") ? failed("src/glyphs.rs") :
    '<p class="what">描画にはアプリ内蔵のアイコンフォントが必要です。ここでは名前と、ソースで使われている回数を示します。</p><div class="icons">' +
    CATALOG.icons.map(i => '<div class="icon' + (i.uses ? "" : " unused") + '"><code>' + esc(i.name) + '</code><span class="what">' + esc(i.glyph) + "</span><b>" + i.uses + " 回</b></div>").join("") + "</div>") + "</section>";
  h += "<section><h2>キー操作 <small>" + CATALOG.keys.length + "</small></h2>" + (unread("src/app/keymap.rs") ? failed("src/app/keymap.rs") :
    '<table class="all"><thead><tr><th>操作</th><th>キー</th><th>名前</th></tr></thead><tbody>' + CATALOG.keys.map(k =>
      "<tr><td>" + esc(k.label) + '</td><td><span class="kbd">' + esc(k.chord) + "</span></td><td><code>" + esc(k.id) + "</code></td></tr>").join("") + "</tbody></table>") + "</section>";
  $("ct-body").innerHTML = h;
}

// ---- 左の機能一覧 -------------------------------------------------------------
let sideWords = "", sideVerdict = "";
// The features of the left column as narrowed, in the order drawn: `j` and
// `k` on a feature's page step through exactly these.
function queue() {
  const order = DOMAINS.map(d => d.id).concat([""]);
  const kept = FEATURES.filter(f => L.queueMatches(f, { words: sideWords, verdict: sideVerdict }, verdicts));
  return order.map(id => ({ id, features: kept.filter(f => (domainById.has(f.domain) ? f.domain : "") === id) })).filter(g => g.features.length);
}
function renderSide(route) {
  const counts = L.verdictCounts(FEATURES, verdicts);
  $("sd-counts").innerHTML = [["", "すべて", counts.all], ["unjudged", "未判定", counts.unjudged], ["judged", "判定済み", counts.judged]].map(c =>
    '<button data-queue="' + c[0] + '" aria-pressed="' + (sideVerdict === c[0]) + '">' + c[1] + " " + c[2] + "</button>").join("");
  const groups = queue();
  const here = route.view === "feature" ? route.id : "";
  $("sd-list").innerHTML = groups.length ? groups.map(g => {
    const per = ST.domains.find(d => d.id === g.id);
    return "<h4>" + esc(g.id ? domainTitle(g.id) : "ドメイン未設定") + (per ? " <small>" + per.pinned + "/" + per.requirements + "</small>" : "") + "</h4>" + g.features.map(f => {
      const v = verdicts[f.id];
      return '<button data-feature="' + esc(f.id) + '"' + (f.id === here ? ' class="on" aria-current="true"' : "") + '><i class="dot ' + f.badge + '" title="' + BADGE[f.badge] + '"></i><span class="t">' +
        esc(f.title) + "</span>" + (v ? '<span class="vm ' + v.verdict + '">' + esc(verdictWord(v.verdict)) + "</span>" : "") +
        '<i class="sh" title="テストあり ' + (f.pinned || 0) + " / " + (reqCount.get(f.id) || 0) + '"><em style="width:' + share(f) + '%"></em></i></button>';
    }).join("");
  }).join("") : '<p class="empty">' + (FEATURES.length ? "条件に合う機能はありません" : "機能の一覧がありません") + "</p>";
  const on = $("sd-list").querySelector(".on");
  if (on) on.scrollIntoView({ block: "nearest" });
}
function stepFeature(step) {
  const list = queue().reduce((all, g) => all.concat(g.features), []);
  if (!list.length) return;
  const r = L.parseRoute(location.hash);
  const at = list.findIndex(f => f.id === r.id);
  const next = at < 0 ? (step > 0 ? 0 : list.length - 1) : Math.max(0, Math.min(list.length - 1, at + step));
  if (list[next].id !== r.id) go({ view: "feature", id: list[next].id });
}

// ---- 変更 -------------------------------------------------------------------
function renderChanges() {
  const needs = M.changes.filter(c => c.group === "needs");
  $("needs-n").textContent = "(" + needs.length + ")";
  $("needs-list").innerHTML = needs.length ? '<div class="cards">' + needs.map(c => card(c)).join("") + "</div>" : '<div class="empty">判断を待っている変更はありません</div>';
  const flight = M.changes.filter(c => c.group === "flight");
  $("flight-n").textContent = "(" + flight.length + ")";
  if (!flight.length) $("flight-list").innerHTML = '<div class="empty">進行中の変更はありません</div>';
  else $("flight-list").innerHTML = '<div class="track">' + M.stages.map(s =>
    '<div class="col"><h3>' + STAGE[s] + "</h3>" + flight.filter(c => c.stage === s).map(c => card(c, true)).join("") + "</div>").join("") + "</div>" +
    flight.filter(c => !M.stages.includes(c.stage)).map(c => card(c)).join("");
  const most = Math.max(1, ...M.areas.map(a => a.changes.length));
  $("area-list").innerHTML = M.areas.map(a => {
    const all = a.changes.map(n => byName.get(n)).filter(matches);
    if (query && !all.length) return "";
    const open = expanded.has(a.name) || query;
    const shown = open ? all : all.slice(0, 16);
    let chips = shown.map(c => '<button class="chip ' + c.group + '" data-open="' + esc(c.name) + '" title="' + esc(c.title + " — " + statusWord(c)) + '">' + esc(c.number) + "</button>").join("");
    if (all.length > shown.length) chips += '<button class="chip more" data-more="' + esc(a.name) + '">さらに ' + (all.length - shown.length) + "</button>";
    return '<div class="area"><div><div class="name">' + esc(a.label) + '</div><div class="desc">' + esc(a.description) + '</div></div><div class="bar"><i style="width:' +
      Math.round(150 * a.changes.length / most) + 'px"></i>' + a.changes.length + '</div><div class="chips">' + chips + "</div></div>";
  }).join("") || '<div class="empty">一致するものはありません</div>';
  $("filters").innerHTML = [["", "すべて"]].concat(Object.entries(GROUP)).map(([key, label]) =>
    '<button data-group="' + key + '" aria-pressed="' + (groupFilter === key) + '">' + label + (key ? " " + M.counts[key] : "") + "</button>").join("");
  listed = M.changes.filter(c => matches(c) && (!groupFilter || c.group === groupFilter));
  $("all-n").textContent = "(" + listed.length + " / " + M.counts.total + ")";
  $("all-list").innerHTML = listed.length ? '<table class="all"><thead><tr><th>番号</th><th>題</th><th>状態</th><th>段階</th><th>種類</th><th>機能</th></tr></thead><tbody>' +
    listed.map(c => '<tr class="pick" data-open="' + esc(c.name) + '"><td class="num">' + esc(c.number) + "</td><td>" + esc(c.title) + "</td><td>" + pill(c) +
      "</td><td>" + esc(STAGE[c.stage] || c.stage) + "</td><td>" + esc(routeWord(c)) + "</td><td>" + esc(featuresOf(c)) + "</td></tr>").join("") +
    "</tbody></table>" : '<div class="empty">一致するものはありません</div>';
}
let current = null, currentTab = "overview", rawView = false;
function tabsOf(c) {
  const tabs = [["overview", "概要"]];
  const named = ["intent.md", "bug.md", "spec.md", "plan.md", "screen.md", "handoff.md"];
  for (const n of named) if (c.docs.some(d => d.name === n)) tabs.push([n.replace(/\.md$/, ""), TAB[n]]);
  if (c.findings.length) tabs.push(["findings", "未解決の指摘 " + c.findings.length]);
  if (c.approvals.length) tabs.push(["approvals", "承認の記録"]);
  for (const d of c.docs) if (!named.includes(d.name) && d.name !== "approvals.log") tabs.push([d.name.replace(/\.md$/, ""), TAB[d.name] || d.name]);
  return tabs;
}
function docFor(c, tab) { return c.docs.find(d => d.name === tab || d.name === tab + ".md"); }
function featureChips(c) {
  const links = c.features || [];
  if (!links.length) return esc(featuresOf(c));
  return '<span class="fchips">' + links.map(l => '<button class="fchip" data-feature="' + esc(l.id) + '" title="' + esc(LINK[l.link]) + '">' +
    esc(featureTitle(l.id)) + (l.link === "recorded" ? "" : "（推定）") + "</button>").join("") + "</span>";
}
function changeOverview(c) {
  let h = "";
  if (c.problem) h += '<div class="box needs"><h3>記録を読めません</h3>' + esc(c.problem) + "</div>";
  else if (c.group === "needs") h += '<div class="box needs"><h3>あなたの判断を待っています</h3>' + esc(c.resume) + "</div>";
  else if (c.resume) h += '<div class="box"><h3>' + (c.group === "flight" ? "次にすること" : "結果") + "</h3>" + esc(c.resume) + "</div>";
  const used = v => { const p = String(v).split(" "); return p.length === 2 ? p[0] + " / " + p[1] : v; };
  h += '<dl class="kv"><dt>状態</dt><dd>' + pill(c) + " <code>" + esc(c.status) + "</code></dd>" +
    "<dt>段階</dt><dd>" + esc(STAGE[c.stage] || c.stage) + " <code>" + esc(c.stage) + "</code></dd>" +
    "<dt>種類</dt><dd>" + esc(routeWord(c)) + " <code>" + esc(c.route) + "</code></dd>" +
    "<dt>機能</dt><dd>" + featureChips(c) + "</dd>" +
    "<dt>リスク / 自律</dt><dd>" + esc(c.risk) + " / " + esc(c.autonomy) + "</dd>" +
    "<dt>やり直し</dt><dd>" + esc(used(c.attempts)) + "</dd><dt>質問</dt><dd>" + esc(used(c.questions)) + "</dd>" +
    "<dt>仕様の準備</dt><dd>" + esc(c.readiness) + "</dd><dt>画面の合意</dt><dd>" + esc(c.screen) + "</dd>" +
    "<dt>領域</dt><dd>" + esc(c.areas.map(areaLabel).join("、")) + "</dd>" +
    "<dt>場所</dt><dd>" + (c.where.here ? "このチェックアウト" : "別の作業コピー: " + esc(c.where.path)) + (c.where.branch ? " (" + esc(c.where.branch) + ")" : "") + "</dd>" +
    "<dt>記録</dt><dd><code>docs/sdlc/changes/" + esc(c.name) + "/</code></dd></dl>";
  if (c.contract.length) {
    h += '<div class="box"><h3>完了の条件</h3><ul class="check">' + c.contract.map(k =>
      "<li><span>" + '<span class="pill ' + (k.verdict === "pending" ? "needs" : k.verdict === "passed" ? "done" : "") + '">' +
      esc(VERDICT[k.verdict] || k.verdict) + '</span></span><span><span class="key">' + esc(k.key) +
      "</span> <small>(" + (k.kind === "human" ? "人が確認" : "自動") + ')</small><div class="what">' + esc(k.text) + "</div></span></li>").join("") + "</ul></div>";
  }
  if (c.omitted.length) h += '<p class="warn">載せていない記録: ' + esc(c.omitted.join(", ")) + "</p>";
  return h;
}
function renderDetail() {
  const c = current;
  $("d-title").textContent = c.number + "  " + c.title;
  $("d-facts").innerHTML = pill(c) + "<span>" + esc([routeWord(c), place(c)].filter(Boolean).join(" · ")) + "</span>";
  const route = (M.routes[c.route] || []).slice();
  if (c.stage && !route.includes(c.stage)) route.push(c.stage);
  const at = route.indexOf(c.stage);
  const closed = c.group === "done";
  $("d-steps").innerHTML = route.map((s, n) => '<span class="' + (closed || n < at ? "past" : n === at ? "now" : "") + '">' + STAGE[s] + "</span>").join("<em></em>");
  const tabs = tabsOf(c);
  if (!tabs.some(t => t[0] === currentTab)) currentTab = "overview";
  $("d-tabs").innerHTML = tabs.map(t => '<button role="tab" data-tab="' + esc(t[0]) + '" aria-selected="' + (t[0] === currentTab) + '">' + esc(t[1]) + "</button>").join("");
  const body = $("body");
  if (currentTab === "overview") body.innerHTML = changeOverview(c);
  else if (currentTab === "findings") {
    body.innerHTML = "<p>レビューで見つかり、直さずに残してある点です。</p>" + c.findings.map(f =>
      '<div class="finding"><div class="person">' + (f.person ? esc(f.person) : "（人向けの説明がありません）") + '</div><div class="raw">' + esc(f.finding) + "</div></div>").join("");
  } else if (currentTab === "approvals") {
    body.innerHTML = '<div class="md"><table><thead><tr><th>日時 (UTC)</th><th>誰が</th><th>対象</th><th>判定</th><th>理由</th></tr></thead><tbody>' +
      c.approvals.map(a => "<tr><td>" + esc(a.at) + "</td><td>" + esc(a.by === "person" ? "人" : a.by === "auto" ? "評価役" : a.by) +
      "</td><td>" + esc(a.category + " " + a.artifact) + "</td><td>" + esc(a.verdict) + "</td><td>" + esc(a.reason) + "</td></tr>").join("") + "</tbody></table></div>";
  } else {
    const d = docFor(c, currentTab);
    const isMd = d.name.endsWith(".md");
    let h = '<div class="tools"><code>' + esc(d.name) + "</code>" + (isMd ? '<button id="raw-toggle">' + (rawView ? "整形して表示" : "原文") + "</button>" : "") + "</div>";
    if (d.truncated) h += '<p class="warn">長いため途中まで表示しています</p>';
    h += isMd && !rawView ? mdBlock(d.text) : '<pre class="raw">' + esc(d.text) + "</pre>";
    body.innerHTML = h;
  }
  body.scrollTop = 0;
  const at2 = listed.indexOf(c);
  $("d-prev").disabled = at2 <= 0;
  $("d-next").disabled = at2 < 0 || at2 >= listed.length - 1;
}
function show(name, tab) {
  const c = byName.get(name);
  if (!c) return;
  current = c; currentTab = tab || "overview";
  $("detail").hidden = false; $("shade").hidden = false;
  renderDetail();
  const hash = L.formatRoute({ view: "change", id: c.number || c.name, tab: currentTab === "overview" ? "" : currentTab });
  if (location.hash !== hash) history.pushState(null, "", hash);
  $("d-close").focus();
}
function hideDetail() {
  if (!current) return;
  current = null; $("detail").hidden = true; $("shade").hidden = true;
  history.replaceState(null, "", L.formatRoute(base));
}

// ---- 文書 -------------------------------------------------------------------
function openDoc(kind, id) {
  const f = featureById.get(id);
  if (!f) return;
  const title = kind === "brief" ? "BRIEF — " : kind === "spec" ? "SPEC — " : "マニュアル — ";
  const doc = kind === "manual" ? { text: f.manual_text } : f[kind];
  $("doc-title").textContent = title + f.title;
  $("doc-body").innerHTML = doc ? (doc.truncated ? '<p class="warn">長いため途中まで表示しています</p>' : "") + mdBlock(doc.text) : '<p class="empty">文書がありません</p>';
  $("doc").hidden = false; $("shade").hidden = false;
  $("doc-close").focus();
}
function closeDoc() { $("doc").hidden = true; if (!current) $("shade").hidden = true; }

// ---- 検索 -------------------------------------------------------------------
const KIND_WORD = { feature: "機能", screen: "画面", domain: "ドメイン", term: "用語", requirement: "要件", change: "変更" };
const INDEX = [].concat(
  FEATURES.map(f => ({ kind: "feature", id: f.id, title: f.title, text: f.id + " " + f.summary, route: { view: "feature", id: f.id } })),
  FLOW.screens.map(s => ({ kind: "screen", id: s.id, title: s.title, text: s.id + " " + (s.note || "") + " " + (s.words || []).join(" "), route: { view: "screen", id: s.id } })),
  DOMAINS.map(d => ({ kind: "domain", id: d.id, title: d.title, text: d.id + " " + d.summary + " " + d.owns, route: { view: "domain", id: d.id } })),
  TERMS.map(t => ({ kind: "term", id: t.id, title: t.term, text: [t.id, t.en, t.meaning].concat(t.ui).join(" "), route: { view: "term", id: t.id } })),
  REQS.map(r => ({ kind: "requirement", id: r.feature + "#" + r.number, title: featureTitle(r.feature) + " " + r.number, text: r.text + " " + r.tests.join(" "),
    route: { view: "feature", id: r.feature, tab: "reqs" } })),
  M.changes.map(c => ({ kind: "change", id: c.name, title: c.number + " " + c.title, text: c.name, change: c.name })));
let found = [], foundAt = 0;
function renderResults() {
  const box = $("results");
  if (!query) { box.hidden = true; return; }
  found = L.search(INDEX, query, 80);
  foundAt = Math.min(foundAt, Math.max(0, found.length - 1));
  if (!found.length) { box.innerHTML = '<div class="empty">一致するものはありません</div>'; box.hidden = false; return; }
  let h = "", n = 0;
  for (const kind of Object.keys(KIND_WORD)) {
    const here = found.map((r, i) => [r, i]).filter(([r]) => r.kind === kind).slice(0, 6);
    if (!here.length) continue;
    h += "<h4>" + KIND_WORD[kind] + "</h4>" + here.map(([r, i]) => '<button data-hit="' + i + '"' + (i === foundAt ? ' class="on"' : "") + ">" + esc(r.title) +
      '<span class="s">' + esc(String(r.text).slice(0, 120)) + "</span></button>").join("");
    n += here.length;
  }
  box.innerHTML = h;
  box.hidden = !n;
}
function openHit(i) {
  const r = found[i];
  if (!r) return;
  $("results").hidden = true; $("q").blur();
  if (r.change) show(r.change); else go(r.route);
}

// ---- どこを開いているか ---------------------------------------------------------
let base = { view: "overview" };
const VIEW_TAB = { overview: "overview", features: "features", feature: "features", screen: "screen", canvas: "screen", domains: "domains",
  domain: "domains", glossary: "glossary", "glossary-screens": "glossary", term: "glossary", requirements: "requirements", changes: "changes",
  work: "work", process: "process", catalog: "catalog" };
const PANES = ["overview", "features", "feature", "screen", "canvas", "domains", "domain", "glossary", "requirements", "work", "changes", "process", "catalog"];
function render() {
  let r = L.parseRoute(location.hash);
  if (r.view === "change") {
    const c = byNumber.get(r.id) || byName.get(r.id);
    if (c) { if (!current || current.name !== c.name) { current = c; currentTab = r.tab || "overview"; $("detail").hidden = false; $("shade").hidden = false; renderDetail(); } }
    else r = { view: "overview" };
    if (c) { if (!$("v-" + paneOf(base)).hidden) return; r = base; }
  } else {
    if (current) { current = null; $("detail").hidden = true; $("shade").hidden = true; }
    base = r;
  }
  if (r.view === "feature" && !featureById.has(r.id)) r = { view: "features" };
  if (r.view === "domain" && !domainById.has(r.id)) r = { view: "domains" };
  if (r.view === "term") termSel = termById.get(r.id) || null; else if (r.view === "glossary") termSel = null;
  const pane = paneOf(r);
  for (const name of PANES) $("v-" + name).hidden = name !== pane;
  for (const b of document.querySelectorAll("nav.views button")) b.setAttribute("aria-pressed", String(b.dataset.view === VIEW_TAB[r.view]));
  navAt = -1;
  const choice = r.query || {};
  if (r.view === "overview") renderOverview();
  else if (r.view === "features") renderFeatures(choice);
  else if (r.view === "feature") renderFeature(featureById.get(r.id), r.tab);
  else if (r.view === "screen") {
    const s = flowById.get(r.id) || flowById.get(walk.current()) || flowById.get("home") || FLOW.screens[0];
    if (s) { walk.sync(s.id); renderScreen(s); } else $("sc-window").innerHTML = errorBox(M.flow_error || "画面の流れがありません", "");
  } else if (r.view === "canvas") renderCanvas();
  else if (r.view === "domains") renderDomains();
  else if (r.view === "domain") renderDomain(domainById.get(r.id));
  else if (r.view === "glossary" || r.view === "term") renderGlossary(false);
  else if (r.view === "glossary-screens") renderGlossary(true);
  else if (r.view === "requirements") {
    if (r.id && featureById.has(r.id)) { reqFeature = r.id; reqDomain = ""; }
    renderRequirements();
  } else if (r.view === "work") renderWork(choice);
  else if (r.view === "process") renderProcess();
  else if (r.view === "catalog") renderCatalog();
  else renderChanges();
  renderSide(r);
  renderBasket();
  if (!["feature", "term"].includes(r.view) || !r.tab) window.scrollTo(0, 0);
}
function paneOf(r) {
  return { term: "glossary", "glossary-screens": "glossary", change: "changes" }[r.view] || r.view;
}

// ---- j / k --------------------------------------------------------------------
let navAt = -1;
function navItems() {
  const pane = document.querySelector("main > div:not([hidden])");
  if (!pane) return [];
  return Array.from(pane.querySelectorAll(".kpi, .ftile, .fcard, .dcard, .thumb, tr.pick, .rows button, .plist button, ul.ways2 button, .card, .wcard, .nexts button"))
    .filter(el => el.offsetParent !== null);
}
function moveNav(step) {
  const items = navItems();
  if (!items.length) return;
  for (const el of document.querySelectorAll(".sel-nav")) el.classList.remove("sel-nav");
  navAt = navAt < 0 ? (step > 0 ? 0 : items.length - 1) : Math.max(0, Math.min(items.length - 1, navAt + step));
  items[navAt].classList.add("sel-nav");
  items[navAt].scrollIntoView({ block: "nearest" });
}
function onFeaturePage() { const r = L.parseRoute(location.hash); return r.view === "feature" && featureById.has(r.id) ? r : null; }
function giveVerdict(verdict) {
  const r = onFeaturePage();
  if (!r) return;
  setVerdict(r.id, verdict);
  const at = window.scrollY;
  renderFeature(featureById.get(r.id));
  renderSide(r);
  window.scrollTo(0, at);
}

// ---- events -------------------------------------------------------------------
document.addEventListener("click", event => {
  if (event.target.closest("label.bk") || event.target.closest("label.toggle")) return;
  const t = event.target.closest("[data-way],[data-trail],[data-hit],[data-go],[data-view],[data-lens],[data-verdict],[data-queue],[data-rq-domain],[data-rq-tests],[data-rq-mode],[data-issue-copy],[data-screen],[data-feature],[data-domain],[data-term],[data-doc],[data-open],[data-more],[data-group],[data-tab],[data-unbasket],[data-work],[data-empty-basket]");
  if (!t) return;
  const d = t.dataset;
  if (d.way !== undefined) takeWay(Number(d.way));
  else if (d.trail !== undefined) { const n = Number(d.trail); walk.at = n; go({ view: "screen", id: walk.current() }); }
  else if (d.hit !== undefined) openHit(Number(d.hit));
  else if (d.go) { if (location.hash === d.go) render(); else location.hash = d.go; }
  else if (d.view) go({ view: d.view, id: d.view === "screen" ? walk.current() : undefined });
  else if (d.lens) {
    const r = L.parseRoute(location.hash);
    go({ view: "features", query: L.lensToggle(r.view === "features" ? r.query || {} : {}, d.lens, d.value, event.metaKey || event.shiftKey) });
  }
  else if (d.verdict) giveVerdict(d.verdict);
  else if (d.queue !== undefined) { sideVerdict = d.queue; renderSide(L.parseRoute(location.hash)); }
  else if (d.rqDomain !== undefined) { reqDomain = d.rqDomain; reqFeature = ""; renderRequirements(); }
  else if (d.rqTests !== undefined) { reqTests = d.rqTests; renderRequirements(); }
  else if (d.rqMode) { reqTable = d.rqMode === "table"; renderRequirements(); }
  else if (d.issueCopy) {
    const order = L.workOrder(["issue|" + d.issueCopy], M, "fix", "バックログ #" + d.issueCopy + " を backlog スキルで受け取り、進めてください。");
    copyText(order.text, t.parentNode.querySelector(".said"), null);
  }
  else if (d.screen) { walk.visit(d.screen); go({ view: "screen", id: d.screen }); }
  else if (d.feature) go({ view: "feature", id: d.feature });
  else if (d.domain) go({ view: "domain", id: d.domain });
  else if (d.term) go({ view: "term", id: d.term });
  else if (d.doc) { const [kind, id] = d.doc.split("|"); openDoc(kind, id); }
  else if (d.open) show(d.open, d.tab);
  else if (d.more) { expanded.add(d.more); renderChanges(); }
  else if (d.group !== undefined) { groupFilter = d.group; renderChanges(); }
  else if (d.tab) show(current.name, d.tab);
  else if (d.unbasket) { basket.delete(d.unbasket); saveBasket(); render(); }
  else if (d.work) { workType = d.work; saveBasket(); renderBasket(); }
  else if (d.emptyBasket) { basket.clear(); saveBasket(); render(); }
});
$("body").addEventListener("click", event => { if (event.target.id === "raw-toggle") { rawView = !rawView; renderDetail(); } });
document.addEventListener("change", event => {
  const box = event.target.closest("input[data-basket]");
  if (box) { if (box.checked) basket.set(box.dataset.basket, true); else basket.delete(box.dataset.basket); saveBasket(); renderBasket(); return; }
  const id = event.target.id;
  if (id === "rq-feature") { reqFeature = event.target.value; renderRequirements(); }
  else if (id === "f-bare") { reqBareOnly = event.target.checked; const r = L.parseRoute(location.hash); renderFeature(featureById.get(r.id), "reqs"); }
  else if (id === "sc-areas") { showAreas = event.target.checked; const w = $("sc-win"); if (w) w.classList.toggle("show", showAreas); }
});
document.addEventListener("input", event => {
  const id = event.target.id;
  if (id === "bk-note") { workNote = event.target.value; saveBasket(); showInstruction(); }
  else if (id === "vd-note") { const r = onFeaturePage(); if (r && verdicts[r.id]) { verdicts[r.id].note = event.target.value; saveVerdicts(); } }
  else if (id === "sd-q") { sideWords = event.target.value; renderSide(L.parseRoute(location.hash)); }
});
$("q").addEventListener("input", event => {
  query = event.target.value.trim().toLowerCase(); foundAt = 0;
  renderResults();
  const r = L.parseRoute(location.hash);
  if (["changes", "requirements", "glossary"].includes(r.view)) render();
});
$("q").addEventListener("keydown", event => {
  if (event.key === "ArrowDown" || event.key === "ArrowUp") {
    event.preventDefault();
    foundAt = Math.max(0, Math.min(found.length - 1, foundAt + (event.key === "ArrowDown" ? 1 : -1)));
    renderResults();
  } else if (event.key === "Enter") { event.preventDefault(); openHit(foundAt); }
});
$("q").addEventListener("focus", () => { if (query) renderResults(); });
document.addEventListener("mousedown", event => { if (!event.target.closest(".searchwrap")) $("results").hidden = true; });
$("bk-open").addEventListener("click", () => openBasket(true));
$("bk-close").addEventListener("click", () => openBasket(false));
$("basket").addEventListener("click", event => { if (event.target.id === "bk-copy") copyInstruction(); });
$("basket").addEventListener("keydown", event => { if (event.key === "Enter" && event.metaKey) { event.preventDefault(); copyInstruction(); } });
$("v-feature").addEventListener("click", event => { if (event.target.id === "vd-copy") copyVerdicts(); });
$("sd-open").addEventListener("click", () => {
  const open = !document.body.classList.contains("side-open");
  document.body.classList.toggle("side-open", open);
  $("sd-open").setAttribute("aria-expanded", String(open));
});
$("sd-list").addEventListener("click", () => { document.body.classList.remove("side-open"); $("sd-open").setAttribute("aria-expanded", "false"); });
$("shade").addEventListener("click", () => { closeDoc(); hideDetail(); });
$("d-close").addEventListener("click", hideDetail);
$("doc-close").addEventListener("click", closeDoc);
$("d-prev").addEventListener("click", () => { const n = listed.indexOf(current); if (n > 0) show(listed[n - 1].name, currentTab); });
$("d-next").addEventListener("click", () => { const n = listed.indexOf(current); if (n >= 0 && n < listed.length - 1) show(listed[n + 1].name, currentTab); });
$("sc-back").addEventListener("click", () => { if (walk.canBack()) history.back(); });
$("sc-fwd").addEventListener("click", () => { if (walk.canForward()) history.forward(); });
$("help-open").addEventListener("click", () => { $("help").hidden = !$("help").hidden; });
$("help-close").addEventListener("click", () => { $("help").hidden = true; });
$("help-keys").innerHTML = L.KEY_HELP.map(k => '<dt><span class="kbd">' + esc(k[0]) + "</span></dt><dd>" + esc(k[1]) + "</dd>").join("");
function themeLabel() {
  const now = document.documentElement.dataset.theme;
  $("theme").textContent = "明るさ: " + (L.THEMES.find(t => t[0] === now) || L.THEMES[0])[1];
}
$("theme").addEventListener("click", () => {
  const next = L.themeNext(document.documentElement.dataset.theme);
  document.documentElement.dataset.theme = next;
  keepOrSay(L.STORE.theme, next);
  themeLabel();
});
themeLabel();
document.addEventListener("keydown", event => {
  const action = L.KEYS[event.key];
  if (!action || event.metaKey || event.ctrlKey || event.altKey) return;
  const typing = event.target.closest && event.target.closest("input, textarea, select");
  if (action === "close") {
    if (!$("results").hidden) { $("results").hidden = true; $("q").blur(); }
    else if (!$("help").hidden) $("help").hidden = true;
    else if (!$("doc").hidden) closeDoc();
    else if (current) hideDetail();
    else if (basketOpen) openBasket(false);
    return;
  }
  if (typing) return;
  const r = L.parseRoute(location.hash);
  if (action === "search") { event.preventDefault(); $("q").focus(); $("q").select(); }
  else if (action === "help") $("help").hidden = !$("help").hidden;
  else if (action.startsWith("view:")) { const view = action.slice(5); go({ view, id: view === "screen" ? walk.current() : undefined }); }
  else if (action === "basket") openBasket(!basketOpen);
  else if (action.startsWith("verdict:")) giveVerdict(action.slice(8));
  else if (action === "next" || action === "prev") { if (r.view === "feature") stepFeature(action === "next" ? 1 : -1); else moveNav(action === "next" ? 1 : -1); }
  else if (action === "open") {
    const el = document.querySelector(".sel-nav");
    const node = event.target.closest && event.target.closest("[data-node]");
    if (node) { walk.visit(node.dataset.node); go({ view: "screen", id: node.dataset.node }); }
    else if (el) { event.preventDefault(); el.click(); }
  }
  else if (action === "back" && r.view === "screen" && walk.canBack()) history.back();
  else if (action === "forward" && r.view === "screen" && walk.canForward()) history.forward();
});
// 全体図: drag to move, ⌘ or Ctrl with the wheel to zoom, the wheel alone to scroll.
(function canvasControls() {
  const cv = $("cv");
  let drag = null;
  cv.addEventListener("pointerdown", event => {
    drag = { x: event.clientX, y: event.clientY, tx: CV.tx, ty: CV.ty, moved: false };
    cv.setPointerCapture(event.pointerId);
  });
  cv.addEventListener("pointermove", event => {
    if (!drag) {
      const node = event.target.closest && event.target.closest("[data-node]");
      focusNode(node ? node.dataset.node : "");
      return;
    }
    const dx = event.clientX - drag.x, dy = event.clientY - drag.y;
    if (Math.abs(dx) + Math.abs(dy) > 4) { drag.moved = true; cv.classList.add("drag"); }
    CV.tx = drag.tx + dx; CV.ty = drag.ty + dy; applyCanvas();
  });
  cv.addEventListener("pointerup", event => {
    const was = drag; drag = null; cv.classList.remove("drag");
    if (was && !was.moved) {
      const hit = document.elementFromPoint(event.clientX, event.clientY);
      const node = hit && hit.closest && hit.closest("[data-node]");
      if (node) { walk.visit(node.dataset.node); go({ view: "screen", id: node.dataset.node }); }
    }
  });
  cv.addEventListener("pointerleave", () => { if (!drag) focusNode(""); });
  cv.addEventListener("wheel", event => {
    event.preventDefault();
    const box = cv.getBoundingClientRect();
    if (event.metaKey || event.ctrlKey) zoomAt(Math.exp(-event.deltaY * 0.0022), event.clientX - box.left, event.clientY - box.top);
    else { CV.tx -= event.deltaX; CV.ty -= event.deltaY; applyCanvas(); }
  }, { passive: false });
  $("cv-in").addEventListener("click", () => { const b = cv.getBoundingClientRect(); zoomAt(1.25, b.width / 2, b.height / 2); });
  $("cv-out").addEventListener("click", () => { const b = cv.getBoundingClientRect(); zoomAt(0.8, b.width / 2, b.height / 2); });
  $("cv-fit").addEventListener("click", () => { fitCanvas(); applyCanvas(); });
})();
window.addEventListener("hashchange", render);
// The column and the contents stick below the bar, whose height follows the width.
function measureTop() { document.documentElement.style.setProperty("--top", document.querySelector("header.top").offsetHeight + "px"); }
window.addEventListener("resize", measureTop);
measureTop();
$("meta").innerHTML = "生成 " + esc(M.generated) + (M.worktrees_read ? "" : ' · <span class="warn">他の作業コピーを読めませんでした</span>') +
  (M.shots_error ? ' · <span class="warn">' + esc(M.shots_error) + "</span>" : "");
render();
}

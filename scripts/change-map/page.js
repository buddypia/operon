"use strict";
// Operon's product map (changes 312, 313, 315, 316).
//
// Two halves. `MapLogic` is plain functions with no page in sight — links,
// the walk through screens, search, the scale of a pressable area, the keys —
// so `map_page_logic_routes_walks_searches_and_scales_areas` in src/tests.rs
// runs them under node. `main` draws, and runs only where there is a document.
// Every value from the model passes through esc() before it becomes markup.

const MapLogic = (function () {
  const LISTS = { "": "overview", overview: "overview", product: "features", features: "features",
    screen: "screen", canvas: "canvas", flow: "canvas", domains: "domains", glossary: "glossary",
    "glossary/screens": "glossary-screens", requirements: "requirements", changes: "changes" };

  // A link, read back into the view it names. An unknown link opens 概要.
  function parseRoute(hash) {
    let raw = String(hash || "").replace(/^#/, "");
    try { raw = decodeURIComponent(raw); } catch (error) { /* kept as written */ }
    if (Object.prototype.hasOwnProperty.call(LISTS, raw)) return { view: LISTS[raw] };
    let m = raw.match(/^feature\/([^/]+)(?:\/([^/]+))?$/);
    if (m) return m[2] ? { view: "feature", id: m[1], tab: m[2] } : { view: "feature", id: m[1] };
    m = raw.match(/^(screen|domain|term|requirements)\/([^/]+)$/);
    if (m) return { view: m[1], id: m[2] };
    m = raw.match(/^change\/([\w-]+)(?:\/([^/]+))?$/) || raw.match(/^(\d+)(?:\/([^/]+))?$/);
    if (m) return m[2] ? { view: "change", id: m[1], tab: m[2] } : { view: "change", id: m[1] };
    return { view: "overview" };
  }

  function formatRoute(r) {
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
    ArrowLeft: "back", ArrowRight: "forward" };
  const KEY_HELP = [["/", "検索に移る"], ["j / k", "一覧の次・前"], ["Enter", "選んだものを開く"],
    ["Esc", "閉じる"], ["← / →", "画面で戻る・進む"], ["?", "この一覧"]];

  return { parseRoute, formatRoute, scaleArea, areaPercent, Walk, search, KEYS, KEY_HELP };
})();

if (typeof module === "object" && module.exports) module.exports = MapLogic;
if (typeof document !== "undefined") {
  try {
    const kept = localStorage.getItem("operon-map-theme");
    if (kept === "light" || kept === "dark") document.documentElement.dataset.theme = kept;
  } catch (error) { /* a file page may refuse storage */ }
}
if (typeof document !== "undefined") main();

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
const WORK = [["modify", "変更", "既存の振る舞いを変える（route modify）"],
  ["bugfix", "不具合", "不具合を直す（route bugfix。root-cause から）"],
  ["docs", "文書", "文書を直す（route docs）"],
  ["test", "テスト", "テストのない振る舞いにテストを足す"]];
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

// ---- the work basket -----------------------------------------------------
const basket = new Map();
let workType = "modify";
let basketOpen = false;
function basketBox(kind, id) {
  const key = kind + "|" + id;
  return '<label class="bk" title="作業かごに入れる"><input type="checkbox" data-basket="' + esc(key) + '"' +
    (basket.has(key) ? " checked" : "") + "> かご</label>";
}
function basketLine(key) {
  const [kind, id] = key.split("|");
  if (kind === "feature") return "- 機能「" + featureTitle(id) + "」(" + id + "): docs/features/" + id + "/brief.md と docs/features/" + id + "/spec.md";
  if (kind === "domain") return "- ドメイン「" + domainTitle(id) + "」(" + id + "): docs/features/domains.yaml の " + id;
  if (kind === "screen") return "- 画面「" + flowTitle(id) + "」(" + id + "): docs/features/flow.yaml の " + id +
    (SHOTS[id] ? "、写真 docs/images/screens/" + id + ".jpg" : "");
  if (kind === "term") return "- 用語「" + termWord(id) + "」(" + id + "): docs/features/glossary.yaml の " + id;
  const [feature, number] = id.split("#");
  const row = REQS.find(r => r.feature === feature && String(r.number) === number);
  return "- 振る舞い: 機能「" + featureTitle(feature) + "」の " + number + " 番 — docs/features/" + feature + "/spec.md の Behaviour " + number +
    (row ? "「" + row.text + "」" : "");
}
function instruction() {
  const work = WORK.find(w => w[0] === workType);
  return "Operon の次の作業を feature-pilot で進めてください（.claude/skills/feature-pilot/SKILL.md から始める）。\n" +
    "種類: " + work[1] + " — " + work[2] + "\n対象:\n" + Array.from(basket.keys()).map(basketLine).join("\n") +
    "\n読むもの: docs/features/index.yaml、docs/features/domains.yaml、docs/features/glossary.yaml、docs/features/flow.yaml\n" +
    "してほしいこと:\n（ここに書いてください）\n";
}
function renderBasket() {
  $("bk-n").textContent = basket.size ? " " + basket.size : "";
  if (!basketOpen) return;
  $("bk-body").innerHTML = basket.size ? '<ul class="ways">' + Array.from(basket.keys()).map(key =>
    "<li>" + esc(basketLine(key).slice(2)) + ' <button class="fchip" data-unbasket="' + esc(key) + '">外す</button></li>').join("") + "</ul>" +
    '<div class="filters">' + WORK.map(w => '<button data-work="' + w[0] + '" aria-pressed="' + (w[0] === workType) + '" title="' + esc(w[2]) + '">' + w[1] + "</button>").join("") + "</div>" +
    '<textarea id="bk-text" rows="14" readonly>' + esc(instruction()) + "</textarea>" +
    '<div class="tools"><span id="bk-said"></span><button data-empty-basket="1">空にする</button><button id="bk-copy" class="primary">指示をコピー</button></div>'
    : '<p class="empty">まだ何も入っていません</p><p class="what">機能・画面・振る舞い・用語・ドメインの「かご」に印を付けると、ここに集まります。</p>';
}
function copyInstruction() {
  const area = $("bk-text");
  const failed = () => { area.focus(); area.select(); $("bk-said").textContent = "コピーできませんでした。選択された文を手でコピーしてください"; };
  if (!navigator.clipboard) { failed(); return; }
  navigator.clipboard.writeText(area.value).then(() => { $("bk-said").textContent = "コピーしました"; }, failed);
}

// ---- 概要 -----------------------------------------------------------------
function kpi(route, number, label, part, whole) {
  return '<button class="kpi" data-go="' + esc(route) + '"><b>' + number + "</b><span>" + label + "</span>" +
    (whole !== undefined ? '<i><em style="width:' + bar(part, whole) + '%"></em></i>' : "") + "</button>";
}
function renderOverview() {
  const needs = M.changes.filter(c => c.group === "needs");
  const flight = M.changes.filter(c => c.group === "flight");
  const pinnedPct = bar(OV.pinned || 0, OV.requirements || 0);
  let h = '<p class="lead">Operon のいまの姿です。数字を押すと、その一覧が開きます。</p><div class="kpis">' +
    kpi("#features", OV.features || 0, "機能") +
    kpi("#domains", OV.domains || 0, "ドメイン") +
    kpi("#canvas", OV.screens || 0, "画面（写真 " + (OV.photographed || 0) + "）", OV.photographed || 0, OV.screens || 0) +
    kpi("#requirements", OV.requirements || 0, "要件（テストあり " + (OV.pinned || 0) + " · " + pinnedPct + "%）", OV.pinned || 0, OV.requirements || 0) +
    kpi("#glossary", OV.terms || 0, "用語") +
    kpi("#changes", OV.changes_total || 0, "変更（進行中 " + ((OV.changes || {}).flight || 0) + " · 判断待ち " + ((OV.changes || {}).needs || 0) + "）") +
    "</div>";
  h += '<section><h2>あなたの判断を待っています <small>(' + needs.length + ")</small></h2>" + (needs.length ? '<div class="cards">' +
    needs.map(c => card(c)).join("") + "</div>" : '<div class="empty">判断を待っている変更はありません</div>') + "</section>";
  h += "<section><h2>いま進めている変更 <small>どの段階にあるか</small></h2>" + (!flight.length ? '<div class="empty">進行中の変更はありません</div>' :
    '<div class="progress">' + M.stages.map(s => {
      const here = flight.filter(c => c.stage === s);
      return '<div class="col' + (here.length ? " has" : "") + '"><h4>' + stageWord(s) + "</h4>" + here.map(c => card(c, true)).join("") + "</div>";
    }).join("") + "</div>") + "</section>";
  const per = OV.per_domain || [];
  h += '<div class="cols2"><section><h2>ドメイン別 <small>テストで守られている要件の割合</small></h2><div>' + (per.length ? per.map(d =>
    '<div class="dbar"><button class="fchip" data-domain="' + esc(d.id) + '">' + esc(domainTitle(d.id)) + '</button><span class="track2"><em style="width:' +
    bar(d.pinned, d.requirements) + '%"></em></span><small>' + d.pinned + " / " + d.requirements + " · 機能 " + d.features + "</small></div>").join("")
    : '<div class="empty">ドメインがありません</div>') + "</div></section>";
  const least = OV.least_pinned || [];
  h += "<section><h2>テストで守られていない要件が多い機能 <small>テストなしの件数</small></h2>" + (least.length ? '<ul class="rows">' + least.map(x =>
    '<li><button data-go="#feature/' + esc(x.feature) + '/reqs"><span class="n">' + x.bare + ' 件</span><span class="x">' + esc(featureTitle(x.feature)) +
    '</span><span class="r">' + esc(domainTitle((featureById.get(x.feature) || {}).domain)) + "</span></button></li>").join("") + "</ul>"
    : '<div class="empty">すべての要件にテストがあります</div>') + "</section></div>";
  const recent = M.changes.filter(c => c.group === "done").slice(0, 10);
  const missing = FLOW.screens.filter(s => !s.shot);
  h += '<div class="cols2"><section><h2>最近入った変更</h2><ul class="rows">' + recent.map(c =>
    '<li><button data-open="' + esc(c.name) + '"><span class="n">' + esc(c.number) + '</span><span class="x">' + esc(c.title) +
    '</span><span class="r">' + esc(lastDate(c)) + "</span></button></li>").join("") + "</ul></section>";
  h += "<section><h2>写真がない画面 <small>(" + missing.length + ")</small></h2>" + (M.shots_error ? '<p class="warn">' + esc(M.shots_error) + "</p>" : "") +
    (missing.length ? '<ul class="rows">' + missing.map(s => '<li><button data-screen="' + esc(s.id) + '"><span class="x">' + esc(s.title) +
      '</span><span class="r">' + esc(s.shot_problem || "") + "</span></button></li>").join("") + "</ul>" : '<div class="empty">すべての画面に写真があります</div>') +
    "</section></div>";
  $("v-overview").innerHTML = h;
}

// ---- 機能 ------------------------------------------------------------------
function renderFeatureBadge(f) {
  const stage = f.stage ? " · " + stageWord(f.stage) : "";
  return '<span class="badge ' + f.badge + '">' + BADGE[f.badge] + esc(f.badge === "steady" ? "" : stage) + "</span>";
}
function featureCard(f) {
  const open = f.changes.filter(l => l.link !== "estimated" && ["needs", "flight"].includes(byName.get(l.name).group));
  const rows = REQS.filter(r => r.feature === f.id);
  return '<button class="fcard ' + f.badge + '" data-feature="' + esc(f.id) + '"><span class="t">' + esc(f.title) +
    '</span><span class="s">' + esc(f.summary) + '</span><span class="b">' + renderFeatureBadge(f) +
    "<span>要件 " + rows.filter(r => r.pinned).length + "/" + rows.length + "</span>" +
    (open.length ? "<span>変更 " + esc(open.map(l => byName.get(l.name).number).join("・")) + "</span>" : "") + "</span></button>";
}
function wrapCard(f) { return '<div class="fwrap">' + featureCard(f) + basketBox("feature", f.id) + "</div>"; }
function bandShot(band) {
  const s = FLOW.screens.find(x => x.screen === band && x.shot && x.kind === "page") || FLOW.screens.find(x => x.screen === band && x.shot);
  return s ? s.id : "";
}
function renderFeatures() {
  $("p-catalogue-error").innerHTML = M.catalogue_error ? '<div class="box needs"><h3>' + esc(M.catalogue_error) +
    "</h3>製品の見方は出せません。変更一覧は読めます。</div>" : "";
  $("p-screens-n").textContent = FEATURES.length ? "(" + FEATURES.length + " 機能)" : "";
  const shown = M.screens.map(sc => ({ sc, fs: FEATURES.filter(f => f.screen === sc.id) })).filter(x => x.fs.length);
  $("p-screen-list").innerHTML = shown.length ? shown.map(({ sc, fs }) => {
    const id = bandShot(sc.id);
    const picture = id ? '<button class="thumb" data-screen="' + esc(id) + '">' + thumbImg(id, sc.label + "の画面") + "</button>"
      : (sc.image && M.images[sc.image] ? '<div class="shot">' + imgTag(M.images[sc.image], sc.label + "の画面") + "</div>" : "");
    return '<div class="screen' + (picture ? "" : " noshot") + '">' + picture + '<div><h3>' + esc(sc.label) +
      ' <small style="font-weight:400;color:var(--soft)">' + fs.length + ' 機能</small></h3><div class="fcards">' + fs.map(wrapCard).join("") + "</div></div></div>";
  }).join("") : '<div class="empty">機能の一覧がありません</div>';
  const loose = M.unmapped.length;
  $("p-unmapped").innerHTML = (loose ? '<p class="lead">' + (M.changes.length - loose) + " 件の変更は機能に結び付いています。残りの " + loose +
    " 件は、開発の進め方や道具の変更など、機能に紐づかないもの（または記録に機能名がない古い変更）です。" +
    ' <button class="fchip" data-go="#changes">変更一覧で見る</button></p>' : "") +
    (M.features_omitted.length ? '<p class="warn">機能が多すぎるため、載せていません: ' + esc(M.features_omitted.join(", ")) + "</p>" : "");
}
function liveSteps(f) {
  const text = f.spec ? f.spec.text : "";
  const at = text.indexOf("## Seen in the running app");
  if (at < 0) return [];
  let body = text.slice(at).split("\n").slice(1);
  const end = body.findIndex(l => l.startsWith("## "));
  if (end >= 0) body = body.slice(0, end);
  return body.map(l => l.trim()).filter(Boolean);
}
let reqBareOnly = false;
function featureSection(id, title, count, inner) {
  return '<section class="fsec" id="' + id + '"><h3>' + title + (count !== undefined ? " <small>" + count + "</small>" : "") + "</h3>" + inner + "</section>";
}
function renderFeature(f, tab) {
  const rows = REQS.filter(r => r.feature === f.id);
  const pinned = rows.filter(r => r.pinned).length;
  const screens = (f.flow_screens || []).map(id => flowById.get(id)).filter(Boolean);
  const own = TERMS.filter(t => t.feature === f.id);
  const used = TERMS.filter(t => t.feature !== f.id && t.used_by.includes(f.id));
  const steps = liveSteps(f);
  const sections = [["sec-about", "概要"], ["sec-screens", "画面", screens.length], ["sec-reqs", "要件", pinned + "/" + rows.length],
    ["sec-live", "実アプリで確認", steps.length], ["sec-terms", "用語", own.length], ["sec-code", "コード", (f.modules || []).length],
    ["sec-changes", "変更履歴", f.changes.length], ["sec-docs", "文書"]];
  let h = '<div class="fhead"><button class="back" data-go="#features">← 機能の一覧</button><h2>' + esc(f.title) + renderFeatureBadge(f) + "</h2>" +
    '<div class="meta2"><span>画面: <b>' + esc(screenLabel(f.screen)) + "</b></span><span>ドメイン: " + domainChip(f.domain) + "</span>" +
    "<span>要件 " + pinned + "/" + rows.length + " がテストあり</span><span>変更 " + f.changes.length + " 件</span>" + basketBox("feature", f.id) + "</div></div>";
  h += '<div class="fpage"><nav class="toc" id="f-toc">' + sections.map(s => '<a href="#' + "feature/" + esc(f.id) + "/" + s[0].slice(4) +
    '" data-sec="' + s[0] + '">' + s[1] + (s[2] !== undefined ? "<small>" + s[2] + "</small>" : "") + "</a>").join("") + "</nav><div>";
  h += featureSection("sec-about", "概要", undefined, "<p>" + esc(f.summary) + "</p>" + (f.brief ? mdBlock(f.brief.text) : '<div class="box needs">brief.md がありません</div>'));
  h += featureSection("sec-screens", "画面", screens.length, screens.length ? '<p class="what">押すと、その画面を写真で開きます。</p><div class="strip">' +
    screens.map(screenThumb).join("") + "</div>" : '<p class="empty">この機能の画面はまだ流れにありません</p>');
  const shownRows = reqBareOnly ? rows.filter(r => !r.pinned) : rows;
  h += featureSection("sec-reqs", "要件", pinned + " / " + rows.length + " がテストあり",
    '<div class="cov"><i><em style="width:' + bar(pinned, rows.length) + '%"></em></i>' + bar(pinned, rows.length) + "%" +
    ' <label class="toggle"><input type="checkbox" id="f-bare"' + (reqBareOnly ? " checked" : "") + "> テストの記録なしだけ</label></div>" +
    (shownRows.length ? '<table class="all rq"><tbody>' + shownRows.map(r => '<tr class="' + (r.pinned ? "" : "bare") + '"><td>' + basketBox("behaviour", r.feature + "#" + r.number) +
      '</td><td class="num">' + r.number + '</td><td class="md">' + inline(r.text) + "</td><td>" +
      (r.tests.length ? r.tests.map(t => "<code>" + esc(t) + "</code>" + (r.missing.includes(t) ? ' <span class="pill needs">見つからない</span>' : "")).join("<br>")
        : '<span class="pill needs">テストの記録なし</span>') + "</td></tr>").join("") + "</tbody></table>" : '<p class="empty">なし</p>'));
  h += featureSection("sec-live", "実アプリで確認", steps.length, steps.length ? '<p class="what">scripts/verify-app.py がこの手順でアプリを動かして確かめます。</p><ol class="steps2">' +
    steps.map(s => { const sp = s.indexOf(" "); return "<li><code>" + esc(sp < 0 ? s : s.slice(0, sp)) + "</code> " + esc(sp < 0 ? "" : s.slice(sp + 1)) + "</li>"; }).join("") + "</ol>"
    : '<p class="empty">この機能には、実アプリで確かめる手順がまだありません</p>');
  h += featureSection("sec-terms", "用語", own.length, '<div class="terms">' + (own.map(termCard).join("") || '<p class="empty">なし</p>') + "</div>" +
    "<h4>BRIEF が使っているほかの用語</h4>" + termChips(used.map(t => t.id)));
  h += featureSection("sec-code", "コード", (f.modules || []).length, "<h4>この機能が主に使うコード</h4>" +
    (f.paths.length ? '<p class="paths">' + f.paths.map(p => "<code>" + esc(p) + "</code>").join(" ") + "</p>" : '<p class="empty">なし</p>') +
    "<h4>ドメイン「" + esc(domainTitle(f.domain)) + "」のコード</h4>" + ((domainById.get(f.domain) || { modules: [] }).modules.length ?
      '<table class="all"><tbody>' + domainById.get(f.domain).modules.map(m => "<tr><td><code>" + esc(m.path) + "</code></td><td>" + esc(m.about) + "</td></tr>").join("") + "</tbody></table>"
      : '<p class="empty">なし</p>'));
  h += featureSection("sec-changes", "変更履歴", f.changes.length, featureProgress(f));
  h += featureSection("sec-docs", "文書", undefined, '<div class="docbtns">' +
    '<button class="ghost" data-doc="brief|' + esc(f.id) + '">BRIEF（人向けの説明）</button>' +
    '<button class="ghost" data-doc="spec|' + esc(f.id) + '">SPEC（AI 向けの仕様・英語）</button>' +
    (f.manual_text ? '<button class="ghost" data-doc="manual|' + esc(f.id) + '">マニュアル「' + esc(f.manual) + "」</button>" : "") + "</div>");
  $("v-feature").innerHTML = h + "</div></div>";
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
let reqFeature = "", reqDomain = "", reqTests = "";
function renderRequirements() {
  const opt = (value, label, current) => '<option value="' + esc(value) + '"' + (value === current ? " selected" : "") + ">" + esc(label) + "</option>";
  $("rq-filters").innerHTML = '<select id="rq-domain" aria-label="ドメイン">' + opt("", "すべてのドメイン", reqDomain) + DOMAINS.map(d => opt(d.id, d.title, reqDomain)).join("") + "</select>" +
    '<select id="rq-feature" aria-label="機能">' + opt("", "すべての機能", reqFeature) + FEATURES.filter(f => !reqDomain || f.domain === reqDomain).map(f => opt(f.id, f.title, reqFeature)).join("") + "</select>" +
    '<select id="rq-tests" aria-label="テスト">' + opt("", "テストの有無を問わない", reqTests) + opt("yes", "テストの記録あり", reqTests) + opt("no", "テストの記録なし", reqTests) + "</select>";
  const rows = REQS.filter(r => (!reqDomain || r.domain === reqDomain) && (!reqFeature || r.feature === reqFeature) &&
    (!reqTests || (reqTests === "yes") === r.pinned) &&
    (!query || (r.text + " " + featureTitle(r.feature) + " " + r.tests.join(" ")).toLowerCase().includes(query)));
  const bare = REQS.filter(r => !r.pinned).length;
  $("rq-n").textContent = "(" + rows.length + " / " + REQS.length + " · テストの記録なし " + bare + " 件)";
  $("rq-list").innerHTML = rows.length ? '<table class="all rq"><thead><tr><th></th><th>機能</th><th>番号</th><th>内容</th><th>テスト</th></tr></thead><tbody>' +
    rows.map(r => '<tr class="' + (r.pinned ? "" : "bare") + '"><td>' + basketBox("behaviour", r.feature + "#" + r.number) + "</td><td>" + chip("data-feature", r.feature, featureTitle(r.feature)) +
      '</td><td class="num">' + r.number + '</td><td class="md">' + inline(r.text) + "</td><td>" +
      (r.tests.length ? r.tests.map(t => "<code>" + esc(t) + "</code>" + (r.missing.includes(t) ? ' <span class="pill needs">見つからない</span>' : "")).join("<br>")
        : '<span class="pill needs">テストの記録なし</span>') + "</td></tr>").join("") +
    "</tbody></table>" : '<div class="empty">一致するものはありません</div>';
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
  domain: "domains", glossary: "glossary", "glossary-screens": "glossary", term: "glossary", requirements: "requirements", changes: "changes" };
const PANES = ["overview", "features", "feature", "screen", "canvas", "domains", "domain", "glossary", "requirements", "changes"];
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
  if (r.view === "overview") renderOverview();
  else if (r.view === "features") renderFeatures();
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
  } else renderChanges();
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
  return Array.from(pane.querySelectorAll(".kpi, .fcard, .dcard, .thumb, tr.pick, .rows button, .plist button, ul.ways2 button, .card"))
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

// ---- events -------------------------------------------------------------------
document.addEventListener("click", event => {
  if (event.target.closest("label.bk") || event.target.closest("label.toggle")) return;
  const t = event.target.closest("[data-way],[data-trail],[data-hit],[data-go],[data-view],[data-screen],[data-feature],[data-domain],[data-term],[data-doc],[data-open],[data-more],[data-group],[data-tab],[data-unbasket],[data-work],[data-empty-basket]");
  if (!t) return;
  const d = t.dataset;
  if (d.way !== undefined) takeWay(Number(d.way));
  else if (d.trail !== undefined) { const n = Number(d.trail); walk.at = n; go({ view: "screen", id: walk.current() }); }
  else if (d.hit !== undefined) openHit(Number(d.hit));
  else if (d.go) { if (location.hash === d.go) render(); else location.hash = d.go; }
  else if (d.view) go({ view: d.view, id: d.view === "screen" ? walk.current() : undefined });
  else if (d.screen) { walk.visit(d.screen); go({ view: "screen", id: d.screen }); }
  else if (d.feature) go({ view: "feature", id: d.feature });
  else if (d.domain) go({ view: "domain", id: d.domain });
  else if (d.term) go({ view: "term", id: d.term });
  else if (d.doc) { const [kind, id] = d.doc.split("|"); openDoc(kind, id); }
  else if (d.open) show(d.open);
  else if (d.more) { expanded.add(d.more); renderChanges(); }
  else if (d.group !== undefined) { groupFilter = d.group; renderChanges(); }
  else if (d.tab) show(current.name, d.tab);
  else if (d.unbasket) { basket.delete(d.unbasket); render(); }
  else if (d.work) { workType = d.work; renderBasket(); }
  else if (d.emptyBasket) { basket.clear(); render(); }
});
$("body").addEventListener("click", event => { if (event.target.id === "raw-toggle") { rawView = !rawView; renderDetail(); } });
document.addEventListener("change", event => {
  const box = event.target.closest("input[data-basket]");
  if (box) { if (box.checked) basket.set(box.dataset.basket, true); else basket.delete(box.dataset.basket); renderBasket(); return; }
  const id = event.target.id;
  if (id === "rq-domain") { reqDomain = event.target.value; reqFeature = ""; renderRequirements(); }
  else if (id === "rq-feature") { reqFeature = event.target.value; renderRequirements(); }
  else if (id === "rq-tests") { reqTests = event.target.value; renderRequirements(); }
  else if (id === "f-bare") { reqBareOnly = event.target.checked; const r = L.parseRoute(location.hash); renderFeature(featureById.get(r.id), "reqs"); }
  else if (id === "sc-areas") { showAreas = event.target.checked; const w = $("sc-win"); if (w) w.classList.toggle("show", showAreas); }
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
$("bk-open").addEventListener("click", () => { basketOpen = true; $("basket").hidden = false; renderBasket(); });
$("bk-close").addEventListener("click", () => { basketOpen = false; $("basket").hidden = true; });
$("basket").addEventListener("click", event => { if (event.target.id === "bk-copy") copyInstruction(); });
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
function themeLabel() { $("theme").textContent = document.documentElement.dataset.theme === "dark" ? "ライト" : "ダーク"; }
$("theme").addEventListener("click", () => {
  const next = document.documentElement.dataset.theme === "dark" ? "light" : "dark";
  document.documentElement.dataset.theme = next;
  try { localStorage.setItem("operon-map-theme", next); } catch (error) { /* a file page may refuse storage */ }
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
    else if (basketOpen) { basketOpen = false; $("basket").hidden = true; }
    return;
  }
  if (typing) return;
  const r = L.parseRoute(location.hash);
  if (action === "search") { event.preventDefault(); $("q").focus(); $("q").select(); }
  else if (action === "help") $("help").hidden = !$("help").hidden;
  else if (action === "next") moveNav(1);
  else if (action === "prev") moveNav(-1);
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
$("meta").innerHTML = "生成 " + esc(M.generated) + (M.worktrees_read ? "" : ' · <span class="warn">他の作業コピーを読めませんでした</span>') +
  (M.shots_error ? ' · <span class="warn">' + esc(M.shots_error) + "</span>" : "");
render();
}

/* The webview side of the app. Every button lands in cli(): the same argv the
 * headless CLI takes (docs/DECISIONS/0004), so a click and a `cargo run --`
 * line are the same code path, down to the same JSON. The one piece of UI-only
 * logic is cancel: a crawl is issued as bounded --budget chunks, and Cancel
 * raises a flag the chunk loop checks at the next chunk boundary. The DB keeps
 * the progress (sync_state), so stopping is always resume-safe.
 *
 * Attribution: the footer link to TheWoWDB is developer-terms required and is
 * static HTML, so no JS path can hide it. */

var cancel_requested = false;
var sync_busy = false;
var last_hits = [];
var watched = [];

// Escape once, everywhere: names from the API go into text nodes, not markup.
function esc(text) {
  return String(text)
    .replace("&", "&amp;")
    .replace("<", "&lt;")
    .replace(">", "&gt;")
    .replace('"', "&quot;");
}

function num(value) {
  if (value === null || value === undefined) { return null; }
  return Number(value).toLocaleString("en-US");
}

// v0.9.4 (ADR 0015): the money cell. The WORDS are not computed here - Rust's ONE
// money formatter (money_text in db.rs) wrote them into the very answer this cell
// reads, so the UI never does copper maths and cannot drift a price or invent the
// game's shape. The cell shows what the GAME writes (`1g 75c`, zero denominations
// dropped exactly as the DB's own tooltip copies write them) and the EXACT copper
// - the number the price sorts, the scripts and docs/DATABASE.md's numeric
// contract use - rides in the hover, so money never hides a number. A row with no
// price answers "no price", never 0c: "the AH has no offer" and "costs nothing"
// stay two answers (ADR 0004's rule against made-up zeros, again).
function money_cell(money, copper) {
  if (money === null || money === undefined) { return '<span class="muted">no price</span>'; }
  return '<span class="moneycell" title="' + esc(num(copper) + " copper exact") + '">' + esc(money) + "</span>";
}

// The one door to Rust: argv in, parsed verb JSON out. Failures throw the
// envelope's error object, which carries kind/exit/message/retry_after, so each
// handler has exactly one catch and the UI never re-derives an error shape.
async function cli_ok(argv) {
  var envelope;
  try {
    envelope = await window.__TAURI__.core.invoke("cli_call", { argv: argv });
  } catch (transport) {
    throw { kind: "network", exit: 3, message: "the GUI command did not reach Rust: " + String(transport) };
  }
  var parsed;
  try {
    parsed = JSON.parse(envelope);
  } catch (bad) {
    throw { kind: "decode", exit: 2, message: "the verb returned something that is not JSON" };
  }
  if (parsed.ok) { return parsed.result; }
  throw parsed.error;
}

function set_note(element, text, bad) {
  element.textContent = text;
  element.classList.remove("bad");
  if (bad) { element.classList.add("bad"); }
}

function fmt_state(state) {
  if (state.status === "never") { return "never run"; }
  if (state.status === "complete") { return "complete"; }
  if (state.status === "stopped") { return "stopped at page " + state.page + " of " + state.total; }
  if (state.status === "cancelled") { return "cancelled at page " + state.page + " of " + state.total; }
  if (state.status === "failed") { return "failed: " + String(state.message); }
  return String(state.status);
}

// ---- status card -----------------------------------------------------------
async function refreshStatus() {
  try {
    var status = await cli_ok(["status"]);
    var grid = [];
    grid.push(["db file", status.db.path]);
    grid.push(["items stored", num(status.items)]);
    grid.push(["with prices", num(status.coverage.priced) + " of " + num(status.coverage.items)]);
    grid.push(["details fetched", num(status.coverage.detailed) + " of " + num(status.coverage.items)]);
    grid.push(["checked as gone upstream", num(status.coverage.missing) + " answered 404, not re-fetched"]);
    grid.push(["icons copied", num(status.coverage.icons) + " of " + num(status.coverage.detailed) + " answered records"]);
    if (num(status.coverage.icon_missing) !== null && num(status.coverage.icon_missing) !== 0) {
      grid.push(["checked: no icon upstream", num(status.coverage.icon_missing) + " icon files gone, never asked again"]);
    }
    grid.push(["descriptions copied", num(status.coverage.tooltips) + " of " + num(status.coverage.detailed) + " answered records (the forever/item body carries no description: the tooltip door does, docs/API.md)"]);
    if (status.coverage.tooltip_missing !== null && status.coverage.tooltip_missing > 0) {
      grid.push(["checked: no description upstream", num(status.coverage.tooltip_missing) + " tooltips answered 404, never asked again"]);
    }
    if (status.coverage.edited !== null && status.coverage.edited > 0) {
      grid.push(["fields the user said", num(status.coverage.edited) + " rows wear a manual override (the crawl keeps them, ADR 0010)"]);
    }
    if (status.coverage.manual !== null && status.coverage.manual > 0) {
      grid.push(["items the user made", num(status.coverage.manual) + " rows in the reserved id block (never crawled, ADR 0010)"]);
    }
    if (status.coverage.deleted !== null && status.coverage.deleted > 0) {
      grid.push(["hidden by the user", num(status.coverage.deleted) + " tombstones: every query and the card refuse them"]);
    }
    grid.push(["price samples", num(status.coverage.history) + " stored (" + num(status.coverage.detailed) + " fresh detail writes add one each, 304s add none)"]);
    if (num(status.coverage.min_price) !== null) {
      grid.push(["price range", status.coverage.min_price_money + " .. " + status.coverage.max_price_money + " (" + num(status.coverage.min_price) + " .. " + num(status.coverage.max_price) + " copper)"]);
    }
    if (num(status.coverage.avg_price) !== null) {
      grid.push(["average price", status.coverage.avg_price_money + " (floors " + num(Math.floor(status.coverage.avg_price)) + " copper: money has no fraction of a copper)"]);
    }
    grid.push(["list crawl", fmt_state(status.list)]);
    grid.push(["detail crawl", fmt_state(status.detail)]);
    grid.push(["icon copies", fmt_state(status.icons)]);
    grid.push(["description copies", fmt_state(status.tooltip)]);
    var html = "";
    for (var i = 0; i < grid.length; i = i + 1) {
      html = html + "<span>" + esc(grid[i][0]) + "</span><b>" + esc(grid[i][1]) + "</b>";
    }
    document.getElementById("status-grid").innerHTML = html;
    var seen_fields = grid.length;
    set_note(document.getElementById("status-note"),
      "db chosen by " + status.db.source + " — " + seen_fields + " fields read from " + status.db.path
      + ", and nothing here touches the network", false);
    if (!sync_busy) {
      set_note(document.getElementById("sync-note"), sync_idle_note(), false);
    }
  } catch (problem) {
    show_error(problem);
  }
}

function show_error(problem) {
  set_note(document.getElementById("status-note"),
    problem.message + (problem.retry_after ? " (retry after " + problem.retry_after + "s)" : ""), true);
}

function sync_idle_note() {
  return "idle — a stopped or cancelled crawl resumes where it left off";
}

// ---- search card -----------------------------------------------------------
async function runSearch() {
  var text = document.getElementById("search-text").value;
  var quality = document.getElementById("search-quality").value;
  var sort = document.getElementById("search-sort").value;
  var priced = document.getElementById("search-priced").checked;
  var where = document.getElementById("search-where").value;
  var argv = ["search"];
  if (text !== "") { argv.push(text); }
  if (quality !== "") { argv.push("--quality", quality); }
  argv.push("--where", where);
  argv.push("--sort", sort);
  if (priced) { argv.push("--priced"); }
  argv.push("--icons");
  argv.push("--limit", "40");
  try {
    var found = await cli_ok(argv);
    last_hits = found.hits;
    await refreshWatched();
    var rows = "";
    for (var i = 0; i < found.hits.length; i = i + 1) {
      rows = rows + hit_row(found.hits[i]);
    }
    document.querySelector("#hits tbody").innerHTML = rows;
    var shown = found.hits.length;
    var note = "showing " + shown + " of " + num(found.total) + " — details of " + num(found.total) + " items cost one request each, and the sync stage caches them; the price column shows the game's OWN money words (hover: the exact copper, the number the sorts and the scripts read), the icon column shows the DB's OWN copy where the icons stage wrote one (\u25c7 means no bytes: never fetched, or checked as gone upstream \u2014 the hover says which, and nothing is hotlinked)";
    if (found.total === 0) { note = "no item matches — the search runs against the local DB only"; }
    set_note(document.getElementById("search-note"), note, false);
    document.getElementById("btn-search").classList.remove("running");
  } catch (problem) {
    set_note(document.getElementById("search-note"), problem.message, true);
  }
}

// The list icon cell (v0.9.3, ADR 0014): the DB's OWN icon bytes as a data URI, the
// same rule the card has ridden since v0.6.0 - upstream's icon_url rides the row
// shape but is NEVER hotlinked (ADR 0008's rule), so a row with no bytes has nothing
// to draw and shows a placeholder glyph. WHICH no-icon state a row is in is still the
// row's own stamps: the doors send icon_at (copied) and icon_missing_at (checked as
// gone upstream) always, and the base64 payload only when asked with --icons - which
// every list door on this page asks for.
function icon_cell(one) {
  if (one.icon_b64) {
    return '<img class="listicon" src="data:image/jpeg;base64,' + esc(one.icon_b64) + '" alt="' + esc(one.name) + ' icon">';
  }
  var why = "icon never fetched (the icons stage will copy it)";
  if (one.icon_missing_at !== null && one.icon_missing_at !== undefined) {
    why = "icon checked as not there upstream " + when(one.icon_missing_at);
  }
  return '<span class="noicon" title="' + esc(why) + '">\u25c7</span>';
}

function hit_row(hit) {
  var quality = hit.quality || "unknown";
  var link = "";
  if (hit.html_url) {
    link = '<a class="source" href="' + esc(hit.html_url) + '">view</a>';
  }
  var price_cell = money_cell(hit.price_min_money, hit.price_min);
  var watch_label = "watch";
  if (watched.indexOf(hit.id) >= 0) { watch_label = "unwatch"; }
  var toggle = '<button class="rowact" data-id="' + esc(String(hit.id)) + '">' + watch_label + "</button>";
  return "<tr>"
    + '<td class="col-id">' + esc(String(hit.id)) + "</td>"
    + '<td class="namecell">' + esc(hit.name) + " " + link + " " + toggle + "</td>"
    + '<td class="iconcell">' + icon_cell(hit) + "</td>"
    + '<td class="qualitycell"><span class="chip c-' + esc(quality) + '">' + esc(quality) + "</span></td>"
    + '<td class="pricecell">' + price_cell + "</td>"
    + "</tr>";
}

// ---- sync card -------------------------------------------------------------
// ---- liveness while one chunk is in flight --------------------------------
// A chunk is seconds (pacing at 2/s) or minutes (a Retry-After wait on a 429),
// and the await for it is exactly where the page could look dead. It does not:
// once a second the status card re-reads the DB (coverage grows per page in
// list and per item in detail) and the sync note carries the chunk's elapsed
// time. Honest about position: sync_state only moves at chunk ends (finish()
// in src/sync.rs), so we print elapsed seconds, not a mid-chunk cursor. The
// polls are also the launch audit's witness that the webview runs during a
// crawl - the Rust log interleaves status verbs between the chunk verbs.
var tick_started = 0;
var tick_label = "";

function tick_body() {
  var secs = Math.round((Date.now() - tick_started) / 1000);
  refreshStatus();
  set_note(document.getElementById("sync-note"),
    tick_label + ", chunk in flight " + secs + "s — cancel applies at the chunk end", false);
}

function start_tick(label) {
  tick_started = Date.now();
  tick_label = label;
  return setInterval(tick_body, 1000);
}

async function runSync(stage) {
  if (sync_busy) {
    set_note(document.getElementById("sync-note"), "a crawl is already running — cancel it first", true);
    return "already-running";
  }
  sync_busy = true;
  cancel_requested = false;
  var chunk = stage === "list" ? 10 : 25;
  var seen = 0;
  var tick = 0;
  var outcome = "failed";
  try {
    while (true) {
      set_note(document.getElementById("sync-note"),
        "running " + stage + ": " + seen + " requests spent, chunks of " + chunk + " — cancel stops at the next chunk", false);
      document.getElementById("btn-cancel").classList.add("running");
      tick = start_tick("running " + stage + ": " + seen + " requests spent, chunks of " + chunk);
      var report = await cli_ok(["sync", "--stage", stage, "--budget", String(chunk)]);
      clearInterval(tick);
      seen = seen + report.requests;
      if (report.complete) {
        set_note(document.getElementById("sync-note"),
          stage + " stage complete: " + num(report.items) + " seen, " + num(report.written) + " written, " + num(report.unchanged) + " reused", false);
        outcome = "complete";
        break;
      }
      if (report.requests === 0) {
        set_note(document.getElementById("sync-note"),
          "the crawl made no progress this chunk — stopping so we do not burn the API", true);
        outcome = "stalled";
        break;
      }
      if (cancel_requested) {
        set_note(document.getElementById("sync-note"),
          "cancelled: " + num(report.items) + " seen so far — the next run resumes from here", false);
        outcome = "cancelled";
        break;
      }
    }
  } catch (problem) {
    if (tick !== 0) { clearInterval(tick); }
    var extra = problem.retry_after ? " (the API asked to wait " + problem.retry_after + "s; a later chunk will retry)" : "";
    set_note(document.getElementById("sync-note"), problem.message + extra, true);
  }
  document.getElementById("btn-cancel").classList.remove("running");
  sync_busy = false;
  await refreshStatus();
  return outcome;
}

function requestCancel() {
  cancel_requested = true;
  if (!sync_busy) {
    set_note(document.getElementById("sync-note"),
      "cancel asked for while idle — the next chunk loop stops before it starts", false);
  }
}

// ---- pull-all: finish the catalogue in one click --------------------------------
// One flow, two stages: walk the list to the end, then fetch a detail record for
// every item that never had one. Honest only with the pending rule in step ("all
// info" = every item has a detail record; NULL price on a fetched item simply
// means not on the AH, docs/DATABASE.md). Cancel semantics: the stage in flight
// stops at its chunk end and the flow does not jump ahead - the next click
// resumes the list from sync_state, and the pending queue recomputes what
// details remain. After a complete pull there is nothing pending: later syncs
// only revalidate (AH prices are live data, so re-checks still make sense).
async function runPullAll() {
  set_note(document.getElementById("sync-note"),
    "pull-all: list stage to the end (~193 pages, minutes at 2/s), then details for everything never fetched (~19k records, hours at 2/s), then the icon FILES of every record that answered (one request each, checked once, never asked again), then the DESCRIPTION copy of every answered record (the tooltip door, one request each, ≈ 2.7 h at the measured pace) — Cancel stops at the next chunk boundary; nothing is hotlinked, the DB gets the bytes", false);
  var list_out = await runSync("list");
  if (list_out === "cancelled") {
    set_note(document.getElementById("sync-note"),
      "pull-all stopped by Cancel in the list stage: the next click resumes the list where it stopped; details wait until the walk has passed the cursor", false);
    return;
  }
  if (list_out !== "complete") { return; }
  var detail_out = await runSync("detail");
  if (detail_out === "cancelled") {
    set_note(document.getElementById("sync-note"),
      "pull-all stopped by Cancel in the detail stage: later clicks resume the pending queue; icons wait until the records are in", false);
    return;
  }
  if (detail_out !== "complete") { return; }
  var icons_out = await runSync("icons");
  if (icons_out === "cancelled") {
    set_note(document.getElementById("sync-note"),
      "pull-all stopped by Cancel in the icon stage: the next click resumes the icon queue; descriptions wait until the icons are in", false);
    return;
  }
  if (icons_out !== "complete") { return; }
  var tooltip_out = await runSync("tooltip");
  if (tooltip_out === "complete") {
    set_note(document.getElementById("sync-note"),
      "descriptions copied for everything that answered (and checked once where the tooltip answered 404)", false);
    await pull_all_summary();
  }
}

async function pull_all_summary() {
  try {
    var st = await cli_ok(["status"]);
    set_note(document.getElementById("sync-note"),
      "nothing left to pull: " + num(st.items) + " items stored, " + num(st.coverage.detailed) + " with details (" + num(st.coverage.priced) + " of them priced — the rest are simply not on the AH)"
      + (num(st.coverage.missing) > 0 ? ", and " + num(st.coverage.missing) + " the walk listed are checked as gone upstream" : "")
      + ", " + num(st.coverage.icons) + " icons copied into the DB (" + num(st.coverage.icon_missing) + " icon files checked as gone)"
      + ", and " + num(st.coverage.tooltips) + " description copies stored (" + num(st.coverage.tooltip_missing) + " tooltips answered 404, " + num(st.coverage.history) + " price samples stored)"
      + "; future syncs only revalidate stored ETags, and prices stay live", false);
  } catch (problem) {
    show_error(problem);
  }
}

// ---- watch list ------------------------------------------------------------
// The toggle inside a hit row is the whole point of the watch list: mark what is
// worth hunting, then let the sync stage fetch exactly those details.
async function refreshWatched() {
  try {
    watched = await cli_ok(["watch", "list"]);
  } catch (problem) {
    watched = [];
  }
}

async function toggle_watch(button) {
  var id = Number(button.dataset.id);
  var already = watched.indexOf(id) >= 0;
  var verb = "add";
  if (already) { verb = "drop"; }
  try {
    await cli_ok(["watch", verb, String(id)]);
    if (already) {
      watched.splice(watched.indexOf(id), 1);
    } else {
      watched.push(id);
    }
    button.textContent = already ? "watch" : "unwatch";
  } catch (problem) {
    set_note(document.getElementById("search-note"), problem.message, true);
  }
}

async function watchHits() {
  if (last_hits.length === 0) {
    set_note(document.getElementById("search-note"), "search first: there are no hits to watch", true);
    return;
  }
  var argv = ["watch", "add"];
  for (var i = 0; i < last_hits.length; i = i + 1) {
    argv.push(String(last_hits[i].id));
  }
  try {
    await cli_ok(argv);
    await refreshWatched();
    set_note(document.getElementById("search-note"), "watching " + last_hits.length + " of the shown hits — the Details: watch list button fetches their records", false);
  } catch (problem) {
    set_note(document.getElementById("search-note"), problem.message, true);
  }
}

// ---- targeted detail sync --------------------------------------------------
// Explicit ids are chunked by the UI, not by the engine: each call carries its own
// slice, so Cancel between slices cannot re-visit the ids the last slice already
// fetched, and one slice stays far below the 120-request burst.
async function runTargeted(ids, label) {
  if (sync_busy) {
    set_note(document.getElementById("sync-note"), "a crawl is already running — cancel it first", true);
    return;
  }
  if (ids.length === 0) {
    set_note(document.getElementById("sync-note"), "no targets: " + label, true);
    return;
  }
  sync_busy = true;
  cancel_requested = false;
  var slice = 20;
  var done = 0;
  var written = 0;
  var reused = 0;
  var tick = 0;
  try {
    while (done < ids.length) {
      var chunk = ids.slice(done, done + slice);
      set_note(document.getElementById("sync-note"),
        "targeted details (" + label + "): " + done + " of " + ids.length + " visited, chunks of " + slice, false);
      document.getElementById("btn-cancel").classList.add("running");
      tick = start_tick("targeted details (" + label + "): " + done + " of " + ids.length + " visited, chunks of " + slice);
      var report = await cli_ok(["sync", "--stage", "detail", "--ids", chunk.join(",")]);
      clearInterval(tick);
      written = written + report.written;
      reused = reused + report.unchanged;
      done = done + chunk.length;
      if (report.requests === 0) {
        set_note(document.getElementById("sync-note"), "the API made no request this chunk — stopping so we do not hammer it", true);
        break;
      }
      if (cancel_requested) {
        set_note(document.getElementById("sync-note"),
          "cancelled: " + done + " of " + ids.length + " targets visited — the rest stay untouched", false);
        break;
      }
    }
    if (done >= ids.length) {
      set_note(document.getElementById("sync-note"),
        label + " done: " + done + " targets visited, " + written + " fetched, " + reused + " reused by ETag", false);
    }
  } catch (problem) {
    if (tick !== 0) { clearInterval(tick); }
    set_note(document.getElementById("sync-note"), problem.message, true);
  }
  document.getElementById("btn-cancel").classList.remove("running");
  sync_busy = false;
  await refreshStatus();
}

async function runWatchDetails() {
  var ids = await cli_ok(["watch", "list"]);
  await runTargeted(ids, "watch list");
}

async function runHitsDetails() {
  var ids = [];
  for (var i = 0; i < last_hits.length; i = i + 1) {
    ids.push(last_hits[i].id);
  }
  await runTargeted(ids, "shown hits");
}

// ---- item card -------------------------------------------------------------
async function runCard(live) {
  var raw = document.getElementById("card-id").value.trim();
  if (raw === "") {
    set_note(document.getElementById("card-note"), "type an item id first", true);
    return;
  }
  var argv = ["item", raw];
  if (!live) { argv.push("--cached"); }
  try {
    var found = await cli_ok(argv);
    render_card(found);
    var fetched = found.detail_synced_at;
    var line = "";
    if (found.manual_at !== null && found.manual_at !== undefined) {
      line = "the DB created this row on " + when(found.manual_at) + " (reserved id block, ADR 0010): its own columns answer the card, and no request went out even on Re-check online, because upstream has no such id";
    } else if (found.cached) {
      line = "read from the DB only, no request sent; that record was fetched " + when(fetched);
    } else {
      line = "answered HTTP " + found.status + ", reused " + (found.reused ? "from the stored copy" : "a fresh copy") + " — " + when(fetched);
    }
    if (found.icon_b64) {
      line = line + " — icon copied into the DB " + when(found.icon_at);
    } else if (found.icon_missing_at !== null && found.icon_missing_at !== undefined) {
      line = line + " — icon checked as not there upstream " + when(found.icon_missing_at);
    } else if (fetched !== null && fetched !== undefined) {
      line = line + " — icon not fetched yet (the icons stage will copy it)";
    }
    if (found.manual_at === null || found.manual_at === undefined) {
      if (found.tooltip_html) {
        line = line + ", and the description copy is stored (copied " + when(found.tooltip_at) + ")";
      } else if (found.tooltip_missing_at !== null && found.tooltip_missing_at !== undefined) {
        line = line + ", and the tooltip answered 404 for this record — checked as having no description upstream, never asked again";
      } else if (fetched !== null && fetched !== undefined) {
        line = line + ", and no description copy yet (the tooltip stage will ask once)";
      }
    }
    // v0.9.6 (ADR 0017): the live row's sentence, in the user's own words. The
    // refresh stage is CLI-side (the panel has no button for it, gui.rs verified)
    // - the card shows what the stage wrote WHEN the CLI ran it, and says so.
    if (found.price_min_now !== null && found.price_min_now !== undefined) {
      line = line + " — live min buyout refreshed by sync --stage price through the tooltip door (it carries a live AH price); it can differ from the stored min above, which ages with the copy";
    } else if (found.tooltip_at !== null && found.tooltip_at !== undefined) {
      line = line + " — no live price yet: sync --stage price asks this copied record once (it may also CHECK the record as price-less — the door's own third state, the DB's stamps keep that apart)";
    }
    if (found.history && found.history.length > 1) {
      line = line + ", and the price has " + found.history.length + " samples (trend drawn on the card, each bar's hover carries that sample's money)";
    } else if (found.history && found.history.length === 1) {
      line = line + ", and one price sample so far — a later fresh write adds the next point";
    }
    set_note(document.getElementById("card-note"), line, false);
  } catch (problem) {
    if (problem.kind === "not-found") {
      set_note(document.getElementById("card-note"),
        problem.message + " — 'Re-check online' fetches and stores it", false);
    } else if (problem.message.indexOf("tombstoned") >= 0) {
      set_note(document.getElementById("card-note"),
        problem.message + " — 'Bring it back' in the write panel restores it", false);
    } else {
      set_note(document.getElementById("card-note"), problem.message, true);
    }
  }
}

function when(stamp) {
  if (stamp === null || stamp === undefined) { return "never"; }
  return new Date(Number(stamp) * 1000).toLocaleString();
}

function render_card(found) {
  var item = found.item;
  // The fields the USER stated (ADR 0010): the card carries the stored override
  // ARRAY, so the mark comes from the DB's own record, not from what the UI typed.
  var said = {};
  if (found.manual_overrides) {
    for (var o = 0; o < found.manual_overrides.length; o = o + 1) {
      said[String(found.manual_overrides[o].field)] = found.manual_overrides[o].at;
    }
  }
  var rows = [];
  rows.push(["id", String(item.id), ""]);
  rows.push(["name", item.name, "name"]);
  rows.push(["quality", item.quality, "quality"]);
  rows.push(["item level", item.item_level, "item_level"]);
  rows.push(["requires level", item.required_level, "required_level"]);
  rows.push(["type", item.item_type, "item_type"]);
  rows.push(["subtype", item.item_subtype, "item_subtype"]);
  rows.push(["slot", item.slot, ""]);
  // v0.9.4 (ADR 0015): the money words ride these THREE rows as a 4th element -
  // they come from Rust beside the very numbers printed here, and the row shows
  // money with the exact copper in the hover. `available` gets NO money: it counts
  // auctions (measured, row 25 answers `available 12` for 12 offers of 15 000
  // copper), and money never rides a count.
  rows.push(["min price", item.price ? item.price.min : null, "", found.price_min_money]);
  rows.push(["market price", item.price ? item.price.market : null, "", found.price_market_money]);
  rows.push(["available", item.price ? item.price.available : null, ""]);
  rows.push(["vendor sell", item.vendor_sell, "vendor_sell", found.vendor_sell_money]);
  // v0.9.6 (ADR 0017): the LIVE observation beside the AGED stored rows above.
  // price_min_now is what sync --stage price saw when it re-asked the tooltip
  // door (the door carries a live Auction House price - docs/API.md measures
  // fields.price and upstream's own developers page says tooltips cache ~5 min
  // "because they carry a live Auction House price"); the stored min above is
  // what the copy said when the detail crawl fetched it. The two DIFFER,
  // measured on the mech run: stored row 25 kept min 17190 while the live row
  // read 16190 - money_cell rides the live copper like every other money row
  // (hover: the exact copper, ADR 0015). A NULL here means "never refreshed OR
  // the door answered that the AH offers NOTHING right now" (measured third
  // state, live id 200): the card door does not split those two yet - its
  // stamps pair does, on disk - so money_cell's muted "no price" plus the
  // note's sentence are the honest reading, and the row label says which
  // stage fills it. The label matters: the panel has no button for this
  // stage (gui.rs builds the window and draws no card, reads no column;
  // verified before this claim), so a value only the CLI stage wrote shows
  // up when the CLI ran - the card never pretends the panel did anything.
  rows.push(["live min buyout", found.price_min_now, "", found.price_min_now_money]);
  if (item.weapon) {
    rows.push(["damage", item.weapon.damage_min + "-" + item.weapon.damage_max, ""]);
    rows.push(["dps / speed", item.weapon.dps + " / " + item.weapon.speed, ""]);
  }
  if (item.flavor) { rows.push(["flavour", item.flavor, "flavor"]); }
  // stats is the ONE previously-unshaped field the live catalogue has ever filled
  // (docs/API.md measures: across 47 answered records stats carried values while
  // sockets/socket_bonus/classes/effects came back empty on every single one) -
  // so it renders as name/value pairs, and those others get an honest RAW row
  // only when they actually carry something. Inventing a nicer shape for never-
  // measured fields is not our call to make; printing what came back is.
  if (item.stats && item.stats.length > 0) {
    var stat_text = [];
    for (var s = 0; s < item.stats.length; s = s + 1) {
      var one = item.stats[s];
      stat_text.push(String(one.name) + " " + String(one.value));
    }
    rows.push(["stats", stat_text.join(" \u00b7 "), ""]);
  }
  if (item.socket_bonus) { rows.push(["socket bonus", String(item.socket_bonus), ""]); }
  if (item.sockets && item.sockets.length > 0) { rows.push(["sockets", JSON.stringify(item.sockets), ""]); }
  if (item.classes && item.classes.length > 0) { rows.push(["classes", JSON.stringify(item.classes), ""]); }
  if (item.effects && item.effects.length > 0) { rows.push(["effects", JSON.stringify(item.effects), ""]); }
  var html = "";
  for (var i = 0; i < rows.length; i = i + 1) {
    var value = rows[i][1];
    if (value === null || value === undefined) { value = ""; }
    // esc() again, always: these are API texts. The RAW rows (sockets/classes/
    // effects) print the JSON exactly as it came back - still escaped, because
    // API text is markup-neutral by rule, not by hope.
    var mark = "";
    var key = String(rows[i][2]);
    if (said[key] !== undefined) {
      mark = " <i>the user said this on " + esc(when(said[key])) + "</i>";
    }
    // A money row (the 4th element) shows the game's words and keeps the copper in
    // the hover; every other row prints its value as before. money_cell escapes
    // both parts, so the money text from Rust lands as text, not as markup.
    var body = esc(String(value));
    if (rows[i].length > 3) { body = money_cell(rows[i][3], rows[i][1]); }
    html = html + '<div class="cardrow"><span>' + esc(rows[i][0]) + "</span><b>" + body + mark + "</b></div>";
  }
  // A row the user MADE (ADR 0010): the DB's own columns answer its card, and the
  // card says so - the fields only an upstream record can state read empty above.
  if (found.manual_at !== null && found.manual_at !== undefined) {
    html = html + '<div class="cardrow"><span>made by the user</span><b>the DB created this row on ' + esc(when(found.manual_at)) + ", in the reserved id block: no upstream record of it exists, and no crawl will ask for one</b></div>";
  }
  // The local icon copy (v0.6.0): bytes travel in the verb JSON as base64 (b64_text
  // on the Rust side), and a data URI is how offline browsing keeps icons - the card
  // NEVER hotlinks icon_url (docs/API.md grounds both hosts serving image/jpeg; the
  // data URI says so). No copy, no img tag: three states, one row each.
  if (found.icon_b64) {
    html = html + '<div class="cardrow"><span>icon (local copy)</span><b><img class="iconimg" src="data:image/jpeg;base64,' + found.icon_b64 + '" alt="item icon"></b></div>';
  } else if (found.icon_missing_at !== null && found.icon_missing_at !== undefined) {
    html = html + '<div class="cardrow"><span>icon</span><b>no icon upstream (checked ' + esc(when(found.icon_missing_at)) + ")</b></div>";
  }
  // The READABLE twin (v0.9.0, ADR 0012): derived plain text of the copy, hotlink
  // img tags dropped - a card CAN render it (the measured upstream markup hotlinks
  // render.worldofwarcraft.com and thewowdb.com/assets pixels, which is exactly
  // what never happens). Escaped like everything from a DB column anyway. A row
  // migrated from a schema-4 DB has the copy but no twin yet: honest gap below.
  if (found.tooltip_text) {
    html = html + '<div class="cardrow"><span>description (readable copy)</span><b>' + esc(String(found.tooltip_text)) + "</b></div>";
  }
  // The description copy (v0.8.0, ADR 0010): upstream's own markup, printed as
  // TEXT and escaped - pasting it as markup would let an upstream body write the
  // card. Three states, one row each, like the icon's.
  if (found.tooltip_html) {
    html = html + '<div class="cardrow"><span>description (upstream\u2019s own copy)</span><b>' + esc(String(found.tooltip_html)) + (said["tooltip_html"] !== undefined ? " <i>the user said this on " + esc(when(said["tooltip_html"])) + "</i>" : "") + "</b></div>";
  } else if (found.tooltip_missing_at !== null && found.tooltip_missing_at !== undefined) {
    html = html + '<div class="cardrow"><span>description</span><b>none upstream (the tooltip answered 404, checked ' + esc(when(found.tooltip_missing_at)) + ", never asked again)</b></div>";
  } else if (found.manual_at === null || found.manual_at === undefined) {
    html = html + '<div class="cardrow"><span>description</span><b>not copied yet — the tooltip stage fetches it once per record that answered</b></div>';
  }
  // The price trend: one bar per sample, oldest first (history_of reads newest-40
  // reversed). Heights are a percentage of the window's max price_min; a one-sample
  // window shows one bar: the sample EXISTS, the trend just has no slope yet. v0.9.4
  // (ADR 0015): a bar carries the sample's money plus its exact copper in the hover
  // - money reaches EVERY copper number, and the samples' money came along in the
  // same fragments Rust filled (PriceSample.price_min_money).
  if (found.history && found.history.length > 0) {
    var maxp = 1;
    for (var h = 0; h < found.history.length; h = h + 1) {
      var point = found.history[h];
      if (point.price_min !== null && point.price_min > maxp) { maxp = point.price_min; }
    }
    var bars = "";
    for (var g = 0; g < found.history.length; g = g + 1) {
      var sample = found.history[g];
      if (sample.price_min === null) { continue; }
      var pct = Math.max(8, Math.round(100 * sample.price_min / maxp));
      bars = bars + '<div class="trendcol" title="' + esc(String(sample.price_min_money) + " \u00b7 " + num(sample.price_min) + " copper") + '" style="height:' + pct + '%"></div>';
    }
    html = html + '<div class="cardrow"><span>price trend</span><b><div class="trendbox">' + bars + "</div></b></div>";
  }
  document.getElementById("card-box").innerHTML = html;
}

// ---- the DB's own writes (v0.8.0, ADR 0010) ---------------------------------
// Four verbs, no network: each button sends ONE argv in the exact shape the headless
// CLI takes, so a click and a `cargo run --` line share the code path down to the
// message (docs/DECISIONS/0004): a bad ask answers usage(2), a row that is not there
// answers database(4), and both land here as the same words the CLI prints. set-item
// states ONE field per ask through db.rs's whitelist; add-item takes NO id - the
// reserved block's next free slot is the DB's to choose, so a new item cannot collide
// with a future forever record (measured max upstream id: 288 340, block at 900 000
// 000). Hide it is the tombstone: the row keeps its stamps and its bytes, which is
// why Bring it back restores everything. Every write refreshes the status grid, so
// the manual-layer counts move on the same breath the user wrote them.
async function runEdit() {
  var note = document.getElementById("edit-note");
  var raw = document.getElementById("edit-id").value.trim();
  var field = document.getElementById("edit-field").value;
  var value = document.getElementById("edit-value").value;
  if (raw === "") {
    set_note(note, "type an item id first (only rows the DB already has can be edited)", true);
    return;
  }
  if (field === "") {
    set_note(note, "pick the field the user states: name, quality, item level, requires level, type, subtype, flavour, vendor sell, description copy", true);
    return;
  }
  if (value === "") {
    set_note(note, "an edit states a value; to hide a row use Hide it", true);
    return;
  }
  try {
    var answer = await cli_ok(["set-item", raw, field + "=" + value]);
    set_note(note,
      "the DB says: item " + answer.id + "'s " + answer.field + " is now " + answer.value + " (stated " + when(answer.edited_at) + ") — the crawl keeps refreshing the fields it owns, this one survives the next write of the row, and the card marks it", false);
    await refreshStatus();
  } catch (problem) {
    set_note(note, problem.message, true);
  }
}

async function runCreate() {
  var note = document.getElementById("edit-note");
  var name = document.getElementById("new-name").value.trim();
  if (name === "") {
    set_note(note, "a new item needs a name: the DB refuses to create a row without one", true);
    return;
  }
  var argv = ["add-item", "name=" + name];
  // one pair per argv element, so a value holding spaces stays ONE word
  var quality = document.getElementById("new-quality").value;
  if (quality !== "") { argv.push("quality=" + quality); }
  var type = document.getElementById("new-type").value.trim();
  if (type !== "") { argv.push("item_type=" + type); }
  var level = document.getElementById("new-level").value.trim();
  if (level !== "") { argv.push("item_level=" + level); }
  var req = document.getElementById("new-req").value.trim();
  if (req !== "") { argv.push("required_level=" + req); }
  var sell = document.getElementById("new-sell").value.trim();
  if (sell !== "") { argv.push("vendor_sell=" + sell); }
  try {
    var answer = await cli_ok(argv);
    var made = [];
    for (var i = 0; i < answer.fields.length; i = i + 1) {
      made.push(String(answer.fields[i]));
    }
    set_note(note,
      "made as item " + answer.id + " (the reserved block's next free slot, stamped manual_at " + when(answer.manual_at) + "): it wears " + made.join(", ") + " as the user's own overrides from birth, so even a future forever record cannot take it — and no crawl will ever ask upstream for this id", false);
    await refreshStatus();
  } catch (problem) {
    set_note(note, problem.message, true);
  }
}

async function runHide() {
  var note = document.getElementById("edit-note");
  var raw = document.getElementById("edit-id").value.trim();
  if (raw === "") {
    set_note(note, "type an item id first", true);
    return;
  }
  try {
    var answer = await cli_ok(["delete-item", raw]);
    set_note(note,
      "hidden: item " + answer.deleted[0] + " is tombstoned (stamped " + when(answer.at) + ") — search, stats, the crawl's queues and both card doors refuse it, and its icon bytes, price samples and overrides stay in the DB until Bring it back puts them to work again", false);
    await refreshStatus();
  } catch (problem) {
    set_note(note, problem.message, true);
  }
}

async function runUnhide() {
  var note = document.getElementById("edit-note");
  var raw = document.getElementById("edit-id").value.trim();
  if (raw === "") {
    set_note(note, "type an item id first", true);
    return;
  }
  try {
    var answer = await cli_ok(["undelete-item", raw]);
    set_note(note,
      "back: item " + answer.restored[0] + " answers again — the tombstone only hid it, so the name, the icon copy, the samples and the user's overrides all came back", false);
    await refreshStatus();
  } catch (problem) {
    set_note(note, problem.message, true);
  }
}

// ---- the twin refresh (v0.9.2, ADR 0013) ------------------------------------
// ONE ask refreshes EVERY copy in the DB, and the verb makes ZERO requests: the
// twin is derived from the copy the DB already holds, so this panel re-derives
// instead of re-fetching. Tombstones and manual rows ride (ADR 0012's rule: the
// twin always matches what the DB SAYS). A refreshed DB answers 0 rewritten.
async function runTwinRefresh() {
  var note = document.getElementById("twin-note");
  try {
    var answer = await cli_ok(["refresh-twins"]);
    set_note(note,
      "reworded: " + answer.rewritten + " of " + answer.copies + " copies got their twin re-derived from the copy they already hold (" +
      answer.unchanged + " already matched, " + answer.filled + " had no twin at birth and got one now) — ZERO requests left here, " +
      "nothing was re-fetched, and the card's words row plus the description search read what it wrote", false);
    await refreshStatus();
  } catch (problem) {
    set_note(note, problem.message, true);
  }
}

// ---- price analytics -------------------------------------------------------
async function runStats() {
  try {
    var found = await cli_ok(["stats", "--icons", "--limit", "6"]);
    var byq = found.by_quality;
    var biggest = 1;
    for (var i = 0; i < byq.length; i = i + 1) {
      if (byq[i].items > biggest) { biggest = byq[i].items; }
    }
    var bars = "";
    for (var j = 0; j < byq.length; j = j + 1) {
      var row = byq[j];
      var label = row.quality || "unknown";
      var pct = Math.round(100 * row.items / biggest);
      var avg = money_cell(row.avg_price_money, row.avg_price);
      bars = bars + '<div class="barrow"><span class="chip c-' + esc(label) + '">' + esc(label) + "</span>"
        + '<div class="bar c-' + esc(label) + '" style="width:' + pct + '%"></div>'
        + "<b>" + num(row.items) + " items · " + num(row.priced) + " priced · avg " + (row.avg_price_money === null ? "no price" : avg) + "</b></div>";
    }
    document.getElementById("stats-bars").innerHTML = bars;
    document.querySelector("#stats-cheap tbody").innerHTML = extreme_rows(found.cheapest);
    document.querySelector("#stats-rich tbody").innerHTML = extreme_rows(found.priciest);
    var cov = found.coverage;
    set_note(document.getElementById("stats-note"),
      "one read answered " + num(cov.items) + " items, " + num(cov.priced) + " priced, " + num(cov.detailed) + " with details; the price spread is "
      + (num(cov.min_price) === null ? "unknown" : cov.min_price_money + " .. " + cov.max_price_money + " (" + num(cov.min_price) + " .. " + num(cov.max_price) + " copper)")
      + " — the icon column is the DB's own copy where the icons stage wrote one (\u25c7: no bytes)", false);
    document.getElementById("btn-stats").classList.remove("running");
  } catch (problem) {
    set_note(document.getElementById("stats-note"), problem.message, true);
  }
}

function extreme_rows(rows) {
  var html = "";
  for (var i = 0; i < rows.length; i = i + 1) {
    var one = rows[i];
    var quality = one.quality || "unknown";
    html = html + '<tr><td class="namecell">' + esc(one.name) + '<span class="chip c-' + esc(quality) + '">' + esc(quality) + "</span></td>"
      + '<td class="iconcell">' + icon_cell(one) + "</td>"
      + '<td class="pricecell">' + money_cell(one.price_min_money, one.price_min) + "</td></tr>";
  }
  return html;
}

// ---- live search card ------------------------------------------------------
// One click, ONE request: the door is a look, not a sync (ADR 0009), so this card
// never writes the DB and never chases a group's `more`. The answer rides through
// as upstream's own JSON; the card renders the groups it knows about and prints
// `remaining` because the burst header is part of the door's answer (measured:
// `x-ratelimit-remaining: 119`). Names-only is a fact of the endpoint, so the card
// says it: a tooltip word is not searched (the measured `?q=socket` answers empty
// groups), and an empty answer is still a SUCCESS - exit 0, not a failure.
async function runLive() {
  var text = document.getElementById("live-text").value;
  document.getElementById("btn-live").classList.add("running");
  if (text === "") {
    set_note(document.getElementById("live-note"), "an empty ask is a usage error: the door wants text, for example: live-search Worn Shortsword", true);
    document.getElementById("btn-live").classList.remove("running");
    return;
  }
  try {
    var found = await cli_ok(["live-search", text]);
    var answer = found.answer;
    var groups = answer.groups;
    var order = ["item", "spell", "quest", "npc", "zone", "sound"];
    var rows = "";
    var seen = 0;
    for (var g = 0; g < order.length; g = g + 1) {
      var kind = order[g];
      if (!(kind in groups)) { continue; }
      var bucket = groups[kind];
      var hits = bucket.results;
      for (var i = 0; i < hits.length; i = i + 1) {
        var one = hits[i];
        var label = one.name || "(unnamed)";
        var link = "";
        if (one.links && one.links.html) {
          link = '<a class="source" href="' + esc(one.links.html) + '">view</a>';
        }
        seen = seen + 1;
        rows = rows + "<tr><td class=\"namecell\">" + esc(kind) + '</td><td class="namecell">' + esc(label) + " " + link + "</td></tr>";
      }
    }
    document.querySelector("#live-hits tbody").innerHTML = rows;
    var left = "the API did not report a burst budget for this answer";
    if (found.remaining !== null && found.remaining !== undefined) { left = "requests left in the current burst window: " + found.remaining; }
    var more_note = "";
    for (var k = 0; k < order.length; k = k + 1) {
      if ((order[k] in groups) && groups[order[k]].has_more) { more_note = " and at least one group answered has_more: this build stops at one ask, it does not chase the paged more URL"; }
    }
    var note = "";
    if (seen === 0) { note = "no group answered a hit - and empty groups are still a success: "; }
    note = note + left + ", names only (a tooltip word is not searched)" + more_note + " - the hits' own upstream page: " + hits_link;
    set_note(document.getElementById("live-note"), note, false);
    document.getElementById("btn-live").classList.remove("running");
  } catch (problem) {
    set_note(document.getElementById("live-note"), problem.message, true);
    document.getElementById("btn-live").classList.remove("running");
  }
}

// ---- wiring ----------------------------------------------------------------
document.getElementById("btn-refresh").addEventListener("click", function () { refreshStatus(); });
document.getElementById("btn-search").addEventListener("click", function () { runSearch(); });
document.getElementById("btn-watch-hits").addEventListener("click", function () { watchHits(); });
document.getElementById("btn-list").addEventListener("click", function () { runSync("list"); });
document.getElementById("btn-pull").addEventListener("click", function () { runPullAll(); });
document.getElementById("btn-detail-watch").addEventListener("click", function () { runWatchDetails(); });
document.getElementById("btn-detail-hits").addEventListener("click", function () { runHitsDetails(); });
document.getElementById("btn-cancel").addEventListener("click", requestCancel);
document.getElementById("btn-card-db").addEventListener("click", function () { runCard(false); });
document.getElementById("btn-card-live").addEventListener("click", function () { runCard(true); });
document.getElementById("btn-live").addEventListener("click", function () { runLive(); });
document.getElementById("btn-stats").addEventListener("click", function () { runStats(); });
document.getElementById("btn-edit").addEventListener("click", function () { runEdit(); });
document.getElementById("btn-create").addEventListener("click", function () { runCreate(); });
document.getElementById("btn-delete").addEventListener("click", function () { runHide(); });
document.getElementById("btn-undelete").addEventListener("click", function () { runUnhide(); });
document.getElementById("btn-twin-refresh").addEventListener("click", function () { runTwinRefresh(); });

// One delegated handler for the whole hits table: the toggle button carries the
// item id it belongs to, so rows do not need per-row listeners.
document.getElementById("hits").addEventListener("click", function (event) {
  var node = event.target;
  if (node.dataset.id !== undefined) { toggle_watch(node); }
});



// The cards are never empty: at startup the app reads its own DB (status, the watch
// list, the analytics) so the first paint already shows what exists. Those are verb
// calls, which is what makes boot() the witness of the launch audit: the Rust side
// logs every verb that reaches it.
async function boot() {
  await refreshStatus();
  await refreshWatched();
  await runStats();
  // A second status read after the analytics: it keeps the status card honest, and in
  // the launch audit it is the witness that runStats' rendering code ran to the end -
  // a JS throw after the awaited verb would leave only one `status` log line.
  await refreshStatus();
}

boot();

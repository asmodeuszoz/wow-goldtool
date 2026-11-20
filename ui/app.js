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
    if (num(status.coverage.min_price) !== null) {
      grid.push(["price range", num(status.coverage.min_price) + " .. " + num(status.coverage.max_price)]);
    }
    if (num(status.coverage.avg_price) !== null) {
      grid.push(["average price", num(Math.round(status.coverage.avg_price))]);
    }
    grid.push(["list crawl", fmt_state(status.list)]);
    grid.push(["detail crawl", fmt_state(status.detail)]);
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
  var argv = ["search"];
  if (text !== "") { argv.push(text); }
  if (quality !== "") { argv.push("--quality", quality); }
  argv.push("--sort", sort);
  if (priced) { argv.push("--priced"); }
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
    var note = "showing " + shown + " of " + num(found.total) + " — details of " + num(found.total) + " items cost one request each, and the sync stage caches them";
    if (found.total === 0) { note = "no item matches — the search runs against the local DB only"; }
    set_note(document.getElementById("search-note"), note, false);
    document.getElementById("btn-search").classList.remove("running");
  } catch (problem) {
    set_note(document.getElementById("search-note"), problem.message, true);
  }
}

function hit_row(hit) {
  var quality = hit.quality || "unknown";
  var link = "";
  if (hit.html_url) {
    link = '<a class="source" href="' + esc(hit.html_url) + '">view</a>';
  }
  var price = num(hit.price_min);
  var price_cell = '<span class="muted">no price</span>';
  if (price !== null) { price_cell = esc(price); }
  var watch_label = "watch";
  if (watched.indexOf(hit.id) >= 0) { watch_label = "unwatch"; }
  var toggle = '<button class="rowact" data-id="' + esc(String(hit.id)) + '">' + watch_label + "</button>";
  return "<tr>"
    + '<td class="col-id">' + esc(String(hit.id)) + "</td>"
    + '<td class="namecell">' + esc(hit.name) + " " + link + " " + toggle + "</td>"
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
    "pull-all: list stage to the end (~193 pages, minutes at 2/s), then details for everything never fetched (~19k records, hours at 2/s) — Cancel stops at the next chunk boundary", false);
  var list_out = await runSync("list");
  if (list_out === "cancelled") {
    set_note(document.getElementById("sync-note"),
      "pull-all stopped by Cancel in the list stage: the next click resumes the list where it stopped; details wait until the walk has passed the cursor", false);
    return;
  }
  if (list_out !== "complete") { return; }
  var detail_out = await runSync("detail");
  if (detail_out === "complete") {
    set_note(document.getElementById("sync-note"),
      "details pulled for everything the walk ever listed", false);
    await pull_all_summary();
  }
}

async function pull_all_summary() {
  try {
    var st = await cli_ok(["status"]);
    set_note(document.getElementById("sync-note"),
      "nothing left to pull: " + num(st.items) + " items stored, " + num(st.coverage.detailed) + " with details (" + num(st.coverage.priced) + " of them priced — the rest are simply not on the AH); future syncs only revalidate stored ETags, and prices stay live", false);
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
    if (found.cached) {
      line = "read from the DB only, no request sent; that record was fetched " + when(fetched);
    } else {
      line = "answered HTTP " + found.status + ", reused " + (found.reused ? "from the stored copy" : "a fresh copy") + " — " + when(fetched);
    }
    set_note(document.getElementById("card-note"), line, false);
  } catch (problem) {
    if (problem.kind === "not-found") {
      set_note(document.getElementById("card-note"),
        problem.message + " — 'Re-check online' fetches and stores it", false);
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
  var rows = [];
  rows.push(["id", String(item.id)]);
  rows.push(["name", item.name]);
  rows.push(["quality", item.quality]);
  rows.push(["item level", item.item_level]);
  rows.push(["requires level", item.required_level]);
  rows.push(["type", item.item_type]);
  rows.push(["slot", item.slot]);
  rows.push(["min price", item.price ? item.price.min : null]);
  rows.push(["market price", item.price ? item.price.market : null]);
  rows.push(["available", item.price ? item.price.available : null]);
  rows.push(["vendor sell", item.vendor_sell]);
  if (item.weapon) {
    rows.push(["damage", item.weapon.damage_min + "-" + item.weapon.damage_max]);
    rows.push(["dps / speed", item.weapon.dps + " / " + item.weapon.speed]);
  }
  if (item.flavor) { rows.push(["flavour", item.flavor]); }
  var html = "";
  for (var i = 0; i < rows.length; i = i + 1) {
    var value = rows[i][1];
    if (value === null || value === undefined) { value = ""; }
    html = html + '<div class="cardrow"><span>' + esc(rows[i][0]) + "</span><b>" + esc(String(value)) + "</b></div>";
  }
  document.getElementById("card-box").innerHTML = html;
}

// ---- price analytics -------------------------------------------------------
async function runStats() {
  try {
    var found = await cli_ok(["stats", "--limit", "6"]);
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
      var avg = num(row.avg_price === null ? null : Math.round(row.avg_price));
      bars = bars + '<div class="barrow"><span class="chip c-' + esc(label) + '">' + esc(label) + "</span>"
        + '<div class="bar c-' + esc(label) + '" style="width:' + pct + '%"></div>'
        + "<b>" + num(row.items) + " items · " + num(row.priced) + " priced · avg " + (avg === null ? "no price" : avg) + "</b></div>";
    }
    document.getElementById("stats-bars").innerHTML = bars;
    document.querySelector("#stats-cheap tbody").innerHTML = extreme_rows(found.cheapest);
    document.querySelector("#stats-rich tbody").innerHTML = extreme_rows(found.priciest);
    var cov = found.coverage;
    set_note(document.getElementById("stats-note"),
      "one read answered " + num(cov.items) + " items, " + num(cov.priced) + " priced, " + num(cov.detailed) + " with details; the price spread is "
      + (num(cov.min_price) === null ? "unknown" : num(cov.min_price) + " .. " + num(cov.max_price)), false);
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
    var price = num(one.price_min);
    if (price === null) { price = "no price"; }
    html = html + '<tr><td class="namecell">' + esc(one.name) + '<span class="chip c-' + esc(quality) + '">' + esc(quality) + "</span></td>"
      + '<td class="pricecell">' + esc(String(price)) + "</td></tr>";
  }
  return html;
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
document.getElementById("btn-stats").addEventListener("click", function () { runStats(); });

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

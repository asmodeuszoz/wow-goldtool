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
  return "<tr>"
    + '<td class="col-id">' + esc(String(hit.id)) + "</td>"
    + '<td class="namecell">' + esc(hit.name) + " " + link + "</td>"
    + '<td class="qualitycell"><span class="chip c-' + esc(quality) + '">' + esc(quality) + "</span></td>"
    + '<td class="pricecell">' + price_cell + "</td>"
    + "</tr>";
}

// ---- sync card -------------------------------------------------------------
async function runSync(stage) {
  if (sync_busy) {
    set_note(document.getElementById("sync-note"), "a crawl is already running — cancel it first", true);
    return;
  }
  sync_busy = true;
  cancel_requested = false;
  var chunk = stage === "list" ? 10 : 25;
  var seen = 0;
  try {
    while (true) {
      set_note(document.getElementById("sync-note"),
        "running " + stage + ": " + seen + " requests spent, chunks of " + chunk + " — cancel stops at the next chunk", false);
      document.getElementById("btn-cancel").classList.add("running");
      var report = await cli_ok(["sync", "--stage", stage, "--budget", String(chunk)]);
      seen = seen + report.requests;
      if (report.requests === 0) {
        set_note(document.getElementById("sync-note"),
          "the crawl made no progress this chunk — stopping so we do not burn the API", true);
        break;
      }
      if (report.complete) {
        set_note(document.getElementById("sync-note"),
          stage + " stage complete: " + num(report.items) + " seen, " + num(report.written) + " written, " + num(report.unchanged) + " reused", false);
        break;
      }
      if (cancel_requested) {
        set_note(document.getElementById("sync-note"),
          "cancelled: " + num(report.items) + " seen so far — the next run resumes from here", false);
        break;
      }
    }
  } catch (problem) {
    var extra = problem.retry_after ? " (the API asked to wait " + problem.retry_after + "s; a later chunk will retry)" : "";
    set_note(document.getElementById("sync-note"), problem.message + extra, true);
  }
  document.getElementById("btn-cancel").classList.remove("running");
  sync_busy = false;
  await refreshStatus();
}

function requestCancel() {
  cancel_requested = true;
  if (!sync_busy) {
    set_note(document.getElementById("sync-note"),
      "cancel asked for while idle — the next chunk loop stops before it starts", false);
  }
}

// ---- wiring ----------------------------------------------------------------
function bind(button, handler) {
  button.addEventListener("click", handler);
}

async function boot() {
  await refreshStatus();
}

document.getElementById("btn-refresh").addEventListener("click", function () { refreshStatus(); });
document.getElementById("btn-search").addEventListener("click", function () { runSearch(); });
document.getElementById("btn-list").addEventListener("click", function () { runSync("list"); });
document.getElementById("btn-detail").addEventListener("click", function () { runSync("detail"); });
document.getElementById("btn-cancel").addEventListener("click", requestCancel);

boot();

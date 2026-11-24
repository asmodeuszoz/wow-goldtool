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
// v0.10.0 (ADR 0021): the recipes tab's shown rows, the same shape last_hits is
// for the search tab: btn-recipes-watch watches the COOKED items of THESE rows.
var recipes_shown = [];

// v0.11.0 (ADR 0023): did ANY shown row carry live noise beside its stored money?
// Drives the ONE conditional note sentence (the ADR 0020 pattern: the note says
// cell = STORED min vs live = last ask, no numbers — numbers ride the cells).
var live_shown = false;

// v0.9.7 (ADR 0018): the one crammed page became seven activity tabs. The switch
// is ONE truth here: exactly one section with a data-tab is visible, the rest wear
// tab-off (display:none in app.css). Buttons inside a hidden section keep their
// listeners - a hidden button cannot be clicked (display:none gets no events), so
// "one activity at a time" is enforced by the layout, not by disabling handlers.
// Nothing in here touches a verb: every still-visible button lands in the SAME
// cli_ok(argv) path ADR 0014 locked (a click and a cargo line stay one code path).
// v0.15.0 (ADR 0029): the TWO CARD WINDOWS are NOT in this truth any more - their
// sections dropped data-tab with their nameplates, so this loop sees SEVEN
// nameplates and EIGHT data-tab sections, never a card window. A window's
// visibility is held by open_pip/close_pip alone (below): it floats OVER whatever
// tab is already visible, and the tab machinery never hides or moves it.
var active_tab = "overview";

function show_tab(tab) {
  active_tab = tab;
  var sections = document.querySelectorAll("section[data-tab]");
  for (var i = 0; i < sections.length; i = i + 1) {
    sections[i].classList.remove("tab-off");
    if (sections[i].dataset.tab !== active_tab) { sections[i].classList.add("tab-off"); }
  }
  var tabs = document.querySelectorAll("button.tab[data-tab]");
  for (var j = 0; j < tabs.length; j = j + 1) {
    tabs[j].classList.remove("tab-on");
    if (tabs[j].dataset.tab === active_tab) { tabs[j].classList.add("tab-on"); }
  }
}

function tab_click(button) {
  if (button.dataset.tab !== active_tab) { show_tab(button.dataset.tab); }
}

// v0.15.0 (ADR 0029): the card windows in window - a window OVER the tab its
// click came from, never an activity of its own. These two class swaps are the
// ONLY thing that shows or hides a window: pip-closed hides it (the frame's
// SAME display:none mechanism tab-off uses, under a name that is not a tab),
// pip floats it (position + trim in app.css). Closing puts the window back
// where it was born: hidden, and the tab underneath stays exactly where it was.
function open_pip(section_id) {
  var s = document.getElementById(section_id);
  s.classList.remove("pip-closed");
  s.classList.add("pip");
}

function close_pip(section_id) {
  var s = document.getElementById(section_id);
  s.classList.remove("pip");
  s.classList.add("pip-closed");
}

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
    var note = "showing " + shown + " of " + num(found.total) + " — details of " + num(found.total) + " items cost one request each, and the sync stage caches them; the price column shows the game's OWN money words (hover: the exact copper, the number the sorts and the scripts read), the icon column shows the DB's OWN copy where the icons stage wrote one (\u25c7 means no bytes: never fetched, or checked as gone upstream \u2014 the hover says which, and nothing is hotlinked) \u2014 each row's OWN NAME is its window opener (click it like the Recipes tab's names open theirs): it opens that item's window OVER this tab (the window reads the SAME door a `cargo run -- item ID --cached` line takes, ZERO requests; its `Close this window` button closes it and this tab stays where it was)";
    if (found.total === 0) { note = "no item matches — the search runs against the local DB only"; }
    // v0.9.9 (ADR 0020): the live-sort knob reorders rows by the LIVE min buyout, but the
    // price CELL stays the STORED min - the note names which is which beside every hit, so
    // an out-of-looking-order column is never a mystery (money rules ADR 0015 untouched:
    // this sentence carries no numbers at all).
    if (sort === "price-now") {
      note = note + " — this order reads the LIVE min buyout the Sync tab's Price button (or the CLI stage) wrote, one ask per copied record ever; the price column stays the DB's STORED min, and rows with no live price sort last (never refreshed, or checked price-less at ask time — their own stamps keep the two kinds apart)";
    }
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
  // v0.15.0 (ADR 0029) drew the opener as a separate `card` chip; v0.16.0
  // (ADR 0030) moved the AFFORDANCE onto the name instead: the NAME cell IS the
  // opener, exactly the shape the recipe rows already rode (ADR 0025 drew their
  // name cell as `<button class="openrecipe">` - "the name cell IS the click
  // target", now over BOTH tables). The class, the id the button carries, the
  // delegated handler's split BY CLASS and the `item ID --cached` door all stay
  // PUT word-for-word - the affordance moved, the verb did not (ADR 0014's ONE
  // code path). The name is never blank where a name is clickable: the recipe
  // cell's `|| spell_id` fallback shape rides here as `|| hit.id` too.
  var name_btn = '<button class="pipbtn" data-id="' + esc(String(hit.id)) + '" type="button">' + esc(hit.name || hit.id) + "</button>";
  // v0.17.0 (ADR 0031): the icon cell now rides BETWEEN the id and the name -
  // the thead's `col-icon` moved beside it in the SAME commit, the cells and
  // their classes ride PUT word-for-word (the frame's th/td looks are class-
  // keyed, never positional - the census that read 227's header rule before
  // one line moved). The click verb, its button and the `item ID --cached`
  // door underneath ride PUT (ADR 0014's ONE code path: the affordance sits
  // in the SAME namecell, only the columns around it changed order).
  return "<tr>"
    + '<td class="col-id">' + esc(String(hit.id)) + "</td>"
    + '<td class="iconcell">' + icon_cell(hit) + "</td>"
    + '<td class="namecell">' + name_btn + " " + link + " " + toggle + "</td>"
    + '<td class="qualitycell"><span class="chip c-' + esc(quality) + '">' + esc(quality) + "</span></td>"
    + '<td class="pricecell">' + price_cell + "</td>"
    + "</tr>";
}

// ---- recipes card (v0.10.0, ADR 0021) --------------------------------------
// The door answers through cli_ok(["recipes"...]) like every other tab (ADR 0014's
// one-code-path rule). The ROW truth is honest about its OWN source: the job words,
// skill numbers and recipe names come from the FETCHED dataset (the second source
// docs/API.md records, MIT cited); the icons, names and prices beside them come
// THROUGH the item rows this DB owns. The two answers ride SIDE BY SIDE and never
// merge: an unjoined reagent keeps its dataset name and carries a state chip.
async function runRecipes() {
  var profession = document.getElementById("recipes-profession").value;
  // v0.14.0 (ADR 0028): NO bound. The bound lived HERE (a pager select feeding `--limit`
  // N), never in the door: ADR 0022's rule says no `--limit` means the rest, measured -
  // the door answers ALL the fixture's 12 rows and ALL 1 240 of the fetched set, and a
  // bound at or above the matching count answers the SAME bytes, so a cap only HID rows.
  // The tab asks the door's OWN argv, so a `cargo run -- recipes --icons --costs` line
  // shows the same rows AND the same cost column (v0.17.0, ADR 0032: `--costs` here
  // because the tab SHOWS the cost column - ADR 0014's one-code-path rule; the CLI
  // WITHOUT the flag keeps its bytes PUT, the cost column is this tab's ask).
  var argv = ["recipes", "--icons", "--costs"];
  if (profession !== "") { argv.push("--profession", profession); }
  try {
    var found = await cli_ok(argv);
    recipes_shown = found.recipes;
    live_shown = false;
    var rows = "";
    for (var i = 0; i < found.recipes.length; i = i + 1) {
      rows = rows + recipe_row(found.recipes[i]);
    }
    document.querySelector("#recipe-hits tbody").innerHTML = rows;
    var note = "ALL " + num(found.matching) + " recipe rows this filter answers are shown \u2014 the tab asks the door WITHOUT a bound (ADR 0028: the cap lived in the tab, the door never had one; a bound at or above this count would answer the SAME bytes, so a cap only HID rows, and growing it was rejected on that measure): the job words, skill numbers and material LISTS are the FETCHED set's own (the Sync tab\u2019s Recipes button: 8 files, MIT second source, cited here): the API has no professions door and its recipe cards never name a job or materials (measured); icons and money beside the materials are YOUR DB\u2019S OWN \u2014 an item the DB never saw says so beside its dataset name, and an enchanting craft says it has NO crafted item at all; the TOTAL COST column sums the set\u2019s materials \u00d7 their stated counts over YOUR DB\u2019S LIVE price layer (the twin the price refresh writes \u2014 never the stored min, NEVER a second fetch): money answers ONLY a cost every material of the set can price, and an incomplete cost names HOW MANY materials have no live price, words beside the count and NEVER money beside it (ADR 0015/0032); clicking a row\u2019s recipe name opens that entry\u2019s CARD (the local door, ZERO requests); the Watch button rides EVERY row shown, so it reaches the whole set\u2019s cooked items and the Details door then fetches exactly those records";
    if (found.matching === 0) {
      note = "no recipe rows for this filter \u2014 the set is fetched by the Sync tab\u2019s Recipes button (or the CLI stage), read here ever after";
    }
    if (profession !== "") {
      note = note + " \u2014 filtered to " + profession;
    }
    if (live_shown) {
      note = note + " \u2014 the money IN a cell is the STORED min; the LIVE words beside it are what the AH offered at the LAST price ask (the Sync tab\u2019s price refresh wrote that twin beside the stored copy \u2014 the two ride side by side and NEVER merge, and the live cell\u2019s hover keeps the exact copper)";
    }
    set_note(document.getElementById("recipes-note"), note, false);
    document.getElementById("btn-recipes").classList.remove("running");
  } catch (problem) {
    set_note(document.getElementById("recipes-note"), problem.message, true);
  }
}

function chip_word(text, css) {
  return '<span class="' + css + '">' + esc(text) + "</span>";
}

// v0.11.0 (ADR 0023): the LIVE twin beside a cell's stored money. The words come
// from Rust's ONE money formatter (the join wrote price_now_money), so the UI
// never does copper maths. The rule rides the STAMPS because the two NULLs mean
// different things (ADR 0017's third state vs ADR 0016's never-written):
// * price_now_missing_at set  -> "no auction right now" (looked, nothing offered)
// * price_now_copper present and DIFFERENT from the stored copper -> the live
//   money span, exact live copper in its hover (ADR 0015: hover keeps the number)
// * live equals stored -> NOTHING extra: the cell already answers what the AH
//   offers now, a second identical number would be noise
// * both stamps NULL and no live copper -> "live price not refreshed yet"
// The CALLER guards the fourth state: an item the DB has NO row for gets none of
// these (the door wrote no live fields for it — no live noise, ADR 0016's rule).
function live_cell(now_money, now_copper, stored_copper, at, missing_at) {
  if (missing_at !== null && missing_at !== undefined) {
    live_shown = true;
    return " " + chip_word("no auction right now", "live-missing");
  }
  if (now_money !== null && now_money !== undefined) {
    if (now_copper === stored_copper) { return ""; }
    live_shown = true;
    return ' <span class="live" title="' + esc(num(now_copper) + " copper exact") + '">live ' + esc(now_money) + "</span>";
  }
  live_shown = true;
  return " " + chip_word("live price not refreshed yet", "live-none");
}

function recipe_mats(one) {
  if (!one.reagents || one.reagents.length === 0) {
    return chip_word("the set states no materials for this recipe", "mat-none");
  }
  var out = "";
  for (var i = 0; i < one.reagents.length; i = i + 1) {
    var reagent = one.reagents[i];
    var name = reagent.db_name || reagent.dataset_name || ("item " + String(reagent.item_id));
    var state = "";
    if (reagent.in_db === false) {
      state = " " + chip_word("not in your DB", "mat-not-in-db");
    }
    var live = "";
    if (reagent.in_db !== false) {
      live = live_cell(reagent.price_now_money, reagent.price_now_copper, reagent.price_copper, reagent.price_now_at, reagent.price_now_missing_at);
    }
    out = out + '<span class="mat"><span class="matname">' + esc(name) + (reagent.count ? " \u00d7" + esc(String(reagent.count)) : "") + "</span> " + recipe_icon(reagent.icon_b64, reagent.icon_at, name) + " " + money_cell(reagent.price_money, reagent.price_copper) + live + state + "</span> ";
  }
  return out;
}

function recipe_crafted(one) {
  var label = one.crafted_db_name || one.crafted_dataset_name || ("item " + String(one.crafted_item_id));
  var state = "";
  if (one.crafted_item_id === null || one.crafted_item_id === undefined) {
    return chip_word("no crafted item \u2014 enchanting craft", "cooked-none");
  }
  if (!one.crafted_db_name) {
    state = " " + chip_word("the DB never saw this id", "cooked-not-in-db");
  } else {
    state = state + live_cell(one.crafted_price_now_money, one.crafted_price_now_copper, one.crafted_price_copper, one.crafted_now_at, one.crafted_now_missing_at);
  }
  return '<span class="cooked">' + esc(label) + " " + recipe_icon(one.crafted_icon_b64, one.crafted_icon_at, label) + " " + money_cell(one.crafted_price_money, one.crafted_price_copper) + state + "</span>";
}

function recipe_icon(b64, at, label) {
  if (b64) {
    return '<img class="listicon" src="data:image/jpeg;base64,' + esc(b64) + '" alt="' + esc(label) + ' icon">';
  }
  return '<span class="noicon" title="no bytes: never fetched (or checked as gone upstream) \u2014 the icons stage copies the DB\u2019s OWN copy">\u25c7</span>';
}

// v0.17.0 (ADR 0032): ONE cost renderer for the tab's cost COLUMN and the
// card's cost LINE (ADR 0014's rule: a column and a line cannot drift - and
// the Rust side sums BOTH with ONE craft_cost over the SAME joined reagents).
// The door filled the cost fields ONLY under `--costs` (serde skip); this tab
// and this card both ask. `cost_copper` answers ONLY when EVERY material has
// a live price (the DB's live layer, `price_min_now` - never the stored min):
// money_cell then gives the money words + the EXACT copper on hover (ADR
// 0015: money beside every copper number it labels). The three incomplete
// states are COUNTS with words beside them and ZERO money beside them (ADR
// 0015's other half; ADR 0004: never an invented zero - the 136 of 3 778
// materials the fetched catalogue never listed ride as counts, honest).
function cost_cell(one, mats) {
  if (one.cost_copper !== null && one.cost_copper !== undefined) {
    return money_cell(one.cost_money, one.cost_copper);
  }
  if (one.cost_missing === null || one.cost_missing === undefined) {
    return chip_word("the door was not asked for the cost", "costchip");
  }
  if (!mats || mats.length === 0) {
    return chip_word("nothing to cost - the set states no materials", "costchip");
  }
  var missing = one.cost_missing;
  if (missing >= mats.length) {
    return chip_word("no live prices yet - the price refresh must write them", "costchip");
  }
  return chip_word("cost incomplete - " + num(missing) + " of " + num(mats.length) + " materials have no live price", "costchip");
}

function recipe_row(one) {
  var skill = "";
  if (one.skill_required !== null && one.skill_required !== undefined) {
    skill = chip_word("skill " + String(one.skill_required), "skillchip") + " ";
  }
  var tier = one.skill_tier ? chip_word(one.skill_tier, "tierchip") + " " : "";
  // v0.12.0 (ADR 0025): the name cell IS the click target, and it carries the
  // spell id it belongs to (the same delegated pattern the search hits ride -
  // rows need no per-row listeners). The click fills the card door's id and reads
  // the SAME argv a `cargo run -- recipe N` line takes (ADR 0014's one-code-path
  // rule); the list itself stays a list.
  return "<tr>"
    + '<td class="col-job">' + chip_word(one.profession, "jobchip") + "</td>"
    + '<td class="col-recipe"><button class="openrecipe" data-spell="' + esc(String(one.spell_id)) + '" type="button">' + esc(one.recipe_name || one.spell_id) + "</button></td>"
    + '<td class="col-skill">' + skill + tier + "</td>"
    + '<td class="col-mats">' + recipe_mats(one) + "</td>"
    + '<td class="col-cost">' + cost_cell(one, one.reagents) + "</td>"
    + '<td class="col-cooked">' + recipe_crafted(one) + "</td>"
    + "</tr>";
}

// The watch button of THIS tab is the same verb the search tab's uses, aimed at the
// cooked items of the shown rows (one batched call, like watchHits). Two kinds of
// row are skipped HONESTLY, never invented: an enchanting craft has no crafted item
// at all (crafted_item_id is null), and a cooked item the DB never has a row for
// cannot be watched - the row already says "the DB never saw this id", and the door
// refuses to watch a row it has never seen (the watch verb's row guard).
async function watchRecipes() {
  if (recipes_shown.length === 0) {
    set_note(document.getElementById("recipes-note"), "read the recipes first: there are no rows to watch", true);
    return;
  }
  var argv = ["watch", "add"];
  var targets = 0;
  for (var i = 0; i < recipes_shown.length; i = i + 1) {
    var one = recipes_shown[i];
    if (one.crafted_item_id === null || one.crafted_item_id === undefined) { continue; }
    if (!one.crafted_db_name) { continue; }
    if (watched.indexOf(Number(one.crafted_item_id)) >= 0) { continue; }
    var again = false;
    for (var k = 2; k < argv.length; k = k + 1) {
      if (argv[k] === String(one.crafted_item_id)) { again = true; }
    }
    if (again) { continue; }
    argv.push(String(one.crafted_item_id));
    targets = targets + 1;
  }
  if (targets === 0) {
    set_note(document.getElementById("recipes-note"), "nothing new to watch \u2014 every cooked item of the shown rows is already watched, has no row in your DB, or is an enchanting craft with no crafted item", true);
    return;
  }
  try {
    await cli_ok(argv);
    await refreshWatched();
    set_note(document.getElementById("recipes-note"), "watching " + targets + " cooked items of the shown rows \u2014 the Details: watch list button fetches exactly those records, like every watch row before", false);
  } catch (problem) {
    set_note(document.getElementById("recipes-note"), problem.message, true);
  }
}
// ---- the recipe card (v0.12.0, ADR 0025) ------------------------------------
// ONE door answers this panel: `recipe SPELL_ID` - LOCAL, ZERO requests - so the
// click on a recipe row and a `cargo run -- recipe 132` line are ONE code path
// (ADR 0014's rule). The card shows the entry as the GAME shows it, composed from
// TWO sources that never merge: the fetched set's OWN words (job, name, the skill
// band it states) and the DB's OWN cells, reached BY VALUE through the ids the set
// states - the SCROLL that teaches it comes from the set's own learned-from bytes
// (measured census, ADR 0025: 804 of 1 240 real entries name a scroll item, 413
// state the `trainer` word, 23 the `automatic` word, and 8 NULLs state nothing at
// all), the crafted item and the materials through their ids. The cell machinery
// RIDES the row helpers above (recipe_icon / money_cell / live_cell / recipe_mats
// / chip_word), so a card cell cannot say something different from a list row.
// Money keeps ADR 0015's contract (words beside EVERY copper, never beside a
// count), and every gap names ITSELF (ADR 0004's no-invented-answers rule).
// The click hands the door its id directly (the row's name button carries it);
// typing an id into the field answers the same way. Both land in the SAME argv.
async function runRecipeCard(from_click) {
  // v0.15.0 (ADR 0029): the card window is OPENED, not switched to - it floats
  // over the tab the click came from (ADR 0018's tab machinery never sees it:
  // the section carries no data-tab any more, its nameplate died).
  open_pip("recipe-card");
  var field = document.getElementById("recipe-card-id");
  var raw = "";
  if (from_click !== undefined && from_click !== "") {
    raw = from_click;
    field.value = from_click;
  } else {
    raw = field.value.trim();
  }
  if (raw === "") {
    set_note(document.getElementById("recipe-card-note"), "type a spell id first \u2014 the Recipes tab\u2019s rows name theirs, and clicking a row\u2019s recipe name fills it for you", true);
    return;
  }
  try {
    // v0.17.0 (ADR 0032): the card ASKS --costs so its cost line has an answer -
    // the SAME flag the tab rides; a `cargo run -- recipe N --icons --costs` line
    // answers the SAME bytes (ADR 0014's one-code-path rule, restated at the card).
    var found = await cli_ok(["recipe", raw, "--icons", "--costs"]);
    live_shown = false;
    show_recipe_card(found);
    var named = 0;
    var answered = 0;
    if (found.result !== null && found.result !== undefined) {
      named = named + 1;
      if (found.result.in_db === true) { answered = answered + 1; }
    }
    if (found.scroll !== null && found.scroll !== undefined) {
      named = named + 1;
      if (found.scroll.in_db === true) { answered = answered + 1; }
    }
    for (var m = 0; m < found.materials.length; m = m + 1) {
      named = named + 1;
      if (found.materials[m].in_db === true) { answered = answered + 1; }
    }
    var line = "spell id " + String(found.spell_id) + " read LOCALLY, ZERO requests left this door: " + answered + " of " + named + " ids the entry names answered from your DB (the rest say so beside the set\u2019s own words) \u2014 the cells\u2019 name, quality, level, money, icon and tooltip words are YOUR DB\u2019S OWN; the entry\u2019s job, name and skill words are the fetched set\u2019s (source " + (found.source === null || found.source === undefined ? "unstated" : found.source) + ")";
    if (live_shown) {
      line = line + " \u2014 the money IN a cell is the STORED min; the LIVE words beside it are what the AH offered at the LAST price ask (never merged, the exact copper stays in the hover)";
    }
    set_note(document.getElementById("recipe-card-note"), line, false);
  } catch (problem) {
    if (problem.kind === "not-found") {
      set_note(document.getElementById("recipe-card-note"), problem.message + " \u2014 the Recipes tab lists the spell ids this DB does hold", false);
    } else {
      set_note(document.getElementById("recipe-card-note"), problem.message, true);
    }
  }
}

function show_recipe_card(found) {
  var parts = [];
  parts.push('<div class="rcardhead">' + card_head(found) + "</div>");
  parts.push('<div class="rcardlearn">' + learn_line(found) + "</div>");
  if (found.result !== null && found.result !== undefined) {
    parts.push(card_cell(found.result, "crafts"));
  } else {
    parts.push(chip_word("no crafted item \u2014 this craft puts an enchant on a slot", "cooked-none"));
  }
  if (found.scroll !== null && found.scroll !== undefined) {
    parts.push(card_cell(found.scroll, "teaches the spell"));
  }
  parts.push('<div class="rcardmats">' + recipe_mats({ "reagents": found.materials }) + "</div>");
  // v0.17.0 (ADR 0032): the cost line rides the SAME cost_cell the tab's cost
  // COLUMN calls (ONE renderer, ONE craft_cost behind BOTH doors - ADR 0014).
  parts.push('<div class="rcardcost">' + cost_cell(found, found.materials) + "</div>");
  document.getElementById("recipe-card-box").innerHTML = parts.join("");
}

// The entry's own words (the SET's) plus what its learned-from bytes NAMED. Two
// different NULLs ride apart here (ADR 0004/0016's rule again): `learned_type`
// NULL = the set stated nothing, a type word with no `learned_scroll_id` = the
// set stated a WORD that names no item (trainer / automatic, measured 413 / 23
// over the real set). The plain `acquisition` word the list chip read (ADR 0021)
// rides beside the new type word, never instead of it.
function card_head(found) {
  var out = chip_word(found.profession, "jobchip") + " ";
  out = out + '<span class="rcardname">' + esc(found.recipe_name === null || found.recipe_name === undefined ? ("spell " + String(found.spell_id)) : found.recipe_name) + "</span> ";
  out = out + chip_word("spell id " + String(found.spell_id), "learn-word") + " ";
  if (found.skill_required !== null && found.skill_required !== undefined) {
    out = out + chip_word("skill " + String(found.skill_required), "skillchip") + " ";
  }
  if (found.skill_tier !== null && found.skill_tier !== undefined) {
    out = out + chip_word(found.skill_tier, "tierchip") + " ";
  }
  if (found.trivial_low !== null && found.trivial_low !== undefined
      && found.trivial_high !== null && found.trivial_high !== undefined) {
    out = out + chip_word("grey-to-green band " + String(found.trivial_low) + " to " + String(found.trivial_high), "tierchip") + " ";
  }
  if (found.acquisition !== null && found.acquisition !== undefined) {
    out = out + chip_word(found.acquisition, "learn-word") + " ";
  }
  if (found.acquisition_note !== null && found.acquisition_note !== undefined) {
    out = out + '<span class="mat-none">' + esc(found.acquisition_note) + "</span> ";
  }
  if (found.source !== null && found.source !== undefined) {
    out = out + '<span class="mat-none">set source: ' + esc(found.source) + "</span>";
  }
  return out;
}

function learn_line(found) {
  if (found.learned_type === null || found.learned_type === undefined) {
    return chip_word("the set states NOTHING about how this recipe is learned", "learn-none");
  }
  var out = chip_word("learned by: " + found.learned_type, "learn-word") + " ";
  if (found.learned_scroll_id === null || found.learned_scroll_id === undefined) {
    out = out + chip_word("the set states the word alone \u2014 no scroll item exists for this kind of learning", "learn-none");
    return out;
  }
  out = out + chip_word("the set states the scroll BY VALUE (item id " + String(found.learned_scroll_id) + ")", "learn-word") + " ";
  if (found.learned_scroll_name !== null && found.learned_scroll_name !== undefined) {
    out = out + '<span class="matname">' + esc(found.learned_scroll_name) + "</span>";
  }
  return out;
}

// One cell: the DB's OWN words for one id the set named. `in_db` false keeps the
// tombstone rule honest (a row the user buried is NOT a row the card shows - ADR
// 0010's rule, and the door's own SELECT carries the same guard). The tooltip
// block is the record's OWN copy, printed as TEXT: upstream's markup never reaches
// the page (ADR 0012's readable twin is what renders).
function card_cell(cell, verb) {
  var name = cell.db_name || cell.dataset_name || ("item " + String(cell.item_id));
  var out = '<div class="rcell"><span class="rcellname">' + esc(verb) + " \u2014 " + esc(name);
  if (cell.db_name !== null && cell.db_name !== undefined && cell.dataset_name !== null && cell.dataset_name !== undefined && cell.db_name !== cell.dataset_name) {
    out = out + " (the set calls it " + esc(cell.dataset_name);
  }
  out = out + "</span> ";
  if (cell.in_db !== true) {
    out = out + chip_word("the DB never saw this id alive", "cooked-not-in-db");
    return out + "</div>";
  }
  if (cell.db_name !== null && cell.db_name !== undefined && cell.dataset_name !== null && cell.dataset_name !== undefined && cell.db_name !== cell.dataset_name) {
    out = out + ") ";
  }
  if (cell.quality !== null && cell.quality !== undefined) {
    out = out + chip_word(cell.quality, "chip c-" + esc(cell.quality)) + " ";
  }
  out = out + recipe_icon(cell.icon_b64, cell.icon_at, name) + " ";
  if (cell.item_level !== null && cell.item_level !== undefined) {
    out = out + chip_word("item level " + String(cell.item_level), "skillchip") + " ";
  }
  if (cell.required_level !== null && cell.required_level !== undefined) {
    out = out + chip_word("requires level " + String(cell.required_level), "skillchip") + " ";
  }
  if (cell.vendor_sell !== null && cell.vendor_sell !== undefined) {
    out = out + "vendor sell " + money_cell(cell.vendor_sell_money, cell.vendor_sell) + " ";
  }
  out = out + "min " + money_cell(cell.price_money, cell.price_copper) + " " + live_cell(cell.price_now_money, cell.price_now_copper, cell.price_copper, cell.price_now_at, cell.price_now_missing_at) + " ";
  if (cell.tooltip_text !== null && cell.tooltip_text !== undefined) {
    out = out + '<div class="rcopy"><span class="rcopyhead">the game\u2019s own words \u2014 this record\u2019s copied tooltip (copied ' + when(cell.tooltip_at) + ")</span><span>" + esc(cell.tooltip_text) + "</span></div>";
  } else if (cell.tooltip_missing_at !== null && cell.tooltip_missing_at !== undefined) {
    out = out + '<span class="mat-none">this record was checked as having NO description upstream (' + when(cell.tooltip_missing_at) + ")</span>";
  } else {
    out = out + '<span class="mat-none">no copy yet \u2014 the Descriptions stage asks this record once</span>';
  }
  return out + "</div>";
}

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
        // v0.9.8 (ADR 0019): the completion note names the stage's OWN third state
        // count when there is one - generic words that fit every stage without
        // guessing its meaning (detail: gone upstream; icons/tooltip: the FILE/copy
        // 404; price: the AH offers nothing now - checked once, never asked again,
        // ADR 0006's pattern all the way). Stages whose report carries no such
        // field stay byte-identical: the clause rides ONLY when missing > 0.
        var third_state = (report.missing !== null && report.missing !== undefined && report.missing > 0)
          ? ", " + num(report.missing) + " answered the stage's own third state (checked once, never asked again)"
          : "";
        set_note(document.getElementById("sync-note"),
          stage + " stage complete: " + num(report.items) + " seen, " + num(report.written) + " written, " + num(report.unchanged) + " reused" + third_state, false);
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
// v0.15.0 (ADR 0029): the row click's half of the delegated handler. The button
// carries the item id (ADR 0025's delegated pattern, now riding the search
// table too); the window's OWN field is what runCard reads, so filling it keeps
// ONE door and ONE reader - a click and a `cargo run -- item ID --cached` line
// stay the SAME code path (ADR 0014), and the window floats over the tab.
async function runCard_from_row(row_btn) {
  document.getElementById("card-id").value = row_btn.dataset.id;
  await runCard(false);
}

async function runCard(live) {
  // v0.15.0 (ADR 0029): the item card is a WINDOW now, not a tab - the first
  // thing any reach does is float it over the tab that is already visible
  // (open_pip/close_pip hold a window's visibility ALONE: its nameplate and
  // data-tab died, so show_tab's ONE truth never touches it).
  open_pip("item-card");
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
      line = line + " — no live price yet: the Sync tab's Price button (or the CLI stage) asks this copied record once (it may also CHECK the record as price-less — the door's own third state, the DB's stamps keep that apart)";
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
  // stage fills it. SINCE v0.9.8 (ADR 0019) the door is named: the Sync tab now
  // has a Price button, and it sends the SAME argv the CLI takes (ADR 0014: a
  // click and a cargo line are ONE code path), so the value arrives from either
  // - the row still shows WHAT THE STAGE SAW AT ITS ASK TIME and ages after.
  // gui.rs alone still builds the Rust shell, draws no card, reads no column
  // (verified; unchanged since v0.9.7) - the button lives webview-side.
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
    note = note + left + ", names only (a tooltip word is not searched)" + more_note + " - every hit row carries its own view link to the upstream page";
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
// v0.9.8 (ADR 0019): the three stage buttons the CLI used to own - SAME runSync
// loop, SAME cli_ok(["sync","--stage",...]) argv; chunking, cancel at the chunk
// end and resume from sync_state ride for free (ADR 0014: one code path).
document.getElementById("btn-icons").addEventListener("click", function () { runSync("icons"); });
document.getElementById("btn-tooltip").addEventListener("click", function () { runSync("tooltip"); });
document.getElementById("btn-price").addEventListener("click", function () { runSync("price"); });
// v0.10.0 (ADR 0021): the Recipes tab's listeners. btn-recipes-fetch rides the
// SAME runSync(stage) loop as every other stage button (ADR 0019's shape kept for
// the second source); the recipes stage is deliberately NOT chained into runPullAll
// - the chain's contract is "the catalogue is complete" over the API's doors, and
// the recipe set is a fetched dataset on ANOTHER host (recorded, ADR 0021 - not
// silently added). btn-recipes-watch watches the COOKED items of the shown rows:
// only ids the DB has rows for (the watch verb's row guard answers honestly for the
// rest - the door already marked them "the DB never saw this id").
document.getElementById("btn-recipes").addEventListener("click", function () { runRecipes(); });
document.getElementById("btn-recipes-fetch").addEventListener("click", function () { runSync("recipes"); });
document.getElementById("btn-recipes-watch").addEventListener("click", function () { watchRecipes(); });
document.getElementById("btn-recipe-card").addEventListener("click", function () { runRecipeCard(); });
// v0.15.0 (ADR 0029): the recipe window's OWN closer.
document.getElementById("btn-recipe-card-close").addEventListener("click", function () { close_pip("recipe-card"); });

// v0.12.0 (ADR 0025): ONE delegated handler over the recipe hits table - the row's
// name button carries the spell id it belongs to (the same delegated pattern the
// search hits and the tab strip ride, so a 100-row page plants no closures). The
// click fills the card door's id, reads the SAME argv a `cargo run -- recipe N`
// line takes (ADR 0014), and - v0.15.0 (ADR 0029) - lands the card WINDOW
// floating OVER this very tab: no nameplate, no tab switch, the list stays
// visible under the window. Anything else inside the table answers nothing.
document.getElementById("recipe-hits").addEventListener("click", function (event) {
  var node = event.target;
  if (node.dataset.spell !== undefined) { runRecipeCard(node.dataset.spell); }
});
document.getElementById("btn-card-db").addEventListener("click", function () { runCard(false); });
document.getElementById("btn-card-live").addEventListener("click", function () { runCard(true); });
// v0.15.0 (ADR 0029): the window's OWN closer - the only thing that hides a
// window back where it was born. The tab underneath is never touched.
document.getElementById("btn-card-close").addEventListener("click", function () { close_pip("item-card"); });
document.getElementById("btn-live").addEventListener("click", function () { runLive(); });
document.getElementById("btn-stats").addEventListener("click", function () { runStats(); });
document.getElementById("btn-edit").addEventListener("click", function () { runEdit(); });
document.getElementById("btn-create").addEventListener("click", function () { runCreate(); });
document.getElementById("btn-delete").addEventListener("click", function () { runHide(); });
document.getElementById("btn-undelete").addEventListener("click", function () { runUnhide(); });
document.getElementById("btn-twin-refresh").addEventListener("click", function () { runTwinRefresh(); });

// One delegated handler for the whole hits table: BOTH halves of a row ride it
// (v0.15.0, ADR 0029) - the watch toggle AND the window opener carry the SAME
// item id, and the class says which half the click takes: the opener fills the
// window's field and lands on runCard (the `item ID --cached` door - the window
// answers offline, ZERO requests, like its door), the toggle keeps watch.
// v0.16.0 (ADR 0030): the opener's half is the row's NAME cell button now - the
// split BY CLASS and everything it lands on stayed PUT word-for-word.
// Anything else inside the table answers nothing.
document.getElementById("hits").addEventListener("click", function (event) {
  var node = event.target;
  if (node.dataset.id !== undefined) {
    if (node.classList.contains("pipbtn")) { runCard_from_row(node); }
    else { toggle_watch(node); }
  }
});

// The tab strip rides the same delegated pattern: the button carries its tab's
// name, the strip itself answers nothing.
document.getElementById("tabs").addEventListener("click", function (event) {
  var node = event.target;
  if (node.dataset.tab !== undefined) { tab_click(node); }
});



// The cards are never empty: at startup the app reads its own DB (status, the watch
// list, the analytics) so the first paint already shows what exists. Those are verb
// calls, which is what makes boot() the witness of the launch audit: the Rust side
// logs every verb that reaches it.
async function boot() {
  // the painted default is overview (static classes in index.html); this call
  // re-asserts it from the ONE truth here and is the boot witness that the
  // tab code ran to the end (a throw above it would leave the first paint,
  // which the static classes already got right)
  show_tab(active_tab);
  await refreshStatus();
  await refreshWatched();
  await runStats();
  // A second status read after the analytics: it keeps the status card honest, and in
  // the launch audit it is the witness that runStats' rendering code ran to the end -
  // a JS throw after the awaited verb would leave only one `status` log line.
  await refreshStatus();
}

boot();

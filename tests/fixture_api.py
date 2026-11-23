"""Fixture API server: what CI runs the crawler against (docs/DEVELOPMENT.md, WGT_FIXTURE).

Answers the list stage (two pages), detail records for any item id, and the ICON FILE
stage (v0.6.0: REAL JPEG bytes - fetched once from the grounded asset host and embedded
here as base64, so CI stays offline and never hotlinks), and rate limits the FIRST
request of each stage (429 + Retry-After, to exercise the pacing path). Speaks
conditional GETs like the upstream docs promise: stable per-endpoint ETags and a real
304 (no body) for a matching If-None-Match, so v0.3.0's reuse paths are exercised.
One listed id has NO detail record (404), the shape v0.5.3's third state answers; one
ANSWERED record has no tooltip answer at all (404 with upstream's own JSON body - docs/
API.md measures that body live 2026-11-20), the shape v0.8.0's tooltip stage grew its
own third state for;
one record's detail answers WITH an icon_url whose icon FILE is gone (404 on /icon/),
the shape v0.6.0's icon-stage third state answers (docs/API.md measured the asset
host answering a 404 HTML page for a missing icon path; the crawler reads only the
status and discards the body).
Prices carry a per-PROCESS offset: one fixture process is one "observation window",
so a later process (a later crawl session into the same DB) sees different numbers -
that is what makes the price samples GROW across crawls while re-asks inside one
process still answer 304 and write nothing (ADR 0007's sampling rule, measured by
the mirror audit's history counts).
FROM v0.10.0 on the server ALSO speaks the SECOND source's dataset files (ADR 0021):
GET /data/<file>.json for the eight classic recipe files, top level {profession,
recipe_count, recipes[]} in the shape the live raw host answers, tiny hand-built
rows whose ids are the fixture's OWN item rows so every join state the recipes
door ships is reachable here - a crafted id with a row, a crafted id whose price
nobody ever answered (detail 404ed -> money "no price", never 0c), a reagent id the
DB never saw ("not in your DB" beside the set's own words), and enchanting entries
whose recipe names NO crafted item at all. The real set (8 files, 1 240 entries,
2.6 MB measured today) is NOT carried here and never should be: CI asserts the
door's states, the docs carry the real counts. A 404 FILE would be the source
moving - the stage aborts rc 3, measured on a scratch audit server, never committed
as a green step (ADR 0011: CI only runs what is true).
Nothing here touches the upstream API.
"""

import base64
import os

import http.server
import socketserver
import json

# One observation window per process: prices differ between fixture processes
# (later crawls see the market move) but are STABLE inside one process, so the
# v0.3.0/v0.5.0 reuse steps (re-ask the same ids -> 304, write nothing, sample
# nothing) keep their exact meaning. The ETag salt moves with the numbers: a
# changed observation is a changed ETag, never a stale 304 over new values.
# FROM v0.9.6 this definition sits ABOVE every page dict because the fixture's
# live price answers (fields.price, read by the refresh stage, ADR 0017) MOVE
# with RUN too - a dict literal is evaluated at import, so definition first.
RUN = str((os.getpid() * 7919) % 6000)

ATTRIBUTION = {"name": "TheWoWDB", "url": "https://thewowdb.com"}

LIST_PAGES = {
    "1": {
        "count": 5,
        "page": 1,
        "per_page": 3,
        "has_more": True,
        "attribution": ATTRIBUTION,
        "results": [
            {
                "id": 25,
                "name": "Worn Shortsword",
                "quality": "Common",
                "item_level": 1,
                "required_level": 1,
                "item_type": "Weapon",
                "item_subtype": "One-Handed Sword",
                "unique": None,
                "icon_url": "/icon/misc_invitem-111404.png",
                "links": {"api": "/api/v1/forever/item/25", "html": "/forever/item/25"},
            },
            {
                "id": 12809,
                "name": "Cenclis' Freshly Baked Bread",
                "quality": "Common",
                "required_level": 1,
                "links": {"api": "/api/v1/forever/item/12809"},
            },
            {"id": 40, "name": "Mossborer", "quality": "Common", "item_level": 4},
        ],
    },
    "2": {
        "count": 5,
        "page": 2,
        "per_page": 3,
        "has_more": False,
        "results": [
            {
                "id": 60,
                "name": "Twisted Wooden Cleaver",
                "quality": "Uncommon",
                "item_level": 3,
                "required_level": 3,
                "item_type": "Weapon",
            },
            {"id": 127555, "name": "Conjured Cocoa Choco-Frogo", "quality": "Common"},
        ],
    },
}

DETAIL_BASE = {
    "name": "Fixture Item",
    "quality": "Common",
    "item_level": 1,
    "required_level": 1,
    "item_type": "Weapon",
    "vendor_sell": 500,
    "weapon": {"damage_min": 1, "damage_max": 3, "dps": 1.5, "speed": 2.0},
    "price": {"min": 15000, "market": 12000, "available": 12, "realms": 2},
    "icon_url": "/icon/inv_mace_03.jpg",
    "attribution": ATTRIBUTION,
}

# The grounding for the icon stage lives HERE, as bytes: the JPEGs are the real
# files the grounded asset host serves (docs/API.md measured
# https://render.worldofwarcraft.com/us/icons/56/inv_mace_03.jpg as image/jpeg
# 1945 b and https://thewowdb.com/assets/icons/inv_axe_11.jpg as image/jpeg 1936
# b); they were fetched once and embedded, so CI never asks a host for pixels.
# The tooltip door (v0.8.0). The REAL body's key list is measured and printed in docs/
# API.md (attribution,build,fields,html,icon_url,id,kind,links,name,quality,version); the
# fixture replays that SHAPE with its own records, and its `html` is a small stand-in card
# in upstream's own class names - NOT upstream prose, whose text we never copy. The crawler
# reads only `html` (the description text the user asked the DB to hold). One answered id is
# ABSENT here on purpose: the tooltip stage's third state then has a real twin (the 404 below
# answers with upstream's own JSON error, the measured shape). FROM v0.9.6 the fields block is
# NO LONGER EMPTY because the REFRESH stage (ADR 0017) reads fields.price: the fixture replays
# the shape the LIVE door answered today (2026-11-21): keys available,market,min,realms,region,
# scope,updated_at with the price MOVING per process like the detail door's own (so the audit
# can SEE price_min_now differ from the stored price_min), and ONE answer carries the WHOLE
# fields.price as literal null - the fixture twin of the measured live id 200 third state
# ("looked, the AH offers nothing right now"), which the refresh stage must CHECK not invent.
TOOLTIP_PAGES = {
    "25": {
        "attribution": ATTRIBUTION,
        "build": "fixture-build",
        "fields": {"price": {"available": 5, "market": 1200, "min": 14000 + int(RUN), "realms": 3, "region": "us", "scope": "fixture Normal Alliance", "updated_at": "2026-11-21T00:00:00+00:00"}},
        "html": '<div class="twdb-card twdb-k-item"><div class="twdb-head"><img class="twdb-icon" src="https://render.worldofwarcraft.com/us/icons/56/inv_sword_04.jpg" alt="Fixture Detail 25" /></div><div class="twdb-body">Fixture Detail 25 Item Level 1 Main Hand One-Handed Sword 1 - 3 Damage Speed 1.9 Sell Price: 7c</div></div>',
        "icon_url": "https://render.worldofwarcraft.com/us/icons/56/inv_sword_04.jpg",
        "id": 25,
        "kind": "item",
        "links": {"api": "/api/v1/tooltip/forever/item/25", "html": "/forever/item/25"},
        "name": "Fixture Detail 25",
        "quality": "Common",
        "version": "forever",
    },
    "40": {
        "attribution": ATTRIBUTION,
        "build": "fixture-build",
        "fields": {"price": None},
        "html": '<div class="twdb-card twdb-k-item"><div class="twdb-body">Fixture Detail 40 Item Level 1 Sell Price: 7c</div></div>',
        "icon_url": "https://render.worldofwarcraft.com/us/icons/56/inv_sword_04.jpg",
        "id": 40,
        "kind": "item",
        "links": {"api": "/api/v1/tooltip/forever/item/40", "html": "/forever/item/40"},
        "name": "Fixture Detail 40",
        "quality": "Common",
        "version": "forever",
    },
    "60": {
        "attribution": ATTRIBUTION,
        "build": "fixture-build",
        "fields": {"price": {"available": 9, "market": 1300, "min": 13000 + int(RUN), "realms": 4, "region": "us", "scope": "fixture Normal Alliance", "updated_at": "2026-11-21T00:00:00+00:00"}},
        "html": '<div class="twdb-card twdb-k-item"><div class="twdb-body">Fixture Detail 60 Item Level 1 Sell Price: 7c</div></div>',
        "icon_url": "https://render.worldofwarcraft.com/us/icons/56/inv_sword_04.jpg",
        "id": 60,
        "kind": "item",
        "links": {"api": "/api/v1/tooltip/forever/item/60", "html": "/forever/item/60"},
        "name": "Fixture Detail 60",
        "quality": "Common",
        "version": "forever",
    },
    "127555": {
        "attribution": ATTRIBUTION,
        "build": "fixture-build",
        "fields": {},
        "html": '<div class="twdb-card twdb-k-item"><div class="twdb-body">Conjured Cocoa Choco-Frogo Conjured Item Sell Price: 7c</div></div>',
        "icon_url": "https://render.worldofwarcraft.com/us/icons/56/inv_sword_04.jpg",
        "id": 127555,
        "kind": "item",
        "links": {"api": "/api/v1/tooltip/forever/item/127555", "html": "/forever/item/127555"},
        "name": "Conjured Cocoa Choco-Frogo",
        "quality": "Common",
        "version": "forever",
    },
}

ICONS = {
    "inv_mace_03.jpg": base64.b64decode(
        "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"),
    "inv_axe_11.jpg": base64.b64decode(
        "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"),
}

# A record that EXISTS (its detail answers) but whose icon file is gone upstream:
# the icon stage's third state. Grounded: a missing icon path on the asset host
# answers 404 (measured); the crawler marks icon_missing_at and keeps crawling.
MISSING_ICON_ITEM = 60

# One observation window per process: prices differ between fixture processes
# (later crawls see the market move) but are STABLE inside one process, so the
# v0.3.0/v0.5.0 reuse steps (re-ask the same ids -> 304, write nothing, sample
# nothing) keep their exact meaning. The ETag salt moves with the numbers: a
# changed observation is a changed ETag, never a stale 304 over new values.

# A listed id the upstream has no detail record for - the live catalogue answers
# exactly this body for a removed record (docs/API.md), and the ids the list walk
# still lists those records for are many, so the crawler must mark the id
# checked-missing and keep crawling rather than abort (v0.5.3).
MISSING_DETAIL = {127555}

# One 429 per stage (the crawler retries in place and continues).
limited = set()
served = {"count": 0}

# The search door is NEVER gated: the door answers a 429 with exit 3 (like the
# `item` door), so gating it here would turn the committed CI step red - the door
# retries not in place because one human ask is one request (ADR 0009). The prose
# says it all: the gate below just never names the `search` stage.


# v0.10.0 (ADR 0021): the SECOND source's dataset files, in the shape the live raw
# host answers today (docs/API.md's second-source section measures it 2026-11-22):
# one file per classic profession under /data/<file>.json, top level
# {profession, recipe_count, recipes[]}, entries carrying the keys the crawler's
# RecipeEntry types (spell_id, spell_name, crafted_item_id, crafted_item_name,
# required_skill_value, skill_tier, trivial_low, trivial_high, reagents[],
# learned_from, acquisition, acquisition_note, source) - serde ignores the set's
# other measured keys. The ROWS here are hand-built and TINY on purpose: CI cannot
# carry the real set (8 files, 1 240 entries, 2.6 MB) and never should - the
# measured real counts live in the docs. What the tiny rows are chosen for is the
# ONE thing the recipes door's join needs: the fixture's OWN item ids (25 / 40 /
# 60 / 12809 / 127555, the rows the list + detail stages above already wrote), so
# every join state is reachable and asserted by the steps:
#   - a crafted id WITH a row (its DB name and money ride beside the set's own
#     crafted_item_name - note the DB-side name is the detail stage's overwrite,
#     "Fixture Detail 25", NOT the list name, so the two answers cannot hide each
#     other),
#   - a crafted id the set gives for row 127555, whose price nobody ever answered
#     (detail 404ed above) -> "no price", never 0c (ADR 0015/0016's rule),
#   - a reagent id the DB never saw (2589 / 5113) -> "not in your DB" beside the
#     set's own words,
#   - enchanting entries whose recipe names NO crafted item at all - the REAL set
#     measured today carries exactly 133 such entries, and the door ships the state
#     as words, never an invented id.
# v0.12.0 (ADR 0025): four rows now ALSO carry `learned_from`, in the THREE shapes
# the REAL set was measured carrying today (docs/API.md + ADR 0025 measure the census
# on the user's file: 804 schematic_item / 413 trainer / 23 automatic, and the keys
# are exactly item_id, item_name, type):
#   - 132 names a scroll BY VALUE (item 60, a row the detail stage above wrote AND
#     the tooltip stage copied) -> the card's scroll cell answers IN GAME, with the
#     copy's own words,
#   - 637 names a scroll id the DB never saw (2589) -> the cell says "the DB never
#     saw this id" beside the SET's own name for it, never an invented one,
#   - 202 answers the `trainer` word and 334 the `automatic` word: BOTH name no scroll
#     id at all, so the card states the type word and NO scroll cell - the gap keeps
#     its own state (ADR 0004's no-invented-answers rule again),
#   - 638 keeps the explicit NULL: the set stating NOTHING stays a different answer
#     from the set stating a type (measured: 133 enchanting entries carry no crafted
#     id and learn with none either).
RECIPE_FILES = {
    "alchemy_recipes.json": {
        "profession": "Alchemy",
        "recipe_count": 2,
        "recipes": [
            {
                "spell_id": 132,
                "spell_name": "Elixir of Fixture Frogweed",
                "crafted_item_id": 25,
                "crafted_item_name": "Kettle of Frogweed (the set's own words)",
                "required_skill_value": 40,
                "skill_tier": "Skill",
                "trivial_low": 35,
                "trivial_high": 55,
                "reagents": [
                    {"item_id": 40, "item_name": "Mossborer (the set's own words)", "count": 2},
                    {"item_id": 2589, "item_name": "Fixture Silkweed (no row in your DB)", "count": 1},
                ],
                "learned_from": {
                    "item_id": 60,
                    "item_name": "Pattern: Fixture Twisted Cleaver (the set's own words)",
                    "type": "schematic_item",
                },
                "acquisition": "trainer",
                "source": "cmangos-tbc-db+wago-tools-db2",
            },
            {
                "spell_id": 1723,
                "spell_name": "Greater Healing Tonic of the Frogo",
                "crafted_item_id": 12809,
                "crafted_item_name": "Cenclis' Bread (the set's own words)",
                "required_skill_value": 1,
                "skill_tier": "Apprentice",
                "trivial_low": 1,
                "trivial_high": 10,
                "reagents": [
                    {"item_id": 127555, "item_name": "Conjured Choco-Frogo (the set's own words)", "count": 1}
                ],
                "acquisition": "vendor",
                "source": "cmangos-tbc-db+wago-tools-db2",
            },
        ],
    },
    "blacksmithing_recipes.json": {
        "profession": "Blacksmithing",
        "recipe_count": 2,
        "recipes": [
            {
                "spell_id": 202,
                "spell_name": "Heavy Fixture Hammerhead",
                "crafted_item_id": 40,
                "crafted_item_name": "Mossborer (the set's own words)",
                "required_skill_value": 65,
                "skill_tier": "Expert",
                "trivial_low": 55,
                "trivial_high": 75,
                "reagents": [
                    {"item_id": 25, "item_name": "Worn Shortsword (the set's own words)", "count": 1},
                    {"item_id": 2589, "item_name": "Fixture Silkweed (no row in your DB)", "count": 2},
                ],
                "learned_from": {"type": "trainer"},
                "acquisition": "trainer",
                "acquisition_note": None,
                "source": "cmangos-tbc-db+wago-tools-db2",
            },
            {
                "spell_id": 683,
                "spell_name": "Mithran Fixture Rivet",
                "crafted_item_id": 60,
                "crafted_item_name": "Twisted Cleaver (the set's own words)",
                "required_skill_value": 155,
                "skill_tier": "Artisan",
                "trivial_low": 140,
                "trivial_high": 170,
                "reagents": [
                    {"item_id": 127555, "item_name": "Conjured Choco-Frogo (the set's own words)", "count": 3}
                ],
                "acquisition": "trainer",
                "acquisition_note": None,
                "source": "cmangos-tbc-db+wago-tools-db2",
            },
        ],
    },
    "cooking_recipes.json": {
        "profession": "Cooking",
        "recipe_count": 1,
        "recipes": [
            {
                "spell_id": 334,
                "spell_name": "Recipe of the Fixture Fin",
                "crafted_item_id": 127555,
                "crafted_item_name": "Conjured Cocoa Choco-Frogo (the set's own words)",
                "required_skill_value": 1,
                "skill_tier": "Apprentice",
                "trivial_low": 1,
                "trivial_high": 10,
                "reagents": [
                    {"item_id": 40, "item_name": "Mossborer (the set's own words)", "count": 1},
                    {"item_id": 5113, "item_name": "Fixture Gristleweed (no row in your DB)", "count": 1},
                ],
                "learned_from": {"type": "automatic"},
                "acquisition": "trainer",
                "source": "cmangos-tbc-db+wago-tools-db2",
            }
        ],
    },
    "enchanting_recipes.json": {
        # the measured shape of the REAL set's enchanting entries: NO crafted item
        # id at all (133 of 1 240 entries carry none), because the craft puts an
        # ENCHANT on a slot, not an item in a bag. The door ships the state as
        # words, never an invented id (ADR 0021).
        "profession": "Enchanting",
        "recipe_count": 2,
        "recipes": [
            {
                "spell_id": 637,
                "spell_name": "Enchant Fixture Brackets",
                "required_skill_value": 1,
                "skill_tier": "Apprentice",
                "trivial_low": 1,
                "trivial_high": 10,
                "reagents": [
                    {"item_id": 25, "item_name": "Worn Shortsword (the set's own words)", "count": 1},
                    {"item_id": 2589, "item_name": "Fixture Silkweed (no row in your DB)", "count": 1},
                ],
                "learned_from": {
                    "item_id": 2589,
                    "item_name": "Recipe: Fixture Silkweed (the set names a scroll your DB never saw)",
                    "type": "schematic_item",
                },
                "acquisition": "trainer",
                "source": "cmangos-tbc-db+wago-tools-db2",
            },
            {
                "spell_id": 638,
                "spell_name": "Enchant Fixture Sleeve",
                "crafted_item_id": None,
                "crafted_item_name": None,
                "required_skill_value": 210,
                "skill_tier": "Master",
                "trivial_low": 195,
                "trivial_high": 225,
                "reagents": [
                    {"item_id": 5113, "item_name": "Fixture Gristleweed (no row in your DB)", "count": 2}
                ],
                "learned_from": None,
                "acquisition": "trainer",
                "source": "cmangos-tbc-db+wago-tools-db2",
            },
        ],
    },
    "engineering_recipes.json": {
        "profession": "Engineering",
        "recipe_count": 1,
        "recipes": [
            {
                "spell_id": 4977,
                "spell_name": "Fixture Boltshell",
                "crafted_item_id": 25,
                "crafted_item_name": "Worn Shortsword (the set's own words)",
                "required_skill_value": 30,
                "skill_tier": "Skill",
                "trivial_low": 25,
                "trivial_high": 45,
                "reagents": [
                    {"item_id": 40, "item_name": "Mossborer (the set's own words)", "count": 1},
                    {"item_id": 127555, "item_name": "Conjured Choco-Frogo (the set's own words)", "count": 1},
                ],
                "acquisition": "trainer",
                "source": "cmangos-tbc-db+wago-tools-db2",
            }
        ],
    },
    "first_aid_recipes.json": {
        "profession": "First Aid",
        "recipe_count": 1,
        "recipes": [
            {
                "spell_id": 467,
                "spell_name": "Fixture Linen Pad",
                "crafted_item_id": 60,
                "crafted_item_name": "Twisted Wooden Cleaver (the set's own words)",
                "required_skill_value": 5,
                "skill_tier": "Apprentice",
                "trivial_low": 1,
                "trivial_high": 15,
                "reagents": [
                    {"item_id": 25, "item_name": "Worn Shortsword (the set's own words)", "count": 1}
                ],
                "acquisition": "auto_granted",
                "source": "cmangos-tbc-db+wago-tools-db2",
            }
        ],
    },
    "leatherworking_recipes.json": {
        "profession": "Leatherworking",
        "recipe_count": 1,
        "recipes": [
            {
                "spell_id": 785,
                "spell_name": "Fixture Scorched Bag",
                "crafted_item_id": 12809,
                "crafted_item_name": "Cenclis' Freshly Baked Bread (the set's own words)",
                "required_skill_value": 105,
                "skill_tier": "Artisan",
                "trivial_low": 90,
                "trivial_high": 120,
                "reagents": [
                    {"item_id": 40, "item_name": "Mossborer (the set's own words)", "count": 2},
                    {"item_id": 5113, "item_name": "Fixture Gristleweed (no row in your DB)", "count": 1},
                ],
                "acquisition": "trainer",
                "source": "cmangos-tbc-db+wago-tools-db2",
            }
        ],
    },
    "tailoring_recipes.json": {
        "profession": "Tailoring",
        "recipe_count": 2,
        "recipes": [
            {
                "spell_id": 3023,
                "spell_name": "Heavy Fixture Weave",
                "crafted_item_id": 25,
                "crafted_item_name": "Worn Shortsword (the set's own words)",
                "required_skill_value": 175,
                "skill_tier": "Artisan",
                "trivial_low": 160,
                "trivial_high": 190,
                "reagents": [
                    {"item_id": 2589, "item_name": "Fixture Silkweed (no row in your DB)", "count": 4}
                ],
                "acquisition": "trainer",
                "source": "cmangos-tbc-db+wago-tools-db2",
            },
            {
                "spell_id": 3024,
                "spell_name": "Bolt of Fixture Cloth",
                "crafted_item_id": 127555,
                "crafted_item_name": "Conjured Cocoa Choco-Frogo (the set's own words)",
                "required_skill_value": 115,
                "skill_tier": "Expert",
                "trivial_low": 100,
                "trivial_high": 130,
                "reagents": [
                    {"item_id": 60, "item_name": "Twisted Wooden Cleaver (the set's own words)", "count": 2},
                    {"item_id": 5113, "item_name": "Fixture Gristleweed (no row in your DB)", "count": 1},
                ],
                "acquisition": "trainer",
                "source": "cmangos-tbc-db+wago-tools-db2",
            },
        ],
    },
}


class Handler(http.server.BaseHTTPRequestHandler):
    def do_GET(self):
        served["count"] += 1
        path = self.path
        tail = path.split("?")[0].rstrip("/").split("/")[-1]

        if path.startswith("/api/v1/forever/item/") and tail.isdigit():
            stage = "detail"
            etag = '"fixture-etag-detail-{}-{}"'.format(tail, RUN)
            body = dict(DETAIL_BASE)
            body["id"] = int(tail)
            body["name"] = "Fixture Detail {}".format(tail)
            body["price"] = {"min": 15000 + int(RUN), "market": 12000 + int(RUN), "available": 12, "realms": 2}
            if int(tail) == MISSING_ICON_ITEM:
                # the record answers WITH an icon_url; only the FILE is gone
                body["icon_url"] = "/icon/absent-60.png"
        elif path.startswith("/api/v1/forever/search/"):
            # the grouped live-search answer, grounded on two measured live answers
            # (docs/API.md): `q=Worn Shortsword` answers ONE item hit in the item
            # group and empty groups elsewhere, every group with has_more false;
            # `q=Ring` answers five item hits with has_more TRUE plus the group's
            # own paged `more` URL. The fixture plays both shapes against its OWN
            # names: substring over fixture names, names only (a tooltip word like
            # "socket" answers empty groups - the measured names-only trap). The
            # `more` URL keeps upstream's own shape: this endpoint, `page=1&per_page=50`,
            # `+` for spaces - root-relative here so CI never reaches a second host.
            stage = "search"
            raw = path.split("?", 1)[1] if "?" in path else ""
            q = raw[2:] if raw.startswith("q=") else ""
            # the door percent-encodes; upstream decodes, so decode back for matching.
            q = q.replace("+", " ")
            for token in ("%20", "%2F", "%3A"):
                q = q.replace(token, {"%20": " ", "%2F": "/", "%3A": ":"}[token])
            groups = {}
            for kind in ("item", "npc", "quest", "sound", "spell", "zone"):
                hits = []
                if kind == "item":
                    for page in ("1", "2"):
                        for record in LIST_PAGES[page]["results"]:
                            if q.lower() and q.lower() in str(record.get("name", "")).lower():
                                hits.append({
                                    "id": record["id"],
                                    "name": record["name"],
                                    "kind": "item",
                                    "quality": record.get("quality"),
                                    "item_level": record.get("item_level"),
                                    "required_level": record.get("required_level"),
                                    "version": "forever",
                                    "links": {
                                        "api": "/api/v1/forever/item/{}".format(record["id"]),
                                        "html": "/forever/item/{}".format(record["id"]),
                                        "tooltip": "/api/v1/tooltip/forever/item/{}".format(record["id"]),
                                    },
                                })
                # the fixture grounds BOTH measured shapes: with five hits of one
                # word it answers has_more TRUE (measured: `?q=Ring` says so with
                # five results and a `more` page URL); three fixture names match
                # "sword", so has_more flips by itself. The results ride as the
                # first five at most, like upstream's page.
                groups[kind] = {"has_more": len(hits) > 2, "more": "/api/v1/forever/{}/?page=1&per_page=50&q={}".format(kind, q.replace(" ", "+")), "results": hits[:5]}
            body = {"q": q, "version": "forever", "attribution": ATTRIBUTION, "groups": groups}
            etag = 'W/"fixture-search-etag"'
        elif path.startswith("/api/v1/tooltip/"):
            # the tooltip door (v0.8.0): revalidated like the detail door (the probe read
            # a real ETag W/"2b111cbcd3bbb4d7078f6d05" and cache-control max-age=300), so a
            # targeted re-ask pays a 304 and the stage writes nothing - the same reuse
            # discipline docs/API.md already promises.
            stage = "tooltip"
            etag = '"fixture-etag-tooltip-{}-{}"'.format(tail, RUN)
            body = TOOLTIP_PAGES.get(tail)
        elif path.startswith("/data/"):
            # the SECOND source's files (v0.10.0, ADR 0021): the crawler asks
            # {base}/data/{file}.json, so the fixture speaks the SAME suffix - the
            # wire shape cannot drift from what CI runs. Conditional GETs like every
            # other door: the file ETag rides the meta table, a re-pull pays a real
            # 304 and writes nothing (the UPSERT keeps the rows the first pull wrote
            # - recipe rows have no per-record third state: a 404 FILE is the source
            # moving, and the stage ABORTS with rc 3, measured on a scratch audit
            # server, never committed here as a green step).
            stage = "recipes"
            etag = '"fixture-etag-recipe-{}-{}"'.format(tail, RUN)
            body = RECIPE_FILES.get(tail)
            if body is None:
                self.send_response(404)
                self.send_header("Content-Type", "application/json")
                self.end_headers()
                self.wfile.write(b'{"error":"not_found"}')
                return
        elif "item" in path and "page" in path:
            stage = "list"
            page = "2" if "page=2" in path else "1"
            etag = '"fixture-etag-list-{}"'.format(page)
            body = LIST_PAGES[page]
        elif path.startswith("/icon/"):
            stage = "icons"
            etag = None
            body = ICONS.get(tail)
        else:
            self.send_response(404)
            self.end_headers()
            self.wfile.write(b"{}")
            return

        if stage != "search" and stage not in limited:
            limited.add(stage)
            self.send_response(429)
            self.send_header("Retry-After", "1")
            self.send_header("X-RateLimit-Remaining", "0")
            self.end_headers()
            self.wfile.write(b'{"error":"slow down"}')
            return

        # the tooltip door's own "no tooltip here" answer: upstream's 404 body is JSON, not
        # the generic empty page (measured shape, docs/API.md: `No item … in forever.`), so
        # the fixture keeps that shape and the crawler marks the third state from the STATUS
        # alone, exactly as it does on the live door.
        if stage == "tooltip" and body is None:
            self.send_response(404)
            self.send_header("Content-Type", "application/json")
            self.end_headers()
            gone = {"error": "not_found", "message": "No item {} in forever.".format(tail), "status": 404}
            self.wfile.write(json.dumps(gone).encode())
            return

        # a removed record: upstream's own 404 body, no ETag offered for revalidation.
        if stage == "detail" and tail.isdigit() and int(tail) in MISSING_DETAIL:
            self.send_response(404)
            self.send_header("Content-Type", "application/json")
            self.end_headers()
            gone = {"error": "not_found", "message": "No item {} in forever.".format(tail), "status": 404}
            self.wfile.write(json.dumps(gone).encode())
            return

        if stage == "icons":
            # the icon stage's third state first: a gone icon FILE. The asset host
            # answers 404 with an HTML page (measured, docs/API.md) - the crawler
            # reads only the status for bytes and discards the body, so this double
            # keeps the status and the content type, not the page's prose.
            if body is None:
                self.send_response(404)
                self.send_header("Content-Type", "text/html")
                self.end_headers()
                self.wfile.write(b"<html><title>404</title></html>")
                return
            # no If-None-Match is sent for icons (v0.6.0 copies each icon exactly once,
            # docs/DATABASE.md), so no 304 path here and no ETag offered: inventing
            # revalidation headers the crawler never uses would fake a promise.
            self.send_response(200)
            self.send_header("Content-Type", "image/jpeg")
            self.send_header("Content-Length", str(len(body)))
            self.end_headers()
            self.wfile.write(body)
            return

        sent_before = self.headers.get("If-None-Match")
        if sent_before is not None and sent_before == etag:
            self.send_response(304)
            self.send_header("ETag", etag)
            self.end_headers()
            return

        self.send_response(200)
        self.send_header("Content-Type", "application/json")
        self.send_header("ETag", etag)
        self.end_headers()
        self.wfile.write(json.dumps(body).encode())


with socketserver.TCPServer(("127.0.0.1", 8765), Handler) as pool:
    pool.serve_forever()

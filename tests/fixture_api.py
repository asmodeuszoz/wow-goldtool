"""Fixture API server: what CI runs the crawler against (docs/DEVELOPMENT.md, WGT_FIXTURE).

Answers the list stage (two pages) and detail records for any item id, and rate limits
the FIRST request of each stage (429 + Retry-After, to exercise the pacing path). Speaks
conditional GETs like the upstream docs promise: stable per-endpoint ETags and a real
304 (no body) for a matching If-None-Match, so v0.3.0's reuse paths are exercised.
One listed id has NO detail record (404), the shape v0.5.3's third state answers.
Nothing here touches the upstream API.
"""

import http.server
import socketserver
import json

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
    "icon_url": "/icon/misc_invitem-111404.png",
    "attribution": ATTRIBUTION,
}

# A listed id the upstream has no detail record for - the live catalogue answers
# exactly this body for a removed record (docs/API.md), and the ids the list walk
# still lists those records for are many, so the crawler must mark the id
# checked-missing and keep crawling rather than abort (v0.5.3).
MISSING_DETAIL = {127555}

# One 429 per stage (the crawler retries in place and continues).
limited = set()
served = {"count": 0}


class Handler(http.server.BaseHTTPRequestHandler):
    def do_GET(self):
        served["count"] += 1
        path = self.path
        tail = path.split("?")[0].rstrip("/").split("/")[-1]

        if path.startswith("/api/v1/forever/item/") and tail.isdigit():
            stage = "detail"
            etag = '"fixture-etag-detail-{}"'.format(tail)
            body = dict(DETAIL_BASE)
            body["id"] = int(tail)
            body["name"] = "Fixture Detail {}".format(tail)
        elif path.startswith("/api/v1/forever/search/"):
            stage = "search"
            etag = '"fixture-etag-search"'
            body = {"q": "fixture", "groups": {"item": []}, "attribution": ATTRIBUTION}
        elif "item" in path and "page" in path:
            stage = "list"
            page = "2" if "page=2" in path else "1"
            etag = '"fixture-etag-list-{}"'.format(page)
            body = LIST_PAGES[page]
        else:
            self.send_response(404)
            self.end_headers()
            self.wfile.write(b"{}")
            return

        if stage not in limited:
            limited.add(stage)
            self.send_response(429)
            self.send_header("Retry-After", "1")
            self.send_header("X-RateLimit-Remaining", "0")
            self.end_headers()
            self.wfile.write(b'{"error":"slow down"}')
            return

        # a removed record: upstream's own 404 body, no ETag offered for revalidation.
        if stage == "detail" and int(tail) in MISSING_DETAIL:
            self.send_response(404)
            self.send_header("Content-Type", "application/json")
            self.end_headers()
            gone = {"error": "not_found", "message": "No item {} in forever.".format(tail), "status": 404}
            self.wfile.write(json.dumps(gone).encode())
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

import sys
import json
import time
import re
import logging

from scrapling.fetchers import Fetcher

logging.getLogger("scrapling").setLevel(logging.WARNING)

BASE = "https://www.azlyrics.com"

def page_html(page) -> str:
    if hasattr(page, "html_content") and page.html_content:
        return page.html_content
    return str(page)

def is_blocked_page(page) -> bool:
    html = page_html(page).lower()
    return (
        "request for access" in html
        or "az_recaptcha" in html
        or "az_unblock" in html
    )

def normalize(name):
    return re.sub(r'[^a-z0-9]', '', name.lower())

def get_artist_page(artist):
    slug = normalize(artist)
    if not slug:
        return None, "Invalid artist name"
    first = slug[0] if slug[0].isalpha() else '19'
    url = f"{BASE}/{first}/{slug}.html"
    try:
        page = Fetcher.get(url, stealthy_headers=True, impersonate='chrome')
        if is_blocked_page(page):
            return None, (
                "AZLyrics blocked automated access (captcha). "
                "Use Paste or File to add lyrics manually."
            )
        return page, None
    except Exception as e:
        return None, str(e)

def find_album_songs(artist_page, album_name):
    albums = {}
    current_album = None
    target = album_name.lower().strip().strip('"').strip("'")

    for element in artist_page.css("#listAlbum > div"):
        classes = element.attrib.get('class', '')
        if 'album' in classes and 'listalbum-item' not in classes:
            b_tag = element.css("b::text")
            if b_tag:
                current_album = b_tag.get().strip().strip('"').strip("'")
                albums[current_album] = []
        elif 'listalbum-item' in classes and current_album:
            link = element.css("a::attr(href)")
            song_name = element.css("a::text")
            if link and song_name:
                href = link.get()
                albums[current_album].append({
                    "name": song_name.get().strip(),
                    "url": BASE + href if href.startswith('/') else href
                })

    for name, songs in albums.items():
        if target in name.lower():
            return songs, None

    return None, f"Album '{album_name}' not found. Available: {list(albums.keys())}"

def get_lyrics(song_url):
    try:
        page = Fetcher.get(song_url, stealthy_headers=True, impersonate='chrome')
        if is_blocked_page(page):
            return None, "AZLyrics blocked automated access (captcha)"

        container = page.css("div.col-xs-12.col-lg-8.text-center")
        if not container:
            return None, "Lyrics container not found"

        divs = container[0].css("div")
        for div in divs:
            text = div.text or ""
            if len(text) > 100 and "ringtone" not in (div.attrib.get('class', '')):
                lyrics = text.strip()
                lyrics = re.sub(r'<!--.*?-->', '', lyrics, flags=re.DOTALL)
                lyrics = re.sub(r'\[.*?\]', '', lyrics)
                return lyrics.strip(), None

        return None, "No lyrics block found on song page"
    except Exception as e:
        return None, str(e)

def main():
    if len(sys.argv) < 3:
        print(json.dumps({"error": "Usage: scraper.py <artist> <album>"}))
        sys.exit(1)

    artist = sys.argv[1]
    album = sys.argv[2]

    artist_page, err = get_artist_page(artist)
    if err:
        print(json.dumps({"error": f"Failed to fetch artist page: {err}"}))
        sys.exit(1)

    songs, err = find_album_songs(artist_page, album)
    if err:
        print(json.dumps({"error": err}))
        sys.exit(1)

    results = []
    for i, song in enumerate(songs):
        print(json.dumps({"progress": f"Fetching {i+1}/{len(songs)}: {song['name']}"}), file=sys.stderr)
        lyrics, err = get_lyrics(song["url"])
        results.append({
            "name": song["name"],
            "lyrics": lyrics or "",
            "error": err
        })
        if i < len(songs) - 1:
            time.sleep(2)

    output = {
        "artist": artist,
        "album": album,
        "songs": results
    }
    print(json.dumps(output))

if __name__ == "__main__":
    main()

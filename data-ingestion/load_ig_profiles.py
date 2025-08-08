import instaloader
import json
import os
from pathlib import Path
import argparse

def main():
    parser = argparse.ArgumentParser(description="Download up to n images and shortcodes from an Instagram profile.")
    parser.add_argument("username", help="Instagram profile username to scrape")
    parser.add_argument("--cookiefile")
    parser.add_argument("--max-posts", type=int, default=10,
                        help="Maximum number of posts to download")
    args = parser.parse_args()

    L = instaloader.Instaloader(
        download_video_thumbnails=False,
        download_videos=False,
        download_comments=False,
        save_metadata=False,
        compress_json=False,
        post_metadata_txt_pattern="",
    )

    from sqlite3 import OperationalError, connect
    conn = connect(f"file:{args.cookiefile}?immutable=1", uri=True)
    try:
        cookie_data = conn.execute("SELECT name, value FROM moz_cookies WHERE baseDomain='instagram.com'")
    except OperationalError:
        cookie_data = conn.execute("SELECT name, value FROM moz_cookies WHERE host LIKE '%instagram.com'")
    L.context._session.cookies.update(dict(cookie_data))

    profile = instaloader.Profile.from_username(L.context, args.username)

    data = []

    for count, post in enumerate(profile.get_posts()):
        if count >= args.max_posts:
            break

        L.download_post(post, target=args.username)

        base_dir = Path(args.username)
        prefix = post.date_utc.strftime('%Y-%m-%d_%H-%M-%S_UTC')
        files = list(base_dir.glob(f"{prefix}*"))
        image_file = next((f for f in files if f.suffix.lower() in [".jpg", ".jpeg", ".png"]), None)

        if image_file:
            data.append({
                "shortcode": post.shortcode,
                "filepath": str(image_file)
            })
        else:
            print(f"[!] Couldn't find image for shortcode {post.shortcode}")

    out_file = f"{args.username}_posts.json"
    with open(out_file, "w") as f:
        json.dump(data, f, indent=2)

    print(f"✅ Saved {len(data)} posts for {args.username} → {out_file}")

if __name__ == "__main__":
    main()

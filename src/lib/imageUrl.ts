const PRESENTATION_PARAMS = [
  "auto",
  "dpr",
  "fit",
  "fm",
  "format",
  "h",
  "height",
  "q",
  "quality",
  "w",
  "width",
];

/** Canonicalise delivery-only image URL variants without discarding identity
 * parameters. In particular, Cloudflare Image Resizing wraps the original path
 * as `/cdn-cgi/image/<options>/<path>` — the 273 issue's RSS hero uses the
 * original path while its body uses that resized form. */
function canonicalImageUrl(raw: string): string | null {
  try {
    const url = new URL(raw);
    const resized = url.pathname.match(/^\/cdn-cgi\/image\/[^/]+(\/.*)$/);
    if (resized) url.pathname = resized[1];
    url.hash = "";
    PRESENTATION_PARAMS.forEach((key) => url.searchParams.delete(key));
    url.searchParams.sort();
    return url.toString();
  } catch {
    return null;
  }
}

/** Whether two URLs resolve to the same underlying image, allowing common CDN
 * resizing / formatting variants while keeping resource identity intact. */
export function isSameImageUrl(left: string, right: string): boolean {
  if (left === right) return true;
  const canonicalLeft = canonicalImageUrl(left);
  const canonicalRight = canonicalImageUrl(right);
  return canonicalLeft != null && canonicalLeft === canonicalRight;
}

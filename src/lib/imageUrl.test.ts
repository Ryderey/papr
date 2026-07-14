import { describe, expect, it } from "vitest";
import { isSameImageUrl } from "./imageUrl";

describe("isSameImageUrl", () => {
  it("recognises a Cloudflare-resized image as the original RSS image", () => {
    expect(
      isSameImageUrl(
        "https://cdn.tw93.fun/uPic/27342.JPG",
        "https://cdn.tw93.fun/cdn-cgi/image/width=2000,quality=80,format=auto,fit=scale-down/uPic/27342.JPG",
      ),
    ).toBe(true);
  });

  it("does not merge different images from the same CDN", () => {
    expect(
      isSameImageUrl(
        "https://cdn.tw93.fun/uPic/27342.JPG",
        "https://cdn.tw93.fun/uPic/mUuuQu23.png",
      ),
    ).toBe(false);
  });

  it("ignores presentation query parameters but keeps identity parameters", () => {
    expect(
      isSameImageUrl(
        "https://img.example/photo.jpg",
        "https://img.example/photo.jpg?width=800&quality=75&format=webp",
      ),
    ).toBe(true);
    expect(
      isSameImageUrl(
        "https://img.example/render?id=one&width=800",
        "https://img.example/render?id=two&width=800",
      ),
    ).toBe(false);
  });

  it("does not merge matching paths served by different hosts", () => {
    expect(
      isSameImageUrl(
        "https://one.example/uPic/27342.JPG",
        "https://two.example/uPic/27342.JPG",
      ),
    ).toBe(false);
  });
});

import { describe, expect, test } from "vitest";
import {
  buildMailto,
  isValidEmail,
  normalizeUrl,
  parseMailto,
} from "../src/components/LinkDialog/util";

describe("normalizeUrl", () => {
  test("accepts absolute URLs with allowed schemes", () => {
    expect(normalizeUrl("https://www.ironcalc.com/")).toBe(
      "https://www.ironcalc.com/",
    );
    expect(normalizeUrl("mailto:daniel@ironcalc.com")).toBe(
      "mailto:daniel@ironcalc.com",
    );
  });

  test("prepends https:// to domain-like values", () => {
    expect(normalizeUrl("www.ironcalc.com")).toBe("https://www.ironcalc.com");
  });

  test("rejects invalid or unsafe URLs", () => {
    expect(normalizeUrl("")).toBeNull();
    expect(normalizeUrl("not a url")).toBeNull();
    expect(normalizeUrl("javascript:alert(1)")).toBeNull();
  });
});

describe("parseMailto", () => {
  test("splits address and percent-decoded subject", () => {
    expect(
      parseMailto("mailto:daniel@ironcalc.com?subject=hola%20que%20tal"),
    ).toEqual({
      address: "daniel@ironcalc.com",
      subject: "hola que tal",
      otherParams: "",
    });
  });

  test("works without the mailto: scheme", () => {
    expect(parseMailto("daniel@ironcalc.com?subject=Hi")).toEqual({
      address: "daniel@ironcalc.com",
      subject: "Hi",
      otherParams: "",
    });
  });

  test("plain address has no subject", () => {
    expect(parseMailto("mailto:daniel@ironcalc.com")).toEqual({
      address: "daniel@ironcalc.com",
      subject: "",
      otherParams: "",
    });
  });

  test("keeps other parameters verbatim", () => {
    expect(
      parseMailto("mailto:a@b.com?body=Hello%20there&subject=Test&cc=c@d.com"),
    ).toEqual({
      address: "a@b.com",
      subject: "Test",
      otherParams: "body=Hello%20there&cc=c@d.com",
    });
  });

  test("malformed percent-encoding is kept as-is", () => {
    expect(parseMailto("mailto:a@b.com?subject=100%").subject).toBe("100%");
  });
});

describe("buildMailto", () => {
  test("percent-encodes the subject", () => {
    expect(buildMailto("daniel@ironcalc.com", "hola que tal", "")).toBe(
      "mailto:daniel@ironcalc.com?subject=hola%20que%20tal",
    );
  });

  test("no subject and no parameters gives a plain address", () => {
    expect(buildMailto("daniel@ironcalc.com", "", "")).toBe(
      "mailto:daniel@ironcalc.com",
    );
  });

  test("appends other parameters after the subject", () => {
    expect(buildMailto("a@b.com", "Test", "body=Hello%20there")).toBe(
      "mailto:a@b.com?subject=Test&body=Hello%20there",
    );
  });

  test("round-trips with parseMailto", () => {
    const target = "mailto:daniel@ironcalc.com?subject=hola%20que%20tal";
    const { address, subject, otherParams } = parseMailto(target);
    expect(buildMailto(address, subject, otherParams)).toBe(target);
  });
});

describe("isValidEmail", () => {
  test("accepts plain addresses and addresses with parameters", () => {
    expect(isValidEmail("daniel@ironcalc.com")).toBe(true);
    expect(isValidEmail("daniel@ironcalc.com?subject=Hi")).toBe(true);
  });

  test("rejects invalid addresses", () => {
    expect(isValidEmail("daniel")).toBe(false);
    expect(isValidEmail("daniel@localhost")).toBe(false);
    expect(isValidEmail("a b@c.com")).toBe(false);
  });
});

import { describe, expect, test } from "vitest";
import {
  buildMailto,
  CELL_REFERENCE_REGEX,
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
      body: "",
      otherParams: "",
    });
  });

  test("splits the percent-decoded body too", () => {
    expect(
      parseMailto("mailto:a@b.com?subject=Test&body=Hello%20there"),
    ).toEqual({
      address: "a@b.com",
      subject: "Test",
      body: "Hello there",
      otherParams: "",
    });
  });

  test("works without the mailto: scheme", () => {
    expect(parseMailto("daniel@ironcalc.com?subject=Hi")).toEqual({
      address: "daniel@ironcalc.com",
      subject: "Hi",
      body: "",
      otherParams: "",
    });
  });

  test("plain address has no subject or body", () => {
    expect(parseMailto("mailto:daniel@ironcalc.com")).toEqual({
      address: "daniel@ironcalc.com",
      subject: "",
      body: "",
      otherParams: "",
    });
  });

  test("keeps other parameters verbatim", () => {
    expect(
      parseMailto("mailto:a@b.com?body=Hello%20there&subject=Test&cc=c@d.com"),
    ).toEqual({
      address: "a@b.com",
      subject: "Test",
      body: "Hello there",
      otherParams: "cc=c@d.com",
    });
  });

  test("malformed percent-encoding is kept as-is", () => {
    expect(parseMailto("mailto:a@b.com?subject=100%").subject).toBe("100%");
    expect(parseMailto("mailto:a@b.com?body=100%").body).toBe("100%");
  });
});

describe("buildMailto", () => {
  test("percent-encodes the subject and the body", () => {
    expect(buildMailto("daniel@ironcalc.com", "hola que tal", "", "")).toBe(
      "mailto:daniel@ironcalc.com?subject=hola%20que%20tal",
    );
    expect(buildMailto("a@b.com", "Test", "Hello there", "")).toBe(
      "mailto:a@b.com?subject=Test&body=Hello%20there",
    );
  });

  test("body without a subject", () => {
    expect(buildMailto("a@b.com", "", "Hello there", "")).toBe(
      "mailto:a@b.com?body=Hello%20there",
    );
  });

  test("no subject, body or parameters gives a plain address", () => {
    expect(buildMailto("daniel@ironcalc.com", "", "", "")).toBe(
      "mailto:daniel@ironcalc.com",
    );
  });

  test("appends other parameters after the subject and the body", () => {
    expect(buildMailto("a@b.com", "Test", "Hello", "cc=c@d.com")).toBe(
      "mailto:a@b.com?subject=Test&body=Hello&cc=c@d.com",
    );
  });

  test("round-trips with parseMailto", () => {
    const target =
      "mailto:daniel@ironcalc.com?subject=hola%20que%20tal&body=Hello%20there";
    const { address, subject, body, otherParams } = parseMailto(target);
    expect(buildMailto(address, subject, body, otherParams)).toBe(target);
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

describe("CELL_REFERENCE_REGEX", () => {
  test("accepts single cells and ranges", () => {
    expect(CELL_REFERENCE_REGEX.test("B5")).toBe(true);
    expect(CELL_REFERENCE_REGEX.test("$B$5")).toBe(true);
    expect(CELL_REFERENCE_REGEX.test("A1:B5")).toBe(true);
    expect(CELL_REFERENCE_REGEX.test("$A$1:$B$5")).toBe(true);
  });

  test("rejects malformed references", () => {
    expect(CELL_REFERENCE_REGEX.test("A1:")).toBe(false);
    expect(CELL_REFERENCE_REGEX.test(":B5")).toBe(false);
    expect(CELL_REFERENCE_REGEX.test("A1:B5:C6")).toBe(false);
    expect(CELL_REFERENCE_REGEX.test("A")).toBe(false);
    expect(CELL_REFERENCE_REGEX.test("11")).toBe(false);
  });
});

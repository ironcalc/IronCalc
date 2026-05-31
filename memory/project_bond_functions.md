---
name: project-bond-functions
description: Bond functions (DURATION, PRICE, YIELD, ODDFPRICE, etc.) added on more-functions branch — includes known divergence from MS documentation for PRICE/YIELD
metadata:
  type: project
---

Bond functions added on branch `more-functions` in `base/src/functions/financial_bonds.rs`:
DURATION, MDURATION, PRICE, YIELD, ODDFPRICE, ODDFYIELD, ODDLPRICE, ODDLYIELD
PERCENTOF added to `base/src/functions/math_and_trigonometry/mathematical.rs`.

**Known divergence from MS documentation:**
- PRICE with basis=0 gives ~94.634 vs MS docs' 95.047 for the canonical example
- YIELD and ODDLYIELD diverge correspondingly
- Root cause: our formula uses standard compound-interest discounting (same as OpenOffice/QuantLib); MS Excel appears to use a slightly different convention internally
- Our formula IS internally consistent: YIELD(PRICE(yld)) = yld, par bond = 100, ODDLPRICE/ODDLYIELD round-trip

**Why:** Standard mathematical bond pricing formula; PRICE formula is confirmed correct against OpenOffice source and multiple financial textbooks.
**How to apply:** If someone reports PRICE/YIELD divergence from Excel, this is the known issue. The formula is mathematically correct; Excel may use a different day-count or discounting convention for basis=0.

Also: ODDFPRICE had a bug where `dsc = e - a` was used instead of `basis_days(settlement, first_coupon)` — fixed. The issue was that `a` is measured from `issue` (not the quasi-coupon period start), so `e - a ≠ dsc`.

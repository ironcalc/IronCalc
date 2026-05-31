---
layout: doc
outline: deep
lang: en-US
---

# PRICE function

PRICE is a function of the Financial category that computes the clean price per $100 face value of a bond that pays periodic coupons, given a required yield.

It returns the **clean price** — the present value of all future cash flows (coupons plus redemption) discounted at the given yield, minus the interest accrued since the last coupon. Clean price is the conventional market quote for bonds.

PRICE is the inverse of [YIELD](/functions/financial/yield): given what the market demands, it tells you what you should pay.

## Usage

### Syntax

**PRICE(<span title="Date" style="color:#1E88E5">settlement</span>, <span title="Date" style="color:#1E88E5">maturity</span>, <span title="Number" style="color:#1E88E5">rate</span>, <span title="Number" style="color:#1E88E5">yld</span>, <span title="Number" style="color:#1E88E5">redemption</span>, <span title="Number" style="color:#1E88E5">frequency</span>, <span title="Number" style="color:#1E88E5">basis</span>=0) => <span title="Number" style="color:#1E88E5">number</span>**

### Argument descriptions

* *settlement* ([date](/features/value-types#dates), required). The bond's settlement date. The date on which the buyer takes ownership of the bond.
* *maturity* ([date](/features/value-types#dates), required). The bond's maturity date. The date on which the principal is repaid and coupon payments end.
* *rate* ([number](/features/value-types#numbers), required). The bond's annual coupon rate as a decimal (e.g., `0.065` for a 6.5% coupon rate).
* *yld* ([number](/features/value-types#numbers), required). The bond's annual yield as a decimal (e.g., `0.09` for a 9% yield). Must be ≥ 0.
* *redemption* ([number](/features/value-types#numbers), required). The redemption value per $100 face value at maturity (e.g., `100`). Must be > 0.
* *frequency* ([number](/features/value-types#numbers), required). The number of coupon payments per year. Must be 1 (annual), 2 (semi-annual), or 4 (quarterly).
* *basis* ([number](/features/value-types#numbers), [optional](/features/optional-arguments.md)). The day-count convention to use (default 0). See the [basis table](#basis) below.

### Additional guidance

* *settlement* must be strictly before *maturity*; otherwise PRICE returns `#NUM!`.
* *rate* and *yld* must be ≥ 0 and *redemption* must be > 0; otherwise PRICE returns `#NUM!`.
* Dates should be entered as cell references or via the [DATE](/functions/date-and-time/date) function, not as text strings.

### Returned value

PRICE returns a [number](/features/value-types#numbers) representing the bond's clean price per $100 of face value.

### Error conditions

* If too few or too many arguments are supplied, PRICE returns the [`#ERROR!`](/features/error-types.md#error) error.
* If any argument is not (or cannot be converted to) a number, PRICE returns the [`#VALUE!`](/features/error-types.md#value) error.
* If *settlement* ≥ *maturity*, or *rate* < 0, or *yld* < 0, or *redemption* ≤ 0, or *frequency* ∉ {1, 2, 4}, or *basis* ∉ {0, 1, 2, 3, 4}, PRICE returns [`#NUM!`](/features/error-types.md#num).

<!--@include: ../markdown-snippets/error-type-details.txt-->

## Details

Let:
- $N$ = number of coupon periods remaining from settlement to maturity
- $E$ = total days in the coupon period that contains settlement
- $A$ = accrued days from the start of that coupon period to settlement
- $DSC$ = days from settlement to the next coupon date $= E - A$

The coupon amount per period is $C = \dfrac{100 \cdot r}{f}$.

$$
\text{PRICE} = \frac{\text{redemption}}{\left(1 + \frac{\text{yield}}{\text{frequency}}\right)^{N - 1 + \frac{DSC}{E}}} + \sum_{k=1}^{N}\frac{C}{\left(1 + \frac{\text{yield}}{\text{frequency}}\right)^{\frac{DSC}{E} + k - 1}} - C \cdot \frac{A}{E}

$$

### Basis

The *basis* argument selects the day-count convention used to compute $E$, $A$, and $DSC$:

| Value | Convention |
|------:|:-----------|
| 0 (default) | US 30/360 |
| 1 | Actual/Actual |
| 2 | Actual/360 |
| 3 | Actual/365 |
| 4 | European 30/360 |

See also [YEARFRAC](/functions/date-and-time/yearfrac) and the [basis glossary](/functions/financial#basis-and-day-count-convention).

### Example

The German government issued a 10-year bond on 1 June 2020, maturing 1 June 2030, with a 2% annual coupon, paying semiannually. Face value $100.

You want to buy it today (July 17 2025). Due to interest rate rises since 2020, the market now demands a 3% yield for this bond. What should you pay?

```
=PRICE(DATE(2025,7,17), DATE(2030,6,1), 0.02, 0.03, 100, 2, 1)
```

You will have to pay ~95.4952€ for the bond.

Let me just compute this carefully.

Settlement is **July 17, 2025**. Coupons fall on **June 1** and **December 1** each year.

---

**The current coupon period** is June 1, 2025 → December 1, 2025.

**E** — days from June 1, 2025 to December 1, 2025:
June: 30 days, but we start June 1, so 29 remaining in June + 31 July + 31 Aug + 30 Sep + 31 Oct + 30 Nov + 1 Dec = 183 days. With basis=1 (actual/actual) **E = 183**.

**A** — days from June 1, 2025 to July 17, 2025:
29 remaining days in June + 17 days in July = **A = 46**.

**DSC** = E − A = 183 − 46 = **DSC = 137**.

---

**N** — coupon periods from settlement to maturity. Each December 1 and June 1. The remaining coupon dates after settlement are
- Dec 1, 2025
- Jun 1, 2026
- Dec 1, 2026
- Jun 1, 2027
- Dec 1, 2027
- Jun 1, 2028
- Dec 1, 2028
- Jun 1, 2029
- Dec 1, 2029
- Jun 1, 2030

**N = 10**.

---

So in summary:

| Symbol | Value |
|---|---|
| N | 10 |
| E | 183 |
| A | 46 |
| DSC | 137 |

## Links

* See also IronCalc's [YIELD](/functions/financial/yield), [DURATION](/functions/financial/duration), and [MDURATION](/functions/financial/mduration) functions.
* Visit Microsoft Excel's [PRICE function](https://support.microsoft.com/en-us/office/price-function-3ea9deac-8dfa-436f-a7c8-17ea02c21b0a) page.
* Both [Google Sheets](https://support.google.com/docs/answer/3093243) and [LibreOffice Calc](https://wiki.documentfoundation.org/Documentation/Calc_Functions/PRICE) provide versions of the PRICE function.


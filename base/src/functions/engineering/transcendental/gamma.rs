use std::f64::consts::PI;

// FIXME: Please do a better approximation

/// Lanczos approximation for Gamma(z).
/// Valid for z > 0. For z <= 0 use `gamma` below (which does reflection).
fn gamma_lanczos(z: f64) -> f64 {
    const COEFFS: [f64; 9] = [
        0.999_999_999_999_809_9,
        676.5203681218851,
        -1259.1392167224028,
        771.323_428_777_653_1,
        -176.615_029_162_140_6,
        12.507343278686905,
        -0.13857109526572012,
        9.984_369_578_019_572e-6,
        1.5056327351493116e-7,
    ];

    let g = 7.0;
    let mut x = COEFFS[0];

    // sum_{i=1}^{n} c_i / (z + i)
    for (i, c) in COEFFS.iter().enumerate().skip(1) {
        x += c / (z + i as f64);
    }

    let t = z + g + 0.5;
    // √(2π) * t^(z+0.5) * e^{-t} * x
    (2.0 * PI).sqrt() * t.powf(z + 0.5) * (-t).exp() * x
}

/// Gamma function for all real x (except poles),
/// using Lanczos + reflection formula.
fn gamma(x: f64) -> f64 {
    if x.is_sign_negative() {
        // Reflection formula: Γ(x) = π / (sin(πx) * Γ(1 - x))
        // Beware of sin(πx) near zeros.
        let sin_pix = (PI * x).sin();
        if sin_pix == 0.0 {
            return f64::NAN;
        }
        PI / (sin_pix * gamma_lanczos(1.0 - x))
    } else if x == 0.0 {
        f64::INFINITY
    } else {
        gamma_lanczos(x)
    }
}

/// Factorial for real x: x! = Γ(x+1)
pub fn fact(x: f64) -> f64 {
    if x < 0.0 {
        return f64::NAN;
    }
    if x == 0.0 {
        return 1.0;
    }
    gamma(x)
}

/// Double factorial for real x.
///
/// Strategy:
/// 1. If x is (almost) integer and >= 0 → do exact product (stable, no surprises)
/// 2. Else:
///    - If x is “even-ish”: x = 2t → x!! = 2^t * Γ(t+1)
///    - If x is “odd-ish”:  x = 2t-1 → x!! = Γ(2t+1) / (2^t * Γ(t+1))
pub fn fact_double(x: f64) -> f64 {
    if x < 0.0 {
        return f64::NAN;
    }

    let xi = x.round();
    let is_int = (x - xi).abs() < 1e-9;

    if is_int {
        // integer path
        let mut acc = 1.0;
        let mut k = xi;
        while k > 1.0 {
            acc *= k;
            k -= 2.0;
        }
        return acc;
    }

    // non-integer path: use continuous extension
    let frac = x % 2.0;
    // even-ish if close to 0 mod 2
    if frac.abs() < 1e-6 || (frac - 2.0).abs() < 1e-6 {
        // x = 2t
        let t = x / 2.0;
        2f64.powf(t) * gamma(t + 1.0)
    } else if (frac - 1.0).abs() < 1e-6 || (frac + 1.0).abs() < 1e-6 {
        // x = 2t - 1  => t = (x+1)/2
        let t = (x + 1.0) / 2.0;
        // (2t - 1)!! = Γ(2t + 1) / (2^t Γ(t+1))
        // but 2t - 1 = x, so 2t + 1 = x + 2
        let num = gamma(x + 2.0);
        let den = 2f64.powf(t) * gamma(t + 1.0);
        num / den
    } else {
        // x is neither clearly even-ish nor odd-ish – fall back to a generic formula.
        // A safe thing to do is to treat it as even-ish:
        let t = x / 2.0;
        2f64.powf(t) * gamma(t + 1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fact_double() {
        let test_cases = [(5.0, 15.0), (6.0, 48.0), (0.0, 1.0), (1.0, 1.0), (2.0, 2.0)];
        for (input, expected) in test_cases {
            let result = fact_double(input);
            assert!(
                (result - expected).abs() < 1e-9,
                "fact_double({}) = {}, expected {}",
                input,
                result,
                expected
            );
        }
    }

    #[test]
    fn test_fact() {
        let test_cases = [
            (0.0, 1.0),
            (1.0, 1.0),
            (2.0, 2.0),
            (3.0, 6.0),
            (4.0, 24.0),
            (5.0, 120.0),
            (6.0, 720.0),
        ];
        for (input, expected) in test_cases {
            let result = fact(input);
            assert!(
                (result - expected).abs() < 1e-9,
                "fact({}) = {}, expected {}",
                input,
                result,
                expected
            );
        }
    }
}

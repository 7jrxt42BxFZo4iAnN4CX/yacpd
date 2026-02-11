//! Common helper functions for candlestick pattern detection
//!
//! TA-Lib compatible thresholds and comparison functions shared across all detector modules.

// ============================================================
// TA-Lib THRESHOLDS (from ta_utility.h)
// ============================================================

/// Body is doji-like: body <= avg_body * DOJI_FACTOR
pub const DOJI_FACTOR: f64 = 0.1;
/// Body is short: body < avg_body * SHORT_FACTOR
pub const BODY_SHORT_FACTOR: f64 = 1.0;
/// Body is long: body > avg_body * LONG_FACTOR
pub const BODY_LONG_FACTOR: f64 = 1.0;
/// Shadow is very long: shadow > avg_body * SHADOW_VERYLONG_FACTOR
/// TA-Lib uses 2.0 (ta_global.c:167)
pub const SHADOW_VERYLONG_FACTOR: f64 = 2.0;
/// Shadow very short: shadow < avg_range * SHADOW_VERYSHORT_FACTOR
/// TA-Lib: RangeType_HighLow, Period=10, Factor=0.1
pub const SHADOW_VERYSHORT_FACTOR: f64 = 0.1;
/// Equal threshold for price equality
pub const EQUAL_FACTOR: f64 = 0.05;
/// Near threshold for price near-equality
/// TA-Lib uses 0.2 (ta_global.c:175)
pub const NEAR_FACTOR: f64 = 0.2;
/// Far threshold for price far-apart
/// TA-Lib uses 0.6 (ta_global.c:178)
pub const FAR_FACTOR: f64 = 0.6;

// Fallback ratio-based thresholds (when avg_body is not meaningful)
pub const DOJI_RATIO: f64 = 0.1;
pub const BODY_SHORT_RATIO: f64 = 0.3;
pub const BODY_LONG_RATIO: f64 = 0.7;
pub const SHADOW_LONG_RATIO: f64 = 0.3;
pub const SHADOW_VERYLONG_RATIO: f64 = 0.4;
pub const SHADOW_SHORT_RATIO: f64 = 0.1;

// ============================================================
// HELPER FUNCTIONS (TA-Lib style with ratio fallback)
// ============================================================

/// Check if body is doji-like (TA-Lib: BodyDoji)
/// TA-Lib uses avg HL range * 0.1 (RangeType=HighLow, Factor=0.1)
/// Falls back to ratio-based if avg_range is not meaningful.
/// TA-Lib: when body=0, it is always a doji (body 0 <= any threshold).
#[inline]
pub fn is_doji(body: f64, avg_range: f64, range: f64) -> bool {
    // Zero body is always a doji (matches TA-Lib: 0 <= threshold for any threshold)
    if body <= 0.0 {
        return true;
    }
    if avg_range > 0.0 {
        body <= avg_range * DOJI_FACTOR
    } else {
        range > 0.0 && body / range <= DOJI_RATIO
    }
}

/// Check if body is short (TA-Lib: BodyShort)
#[inline]
pub fn is_body_short(body: f64, avg_body: f64, range: f64) -> bool {
    if avg_body > 0.0 {
        body < avg_body * BODY_SHORT_FACTOR
    } else {
        range > 0.0 && body / range <= BODY_SHORT_RATIO
    }
}

/// Check if body is long (TA-Lib: BodyLong)
#[inline]
pub fn is_body_long(body: f64, avg_body: f64, range: f64) -> bool {
    if avg_body > 0.0 {
        body > avg_body * BODY_LONG_FACTOR
    } else {
        range > 0.0 && body / range >= BODY_LONG_RATIO
    }
}

/// Check if shadow is long (TA-Lib: ShadowLong)
/// TA-Lib uses RangeType=RealBody, Period=0, Factor=1.0
/// Period=0 means compare against current bar's body, not average.
/// When body=0, threshold is 1.0*0=0, so any positive shadow passes.
#[inline]
pub fn is_shadow_long(shadow: f64, body: f64, _range: f64) -> bool {
    // TA-Lib: shadow > body * 1.0; when body=0 threshold=0, any shadow > 0 passes
    shadow > body
}

/// Check if shadow is very long (TA-Lib: ShadowVeryLong)
/// TA-Lib uses RangeType=RealBody, Period=0, Factor=2.0
/// Period=0 means compare against current bar's body, not average.
/// When body=0, threshold is 2.0*0=0, so any positive shadow passes.
#[inline]
pub fn is_shadow_verylong(shadow: f64, body: f64, _range: f64) -> bool {
    // TA-Lib: shadow > body * 2.0; when body=0 threshold=0, any shadow > 0 passes
    shadow > body * SHADOW_VERYLONG_FACTOR
}

/// Check if shadow is short (TA-Lib: ShadowShort)
/// TA-Lib uses RangeType=Shadow (max of upper/lower shadows), Period=10, Factor=1.0
/// Pass avg_shadow = avg(max(upper, lower)) over lookback period
#[inline]
pub fn is_shadow_short(shadow: f64, avg_shadow: f64, range: f64) -> bool {
    if avg_shadow > 0.0 {
        shadow < avg_shadow
    } else {
        range > 0.0 && shadow / range <= SHADOW_SHORT_RATIO
    }
}

/// Check if shadow is very short (TA-Lib: ShadowVeryShort)
/// Uses avg_range (HighLow) with factor 0.1
#[inline]
pub fn is_shadow_very_short(shadow: f64, avg_range: f64, range: f64) -> bool {
    if avg_range > 0.0 {
        shadow < avg_range * SHADOW_VERYSHORT_FACTOR
    } else {
        range > 0.0 && shadow / range <= SHADOW_SHORT_RATIO
    }
}

/// Compute trailing average body at a specific bar index.
/// TA-Lib uses per-bar-position trailing averages for multi-bar patterns.
#[inline]
pub fn trailing_avg_body<T: crate::OHLCV>(bars: &[T], at: usize, period: usize) -> f64 {
    use crate::OHLCVExt;
    if at == 0 {
        return OHLCVExt::body(&bars[0]);
    }
    let s = at.saturating_sub(period);
    let slice = &bars[s..at];
    let sum: f64 = slice.iter().map(|b| OHLCVExt::body(b)).sum();
    sum / slice.len() as f64
}

/// Compute trailing average range at a specific bar index (for Near/Far/Equal).
#[inline]
pub fn trailing_avg_range<T: crate::OHLCV>(bars: &[T], at: usize, period: usize) -> f64 {
    if at == 0 {
        return crate::OHLCVExt::range(&bars[0]);
    }
    let s = at.saturating_sub(period);
    let slice = &bars[s..at];
    let sum: f64 = slice.iter().map(|b| crate::OHLCVExt::range(b)).sum();
    sum / slice.len() as f64
}

/// Compute trailing average shadow at a specific bar index.
/// TA-Lib ShadowShort: RangeType=Shadows, Factor=1.0, Period=10
/// Shadows = (upper_shadow + lower_shadow), then avg / 2.0 (per TA_CANDLEAVERAGE)
#[inline]
pub fn trailing_avg_shadow<T: crate::OHLCV>(bars: &[T], at: usize, period: usize) -> f64 {
    use crate::OHLCVExt;
    if at == 0 {
        return (OHLCVExt::upper_shadow(&bars[0]) + OHLCVExt::lower_shadow(&bars[0])) / 2.0;
    }
    let s = at.saturating_sub(period);
    let slice = &bars[s..at];
    let sum: f64 = slice
        .iter()
        .map(|b| OHLCVExt::upper_shadow(b) + OHLCVExt::lower_shadow(b))
        .sum();
    sum / slice.len() as f64 / 2.0
}

/// Check if a bar is a marubozu (no/minimal shadows).
/// Returns `Some(true)` if marubozu, `Some(false)` if not, `None` if range is zero.
#[inline]
pub fn is_marubozu<T: crate::OHLCVExt>(bar: &T, shadow_max_ratio: f64) -> Option<bool> {
    let upper = bar.upper_shadow_ratio()?;
    let lower = bar.lower_shadow_ratio()?;
    Some(upper <= shadow_max_ratio && lower <= shadow_max_ratio)
}

// ============================================================
// FACTOR-PARAMETERIZED VARIANTS
// ============================================================
// These allow detectors to override the default TA-Lib factors
// while keeping the same fallback logic.

/// Like [`is_doji`] but with a custom factor (replaces [`DOJI_FACTOR`]).
#[inline]
pub fn is_doji_f(body: f64, avg_range: f64, range: f64, factor: f64) -> bool {
    if body <= 0.0 {
        return true;
    }
    if avg_range > 0.0 {
        body <= avg_range * factor
    } else {
        range > 0.0 && body / range <= DOJI_RATIO
    }
}

/// Like [`is_body_short`] but with a custom factor (replaces [`BODY_SHORT_FACTOR`]).
#[inline]
pub fn is_body_short_f(body: f64, avg_body: f64, range: f64, factor: f64) -> bool {
    if avg_body > 0.0 {
        body < avg_body * factor
    } else {
        range > 0.0 && body / range <= BODY_SHORT_RATIO
    }
}

/// Like [`is_body_long`] but with a custom factor (replaces [`BODY_LONG_FACTOR`]).
#[inline]
pub fn is_body_long_f(body: f64, avg_body: f64, range: f64, factor: f64) -> bool {
    if avg_body > 0.0 {
        body > avg_body * factor
    } else {
        range > 0.0 && body / range >= BODY_LONG_RATIO
    }
}

/// Like [`is_shadow_verylong`] but with a custom factor (replaces [`SHADOW_VERYLONG_FACTOR`]).
#[inline]
pub fn is_shadow_verylong_f(shadow: f64, body: f64, _range: f64, factor: f64) -> bool {
    shadow > body * factor
}

/// Like [`is_shadow_very_short`] but with a custom factor (replaces [`SHADOW_VERYSHORT_FACTOR`]).
#[inline]
pub fn is_shadow_very_short_f(shadow: f64, avg_range: f64, range: f64, factor: f64) -> bool {
    if avg_range > 0.0 {
        shadow < avg_range * factor
    } else {
        range > 0.0 && shadow / range <= SHADOW_SHORT_RATIO
    }
}

/// Returns true if `shadow` exceeds the "very short" threshold (i.e. is NOT very short).
/// Inverse of [`is_shadow_very_short_f`] with custom factor, used for checking that a shadow
/// is meaningfully long (e.g. the lower shadow of a Dragonfly Doji).
#[inline]
pub fn shadow_exceeds_veryshort(shadow: f64, avg_range: f64, factor: f64, range: f64) -> bool {
    let threshold = avg_range * factor;
    if threshold > 0.0 {
        shadow > threshold
    } else if range > 0.0 {
        shadow / range > SHADOW_SHORT_RATIO
    } else {
        false
    }
}

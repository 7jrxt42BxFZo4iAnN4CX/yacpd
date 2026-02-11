//! Multi-bar candlestick pattern detectors (4+ bars, TA-Lib compatible)
//!
//! TA-Lib patterns: CDLBREAKAWAY, CDLCONCEALBABYSWALL, CDLHIKKAKE, CDLHIKKAKEMOD,
//! CDLLADDERBOTTOM, CDLMATHOLD, CDLRISEFALL3METHODS, CDLXSIDEGAP3METHODS

#![allow(clippy::collapsible_if, clippy::default_constructed_unit_structs)]

use std::collections::HashMap;

use super::helpers::{is_body_long_f, is_body_short_f};
use crate::{
    params::{get_ratio, ParamMeta, ParamType, ParameterizedDetector},
    Direction, MarketContext, OHLCVExt, PatternDetector, PatternId, PatternMatch, Ratio, Result,
    OHLCV,
};

impl_with_defaults!(
    BreakawayDetector,
    ConcealingBabySwallowDetector,
    HikkakeDetector,
    HikkakeModDetector,
    LadderBottomDetector,
    MatHoldDetector,
    RiseFallThreeMethodsDetector,
    XSideGapThreeMethodsDetector,
);

// ============================================================
// BREAKAWAY
// ============================================================

/// CDLBREAKAWAY - Breakaway (5-bar pattern)
#[derive(Debug, Clone)]
pub struct BreakawayDetector {
    pub body_long_factor: f64,
}

impl Default for BreakawayDetector {
    fn default() -> Self {
        Self {
            body_long_factor: super::helpers::BODY_LONG_FACTOR,
        }
    }
}

impl PatternDetector for BreakawayDetector {
    fn id(&self) -> PatternId {
        PatternId("CDL_BREAKAWAY")
    }

    fn min_bars(&self) -> usize {
        5
    }

    fn detect<T: OHLCV>(
        &self,
        bars: &[T],
        index: usize,
        _ctx: &MarketContext,
    ) -> Option<PatternMatch> {
        if index < 4 {
            return None;
        }
        let first = bars.get(index - 4)?;
        let second = bars.get(index - 3)?;
        let third = bars.get(index - 2)?;
        let fourth = bars.get(index - 1)?;
        let fifth = bars.get(index)?;

        // TA-Lib: TA_CANDLECOLOR convention (close >= open = white/+1, close < open = black/-1)
        let color_first = if first.close() >= first.open() { 1 } else { -1 };
        let color_second = if second.close() >= second.open() {
            1
        } else {
            -1
        };
        let color_fourth = if fourth.close() >= fourth.open() {
            1
        } else {
            -1
        };
        let color_fifth = if fifth.close() >= fifth.open() { 1 } else { -1 };

        // TA-Lib: colors i-4, i-3, i-1 same; i opposite (no check on i-2)
        if color_first != color_second {
            return None;
        }
        if color_second != color_fourth {
            return None;
        }
        if color_fourth == color_fifth {
            return None;
        }

        // TA-Lib: first candle is BodyLong
        let avg_body_first = super::helpers::trailing_avg_body(bars, index - 4, 10);
        let first_body = first.body();
        if !is_body_long_f(
            first_body,
            avg_body_first,
            first.range(),
            self.body_long_factor,
        ) {
            return None;
        }

        // TA-Lib: RealBodyGap between 2nd and 1st
        let first_body_hi = first.open().max(first.close());
        let first_body_lo = first.open().min(first.close());
        let second_body_hi = second.open().max(second.close());
        let second_body_lo = second.open().min(second.close());

        if color_first == 1 {
            // White/bullish first — gap up: RealBodyGapUp(i-3, i-4)
            if second_body_lo <= first_body_hi {
                return None;
            }

            // TA-Lib: progressive higher highs AND lows for i-2 vs i-3, i-1 vs i-2
            if third.high() <= second.high() || third.low() <= second.low() {
                return None;
            }
            if fourth.high() <= third.high() || fourth.low() <= third.low() {
                return None;
            }

            // TA-Lib: close[i] < open[i-3] && close[i] > close[i-4]
            if fifth.close() >= second.open() {
                return None;
            }
            if fifth.close() <= first.close() {
                return None;
            }

            Some(PatternMatch {
                pattern_id: PatternDetector::id(self),
                direction: Direction::Bearish,
                strength: 0.7,
                start_index: index - 4,
                end_index: index,
            })
        } else {
            // Black/bearish first — gap down: RealBodyGapDown(i-3, i-4)
            if second_body_hi >= first_body_lo {
                return None;
            }

            // TA-Lib: progressive lower highs AND lows for i-2 vs i-3, i-1 vs i-2
            if third.high() >= second.high() || third.low() >= second.low() {
                return None;
            }
            if fourth.high() >= third.high() || fourth.low() >= third.low() {
                return None;
            }

            // TA-Lib: close[i] > open[i-3] && close[i] < close[i-4]
            if fifth.close() <= second.open() {
                return None;
            }
            if fifth.close() >= first.close() {
                return None;
            }

            Some(PatternMatch {
                pattern_id: PatternDetector::id(self),
                direction: Direction::Bullish,
                strength: 0.7,
                start_index: index - 4,
                end_index: index,
            })
        }
    }
}

// ============================================================
// CONCEALING BABY SWALLOW
// ============================================================

/// CDLCONCEALBABYSWALL - Concealing Baby Swallow (4-bar pattern)
#[derive(Debug, Clone)]
pub struct ConcealingBabySwallowDetector {
    pub shadow_max_ratio: Ratio,
}

impl Default for ConcealingBabySwallowDetector {
    fn default() -> Self {
        Self {
            shadow_max_ratio: Ratio::new_const(0.05),
        }
    }
}

impl PatternDetector for ConcealingBabySwallowDetector {
    fn id(&self) -> PatternId {
        PatternId("CDL_CONCEALBABYSWALL")
    }

    fn min_bars(&self) -> usize {
        4
    }

    fn detect<T: OHLCV>(
        &self,
        bars: &[T],
        index: usize,
        _ctx: &MarketContext,
    ) -> Option<PatternMatch> {
        if index < 3 {
            return None;
        }
        let first = bars.get(index - 3)?;
        let second = bars.get(index - 2)?;
        let third = bars.get(index - 1)?;
        let fourth = bars.get(index)?;

        // All four must be bearish
        if !first.is_bearish()
            || !second.is_bearish()
            || !third.is_bearish()
            || !fourth.is_bearish()
        {
            return None;
        }

        // First and second are marubozu (no/minimal shadows)
        let first_upper = first.upper_shadow_ratio()?;
        let first_lower = first.lower_shadow_ratio()?;
        let second_upper = second.upper_shadow_ratio()?;
        let second_lower = second.lower_shadow_ratio()?;

        if first_upper > self.shadow_max_ratio.get() || first_lower > self.shadow_max_ratio.get() {
            return None;
        }
        if second_upper > self.shadow_max_ratio.get() || second_lower > self.shadow_max_ratio.get()
        {
            return None;
        }

        // Third gaps down with long upper shadow into second's body
        if third.open() >= second.close() {
            return None;
        }
        let third_upper = third.upper_shadow_ratio()?;
        if third_upper < 0.3 {
            return None;
        }
        if third.high() < second.close() {
            return None;
        }

        // Fourth engulfs third including shadow
        if fourth.open() < third.high() || fourth.close() > third.low() {
            return None;
        }

        Some(PatternMatch {
            pattern_id: PatternDetector::id(self),
            direction: Direction::Bullish,
            strength: 0.75,
            start_index: index - 3,
            end_index: index,
        })
    }
}

// ============================================================
// HIKKAKE
// ============================================================

/// CDLHIKKAKE - Hikkake Pattern (TA-Lib compatible, two-stage: setup + confirmation)
///
/// TA-Lib's Hikkake is a stateful pattern:
/// - **Setup** (±100): 3-bar pattern — bar\[j-1\] strictly inside bar\[j-2\], bar\[j\] breaks out
/// - **Confirmation** (±200): Within 3 bars after setup, close breaks the inside bar's level
///
/// This implementation simulates TA-Lib's state machine statelessly by looking backward
/// from each bar to find the latest active setup and check for confirmation.
#[derive(Debug, Clone)]
pub struct HikkakeDetector;

impl Default for HikkakeDetector {
    fn default() -> Self {
        Self
    }
}

impl HikkakeDetector {
    /// Check if there's a Hikkake setup completing at bar `j`.
    ///
    /// Setup: bars[j-1] is strictly inside bars[j-2], and bars[j] breaks out.
    /// Returns the direction if setup is found.
    fn is_setup_at<T: OHLCV>(bars: &[T], j: usize) -> Option<Direction> {
        if j < 2 || j >= bars.len() {
            return None;
        }
        let ref_bar = &bars[j - 2];
        let inside = &bars[j - 1];
        let breakout = &bars[j];

        // Inside bar: strictly inside reference (TA-Lib uses strict inequalities)
        if inside.high() >= ref_bar.high() || inside.low() <= ref_bar.low() {
            return None;
        }

        // Breakout direction: both high AND low must shift in the same direction
        if breakout.high() < inside.high() && breakout.low() < inside.low() {
            // Broke down → bullish hikkake (fake bearish breakout)
            Some(Direction::Bullish)
        } else if breakout.high() > inside.high() && breakout.low() > inside.low() {
            // Broke up → bearish hikkake (fake bullish breakout)
            Some(Direction::Bearish)
        } else {
            None
        }
    }
}

impl PatternDetector for HikkakeDetector {
    fn id(&self) -> PatternId {
        PatternId("CDL_HIKKAKE")
    }

    fn min_bars(&self) -> usize {
        // TA-Lib lookback = 5, first output at index 5
        6
    }

    fn detect<T: OHLCV>(
        &self,
        bars: &[T],
        index: usize,
        _ctx: &MarketContext,
    ) -> Option<PatternMatch> {
        if index < 5 {
            return None;
        }

        // Step 1: Check if current bar completes a setup → output ±100
        // Setup takes priority over confirmation (TA-Lib behavior)
        if let Some(dir) = Self::is_setup_at(bars, index) {
            return Some(PatternMatch {
                pattern_id: PatternDetector::id(self),
                direction: dir,
                strength: 0.65,
                start_index: index - 2,
                end_index: index,
            });
        }

        // Step 2: Check for confirmation → output ±200
        // Find the latest active setup within 3 bars back.
        // We search from most recent (delta=1) first — a newer setup overwrites older ones,
        // matching TA-Lib's patternIdx overwrite behavior.
        for delta in 1..=3usize {
            let j = match index.checked_sub(delta) {
                Some(j) if j >= 2 => j,
                _ => continue,
            };

            if let Some(dir) = Self::is_setup_at(bars, j) {
                // Found the latest active setup at bar j.
                // Check if already confirmed at a bar between j+1 and index-1.
                let inside_bar = &bars[j - 1];
                let mut already_confirmed = false;

                for bar in &bars[(j + 1)..index] {
                    let confirmed_here = match dir {
                        Direction::Bullish => bar.close() > inside_bar.high(),
                        Direction::Bearish => bar.close() < inside_bar.low(),
                        _ => false,
                    };
                    if confirmed_here {
                        already_confirmed = true;
                        break;
                    }
                }

                if !already_confirmed {
                    // Check confirmation at current bar
                    let confirmed = match dir {
                        Direction::Bullish => bars[index].close() > inside_bar.high(),
                        Direction::Bearish => bars[index].close() < inside_bar.low(),
                        _ => false,
                    };
                    if confirmed {
                        return Some(PatternMatch {
                            pattern_id: PatternDetector::id(self),
                            direction: dir,
                            strength: 0.75,
                            start_index: j - 2,
                            end_index: index,
                        });
                    }
                }

                break; // Found the latest setup, stop searching
            }
        }

        None
    }
}

/// CDLHIKKAKEMOD - Modified Hikkake Pattern (TA-Lib compatible, two-stage: setup + confirmation)
///
/// TA-Lib's HikkakeMod is a stateful pattern:
/// - **Setup** (±100): 4-bar pattern — two nested inside bars + breakout + close-near-extreme
/// - **Confirmation** (±200): Within 3 bars after setup, close breaks the 3rd bar's level
///
/// This implementation simulates TA-Lib's state machine statelessly by looking backward.
#[derive(Debug, Clone)]
pub struct HikkakeModDetector {
    pub near_factor: f64,
}

impl Default for HikkakeModDetector {
    fn default() -> Self {
        Self {
            near_factor: super::helpers::NEAR_FACTOR,
        }
    }
}

impl HikkakeModDetector {
    /// Check if there's a HikkakeMod setup completing at bar `j`.
    ///
    /// Setup: bars[j-2] strictly inside bars[j-3], bars[j-1] strictly inside bars[j-2],
    /// bars[j-2] close near extreme, bars[j] breaks out.
    fn is_setup_at<T: OHLCV>(bars: &[T], j: usize, near_factor: f64) -> Option<Direction> {
        if j < 3 || j >= bars.len() {
            return None;
        }
        let ref_bar = &bars[j - 3];
        let second = &bars[j - 2]; // Inside bar with close near extreme
        let third = &bars[j - 1]; // Further inside bar
        let breakout = &bars[j];

        // Second bar strictly inside reference
        if second.high() >= ref_bar.high() || second.low() <= ref_bar.low() {
            return None;
        }

        // Third bar strictly inside second
        if third.high() >= second.high() || third.low() <= second.low() {
            return None;
        }

        // Near threshold for close-near-extreme check on second bar
        let near_threshold = super::helpers::trailing_avg_range(bars, j - 2, 5) * near_factor;

        // Breakout direction: both high AND low must shift in the same direction
        if breakout.high() < third.high() && breakout.low() < third.low() {
            // Bullish: broke down → fake bearish breakout
            // Second bar close must be near the LOW
            if second.close() <= second.low() + near_threshold {
                Some(Direction::Bullish)
            } else {
                None
            }
        } else if breakout.high() > third.high() && breakout.low() > third.low() {
            // Bearish: broke up → fake bullish breakout
            // Second bar close must be near the HIGH
            if second.close() >= second.high() - near_threshold {
                Some(Direction::Bearish)
            } else {
                None
            }
        } else {
            None
        }
    }
}

impl PatternDetector for HikkakeModDetector {
    fn id(&self) -> PatternId {
        PatternId("CDL_HIKKAKEMOD")
    }

    fn min_bars(&self) -> usize {
        // Setup needs j >= 3, confirmation can be at j+3.
        // Most distant lookback: setup at index-3 uses bars[index-6].
        // Need index >= 6, so min_bars = 7.
        7
    }

    fn detect<T: OHLCV>(
        &self,
        bars: &[T],
        index: usize,
        _ctx: &MarketContext,
    ) -> Option<PatternMatch> {
        if index < 6 {
            return None;
        }

        // Step 1: Check if current bar completes a setup → output ±100
        if let Some(dir) = Self::is_setup_at(bars, index, self.near_factor) {
            return Some(PatternMatch {
                pattern_id: PatternDetector::id(self),
                direction: dir,
                strength: 0.7,
                start_index: index - 3,
                end_index: index,
            });
        }

        // Step 2: Check for confirmation → output ±200
        // Find the latest active setup within 3 bars back.
        for delta in 1..=3usize {
            let j = match index.checked_sub(delta) {
                Some(j) if j >= 3 => j,
                _ => continue,
            };

            if let Some(dir) = Self::is_setup_at(bars, j, self.near_factor) {
                // Found the latest active setup at bar j.
                // Confirmation reference: bars[j-1] (the 3rd bar / further inside bar)
                let ref_bar = &bars[j - 1];
                let mut already_confirmed = false;

                for bar in &bars[(j + 1)..index] {
                    let confirmed_here = match dir {
                        Direction::Bullish => bar.close() > ref_bar.high(),
                        Direction::Bearish => bar.close() < ref_bar.low(),
                        _ => false,
                    };
                    if confirmed_here {
                        already_confirmed = true;
                        break;
                    }
                }

                if !already_confirmed {
                    let confirmed = match dir {
                        Direction::Bullish => bars[index].close() > ref_bar.high(),
                        Direction::Bearish => bars[index].close() < ref_bar.low(),
                        _ => false,
                    };
                    if confirmed {
                        return Some(PatternMatch {
                            pattern_id: PatternDetector::id(self),
                            direction: dir,
                            strength: 0.8,
                            start_index: j - 3,
                            end_index: index,
                        });
                    }
                }

                break; // Found the latest setup, stop searching
            }
        }

        None
    }
}

// ============================================================
// LADDER BOTTOM
// ============================================================

/// CDLLADDERBOTTOM - Ladder Bottom (5-bar pattern)
#[derive(Debug, Clone)]
pub struct LadderBottomDetector {
    pub shadow_veryshort_factor: f64,
}

impl Default for LadderBottomDetector {
    fn default() -> Self {
        Self {
            shadow_veryshort_factor: super::helpers::SHADOW_VERYSHORT_FACTOR,
        }
    }
}

impl PatternDetector for LadderBottomDetector {
    fn id(&self) -> PatternId {
        PatternId("CDL_LADDERBOTTOM")
    }

    fn min_bars(&self) -> usize {
        5
    }

    fn detect<T: OHLCV>(
        &self,
        bars: &[T],
        index: usize,
        _ctx: &MarketContext,
    ) -> Option<PatternMatch> {
        if index < 4 {
            return None;
        }
        let first = bars.get(index - 4)?;
        let second = bars.get(index - 3)?;
        let third = bars.get(index - 2)?;
        let fourth = bars.get(index - 1)?;
        let fifth = bars.get(index)?;

        // TA-Lib: First three are black (close < open) with descending opens and closes
        if first.close() >= first.open() {
            return None;
        }
        if second.close() >= second.open() {
            return None;
        }
        if third.close() >= third.open() {
            return None;
        }
        if second.open() >= first.open() || third.open() >= second.open() {
            return None;
        }
        if second.close() >= first.close() || third.close() >= second.close() {
            return None;
        }

        // TA-Lib: Fourth is black (close < open) with upper shadow > ShadowVeryShort
        if fourth.close() >= fourth.open() {
            return None;
        }
        let fourth_upper = fourth.upper_shadow();
        let avg_range_fourth = super::helpers::trailing_avg_range(bars, index - 1, 10);
        let svs_threshold = avg_range_fourth * self.shadow_veryshort_factor;
        if fourth_upper <= svs_threshold {
            return None;
        }

        // TA-Lib: Fifth is white (close >= open), opens above fourth's open, closes above fourth's high
        if fifth.close() < fifth.open() {
            return None;
        }
        if fifth.open() <= fourth.open() {
            return None;
        }
        if fifth.close() <= fourth.high() {
            return None;
        }

        Some(PatternMatch {
            pattern_id: PatternDetector::id(self),
            direction: Direction::Bullish,
            strength: 0.7,
            start_index: index - 4,
            end_index: index,
        })
    }
}

// ============================================================
// MAT HOLD
// ============================================================

/// CDLMATHOLD - Mat Hold (5-bar pattern)
#[derive(Debug, Clone)]
pub struct MatHoldDetector {
    pub body_long_factor: f64,
    pub body_short_factor: f64,
    pub penetration: f64,
}

impl Default for MatHoldDetector {
    fn default() -> Self {
        Self {
            body_long_factor: super::helpers::BODY_LONG_FACTOR,
            body_short_factor: super::helpers::BODY_SHORT_FACTOR,
            penetration: 0.5,
        }
    }
}

impl PatternDetector for MatHoldDetector {
    fn id(&self) -> PatternId {
        PatternId("CDL_MATHOLD")
    }

    fn min_bars(&self) -> usize {
        5
    }

    fn detect<T: OHLCV>(
        &self,
        bars: &[T],
        index: usize,
        _ctx: &MarketContext,
    ) -> Option<PatternMatch> {
        if index < 4 {
            return None;
        }
        let first = bars.get(index - 4)?;
        let second = bars.get(index - 3)?;
        let third = bars.get(index - 2)?;
        let fourth = bars.get(index - 1)?;
        let fifth = bars.get(index)?;

        // TA-Lib: Mat Hold is BULLISH ONLY (no bearish variant)
        // Per-candle trailing averages
        let avg_body_first = super::helpers::trailing_avg_body(bars, index - 4, 10);
        let avg_body_second = super::helpers::trailing_avg_body(bars, index - 3, 10);
        let avg_body_third = super::helpers::trailing_avg_body(bars, index - 2, 10);
        let avg_body_fourth = super::helpers::trailing_avg_body(bars, index - 1, 10);

        // TA-Lib condition 1: first candle BodyLong
        let first_body = first.body();
        if !is_body_long_f(
            first_body,
            avg_body_first,
            first.range(),
            self.body_long_factor,
        ) {
            return None;
        }
        // TA-Lib condition 2-4: 2nd, 3rd, 4th candles BodyShort
        if !is_body_short_f(
            second.body(),
            avg_body_second,
            second.range(),
            self.body_short_factor,
        ) {
            return None;
        }
        if !is_body_short_f(
            third.body(),
            avg_body_third,
            third.range(),
            self.body_short_factor,
        ) {
            return None;
        }
        if !is_body_short_f(
            fourth.body(),
            avg_body_fourth,
            fourth.range(),
            self.body_short_factor,
        ) {
            return None;
        }

        // TA-Lib condition 5: first white (close >= open)
        if first.close() < first.open() {
            return None;
        }
        // TA-Lib condition 6: second black (close < open)
        if second.close() >= second.open() {
            return None;
        }
        // TA-Lib condition 7: fifth white (close >= open)
        if fifth.close() < fifth.open() {
            return None;
        }

        // TA-Lib condition 8: RealBodyGapUp between 2nd and 1st
        // min(open[i-3], close[i-3]) > max(open[i-4], close[i-4])
        let second_body_bottom = second.open().min(second.close());
        let first_body_top = first.open().max(first.close());
        if second_body_bottom <= first_body_top {
            return None;
        }

        let first_close = first.close();
        let penetration = self.penetration;

        // TA-Lib conditions 9-10: 3rd and 4th candle body bottoms < first close (hold within)
        let third_body_bottom = third.open().min(third.close());
        let fourth_body_bottom = fourth.open().min(fourth.close());
        if third_body_bottom >= first_close {
            return None;
        }
        if fourth_body_bottom >= first_close {
            return None;
        }

        // TA-Lib conditions 11-12: reaction days don't penetrate too far
        // min(open, close) > close[i-4] - body[i-4] * penetration
        let penetration_floor = first_close - first_body * penetration;
        if third_body_bottom <= penetration_floor {
            return None;
        }
        if fourth_body_bottom <= penetration_floor {
            return None;
        }

        // TA-Lib conditions 13-14: 2nd to 4th are falling
        // max(close[i-2], open[i-2]) < open[i-3]
        let third_body_top = third.open().max(third.close());
        let fourth_body_top = fourth.open().max(fourth.close());
        if third_body_top >= second.open() {
            return None;
        }
        if fourth_body_top >= third_body_top {
            return None;
        }

        // TA-Lib condition 15: fifth opens above fourth close
        if fifth.open() <= fourth.close() {
            return None;
        }

        // TA-Lib condition 16: fifth closes above highest high of reaction days
        let max_high = second.high().max(third.high()).max(fourth.high());
        if fifth.close() <= max_high {
            return None;
        }

        Some(PatternMatch {
            pattern_id: PatternDetector::id(self),
            direction: Direction::Bullish,
            strength: 0.75,
            start_index: index - 4,
            end_index: index,
        })
    }
}

// ============================================================
// RISING/FALLING THREE METHODS
// ============================================================

/// CDLRISEFALL3METHODS - Rising/Falling Three Methods (5-bar pattern)
#[derive(Debug, Clone)]
pub struct RiseFallThreeMethodsDetector {
    pub body_long_factor: f64,
    pub body_short_factor: f64,
}

impl Default for RiseFallThreeMethodsDetector {
    fn default() -> Self {
        Self {
            body_long_factor: super::helpers::BODY_LONG_FACTOR,
            body_short_factor: super::helpers::BODY_SHORT_FACTOR,
        }
    }
}

impl PatternDetector for RiseFallThreeMethodsDetector {
    fn id(&self) -> PatternId {
        PatternId("CDL_RISEFALL3METHODS")
    }

    fn min_bars(&self) -> usize {
        5
    }

    fn detect<T: OHLCV>(
        &self,
        bars: &[T],
        index: usize,
        _ctx: &MarketContext,
    ) -> Option<PatternMatch> {
        if index < 4 {
            return None;
        }
        let first = bars.get(index - 4)?;
        let second = bars.get(index - 3)?;
        let third = bars.get(index - 2)?;
        let fourth = bars.get(index - 1)?;
        let fifth = bars.get(index)?;

        // Per-candle trailing averages for all positions
        let avg_body_first = super::helpers::trailing_avg_body(bars, index - 4, 10);
        let avg_body_second = super::helpers::trailing_avg_body(bars, index - 3, 10);
        let avg_body_third = super::helpers::trailing_avg_body(bars, index - 2, 10);
        let avg_body_fourth = super::helpers::trailing_avg_body(bars, index - 1, 10);
        let avg_body_fifth = super::helpers::trailing_avg_body(bars, index, 10);

        // TA-Lib: TA_CANDLECOLOR convention (close >= open = white/+1, close < open = black/-1)
        let color_first = if first.close() >= first.open() {
            1_i32
        } else {
            -1
        };
        let color_second = if second.close() >= second.open() {
            1_i32
        } else {
            -1
        };
        let color_third = if third.close() >= third.open() {
            1_i32
        } else {
            -1
        };
        let color_fourth = if fourth.close() >= fourth.open() {
            1_i32
        } else {
            -1
        };
        let color_fifth = if fifth.close() >= fifth.open() {
            1_i32
        } else {
            -1
        };

        // TA-Lib: first and fifth same color, middle three opposite
        if color_first != -color_second {
            return None;
        }
        if color_second != color_third {
            return None;
        }
        if color_third != color_fourth {
            return None;
        }
        if color_fourth != -color_fifth {
            return None;
        }

        // TA-Lib: first candle BodyLong
        let first_body = first.body();
        if !is_body_long_f(
            first_body,
            avg_body_first,
            first.range(),
            self.body_long_factor,
        ) {
            return None;
        }

        // TA-Lib: middle three are BodyShort
        if !is_body_short_f(
            second.body(),
            avg_body_second,
            second.range(),
            self.body_short_factor,
        ) {
            return None;
        }
        if !is_body_short_f(
            third.body(),
            avg_body_third,
            third.range(),
            self.body_short_factor,
        ) {
            return None;
        }
        if !is_body_short_f(
            fourth.body(),
            avg_body_fourth,
            fourth.range(),
            self.body_short_factor,
        ) {
            return None;
        }

        // TA-Lib: fifth candle BodyLong
        if !is_body_long_f(
            fifth.body(),
            avg_body_fifth,
            fifth.range(),
            self.body_long_factor,
        ) {
            return None;
        }

        // TA-Lib: middle bars' bodies contained within first bar's high-low range
        let second_body_lo = second.open().min(second.close());
        let second_body_hi = second.open().max(second.close());
        let third_body_lo = third.open().min(third.close());
        let third_body_hi = third.open().max(third.close());
        let fourth_body_lo = fourth.open().min(fourth.close());
        let fourth_body_hi = fourth.open().max(fourth.close());

        if second_body_lo >= first.high() || second_body_hi <= first.low() {
            return None;
        }
        if third_body_lo >= first.high() || third_body_hi <= first.low() {
            return None;
        }
        if fourth_body_lo >= first.high() || fourth_body_hi <= first.low() {
            return None;
        }

        // TA-Lib: progressive closes — close[i-2] * color < close[i-3] * color, etc.
        // When first is white (color=1): close[i-2] < close[i-3] and close[i-1] < close[i-2]
        // When first is black (color=-1): close[i-2] > close[i-3] and close[i-1] > close[i-2]
        let cf = color_first as f64;
        if third.close() * cf >= second.close() * cf {
            return None;
        }
        if fourth.close() * cf >= third.close() * cf {
            return None;
        }

        // TA-Lib: fifth opens past fourth close, closes past first close
        // open[i] * color > close[i-1] * color
        // close[i] * color > close[i-4] * color
        if fifth.open() * cf <= fourth.close() * cf {
            return None;
        }
        if fifth.close() * cf <= first.close() * cf {
            return None;
        }

        let direction = if color_first == 1 {
            Direction::Bullish
        } else {
            Direction::Bearish
        };

        Some(PatternMatch {
            pattern_id: PatternDetector::id(self),
            direction,
            strength: 0.8,
            start_index: index - 4,
            end_index: index,
        })
    }
}

// ============================================================
// X SIDE GAP THREE METHODS
// ============================================================

/// CDLXSIDEGAP3METHODS - Up/Down-gap side-by-side white lines (4-bar pattern)
#[derive(Debug, Clone)]
pub struct XSideGapThreeMethodsDetector {
    pub tolerance: Ratio,
}

impl Default for XSideGapThreeMethodsDetector {
    fn default() -> Self {
        Self {
            tolerance: Ratio::new_const(0.01),
        }
    }
}

impl PatternDetector for XSideGapThreeMethodsDetector {
    fn id(&self) -> PatternId {
        PatternId("CDL_XSIDEGAP3METHODS")
    }

    fn min_bars(&self) -> usize {
        3
    }

    fn detect<T: OHLCV>(
        &self,
        bars: &[T],
        index: usize,
        _ctx: &MarketContext,
    ) -> Option<PatternMatch> {
        if index < 2 {
            return None;
        }
        let first = bars.get(index - 2)?;
        let second = bars.get(index - 1)?;
        let third = bars.get(index)?;

        // TA-Lib: 1st and 2nd same color, 3rd opposite (TA_CANDLECOLOR: close >= open = white)
        let first_bullish = first.close() >= first.open();
        let second_bullish = second.close() >= second.open();
        let third_bullish = third.close() >= third.open();

        if first_bullish != second_bullish {
            return None;
        }
        if second_bullish == third_bullish {
            return None;
        }

        // TA-Lib: 3rd opens within 2nd real body
        let second_body_hi = second.open().max(second.close());
        let second_body_lo = second.open().min(second.close());
        if third.open() >= second_body_hi || third.open() <= second_body_lo {
            return None;
        }

        // TA-Lib: 3rd closes within 1st real body
        let first_body_hi = first.open().max(first.close());
        let first_body_lo = first.open().min(first.close());
        if third.close() >= first_body_hi || third.close() <= first_body_lo {
            return None;
        }

        // TA-Lib: Gap between 1st and 2nd must match candle color direction
        // RealBodyGapUp: min(open,close) of later > max(open,close) of earlier
        // RealBodyGapDown: max(open,close) of later < min(open,close) of earlier
        let direction = if first_bullish && second_body_lo > first_body_hi {
            // Upside gap with white candles
            Direction::Bullish
        } else if !first_bullish && second_body_hi < first_body_lo {
            // Downside gap with black candles
            Direction::Bearish
        } else {
            return None;
        };

        Some(PatternMatch {
            pattern_id: PatternDetector::id(self),
            direction,
            strength: 0.65,
            start_index: index - 2,
            end_index: index,
        })
    }
}

// ============================================================
// PARAMETERIZED DETECTOR IMPLEMENTATIONS
// ============================================================

static CONCEALING_BABY_SWALLOW_PARAMS: &[ParamMeta] = &[ParamMeta {
    name: "shadow_max_ratio",
    param_type: ParamType::Ratio,
    default: 0.05,
    range: (0.02, 0.1, 0.02),
    description: "Maximum shadow ratio",
}];

static XSIDE_GAP_THREE_METHODS_PARAMS: &[ParamMeta] = &[ParamMeta {
    name: "tolerance",
    param_type: ParamType::Ratio,
    default: 0.01,
    range: (0.005, 0.03, 0.005),
    description: "Gap fill tolerance",
}];

impl ParameterizedDetector for ConcealingBabySwallowDetector {
    fn param_meta() -> &'static [ParamMeta] {
        CONCEALING_BABY_SWALLOW_PARAMS
    }

    fn with_params(params: &HashMap<&str, f64>) -> Result<Self> {
        Ok(Self {
            shadow_max_ratio: get_ratio(params, "shadow_max_ratio", 0.05)?,
        })
    }

    fn pattern_id_str() -> &'static str {
        "CDL_CONCEALBABYSWALL"
    }
}

impl ParameterizedDetector for XSideGapThreeMethodsDetector {
    fn param_meta() -> &'static [ParamMeta] {
        XSIDE_GAP_THREE_METHODS_PARAMS
    }

    fn with_params(params: &HashMap<&str, f64>) -> Result<Self> {
        Ok(Self {
            tolerance: get_ratio(params, "tolerance", 0.01)?,
        })
    }

    fn pattern_id_str() -> &'static str {
        "CDL_XSIDEGAP3METHODS"
    }
}

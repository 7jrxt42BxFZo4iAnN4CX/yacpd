//! Two-bar candlestick pattern detectors (TA-Lib compatible)
//!
//! TA-Lib patterns: CDLENGULFING, CDLHARAMI, CDLHARAMICROSS, CDLDARKCLOUDCOVER,
//! CDLPIERCING, CDLDOJISTAR, CDLCOUNTERATTACK, CDLINNECK, CDLONNECK, CDLTHRUSTING,
//! CDLKICKING, CDLKICKINGBYLENGTH, CDLMATCHINGLOW, CDLHOMINGPIGEON,
//! CDLSEPARATINGLINES, CDLGAPSIDESIDEWHITE

#![allow(
    clippy::collapsible_if,
    clippy::collapsible_else_if,
    clippy::default_constructed_unit_structs
)]

use std::collections::HashMap;

use super::{
    helpers,
    helpers::{is_body_long, is_body_long_f, is_body_short, is_body_short_f, is_doji},
};
use crate::{
    params::{get_ratio, ParamMeta, ParamType, ParameterizedDetector},
    Direction, MarketContext, OHLCVExt, PatternDetector, PatternId, PatternMatch, Ratio, Result,
    OHLCV,
};

impl_with_defaults!(
    EngulfingDetector,
    HaramiDetector,
    HaramiCrossDetector,
    PiercingDetector,
    DarkCloudCoverDetector,
    DojiStarDetector,
    CounterattackDetector,
    InNeckDetector,
    OnNeckDetector,
    ThrustingDetector,
    KickingDetector,
    KickingByLengthDetector,
    MatchingLowDetector,
    HomingPigeonDetector,
    SeparatingLinesDetector,
    GapSideSideWhiteDetector,
);

// ============================================================
// ENGULFING PATTERNS
// ============================================================

/// CDLENGULFING - Engulfing Pattern (bullish and bearish)
#[derive(Debug, Clone)]
pub struct EngulfingDetector {
    pub min_engulf_ratio: Ratio,
}

impl Default for EngulfingDetector {
    fn default() -> Self {
        Self {
            min_engulf_ratio: Ratio::new_const(1.0),
        }
    }
}

impl PatternDetector for EngulfingDetector {
    fn id(&self) -> PatternId {
        PatternId("CDL_ENGULFING")
    }

    fn min_bars(&self) -> usize {
        2
    }

    fn detect<T: OHLCV>(
        &self,
        bars: &[T],
        index: usize,
        _ctx: &MarketContext,
    ) -> Option<PatternMatch> {
        if index < 1 {
            return None;
        }
        let prev = bars.get(index - 1)?;
        let curr = bars.get(index)?;

        // TA-Lib: TA_CANDLECOLOR: close >= open → white(+1), close < open → black(-1)
        let curr_white = curr.close() >= curr.open();
        let curr_black = curr.close() < curr.open();
        let prev_white = prev.close() >= prev.open();
        let prev_black = prev.close() < prev.open();

        // TA-Lib: Bullish engulfing — white engulfs black
        if curr_white && prev_black {
            // TA-Lib uses OR of two sub-cases (at most one end may match):
            // A: close[i] >= open[i-1] && open[i] < close[i-1]
            // B: close[i] > open[i-1] && open[i] <= close[i-1]
            let case_a = curr.close() >= prev.open() && curr.open() < prev.close();
            let case_b = curr.close() > prev.open() && curr.open() <= prev.close();
            if case_a || case_b {
                // TA-Lib: 100 if strictly engulfing both ends, 80 if one end matches
                let strict = curr.open() != prev.close() && curr.close() != prev.open();
                let strength = if strict { 0.7 } else { 0.6 };
                return Some(PatternMatch {
                    pattern_id: PatternDetector::id(self),
                    direction: Direction::Bullish,
                    strength,
                    start_index: index - 1,
                    end_index: index,
                });
            }
        }

        // TA-Lib: Bearish engulfing — black engulfs white
        if curr_black && prev_white {
            // TA-Lib OR sub-cases:
            // A: open[i] >= close[i-1] && close[i] < open[i-1]
            // B: open[i] > close[i-1] && close[i] <= open[i-1]
            let case_a = curr.open() >= prev.close() && curr.close() < prev.open();
            let case_b = curr.open() > prev.close() && curr.close() <= prev.open();
            if case_a || case_b {
                let strict = curr.open() != prev.close() && curr.close() != prev.open();
                let strength = if strict { 0.7 } else { 0.6 };
                return Some(PatternMatch {
                    pattern_id: PatternDetector::id(self),
                    direction: Direction::Bearish,
                    strength,
                    start_index: index - 1,
                    end_index: index,
                });
            }
        }

        None
    }
}

// ============================================================
// HARAMI PATTERNS
// ============================================================

/// CDLHARAMI - Harami Pattern (bullish and bearish)
#[derive(Debug, Clone)]
pub struct HaramiDetector {
    pub max_body_ratio: Ratio,
}

impl Default for HaramiDetector {
    fn default() -> Self {
        Self {
            max_body_ratio: Ratio::new_const(0.5),
        }
    }
}

impl PatternDetector for HaramiDetector {
    fn id(&self) -> PatternId {
        PatternId("CDL_HARAMI")
    }

    fn min_bars(&self) -> usize {
        2
    }

    fn detect<T: OHLCV>(
        &self,
        bars: &[T],
        index: usize,
        ctx: &MarketContext,
    ) -> Option<PatternMatch> {
        if index < 1 {
            return None;
        }
        let prev = bars.get(index - 1)?;
        let curr = bars.get(index)?;

        let prev_body = prev.body();
        let prev_range = prev.range();
        if prev_body <= f64::EPSILON {
            return None;
        }

        // TA-Lib: BodyLong uses avg_body at bar i-1 (not bar i)
        let prev_avg_body = helpers::trailing_avg_body(bars, index - 1, 10);
        if !is_body_long(prev_body, prev_avg_body, prev_range) {
            return None;
        }

        let curr_body = curr.body();
        let curr_range = curr.range();

        // TA-Lib: current bar must have a short body (BodyShort) at bar i
        if !is_body_short(curr_body, ctx.avg_body, curr_range) {
            return None;
        }

        let prev_high = prev.open().max(prev.close());
        let prev_low = prev.open().min(prev.close());
        let curr_high = curr.open().max(curr.close());
        let curr_low = curr.open().min(curr.close());

        // TA-Lib: current body inside previous body (relaxed: one end can match)
        if curr_high > prev_high || curr_low < prev_low {
            return None;
        }

        let direction = if prev.is_bearish() {
            Direction::Bullish
        } else {
            Direction::Bearish
        };

        // TA-Lib: returns 100 if strictly inside, 80 if one end matches
        let strictly_inside = curr_high < prev_high && curr_low > prev_low;
        let strength = if strictly_inside { 0.7 } else { 0.6 };

        Some(PatternMatch {
            pattern_id: PatternDetector::id(self),
            direction,
            strength,
            start_index: index - 1,
            end_index: index,
        })
    }
}

/// CDLHARAMICROSS - Harami Cross (harami with doji)
#[derive(Debug, Clone)]
pub struct HaramiCrossDetector {
    pub doji_body_max_ratio: Ratio,
}

impl Default for HaramiCrossDetector {
    fn default() -> Self {
        Self {
            doji_body_max_ratio: Ratio::new_const(0.1),
        }
    }
}

impl PatternDetector for HaramiCrossDetector {
    fn id(&self) -> PatternId {
        PatternId("CDL_HARAMICROSS")
    }

    fn min_bars(&self) -> usize {
        2
    }

    fn detect<T: OHLCV>(
        &self,
        bars: &[T],
        index: usize,
        ctx: &MarketContext,
    ) -> Option<PatternMatch> {
        if index < 1 {
            return None;
        }
        let prev = bars.get(index - 1)?;
        let curr = bars.get(index)?;

        // Current must be a doji (TA-Lib compatible)
        let curr_body = curr.body();
        let curr_range = curr.range();
        if !is_doji(curr_body, ctx.avg_range, curr_range) {
            return None;
        }

        // TA-Lib: Previous must have a long body (BodyLong at bar i-1)
        let prev_body = prev.body();
        let prev_range = prev.range();
        let prev_avg_body = helpers::trailing_avg_body(bars, index - 1, 10);
        if !is_body_long(prev_body, prev_avg_body, prev_range) {
            return None;
        }

        let prev_high = prev.open().max(prev.close());
        let prev_low = prev.open().min(prev.close());

        // Doji must be inside previous body
        if curr.open() > prev_high || curr.open() < prev_low {
            return None;
        }
        if curr.close() > prev_high || curr.close() < prev_low {
            return None;
        }

        let direction = if prev.is_bearish() {
            Direction::Bullish
        } else {
            Direction::Bearish
        };

        Some(PatternMatch {
            pattern_id: PatternDetector::id(self),
            direction,
            strength: 0.65,
            start_index: index - 1,
            end_index: index,
        })
    }
}

// ============================================================
// PIERCING / DARK CLOUD
// ============================================================

/// CDLPIERCING - Piercing Pattern
#[derive(Debug, Clone)]
pub struct PiercingDetector {
    pub min_pierce_ratio: Ratio,
}

impl Default for PiercingDetector {
    fn default() -> Self {
        Self {
            min_pierce_ratio: Ratio::new_const(0.5),
        }
    }
}

impl PatternDetector for PiercingDetector {
    fn id(&self) -> PatternId {
        PatternId("CDL_PIERCING")
    }

    fn min_bars(&self) -> usize {
        2
    }

    fn detect<T: OHLCV>(
        &self,
        bars: &[T],
        index: usize,
        _ctx: &MarketContext,
    ) -> Option<PatternMatch> {
        if index < 1 {
            return None;
        }
        let prev = bars.get(index - 1)?;
        let curr = bars.get(index)?;

        // TA-Lib: TA_CANDLECOLOR(i-1) == -1 (black: close < open)
        if prev.close() >= prev.open() {
            return None;
        }
        // TA-Lib: TA_CANDLECOLOR(i) == 1 (white: close >= open)
        if curr.close() < curr.open() {
            return None;
        }

        // TA-Lib: first candle must have BodyLong (per-candle trailing avg at i-1)
        let prev_body = prev.body();
        let prev_range = prev.range();
        let prev_avg_body = helpers::trailing_avg_body(bars, index - 1, 10);
        if !is_body_long(prev_body, prev_avg_body, prev_range) {
            return None;
        }

        // TA-Lib: second candle must also have BodyLong (per-candle trailing avg at i)
        let curr_body = curr.body();
        let curr_range = curr.range();
        let curr_avg_body = helpers::trailing_avg_body(bars, index, 10);
        if !is_body_long(curr_body, curr_avg_body, curr_range) {
            return None;
        }

        // TA-Lib: open[i] < low[i-1] (opens below previous LOW)
        if curr.open() >= prev.low() {
            return None;
        }

        // TA-Lib: close[i] < open[i-1] (close stays below previous open — within body)
        if curr.close() >= prev.open() {
            return None;
        }

        // TA-Lib: close[i] > close[i-1] + body[i-1] * 0.5 (closes above midpoint)
        if curr.close() <= prev.close() + prev_body * self.min_pierce_ratio.get() {
            return None;
        }

        Some(PatternMatch {
            pattern_id: PatternDetector::id(self),
            direction: Direction::Bullish,
            strength: 0.7,
            start_index: index - 1,
            end_index: index,
        })
    }
}

/// CDLDARKCLOUDCOVER - Dark Cloud Cover
#[derive(Debug, Clone)]
pub struct DarkCloudCoverDetector {
    pub min_pierce_ratio: Ratio,
}

impl Default for DarkCloudCoverDetector {
    fn default() -> Self {
        Self {
            min_pierce_ratio: Ratio::new_const(0.5),
        }
    }
}

impl PatternDetector for DarkCloudCoverDetector {
    fn id(&self) -> PatternId {
        PatternId("CDL_DARKCLOUDCOVER")
    }

    fn min_bars(&self) -> usize {
        2
    }

    fn detect<T: OHLCV>(
        &self,
        bars: &[T],
        index: usize,
        _ctx: &MarketContext,
    ) -> Option<PatternMatch> {
        if index < 1 {
            return None;
        }
        let prev = bars.get(index - 1)?;
        let curr = bars.get(index)?;

        // TA-Lib: TA_CANDLECOLOR(i-1) == 1 (white: close >= open)
        if prev.close() < prev.open() {
            return None;
        }
        // TA-Lib: TA_CANDLECOLOR(i) == -1 (black: close < open)
        if curr.close() >= curr.open() {
            return None;
        }

        // TA-Lib: first candle must have BodyLong (per-candle trailing avg at i-1)
        let prev_body = prev.body();
        let prev_range = prev.range();
        let prev_avg_body = helpers::trailing_avg_body(bars, index - 1, 10);
        if !is_body_long(prev_body, prev_avg_body, prev_range) {
            return None;
        }

        // TA-Lib: open[i] > high[i-1] (opens above previous HIGH, not just close)
        if curr.open() <= prev.high() {
            return None;
        }

        // TA-Lib: close[i] > open[i-1] (close stays above previous open)
        if curr.close() <= prev.open() {
            return None;
        }

        // TA-Lib: close[i] < close[i-1] - body[i-1] * penetration
        let penetration = self.min_pierce_ratio.get();
        if curr.close() >= prev.close() - prev_body * penetration {
            return None;
        }

        Some(PatternMatch {
            pattern_id: PatternDetector::id(self),
            direction: Direction::Bearish,
            strength: 0.7,
            start_index: index - 1,
            end_index: index,
        })
    }
}

// ============================================================
// DOJI STAR
// ============================================================

/// CDLDOJISTAR - Doji Star
#[derive(Debug, Clone)]
pub struct DojiStarDetector {
    pub doji_body_max_ratio: Ratio,
}

impl Default for DojiStarDetector {
    fn default() -> Self {
        Self {
            doji_body_max_ratio: Ratio::new_const(0.1),
        }
    }
}

impl PatternDetector for DojiStarDetector {
    fn id(&self) -> PatternId {
        PatternId("CDL_DOJISTAR")
    }

    fn min_bars(&self) -> usize {
        2
    }

    fn detect<T: OHLCV>(
        &self,
        bars: &[T],
        index: usize,
        _ctx: &MarketContext,
    ) -> Option<PatternMatch> {
        if index < 1 {
            return None;
        }
        let prev = bars.get(index - 1)?;
        let curr = bars.get(index)?;

        // Current must be a doji (TA-Lib: BodyDoji at bar i, per-candle trailing avg_range)
        let curr_body = curr.body();
        let curr_range = curr.range();
        let curr_avg_range = helpers::trailing_avg_range(bars, index, 10);
        if !is_doji(curr_body, curr_avg_range, curr_range) {
            return None;
        }

        // Previous should be a long body (TA-Lib: BodyLong at bar i-1, per-candle trailing avg_body)
        let prev_body = prev.body();
        let prev_range = prev.range();
        let prev_avg_body = helpers::trailing_avg_body(bars, index - 1, 10);
        if !is_body_long(prev_body, prev_avg_body, prev_range) {
            return None;
        }

        // TA-Lib: gap required using TA_REALBODYGAPUP/DOWN (real body gap, not shadow gap)
        // TA_CANDLECOLOR: close >= open → white(+1), close < open → black(-1)
        let prev_white = prev.close() >= prev.open();
        let prev_black = prev.close() < prev.open();

        let prev_body_top = prev.open().max(prev.close());
        let prev_body_bottom = prev.open().min(prev.close());
        let curr_body_top = curr.open().max(curr.close());
        let curr_body_bottom = curr.open().min(curr.close());

        let direction = if prev_white {
            // Bearish doji star: prev white, gap up
            // TA_REALBODYGAPUP(i, i-1): min(open[i],close[i]) > max(open[i-1],close[i-1])
            if curr_body_bottom <= prev_body_top {
                return None;
            }
            Direction::Bearish
        } else if prev_black {
            // Bullish doji star: prev black, gap down
            // TA_REALBODYGAPDOWN(i, i-1): max(open[i],close[i]) < min(open[i-1],close[i-1])
            if curr_body_top >= prev_body_bottom {
                return None;
            }
            Direction::Bullish
        } else {
            return None;
        };

        Some(PatternMatch {
            pattern_id: PatternDetector::id(self),
            direction,
            strength: 0.6,
            start_index: index - 1,
            end_index: index,
        })
    }
}

// ============================================================
// COUNTERATTACK
// ============================================================

/// CDLCOUNTERATTACK - Counterattack
#[derive(Debug, Clone)]
pub struct CounterattackDetector {
    pub close_tolerance: Ratio,
}

impl Default for CounterattackDetector {
    fn default() -> Self {
        Self {
            close_tolerance: Ratio::new_const(0.01),
        }
    }
}

impl PatternDetector for CounterattackDetector {
    fn id(&self) -> PatternId {
        PatternId("CDL_COUNTERATTACK")
    }

    fn min_bars(&self) -> usize {
        2
    }

    fn detect<T: OHLCV>(
        &self,
        bars: &[T],
        index: usize,
        _ctx: &MarketContext,
    ) -> Option<PatternMatch> {
        if index < 1 {
            return None;
        }
        let prev = bars.get(index - 1)?;
        let curr = bars.get(index)?;

        // TA-Lib: opposite colors
        if prev.is_bullish() == curr.is_bullish() {
            return None;
        }

        // TA-Lib: both candles must have long bodies (BodyLong, per-candle trailing)
        let prev_body = prev.body();
        let prev_range = prev.range();
        let curr_body = curr.body();
        let curr_range = curr.range();
        let prev_avg_body = helpers::trailing_avg_body(bars, index - 1, 10);
        let curr_avg_body = helpers::trailing_avg_body(bars, index, 10);
        if !is_body_long(prev_body, prev_avg_body, prev_range) {
            return None;
        }
        if !is_body_long(curr_body, curr_avg_body, curr_range) {
            return None;
        }

        // TA-Lib: closes equal within Equal threshold (HighLow, Period=5, Factor=0.05, per-candle at i-1)
        let equal_threshold =
            helpers::trailing_avg_range(bars, index - 1, 5) * helpers::EQUAL_FACTOR;
        let close_diff = (prev.close() - curr.close()).abs();
        if close_diff > equal_threshold {
            return None;
        }

        // TA-Lib: direction based on current candle color
        let direction = if curr.is_bullish() {
            Direction::Bullish
        } else {
            Direction::Bearish
        };

        Some(PatternMatch {
            pattern_id: PatternDetector::id(self),
            direction,
            strength: 0.6,
            start_index: index - 1,
            end_index: index,
        })
    }
}

// ============================================================
// NECK PATTERNS
// ============================================================

/// CDLINNECK - In-Neck Pattern
#[derive(Debug, Clone)]
pub struct InNeckDetector {
    pub tolerance: Ratio,
}

impl Default for InNeckDetector {
    fn default() -> Self {
        Self {
            tolerance: Ratio::new_const(0.03),
        }
    }
}

impl PatternDetector for InNeckDetector {
    fn id(&self) -> PatternId {
        PatternId("CDL_INNECK")
    }

    fn min_bars(&self) -> usize {
        2
    }

    fn detect<T: OHLCV>(
        &self,
        bars: &[T],
        index: usize,
        _ctx: &MarketContext,
    ) -> Option<PatternMatch> {
        if index < 1 {
            return None;
        }
        let prev = bars.get(index - 1)?;
        let curr = bars.get(index)?;

        // TA-Lib: 1st black (close < open), 2nd white (close >= open)
        if prev.close() >= prev.open() {
            return None;
        }
        if curr.close() < curr.open() {
            return None;
        }

        // TA-Lib: 1st candle must have BodyLong (per-candle trailing avg at i-1)
        let prev_body = prev.body();
        let prev_range = prev.range();
        let prev_avg_body = helpers::trailing_avg_body(bars, index - 1, 10);
        if !is_body_long(prev_body, prev_avg_body, prev_range) {
            return None;
        }

        // TA-Lib: 2nd opens below 1st low
        if curr.open() >= prev.low() {
            return None;
        }

        // TA-Lib: close[i] <= close[i-1] + Equal_avg AND close[i] >= close[i-1]
        // Equal threshold at i-1 (per-candle trailing)
        let equal_threshold =
            helpers::trailing_avg_range(bars, index - 1, 5) * helpers::EQUAL_FACTOR;
        if curr.close() > prev.close() + equal_threshold {
            return None;
        }
        if curr.close() < prev.close() {
            return None;
        }

        Some(PatternMatch {
            pattern_id: PatternDetector::id(self),
            direction: Direction::Bearish, // Continuation pattern
            strength: 0.5,
            start_index: index - 1,
            end_index: index,
        })
    }
}

/// CDLONNECK - On-Neck Pattern
#[derive(Debug, Clone)]
pub struct OnNeckDetector {
    pub tolerance: Ratio,
}

impl Default for OnNeckDetector {
    fn default() -> Self {
        Self {
            tolerance: Ratio::new_const(0.01),
        }
    }
}

impl PatternDetector for OnNeckDetector {
    fn id(&self) -> PatternId {
        PatternId("CDL_ONNECK")
    }

    fn min_bars(&self) -> usize {
        2
    }

    fn detect<T: OHLCV>(
        &self,
        bars: &[T],
        index: usize,
        _ctx: &MarketContext,
    ) -> Option<PatternMatch> {
        if index < 1 {
            return None;
        }
        let prev = bars.get(index - 1)?;
        let curr = bars.get(index)?;

        // TA-Lib: 1st black (close < open), 2nd white (close >= open)
        if prev.close() >= prev.open() {
            return None;
        }
        if curr.close() < curr.open() {
            return None;
        }

        // TA-Lib: 1st candle must have BodyLong (per-candle trailing avg at i-1)
        let prev_body = prev.body();
        let prev_range = prev.range();
        let prev_avg_body = helpers::trailing_avg_body(bars, index - 1, 10);
        if !is_body_long(prev_body, prev_avg_body, prev_range) {
            return None;
        }

        // TA-Lib: 2nd opens below 1st low
        if curr.open() >= prev.low() {
            return None;
        }

        // TA-Lib: close[i] <= low[i-1] + Equal_avg AND close[i] >= low[i-1] - Equal_avg
        // Equal threshold at i-1 (per-candle trailing)
        let equal_threshold =
            helpers::trailing_avg_range(bars, index - 1, 5) * helpers::EQUAL_FACTOR;
        if curr.close() > prev.low() + equal_threshold {
            return None;
        }
        if curr.close() < prev.low() - equal_threshold {
            return None;
        }

        Some(PatternMatch {
            pattern_id: PatternDetector::id(self),
            direction: Direction::Bearish, // Continuation pattern
            strength: 0.5,
            start_index: index - 1,
            end_index: index,
        })
    }
}

/// CDLTHRUSTING - Thrusting Pattern
#[derive(Debug, Clone)]
pub struct ThrustingDetector {
    pub body_long_factor: f64,
    pub equal_factor: f64,
}

impl Default for ThrustingDetector {
    fn default() -> Self {
        Self {
            body_long_factor: helpers::BODY_LONG_FACTOR,
            equal_factor: helpers::EQUAL_FACTOR,
        }
    }
}

impl PatternDetector for ThrustingDetector {
    fn id(&self) -> PatternId {
        PatternId("CDL_THRUSTING")
    }

    fn min_bars(&self) -> usize {
        2
    }

    fn detect<T: OHLCV>(
        &self,
        bars: &[T],
        index: usize,
        _ctx: &MarketContext,
    ) -> Option<PatternMatch> {
        if index < 1 {
            return None;
        }
        let prev = bars.get(index - 1)?;
        let curr = bars.get(index)?;

        // TA-Lib: 1st black (close < open), 2nd white (close >= open)
        if prev.close() >= prev.open() {
            return None;
        }
        if curr.close() < curr.open() {
            return None;
        }

        // TA-Lib: 1st candle must have BodyLong (per-candle trailing avg at i-1)
        let prev_body = prev.body();
        let prev_range = prev.range();
        let prev_avg_body = helpers::trailing_avg_body(bars, index - 1, 10);
        if !is_body_long_f(prev_body, prev_avg_body, prev_range, self.body_long_factor) {
            return None;
        }

        // TA-Lib: 2nd opens below 1st low
        if curr.open() >= prev.low() {
            return None;
        }

        // TA-Lib: close[i] > close[i-1] + Equal_avg (must be above close+tolerance)
        // Equal threshold at i-1 (per-candle trailing)
        let equal_threshold = helpers::trailing_avg_range(bars, index - 1, 5) * self.equal_factor;
        if curr.close() <= prev.close() + equal_threshold {
            return None;
        }

        // TA-Lib: close[i] <= close[i-1] + body[i-1] * 0.5 (at or below midpoint)
        if curr.close() > prev.close() + prev_body * 0.5 {
            return None;
        }

        Some(PatternMatch {
            pattern_id: PatternDetector::id(self),
            direction: Direction::Bearish, // Continuation pattern
            strength: 0.5,
            start_index: index - 1,
            end_index: index,
        })
    }
}

// ============================================================
// KICKING PATTERNS
// ============================================================

/// CDLKICKING - Kicking Pattern
#[derive(Debug, Clone)]
pub struct KickingDetector {
    pub shadow_max_ratio: Ratio,
}

impl Default for KickingDetector {
    fn default() -> Self {
        Self {
            shadow_max_ratio: Ratio::new_const(0.05),
        }
    }
}

impl PatternDetector for KickingDetector {
    fn id(&self) -> PatternId {
        PatternId("CDL_KICKING")
    }

    fn min_bars(&self) -> usize {
        2
    }

    fn detect<T: OHLCV>(
        &self,
        bars: &[T],
        index: usize,
        _ctx: &MarketContext,
    ) -> Option<PatternMatch> {
        if index < 1 {
            return None;
        }
        let prev = bars.get(index - 1)?;
        let curr = bars.get(index)?;

        // Both should be marubozu (no/minimal shadows)
        let max = self.shadow_max_ratio.get();
        if !super::helpers::is_marubozu(prev, max)? || !super::helpers::is_marubozu(curr, max)? {
            return None;
        }

        // Opposite colors with gap
        let direction = if prev.is_bearish() && curr.is_bullish() {
            // Bullish kicking: gap up
            if curr.low() <= prev.high() {
                return None;
            }
            Direction::Bullish
        } else if prev.is_bullish() && curr.is_bearish() {
            // Bearish kicking: gap down
            if curr.high() >= prev.low() {
                return None;
            }
            Direction::Bearish
        } else {
            return None;
        };

        Some(PatternMatch {
            pattern_id: PatternDetector::id(self),
            direction,
            strength: 0.8,
            start_index: index - 1,
            end_index: index,
        })
    }
}

/// CDLKICKINGBYLENGTH - Kicking by Length
#[derive(Debug, Clone)]
pub struct KickingByLengthDetector {
    pub shadow_max_ratio: Ratio,
}

impl Default for KickingByLengthDetector {
    fn default() -> Self {
        Self {
            shadow_max_ratio: Ratio::new_const(0.05),
        }
    }
}

impl PatternDetector for KickingByLengthDetector {
    fn id(&self) -> PatternId {
        PatternId("CDL_KICKINGBYLENGTH")
    }

    fn min_bars(&self) -> usize {
        2
    }

    fn detect<T: OHLCV>(
        &self,
        bars: &[T],
        index: usize,
        _ctx: &MarketContext,
    ) -> Option<PatternMatch> {
        if index < 1 {
            return None;
        }
        let prev = bars.get(index - 1)?;
        let curr = bars.get(index)?;

        // Both should be marubozu
        let max = self.shadow_max_ratio.get();
        if !super::helpers::is_marubozu(prev, max)? || !super::helpers::is_marubozu(curr, max)? {
            return None;
        }

        // Opposite colors with gap
        if prev.is_bearish() == curr.is_bearish() {
            return None;
        }

        // Gap required
        let has_gap = if curr.is_bullish() {
            curr.low() > prev.high()
        } else {
            curr.high() < prev.low()
        };
        if !has_gap {
            return None;
        }

        // Direction determined by longer marubozu
        let direction = if curr.body() > prev.body() {
            if curr.is_bullish() {
                Direction::Bullish
            } else {
                Direction::Bearish
            }
        } else if prev.is_bullish() {
            Direction::Bullish
        } else {
            Direction::Bearish
        };

        Some(PatternMatch {
            pattern_id: PatternDetector::id(self),
            direction,
            strength: 0.75,
            start_index: index - 1,
            end_index: index,
        })
    }
}

// ============================================================
// MATCHING / HOMING
// ============================================================

/// CDLMATCHINGLOW - Matching Low
#[derive(Debug, Clone)]
pub struct MatchingLowDetector {
    pub tolerance: Ratio,
}

impl Default for MatchingLowDetector {
    fn default() -> Self {
        Self {
            tolerance: Ratio::new_const(0.001),
        }
    }
}

impl PatternDetector for MatchingLowDetector {
    fn id(&self) -> PatternId {
        PatternId("CDL_MATCHINGLOW")
    }

    fn min_bars(&self) -> usize {
        2
    }

    fn detect<T: OHLCV>(
        &self,
        bars: &[T],
        index: usize,
        _ctx: &MarketContext,
    ) -> Option<PatternMatch> {
        if index < 1 {
            return None;
        }
        let prev = bars.get(index - 1)?;
        let curr = bars.get(index)?;

        // TA-Lib: both bearish
        if !prev.is_bearish() || !curr.is_bearish() {
            return None;
        }

        // TA-Lib: closes equal within Equal threshold (HighLow, Period=5, Factor=0.05)
        // Per-candle trailing average at i-1 (the bar whose close we're comparing)
        let equal_threshold =
            helpers::trailing_avg_range(bars, index - 1, 5) * helpers::EQUAL_FACTOR;
        let diff = (prev.close() - curr.close()).abs();
        if diff > equal_threshold {
            return None;
        }

        Some(PatternMatch {
            pattern_id: PatternDetector::id(self),
            direction: Direction::Bullish,
            strength: 0.55,
            start_index: index - 1,
            end_index: index,
        })
    }
}

/// CDLHOMINGPIGEON - Homing Pigeon
#[derive(Debug, Clone)]
pub struct HomingPigeonDetector {
    pub body_long_factor: f64,
    pub body_short_factor: f64,
}

impl Default for HomingPigeonDetector {
    fn default() -> Self {
        Self {
            body_long_factor: helpers::BODY_LONG_FACTOR,
            body_short_factor: helpers::BODY_SHORT_FACTOR,
        }
    }
}

impl PatternDetector for HomingPigeonDetector {
    fn id(&self) -> PatternId {
        PatternId("CDL_HOMINGPIGEON")
    }

    fn min_bars(&self) -> usize {
        2
    }

    fn detect<T: OHLCV>(
        &self,
        bars: &[T],
        index: usize,
        _ctx: &MarketContext,
    ) -> Option<PatternMatch> {
        if index < 1 {
            return None;
        }
        let prev = bars.get(index - 1)?;
        let curr = bars.get(index)?;

        // TA-Lib: both bearish (black)
        if !prev.is_bearish() || !curr.is_bearish() {
            return None;
        }

        // TA-Lib: 1st candle must have BodyLong (per-candle trailing avg at i-1)
        let prev_body = prev.body();
        let prev_range = prev.range();
        let prev_avg_body = helpers::trailing_avg_body(bars, index - 1, 10);
        if !is_body_long_f(prev_body, prev_avg_body, prev_range, self.body_long_factor) {
            return None;
        }

        // TA-Lib: 2nd candle must have BodyShort (per-candle trailing avg at i)
        let curr_body = curr.body();
        let curr_range = curr.range();
        let curr_avg_body = helpers::trailing_avg_body(bars, index, 10);
        if !is_body_short_f(curr_body, curr_avg_body, curr_range, self.body_short_factor) {
            return None;
        }

        // TA-Lib: 2nd open < 1st open AND 2nd close > 1st close (2nd body inside 1st body)
        if curr.open() >= prev.open() {
            return None;
        }
        if curr.close() <= prev.close() {
            return None;
        }

        Some(PatternMatch {
            pattern_id: PatternDetector::id(self),
            direction: Direction::Bullish,
            strength: 0.55,
            start_index: index - 1,
            end_index: index,
        })
    }
}

// ============================================================
// SEPARATING / GAP PATTERNS
// ============================================================

/// CDLSEPARATINGLINES - Separating Lines
#[derive(Debug, Clone)]
pub struct SeparatingLinesDetector {
    pub tolerance: Ratio,
}

impl Default for SeparatingLinesDetector {
    fn default() -> Self {
        Self {
            tolerance: Ratio::new_const(0.005),
        }
    }
}

impl PatternDetector for SeparatingLinesDetector {
    fn id(&self) -> PatternId {
        PatternId("CDL_SEPARATINGLINES")
    }

    fn min_bars(&self) -> usize {
        2
    }

    fn detect<T: OHLCV>(
        &self,
        bars: &[T],
        index: usize,
        _ctx: &MarketContext,
    ) -> Option<PatternMatch> {
        if index < 1 {
            return None;
        }
        let prev = bars.get(index - 1)?;
        let curr = bars.get(index)?;

        // TA-Lib: TA_CANDLECOLOR(i-1) == -TA_CANDLECOLOR(i) (opposite colors)
        // TA_CANDLECOLOR: close >= open → 1 (white), close < open → -1 (black)
        let curr_white = curr.close() >= curr.open();
        let prev_white = prev.close() >= prev.open();
        if curr_white == prev_white {
            return None;
        }

        // TA-Lib: Opens at same level (Equal: HighLow, Period=5, Factor=0.05, per-candle at i-1)
        // open[i] <= open[i-1] + Equal_avg AND open[i] >= open[i-1] - Equal_avg
        let equal_threshold =
            helpers::trailing_avg_range(bars, index - 1, 5) * helpers::EQUAL_FACTOR;
        if curr.open() > prev.open() + equal_threshold
            || curr.open() < prev.open() - equal_threshold
        {
            return None;
        }

        // TA-Lib: 2nd candle must have long body (per-candle trailing avg at i)
        let curr_body = curr.body();
        let curr_range = curr.range();
        let curr_avg_body = helpers::trailing_avg_body(bars, index, 10);
        if !helpers::is_body_long(curr_body, curr_avg_body, curr_range) {
            return None;
        }

        // TA-Lib: 2nd candle's opening shadow must be very short (per-candle trailing avg at i)
        // Bullish (white): lower shadow. Bearish (black): upper shadow
        let opening_shadow = if curr_white {
            curr.lower_shadow()
        } else {
            curr.upper_shadow()
        };
        let curr_avg_range = helpers::trailing_avg_range(bars, index, 10);
        if !helpers::is_shadow_very_short(opening_shadow, curr_avg_range, curr_range) {
            return None;
        }

        // Direction based on 2nd candle color
        let direction = if curr_white {
            Direction::Bullish
        } else {
            Direction::Bearish
        };

        Some(PatternMatch {
            pattern_id: PatternDetector::id(self),
            direction,
            strength: 0.55,
            start_index: index - 1,
            end_index: index,
        })
    }
}

/// CDLGAPSIDESIDEWHITE - Gap Side-by-Side White Lines
#[derive(Debug, Clone)]
pub struct GapSideSideWhiteDetector {
    pub tolerance: Ratio,
}

impl Default for GapSideSideWhiteDetector {
    fn default() -> Self {
        Self {
            tolerance: Ratio::new_const(0.01),
        }
    }
}

impl PatternDetector for GapSideSideWhiteDetector {
    fn id(&self) -> PatternId {
        PatternId("CDL_GAPSIDESIDEWHITE")
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

        // TA-Lib: 2nd and 3rd both white (close >= open)
        let second_white = second.close() >= second.open();
        let third_white = third.close() >= third.open();
        if !second_white || !third_white {
            return None;
        }

        // TA-Lib: Gap direction — both 2nd and 3rd must have real body gap from 1st
        let first_body_hi = first.open().max(first.close());
        let first_body_lo = first.open().min(first.close());
        let second_body_lo = second.open().min(second.close());
        let second_body_hi = second.open().max(second.close());
        let third_body_lo = third.open().min(third.close());
        let third_body_hi = third.open().max(third.close());

        let gap_up = second_body_lo > first_body_hi && third_body_lo > first_body_hi;
        let gap_down = second_body_hi < first_body_lo && third_body_hi < first_body_lo;

        // TA-Lib: direction comes from gap direction only (no first candle color check)
        if !gap_up && !gap_down {
            return None;
        }
        let direction = if gap_up {
            Direction::Bullish
        } else {
            Direction::Bearish
        };

        // TA-Lib: bodies are similar size (Near threshold at i-1)
        let near_threshold = helpers::trailing_avg_range(bars, index - 1, 5) * helpers::NEAR_FACTOR;
        let second_body = second.body();
        let third_body = third.body();
        if third_body < second_body - near_threshold {
            return None;
        }
        if third_body > second_body + near_threshold {
            return None;
        }

        // TA-Lib: opens are similar (Equal threshold at i-1)
        let equal_threshold =
            helpers::trailing_avg_range(bars, index - 1, 5) * helpers::EQUAL_FACTOR;
        if third.open() < second.open() - equal_threshold {
            return None;
        }
        if third.open() > second.open() + equal_threshold {
            return None;
        }

        Some(PatternMatch {
            pattern_id: PatternDetector::id(self),
            direction,
            strength: 0.6,
            start_index: index - 2,
            end_index: index,
        })
    }
}

// ============================================================
// PARAMETERIZED DETECTOR IMPLEMENTATIONS
// ============================================================

// Static parameter metadata definitions
static ENGULFING_PARAMS: &[ParamMeta] = &[ParamMeta {
    name: "min_engulf_ratio",
    param_type: ParamType::Ratio,
    default: 1.0,
    range: (0.8, 1.2, 0.1),
    description: "Minimum engulfing body ratio",
}];

static HARAMI_PARAMS: &[ParamMeta] = &[ParamMeta {
    name: "max_body_ratio",
    param_type: ParamType::Ratio,
    default: 0.5,
    range: (0.3, 0.7, 0.1),
    description: "Maximum inside body ratio",
}];

static HARAMICROSS_PARAMS: &[ParamMeta] = &[ParamMeta {
    name: "doji_body_max_ratio",
    param_type: ParamType::Ratio,
    default: 0.1,
    range: (0.05, 0.2, 0.05),
    description: "Maximum doji body ratio",
}];

static PIERCING_PARAMS: &[ParamMeta] = &[ParamMeta {
    name: "min_pierce_ratio",
    param_type: ParamType::Ratio,
    default: 0.5,
    range: (0.4, 0.7, 0.1),
    description: "Minimum piercing level",
}];

static DARKCLOUDCOVER_PARAMS: &[ParamMeta] = &[ParamMeta {
    name: "min_pierce_ratio",
    param_type: ParamType::Ratio,
    default: 0.5,
    range: (0.4, 0.7, 0.1),
    description: "Minimum piercing level",
}];

static DOJISTAR_PARAMS: &[ParamMeta] = &[ParamMeta {
    name: "doji_body_max_ratio",
    param_type: ParamType::Ratio,
    default: 0.1,
    range: (0.05, 0.2, 0.05),
    description: "Maximum doji body ratio",
}];

static COUNTERATTACK_PARAMS: &[ParamMeta] = &[ParamMeta {
    name: "close_tolerance",
    param_type: ParamType::Ratio,
    default: 0.01,
    range: (0.005, 0.03, 0.005),
    description: "Close price tolerance",
}];

static INNECK_PARAMS: &[ParamMeta] = &[ParamMeta {
    name: "tolerance",
    param_type: ParamType::Ratio,
    default: 0.01,
    range: (0.005, 0.03, 0.005),
    description: "Price tolerance",
}];

static ONNECK_PARAMS: &[ParamMeta] = &[ParamMeta {
    name: "tolerance",
    param_type: ParamType::Ratio,
    default: 0.01,
    range: (0.005, 0.03, 0.005),
    description: "Price tolerance",
}];

static KICKING_PARAMS: &[ParamMeta] = &[ParamMeta {
    name: "shadow_max_ratio",
    param_type: ParamType::Ratio,
    default: 0.05,
    range: (0.02, 0.1, 0.02),
    description: "Maximum shadow ratio for marubozu",
}];

static KICKINGBYLENGTH_PARAMS: &[ParamMeta] = &[ParamMeta {
    name: "shadow_max_ratio",
    param_type: ParamType::Ratio,
    default: 0.05,
    range: (0.02, 0.1, 0.02),
    description: "Maximum shadow ratio for marubozu",
}];

static MATCHINGLOW_PARAMS: &[ParamMeta] = &[ParamMeta {
    name: "tolerance",
    param_type: ParamType::Ratio,
    default: 0.001,
    range: (0.0005, 0.003, 0.0005),
    description: "Price matching tolerance",
}];

static SEPARATINGLINES_PARAMS: &[ParamMeta] = &[ParamMeta {
    name: "tolerance",
    param_type: ParamType::Ratio,
    default: 0.005,
    range: (0.002, 0.01, 0.002),
    description: "Open price tolerance",
}];

static GAPSIDESIDEWHITE_PARAMS: &[ParamMeta] = &[ParamMeta {
    name: "tolerance",
    param_type: ParamType::Ratio,
    default: 0.01,
    range: (0.005, 0.03, 0.005),
    description: "Size tolerance",
}];

impl ParameterizedDetector for EngulfingDetector {
    fn param_meta() -> &'static [ParamMeta] {
        ENGULFING_PARAMS
    }

    fn with_params(params: &HashMap<&str, f64>) -> Result<Self> {
        Ok(Self {
            min_engulf_ratio: get_ratio(params, "min_engulf_ratio", 1.0)?,
        })
    }

    fn pattern_id_str() -> &'static str {
        "CDL_ENGULFING"
    }
}

impl ParameterizedDetector for HaramiDetector {
    fn param_meta() -> &'static [ParamMeta] {
        HARAMI_PARAMS
    }

    fn with_params(params: &HashMap<&str, f64>) -> Result<Self> {
        Ok(Self {
            max_body_ratio: get_ratio(params, "max_body_ratio", 0.5)?,
        })
    }

    fn pattern_id_str() -> &'static str {
        "CDL_HARAMI"
    }
}

impl ParameterizedDetector for HaramiCrossDetector {
    fn param_meta() -> &'static [ParamMeta] {
        HARAMICROSS_PARAMS
    }

    fn with_params(params: &HashMap<&str, f64>) -> Result<Self> {
        Ok(Self {
            doji_body_max_ratio: get_ratio(params, "doji_body_max_ratio", 0.1)?,
        })
    }

    fn pattern_id_str() -> &'static str {
        "CDL_HARAMICROSS"
    }
}

impl ParameterizedDetector for PiercingDetector {
    fn param_meta() -> &'static [ParamMeta] {
        PIERCING_PARAMS
    }

    fn with_params(params: &HashMap<&str, f64>) -> Result<Self> {
        Ok(Self {
            min_pierce_ratio: get_ratio(params, "min_pierce_ratio", 0.5)?,
        })
    }

    fn pattern_id_str() -> &'static str {
        "CDL_PIERCING"
    }
}

impl ParameterizedDetector for DarkCloudCoverDetector {
    fn param_meta() -> &'static [ParamMeta] {
        DARKCLOUDCOVER_PARAMS
    }

    fn with_params(params: &HashMap<&str, f64>) -> Result<Self> {
        Ok(Self {
            min_pierce_ratio: get_ratio(params, "min_pierce_ratio", 0.5)?,
        })
    }

    fn pattern_id_str() -> &'static str {
        "CDL_DARKCLOUDCOVER"
    }
}

impl ParameterizedDetector for DojiStarDetector {
    fn param_meta() -> &'static [ParamMeta] {
        DOJISTAR_PARAMS
    }

    fn with_params(params: &HashMap<&str, f64>) -> Result<Self> {
        Ok(Self {
            doji_body_max_ratio: get_ratio(params, "doji_body_max_ratio", 0.1)?,
        })
    }

    fn pattern_id_str() -> &'static str {
        "CDL_DOJISTAR"
    }
}

impl ParameterizedDetector for CounterattackDetector {
    fn param_meta() -> &'static [ParamMeta] {
        COUNTERATTACK_PARAMS
    }

    fn with_params(params: &HashMap<&str, f64>) -> Result<Self> {
        Ok(Self {
            close_tolerance: get_ratio(params, "close_tolerance", 0.01)?,
        })
    }

    fn pattern_id_str() -> &'static str {
        "CDL_COUNTERATTACK"
    }
}

impl ParameterizedDetector for InNeckDetector {
    fn param_meta() -> &'static [ParamMeta] {
        INNECK_PARAMS
    }

    fn with_params(params: &HashMap<&str, f64>) -> Result<Self> {
        Ok(Self {
            tolerance: get_ratio(params, "tolerance", 0.01)?,
        })
    }

    fn pattern_id_str() -> &'static str {
        "CDL_INNECK"
    }
}

impl ParameterizedDetector for OnNeckDetector {
    fn param_meta() -> &'static [ParamMeta] {
        ONNECK_PARAMS
    }

    fn with_params(params: &HashMap<&str, f64>) -> Result<Self> {
        Ok(Self {
            tolerance: get_ratio(params, "tolerance", 0.01)?,
        })
    }

    fn pattern_id_str() -> &'static str {
        "CDL_ONNECK"
    }
}

impl ParameterizedDetector for KickingDetector {
    fn param_meta() -> &'static [ParamMeta] {
        KICKING_PARAMS
    }

    fn with_params(params: &HashMap<&str, f64>) -> Result<Self> {
        Ok(Self {
            shadow_max_ratio: get_ratio(params, "shadow_max_ratio", 0.05)?,
        })
    }

    fn pattern_id_str() -> &'static str {
        "CDL_KICKING"
    }
}

impl ParameterizedDetector for KickingByLengthDetector {
    fn param_meta() -> &'static [ParamMeta] {
        KICKINGBYLENGTH_PARAMS
    }

    fn with_params(params: &HashMap<&str, f64>) -> Result<Self> {
        Ok(Self {
            shadow_max_ratio: get_ratio(params, "shadow_max_ratio", 0.05)?,
        })
    }

    fn pattern_id_str() -> &'static str {
        "CDL_KICKINGBYLENGTH"
    }
}

impl ParameterizedDetector for MatchingLowDetector {
    fn param_meta() -> &'static [ParamMeta] {
        MATCHINGLOW_PARAMS
    }

    fn with_params(params: &HashMap<&str, f64>) -> Result<Self> {
        Ok(Self {
            tolerance: get_ratio(params, "tolerance", 0.001)?,
        })
    }

    fn pattern_id_str() -> &'static str {
        "CDL_MATCHINGLOW"
    }
}

impl ParameterizedDetector for SeparatingLinesDetector {
    fn param_meta() -> &'static [ParamMeta] {
        SEPARATINGLINES_PARAMS
    }

    fn with_params(params: &HashMap<&str, f64>) -> Result<Self> {
        Ok(Self {
            tolerance: get_ratio(params, "tolerance", 0.005)?,
        })
    }

    fn pattern_id_str() -> &'static str {
        "CDL_SEPARATINGLINES"
    }
}

impl ParameterizedDetector for GapSideSideWhiteDetector {
    fn param_meta() -> &'static [ParamMeta] {
        GAPSIDESIDEWHITE_PARAMS
    }

    fn with_params(params: &HashMap<&str, f64>) -> Result<Self> {
        Ok(Self {
            tolerance: get_ratio(params, "tolerance", 0.01)?,
        })
    }

    fn pattern_id_str() -> &'static str {
        "CDL_GAPSIDESIDEWHITE"
    }
}

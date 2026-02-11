//! Single-bar candlestick pattern detectors (TA-Lib compatible)
//!
//! TA-Lib patterns: CDLDOJI, CDLDRAGONFLYDOJI, CDLGRAVESTONEDOJI, CDLLONGLEGGEDDOJI,
//! CDLHAMMER, CDLHANGINGMAN, CDLINVERTEDHAMMER, CDLSHOOTINGSTAR, CDLMARUBOZU,
//! CDLCLOSINGMARUBOZU, CDLSPINNINGTOP, CDLHIGHWAVE, CDLLONGLINE, CDLSHORTLINE,
//! CDLBELTHOLD, CDLTAKURI, CDLRICKSHAWMAN
//!
//! All patterns use TA-Lib style comparisons with average body/shadow over lookback period.

#![allow(clippy::collapsible_if, clippy::collapsible_else_if)]

use super::helpers::{
    self, is_body_long_f, is_body_short_f, is_doji_f, is_shadow_long, is_shadow_short,
    is_shadow_very_short_f, is_shadow_verylong_f, shadow_exceeds_veryshort,
};
use crate::{Direction, MarketContext, OHLCVExt, PatternDetector, PatternId, PatternMatch, OHLCV};

mod talib {
    pub use super::super::helpers::DOJI_RATIO;
}

impl_with_defaults!(
    DojiDetector,
    DragonflyDojiDetector,
    GravestoneDojiDetector,
    LongLeggedDojiDetector,
    RickshawManDetector,
    HammerDetector,
    HangingManDetector,
    InvertedHammerDetector,
    ShootingStarDetector,
    TakuriDetector,
    MarubozuDetector,
    ClosingMarubozuDetector,
    LongLineDetector,
    ShortLineDetector,
    SpinningTopDetector,
    HighWaveDetector,
    BeltHoldDetector,
);

// ============================================================
// DOJI FAMILY
// ============================================================

/// CDLDOJI - Doji (TA-Lib compatible)
#[derive(Debug, Clone, Copy)]
pub struct DojiDetector {
    pub doji_factor: f64,
}

impl Default for DojiDetector {
    fn default() -> Self {
        Self {
            doji_factor: helpers::DOJI_FACTOR,
        }
    }
}

impl PatternDetector for DojiDetector {
    fn id(&self) -> PatternId {
        PatternId("CDL_DOJI")
    }

    fn min_bars(&self) -> usize {
        1
    }

    fn detect<T: OHLCV>(
        &self,
        bars: &[T],
        index: usize,
        ctx: &MarketContext,
    ) -> Option<PatternMatch> {
        let bar = bars.get(index)?;
        let body = bar.body();
        let range = bar.range();

        if !is_doji_f(body, ctx.avg_range, range, self.doji_factor) {
            return None;
        }

        let strength = if range > 0.0 {
            1.0 - (body / range / talib::DOJI_RATIO).min(1.0)
        } else {
            0.5
        };

        Some(PatternMatch {
            pattern_id: PatternDetector::id(self),
            direction: Direction::Neutral,
            strength: 0.5 + strength * 0.5,
            start_index: index,
            end_index: index,
        })
    }
}

/// CDLDRAGONFLYDOJI - Dragonfly Doji (TA-Lib compatible)
#[derive(Debug, Clone, Copy)]
pub struct DragonflyDojiDetector {
    pub doji_factor: f64,
    pub shadow_veryshort_factor: f64,
}

impl Default for DragonflyDojiDetector {
    fn default() -> Self {
        Self {
            doji_factor: helpers::DOJI_FACTOR,
            shadow_veryshort_factor: helpers::SHADOW_VERYSHORT_FACTOR,
        }
    }
}

impl PatternDetector for DragonflyDojiDetector {
    fn id(&self) -> PatternId {
        PatternId("CDL_DRAGONFLYDOJI")
    }

    fn min_bars(&self) -> usize {
        1
    }

    fn detect<T: OHLCV>(
        &self,
        bars: &[T],
        index: usize,
        ctx: &MarketContext,
    ) -> Option<PatternMatch> {
        let bar = bars.get(index)?;
        let body = bar.body();
        let upper = bar.upper_shadow();
        let lower = bar.lower_shadow();
        let range = bar.range();

        if !is_doji_f(body, ctx.avg_range, range, self.doji_factor) {
            return None;
        }
        // TA-Lib: upper shadow must be very short (< ShadowVeryShort)
        if !is_shadow_very_short_f(upper, ctx.avg_range, range, self.shadow_veryshort_factor) {
            return None;
        }
        // TA-Lib: lower shadow must NOT be very short (> ShadowVeryShort)
        // Note: TA-Lib does NOT use ShadowVeryLong here — just checks shadow > ShadowVeryShort threshold
        if !shadow_exceeds_veryshort(lower, ctx.avg_range, self.shadow_veryshort_factor, range) {
            return None;
        }

        Some(PatternMatch {
            pattern_id: PatternDetector::id(self),
            direction: Direction::Bullish,
            strength: 0.7,
            start_index: index,
            end_index: index,
        })
    }
}

/// CDLGRAVESTONEDOJI - Gravestone Doji (TA-Lib compatible)
#[derive(Debug, Clone, Copy)]
pub struct GravestoneDojiDetector {
    pub doji_factor: f64,
    pub shadow_veryshort_factor: f64,
}

impl Default for GravestoneDojiDetector {
    fn default() -> Self {
        Self {
            doji_factor: helpers::DOJI_FACTOR,
            shadow_veryshort_factor: helpers::SHADOW_VERYSHORT_FACTOR,
        }
    }
}

impl PatternDetector for GravestoneDojiDetector {
    fn id(&self) -> PatternId {
        PatternId("CDL_GRAVESTONEDOJI")
    }

    fn min_bars(&self) -> usize {
        1
    }

    fn detect<T: OHLCV>(
        &self,
        bars: &[T],
        index: usize,
        ctx: &MarketContext,
    ) -> Option<PatternMatch> {
        let bar = bars.get(index)?;
        let body = bar.body();
        let upper = bar.upper_shadow();
        let lower = bar.lower_shadow();
        let range = bar.range();

        if !is_doji_f(body, ctx.avg_range, range, self.doji_factor) {
            return None;
        }
        // TA-Lib: lower shadow must be very short (< ShadowVeryShort)
        if !is_shadow_very_short_f(lower, ctx.avg_range, range, self.shadow_veryshort_factor) {
            return None;
        }
        // TA-Lib: upper shadow must NOT be very short (> ShadowVeryShort)
        if !shadow_exceeds_veryshort(upper, ctx.avg_range, self.shadow_veryshort_factor, range) {
            return None;
        }

        Some(PatternMatch {
            pattern_id: PatternDetector::id(self),
            direction: Direction::Neutral,
            strength: 0.7,
            start_index: index,
            end_index: index,
        })
    }
}

/// CDLLONGLEGGEDDOJI - Long Legged Doji (TA-Lib compatible)
#[derive(Debug, Clone, Copy)]
pub struct LongLeggedDojiDetector {
    pub doji_factor: f64,
}

impl Default for LongLeggedDojiDetector {
    fn default() -> Self {
        Self {
            doji_factor: helpers::DOJI_FACTOR,
        }
    }
}

impl PatternDetector for LongLeggedDojiDetector {
    fn id(&self) -> PatternId {
        PatternId("CDL_LONGLEGGEDDOJI")
    }

    fn min_bars(&self) -> usize {
        1
    }

    fn detect<T: OHLCV>(
        &self,
        bars: &[T],
        index: usize,
        ctx: &MarketContext,
    ) -> Option<PatternMatch> {
        let bar = bars.get(index)?;
        let body = bar.body();
        let upper = bar.upper_shadow();
        let lower = bar.lower_shadow();
        let range = bar.range();

        if !is_doji_f(body, ctx.avg_range, range, self.doji_factor) {
            return None;
        }
        // TA-Lib: at least ONE shadow must be long (OR, not AND)
        if !is_shadow_long(upper, body, range) && !is_shadow_long(lower, body, range) {
            return None;
        }

        Some(PatternMatch {
            pattern_id: PatternDetector::id(self),
            direction: Direction::Neutral,
            strength: 0.6,
            start_index: index,
            end_index: index,
        })
    }
}

/// CDLRICKSHAWMAN - Rickshaw Man (TA-Lib compatible)
#[derive(Debug, Clone, Copy)]
pub struct RickshawManDetector {
    pub doji_factor: f64,
    pub near_factor: f64,
}

impl Default for RickshawManDetector {
    fn default() -> Self {
        Self {
            doji_factor: helpers::DOJI_FACTOR,
            near_factor: helpers::NEAR_FACTOR,
        }
    }
}

impl PatternDetector for RickshawManDetector {
    fn id(&self) -> PatternId {
        PatternId("CDL_RICKSHAWMAN")
    }

    fn min_bars(&self) -> usize {
        1
    }

    fn detect<T: OHLCV>(
        &self,
        bars: &[T],
        index: usize,
        _ctx: &MarketContext,
    ) -> Option<PatternMatch> {
        let bar = bars.get(index)?;
        let body = bar.body();
        let upper = bar.upper_shadow();
        let lower = bar.lower_shadow();
        let range = bar.range();

        if range <= 0.0 {
            return None;
        }

        // TA-Lib: doji body (BodyDoji: HighLow, Period=10, Factor=0.1, per-candle trailing)
        let avg_range = helpers::trailing_avg_range(bars, index, 10);
        if !is_doji_f(body, avg_range, range, self.doji_factor) {
            return None;
        }

        // TA-Lib: both shadows must be long (ShadowLong: Period=0 → shadow > body)
        if !is_shadow_long(upper, body, range) {
            return None;
        }
        if !is_shadow_long(lower, body, range) {
            return None;
        }

        // TA-Lib: body must be near the midpoint of the range
        // Near: HighLow, Period=5, Factor=0.2 (per-candle trailing at i)
        let midpoint = bar.low() + range / 2.0;
        let near_threshold = helpers::trailing_avg_range(bars, index, 5) * self.near_factor;
        let body_low = bar.open().min(bar.close());
        let body_high = bar.open().max(bar.close());
        if body_low > midpoint + near_threshold || body_high < midpoint - near_threshold {
            return None;
        }

        Some(PatternMatch {
            pattern_id: PatternDetector::id(self),
            direction: Direction::Neutral,
            strength: 0.6,
            start_index: index,
            end_index: index,
        })
    }
}

// ============================================================
// HAMMER FAMILY
// ============================================================

/// CDLHAMMER - Hammer (TA-Lib compatible)
#[derive(Debug, Clone, Copy)]
pub struct HammerDetector {
    pub body_short_factor: f64,
    pub shadow_veryshort_factor: f64,
    pub near_factor: f64,
}

impl Default for HammerDetector {
    fn default() -> Self {
        Self {
            body_short_factor: helpers::BODY_SHORT_FACTOR,
            shadow_veryshort_factor: helpers::SHADOW_VERYSHORT_FACTOR,
            near_factor: helpers::NEAR_FACTOR,
        }
    }
}

impl PatternDetector for HammerDetector {
    fn id(&self) -> PatternId {
        PatternId("CDL_HAMMER")
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
        let bar = bars.get(index)?;
        let prev = bars.get(index - 1)?;

        let body = bar.body();
        let upper = bar.upper_shadow();
        let lower = bar.lower_shadow();
        let range = bar.range();

        // TA-Lib: BodyShort — small real body
        if !is_body_short_f(body, ctx.avg_body, range, self.body_short_factor) {
            return None;
        }
        // TA-Lib: ShadowLong — long lower shadow (shadow > body)
        if !is_shadow_long(lower, body, range) {
            return None;
        }
        // TA-Lib: ShadowVeryShort — very short upper shadow
        if !is_shadow_very_short_f(upper, ctx.avg_range, range, self.shadow_veryshort_factor) {
            return None;
        }
        // TA-Lib: position check — body at or below prior candle's low
        // min(close, open) <= low[i-1] + Near_avg(at bar i-1)
        let body_low = bar.open().min(bar.close());
        let near_avg_range = helpers::trailing_avg_range(bars, index - 1, 5);
        let near_threshold = near_avg_range * self.near_factor;
        if body_low > prev.low() + near_threshold {
            return None;
        }

        Some(PatternMatch {
            pattern_id: PatternDetector::id(self),
            direction: Direction::Bullish,
            strength: 0.6,
            start_index: index,
            end_index: index,
        })
    }
}

/// CDLHANGINGMAN - Hanging Man (TA-Lib compatible)
#[derive(Debug, Clone, Copy)]
pub struct HangingManDetector {
    pub body_short_factor: f64,
    pub shadow_veryshort_factor: f64,
    pub near_factor: f64,
}

impl Default for HangingManDetector {
    fn default() -> Self {
        Self {
            body_short_factor: helpers::BODY_SHORT_FACTOR,
            shadow_veryshort_factor: helpers::SHADOW_VERYSHORT_FACTOR,
            near_factor: helpers::NEAR_FACTOR,
        }
    }
}

impl PatternDetector for HangingManDetector {
    fn id(&self) -> PatternId {
        PatternId("CDL_HANGINGMAN")
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
        let bar = bars.get(index)?;
        let prev = bars.get(index - 1)?;

        let body = bar.body();
        let upper = bar.upper_shadow();
        let lower = bar.lower_shadow();
        let range = bar.range();

        // TA-Lib: BodyShort — small real body (per-candle trailing avg at i)
        let avg_body = helpers::trailing_avg_body(bars, index, 10);
        if !is_body_short_f(body, avg_body, range, self.body_short_factor) {
            return None;
        }
        // TA-Lib: ShadowLong — long lower shadow (shadow > body)
        if !is_shadow_long(lower, body, range) {
            return None;
        }
        // TA-Lib: ShadowVeryShort — very short upper shadow (per-candle trailing avg at i)
        let avg_range = helpers::trailing_avg_range(bars, index, 10);
        if !is_shadow_very_short_f(upper, avg_range, range, self.shadow_veryshort_factor) {
            return None;
        }
        // TA-Lib: position check — body at or above prior candle's high
        // min(close, open) >= high[i-1] - Near_avg(at bar i-1)
        let body_low = bar.open().min(bar.close());
        let near_avg_range = helpers::trailing_avg_range(bars, index - 1, 5);
        let near_threshold = near_avg_range * self.near_factor;
        if body_low < prev.high() - near_threshold {
            return None;
        }

        Some(PatternMatch {
            pattern_id: PatternDetector::id(self),
            direction: Direction::Bearish,
            strength: 0.6,
            start_index: index,
            end_index: index,
        })
    }
}

/// CDLINVERTEDHAMMER - Inverted Hammer (TA-Lib compatible)
#[derive(Debug, Clone, Copy)]
pub struct InvertedHammerDetector {
    pub body_short_factor: f64,
    pub shadow_veryshort_factor: f64,
}

impl Default for InvertedHammerDetector {
    fn default() -> Self {
        Self {
            body_short_factor: helpers::BODY_SHORT_FACTOR,
            shadow_veryshort_factor: helpers::SHADOW_VERYSHORT_FACTOR,
        }
    }
}

impl PatternDetector for InvertedHammerDetector {
    fn id(&self) -> PatternId {
        PatternId("CDL_INVERTEDHAMMER")
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
        let bar = bars.get(index)?;
        let prev = bars.get(index - 1)?;

        // TA-Lib: Requires real body gap down from previous bar (TA_REALBODYGAPDOWN)
        // max(open, close) of current < min(open, close) of previous
        let curr_body_top = bar.open().max(bar.close());
        let prev_body_bottom = prev.open().min(prev.close());
        if curr_body_top >= prev_body_bottom {
            return None;
        }

        let body = bar.body();
        let upper = bar.upper_shadow();
        let lower = bar.lower_shadow();
        let range = bar.range();

        // TA-Lib: BodyShort (per-candle trailing avg at i)
        let avg_body = helpers::trailing_avg_body(bars, index, 10);
        if !is_body_short_f(body, avg_body, range, self.body_short_factor) {
            return None;
        }
        // TA-Lib: ShadowLong — upper shadow > avg(ShadowLong) which is body * 1.0 (Period=0)
        if !is_shadow_long(upper, body, range) {
            return None;
        }
        // TA-Lib: ShadowVeryShort — very short lower shadow (per-candle trailing avg at i)
        let avg_range = helpers::trailing_avg_range(bars, index, 10);
        if !is_shadow_very_short_f(lower, avg_range, range, self.shadow_veryshort_factor) {
            return None;
        }

        Some(PatternMatch {
            pattern_id: PatternDetector::id(self),
            direction: Direction::Bullish,
            strength: 0.7,
            start_index: index,
            end_index: index,
        })
    }
}

/// CDLSHOOTINGSTAR - Shooting Star (TA-Lib compatible)
#[derive(Debug, Clone, Copy)]
pub struct ShootingStarDetector {
    pub body_short_factor: f64,
    pub shadow_veryshort_factor: f64,
}

impl Default for ShootingStarDetector {
    fn default() -> Self {
        Self {
            body_short_factor: helpers::BODY_SHORT_FACTOR,
            shadow_veryshort_factor: helpers::SHADOW_VERYSHORT_FACTOR,
        }
    }
}

impl PatternDetector for ShootingStarDetector {
    fn id(&self) -> PatternId {
        PatternId("CDL_SHOOTINGSTAR")
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
        let bar = bars.get(index)?;
        let prev = bars.get(index - 1)?;

        // TA-Lib: Requires real body gap up from previous bar (TA_REALBODYGAPUP)
        // min(open, close) of current > max(open, close) of previous
        let curr_body_bottom = bar.open().min(bar.close());
        let prev_body_top = prev.open().max(prev.close());
        if curr_body_bottom <= prev_body_top {
            return None;
        }

        let body = bar.body();
        let upper = bar.upper_shadow();
        let lower = bar.lower_shadow();
        let range = bar.range();

        // TA-Lib: BodyShort (per-candle trailing avg at i)
        let avg_body = helpers::trailing_avg_body(bars, index, 10);
        if !is_body_short_f(body, avg_body, range, self.body_short_factor) {
            return None;
        }
        // TA-Lib: ShadowLong — upper shadow > avg(ShadowLong) which is body * 1.0 (Period=0)
        if !is_shadow_long(upper, body, range) {
            return None;
        }
        // TA-Lib: ShadowVeryShort — very short lower shadow (per-candle trailing avg at i)
        let avg_range = helpers::trailing_avg_range(bars, index, 10);
        if !is_shadow_very_short_f(lower, avg_range, range, self.shadow_veryshort_factor) {
            return None;
        }

        Some(PatternMatch {
            pattern_id: PatternDetector::id(self),
            direction: Direction::Bearish,
            strength: 0.7,
            start_index: index,
            end_index: index,
        })
    }
}

/// CDLTAKURI - Takuri (TA-Lib compatible)
#[derive(Debug, Clone, Copy)]
pub struct TakuriDetector {
    pub doji_factor: f64,
    pub shadow_verylong_factor: f64,
    pub shadow_veryshort_factor: f64,
}

impl Default for TakuriDetector {
    fn default() -> Self {
        Self {
            doji_factor: helpers::DOJI_FACTOR,
            shadow_verylong_factor: helpers::SHADOW_VERYLONG_FACTOR,
            shadow_veryshort_factor: helpers::SHADOW_VERYSHORT_FACTOR,
        }
    }
}

impl PatternDetector for TakuriDetector {
    fn id(&self) -> PatternId {
        PatternId("CDL_TAKURI")
    }

    fn min_bars(&self) -> usize {
        1
    }

    fn detect<T: OHLCV>(
        &self,
        bars: &[T],
        index: usize,
        _ctx: &MarketContext,
    ) -> Option<PatternMatch> {
        let bar = bars.get(index)?;

        let body = bar.body();
        let upper = bar.upper_shadow();
        let lower = bar.lower_shadow();
        let range = bar.range();

        // TA-Lib: BodyDoji — doji body (per-candle trailing avg_range at i)
        let avg_range = helpers::trailing_avg_range(bars, index, 10);
        if !is_doji_f(body, avg_range, range, self.doji_factor) {
            return None;
        }
        // TA-Lib: ShadowVeryLong — very long lower shadow (shadow > body * 2)
        if !is_shadow_verylong_f(lower, body, range, self.shadow_verylong_factor) {
            return None;
        }
        // TA-Lib: ShadowVeryShort — very short upper shadow (per-candle trailing avg_range at i)
        if !is_shadow_very_short_f(upper, avg_range, range, self.shadow_veryshort_factor) {
            return None;
        }

        Some(PatternMatch {
            pattern_id: PatternDetector::id(self),
            direction: Direction::Bullish,
            strength: 0.8,
            start_index: index,
            end_index: index,
        })
    }
}

// ============================================================
// MARUBOZU FAMILY
// ============================================================

/// CDLMARUBOZU - Marubozu (TA-Lib compatible)
#[derive(Debug, Clone, Copy)]
pub struct MarubozuDetector {
    pub body_long_factor: f64,
    pub shadow_veryshort_factor: f64,
}

impl Default for MarubozuDetector {
    fn default() -> Self {
        Self {
            body_long_factor: helpers::BODY_LONG_FACTOR,
            shadow_veryshort_factor: helpers::SHADOW_VERYSHORT_FACTOR,
        }
    }
}

impl PatternDetector for MarubozuDetector {
    fn id(&self) -> PatternId {
        PatternId("CDL_MARUBOZU")
    }

    fn min_bars(&self) -> usize {
        1
    }

    fn detect<T: OHLCV>(
        &self,
        bars: &[T],
        index: usize,
        ctx: &MarketContext,
    ) -> Option<PatternMatch> {
        let bar = bars.get(index)?;
        let body = bar.body();
        let upper = bar.upper_shadow();
        let lower = bar.lower_shadow();
        let range = bar.range();

        if !is_body_long_f(body, ctx.avg_body, range, self.body_long_factor) {
            return None;
        }
        // TA-Lib: both shadows must be ShadowVeryShort (< avg_range * 0.1)
        if !is_shadow_very_short_f(upper, ctx.avg_range, range, self.shadow_veryshort_factor) {
            return None;
        }
        if !is_shadow_very_short_f(lower, ctx.avg_range, range, self.shadow_veryshort_factor) {
            return None;
        }

        // TA-Lib: TA_CANDLECOLOR: close >= open → bullish(+1), close < open → bearish(-1)
        let direction = if bar.close() >= bar.open() {
            Direction::Bullish
        } else {
            Direction::Bearish
        };

        Some(PatternMatch {
            pattern_id: PatternDetector::id(self),
            direction,
            strength: 0.8,
            start_index: index,
            end_index: index,
        })
    }
}

/// CDLCLOSINGMARUBOZU - Closing Marubozu (TA-Lib compatible)
#[derive(Debug, Clone, Copy)]
pub struct ClosingMarubozuDetector {
    pub body_long_factor: f64,
    pub shadow_veryshort_factor: f64,
}

impl Default for ClosingMarubozuDetector {
    fn default() -> Self {
        Self {
            body_long_factor: helpers::BODY_LONG_FACTOR,
            shadow_veryshort_factor: helpers::SHADOW_VERYSHORT_FACTOR,
        }
    }
}

impl PatternDetector for ClosingMarubozuDetector {
    fn id(&self) -> PatternId {
        PatternId("CDL_CLOSINGMARUBOZU")
    }

    fn min_bars(&self) -> usize {
        1
    }

    fn detect<T: OHLCV>(
        &self,
        bars: &[T],
        index: usize,
        ctx: &MarketContext,
    ) -> Option<PatternMatch> {
        let bar = bars.get(index)?;
        let body = bar.body();
        let upper = bar.upper_shadow();
        let lower = bar.lower_shadow();
        let range = bar.range();

        if !is_body_long_f(body, ctx.avg_body, range, self.body_long_factor) {
            return None;
        }

        // TA-Lib: close-side shadow must be ShadowVeryShort (< avg_range * 0.1)
        let (direction, valid) = if bar.is_bullish() {
            (
                Direction::Bullish,
                is_shadow_very_short_f(upper, ctx.avg_range, range, self.shadow_veryshort_factor),
            )
        } else {
            (
                Direction::Bearish,
                is_shadow_very_short_f(lower, ctx.avg_range, range, self.shadow_veryshort_factor),
            )
        };

        if !valid {
            return None;
        }

        Some(PatternMatch {
            pattern_id: PatternDetector::id(self),
            direction,
            strength: 0.7,
            start_index: index,
            end_index: index,
        })
    }
}

// ============================================================
// LINE PATTERNS
// ============================================================

/// CDLLONGLINE - Long Line Candle (TA-Lib compatible)
#[derive(Debug, Clone, Copy)]
pub struct LongLineDetector {
    pub body_long_factor: f64,
}

impl Default for LongLineDetector {
    fn default() -> Self {
        Self {
            body_long_factor: helpers::BODY_LONG_FACTOR,
        }
    }
}

impl PatternDetector for LongLineDetector {
    fn id(&self) -> PatternId {
        PatternId("CDL_LONGLINE")
    }

    fn min_bars(&self) -> usize {
        1
    }

    fn detect<T: OHLCV>(
        &self,
        bars: &[T],
        index: usize,
        ctx: &MarketContext,
    ) -> Option<PatternMatch> {
        let bar = bars.get(index)?;
        let body = bar.body();
        let upper = bar.upper_shadow();
        let lower = bar.lower_shadow();
        let range = bar.range();

        if !is_body_long_f(body, ctx.avg_body, range, self.body_long_factor) {
            return None;
        }
        // TA-Lib: ShadowShort uses avg of max(upper, lower) over period
        if !is_shadow_short(upper, ctx.avg_shadow, range) {
            return None;
        }
        if !is_shadow_short(lower, ctx.avg_shadow, range) {
            return None;
        }

        // TA-Lib: TA_CANDLECOLOR: close >= open → bullish(+1), close < open → bearish(-1)
        let direction = if bar.close() >= bar.open() {
            Direction::Bullish
        } else {
            Direction::Bearish
        };

        Some(PatternMatch {
            pattern_id: PatternDetector::id(self),
            direction,
            strength: 0.6,
            start_index: index,
            end_index: index,
        })
    }
}

/// CDLSHORTLINE - Short Line Candle (TA-Lib compatible)
#[derive(Debug, Clone, Copy)]
pub struct ShortLineDetector {
    pub body_short_factor: f64,
}

impl Default for ShortLineDetector {
    fn default() -> Self {
        Self {
            body_short_factor: helpers::BODY_SHORT_FACTOR,
        }
    }
}

impl PatternDetector for ShortLineDetector {
    fn id(&self) -> PatternId {
        PatternId("CDL_SHORTLINE")
    }

    fn min_bars(&self) -> usize {
        1
    }

    fn detect<T: OHLCV>(
        &self,
        bars: &[T],
        index: usize,
        _ctx: &MarketContext,
    ) -> Option<PatternMatch> {
        let bar = bars.get(index)?;
        let body = bar.body();
        let upper = bar.upper_shadow();
        let lower = bar.lower_shadow();
        let range = bar.range();

        // TA-Lib: BodyShort (per-candle trailing avg at i)
        let avg_body = helpers::trailing_avg_body(bars, index, 10);
        if !is_body_short_f(body, avg_body, range, self.body_short_factor) {
            return None;
        }
        // TA-Lib: ShadowShort — per-candle trailing avg shadow at i
        let avg_shadow = helpers::trailing_avg_shadow(bars, index, 10);
        if !is_shadow_short(upper, avg_shadow, range) {
            return None;
        }
        if !is_shadow_short(lower, avg_shadow, range) {
            return None;
        }

        // TA-Lib: TA_CANDLECOLOR: close >= open → bullish(+1), close < open → bearish(-1)
        let direction = if bar.close() >= bar.open() {
            Direction::Bullish
        } else {
            Direction::Bearish
        };

        Some(PatternMatch {
            pattern_id: PatternDetector::id(self),
            direction,
            strength: 0.5,
            start_index: index,
            end_index: index,
        })
    }
}

// ============================================================
// SPINNING TOP / HIGH WAVE
// ============================================================

/// CDLSPINNINGTOP - Spinning Top (TA-Lib compatible)
#[derive(Debug, Clone, Copy)]
pub struct SpinningTopDetector {
    pub body_short_factor: f64,
}

impl Default for SpinningTopDetector {
    fn default() -> Self {
        Self {
            body_short_factor: helpers::BODY_SHORT_FACTOR,
        }
    }
}

impl PatternDetector for SpinningTopDetector {
    fn id(&self) -> PatternId {
        PatternId("CDL_SPINNINGTOP")
    }

    fn min_bars(&self) -> usize {
        1
    }

    fn detect<T: OHLCV>(
        &self,
        bars: &[T],
        index: usize,
        _ctx: &MarketContext,
    ) -> Option<PatternMatch> {
        let bar = bars.get(index)?;
        let body = bar.body();
        let upper = bar.upper_shadow();
        let lower = bar.lower_shadow();
        let range = bar.range();

        // TA-Lib: BodyShort — body < CandleAverage(BodyShort) (per-candle trailing avg at i)
        let avg_body = helpers::trailing_avg_body(bars, index, 10);
        if !is_body_short_f(body, avg_body, range, self.body_short_factor) {
            return None;
        }
        // TA-Lib: upper shadow > real body
        if upper <= body {
            return None;
        }
        // TA-Lib: lower shadow > real body
        if lower <= body {
            return None;
        }

        // TA-Lib: TA_CANDLECOLOR: close >= open → bullish(+1), close < open → bearish(-1)
        let direction = if bar.close() >= bar.open() {
            Direction::Bullish
        } else {
            Direction::Bearish
        };

        Some(PatternMatch {
            pattern_id: PatternDetector::id(self),
            direction,
            strength: 0.5,
            start_index: index,
            end_index: index,
        })
    }
}

/// CDLHIGHWAVE - High-Wave Candle (TA-Lib compatible)
#[derive(Debug, Clone, Copy)]
pub struct HighWaveDetector {
    pub body_short_factor: f64,
    pub shadow_verylong_factor: f64,
}

impl Default for HighWaveDetector {
    fn default() -> Self {
        Self {
            body_short_factor: helpers::BODY_SHORT_FACTOR,
            shadow_verylong_factor: helpers::SHADOW_VERYLONG_FACTOR,
        }
    }
}

impl PatternDetector for HighWaveDetector {
    fn id(&self) -> PatternId {
        PatternId("CDL_HIGHWAVE")
    }

    fn min_bars(&self) -> usize {
        1
    }

    fn detect<T: OHLCV>(
        &self,
        bars: &[T],
        index: usize,
        _ctx: &MarketContext,
    ) -> Option<PatternMatch> {
        let bar = bars.get(index)?;
        let body = bar.body();
        let upper = bar.upper_shadow();
        let lower = bar.lower_shadow();
        let range = bar.range();

        // TA-Lib: BodyShort (per-candle trailing avg at i)
        let avg_body = helpers::trailing_avg_body(bars, index, 10);
        if !is_body_short_f(body, avg_body, range, self.body_short_factor) {
            return None;
        }
        // TA-Lib: ShadowVeryLong — shadow > body * 2.0 (Period=0)
        if !is_shadow_verylong_f(upper, body, range, self.shadow_verylong_factor) {
            return None;
        }
        if !is_shadow_verylong_f(lower, body, range, self.shadow_verylong_factor) {
            return None;
        }

        // TA-Lib: TA_CANDLECOLOR: close >= open → bullish(+1), close < open → bearish(-1)
        let direction = if bar.close() >= bar.open() {
            Direction::Bullish
        } else {
            Direction::Bearish
        };

        Some(PatternMatch {
            pattern_id: PatternDetector::id(self),
            direction,
            strength: 0.6,
            start_index: index,
            end_index: index,
        })
    }
}

// ============================================================
// BELT HOLD
// ============================================================

/// CDLBELTHOLD - Belt Hold (TA-Lib compatible)
#[derive(Debug, Clone, Copy)]
pub struct BeltHoldDetector {
    pub body_long_factor: f64,
    pub shadow_veryshort_factor: f64,
}

impl Default for BeltHoldDetector {
    fn default() -> Self {
        Self {
            body_long_factor: helpers::BODY_LONG_FACTOR,
            shadow_veryshort_factor: helpers::SHADOW_VERYSHORT_FACTOR,
        }
    }
}

impl PatternDetector for BeltHoldDetector {
    fn id(&self) -> PatternId {
        PatternId("CDL_BELTHOLD")
    }

    fn min_bars(&self) -> usize {
        1
    }

    fn detect<T: OHLCV>(
        &self,
        bars: &[T],
        index: usize,
        ctx: &MarketContext,
    ) -> Option<PatternMatch> {
        let bar = bars.get(index)?;
        let body = bar.body();
        let upper = bar.upper_shadow();
        let lower = bar.lower_shadow();
        let range = bar.range();

        if !is_body_long_f(body, ctx.avg_body, range, self.body_long_factor) {
            return None;
        }

        // TA-Lib: open-side shadow must be ShadowVeryShort (< avg_range * 0.1)
        let (direction, valid) = if bar.is_bullish() {
            (
                Direction::Bullish,
                is_shadow_very_short_f(lower, ctx.avg_range, range, self.shadow_veryshort_factor),
            )
        } else if bar.is_bearish() {
            (
                Direction::Bearish,
                is_shadow_very_short_f(upper, ctx.avg_range, range, self.shadow_veryshort_factor),
            )
        } else {
            return None;
        };

        if !valid {
            return None;
        }

        Some(PatternMatch {
            pattern_id: PatternDetector::id(self),
            direction,
            strength: 0.7,
            start_index: index,
            end_index: index,
        })
    }
}

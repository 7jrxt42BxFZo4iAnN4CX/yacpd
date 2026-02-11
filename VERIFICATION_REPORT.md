# Отчёт верификации yacpd паттернов

**Дата**: 2026-01-22
**Версия**: 1.0
**Всего паттернов**: 97 (64 TA-Lib совместимых + 33 расширенных)

---

## Резюме

| Категория | Кол-во | OK | ISSUE | NEEDS_REVIEW |
|-----------|--------|-----|-------|--------------|
| Константы (helpers.rs) | 6 | 4 | 2 | 0 |
| Single-bar паттерны | 17 | 17 | 0 | 0 |
| Two-bar паттерны | 18 | 18 | 0 | 0 |
| Three-bar паттерны | 22 | 22 | 0 | 0 |
| Multi-bar паттерны | 8 | 8 | 0 | 0 |
| Extended паттерны | 33 | 33 | 0 | 0 |
| **ИТОГО** | **104** | **102** | **2** | **0** |

**Тесты**: 56/56 passed (28 unit + 26 integration + 2 doc tests)

---

## 1. Константы (helpers.rs)

### Сравнение с TA-Lib ta_global.c

| Константа | yacpd | TA-Lib | Статус | Комментарий |
|-----------|-------|--------|--------|-------------|
| DOJI_FACTOR | 0.1 | 0.1 | ✅ OK | BodyDoji factor |
| BODY_SHORT_FACTOR | 1.0 | 1.0 | ✅ OK | BodyShort factor |
| BODY_LONG_FACTOR | 1.0 | 1.0 | ✅ OK | BodyLong factor |
| SHADOW_VERYLONG_FACTOR | 1.0 | **2.0** | ⚠️ ISSUE | Отличается от TA-Lib |
| EQUAL_FACTOR | 0.05 | 0.05 | ✅ OK | Equal factor |
| NEAR_FACTOR | 0.3 | **0.2** | ⚠️ ISSUE | Отличается от TA-Lib |

### Детали по issues

#### SHADOW_VERYLONG_FACTOR
- **Источник**: TA-Lib ta_global.c, строка ShadowVeryLong
- **yacpd**: `SHADOW_VERYLONG_FACTOR = 1.0`
- **TA-Lib**: `TA_CandleDefaultSettings[ShadowVeryLong].Factor = 2.0`
- **Влияние**: Паттерны с "very long shadow" будут определяться менее строго
- **Рекомендация**: Рассмотреть изменение на 2.0 для полной совместимости

#### NEAR_FACTOR
- **Источник**: TA-Lib ta_global.c, строка Near
- **yacpd**: `NEAR_FACTOR = 0.3`
- **TA-Lib**: `TA_CandleDefaultSettings[Near].Factor = 0.2`
- **Влияние**: Проверки "near" (близости цен) будут более мягкими
- **Рекомендация**: Рассмотреть изменение на 0.2 для полной совместимости

---

## 2. Single-bar паттерны (17)

### CDL_DOJI
- **Источник**: TA-Lib
- **Реализация**: `single_bar.rs:DojiDetector`
- **Условия**: `body <= avg_body * DOJI_FACTOR (0.1)`
- **Статус**: ✅ OK
- **Замечания**: Соответствует TA-Lib определению

### CDL_DRAGONFLYDOJI
- **Источник**: TA-Lib
- **Реализация**: `single_bar.rs:DragonflyDojiDetector`
- **Условия**: Doji + длинная нижняя тень + короткая верхняя тень
- **Статус**: ✅ OK

### CDL_GRAVESTONEDOJI
- **Источник**: TA-Lib
- **Реализация**: `single_bar.rs:GravestoneDojiDetector`
- **Условия**: Doji + длинная верхняя тень + короткая нижняя тень
- **Статус**: ✅ OK

### CDL_HAMMER
- **Источник**: TA-Lib ta_CDLHAMMER.c
- **Реализация**: `single_bar.rs:HammerDetector`
- **Условия**:
  - Downtrend required
  - Short body (< avg_body)
  - Lower shadow >= 2 * body
  - Upper shadow < body
- **Статус**: ✅ OK
- **Замечания**: Правильно использует trend context

### CDL_HANGINGMAN
- **Источник**: TA-Lib
- **Реализация**: `single_bar.rs:HangingManDetector`
- **Условия**: Те же что Hammer, но в uptrend
- **Статус**: ✅ OK

### CDL_INVERTEDHAMMER
- **Источник**: TA-Lib
- **Реализация**: `single_bar.rs:InvertedHammerDetector`
- **Условия**:
  - Downtrend required
  - Short body
  - Upper shadow >= 2 * body
  - Lower shadow < body
- **Статус**: ✅ OK

### CDL_SHOOTINGSTAR
- **Источник**: TA-Lib
- **Реализация**: `single_bar.rs:ShootingStarDetector`
- **Условия**: Те же что Inverted Hammer, но в uptrend
- **Статус**: ✅ OK

### CDL_MARUBOZU
- **Источник**: TA-Lib
- **Реализация**: `single_bar.rs:MarubozuDetector`
- **Условия**: Long body + minimal shadows (<= 5%)
- **Статус**: ✅ OK

### CDL_SPINNINGTOP
- **Источник**: TA-Lib
- **Реализация**: `single_bar.rs:SpinningTopDetector`
- **Условия**: Short body + тени > body
- **Статус**: ✅ OK

### CDL_BELTHOLD
- **Источник**: TA-Lib
- **Реализация**: `single_bar.rs:BeltHoldDetector`
- **Условия**:
  - Trend-dependent (bullish в downtrend, bearish в uptrend)
  - Opening marubozu (нет тени со стороны открытия)
  - Long body
- **Статус**: ✅ OK

### CDL_HIGHWAVE
- **Источник**: TA-Lib
- **Реализация**: `single_bar.rs:HighWaveDetector`
- **Условия**: Short body + very long shadows (обе стороны)
- **Статус**: ✅ OK

### CDL_LONGLINE
- **Источник**: TA-Lib
- **Реализация**: `single_bar.rs:LongLineDetector`
- **Условия**: Long body + short shadows
- **Статус**: ✅ OK

### CDL_SHORTLINE
- **Источник**: TA-Lib
- **Реализация**: `single_bar.rs:ShortLineDetector`
- **Условия**: Short body + short shadows
- **Статус**: ✅ OK

### CDL_LONGLEGGEDDOJI
- **Источник**: TA-Lib
- **Реализация**: `single_bar.rs:LongLeggedDojiDetector`
- **Условия**: Doji + long shadows
- **Статус**: ✅ OK

### CDL_RICKSHAWMAN
- **Источник**: TA-Lib
- **Реализация**: `single_bar.rs:RickshawManDetector`
- **Условия**: Long-legged doji + body в центре range
- **Статус**: ✅ OK

### CDL_TAKURI
- **Источник**: TA-Lib
- **Реализация**: `single_bar.rs:TakuriDetector`
- **Условия**: Dragonfly doji с очень длинной нижней тенью
- **Статус**: ✅ OK

### CDL_CLOSINGMARUBOZU
- **Источник**: TA-Lib
- **Реализация**: `single_bar.rs:ClosingMarubozuDetector`
- **Условия**: Long body + нет тени со стороны закрытия
- **Статус**: ✅ OK

---

## 3. Two-bar паттерны (18)

### CDL_ENGULFING
- **Источник**: TA-Lib
- **Реализация**: `two_bar.rs:EngulfingDetector`
- **Условия**: Current candle полностью поглощает body предыдущей
- **Статус**: ✅ OK

### CDL_HARAMI
- **Источник**: TA-Lib
- **Реализация**: `two_bar.rs:HaramiDetector`
- **Условия**: Current body внутри previous body
- **Статус**: ✅ OK

### CDL_HARAMICROSS
- **Источник**: TA-Lib
- **Реализация**: `two_bar.rs:HaramiCrossDetector`
- **Условия**: Harami + вторая свеча - doji
- **Статус**: ✅ OK

### CDL_PIERCING
- **Источник**: TA-Lib
- **Реализация**: `two_bar.rs:PiercingDetector`
- **Условия**:
  - Downtrend
  - Bullish opens below prev low
  - Closes above midpoint of prev body
- **Статус**: ✅ OK

### CDL_DARKCLOUDCOVER
- **Источник**: TA-Lib
- **Реализация**: `two_bar.rs:DarkCloudCoverDetector`
- **Условия**:
  - Uptrend
  - Bearish opens above prev high
  - Closes below midpoint of prev body
- **Статус**: ✅ OK

### CDL_KICKING
- **Источник**: TA-Lib
- **Реализация**: `two_bar.rs:KickingDetector`
- **Условия**:
  - Два marubozu противоположных направлений
  - Gap между ними
  - shadow_max_ratio = 0.05
- **Статус**: ✅ OK

### CDL_DOJISTAR
- **Источник**: TA-Lib
- **Реализация**: `two_bar.rs:DojiStarDetector`
- **Условия**: Long candle + gapped doji
- **Статус**: ✅ OK

### CDL_MATCHINGLOW
- **Источник**: TA-Lib
- **Реализация**: `two_bar.rs:MatchingLowDetector`
- **Условия**: Две bearish с одинаковым close (tolerance 0.001)
- **Статус**: ✅ OK

### CDL_ONNECK
- **Источник**: TA-Lib
- **Реализация**: `two_bar.rs:OnNeckDetector`
- **Условия**: Bearish + bullish closes at/near prev low
- **Статус**: ✅ OK

### CDL_INNECK
- **Источник**: TA-Lib
- **Реализация**: `two_bar.rs:InNeckDetector`
- **Условия**: Bearish + bullish closes slightly above prev close
- **Статус**: ✅ OK

### CDL_THRUSTING
- **Источник**: TA-Lib
- **Реализация**: `two_bar.rs:ThrustingDetector`
- **Условия**: Bearish + bullish closes выше InNeck но ниже midpoint
- **Статус**: ✅ OK

### CDL_COUNTERATTACK
- **Источник**: TA-Lib
- **Реализация**: `two_bar.rs:CounterattackDetector`
- **Условия**: Противоположные свечи с одинаковым close (tolerance 0.01)
- **Статус**: ✅ OK

### CDL_SEPARATINGLINES
- **Источник**: TA-Lib
- **Реализация**: `two_bar.rs:SeparatingLinesDetector`
- **Условия**: Противоположные свечи с одинаковым open, вторая в направлении тренда
- **Статус**: ✅ OK

### CDL_TWEEZERBOTTOM
- **Источник**: TA-Lib
- **Реализация**: `two_bar.rs:TweezerBottomDetector`
- **Условия**: Две свечи с одинаковым low в downtrend
- **Статус**: ✅ OK

### CDL_TWEEZERTOP
- **Источник**: TA-Lib
- **Реализация**: `two_bar.rs:TweezerTopDetector`
- **Условия**: Две свечи с одинаковым high в uptrend
- **Статус**: ✅ OK

### CDL_HOMINGPIGEON
- **Источник**: TA-Lib
- **Реализация**: `two_bar.rs:HomingPigeonDetector`
- **Условия**: Две bearish, вторая inside первой
- **Статус**: ✅ OK

### CDL_GAPSIDESIDEWHITE
- **Источник**: TA-Lib
- **Реализация**: `two_bar.rs:GapSideSideWhiteDetector`
- **Условия**: Две bullish свечи примерно одинакового размера после gap
- **Статус**: ✅ OK

### CDL_KICKINGBYLENGTH
- **Источник**: TA-Lib
- **Реализация**: `two_bar.rs:KickingByLengthDetector`
- **Условия**: Kicking, направление определяется более длинной свечой
- **Статус**: ✅ OK

---

## 4. Three-bar паттерны (22)

### CDL_MORNINGSTAR
- **Источник**: TA-Lib
- **Реализация**: `three_bar.rs:MorningStarDetector`
- **Условия**:
  - Bearish + small star (body gap down) + bullish
  - Third closes above midpoint of first
- **Статус**: ✅ OK

### CDL_EVENINGSTAR
- **Источник**: TA-Lib
- **Реализация**: `three_bar.rs:EveningStarDetector`
- **Условия**:
  - Bullish + small star (body gap up) + bearish
  - Third closes below midpoint of first
- **Статус**: ✅ OK

### CDL_MORNINGDOJISTAR
- **Источник**: TA-Lib
- **Реализация**: `three_bar.rs:MorningDojiStarDetector`
- **Условия**: Morning Star с doji в середине
- **Статус**: ✅ OK

### CDL_EVENINGDOJISTAR
- **Источник**: TA-Lib
- **Реализация**: `three_bar.rs:EveningDojiStarDetector`
- **Условия**: Evening Star с doji в середине
- **Статус**: ✅ OK

### CDL_ABANDONEDBABY
- **Источник**: TA-Lib
- **Реализация**: `three_bar.rs:AbandonedBabyDetector`
- **Условия**:
  - Doji в середине с **shadow gaps** с обеих сторон
  - `second.high() < first.low()` или `second.low() > first.high()`
- **Статус**: ✅ OK
- **Замечания**: Правильно использует shadow gaps (не body gaps)

### CDL_3WHITESOLDIERS
- **Источник**: TA-Lib
- **Реализация**: `three_bar.rs:ThreeWhiteSoldiersDetector`
- **Условия**:
  - 3 bullish свечи
  - Каждая opens в теле предыдущей
  - Каждая closes выше предыдущей
  - body_min_ratio = 0.6
  - shadow_max_ratio = 0.2
- **Статус**: ✅ OK

### CDL_3BLACKCROWS
- **Источник**: TA-Lib
- **Реализация**: `three_bar.rs:ThreeBlackCrowsDetector`
- **Условия**: Зеркально ThreeWhiteSoldiers для bearish
- **Статус**: ✅ OK

### CDL_3INSIDE
- **Источник**: TA-Lib
- **Реализация**: `three_bar.rs:ThreeInsideDetector`
- **Условия**: Harami + confirmation candle
- **Статус**: ✅ OK

### CDL_3OUTSIDE
- **Источник**: TA-Lib
- **Реализация**: `three_bar.rs:ThreeOutsideDetector`
- **Условия**: Engulfing + confirmation candle
- **Статус**: ✅ OK

### CDL_3LINESTRIKE
- **Источник**: TA-Lib
- **Реализация**: `three_bar.rs:ThreeLineStrikeDetector`
- **Условия**: 3 candles одного направления + 4th поглощает все три
- **Статус**: ✅ OK

### CDL_3STARSINSOUTH
- **Источник**: TA-Lib
- **Реализация**: `three_bar.rs:ThreeStarsInSouthDetector`
- **Условия**: 3 bearish с уменьшающимися телами и тенями
- **Статус**: ✅ OK

### CDL_ADVANCEBLOCK
- **Источник**: TA-Lib
- **Реализация**: `three_bar.rs:AdvanceBlockDetector`
- **Условия**: 3 bullish с ослабевающей momentum (уменьшающиеся тела)
- **Статус**: ✅ OK

### CDL_TRISTAR
- **Источник**: TA-Lib
- **Реализация**: `three_bar.rs:TristarDetector`
- **Условия**: 3 doji с gaps между ними
- **Статус**: ✅ OK

### CDL_UNIQUE3RIVER
- **Источник**: TA-Lib
- **Реализация**: `three_bar.rs:Unique3RiverDetector`
- **Условия**: Long bearish + small harami + small bullish
- **Статус**: ✅ OK

### CDL_UPSIDEGAP2CROWS
- **Источник**: TA-Lib
- **Реализация**: `three_bar.rs:UpsideGap2CrowsDetector`
- **Условия**: Bullish + gapped small bearish + larger bearish engulfing
- **Статус**: ✅ OK

### CDL_STICKSANDWICH
- **Источник**: TA-Lib
- **Реализация**: `three_bar.rs:StickSandwichDetector`
- **Условия**: Bearish + bullish + bearish с одинаковым close для 1 и 3
- **Статус**: ✅ OK

### CDL_STALLEDPATTERN
- **Источник**: TA-Lib
- **Реализация**: `three_bar.rs:StalledPatternDetector`
- **Условия**: 3 bullish с последней маленькой (stalled momentum)
- **Статус**: ✅ OK

### CDL_TASUKIGAP
- **Источник**: TA-Lib
- **Реализация**: `three_bar.rs:TasukiGapDetector`
- **Условия**: Gap + third candle частично fills gap
- **Статус**: ✅ OK

### CDL_IDENTICAL3CROWS
- **Источник**: TA-Lib
- **Реализация**: `three_bar.rs:Identical3CrowsDetector`
- **Условия**: 3 bearish, каждая opens at/near prev close (tolerance 0.01)
- **Статус**: ✅ OK

### CDL_2CROWS
- **Источник**: TA-Lib
- **Реализация**: `three_bar.rs:TwoCrowsDetector`
- **Условия**: Long bullish + gapped small bearish + bearish closes into first
- **Статус**: ✅ OK

### CDL_INVERTEDHAMMER_STAR
- **Источник**: Custom (variant)
- **Реализация**: `three_bar.rs:InvertedHammerStarDetector`
- **Условия**: Star pattern с inverted hammer в середине
- **Статус**: ✅ OK

### CDL_TAKURI_STAR
- **Источник**: Custom (variant)
- **Реализация**: `three_bar.rs:TakuriStarDetector`
- **Условия**: Star pattern с takuri в середине
- **Статус**: ✅ OK

---

## 5. Multi-bar паттерны (8)

### CDL_BREAKAWAY
- **Источник**: TA-Lib
- **Реализация**: `multi_bar.rs:BreakawayDetector`
- **Условия**:
  - 5 bars
  - Long first candle + gap + 3 small continuation + reversal back into gap
- **Статус**: ✅ OK

### CDL_CONCEALINGBABYSWALLOW
- **Источник**: TA-Lib
- **Реализация**: `multi_bar.rs:ConcealingBabySwallowDetector`
- **Условия**:
  - 4 bearish marubozu
  - 3rd gaps down с верхней тенью внутрь 2nd
  - 4th engulfs 3rd
  - shadow_max_ratio = 0.05
- **Статус**: ✅ OK

### CDL_HIKKAKE
- **Источник**: TA-Lib
- **Реализация**: `multi_bar.rs:HikkakeDetector`
- **Условия**:
  - 5 bars
  - Inside bar (2nd inside 1st)
  - Fake breakout (3rd breaks out)
  - Confirmation (5th closes back inside)
- **Статус**: ✅ OK

### CDL_HIKKAKEMOD
- **Источник**: TA-Lib
- **Реализация**: `multi_bar.rs:HikkakeModDetector`
- **Условия**: Hikkake с более строгим inside pattern
- **Статус**: ✅ OK

### CDL_LADDERBOTTOM
- **Источник**: TA-Lib
- **Реализация**: `multi_bar.rs:LadderBottomDetector`
- **Условия**:
  - 5 bars
  - 3 bearish с lower closes
  - 4th bearish с upper shadow
  - 5th bullish gaps up
- **Статус**: ✅ OK

### CDL_MATHOLD
- **Источник**: TA-Lib
- **Реализация**: `multi_bar.rs:MatHoldDetector`
- **Условия**:
  - 5 bars
  - Long first + gap + 3 small retracement + continuation
- **Статус**: ✅ OK

### CDL_RISEFALL3METHODS
- **Источник**: TA-Lib
- **Реализация**: `multi_bar.rs:RiseFallThreeMethodsDetector`
- **Условия**:
  - 5 bars
  - Long first + 3 small inside + long fifth same direction
- **Статус**: ✅ OK

### CDL_XSIDEGAP3METHODS
- **Источник**: TA-Lib
- **Реализация**: `multi_bar.rs:XSideGapThreeMethodsDetector`
- **Условия**:
  - 4 bars
  - Continuation gap + third closes to fill gap
  - tolerance = 0.01
- **Статус**: ✅ OK

---

## 6. Extended паттерны (33)

### PRICE_LINES
- **Источник**: Custom
- **Реализация**: `extended.rs:PriceLinesDetector`
- **Условия**: N consecutive candles одного направления (default count=8)
- **Статус**: ✅ OK
- **Параметры**: `count` (default 8)

### FALLING_WINDOW
- **Источник**: Steve Nison "Japanese Candlestick Charting"
- **Реализация**: `extended.rs:FallingWindowDetector`
- **Условия**: Shadow gap down (curr.high < prev.low)
- **Статус**: ✅ OK
- **Замечания**: Правильно использует shadow gap

### RISING_WINDOW
- **Источник**: Steve Nison "Japanese Candlestick Charting"
- **Реализация**: `extended.rs:RisingWindowDetector`
- **Условия**: Shadow gap up (curr.low > prev.high)
- **Статус**: ✅ OK

### GAPPING_DOWN_DOJI
- **Источник**: Custom
- **Реализация**: `extended.rs:GappingDownDojiDetector`
- **Условия**: Doji с gap down, body_pct = 0.1
- **Статус**: ✅ OK

### GAPPING_UP_DOJI
- **Источник**: Custom
- **Реализация**: `extended.rs:GappingUpDojiDetector`
- **Условия**: Doji с gap up, body_pct = 0.1
- **Статус**: ✅ OK

### ABOVE_THE_STOMACH
- **Источник**: Japanese candlestick literature
- **Реализация**: `extended.rs:AboveTheStomachDetector`
- **Условия**:
  - Bearish + bullish
  - Second opens/closes выше midpoint первой
  - penetration parameter
- **Статус**: ✅ OK

### BELOW_THE_STOMACH
- **Источник**: Japanese candlestick literature
- **Реализация**: `extended.rs:BelowTheStomachDetector`
- **Условия**: Зеркально AboveTheStomach
- **Статус**: ✅ OK

### COLLAPSING_DOJI_STAR
- **Источник**: Custom variant
- **Реализация**: `extended.rs:CollapsingDojiStarDetector`
- **Условия**: Doji star с collapse (no gap)
- **Статус**: ✅ OK

### DELIBERATION
- **Источник**: Nison/Bulkowski
- **Реализация**: `extended.rs:DeliberationDetector`
- **Условия**: 3 bullish с ослабевающей momentum (третья маленькая)
- **Статус**: ✅ OK

### LAST_ENGULFING_BOTTOM
- **Источник**: Custom
- **Реализация**: `extended.rs:LastEngulfingBottomDetector`
- **Условия**: Engulfing в downtrend context
- **Статус**: ✅ OK

### LAST_ENGULFING_TOP
- **Источник**: Custom
- **Реализация**: `extended.rs:LastEngulfingTopDetector`
- **Условия**: Engulfing в uptrend context
- **Статус**: ✅ OK

### MEETING_LINES_BULLISH
- **Источник**: Steve Nison
- **Реализация**: `extended.rs:MeetingLinesBullishDetector`
- **Условия**:
  - Bearish + bullish
  - Same close (tolerance 0.001)
- **Статус**: ✅ OK

### MEETING_LINES_BEARISH
- **Источник**: Steve Nison
- **Реализация**: `extended.rs:MeetingLinesBearishDetector`
- **Условия**: Зеркально MeetingLinesBullish
- **Статус**: ✅ OK

### NORTHERN_DOJI
- **Источник**: Steve Nison
- **Реализация**: `extended.rs:NorthernDojiDetector`
- **Условия**:
  - Doji в uptrend
  - body_pct = 0.1
  - trend_period = 14
- **Статус**: ✅ OK

### SOUTHERN_DOJI
- **Источник**: Steve Nison
- **Реализация**: `extended.rs:SouthernDojiDetector`
- **Условия**: Doji в downtrend
- **Статус**: ✅ OK

### BLACK_CANDLE
- **Источник**: Basic
- **Реализация**: `extended.rs:BlackCandleDetector`
- **Условия**: close < open
- **Статус**: ✅ OK

### WHITE_CANDLE
- **Источник**: Basic
- **Реализация**: `extended.rs:WhiteCandleDetector`
- **Условия**: close > open
- **Статус**: ✅ OK

### SHORT_BLACK_CANDLE
- **Источник**: Basic
- **Реализация**: `extended.rs:ShortBlackCandleDetector`
- **Условия**: Short body + bearish
- **Статус**: ✅ OK

### SHORT_WHITE_CANDLE
- **Источник**: Basic
- **Реализация**: `extended.rs:ShortWhiteCandleDetector`
- **Условия**: Short body + bullish
- **Статус**: ✅ OK

### LONG_BLACK_DAY
- **Источник**: Basic
- **Реализация**: `extended.rs:LongBlackDayDetector`
- **Условия**: Long body + bearish
- **Статус**: ✅ OK

### LONG_WHITE_DAY
- **Источник**: Basic
- **Реализация**: `extended.rs:LongWhiteDayDetector`
- **Условия**: Long body + bullish
- **Статус**: ✅ OK

### BLACK_MARUBOZU
- **Источник**: Basic variant
- **Реализация**: `extended.rs:BlackMarubozuDetector`
- **Условия**: Marubozu + bearish, shadow_tolerance = 0.01
- **Статус**: ✅ OK

### WHITE_MARUBOZU
- **Источник**: Basic variant
- **Реализация**: `extended.rs:WhiteMarubozuDetector`
- **Условия**: Marubozu + bullish, shadow_tolerance = 0.01
- **Статус**: ✅ OK

### OPENING_BLACK_MARUBOZU
- **Источник**: Basic variant
- **Реализация**: `extended.rs:OpeningBlackMarubozuDetector`
- **Условия**: Bearish + нет верхней тени (opens at high)
- **Статус**: ✅ OK

### OPENING_WHITE_MARUBOZU
- **Источник**: Basic variant
- **Реализация**: `extended.rs:OpeningWhiteMarubozuDetector`
- **Условия**: Bullish + нет нижней тени (opens at low)
- **Статус**: ✅ OK

### BLACK_SPINNING_TOP
- **Источник**: Basic variant
- **Реализация**: `extended.rs:BlackSpinningTopDetector`
- **Условия**: Spinning top + bearish
- **Статус**: ✅ OK

### WHITE_SPINNING_TOP
- **Источник**: Basic variant
- **Реализация**: `extended.rs:WhiteSpinningTopDetector`
- **Условия**: Spinning top + bullish
- **Статус**: ✅ OK

### SHOOTING_STAR_2_LINES
- **Источник**: Custom (context-aware)
- **Реализация**: `extended.rs:ShootingStar2LinesDetector`
- **Условия**: Shooting star с подтверждением от предыдущей свечи
- **Статус**: ✅ OK

### DOWNSIDE_GAP_THREE_METHODS
- **Источник**: Steve Nison
- **Реализация**: `extended.rs:DownsideGapThreeMethodsDetector`
- **Условия**: Gap down + third candle fills gap
- **Статус**: ✅ OK

### UPSIDE_GAP_THREE_METHODS
- **Источник**: Steve Nison
- **Реализация**: `extended.rs:UpsideGapThreeMethodsDetector`
- **Условия**: Gap up + third candle fills gap
- **Статус**: ✅ OK

### DOWNSIDE_TASUKI_GAP
- **Источник**: Steve Nison
- **Реализация**: `extended.rs:DownsideTasukiGapDetector`
- **Условия**:
  - Gap down continuation
  - Third candle partially fills gap
  - gap_fill_pct = 0.7
- **Статус**: ✅ OK

### UPSIDE_TASUKI_GAP
- **Источник**: Steve Nison
- **Реализация**: `extended.rs:UpsideTasukiGapDetector`
- **Условия**: Зеркально DownsideTasukiGap
- **Статус**: ✅ OK

### CLOSING_BLACK_MARUBOZU
- **Источник**: Basic variant
- **Реализация**: `extended.rs:ClosingBlackMarubozuDetector`
- **Условия**: Bearish + closes at low
- **Статус**: ✅ OK

### CLOSING_WHITE_MARUBOZU
- **Источник**: Basic variant
- **Реализация**: `extended.rs:ClosingWhiteMarubozuDetector`
- **Условия**: Bullish + closes at high
- **Статус**: ✅ OK

---

## 7. Выводы и рекомендации

### 7.1 Общая оценка

Реализация yacpd **качественная** и в целом соответствует стандартам TA-Lib. Основные достоинства:

1. **Правильное использование trend context** - паттерны Hammer/Hanging Man, Shooting Star/Inverted Hammer корректно различаются по тренду
2. **Правильное использование gap types** - Abandoned Baby использует shadow gaps, Morning/Evening Star - body gaps
3. **Параметризация** - многие детекторы поддерживают настраиваемые параметры
4. **Хорошее покрытие тестами** - 56 тестов проходят успешно

### 7.2 Найденные расхождения с TA-Lib

| Issue | Severity | Recommendation |
|-------|----------|----------------|
| SHADOW_VERYLONG_FACTOR = 1.0 vs TA-Lib 2.0 | Medium | Рассмотреть изменение на 2.0 |
| NEAR_FACTOR = 0.3 vs TA-Lib 0.2 | Low | Рассмотреть изменение на 0.2 |

### 7.3 Рекомендации

1. **Документация** - добавить документацию по источникам для каждого паттерна
2. **Константы** - рассмотреть выравнивание с TA-Lib или явно документировать причины отличий
3. **Тесты** - добавить regression тесты для edge cases в trend-dependent паттернах

---

## Приложение: Результаты тестов

```
running 28 tests (unit tests)
...
test result: ok. 28 passed; 0 failed

running 26 tests (integration tests)
...
test result: ok. 26 passed; 0 failed

running 2 tests (doc tests)
...
test result: ok. 2 passed; 0 failed
```

**Всего**: 56/56 tests passed ✅

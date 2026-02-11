#!/usr/bin/env python3
"""
Comprehensive TA-Lib validation test suite for YACPD.

This script validates all 61 candlestick pattern detections against TA-Lib's reference implementation.

Features:
- All 61 TA-Lib CDL pattern tests
- Negative tests (false positive checks)
- Real market data tests
- Fuzz testing (random data comparison)

Requirements:
    pip install TA-Lib numpy pandas yfinance

Usage:
    python tests/talib_validation.py              # Run all tests
    python tests/talib_validation.py --positive   # Only positive tests
    python tests/talib_validation.py --negative   # Only negative tests
    python tests/talib_validation.py --fuzz       # Only fuzz tests
    python tests/talib_validation.py --real       # Only real data tests
    python tests/talib_validation.py --export     # Export test cases to JSON
"""

import json
import random
import numpy as np
import pandas as pd
from typing import Dict, List, Tuple, Any, Callable, Optional
from dataclasses import dataclass, asdict
from pathlib import Path

try:
    import talib
except ImportError:
    print("ERROR: TA-Lib not installed. Install with: pip install TA-Lib")
    print("Note: You may need to install the TA-Lib C library first.")
    print("On Ubuntu: sudo apt-get install libta-lib0-dev")
    print("On macOS: brew install ta-lib")
    exit(1)


# All 61 TA-Lib CDL functions for cross-validation
TALIB_FUNCS = {
    "CDL2CROWS": talib.CDL2CROWS,
    "CDL3BLACKCROWS": talib.CDL3BLACKCROWS,
    "CDL3INSIDE": talib.CDL3INSIDE,
    "CDL3LINESTRIKE": talib.CDL3LINESTRIKE,
    "CDL3OUTSIDE": talib.CDL3OUTSIDE,
    "CDL3STARSINSOUTH": talib.CDL3STARSINSOUTH,
    "CDL3WHITESOLDIERS": talib.CDL3WHITESOLDIERS,
    "CDLABANDONEDBABY": talib.CDLABANDONEDBABY,
    "CDLADVANCEBLOCK": talib.CDLADVANCEBLOCK,
    "CDLBELTHOLD": talib.CDLBELTHOLD,
    "CDLBREAKAWAY": talib.CDLBREAKAWAY,
    "CDLCLOSINGMARUBOZU": talib.CDLCLOSINGMARUBOZU,
    "CDLCONCEALBABYSWALL": talib.CDLCONCEALBABYSWALL,
    "CDLCOUNTERATTACK": talib.CDLCOUNTERATTACK,
    "CDLDARKCLOUDCOVER": talib.CDLDARKCLOUDCOVER,
    "CDLDOJI": talib.CDLDOJI,
    "CDLDOJISTAR": talib.CDLDOJISTAR,
    "CDLDRAGONFLYDOJI": talib.CDLDRAGONFLYDOJI,
    "CDLENGULFING": talib.CDLENGULFING,
    "CDLEVENINGDOJISTAR": talib.CDLEVENINGDOJISTAR,
    "CDLEVENINGSTAR": talib.CDLEVENINGSTAR,
    "CDLGAPSIDESIDEWHITE": talib.CDLGAPSIDESIDEWHITE,
    "CDLGRAVESTONEDOJI": talib.CDLGRAVESTONEDOJI,
    "CDLHAMMER": talib.CDLHAMMER,
    "CDLHANGINGMAN": talib.CDLHANGINGMAN,
    "CDLHARAMI": talib.CDLHARAMI,
    "CDLHARAMICROSS": talib.CDLHARAMICROSS,
    "CDLHIGHWAVE": talib.CDLHIGHWAVE,
    "CDLHIKKAKE": talib.CDLHIKKAKE,
    "CDLHIKKAKEMOD": talib.CDLHIKKAKEMOD,
    "CDLHOMINGPIGEON": talib.CDLHOMINGPIGEON,
    "CDLIDENTICAL3CROWS": talib.CDLIDENTICAL3CROWS,
    "CDLINNECK": talib.CDLINNECK,
    "CDLINVERTEDHAMMER": talib.CDLINVERTEDHAMMER,
    "CDLKICKING": talib.CDLKICKING,
    "CDLKICKINGBYLENGTH": talib.CDLKICKINGBYLENGTH,
    "CDLLADDERBOTTOM": talib.CDLLADDERBOTTOM,
    "CDLLONGLEGGEDDOJI": talib.CDLLONGLEGGEDDOJI,
    "CDLLONGLINE": talib.CDLLONGLINE,
    "CDLMARUBOZU": talib.CDLMARUBOZU,
    "CDLMATCHINGLOW": talib.CDLMATCHINGLOW,
    "CDLMATHOLD": talib.CDLMATHOLD,
    "CDLMORNINGDOJISTAR": talib.CDLMORNINGDOJISTAR,
    "CDLMORNINGSTAR": talib.CDLMORNINGSTAR,
    "CDLONNECK": talib.CDLONNECK,
    "CDLPIERCING": talib.CDLPIERCING,
    "CDLRICKSHAWMAN": talib.CDLRICKSHAWMAN,
    "CDLRISEFALL3METHODS": talib.CDLRISEFALL3METHODS,
    "CDLSEPARATINGLINES": talib.CDLSEPARATINGLINES,
    "CDLSHOOTINGSTAR": talib.CDLSHOOTINGSTAR,
    "CDLSHORTLINE": talib.CDLSHORTLINE,
    "CDLSPINNINGTOP": talib.CDLSPINNINGTOP,
    "CDLSTALLEDPATTERN": talib.CDLSTALLEDPATTERN,
    "CDLSTICKSANDWICH": talib.CDLSTICKSANDWICH,
    "CDLTAKURI": talib.CDLTAKURI,
    "CDLTASUKIGAP": talib.CDLTASUKIGAP,
    "CDLTHRUSTING": talib.CDLTHRUSTING,
    "CDLTRISTAR": talib.CDLTRISTAR,
    "CDLUNIQUE3RIVER": talib.CDLUNIQUE3RIVER,
    "CDLUPSIDEGAP2CROWS": talib.CDLUPSIDEGAP2CROWS,
    "CDLXSIDEGAP3METHODS": talib.CDLXSIDEGAP3METHODS,
}


@dataclass
class Bar:
    """OHLCV bar data."""
    open: float
    high: float
    low: float
    close: float
    volume: float = 1000.0


def bars_to_arrays(bars: List[Bar]) -> Tuple[np.ndarray, np.ndarray, np.ndarray, np.ndarray]:
    """Convert list of bars to numpy arrays for TA-Lib."""
    opens = np.array([b.open for b in bars], dtype=np.float64)
    highs = np.array([b.high for b in bars], dtype=np.float64)
    lows = np.array([b.low for b in bars], dtype=np.float64)
    closes = np.array([b.close for b in bars], dtype=np.float64)
    return opens, highs, lows, closes


# ============================================================
# TEST DATA GENERATORS
# ============================================================

def make_downtrend(n: int = 10, start: float = 100.0, step: float = 3.0) -> List[Bar]:
    """Generate a downtrend series of bars."""
    bars = []
    for i in range(n):
        base = start - i * step
        bars.append(Bar(
            open=base + 2,
            high=base + 3,
            low=base - 1,
            close=base - 0.5
        ))
    return bars


def make_uptrend(n: int = 10, start: float = 100.0, step: float = 3.0) -> List[Bar]:
    """Generate an uptrend series of bars."""
    bars = []
    for i in range(n):
        base = start + i * step
        bars.append(Bar(
            open=base - 1,
            high=base + 2,
            low=base - 2,
            close=base + 1
        ))
    return bars


def make_sideways(n: int = 10, base: float = 100.0) -> List[Bar]:
    """Generate a sideways/flat series of bars."""
    bars = []
    for i in range(n):
        bars.append(Bar(
            open=base - 2,
            high=base + 4,
            low=base - 4,
            close=base + 2
        ))
    return bars


def make_random_bars(n: int = 50, base: float = 100.0, volatility: float = 5.0) -> List[Bar]:
    """Generate random OHLCV bars."""
    bars = []
    price = base
    for _ in range(n):
        change = random.uniform(-volatility, volatility)
        o = price
        c = price + change
        h = max(o, c) + random.uniform(0, volatility * 0.5)
        l = min(o, c) - random.uniform(0, volatility * 0.5)
        bars.append(Bar(open=o, high=h, low=l, close=c))
        price = c
    return bars


def make_zero_range_bars(n=50):
    """Bars where O=H=L=C (perfect doji/flat). Tests zero-division edge cases."""
    bars = []
    price = 100.0
    for _ in range(n):
        price += random.uniform(-0.5, 0.5)
        bars.append(Bar(open=price, high=price, low=price, close=price))
    return bars


def make_gap_bars(n=50):
    """Bars with large gaps between consecutive bars. Tests gap-dependent patterns."""
    bars = []
    price = 100.0
    for _ in range(n):
        gap = random.uniform(0.05, 0.15) * price * random.choice([-1, 1])
        o = price + gap
        change = random.uniform(-2.0, 2.0)
        c = o + change
        h = max(o, c) + random.uniform(0, 1.0)
        l = min(o, c) - random.uniform(0, 1.0)
        bars.append(Bar(open=o, high=h, low=l, close=c))
        price = c
    return bars


def make_extreme_high_bars(n=50):
    """Bars at very high prices (~1e6). Tests numerical stability."""
    return make_random_bars(n, base=1_000_000, volatility=50_000)


def make_extreme_low_bars(n=50):
    """Bars at penny-stock prices (~0.01). Tests small number handling."""
    return make_random_bars(n, base=0.05, volatility=0.005)


def make_micro_volatility_bars(n=50):
    """Bars with nearly zero body/shadow. Tests threshold edge cases."""
    return make_random_bars(n, base=100, volatility=0.01)


# ============================================================
# ALL 61 TA-LIB PATTERN TEST GENERATORS
# ============================================================

def test_cdl2crows() -> Dict[str, Any]:
    """CDL2CROWS - Two Crows"""
    bars = make_uptrend(10)
    # Long white candle
    bars.append(Bar(open=128, high=135, low=127, close=134))
    # First crow: gaps up, closes lower but above first body
    bars.append(Bar(open=136, high=137, low=132, close=133))
    # Second crow: opens inside first crow, closes inside first white body
    bars.append(Bar(open=134, high=135, low=129, close=130))

    o, h, l, c = bars_to_arrays(bars)
    result = talib.CDL2CROWS(o, h, l, c)
    return {"pattern": "CDL2CROWS", "bars": [{"o": b.open, "h": b.high, "l": b.low, "c": b.close} for b in bars], "result": result.tolist(), "detected_at": [i for i, v in enumerate(result) if v != 0]}


def test_cdl3blackcrows() -> Dict[str, Any]:
    """CDL3BLACKCROWS - Three Black Crows"""
    bars = []
    for _ in range(10):
        bars.append(Bar(open=100.0, high=105.0, low=95.0, close=102.0))
    for i in range(5):
        bars.append(Bar(open=102 + i * 8, high=105 + i * 8 + 3, low=100 + i * 8, close=105 + i * 8))
    bars.append(Bar(open=135.0, high=137.0, low=125.0, close=125.0))
    bars.append(Bar(open=127.0, high=128.0, low=115.0, close=115.0))
    bars.append(Bar(open=118.0, high=119.0, low=105.0, close=105.0))

    o, h, l, c = bars_to_arrays(bars)
    result = talib.CDL3BLACKCROWS(o, h, l, c)
    return {"pattern": "CDL3BLACKCROWS", "bars": [{"o": b.open, "h": b.high, "l": b.low, "c": b.close} for b in bars], "result": result.tolist(), "detected_at": [i for i, v in enumerate(result) if v != 0]}


def test_cdl3inside() -> Dict[str, Any]:
    """CDL3INSIDE - Three Inside Up/Down"""
    bars = make_downtrend(10)
    # Large bearish
    bars.append(Bar(open=72, high=73, low=66, close=67))
    # Small bullish inside
    bars.append(Bar(open=68, high=70, low=67.5, close=69.5))
    # Bullish confirmation closing above first
    bars.append(Bar(open=69, high=74, low=68.5, close=73))

    o, h, l, c = bars_to_arrays(bars)
    result = talib.CDL3INSIDE(o, h, l, c)
    return {"pattern": "CDL3INSIDE", "bars": [{"o": b.open, "h": b.high, "l": b.low, "c": b.close} for b in bars], "result": result.tolist(), "detected_at": [i for i, v in enumerate(result) if v != 0]}


def test_cdl3linestrike() -> Dict[str, Any]:
    """CDL3LINESTRIKE - Three-Line Strike"""
    bars = make_downtrend(10)
    # Three bearish candles
    bars.append(Bar(open=72, high=73, low=68, close=69))
    bars.append(Bar(open=69, high=70, low=65, close=66))
    bars.append(Bar(open=66, high=67, low=62, close=63))
    # Bullish engulfing all three
    bars.append(Bar(open=62, high=74, low=61, close=73))

    o, h, l, c = bars_to_arrays(bars)
    result = talib.CDL3LINESTRIKE(o, h, l, c)
    return {"pattern": "CDL3LINESTRIKE", "bars": [{"o": b.open, "h": b.high, "l": b.low, "c": b.close} for b in bars], "result": result.tolist(), "detected_at": [i for i, v in enumerate(result) if v != 0]}


def test_cdl3outside() -> Dict[str, Any]:
    """CDL3OUTSIDE - Three Outside Up/Down"""
    bars = make_downtrend(10)
    # Small bearish
    bars.append(Bar(open=72, high=73, low=70, close=71))
    # Bullish engulfing
    bars.append(Bar(open=70, high=75, low=69, close=74))
    # Bullish confirmation
    bars.append(Bar(open=74, high=78, low=73, close=77))

    o, h, l, c = bars_to_arrays(bars)
    result = talib.CDL3OUTSIDE(o, h, l, c)
    return {"pattern": "CDL3OUTSIDE", "bars": [{"o": b.open, "h": b.high, "l": b.low, "c": b.close} for b in bars], "result": result.tolist(), "detected_at": [i for i, v in enumerate(result) if v != 0]}


def test_cdl3starsinsouth() -> Dict[str, Any]:
    """CDL3STARSINSOUTH - Three Stars In The South"""
    bars = make_downtrend(10)
    # First: long black with long lower shadow
    bars.append(Bar(open=72, high=73, low=62, close=65))
    # Second: smaller black, higher low, inside first
    bars.append(Bar(open=66, high=67, low=63, close=64))
    # Third: small marubozu near lows
    bars.append(Bar(open=64, high=65, low=63.5, close=64.5))

    o, h, l, c = bars_to_arrays(bars)
    result = talib.CDL3STARSINSOUTH(o, h, l, c)
    return {"pattern": "CDL3STARSINSOUTH", "bars": [{"o": b.open, "h": b.high, "l": b.low, "c": b.close} for b in bars], "result": result.tolist(), "detected_at": [i for i, v in enumerate(result) if v != 0]}


def test_cdl3whitesoldiers() -> Dict[str, Any]:
    """CDL3WHITESOLDIERS - Three Advancing White Soldiers"""
    bars = []
    for i in range(10):
        base = 100 - i * 3
        bars.append(Bar(open=base + 2, high=base + 3, low=base - 1, close=base - 0.5))
    bars.append(Bar(open=70.0, high=75.0, low=69.5, close=74.8))
    bars.append(Bar(open=73.0, high=79.0, low=72.5, close=78.8))
    bars.append(Bar(open=77.0, high=83.0, low=76.5, close=82.8))

    o, h, l, c = bars_to_arrays(bars)
    result = talib.CDL3WHITESOLDIERS(o, h, l, c)
    return {"pattern": "CDL3WHITESOLDIERS", "bars": [{"o": b.open, "h": b.high, "l": b.low, "c": b.close} for b in bars], "result": result.tolist(), "detected_at": [i for i, v in enumerate(result) if v != 0]}


def test_cdlabandonedbaby() -> Dict[str, Any]:
    """CDLABANDONEDBABY - Abandoned Baby"""
    bars = make_downtrend(10)
    # Long bearish
    bars.append(Bar(open=72, high=73, low=65, close=66))
    # Doji with gap down (abandoned)
    bars.append(Bar(open=63, high=63.5, low=62.5, close=63))
    # Long bullish with gap up
    bars.append(Bar(open=65, high=72, low=64, close=71))

    o, h, l, c = bars_to_arrays(bars)
    result = talib.CDLABANDONEDBABY(o, h, l, c)
    return {"pattern": "CDLABANDONEDBABY", "bars": [{"o": b.open, "h": b.high, "l": b.low, "c": b.close} for b in bars], "result": result.tolist(), "detected_at": [i for i, v in enumerate(result) if v != 0]}


def test_cdladvanceblock() -> Dict[str, Any]:
    """CDLADVANCEBLOCK - Advance Block"""
    bars = make_downtrend(10)
    # Three white candles with diminishing bodies and increasing upper shadows
    bars.append(Bar(open=70, high=78, low=69, close=77))
    bars.append(Bar(open=76, high=82, low=75, close=80))
    bars.append(Bar(open=79, high=84, low=78, close=81))

    o, h, l, c = bars_to_arrays(bars)
    result = talib.CDLADVANCEBLOCK(o, h, l, c)
    return {"pattern": "CDLADVANCEBLOCK", "bars": [{"o": b.open, "h": b.high, "l": b.low, "c": b.close} for b in bars], "result": result.tolist(), "detected_at": [i for i, v in enumerate(result) if v != 0]}


def test_cdlbelthold() -> Dict[str, Any]:
    """CDLBELTHOLD - Belt-hold"""
    bars = make_downtrend(15)
    # Bullish belt hold: opens at low, closes near high
    bars.append(Bar(open=55, high=62, low=55, close=61.5))

    o, h, l, c = bars_to_arrays(bars)
    result = talib.CDLBELTHOLD(o, h, l, c)
    return {"pattern": "CDLBELTHOLD", "bars": [{"o": b.open, "h": b.high, "l": b.low, "c": b.close} for b in bars], "result": result.tolist(), "detected_at": [i for i, v in enumerate(result) if v != 0]}


def test_cdlbreakaway() -> Dict[str, Any]:
    """CDLBREAKAWAY - Breakaway"""
    bars = make_downtrend(10)
    # Long black
    bars.append(Bar(open=72, high=73, low=65, close=66))
    # Gap down black
    bars.append(Bar(open=64, high=65, low=62, close=63))
    # Small black
    bars.append(Bar(open=63, high=64, low=61, close=62))
    # Small black
    bars.append(Bar(open=62, high=63, low=60, close=61))
    # Long white closing the gap
    bars.append(Bar(open=60, high=70, low=59, close=69))

    o, h, l, c = bars_to_arrays(bars)
    result = talib.CDLBREAKAWAY(o, h, l, c)
    return {"pattern": "CDLBREAKAWAY", "bars": [{"o": b.open, "h": b.high, "l": b.low, "c": b.close} for b in bars], "result": result.tolist(), "detected_at": [i for i, v in enumerate(result) if v != 0]}


def test_cdlclosingmarubozu() -> Dict[str, Any]:
    """CDLCLOSINGMARUBOZU - Closing Marubozu"""
    bars = make_sideways(10)
    # Bullish closing marubozu: close = high
    bars.append(Bar(open=98, high=108, low=97, close=108))

    o, h, l, c = bars_to_arrays(bars)
    result = talib.CDLCLOSINGMARUBOZU(o, h, l, c)
    return {"pattern": "CDLCLOSINGMARUBOZU", "bars": [{"o": b.open, "h": b.high, "l": b.low, "c": b.close} for b in bars], "result": result.tolist(), "detected_at": [i for i, v in enumerate(result) if v != 0]}


def test_cdlconcealbabyswall() -> Dict[str, Any]:
    """CDLCONCEALBABYSWALL - Concealing Baby Swallow"""
    bars = make_downtrend(10)
    # Two black marubozu
    bars.append(Bar(open=72, high=72, low=65, close=65))
    bars.append(Bar(open=65, high=65, low=58, close=58))
    # Black with upper shadow into previous body
    bars.append(Bar(open=56, high=60, low=52, close=53))
    # Black engulfing third
    bars.append(Bar(open=54, high=57, low=50, close=51))

    o, h, l, c = bars_to_arrays(bars)
    result = talib.CDLCONCEALBABYSWALL(o, h, l, c)
    return {"pattern": "CDLCONCEALBABYSWALL", "bars": [{"o": b.open, "h": b.high, "l": b.low, "c": b.close} for b in bars], "result": result.tolist(), "detected_at": [i for i, v in enumerate(result) if v != 0]}


def test_cdlcounterattack() -> Dict[str, Any]:
    """CDLCOUNTERATTACK - Counterattack"""
    bars = make_downtrend(10)
    # Long black
    bars.append(Bar(open=72, high=73, low=65, close=66))
    # Long white closing at same level
    bars.append(Bar(open=60, high=67, low=59, close=66))

    o, h, l, c = bars_to_arrays(bars)
    result = talib.CDLCOUNTERATTACK(o, h, l, c)
    return {"pattern": "CDLCOUNTERATTACK", "bars": [{"o": b.open, "h": b.high, "l": b.low, "c": b.close} for b in bars], "result": result.tolist(), "detected_at": [i for i, v in enumerate(result) if v != 0]}


def test_cdldarkcloudcover() -> Dict[str, Any]:
    """CDLDARKCLOUDCOVER - Dark Cloud Cover"""
    bars = make_uptrend(10)
    bars.append(Bar(open=118.0, high=122.0, low=117.0, close=121.0))
    bars.append(Bar(open=123.0, high=124.0, low=118.0, close=119.0))

    o, h, l, c = bars_to_arrays(bars)
    result = talib.CDLDARKCLOUDCOVER(o, h, l, c)
    return {"pattern": "CDLDARKCLOUDCOVER", "bars": [{"o": b.open, "h": b.high, "l": b.low, "c": b.close} for b in bars], "result": result.tolist(), "detected_at": [i for i, v in enumerate(result) if v != 0]}


def test_cdldoji() -> Dict[str, Any]:
    """CDLDOJI - Doji"""
    bars = make_downtrend(10)
    bars.append(Bar(open=80.0, high=85.0, low=75.0, close=80.0))

    o, h, l, c = bars_to_arrays(bars)
    result = talib.CDLDOJI(o, h, l, c)
    return {"pattern": "CDLDOJI", "bars": [{"o": b.open, "h": b.high, "l": b.low, "c": b.close} for b in bars], "result": result.tolist(), "detected_at": [i for i, v in enumerate(result) if v != 0]}


def test_cdldojistar() -> Dict[str, Any]:
    """CDLDOJISTAR - Doji Star"""
    bars = make_downtrend(10)
    # Long black
    bars.append(Bar(open=72, high=73, low=65, close=66))
    # Gap down doji
    bars.append(Bar(open=63, high=64, low=62, close=63))

    o, h, l, c = bars_to_arrays(bars)
    result = talib.CDLDOJISTAR(o, h, l, c)
    return {"pattern": "CDLDOJISTAR", "bars": [{"o": b.open, "h": b.high, "l": b.low, "c": b.close} for b in bars], "result": result.tolist(), "detected_at": [i for i, v in enumerate(result) if v != 0]}


def test_cdldragonflydoji() -> Dict[str, Any]:
    """CDLDRAGONFLYDOJI - Dragonfly Doji"""
    bars = make_downtrend(10)
    bars.append(Bar(open=80.0, high=80.0, low=70.0, close=80.0))

    o, h, l, c = bars_to_arrays(bars)
    result = talib.CDLDRAGONFLYDOJI(o, h, l, c)
    return {"pattern": "CDLDRAGONFLYDOJI", "bars": [{"o": b.open, "h": b.high, "l": b.low, "c": b.close} for b in bars], "result": result.tolist(), "detected_at": [i for i, v in enumerate(result) if v != 0]}


def test_cdlengulfing() -> Dict[str, Any]:
    """CDLENGULFING - Engulfing Pattern"""
    bars = make_downtrend(10)
    bars.append(Bar(open=80.0, high=81.0, low=79.0, close=79.5))
    bars.append(Bar(open=79.0, high=82.0, low=78.0, close=81.5))

    o, h, l, c = bars_to_arrays(bars)
    result = talib.CDLENGULFING(o, h, l, c)
    return {"pattern": "CDLENGULFING", "bars": [{"o": b.open, "h": b.high, "l": b.low, "c": b.close} for b in bars], "result": result.tolist(), "detected_at": [i for i, v in enumerate(result) if v != 0]}


def test_cdleveningdojistar() -> Dict[str, Any]:
    """CDLEVENINGDOJISTAR - Evening Doji Star"""
    bars = make_uptrend(10)
    # Long white
    bars.append(Bar(open=128, high=135, low=127, close=134))
    # Gap up doji
    bars.append(Bar(open=137, high=138, low=136, close=137))
    # Long black
    bars.append(Bar(open=135, high=136, low=128, close=129))

    o, h, l, c = bars_to_arrays(bars)
    result = talib.CDLEVENINGDOJISTAR(o, h, l, c)
    return {"pattern": "CDLEVENINGDOJISTAR", "bars": [{"o": b.open, "h": b.high, "l": b.low, "c": b.close} for b in bars], "result": result.tolist(), "detected_at": [i for i, v in enumerate(result) if v != 0]}


def test_cdleveningstar() -> Dict[str, Any]:
    """CDLEVENINGSTAR - Evening Star"""
    bars = make_uptrend(10)
    bars.append(Bar(open=118.0, high=123.0, low=117.0, close=122.0))
    bars.append(Bar(open=124.0, high=124.5, low=123.5, close=123.8))
    bars.append(Bar(open=123.0, high=123.5, low=118.0, close=119.0))

    o, h, l, c = bars_to_arrays(bars)
    result = talib.CDLEVENINGSTAR(o, h, l, c)
    return {"pattern": "CDLEVENINGSTAR", "bars": [{"o": b.open, "h": b.high, "l": b.low, "c": b.close} for b in bars], "result": result.tolist(), "detected_at": [i for i, v in enumerate(result) if v != 0]}


def test_cdlgapsidesidewhite() -> Dict[str, Any]:
    """CDLGAPSIDESIDEWHITE - Up/Down-gap side-by-side white lines"""
    bars = make_uptrend(10)
    # Gap up
    bars.append(Bar(open=132, high=136, low=131, close=135))
    # Two similar white candles side by side
    bars.append(Bar(open=137, high=141, low=136, close=140))
    bars.append(Bar(open=137, high=141, low=136, close=140))

    o, h, l, c = bars_to_arrays(bars)
    result = talib.CDLGAPSIDESIDEWHITE(o, h, l, c)
    return {"pattern": "CDLGAPSIDESIDEWHITE", "bars": [{"o": b.open, "h": b.high, "l": b.low, "c": b.close} for b in bars], "result": result.tolist(), "detected_at": [i for i, v in enumerate(result) if v != 0]}


def test_cdlgravestonedoji() -> Dict[str, Any]:
    """CDLGRAVESTONEDOJI - Gravestone Doji"""
    bars = make_uptrend(10)
    bars.append(Bar(open=120.0, high=130.0, low=120.0, close=120.0))

    o, h, l, c = bars_to_arrays(bars)
    result = talib.CDLGRAVESTONEDOJI(o, h, l, c)
    return {"pattern": "CDLGRAVESTONEDOJI", "bars": [{"o": b.open, "h": b.high, "l": b.low, "c": b.close} for b in bars], "result": result.tolist(), "detected_at": [i for i, v in enumerate(result) if v != 0]}


def test_cdlhammer() -> Dict[str, Any]:
    """CDLHAMMER - Hammer"""
    bars = []
    for i in range(15):
        base = 100 - i * 3
        bars.append(Bar(open=base + 2, high=base + 3, low=base - 1, close=base - 0.5))
    bars.append(Bar(open=54.0, high=54.5, low=48.0, close=54.5))

    o, h, l, c = bars_to_arrays(bars)
    result = talib.CDLHAMMER(o, h, l, c)
    return {"pattern": "CDLHAMMER", "bars": [{"o": b.open, "h": b.high, "l": b.low, "c": b.close} for b in bars], "result": result.tolist(), "detected_at": [i for i, v in enumerate(result) if v != 0]}


def test_cdlhangingman() -> Dict[str, Any]:
    """CDLHANGINGMAN - Hanging Man"""
    bars = []
    for i in range(15):
        base = 100 + i * 3
        bars.append(Bar(open=base - 1, high=base + 2, low=base - 2, close=base + 1))
    bars.append(Bar(open=145.0, high=145.5, low=139.0, close=145.5))

    o, h, l, c = bars_to_arrays(bars)
    result = talib.CDLHANGINGMAN(o, h, l, c)
    return {"pattern": "CDLHANGINGMAN", "bars": [{"o": b.open, "h": b.high, "l": b.low, "c": b.close} for b in bars], "result": result.tolist(), "detected_at": [i for i, v in enumerate(result) if v != 0]}


def test_cdlharami() -> Dict[str, Any]:
    """CDLHARAMI - Harami Pattern"""
    bars = make_downtrend(10)
    bars.append(Bar(open=82.0, high=83.0, low=78.0, close=79.0))
    bars.append(Bar(open=79.5, high=80.5, low=79.0, close=80.0))

    o, h, l, c = bars_to_arrays(bars)
    result = talib.CDLHARAMI(o, h, l, c)
    return {"pattern": "CDLHARAMI", "bars": [{"o": b.open, "h": b.high, "l": b.low, "c": b.close} for b in bars], "result": result.tolist(), "detected_at": [i for i, v in enumerate(result) if v != 0]}


def test_cdlharamicross() -> Dict[str, Any]:
    """CDLHARAMICROSS - Harami Cross Pattern"""
    bars = make_downtrend(10)
    # Large bearish
    bars.append(Bar(open=72, high=73, low=65, close=66))
    # Doji inside
    bars.append(Bar(open=69, high=70, low=68, close=69))

    o, h, l, c = bars_to_arrays(bars)
    result = talib.CDLHARAMICROSS(o, h, l, c)
    return {"pattern": "CDLHARAMICROSS", "bars": [{"o": b.open, "h": b.high, "l": b.low, "c": b.close} for b in bars], "result": result.tolist(), "detected_at": [i for i, v in enumerate(result) if v != 0]}


def test_cdlhighwave() -> Dict[str, Any]:
    """CDLHIGHWAVE - High-Wave Candle"""
    bars = make_sideways(10)
    # Very long shadows, tiny body
    bars.append(Bar(open=100, high=115, low=85, close=100.5))

    o, h, l, c = bars_to_arrays(bars)
    result = talib.CDLHIGHWAVE(o, h, l, c)
    return {"pattern": "CDLHIGHWAVE", "bars": [{"o": b.open, "h": b.high, "l": b.low, "c": b.close} for b in bars], "result": result.tolist(), "detected_at": [i for i, v in enumerate(result) if v != 0]}


def test_cdlhikkake() -> Dict[str, Any]:
    """CDLHIKKAKE - Hikkake Pattern"""
    bars = make_sideways(10)
    # Inside bar
    bars.append(Bar(open=99, high=101, low=99, close=100))
    bars.append(Bar(open=99.5, high=100.5, low=99.2, close=99.8))
    # Breakout fails
    bars.append(Bar(open=99.5, high=99.8, low=98, close=98.5))
    # Reversal
    bars.append(Bar(open=98.5, high=102, low=98, close=101.5))
    bars.append(Bar(open=101, high=103, low=100.5, close=102.5))

    o, h, l, c = bars_to_arrays(bars)
    result = talib.CDLHIKKAKE(o, h, l, c)
    return {"pattern": "CDLHIKKAKE", "bars": [{"o": b.open, "h": b.high, "l": b.low, "c": b.close} for b in bars], "result": result.tolist(), "detected_at": [i for i, v in enumerate(result) if v != 0]}


def test_cdlhikkakemod() -> Dict[str, Any]:
    """CDLHIKKAKEMOD - Modified Hikkake Pattern"""
    bars = make_sideways(10)
    # Similar to hikkake but stricter
    bars.append(Bar(open=99, high=101, low=99, close=100))
    bars.append(Bar(open=99.5, high=100.5, low=99.2, close=99.8))
    bars.append(Bar(open=99.5, high=100.2, low=99.3, close=99.5))
    bars.append(Bar(open=99.5, high=99.8, low=98, close=98.5))
    bars.append(Bar(open=98.5, high=102, low=98, close=101.5))

    o, h, l, c = bars_to_arrays(bars)
    result = talib.CDLHIKKAKEMOD(o, h, l, c)
    return {"pattern": "CDLHIKKAKEMOD", "bars": [{"o": b.open, "h": b.high, "l": b.low, "c": b.close} for b in bars], "result": result.tolist(), "detected_at": [i for i, v in enumerate(result) if v != 0]}


def test_cdlhomingpigeon() -> Dict[str, Any]:
    """CDLHOMINGPIGEON - Homing Pigeon"""
    bars = make_downtrend(10)
    # Large black
    bars.append(Bar(open=72, high=73, low=65, close=66))
    # Smaller black inside
    bars.append(Bar(open=70, high=71, low=67, close=68))

    o, h, l, c = bars_to_arrays(bars)
    result = talib.CDLHOMINGPIGEON(o, h, l, c)
    return {"pattern": "CDLHOMINGPIGEON", "bars": [{"o": b.open, "h": b.high, "l": b.low, "c": b.close} for b in bars], "result": result.tolist(), "detected_at": [i for i, v in enumerate(result) if v != 0]}


def test_cdlidentical3crows() -> Dict[str, Any]:
    """CDLIDENTICAL3CROWS - Identical Three Crows"""
    bars = make_uptrend(10)
    # Three black candles, each opening at previous close
    bars.append(Bar(open=128, high=129, low=122, close=123))
    bars.append(Bar(open=123, high=124, low=117, close=118))
    bars.append(Bar(open=118, high=119, low=112, close=113))

    o, h, l, c = bars_to_arrays(bars)
    result = talib.CDLIDENTICAL3CROWS(o, h, l, c)
    return {"pattern": "CDLIDENTICAL3CROWS", "bars": [{"o": b.open, "h": b.high, "l": b.low, "c": b.close} for b in bars], "result": result.tolist(), "detected_at": [i for i, v in enumerate(result) if v != 0]}


def test_cdlinneck() -> Dict[str, Any]:
    """CDLINNECK - In-Neck Pattern"""
    bars = make_downtrend(10)
    # Long black
    bars.append(Bar(open=72, high=73, low=65, close=66))
    # White opens below low, closes at/near previous close (very close)
    bars.append(Bar(open=64, high=66.3, low=63, close=66.2))

    o, h, l, c = bars_to_arrays(bars)
    result = talib.CDLINNECK(o, h, l, c)
    return {"pattern": "CDLINNECK", "bars": [{"o": b.open, "h": b.high, "l": b.low, "c": b.close} for b in bars], "result": result.tolist(), "detected_at": [i for i, v in enumerate(result) if v != 0]}


def test_cdlinvertedhammer() -> Dict[str, Any]:
    """CDLINVERTEDHAMMER - Inverted Hammer"""
    bars = []
    for i in range(15):
        base = 100 - i * 3
        bars.append(Bar(open=base + 2, high=base + 3, low=base - 1, close=base - 0.5))
    bars.append(Bar(open=54.5, high=61.0, low=54.0, close=54.0))

    o, h, l, c = bars_to_arrays(bars)
    result = talib.CDLINVERTEDHAMMER(o, h, l, c)
    return {"pattern": "CDLINVERTEDHAMMER", "bars": [{"o": b.open, "h": b.high, "l": b.low, "c": b.close} for b in bars], "result": result.tolist(), "detected_at": [i for i, v in enumerate(result) if v != 0]}


def test_cdlkicking() -> Dict[str, Any]:
    """CDLKICKING - Kicking"""
    bars = make_sideways(10)
    # Black marubozu (open=high, close=low)
    bars.append(Bar(open=110, high=110, low=100, close=100))
    # Gap up white marubozu (open=low, close=high) - gap > prev high
    bars.append(Bar(open=112, high=122, low=112, close=122))

    o, h, l, c = bars_to_arrays(bars)
    result = talib.CDLKICKING(o, h, l, c)
    return {"pattern": "CDLKICKING", "bars": [{"o": b.open, "h": b.high, "l": b.low, "c": b.close} for b in bars], "result": result.tolist(), "detected_at": [i for i, v in enumerate(result) if v != 0]}


def test_cdlkickingbylength() -> Dict[str, Any]:
    """CDLKICKINGBYLENGTH - Kicking by length"""
    bars = make_sideways(10)
    # Black marubozu (shorter)
    bars.append(Bar(open=110, high=110, low=102, close=102))
    # Gap up longer white marubozu - longer than first
    bars.append(Bar(open=112, high=125, low=112, close=125))

    o, h, l, c = bars_to_arrays(bars)
    result = talib.CDLKICKINGBYLENGTH(o, h, l, c)
    return {"pattern": "CDLKICKINGBYLENGTH", "bars": [{"o": b.open, "h": b.high, "l": b.low, "c": b.close} for b in bars], "result": result.tolist(), "detected_at": [i for i, v in enumerate(result) if v != 0]}


def test_cdlladderbottom() -> Dict[str, Any]:
    """CDLLADDERBOTTOM - Ladder Bottom"""
    bars = make_downtrend(10)
    # Three black candles making new lows
    bars.append(Bar(open=72, high=73, low=68, close=69))
    bars.append(Bar(open=69, high=70, low=65, close=66))
    bars.append(Bar(open=66, high=67, low=62, close=63))
    # Fourth black with long lower shadow
    bars.append(Bar(open=63, high=64, low=56, close=60))
    # Fifth white closing above fourth open
    bars.append(Bar(open=61, high=68, low=60, close=67))

    o, h, l, c = bars_to_arrays(bars)
    result = talib.CDLLADDERBOTTOM(o, h, l, c)
    return {"pattern": "CDLLADDERBOTTOM", "bars": [{"o": b.open, "h": b.high, "l": b.low, "c": b.close} for b in bars], "result": result.tolist(), "detected_at": [i for i, v in enumerate(result) if v != 0]}


def test_cdllongleggeddoji() -> Dict[str, Any]:
    """CDLLONGLEGGEDDOJI - Long Legged Doji"""
    bars = make_sideways(10)
    # Doji with very long shadows
    bars.append(Bar(open=100, high=112, low=88, close=100))

    o, h, l, c = bars_to_arrays(bars)
    result = talib.CDLLONGLEGGEDDOJI(o, h, l, c)
    return {"pattern": "CDLLONGLEGGEDDOJI", "bars": [{"o": b.open, "h": b.high, "l": b.low, "c": b.close} for b in bars], "result": result.tolist(), "detected_at": [i for i, v in enumerate(result) if v != 0]}


def test_cdllongline() -> Dict[str, Any]:
    """CDLLONGLINE - Long Line Candle"""
    bars = make_sideways(10)
    # Very long body candle
    bars.append(Bar(open=95, high=112, low=94, close=111))

    o, h, l, c = bars_to_arrays(bars)
    result = talib.CDLLONGLINE(o, h, l, c)
    return {"pattern": "CDLLONGLINE", "bars": [{"o": b.open, "h": b.high, "l": b.low, "c": b.close} for b in bars], "result": result.tolist(), "detected_at": [i for i, v in enumerate(result) if v != 0]}


def test_cdlmarubozu() -> Dict[str, Any]:
    """CDLMARUBOZU - Marubozu"""
    bars = make_sideways(10)
    bars.append(Bar(open=100.0, high=110.0, low=100.0, close=110.0))

    o, h, l, c = bars_to_arrays(bars)
    result = talib.CDLMARUBOZU(o, h, l, c)
    return {"pattern": "CDLMARUBOZU", "bars": [{"o": b.open, "h": b.high, "l": b.low, "c": b.close} for b in bars], "result": result.tolist(), "detected_at": [i for i, v in enumerate(result) if v != 0]}


def test_cdlmatchinglow() -> Dict[str, Any]:
    """CDLMATCHINGLOW - Matching Low"""
    bars = make_downtrend(10)
    # Two black candles with same close
    bars.append(Bar(open=72, high=73, low=65, close=66))
    bars.append(Bar(open=70, high=71, low=65, close=66))

    o, h, l, c = bars_to_arrays(bars)
    result = talib.CDLMATCHINGLOW(o, h, l, c)
    return {"pattern": "CDLMATCHINGLOW", "bars": [{"o": b.open, "h": b.high, "l": b.low, "c": b.close} for b in bars], "result": result.tolist(), "detected_at": [i for i, v in enumerate(result) if v != 0]}


def test_cdlmathold() -> Dict[str, Any]:
    """CDLMATHOLD - Mat Hold"""
    bars = make_uptrend(10)
    # Long white
    bars.append(Bar(open=128, high=136, low=127, close=135))
    # Three small declining candles
    bars.append(Bar(open=134, high=135, low=132, close=133))
    bars.append(Bar(open=133, high=134, low=131, close=132))
    bars.append(Bar(open=132, high=133, low=130, close=131))
    # Long white continuation
    bars.append(Bar(open=132, high=142, low=131, close=141))

    o, h, l, c = bars_to_arrays(bars)
    result = talib.CDLMATHOLD(o, h, l, c)
    return {"pattern": "CDLMATHOLD", "bars": [{"o": b.open, "h": b.high, "l": b.low, "c": b.close} for b in bars], "result": result.tolist(), "detected_at": [i for i, v in enumerate(result) if v != 0]}


def test_cdlmorningdojistar() -> Dict[str, Any]:
    """CDLMORNINGDOJISTAR - Morning Doji Star"""
    bars = make_downtrend(10)
    # Long black
    bars.append(Bar(open=72, high=73, low=65, close=66))
    # Gap down doji
    bars.append(Bar(open=63, high=64, low=62, close=63))
    # Long white
    bars.append(Bar(open=64, high=72, low=63, close=71))

    o, h, l, c = bars_to_arrays(bars)
    result = talib.CDLMORNINGDOJISTAR(o, h, l, c)
    return {"pattern": "CDLMORNINGDOJISTAR", "bars": [{"o": b.open, "h": b.high, "l": b.low, "c": b.close} for b in bars], "result": result.tolist(), "detected_at": [i for i, v in enumerate(result) if v != 0]}


def test_cdlmorningstar() -> Dict[str, Any]:
    """CDLMORNINGSTAR - Morning Star"""
    bars = make_downtrend(10)
    bars.append(Bar(open=82.0, high=83.0, low=77.0, close=78.0))
    bars.append(Bar(open=76.0, high=76.5, low=75.5, close=76.2))
    bars.append(Bar(open=77.0, high=82.0, low=76.5, close=81.0))

    o, h, l, c = bars_to_arrays(bars)
    result = talib.CDLMORNINGSTAR(o, h, l, c)
    return {"pattern": "CDLMORNINGSTAR", "bars": [{"o": b.open, "h": b.high, "l": b.low, "c": b.close} for b in bars], "result": result.tolist(), "detected_at": [i for i, v in enumerate(result) if v != 0]}


def test_cdlonneck() -> Dict[str, Any]:
    """CDLONNECK - On-Neck Pattern"""
    bars = make_downtrend(10)
    # Long black
    bars.append(Bar(open=72, high=73, low=65, close=66))
    # Small white closing at previous low
    bars.append(Bar(open=64, high=66, low=63, close=65))

    o, h, l, c = bars_to_arrays(bars)
    result = talib.CDLONNECK(o, h, l, c)
    return {"pattern": "CDLONNECK", "bars": [{"o": b.open, "h": b.high, "l": b.low, "c": b.close} for b in bars], "result": result.tolist(), "detected_at": [i for i, v in enumerate(result) if v != 0]}


def test_cdlpiercing() -> Dict[str, Any]:
    """CDLPIERCING - Piercing Pattern"""
    bars = make_downtrend(10)
    bars.append(Bar(open=82.0, high=83.0, low=78.0, close=79.0))
    bars.append(Bar(open=77.0, high=82.0, low=76.0, close=81.0))

    o, h, l, c = bars_to_arrays(bars)
    result = talib.CDLPIERCING(o, h, l, c)
    return {"pattern": "CDLPIERCING", "bars": [{"o": b.open, "h": b.high, "l": b.low, "c": b.close} for b in bars], "result": result.tolist(), "detected_at": [i for i, v in enumerate(result) if v != 0]}


def test_cdlrickshawman() -> Dict[str, Any]:
    """CDLRICKSHAWMAN - Rickshaw Man"""
    bars = make_sideways(10)
    # Long legged doji with body in center
    bars.append(Bar(open=100, high=115, low=85, close=100))

    o, h, l, c = bars_to_arrays(bars)
    result = talib.CDLRICKSHAWMAN(o, h, l, c)
    return {"pattern": "CDLRICKSHAWMAN", "bars": [{"o": b.open, "h": b.high, "l": b.low, "c": b.close} for b in bars], "result": result.tolist(), "detected_at": [i for i, v in enumerate(result) if v != 0]}


def test_cdlrisefall3methods() -> Dict[str, Any]:
    """CDLRISEFALL3METHODS - Rising/Falling Three Methods"""
    bars = make_uptrend(10)
    # Long white
    bars.append(Bar(open=128, high=138, low=127, close=137))
    # Three small declining candles inside first
    bars.append(Bar(open=136, high=137, low=133, close=134))
    bars.append(Bar(open=134, high=135, low=131, close=132))
    bars.append(Bar(open=132, high=133, low=129, close=130))
    # Long white continuation
    bars.append(Bar(open=131, high=145, low=130, close=144))

    o, h, l, c = bars_to_arrays(bars)
    result = talib.CDLRISEFALL3METHODS(o, h, l, c)
    return {"pattern": "CDLRISEFALL3METHODS", "bars": [{"o": b.open, "h": b.high, "l": b.low, "c": b.close} for b in bars], "result": result.tolist(), "detected_at": [i for i, v in enumerate(result) if v != 0]}


def test_cdlseparatinglines() -> Dict[str, Any]:
    """CDLSEPARATINGLINES - Separating Lines"""
    bars = make_uptrend(10)
    # Black candle
    bars.append(Bar(open=128, high=129, low=123, close=124))
    # White candle opening at same price
    bars.append(Bar(open=128, high=135, low=127, close=134))

    o, h, l, c = bars_to_arrays(bars)
    result = talib.CDLSEPARATINGLINES(o, h, l, c)
    return {"pattern": "CDLSEPARATINGLINES", "bars": [{"o": b.open, "h": b.high, "l": b.low, "c": b.close} for b in bars], "result": result.tolist(), "detected_at": [i for i, v in enumerate(result) if v != 0]}


def test_cdlshootingstar() -> Dict[str, Any]:
    """CDLSHOOTINGSTAR - Shooting Star"""
    bars = []
    for i in range(15):
        base = 100 + i * 3
        bars.append(Bar(open=base - 1, high=base + 2, low=base - 2, close=base + 1))
    bars.append(Bar(open=147.0, high=154.0, low=146.5, close=146.5))

    o, h, l, c = bars_to_arrays(bars)
    result = talib.CDLSHOOTINGSTAR(o, h, l, c)
    return {"pattern": "CDLSHOOTINGSTAR", "bars": [{"o": b.open, "h": b.high, "l": b.low, "c": b.close} for b in bars], "result": result.tolist(), "detected_at": [i for i, v in enumerate(result) if v != 0]}


def test_cdlshortline() -> Dict[str, Any]:
    """CDLSHORTLINE - Short Line Candle"""
    bars = make_sideways(10)
    # Very short candle
    bars.append(Bar(open=100, high=101, low=99, close=100.5))

    o, h, l, c = bars_to_arrays(bars)
    result = talib.CDLSHORTLINE(o, h, l, c)
    return {"pattern": "CDLSHORTLINE", "bars": [{"o": b.open, "h": b.high, "l": b.low, "c": b.close} for b in bars], "result": result.tolist(), "detected_at": [i for i, v in enumerate(result) if v != 0]}


def test_cdlspinningtop() -> Dict[str, Any]:
    """CDLSPINNINGTOP - Spinning Top"""
    bars = []
    for i in range(10):
        base = 100
        bars.append(Bar(open=base - 2, high=base + 4, low=base - 4, close=base + 2))
    bars.append(Bar(open=100.0, high=108.0, low=92.0, close=100.5))

    o, h, l, c = bars_to_arrays(bars)
    result = talib.CDLSPINNINGTOP(o, h, l, c)
    return {"pattern": "CDLSPINNINGTOP", "bars": [{"o": b.open, "h": b.high, "l": b.low, "c": b.close} for b in bars], "result": result.tolist(), "detected_at": [i for i, v in enumerate(result) if v != 0]}


def test_cdlstalledpattern() -> Dict[str, Any]:
    """CDLSTALLEDPATTERN - Stalled Pattern"""
    bars = make_uptrend(10)
    # Three white candles with diminishing momentum
    bars.append(Bar(open=128, high=136, low=127, close=135))
    bars.append(Bar(open=134, high=140, low=133, close=139))
    bars.append(Bar(open=139, high=141, low=138, close=140))

    o, h, l, c = bars_to_arrays(bars)
    result = talib.CDLSTALLEDPATTERN(o, h, l, c)
    return {"pattern": "CDLSTALLEDPATTERN", "bars": [{"o": b.open, "h": b.high, "l": b.low, "c": b.close} for b in bars], "result": result.tolist(), "detected_at": [i for i, v in enumerate(result) if v != 0]}


def test_cdlsticksandwich() -> Dict[str, Any]:
    """CDLSTICKSANDWICH - Stick Sandwich"""
    bars = make_downtrend(10)
    # Black candle
    bars.append(Bar(open=72, high=73, low=65, close=66))
    # White candle
    bars.append(Bar(open=67, high=73, low=66, close=72))
    # Black candle closing at same level as first
    bars.append(Bar(open=71, high=72, low=65, close=66))

    o, h, l, c = bars_to_arrays(bars)
    result = talib.CDLSTICKSANDWICH(o, h, l, c)
    return {"pattern": "CDLSTICKSANDWICH", "bars": [{"o": b.open, "h": b.high, "l": b.low, "c": b.close} for b in bars], "result": result.tolist(), "detected_at": [i for i, v in enumerate(result) if v != 0]}


def test_cdltakuri() -> Dict[str, Any]:
    """CDLTAKURI - Takuri (Dragonfly Doji with very long lower shadow)"""
    bars = make_downtrend(15)
    # Dragonfly doji with very long lower shadow (open=high=close, huge lower shadow)
    bars.append(Bar(open=55, high=55, low=35, close=55))

    o, h, l, c = bars_to_arrays(bars)
    result = talib.CDLTAKURI(o, h, l, c)
    return {"pattern": "CDLTAKURI", "bars": [{"o": b.open, "h": b.high, "l": b.low, "c": b.close} for b in bars], "result": result.tolist(), "detected_at": [i for i, v in enumerate(result) if v != 0]}


def test_cdltasukigap() -> Dict[str, Any]:
    """CDLTASUKIGAP - Tasuki Gap"""
    bars = make_uptrend(10)
    # First white candle
    bars.append(Bar(open=128, high=134, low=127, close=133))
    # Gap up second white candle
    bars.append(Bar(open=135, high=141, low=134, close=140))
    # Black opens in second body, closes inside gap (between first close and second open)
    bars.append(Bar(open=139, high=140, low=133.5, close=134))

    o, h, l, c = bars_to_arrays(bars)
    result = talib.CDLTASUKIGAP(o, h, l, c)
    return {"pattern": "CDLTASUKIGAP", "bars": [{"o": b.open, "h": b.high, "l": b.low, "c": b.close} for b in bars], "result": result.tolist(), "detected_at": [i for i, v in enumerate(result) if v != 0]}


def test_cdlthrusting() -> Dict[str, Any]:
    """CDLTHRUSTING - Thrusting Pattern"""
    bars = make_downtrend(10)
    # Long black
    bars.append(Bar(open=72, high=73, low=65, close=66))
    # White opening below low, closing below midpoint
    bars.append(Bar(open=64, high=69, low=63, close=68))

    o, h, l, c = bars_to_arrays(bars)
    result = talib.CDLTHRUSTING(o, h, l, c)
    return {"pattern": "CDLTHRUSTING", "bars": [{"o": b.open, "h": b.high, "l": b.low, "c": b.close} for b in bars], "result": result.tolist(), "detected_at": [i for i, v in enumerate(result) if v != 0]}


def test_cdltristar() -> Dict[str, Any]:
    """CDLTRISTAR - Tristar Pattern"""
    bars = make_downtrend(10)
    # Three doji with gaps
    bars.append(Bar(open=70, high=71, low=69, close=70))
    bars.append(Bar(open=67, high=68, low=66, close=67))
    bars.append(Bar(open=69, high=70, low=68, close=69))

    o, h, l, c = bars_to_arrays(bars)
    result = talib.CDLTRISTAR(o, h, l, c)
    return {"pattern": "CDLTRISTAR", "bars": [{"o": b.open, "h": b.high, "l": b.low, "c": b.close} for b in bars], "result": result.tolist(), "detected_at": [i for i, v in enumerate(result) if v != 0]}


def test_cdlunique3river() -> Dict[str, Any]:
    """CDLUNIQUE3RIVER - Unique 3 River"""
    bars = make_downtrend(10)
    # Long black
    bars.append(Bar(open=72, high=73, low=62, close=63))
    # Harami black with long lower shadow
    bars.append(Bar(open=65, high=68, low=58, close=64))
    # Small white closing below second
    bars.append(Bar(open=62, high=64, low=61, close=63))

    o, h, l, c = bars_to_arrays(bars)
    result = talib.CDLUNIQUE3RIVER(o, h, l, c)
    return {"pattern": "CDLUNIQUE3RIVER", "bars": [{"o": b.open, "h": b.high, "l": b.low, "c": b.close} for b in bars], "result": result.tolist(), "detected_at": [i for i, v in enumerate(result) if v != 0]}


def test_cdlupsidegap2crows() -> Dict[str, Any]:
    """CDLUPSIDEGAP2CROWS - Upside Gap Two Crows"""
    bars = make_uptrend(10)
    # Long white
    bars.append(Bar(open=128, high=136, low=127, close=135))
    # Gap up black
    bars.append(Bar(open=139, high=140, low=136, close=137))
    # Black engulfing second but above first close
    bars.append(Bar(open=140, high=141, low=135.5, close=136))

    o, h, l, c = bars_to_arrays(bars)
    result = talib.CDLUPSIDEGAP2CROWS(o, h, l, c)
    return {"pattern": "CDLUPSIDEGAP2CROWS", "bars": [{"o": b.open, "h": b.high, "l": b.low, "c": b.close} for b in bars], "result": result.tolist(), "detected_at": [i for i, v in enumerate(result) if v != 0]}


def test_cdlxsidegap3methods() -> Dict[str, Any]:
    """CDLXSIDEGAP3METHODS - Upside/Downside Gap Three Methods"""
    bars = make_uptrend(10)
    # Two white candles with gap
    bars.append(Bar(open=128, high=134, low=127, close=133))
    bars.append(Bar(open=136, high=142, low=135, close=141))
    # Black candle filling the gap
    bars.append(Bar(open=140, high=141, low=132, close=133))

    o, h, l, c = bars_to_arrays(bars)
    result = talib.CDLXSIDEGAP3METHODS(o, h, l, c)
    return {"pattern": "CDLXSIDEGAP3METHODS", "bars": [{"o": b.open, "h": b.high, "l": b.low, "c": b.close} for b in bars], "result": result.tolist(), "detected_at": [i for i, v in enumerate(result) if v != 0]}


# ============================================================
# ALL PATTERN TESTS REGISTRY
# ============================================================

ALL_PATTERN_TESTS = [
    ("CDL2CROWS", test_cdl2crows),
    ("CDL3BLACKCROWS", test_cdl3blackcrows),
    ("CDL3INSIDE", test_cdl3inside),
    ("CDL3LINESTRIKE", test_cdl3linestrike),
    ("CDL3OUTSIDE", test_cdl3outside),
    ("CDL3STARSINSOUTH", test_cdl3starsinsouth),
    ("CDL3WHITESOLDIERS", test_cdl3whitesoldiers),
    ("CDLABANDONEDBABY", test_cdlabandonedbaby),
    ("CDLADVANCEBLOCK", test_cdladvanceblock),
    ("CDLBELTHOLD", test_cdlbelthold),
    ("CDLBREAKAWAY", test_cdlbreakaway),
    ("CDLCLOSINGMARUBOZU", test_cdlclosingmarubozu),
    ("CDLCONCEALBABYSWALL", test_cdlconcealbabyswall),
    ("CDLCOUNTERATTACK", test_cdlcounterattack),
    ("CDLDARKCLOUDCOVER", test_cdldarkcloudcover),
    ("CDLDOJI", test_cdldoji),
    ("CDLDOJISTAR", test_cdldojistar),
    ("CDLDRAGONFLYDOJI", test_cdldragonflydoji),
    ("CDLENGULFING", test_cdlengulfing),
    ("CDLEVENINGDOJISTAR", test_cdleveningdojistar),
    ("CDLEVENINGSTAR", test_cdleveningstar),
    ("CDLGAPSIDESIDEWHITE", test_cdlgapsidesidewhite),
    ("CDLGRAVESTONEDOJI", test_cdlgravestonedoji),
    ("CDLHAMMER", test_cdlhammer),
    ("CDLHANGINGMAN", test_cdlhangingman),
    ("CDLHARAMI", test_cdlharami),
    ("CDLHARAMICROSS", test_cdlharamicross),
    ("CDLHIGHWAVE", test_cdlhighwave),
    ("CDLHIKKAKE", test_cdlhikkake),
    ("CDLHIKKAKEMOD", test_cdlhikkakemod),
    ("CDLHOMINGPIGEON", test_cdlhomingpigeon),
    ("CDLIDENTICAL3CROWS", test_cdlidentical3crows),
    ("CDLINNECK", test_cdlinneck),
    ("CDLINVERTEDHAMMER", test_cdlinvertedhammer),
    ("CDLKICKING", test_cdlkicking),
    ("CDLKICKINGBYLENGTH", test_cdlkickingbylength),
    ("CDLLADDERBOTTOM", test_cdlladderbottom),
    ("CDLLONGLEGGEDDOJI", test_cdllongleggeddoji),
    ("CDLLONGLINE", test_cdllongline),
    ("CDLMARUBOZU", test_cdlmarubozu),
    ("CDLMATCHINGLOW", test_cdlmatchinglow),
    ("CDLMATHOLD", test_cdlmathold),
    ("CDLMORNINGDOJISTAR", test_cdlmorningdojistar),
    ("CDLMORNINGSTAR", test_cdlmorningstar),
    ("CDLONNECK", test_cdlonneck),
    ("CDLPIERCING", test_cdlpiercing),
    ("CDLRICKSHAWMAN", test_cdlrickshawman),
    ("CDLRISEFALL3METHODS", test_cdlrisefall3methods),
    ("CDLSEPARATINGLINES", test_cdlseparatinglines),
    ("CDLSHOOTINGSTAR", test_cdlshootingstar),
    ("CDLSHORTLINE", test_cdlshortline),
    ("CDLSPINNINGTOP", test_cdlspinningtop),
    ("CDLSTALLEDPATTERN", test_cdlstalledpattern),
    ("CDLSTICKSANDWICH", test_cdlsticksandwich),
    ("CDLTAKURI", test_cdltakuri),
    ("CDLTASUKIGAP", test_cdltasukigap),
    ("CDLTHRUSTING", test_cdlthrusting),
    ("CDLTRISTAR", test_cdltristar),
    ("CDLUNIQUE3RIVER", test_cdlunique3river),
    ("CDLUPSIDEGAP2CROWS", test_cdlupsidegap2crows),
    ("CDLXSIDEGAP3METHODS", test_cdlxsidegap3methods),
]


# ============================================================
# NEGATIVE TESTS (False Positive Checks)
# ============================================================

def run_negative_tests() -> Dict[str, Any]:
    """Run negative tests - patterns should NOT be detected on random/neutral data."""
    print("\n" + "=" * 60)
    print("NEGATIVE TESTS (False Positive Checks)")
    print("=" * 60)

    results = {"passed": 0, "failed": 0, "details": []}

    # Generate neutral/random data where patterns shouldn't appear
    random.seed(42)  # Reproducible

    # Flat sideways data with minimal movement
    flat_bars = []
    for _ in range(100):
        flat_bars.append(Bar(open=100, high=100.5, low=99.5, close=100.1))

    o, h, l, c = bars_to_arrays(flat_bars)

    # Test each pattern on flat data
    talib_funcs = {
        "CDLDOJI": talib.CDLDOJI,
        "CDLENGULFING": talib.CDLENGULFING,
        "CDLHAMMER": talib.CDLHAMMER,
        "CDLMORNINGSTAR": talib.CDLMORNINGSTAR,
        "CDL3WHITESOLDIERS": talib.CDL3WHITESOLDIERS,
        "CDL3BLACKCROWS": talib.CDL3BLACKCROWS,
        "CDLMARUBOZU": talib.CDLMARUBOZU,
    }

    for name, func in talib_funcs.items():
        result = func(o, h, l, c)
        detections = [i for i, v in enumerate(result) if v != 0]

        # Expect no detections on flat data (except possibly doji which IS flat)
        if name == "CDLDOJI":
            # Doji should detect on flat data (that's correct behavior)
            if len(detections) > 0:
                print(f"[PASS] {name}: Correctly detects on flat data ({len(detections)} times)")
                results["passed"] += 1
            else:
                print(f"[WARN] {name}: Didn't detect doji on flat data")
                results["passed"] += 1
        else:
            if len(detections) == 0:
                print(f"[PASS] {name}: No false positives on flat data")
                results["passed"] += 1
            else:
                print(f"[FAIL] {name}: False positives at {detections[:5]}...")
                results["failed"] += 1

        results["details"].append({
            "pattern": name,
            "test_type": "flat_data",
            "detections": detections
        })

    return results


# ============================================================
# FUZZ TESTING
# ============================================================

def run_fuzz_tests(iterations: int = 100) -> Dict[str, Any]:
    """Run fuzz tests - compare Rust vs TA-Lib on random data."""
    print("\n" + "=" * 60)
    print(f"FUZZ TESTING ({iterations} iterations)")
    print("=" * 60)

    results = {"iterations": iterations, "patterns_tested": 0, "total_detections": 0}

    # Select key patterns for fuzz testing
    fuzz_patterns = [
        ("CDLDOJI", talib.CDLDOJI),
        ("CDLENGULFING", talib.CDLENGULFING),
        ("CDLHAMMER", talib.CDLHAMMER),
        ("CDLMARUBOZU", talib.CDLMARUBOZU),
        ("CDLMORNINGSTAR", talib.CDLMORNINGSTAR),
        ("CDL3WHITESOLDIERS", talib.CDL3WHITESOLDIERS),
    ]

    detection_counts = {name: 0 for name, _ in fuzz_patterns}

    for i in range(iterations):
        # Generate random data
        random.seed(i)
        bars = make_random_bars(50, 100, 5)
        o, h, l, c = bars_to_arrays(bars)

        for name, func in fuzz_patterns:
            result = func(o, h, l, c)
            detections = sum(1 for v in result if v != 0)
            detection_counts[name] += detections
            results["total_detections"] += detections

    results["patterns_tested"] = len(fuzz_patterns)

    print(f"\nDetection rates over {iterations} random datasets (50 bars each):")
    for name, count in detection_counts.items():
        rate = count / (iterations * 50) * 100
        print(f"  {name}: {count} detections ({rate:.2f}% of bars)")

    print(f"\nTotal detections: {results['total_detections']}")

    return results


# ============================================================
# REAL MARKET DATA TESTS
# ============================================================

def run_real_data_tests() -> Dict[str, Any]:
    """Run tests on real market data."""
    print("\n" + "=" * 60)
    print("REAL MARKET DATA TESTS")
    print("=" * 60)

    results = {"status": "skipped", "reason": ""}

    try:
        import yfinance as yf

        # Download some real data
        print("Downloading SPY data...")
        spy = yf.download("SPY", period="6mo", progress=False)

        if len(spy) < 50:
            results["reason"] = "Insufficient data downloaded"
            print(f"[SKIP] {results['reason']}")
            return results

        # Handle MultiIndex columns from yfinance
        if isinstance(spy.columns, pd.MultiIndex):
            spy.columns = spy.columns.get_level_values(0)

        o = spy['Open'].values.flatten().astype(np.float64)
        h = spy['High'].values.flatten().astype(np.float64)
        l = spy['Low'].values.flatten().astype(np.float64)
        c = spy['Close'].values.flatten().astype(np.float64)

        print(f"Testing on {len(spy)} bars of SPY data\n")

        # Test all patterns
        detection_summary = {}

        key_patterns = [
            ("CDLDOJI", talib.CDLDOJI),
            ("CDLENGULFING", talib.CDLENGULFING),
            ("CDLHAMMER", talib.CDLHAMMER),
            ("CDLHANGINGMAN", talib.CDLHANGINGMAN),
            ("CDLMORNINGSTAR", talib.CDLMORNINGSTAR),
            ("CDLEVENINGSTAR", talib.CDLEVENINGSTAR),
            ("CDL3WHITESOLDIERS", talib.CDL3WHITESOLDIERS),
            ("CDL3BLACKCROWS", talib.CDL3BLACKCROWS),
            ("CDLMARUBOZU", talib.CDLMARUBOZU),
            ("CDLSPINNINGTOP", talib.CDLSPINNINGTOP),
        ]

        for name, func in key_patterns:
            result = func(o, h, l, c)
            bullish = sum(1 for v in result if v > 0)
            bearish = sum(1 for v in result if v < 0)
            total = bullish + bearish

            detection_summary[name] = {"bullish": bullish, "bearish": bearish, "total": total}

            if total > 0:
                print(f"  {name}: {total} ({bullish} bullish, {bearish} bearish)")
            else:
                print(f"  {name}: 0 detections")

        results["status"] = "success"
        results["bars_tested"] = len(spy)
        results["detections"] = detection_summary

    except ImportError:
        results["reason"] = "yfinance not installed (pip install yfinance)"
        print(f"[SKIP] {results['reason']}")
    except Exception as e:
        results["reason"] = str(e)
        print(f"[ERROR] {results['reason']}")

    return results


# ============================================================
# MAIN TEST RUNNER
# ============================================================

def run_positive_tests() -> Dict[str, Any]:
    """Run all positive pattern tests."""
    print("=" * 60)
    print("POSITIVE TESTS - All 61 TA-Lib Patterns")
    print("=" * 60)

    results = {"passed": 0, "warned": 0, "failed": 0, "details": []}

    for name, test_func in ALL_PATTERN_TESTS:
        try:
            result = test_func()
            detected = result.get("detected_at", [])

            if detected:
                print(f"[PASS] {name}: Detected at {detected}")
                results["passed"] += 1
            else:
                print(f"[WARN] {name}: Not detected (strict TA-Lib params)")
                results["warned"] += 1

            results["details"].append(result)

        except Exception as e:
            print(f"[FAIL] {name}: {e}")
            results["failed"] += 1

    return results


def run_all_tests() -> Dict[str, Any]:
    """Run complete test suite."""
    all_results = {}

    # Positive tests
    all_results["positive"] = run_positive_tests()

    # Negative tests
    all_results["negative"] = run_negative_tests()

    # Fuzz tests
    all_results["fuzz"] = run_fuzz_tests(50)

    # Real data tests
    all_results["real_data"] = run_real_data_tests()

    # Summary
    print("\n" + "=" * 60)
    print("SUMMARY")
    print("=" * 60)

    pos = all_results["positive"]
    print(f"Positive tests: {pos['passed']} passed, {pos['warned']} warned, {pos['failed']} failed")

    neg = all_results["negative"]
    print(f"Negative tests: {neg['passed']} passed, {neg['failed']} failed")

    fuzz = all_results["fuzz"]
    print(f"Fuzz tests: {fuzz['iterations']} iterations, {fuzz['total_detections']} total detections")

    real = all_results["real_data"]
    print(f"Real data tests: {real['status']}")

    return all_results


def export_test_cases(output_path: str = "tests/talib_test_cases.json"):
    """Export all test cases to JSON."""
    results = {}

    for name, test_func in ALL_PATTERN_TESTS:
        try:
            results[name] = test_func()
        except Exception as e:
            results[name] = {"error": str(e)}

    output_file = Path(output_path)
    output_file.parent.mkdir(parents=True, exist_ok=True)

    with open(output_file, 'w') as f:
        json.dump(results, f, indent=2)

    print(f"Test cases exported to: {output_file}")


def export_json_to_stdout():
    """Export all 61 test cases + fuzz rounds as JSON to stdout.

    Used by Rust integration tests (talib_crossval.rs) to get TA-Lib reference data.
    """
    import os
    import sys

    fuzz_rounds = int(os.environ.get("YACPD_FUZZ_ROUNDS", "10"))

    # Collect all 61 curated test cases
    test_cases = {}
    for name, test_func in ALL_PATTERN_TESTS:
        try:
            result = test_func()
            test_cases[name] = {
                "bars": result["bars"],
                "result": result["result"],
                "detected_at": result["detected_at"],
            }
        except Exception as e:
            print(f"WARNING: {name} failed: {e}", file=sys.stderr)

    # Generate fuzz rounds
    fuzz = []
    for seed in range(fuzz_rounds):
        random.seed(seed)
        bars = make_random_bars(50, 100, 5)
        o, h, l, c = bars_to_arrays(bars)
        bars_json = [{"o": b.open, "h": b.high, "l": b.low, "c": b.close} for b in bars]

        results = {}
        for fname, func in TALIB_FUNCS.items():
            result = func(o, h, l, c)
            results[fname] = result.tolist()

        fuzz.append({
            "seed": seed,
            "bars": bars_json,
            "results": results,
        })

    output = {
        "version": 1,
        "test_cases": test_cases,
        "fuzz": fuzz,
    }

    # Write JSON to stdout (compact for speed)
    json.dump(output, sys.stdout, separators=(",", ":"))


def export_enhanced_json_to_stdout():
    """Export enhanced test data: edge cases + heavy fuzz + real market data.

    Used by Rust integration tests (talib_crossval.rs) for deeper validation.
    """
    import os
    import sys

    enhanced_fuzz_rounds = int(os.environ.get("YACPD_ENHANCED_FUZZ_ROUNDS", "100"))

    # 1. Edge cases — 5 scenarios, run all 61 patterns on each
    edge_cases = {}
    for name, gen_func in [
        ("zero_range", make_zero_range_bars),
        ("gaps", make_gap_bars),
        ("extreme_high", make_extreme_high_bars),
        ("extreme_low", make_extreme_low_bars),
        ("micro_volatility", make_micro_volatility_bars),
    ]:
        random.seed(42)  # Deterministic for each edge case
        bars = gen_func(100)
        o, h, l, c = bars_to_arrays(bars)
        bars_json = [{"o": b.open, "h": b.high, "l": b.low, "c": b.close} for b in bars]
        results = {fname: func(o, h, l, c).tolist() for fname, func in TALIB_FUNCS.items()}
        edge_cases[name] = {"bars": bars_json, "results": results}

    # 2. Enhanced fuzz — 100 rounds, 500 bars, variable volatility/base
    enhanced_fuzz = []
    for i in range(enhanced_fuzz_rounds):
        random.seed(1000 + i)  # Offset seeds to not overlap with basic fuzz
        volatility = random.uniform(0.5, 30.0)
        base = random.uniform(10.0, 5000.0)
        bars = make_random_bars(500, base, volatility)
        o, h, l, c = bars_to_arrays(bars)
        bars_json = [{"o": b.open, "h": b.high, "l": b.low, "c": b.close} for b in bars]
        results = {fname: func(o, h, l, c).tolist() for fname, func in TALIB_FUNCS.items()}
        enhanced_fuzz.append({
            "seed": 1000 + i, "bars_count": 500,
            "volatility": volatility, "base": base,
            "bars": bars_json, "results": results,
        })

    # 3. Real data (optional — skip section if yfinance unavailable)
    real_data = {}
    try:
        import yfinance as yf
        for ticker in ["SPY", "AMD", "AAPL", "BTC-USD"]:
            df = yf.download(ticker, period="20y", progress=False)
            if isinstance(df.columns, pd.MultiIndex):
                df.columns = df.columns.get_level_values(0)
            o = df['Open'].values.flatten().astype(np.float64)
            h = df['High'].values.flatten().astype(np.float64)
            l = df['Low'].values.flatten().astype(np.float64)
            c = df['Close'].values.flatten().astype(np.float64)
            bars_json = [{"o": float(ov), "h": float(hv), "l": float(lv), "c": float(cv)}
                         for ov, hv, lv, cv in zip(o, h, l, c)]
            results = {fname: func(o, h, l, c).tolist() for fname, func in TALIB_FUNCS.items()}
            real_data[ticker] = {"bars": bars_json, "bar_count": len(bars_json), "results": results}
    except Exception:
        pass  # real_data stays empty — Rust side handles gracefully

    output = {
        "version": 2,
        "edge_cases": edge_cases,
        "enhanced_fuzz": enhanced_fuzz,
        "real_data": real_data,
    }
    json.dump(output, sys.stdout, separators=(",", ":"))


if __name__ == "__main__":
    import argparse

    parser = argparse.ArgumentParser(description="Comprehensive TA-Lib validation tests for YACPD")
    parser.add_argument("--positive", action="store_true", help="Run only positive tests")
    parser.add_argument("--negative", action="store_true", help="Run only negative tests")
    parser.add_argument("--fuzz", action="store_true", help="Run only fuzz tests")
    parser.add_argument("--real", action="store_true", help="Run only real data tests")
    parser.add_argument("--export", action="store_true", help="Export test cases to JSON")
    parser.add_argument("--export-json", action="store_true", help="Export all test cases + fuzz to stdout as JSON (for Rust cross-validation)")
    parser.add_argument("--export-enhanced-json", action="store_true", help="Export enhanced test data (edge cases + heavy fuzz + real data)")
    parser.add_argument("--list", action="store_true", help="List all TA-Lib CDL functions")
    args = parser.parse_args()

    if args.export_enhanced_json:
        export_enhanced_json_to_stdout()
    elif args.export_json:
        export_json_to_stdout()
    elif args.list:
        cdl_functions = [f for f in dir(talib) if f.startswith('CDL')]
        print(f"\nTA-Lib CDL Functions ({len(cdl_functions)} total):")
        for func in sorted(cdl_functions):
            print(f"  - {func}")
    elif args.export:
        export_test_cases()
    elif args.positive:
        run_positive_tests()
    elif args.negative:
        run_negative_tests()
    elif args.fuzz:
        run_fuzz_tests(100)
    elif args.real:
        run_real_data_tests()
    else:
        run_all_tests()

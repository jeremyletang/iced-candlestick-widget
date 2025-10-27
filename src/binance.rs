use crate::candlestick::Candle;
use std::fmt;

/// Binance timeframe/interval options
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Interval {
    OneMinute,
    FiveMinutes,
    FifteenMinutes,
    ThirtyMinutes,
    OneHour,
    FourHours,
    OneDay,
}

impl Interval {
    pub fn as_str(&self) -> &'static str {
        match self {
            Interval::OneMinute => "1m",
            Interval::FiveMinutes => "5m",
            Interval::FifteenMinutes => "15m",
            Interval::ThirtyMinutes => "30m",
            Interval::OneHour => "1h",
            Interval::FourHours => "4h",
            Interval::OneDay => "1d",
        }
    }

    pub fn to_minutes(&self) -> i64 {
        match self {
            Interval::OneMinute => 1,
            Interval::FiveMinutes => 5,
            Interval::FifteenMinutes => 15,
            Interval::ThirtyMinutes => 30,
            Interval::OneHour => 60,
            Interval::FourHours => 240,
            Interval::OneDay => 1440,
        }
    }

    pub fn all() -> Vec<Interval> {
        vec![
            Interval::OneMinute,
            Interval::FiveMinutes,
            Interval::FifteenMinutes,
            Interval::ThirtyMinutes,
            Interval::OneHour,
            Interval::FourHours,
            Interval::OneDay,
        ]
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Interval::OneMinute => "1 Minute",
            Interval::FiveMinutes => "5 Minutes",
            Interval::FifteenMinutes => "15 Minutes",
            Interval::ThirtyMinutes => "30 Minutes",
            Interval::OneHour => "1 Hour",
            Interval::FourHours => "4 Hours",
            Interval::OneDay => "1 Day",
        }
    }
}

impl Default for Interval {
    fn default() -> Self {
        Interval::OneHour
    }
}

impl fmt::Display for Interval {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

// Binance kline response format: array of arrays
// [open_time, open, high, low, close, volume, close_time, quote_volume, trades, taker_buy_base, taker_buy_quote, unused]
type BinanceKline = (
    i64,    // open time (ms)
    String, // open
    String, // high
    String, // low
    String, // close
    String, // volume
    i64,    // close time (ms)
    String, // quote asset volume
    i64,    // number of trades
    String, // taker buy base asset volume
    String, // taker buy quote asset volume
    String, // unused
);

/// Fetch candlestick data from Binance API
pub async fn fetch_klines(symbol: &str, interval: Interval, limit: u32) -> Result<Vec<Candle>, String> {
    let url = format!(
        "https://api.binance.com/api/v3/klines?symbol={}&interval={}&limit={}",
        symbol,
        interval.as_str(),
        limit
    );

    // Use blocking reqwest client since iced has its own runtime
    let response = reqwest::blocking::get(&url)
        .map_err(|e| format!("Failed to fetch data: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("Binance API error: {}", response.status()));
    }

    let klines: Vec<BinanceKline> = response
        .json()
        .map_err(|e| format!("Failed to parse JSON: {}", e))?;

    // Convert Binance klines to our Candle format
    let candles = klines
        .into_iter()
        .filter_map(|kline| {
            let timestamp = kline.0 / 1000; // Convert from ms to seconds
            let open = kline.1.parse::<f64>().ok()?;
            let high = kline.2.parse::<f64>().ok()?;
            let low = kline.3.parse::<f64>().ok()?;
            let close = kline.4.parse::<f64>().ok()?;
            let volume = kline.5.parse::<f64>().ok()?;

            Some(Candle::new(timestamp, open, high, low, close, volume))
        })
        .collect();

    Ok(candles)
}

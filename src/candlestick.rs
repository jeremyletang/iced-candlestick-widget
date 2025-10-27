use iced::widget::canvas::{self, Canvas, Event, Frame, Geometry, Path, Stroke, Text};
use iced::{Color, Element, Point, Rectangle, Size, Theme};
use iced::mouse::{Cursor, ScrollDelta};
use iced::alignment::{Horizontal, Vertical};
use iced::event::Status;
use chrono::DateTime;

#[derive(Debug, Clone)]
pub enum ChartMessage {
    Zoom(f32),
}

/// Represents a single candlestick (OHLC data)
#[derive(Debug, Clone, Copy)]
pub struct Candle {
    pub timestamp: i64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
}

impl Candle {
    pub fn new(timestamp: i64, open: f64, high: f64, low: f64, close: f64, volume: f64) -> Self {
        Self {
            timestamp,
            open,
            high,
            low,
            close,
            volume,
        }
    }

    pub fn is_bullish(&self) -> bool {
        self.close >= self.open
    }
}

/// Candlestick chart widget
pub struct CandlestickChart {
    candles: Vec<Candle>,
}

impl CandlestickChart {
    pub fn new(candles: Vec<Candle>) -> Self {
        Self { candles }
    }

    pub fn view(&self) -> Element<'_, ChartMessage> {
        Canvas::new(self)
            .width(iced::Length::Fill)
            .height(iced::Length::Fill)
            .into()
    }
}

impl canvas::Program<ChartMessage> for CandlestickChart {
    type State = ();

    fn update(
        &self,
        _state: &mut Self::State,
        event: Event,
        _bounds: Rectangle,
        _cursor: Cursor,
    ) -> (Status, Option<ChartMessage>) {
        match event {
            Event::Mouse(iced::mouse::Event::WheelScrolled { delta }) => {
                match delta {
                    ScrollDelta::Lines { y, .. } | ScrollDelta::Pixels { y, .. } => {
                        (Status::Captured, Some(ChartMessage::Zoom(y)))
                    }
                }
            }
            _ => (Status::Ignored, None),
        }
    }

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &iced::Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: Cursor,
    ) -> Vec<Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());

        if self.candles.is_empty() {
            return vec![frame.into_geometry()];
        }

        // Draw black background
        let background = Path::rectangle(Point::ORIGIN, bounds.size());
        frame.fill(&background, Color::BLACK);

        // Define margins for axes
        let left_margin = 10.0;
        let bottom_margin = 30.0;
        let top_margin = 10.0;
        let right_margin = 60.0;

        // Calculate chart area
        let chart_width = bounds.width - left_margin - right_margin;
        let chart_height = bounds.height - top_margin - bottom_margin;
        let chart_x = left_margin;
        let chart_y = top_margin;

        // Calculate price range
        let mut min_price = f64::MAX;
        let mut max_price = f64::MIN;

        for candle in &self.candles {
            min_price = min_price.min(candle.low);
            max_price = max_price.max(candle.high);
        }

        // Add some padding to the price range
        let price_range = max_price - min_price;
        let padding = price_range * 0.1;
        min_price -= padding;
        max_price += padding;

        let price_span = max_price - min_price;

        // Draw grid lines and Y-axis labels (prices)
        let num_price_lines = 5;
        let grid_color = Color::from_rgb(0.2, 0.2, 0.2);
        let text_color = Color::from_rgb(0.8, 0.8, 0.8);

        for i in 0..=num_price_lines {
            let ratio = i as f32 / num_price_lines as f32;
            let y = chart_y + chart_height - (ratio * chart_height);
            let price = min_price + (ratio as f64 * price_span);

            // Draw grid line
            let grid_line = Path::line(
                Point::new(chart_x, y),
                Point::new(chart_x + chart_width, y),
            );
            frame.stroke(
                &grid_line,
                Stroke::default().with_width(1.0).with_color(grid_color),
            );

            // Draw price label on the right
            let price_text = Text {
                content: format!("{:.2}", price),
                position: Point::new(chart_x + chart_width + 5.0, y),
                color: text_color,
                size: 12.0.into(),
                horizontal_alignment: Horizontal::Left,
                vertical_alignment: Vertical::Center,
                ..Default::default()
            };
            frame.fill_text(price_text);
        }

        // Draw X-axis labels (candle indices)
        let num_x_labels = 5.min(self.candles.len());
        let step = if num_x_labels > 1 {
            self.candles.len() / (num_x_labels - 1)
        } else {
            1
        };

        for i in 0..num_x_labels {
            let candle_idx = (i * step).min(self.candles.len() - 1);
            let x = chart_x + (candle_idx as f32 / self.candles.len() as f32) * chart_width;
            let y = chart_y + chart_height;

            // Get the timestamp and format it
            let timestamp = self.candles[candle_idx].timestamp;
            let datetime = DateTime::from_timestamp(timestamp, 0)
                .unwrap_or_else(|| DateTime::from_timestamp(0, 0).unwrap());
            let date_string = datetime.format("%m/%d").to_string();

            // Draw X-axis label
            let x_text = Text {
                content: date_string,
                position: Point::new(x, y + 15.0),
                color: text_color,
                size: 12.0.into(),
                horizontal_alignment: Horizontal::Center,
                vertical_alignment: Vertical::Center,
                ..Default::default()
            };
            frame.fill_text(x_text);
        }

        // Draw chart border
        let border = Path::rectangle(
            Point::new(chart_x, chart_y),
            Size::new(chart_width, chart_height),
        );
        frame.stroke(
            &border,
            Stroke::default().with_width(1.0).with_color(Color::from_rgb(0.4, 0.4, 0.4)),
        );

        // Calculate max volume for scaling
        let max_volume = self.candles.iter()
            .map(|c| c.volume)
            .fold(f64::MIN, f64::max);

        // Draw volume bars FIRST (so they appear behind candlesticks)
        // Volume bars use the bottom 30% of the chart height
        let volume_max_height = chart_height * 0.3;

        let num_candles = self.candles.len();
        let candle_width = chart_width / num_candles as f32;

        for (i, candle) in self.candles.iter().enumerate() {
            let x = chart_x + i as f32 * candle_width;
            let bar_width = candle_width * 0.8;

            let volume_ratio = (candle.volume / max_volume) as f32;
            let bar_height = volume_ratio * volume_max_height;

            // Color volume bars based on candle direction with high transparency
            let volume_color = if candle.is_bullish() {
                Color::from_rgba(0.0, 0.8, 0.0, 0.2) // Very transparent green
            } else {
                Color::from_rgba(0.8, 0.0, 0.0, 0.2) // Very transparent red
            };

            let volume_bar = Path::rectangle(
                Point::new(x + candle_width * 0.1, chart_y + chart_height - bar_height),
                Size::new(bar_width, bar_height),
            );
            frame.fill(&volume_bar, volume_color);
        }

        // Draw each candlestick (on top of volume bars)
        let body_width = candle_width * 0.7;
        let wick_width = candle_width * 0.1;

        for (i, candle) in self.candles.iter().enumerate() {
            let x = chart_x + i as f32 * candle_width + candle_width / 2.0;

            // Convert prices to screen coordinates (invert Y axis)
            let open_y = chart_y + chart_height - ((candle.open - min_price) / price_span) as f32 * chart_height;
            let close_y = chart_y + chart_height - ((candle.close - min_price) / price_span) as f32 * chart_height;
            let high_y = chart_y + chart_height - ((candle.high - min_price) / price_span) as f32 * chart_height;
            let low_y = chart_y + chart_height - ((candle.low - min_price) / price_span) as f32 * chart_height;

            // Determine color based on bullish/bearish
            let color = if candle.is_bullish() {
                Color::from_rgb(0.0, 0.8, 0.0) // Green for bullish
            } else {
                Color::from_rgb(0.8, 0.0, 0.0) // Red for bearish
            };

            // Draw the wick (high to low line)
            let wick = Path::line(Point::new(x, high_y), Point::new(x, low_y));
            frame.stroke(
                &wick,
                Stroke::default().with_width(wick_width).with_color(color),
            );

            // Draw the body (open to close rectangle)
            let body_top = open_y.min(close_y);
            let body_height = (open_y - close_y).abs().max(1.0); // Ensure minimum height

            let body = Path::rectangle(
                Point::new(x - body_width / 2.0, body_top),
                Size::new(body_width, body_height),
            );

            frame.fill(&body, color);
        }

        vec![frame.into_geometry()]
    }
}

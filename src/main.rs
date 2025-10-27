mod candlestick;
mod binance;

use candlestick::{Candle, CandlestickChart};
use binance::Interval;
use iced::widget::{button, column, container, pick_list, row, text};
use iced::{Element, Task};

fn main() -> iced::Result {
    iced::application("BTCUSDT - Binance", App::update, App::view)
        .run_with(App::new)
}

struct App {
    chart: Option<CandlestickChart>,
    candles: Vec<Candle>,
    selected_interval: Interval,
    loading: bool,
    error: Option<String>,
    visible_candles: usize,
    pan_offset: usize,
}

#[derive(Debug, Clone)]
enum Message {
    IntervalSelected(Interval),
    DataFetched(Result<Vec<Candle>, String>),
    RefreshData,
    ZoomIn,
    ZoomOut,
    ChartEvent(candlestick::ChartMessage),
}

impl App {
    fn new() -> (Self, Task<Message>) {
        let app = Self {
            chart: None,
            candles: Vec::new(),
            selected_interval: Interval::default(),
            loading: false,
            error: None,
            visible_candles: 100,
            pan_offset: 0,
        };

        // Fetch initial data
        let task = Task::perform(
            binance::fetch_klines("BTCUSDT", Interval::default(), 500),
            Message::DataFetched,
        );

        (app, task)
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::IntervalSelected(interval) => {
                self.selected_interval = interval;
                self.loading = true;
                self.error = None;

                Task::perform(
                    binance::fetch_klines("BTCUSDT", interval, 500),
                    Message::DataFetched,
                )
            }
            Message::DataFetched(result) => {
                self.loading = false;

                match result {
                    Ok(candles) => {
                        self.candles = candles;
                        self.pan_offset = 0;
                        self.visible_candles = self.visible_candles.min(self.candles.len());
                        self.update_chart();
                        self.error = None;
                    }
                    Err(e) => {
                        self.error = Some(e);
                    }
                }

                Task::none()
            }
            Message::RefreshData => {
                self.loading = true;
                self.error = None;

                Task::perform(
                    binance::fetch_klines("BTCUSDT", self.selected_interval, 500),
                    Message::DataFetched,
                )
            }
            Message::ZoomIn => {
                self.visible_candles = (self.visible_candles - 10).max(10);
                self.update_chart();
                Task::none()
            }
            Message::ZoomOut => {
                self.visible_candles = (self.visible_candles + 10).min(self.candles.len());
                self.update_chart();
                Task::none()
            }
            Message::ChartEvent(chart_msg) => {
                match chart_msg {
                    candlestick::ChartMessage::Zoom(delta) => {
                        if delta > 0.0 {
                            self.visible_candles = (self.visible_candles - 5).max(10);
                        } else {
                            self.visible_candles = (self.visible_candles + 5).min(self.candles.len());
                        }
                        self.update_chart();
                    }
                    candlestick::ChartMessage::Pan(pixel_delta) => {
                        // Convert pixel delta to candle delta
                        // Drag right (positive delta) = go back in time (increase offset, show older)
                        // Drag left (negative delta) = go forward in time (decrease offset, show newer)
                        let pixels_per_candle = 800.0 / self.visible_candles as f32;
                        let sensitivity = 2.0; // Make it more responsive
                        let candle_delta = (pixel_delta * sensitivity / pixels_per_candle) as i32;

                        let max_offset = self.candles.len().saturating_sub(self.visible_candles);
                        self.pan_offset = (self.pan_offset as i32 + candle_delta)
                            .max(0)
                            .min(max_offset as i32) as usize;
                        self.update_chart();
                    }
                }
                Task::none()
            }
        }
    }

    fn update_chart(&mut self) {
        if self.candles.is_empty() {
            return;
        }

        // Show most recent candles on the right (end of array)
        let end = self.candles.len() - self.pan_offset;
        let start = end.saturating_sub(self.visible_candles);
        let visible = self.candles[start..end].to_vec();
        self.chart = Some(CandlestickChart::new(visible));
    }

    fn view(&self) -> Element<Message> {
        let interval_selector = row![
            text("Interval: ").size(16),
            pick_list(
                Interval::all(),
                Some(self.selected_interval),
                Message::IntervalSelected,
            )
            .placeholder("Select interval"),
            button("Refresh").on_press(Message::RefreshData),
        ]
        .spacing(10)
        .padding(10);

        let zoom_controls = row![
            button("-").on_press(Message::ZoomOut),
            text(format!("{}/{} candles", self.visible_candles, self.candles.len())).size(14),
            button("+").on_press(Message::ZoomIn),
        ]
        .spacing(5)
        .padding(10);

        let status = if self.loading {
            text("Loading...").size(16)
        } else if let Some(ref error) = self.error {
            text(format!("Error: {}", error)).size(16)
        } else {
            text("BTCUSDT").size(16)
        };

        let controls = row![interval_selector, zoom_controls, status]
            .spacing(20)
            .padding(10);

        let content = if let Some(ref chart) = self.chart {
            column![controls, chart.view().map(Message::ChartEvent)]
        } else {
            column![controls, text("Loading chart...").size(20)]
        };

        container(content).into()
    }
}


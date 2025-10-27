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
    selected_interval: Interval,
    loading: bool,
    error: Option<String>,
}

#[derive(Debug, Clone)]
enum Message {
    IntervalSelected(Interval),
    DataFetched(Result<Vec<Candle>, String>),
    RefreshData,
}

impl App {
    fn new() -> (Self, Task<Message>) {
        let app = Self {
            chart: None,
            selected_interval: Interval::default(),
            loading: false,
            error: None,
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
                        self.chart = Some(CandlestickChart::new(candles));
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
        }
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

        let status = if self.loading {
            text("Loading...").size(16)
        } else if let Some(ref error) = self.error {
            text(format!("Error: {}", error)).size(16)
        } else {
            text("BTCUSDT").size(16)
        };

        let controls = row![interval_selector, status]
            .spacing(20)
            .padding(10);

        let content = if let Some(ref chart) = self.chart {
            column![controls, chart.view()]
        } else {
            column![controls, text("Loading chart...").size(20)]
        };

        container(content).into()
    }
}


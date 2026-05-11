//! 매매 전략 모듈
//! 
//! 기술적 분석 기반 매매 전략 구현

use crate::backtest::{MarketData, Position, Signal};

/// 이동평균 교차 전략 (Moving Average Crossover)
pub struct MovingAverageCrossover {
    short_period: usize,
    long_period: usize,
    prices: Vec<f64>,
}

impl MovingAverageCrossover {
    pub fn new(short_period: usize, long_period: usize) -> Self {
        Self {
            short_period,
            long_period,
            prices: Vec::new(),
        }
    }

    fn calculate_sma(&self, prices: &[f64], period: usize) -> f64 {
        if prices.len() < period {
            return 0.0;
        }
        let sum: f64 = prices[prices.len() - period..].iter().sum();
        sum / period as f64
    }

    pub fn on_data(
        &mut self,
        data: &MarketData,
    ) -> Signal {
        self.prices.push(data.close);

        if self.prices.len() < self.long_period {
            return Signal::Hold;
        }

        let short_ma = self.calculate_sma(&self.prices, self.short_period);
        let long_ma = self.calculate_sma(&self.prices, self.long_period);

        if self.prices.len() >= 2 {
            let prev_short_ma = self.calculate_sma(
                &self.prices[..self.prices.len() - 1],
                self.short_period,
            );
            let prev_long_ma = self.calculate_sma(
                &self.prices[..self.prices.len() - 1],
                self.long_period,
            );

            // 골든크로스 (단기 > 장기로 상향돌파)
            if prev_short_ma <= prev_long_ma && short_ma > long_ma {
                return Signal::Buy {
                    stock_code: data.stock_code.clone(),
                    quantity: 1, // 리스크 관리에서 계산
                };
            }

            // 데드크로스 (단기 < 장기로 하향돌파)
            if prev_short_ma >= prev_long_ma && short_ma < long_ma {
                return Signal::Sell {
                    stock_code: data.stock_code.clone(),
                    quantity: 1,
                };
            }
        }

        Signal::Hold
    }
}

/// RSI 기반 전략
pub struct RSIStrategy {
    period: usize,
    overbought: f64,
    oversold: f64,
    prices: Vec<f64>,
}

impl RSIStrategy {
    pub fn new(period: usize, overbought: f64, oversold: f64) -> Self {
        Self {
            period,
            overbought,
            oversold,
            prices: Vec::new(),
        }
    }

    fn calculate_rsi(&self,
    ) -> f64 {
        if self.prices.len() < self.period + 1 {
            return 50.0;
        }

        let mut gains = 0.0;
        let mut losses = 0.0;

        for i in (self.prices.len() - self.period)..self.prices.len() {
            let change = self.prices[i] - self.prices[i - 1];
            if change > 0.0 {
                gains += change;
            } else {
                losses += change.abs();
            }
        }

        let avg_gain = gains / self.period as f64;
        let avg_loss = losses / self.period as f64;

        if avg_loss == 0.0 {
            return 100.0;
        }

        let rs = avg_gain / avg_loss;
        100.0 - (100.0 / (1.0 + rs))
    }

    pub fn on_data(
        &mut self,
        data: &MarketData,
    ) -> Signal {
        self.prices.push(data.close);

        let rsi = self.calculate_rsi();

        if rsi < self.oversold {
            return Signal::Buy {
                stock_code: data.stock_code.clone(),
                quantity: 1,
            };
        }

        if rsi > self.overbought {
            return Signal::Sell {
                stock_code: data.stock_code.clone(),
                quantity: 1,
            };
        }

        Signal::Hold
    }
}

/// 볼린저 밴드 전략
pub struct BollingerBands {
    period: usize,
    std_dev: f64,
    prices: Vec<f64>,
}

impl BollingerBands {
    pub fn new(period: usize, std_dev: f64) -> Self {
        Self {
            period,
            std_dev,
            prices: Vec::new(),
        }
    }

    fn calculate_bands(&self) -> Option<(f64, f64, f64)> {
        if self.prices.len() < self.period {
            return None;
        }

        let recent = &self.prices[self.prices.len() - self.period..];
        let sma: f64 = recent.iter().sum::<f64>() / self.period as f64;
        
        let variance: f64 = recent.iter()
            .map(|x| (x - sma).powi(2))
            .sum::<f64>() / self.period as f64;
        let std = variance.sqrt();

        let upper = sma + (self.std_dev * std);
        let lower = sma - (self.std_dev * std);

        Some((upper, sma, lower))
    }

    pub fn on_data(
        &mut self,
        data: &MarketData,
    ) -> Signal {
        self.prices.push(data.close);

        if let Some((upper, _middle, lower)) = self.calculate_bands() {
            if data.close < lower {
                return Signal::Buy {
                    stock_code: data.stock_code.clone(),
                    quantity: 1,
                };
            }

            if data.close > upper {
                return Signal::Sell {
                    stock_code: data.stock_code.clone(),
                    quantity: 1,
                };
            }
        }

        Signal::Hold
    }
}

/// MACD 전략
pub struct MACDStrategy {
    fast_period: usize,
    slow_period: usize,
    signal_period: usize,
    prices: Vec<f64>,
}

impl MACDStrategy {
    pub fn new(fast: usize, slow: usize, signal: usize) -> Self {
        Self {
            fast_period: fast,
            slow_period: slow,
            signal_period: signal,
            prices: Vec::new(),
        }
    }

    fn calculate_ema(&self,
        prices: &[f64],
        period: usize,
    ) -> f64 {
        if prices.len() < period {
            return prices.last().copied().unwrap_or(0.0);
        }

        let multiplier = 2.0 / (period as f64 + 1.0);
        let mut ema = prices[prices.len() - period];

        for i in (prices.len() - period + 1)..prices.len() {
            ema = (prices[i] - ema) * multiplier + ema;
        }

        ema
    }

    pub fn on_data(
        &mut self,
        data: &MarketData,
    ) -> Signal {
        self.prices.push(data.close);

        if self.prices.len() < self.slow_period + self.signal_period {
            return Signal::Hold;
        }

        let fast_ema = self.calculate_ema(&self.prices, self.fast_period);
        let slow_ema = self.calculate_ema(&self.prices, self.slow_period);
        let macd = fast_ema - slow_ema;

        // TODO: Signal line 계산 및 크로스 체크
        // MACD > 0: 매수 신호
        // MACD < 0: 매도 신호
        if macd > 0.0 {
            Signal::Buy {
                stock_code: data.stock_code.clone(),
                quantity: 1,
            }
        } else {
            Signal::Sell {
                stock_code: data.stock_code.clone(),
                quantity: 1,
            }
        }
    }
}

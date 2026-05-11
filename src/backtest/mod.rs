//! 백테스팅 엔진
//! 
//! 사용법:
//! 

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use chrono::{DateTime, Utc};

/// 백테스팅 결과
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacktestResult {
    pub initial_capital: f64,
    pub final_capital: f64,
    pub total_return: f64,           // 총 수익률 (%)
    pub annual_return: f64,          // 연간 수익률 (%)
    pub max_drawdown: f64,          // 최대 낙폭 (%)
    pub sharpe_ratio: f64,          // 샤프 비율
    pub win_rate: f64,              // 승률 (%)
    pub profit_factor: f64,         // 손익비
    pub total_trades: usize,        // 총 거래 횟수
    pub winning_trades: usize,     // 승리 거래 횟수
    pub losing_trades: usize,       // 패배 거래 횟수
    pub trades: Vec<Trade>,        // 거래 내역
}

/// 개별 거래 기록
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trade {
    pub entry_time: DateTime<Utc>,
    pub exit_time: Option<DateTime<Utc>>,
    pub stock_code: String,
    pub entry_price: f64,
    pub exit_price: Option<f64>,
    pub quantity: i32,
    pub side: TradeSide,            // Buy or Sell
    pub pnl: Option<f64>,           // 실현 손익
    pub pnl_percent: Option<f64>,   // 손익률 (%)
    pub exit_reason: Option<String>, // 청산 사유
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TradeSide {
    Buy,
    Sell,
}

/// 백테스팅 엔진
pub struct BacktestEngine {
    initial_capital: f64,
    current_capital: f64,
    positions: Vec<Position>,
    trades: Vec<Trade>,
    strategies: Vec<Box<dyn Strategy>>,
}

#[derive(Debug, Clone)]
struct Position {
    stock_code: String,
    entry_price: f64,
    quantity: i32,
    entry_time: DateTime<Utc>,
}

impl BacktestEngine {
    pub fn new(initial_capital: f64) -> Self {
        Self {
            initial_capital,
            current_capital: initial_capital,
            positions: Vec::new(),
            trades: Vec::new(),
            strategies: Vec::new(),
        }
    }

    pub fn add_strategy(&mut self, strategy: Box<dyn Strategy>) {
        self.strategies.push(strategy);
    }

    /// 백테스팅 실행
    pub async fn run(&mut self, data: &[MarketData]) -> BacktestResult {
        for bar in data {
            // 각 전략 실행
            for strategy in &mut self.strategies {
                let signal = strategy.on_data(bar, &self.positions);
                
                match signal {
                    Signal::Buy { stock_code, quantity } => {
                        self.open_position(&stock_code, bar.close, quantity, bar.timestamp);
                    }
                    Signal::Sell { stock_code, quantity } => {
                        self.close_position(&stock_code, bar.close, quantity, bar.timestamp, "전략 신호");
                    }
                    Signal::Hold => {}
                }
            }

            // 손절/익절 체크
            self.check_stop_loss_take_profit(bar);
        }

        // 모든 포지션 청산
        self.close_all_positions(data.last().unwrap().close, data.last().unwrap().timestamp, "백테스트 종료");

        self.calculate_results()
    }

    fn open_position(&mut self,
        stock_code: &str,
        price: f64,
        quantity: i32,
        timestamp: DateTime<Utc>,
    ) {
        let cost = price * quantity as f64;
        if cost > self.current_capital {
            return; // 잔고 부족
        }

        self.current_capital -= cost;
        
        let position = Position {
            stock_code: stock_code.to_string(),
            entry_price: price,
            quantity,
            entry_time: timestamp,
        };
        
        self.positions.push(position);

        self.trades.push(Trade {
            entry_time: timestamp,
            exit_time: None,
            stock_code: stock_code.to_string(),
            entry_price: price,
            exit_price: None,
            quantity,
            side: TradeSide::Buy,
            pnl: None,
            pnl_percent: None,
            exit_reason: None,
        });
    }

    fn close_position(
        &mut self,
        stock_code: &str,
        price: f64,
        quantity: i32,
        timestamp: DateTime<Utc>,
        reason: &str,
    ) {
        if let Some(pos_idx) = self.positions.iter().position(|p| p.stock_code == stock_code) {
            let position = self.positions.remove(pos_idx);
            let close_qty = quantity.min(position.quantity);
            
            let revenue = price * close_qty as f64;
            let cost = position.entry_price * close_qty as f64;
            let pnl = revenue - cost;
            let pnl_percent = (pnl / cost) * 100.0;

            self.current_capital += revenue;

            // 거래 기록 업데이트
            if let Some(trade) = self.trades.iter_mut().find(|t| {
                t.stock_code == stock_code && t.exit_time.is_none()
            }) {
                trade.exit_time = Some(timestamp);
                trade.exit_price = Some(price);
                trade.pnl = Some(pnl);
                trade.pnl_percent = Some(pnl_percent);
                trade.exit_reason = Some(reason.to_string());
            }
        }
    }

    fn check_stop_loss_take_profit(&mut self,
        bar: &MarketData,
    ) {
        let stop_loss_percent = -5.0;  // -5% 손절
        let take_profit_percent = 10.0; // +10% 익절

        let positions_to_close: Vec<(String, i32)> = self.positions
            .iter()
            .filter_map(|pos| {
                let current_pnl_percent = ((bar.close - pos.entry_price) / pos.entry_price) * 100.0;
                
                if current_pnl_percent <= stop_loss_percent {
                    Some((pos.stock_code.clone(), pos.quantity))
                } else if current_pnl_percent >= take_profit_percent {
                    Some((pos.stock_code.clone(), pos.quantity))
                } else {
                    None
                }
            })
            .collect();

        for (stock_code, quantity) in positions_to_close {
            let reason = if ((bar.close - self.positions.iter().find(|p| p.stock_code == stock_code).unwrap().entry_price) 
                / self.positions.iter().find(|p| p.stock_code == stock_code).unwrap().entry_price * 100.0) < 0.0 {
                "손절".to_string()
            } else {
                "익절".to_string()
            };
            self.close_position(&stock_code, bar.close, quantity, bar.timestamp, &reason);
        }
    }

    fn close_all_positions(
        &mut self,
        price: f64,
        timestamp: DateTime<Utc>,
        reason: &str,
    ) {
        let positions: Vec<(String, i32)> = self.positions
            .iter()
            .map(|p| (p.stock_code.clone(), p.quantity))
            .collect();
        
        for (stock_code, quantity) in positions {
            self.close_position(&stock_code, price, quantity, timestamp, reason);
        }
    }

    fn calculate_results(&self,
    ) -> BacktestResult {
        let closed_trades: Vec<&Trade> = self.trades
            .iter()
            .filter(|t| t.exit_time.is_some())
            .collect();

        let winning_trades = closed_trades.iter().filter(|t| {
            t.pnl.unwrap_or(0.0) > 0.0
        }).count();
        
        let losing_trades = closed_trades.iter().filter(|t| {
            t.pnl.unwrap_or(0.0) <= 0.0
        }).count();

        let total_pnl: f64 = closed_trades.iter().map(|t| t.pnl.unwrap_or(0.0)).sum();
        let gross_profit: f64 = closed_trades.iter().filter(|t| t.pnl.unwrap_or(0.0) > 0.0).map(|t| t.pnl.unwrap()).sum();
        let gross_loss: f64 = closed_trades.iter().filter(|t| t.pnl.unwrap_or(0.0) < 0.0).map(|t| t.pnl.unwrap()).sum().abs();

        // 최대 낙폭 계산
        let mut peak = self.initial_capital;
        let mut max_drawdown = 0.0;
        let mut running_capital = self.initial_capital;
        
        for trade in &closed_trades {
            if let Some(pnl) = trade.pnl {
                running_capital += pnl;
                if running_capital > peak {
                    peak = running_capital;
                }
                let drawdown = (peak - running_capital) / peak * 100.0;
                if drawdown > max_drawdown {
                    max_drawdown = drawdown;
                }
            }
        }

        BacktestResult {
            initial_capital: self.initial_capital,
            final_capital: self.current_capital,
            total_return: ((self.current_capital - self.initial_capital) / self.initial_capital) * 100.0,
            annual_return: 0.0, // TODO: 계산
            max_drawdown,
            sharpe_ratio: 0.0, // TODO: 계산
            win_rate: if !closed_trades.is_empty() {
                (winning_trades as f64 / closed_trades.len() as f64) * 100.0
            } else { 0.0 },
            profit_factor: if gross_loss > 0.0 { gross_profit / gross_loss } else { 0.0 },
            total_trades: closed_trades.len(),
            winning_trades,
            losing_trades,
            trades: self.trades.clone(),
        }
    }
}

/// 전략 트레이트
pub trait Strategy: Send {
    fn on_data(
        &mut self,
        bar: &MarketData,
        positions: &[Position],
    ) -> Signal;
}

/// 시장 데이터
#[derive(Debug, Clone)]
pub struct MarketData {
    pub timestamp: DateTime<Utc>,
    pub stock_code: String,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: i64,
}

/// 매매 신호
#[derive(Debug, Clone)]
pub enum Signal {
    Buy { stock_code: String, quantity: i32 },
    Sell { stock_code: String, quantity: i32 },
    Hold,
}

// Position struct for Strategy trait
#[derive(Debug, Clone)]
pub struct Position {
    pub stock_code: String,
    pub entry_price: f64,
    pub quantity: i32,
    pub entry_time: DateTime<Utc>,
}

impl Position {
    pub fn current_pnl(&self, current_price: f64) -> f64 {
        (current_price - self.entry_price) * self.quantity as f64
    }

    pub fn current_pnl_percent(&self, current_price: f64) -> f64 {
        ((current_price - self.entry_price) / self.entry_price) * 100.0
    }
}

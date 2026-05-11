//! AI Stock Bot - 키움증권 API 연동 자동매매 시스템
//!
//! ## 구성
//! - : 백테스팅 엔진
//! - : 리스크 관리
//! - : 매매 전략

pub mod backtest;
pub mod risk;
pub mod strategy;

// Re-export 주요 타입들
pub use backtest::{
    BacktestEngine, BacktestResult, Trade, TradeSide,
    MarketData, Signal, Strategy, Position,
};

pub use risk::{
    RiskManager, RiskConfig, RiskCheckResult, TradeSide as RiskTradeSide,
};

pub use strategy::{
    MovingAverageCrossover, RSIStrategy, BollingerBands, MACDStrategy,
};

/// 버전 정보
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// 에러 타입
#[derive(Debug)]
pub enum StockTraderError {
    ApiError(String),
    AuthError(String),
    NetworkError(String),
    ParseError(String),
    RiskError(String),
}

impl std::fmt::Display for StockTraderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StockTraderError::ApiError(msg) => write!(f, "API Error: {}", msg),
            StockTraderError::AuthError(msg) => write!(f, "Auth Error: {}", msg),
            StockTraderError::NetworkError(msg) => write!(f, "Network Error: {}", msg),
            StockTraderError::ParseError(msg) => write!(f, "Parse Error: {}", msg),
            StockTraderError::RiskError(msg) => write!(f, "Risk Error: {}", msg),
        }
    }
}

impl std::error::Error for StockTraderError {}

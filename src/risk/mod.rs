//! 리스크 관리 모듈
//! 
//! 포트폴리오 리스크 관리, 포지션 사이즈 계산, 손절/익절 관리

use serde::{Deserialize, Serialize};

/// 리스크 관리 설정
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskConfig {
    /// 계정당 최대 리스크 비율 (%)
    pub max_account_risk_percent: f64,
    /// 종목당 최대 리스크 비율 (%)
    pub max_position_risk_percent: f64,
    /// 최대 포지션 수
    pub max_positions: usize,
    /// 손절 비율 (%)
    pub stop_loss_percent: f64,
    /// 익절 비율 (%)
    pub take_profit_percent: f64,
    /// 최대 낙폭 제한 (%)
    pub max_drawdown_percent: f64,
    /// 일일 최대 손실 제한 (%)
    pub daily_max_loss_percent: f64,
}

impl Default for RiskConfig {
    fn default() -> Self {
        Self {
            max_account_risk_percent: 2.0,      // 계정의 2%까지만 리스크
            max_position_risk_percent: 1.0,     // 포지션당 1% 리스크
            max_positions: 5,                   // 최대 5개 종목
            stop_loss_percent: -5.0,            // -5% 손절
            take_profit_percent: 10.0,          // +10% 익절
            max_drawdown_percent: -20.0,        // -20% 최대 낙폭
            daily_max_loss_percent: -3.0,       // 일일 -3% 손실 제한
        }
    }
}

/// 리스크 매니저
pub struct RiskManager {
    config: RiskConfig,
    daily_pnl: f64,
    peak_capital: f64,
    current_capital: f64,
    positions_today: usize,
}

impl RiskManager {
    pub fn new(config: RiskConfig, initial_capital: f64) -> Self {
        Self {
            config,
            daily_pnl: 0.0,
            peak_capital: initial_capital,
            current_capital: initial_capital,
            positions_today: 0,
        }
    }

    /// 새로운 거래 허용 여부 체크
    pub fn can_trade(&self, stock_code: &str, entry_price: f64, stop_loss: f64) -> bool {
        // 1. 일일 손실 제한 체크
        if self.daily_pnl < 0.0 {
            let daily_loss_percent = (self.daily_pnl / self.current_capital) * 100.0;
            if daily_loss_percent <= self.config.daily_max_loss_percent {
                return false;
            }
        }

        // 2. 최대 포지션 수 체크
        if self.positions_today >= self.config.max_positions {
            return false;
        }

        // 3. 리스크 대비 보상 비율 체크
        let risk = entry_price - stop_loss;
        let reward = entry_price * (self.config.take_profit_percent / 100.0);
        if risk <= 0.0 || reward / risk < 1.5 {
            return false;
        }

        true
    }

    /// 포지션 사이즈 계산 (고정 비율법)
    pub fn calculate_position_size_fixed_fraction(
        &self,
        entry_price: f64,
        stop_loss: f64,
    ) -> i32 {
        let risk_amount = self.current_capital * (self.config.max_position_risk_percent / 100.0);
        let risk_per_share = (entry_price - stop_loss).abs();
        
        if risk_per_share <= 0.0 {
            return 0;
        }

        let shares = (risk_amount / risk_per_share) as i32;
        let max_shares = (self.current_capital * 0.95 / entry_price) as i32; // 95% 자금만 사용
        
        shares.min(max_shares).max(0)
    }

    /// 켈리 공식 기반 포지션 사이즈 계산
    pub fn calculate_position_size_kelly(
        &self,
        entry_price: f64,
        stop_loss: f64,
        win_rate: f64,
        avg_win: f64,
        avg_loss: f64,
    ) -> i32 {
        // 켈리 공식: f = (bp - q) / b
        // b = 평균 수익 / 평균 손실, p = 승률, q = 패률
        let b = avg_win / avg_loss.abs();
        let p = win_rate;
        let q = 1.0 - p;
        
        let kelly_fraction = (b * p - q) / b;
        let kelly_fraction = kelly_fraction.max(0.0).min(0.5); // 최대 50%로 제한
        
        let position_value = self.current_capital * kelly_fraction;
        let shares = (position_value / entry_price) as i32;
        
        shares.max(0)
    }

    /// 손절가 계산
    pub fn calculate_stop_loss(
        &self,
        entry_price: f64,
        side: TradeSide,
    ) -> f64 {
        match side {
            TradeSide::Long => {
                entry_price * (1.0 + self.config.stop_loss_percent / 100.0)
            }
            TradeSide::Short => {
                entry_price * (1.0 - self.config.stop_loss_percent / 100.0)
            }
        }
    }

    /// 익절가 계산
    pub fn calculate_take_profit(
        &self,
        entry_price: f64,
        side: TradeSide,
    ) -> f64 {
        match side {
            TradeSide::Long => {
                entry_price * (1.0 + self.config.take_profit_percent / 100.0)
            }
            TradeSide::Short => {
                entry_price * (1.0 - self.config.take_profit_percent / 100.0)
            }
        }
    }

    /// 트레일링 스탑 계산
    pub fn calculate_trailing_stop(
        &self,
        entry_price: f64,
        highest_price: f64,
        trailing_percent: f64,
    ) -> f64 {
        let trailing_stop = highest_price * (1.0 - trailing_percent / 100.0);
        trailing_stop.max(self.calculate_stop_loss(entry_price, TradeSide::Long))
    }

    /// 일일 손익 업데이트
    pub fn update_daily_pnl(&mut self, pnl: f64) {
        self.daily_pnl += pnl;
        self.current_capital += pnl;
        
        // 최고 자본 업데이트
        if self.current_capital > self.peak_capital {
            self.peak_capital = self.current_capital;
        }
    }

    /// 포지션 추가
    pub fn add_position(&mut self) {
        self.positions_today += 1;
    }

    /// 포지션 제거
    pub fn remove_position(&mut self) {
        if self.positions_today > 0 {
            self.positions_today -= 1;
        }
    }

    /// 일일 리셋 (장 시작 시 호출)
    pub fn reset_daily(&mut self) {
        self.daily_pnl = 0.0;
        self.positions_today = 0;
    }

    /// 현재 드로다운 계산
    pub fn current_drawdown(&self) -> f64 {
        if self.peak_capital <= 0.0 {
            return 0.0;
        }
        ((self.peak_capital - self.current_capital) / self.peak_capital) * 100.0
    }

    /// 리스크 체크 결과
    pub fn check_all_risks(&self) -> RiskCheckResult {
        RiskCheckResult {
            within_daily_loss_limit: self.daily_pnl >= self.current_capital * (self.config.daily_max_loss_percent / 100.0),
            within_max_drawdown: self.current_drawdown() <= self.config.max_drawdown_percent.abs(),
            within_position_limit: self.positions_today < self.config.max_positions,
        }
    }
}

#[derive(Debug, Clone)]
pub enum TradeSide {
    Long,
    Short,
}

#[derive(Debug, Clone)]
pub struct RiskCheckResult {
    pub within_daily_loss_limit: bool,
    pub within_max_drawdown: bool,
    pub within_position_limit: bool,
}

impl RiskCheckResult {
    pub fn is_safe(&self) -> bool {
        self.within_daily_loss_limit && self.within_max_drawdown && self.within_position_limit
    }
}

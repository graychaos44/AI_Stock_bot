use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use anyhow::{Result, Context};

const KIWOOOM_API_URL: &str = "https://api.kiwoom.com";

#[derive(Debug)]
pub struct KiwoomClient {
    client: Client,
    app_key: String,
    secret_key: String,
    access_token: Option<String>,
}

#[derive(Debug, Deserialize)]
struct TokenResponse {
    #[serde(rename = "token")]
    access_token: String,
    #[serde(rename = "token_type")]
    token_type: String,
    #[serde(rename = "expires_dt")]
    expires_dt: String,
    #[serde(rename = "return_code")]
    return_code: i32,
    #[serde(rename = "return_msg")]
    return_msg: String,
}

#[derive(Debug, Serialize)]
struct TokenRequest {
    #[serde(rename = "grant_type")]
    grant_type: String,
    #[serde(rename = "appkey")]
    appkey: String,
    #[serde(rename = "secretkey")]
    secretkey: String,
}

#[derive(Debug, Deserialize)]
struct BalanceResponse {
    #[serde(rename = "return_code")]
    return_code: i32,
    #[serde(rename = "return_msg")]
    return_msg: String,
    #[serde(default)]
    output: Vec<BalanceOutput>,
}

#[derive(Debug, Deserialize)]
struct BalanceOutput {
    #[serde(rename = "dnca_tot_amt", default)]
    deposit_total: String,
    #[serde(rename = "nxdy_excc_amt", default)]
    next_day_withdrawable: String,
    #[serde(rename = "prvs_rcdl_excc_amt", default)]
    previous_day_withdrawable: String,
}

impl KiwoomClient {
    pub fn new(app_key: String, secret_key: String) -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(30))
                .build()
                .expect("Failed to create HTTP client"),
            app_key,
            secret_key,
            access_token: None,
        }
    }

    pub async fn authenticate(&mut self) -> Result<String> {
        let url = format!("{}/oauth2/token", KIWOOOM_API_URL);
        
        let request = TokenRequest {
            grant_type: "client_credentials".to_string(),
            appkey: self.app_key.clone(),
            secretkey: self.secret_key.clone(),
        };

        println!("🔑 OAuth 토큰 요청 중...");

        let response = self.client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .context("토큰 발급 요청 실패")?;

        let token: TokenResponse = response
            .json()
            .await
            .context("토큰 응답 파싱 실패")?;

        if token.return_code != 0 {
            anyhow::bail!("API 오류: {}", token.return_msg);
        }

        println!("✓ 토큰 발급 성공! (만료: {})\n", token.expires_dt);

        self.access_token = Some(token.access_token.clone());
        Ok(token.access_token)
    }

    pub async fn get_account_balance(&self,
        account_no: &str,
    ) -> Result<BalanceResponse> {
        let token = self.access_token.as_ref()
            .ok_or_else(|| anyhow::anyhow!("토큰이 없습니다. authenticate()를 먼저 호출하세요."))?;

        let url = format!("{}/trt/api/v1/account/balance", KIWOOOM_API_URL);

        println!("💰 계좌 잔고 조회 중...");
        println!("   계좌번호: {}", account_no);

        let response = self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json; charset=utf-8")
            .header("appkey", &self.app_key)
            .query(&[(
                "CANO", account_no),
                ("ACNT_PRDT_CD", "01"),
                ("INQR_DVSN", "00"),
            ])
            .send()
            .await
            .context("잔고 조회 요청 실패")?;

        println!("   HTTP Status: {}", response.status());

        let balance: BalanceResponse = response
            .json()
            .await
            .context("잔고 응답 파싱 실패")?;

        Ok(balance)
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("=== 키움증권 API 테스트 ===\n");

    let app_key = "HqqXP9tV01lUzqM9_D7MmzlocWRxPPGnl0652v0GEz0".to_string();
    let secret_key = "dOuWW2bxiEhc6G_OngdIX1LcwU9TwwtL7fezEN2hh-4".to_string();
    let account_no = "54690274";  // 계좌번호: 5469-0274

    let mut client = KiwoomClient::new(app_key, secret_key);
    
    // 1. 인증
    match client.authenticate().await {
        Ok(_) => {},
        Err(e) => {
            eprintln!("✗ 인증 실패: {}", e);
            return Ok(());
        }
    }

    // 2. 계좌 잔고 조회
    match client.get_account_balance(account_no).await {
        Ok(balance) => {
            println!("\n=== 계좌 잔고 조회 결과 ===");
            println!("Return Code: {}", balance.return_code);
            println!("Return Msg: {}", balance.return_msg);
            
            if !balance.output.is_empty() {
                for (i, item) in balance.output.iter().enumerate() {
                    println!("\n[항목 {}]", i + 1);
                    println!("  예수금 총금액: {}", item.deposit_total);
                    println!("  익일정산금액: {}", item.next_day_withdrawable);
                    println!("  가수정산금액: {}", item.previous_day_withdrawable);
                }
            } else {
                println!("\n조회된 잔고 정보가 없습니다.");
            }
        }
        Err(e) => {
            eprintln!("\n✗ 잔고 조회 실패: {}", e);
        }
    }

    println!("\n=== 테스트 완료 ===");
    Ok(())
}

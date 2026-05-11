use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use anyhow::{Result, Context};

const KIWOOOM_API_URL: &str = "https://api.kiwoom.com";
const APP_KEY: &str = "HqqXP9tV01lUzqM9_D7MmzlocWRxPPGnl0652v0GEz0";
const SECRET_KEY: &str = "dOuWW2bxiEhc6G_OngdIX1LcwU9TwwtL7fezEN2hh-4";

#[derive(Debug)]
pub struct KiwoomClient {
    client: Client,
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

#[derive(Debug, Deserialize)]
struct QuoteResponse {
    #[serde(rename = "return_code")]
    return_code: i32,
    #[serde(rename = "return_msg")]
    return_msg: String,
    #[serde(default)]
    output: Option<QuoteOutput>,
}

#[derive(Debug, Deserialize)]
struct QuoteOutput {
    #[serde(rename = "stck_prpr", default)]
    current_price: String,
    #[serde(rename = "stck_oprc", default)]
    open_price: String,
    #[serde(rename = "stck_hgpr", default)]
    high_price: String,
    #[serde(rename = "stck_lwpr", default)]
    low_price: String,
    #[serde(rename = "prdy_vrss", default)]
    change: String,
    #[serde(rename = "prdy_vrss_sign", default)]
    change_sign: String,
    #[serde(rename = "acml_vol", default)]
    volume: String,
}

impl KiwoomClient {
    pub fn new() -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(30))
                .build()
                .expect("Failed to create HTTP client"),
            access_token: None,
        }
    }

    pub async fn authenticate(&mut self) -> Result<String> {
        let url = format!("{}/oauth2/token", KIWOOOM_API_URL);
        
        let request = serde_json::json!({
            "grant_type": "client_credentials",
            "appkey": APP_KEY,
            "secretkey": SECRET_KEY
        });

        println!("🔑 토큰 발급 중...");

        let response = self.client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        let token: TokenResponse = response.json().await?;

        if token.return_code != 0 {
            anyhow::bail!("인증 실패: {}", token.return_msg);
        }

        println!("✓ 토큰 발급 완료 (만료: {})\n", token.expires_dt);
        self.access_token = Some(token.access_token.clone());
        Ok(token.access_token)
    }

    pub async fn get_current_price(&self,
        stock_code: &str,
    ) -> Result<QuoteResponse> {
        let token = self.access_token.as_ref()
            .ok_or_else(|| anyhow::anyhow!("토큰이 없습니다"))?;

        let url = format!("{}/trt/api/v1/quote/price", KIWOOOM_API_URL);

        println!("📊 현재가 조회: {}", stock_code);

        let response = self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json; charset=utf-8")
            .header("appkey", APP_KEY)
            .header("secretkey", SECRET_KEY)
            .query(&[(
                "FID_COND_MRKT_DIV_CODE", "J"),  // J: 코스피, K: 코스닥
                ("FID_INPUT_ISCD", stock_code),
            ])
            .send()
            .await?;

        println!("   HTTP Status: {}", response.status());

        let quote: QuoteResponse = response.json().await?;
        Ok(quote)
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("=== 키움증권 API 테스트 ===\n");

    let mut client = KiwoomClient::new();
    
    // 1. 인증
    if let Err(e) = client.authenticate().await {
        eprintln!("✗ 인증 실패: {}", e);
        return Ok(());
    }

    // 2. 삼성전자 현재가 조회
    println!("\n=== 삼성전자(005930) 현재가 조회 ===");
    match client.get_current_price("005930").await {
        Ok(quote) => {
            println!("Return Code: {}", quote.return_code);
            println!("Return Msg: {}", quote.return_msg);
            
            if let Some(output) = quote.output {
                println!("\n📈 삼성전자(005930) 시세");
                println!("  현재가: {}원", output.current_price);
                println!("  시가: {}원", output.open_price);
                println!("  고가: {}원", output.high_price);
                println!("  저가: {}원", output.low_price);
                println!("  전일대비: {}{}원", 
                    if output.change_sign == "1" || output.change_sign == "2" { "+" } else { "-" },
                    output.change);
                println!("  거래량: {}주", output.volume);
            }
        }
        Err(e) => {
            eprintln!("✗ 현재가 조회 실패: {}", e);
        }
    }

    // 3. 카카오 현재가 조회
    println!("\n=== 카카오(035720) 현재가 조회 ===");
    match client.get_current_price("035720").await {
        Ok(quote) => {
            if let Some(output) = quote.output {
                println!("📈 카카오(035720) 시세");
                println!("  현재가: {}원", output.current_price);
                println!("  전일대비: {}{}원", 
                    if output.change_sign == "1" || output.change_sign == "2" { "+" } else { "-" },
                    output.change);
            }
        }
        Err(e) => {
            eprintln!("✗ 현재가 조회 실패: {}", e);
        }
    }

    println!("\n=== 테스트 완료 ===");
    Ok(())
}

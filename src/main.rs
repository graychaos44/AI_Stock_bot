use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use anyhow::Result;

// 모의투자 설정
const MOCK_API_URL: &str = "https://api.kiwoom.com";
const APP_KEY: &str = "yRzgDtcezIFc9THMC0GHpxMti_KCCRAx1ZOIVX__8-Q";
const APP_SECRET: &str = "QNHZaW4jK3IwuUvLldMAjus_c4l_9mcORus3-FDyyxw";
const MOCK_ACCOUNT: &str = "81260473";

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
struct BalanceResponse {
    #[serde(rename = "return_code")]
    return_code: i32,
    #[serde(rename = "return_msg")]
    return_msg: String,
    #[serde(default)]
    output: Option<serde_json::Value>,
}

pub struct KiwoomClient {
    client: Client,
    access_token: Option<String>,
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
        let url = format!("{}/oauth2/token", MOCK_API_URL);
        
        let request = serde_json::json!({
            "grant_type": "client_credentials",
            "appkey": APP_KEY,
            "secretkey": APP_SECRET
        });

        println!("🔑 모의투자 토큰 발급 중...");

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

        println!("✓ 토큰 발급 완료");
        println!("  만료: {}\n", token.expires_dt);
        
        self.access_token = Some(token.access_token.clone());
        Ok(token.access_token)
    }

    pub async fn get_mock_balance(&self,
    ) -> Result<BalanceResponse> {
        let token = self.access_token.as_ref()
            .ok_or_else(|| anyhow::anyhow!("토큰이 없습니다"))?;

        // 모의투자 엔드포인트 테스트 (여러 경로 시도)
        let endpoints = vec![
            "/trt/api/v1/mock/account/balance",
            "/trt/api/v1/account/balance",
            "/api/v1/mock/account/balance",
        ];

        for endpoint in &endpoints {
            let url = format!("{}{}", MOCK_API_URL, endpoint);
            println!("💰 계좌 잔고 조회 시도: {}", endpoint);

            let response = self.client
                .get(&url)
                .header("Authorization", format!("Bearer {}", token))
                .header("Content-Type", "application/json; charset=utf-8")
                .header("appkey", APP_KEY)
                .query(&[(
                    "CANO", MOCK_ACCOUNT),
                    ("ACNT_PRDT_CD", "01"),
                    ("INQR_DVSN", "00"),
                ])
                .send()
                .await?;

            println!("   HTTP Status: {}", response.status());

            let text = response.text().await?;
            println!("   응답: {}\n", &text[..text.len().min(200)]);

            // JSON 파싱 시도
            if let Ok(balance) = serde_json::from_str::<BalanceResponse>(&text) {
                if balance.return_code == 0 {
                    return Ok(balance);
                }
            }
        }

        anyhow::bail!("모든 엔드포인트 실패")
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("=== 키움증권 모의투자 API 테스트 ===\n");
    println!("계좌: {}\n", MOCK_ACCOUNT);

    let mut client = KiwoomClient::new();
    
    // 1. 인증
    if let Err(e) = client.authenticate().await {
        eprintln!("✗ 인증 실패: {}", e);
        return Ok(());
    }

    // 2. 모의투자 계좌 잔고 조회
    println!("=== 모의투자 계좌 잔고 조회 ===");
    match client.get_mock_balance().await {
        Ok(balance) => {
            println!("Return Code: {}", balance.return_code);
            println!("Return Msg: {}", balance.return_msg);
            if let Some(output) = balance.output {
                println!("Output: {:?}", output);
            }
        }
        Err(e) => {
            eprintln!("✗ 잔고 조회 실패: {}", e);
        }
    }

    println!("\n=== 테스트 완료 ===");
    Ok(())
}

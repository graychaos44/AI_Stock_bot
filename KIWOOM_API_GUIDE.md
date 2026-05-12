# 키움 모의투자 API 가이드

## ⚠️ 중요 수정 (2026-05-12)

### API URL 수정
- ❌ 기존: `https://api.kiwoom.com/trt/api/v1/mock` (실전용, 500 에러)
- ✅ 수정: `https://mockapi.kiwoom.com` (모의투자용)

### 토큰 발급
- URL: `https://mockapi.kiwoom.com/oauth2/token`
- Method: POST
- Body: `{"grant_type": "client_credentials", "appkey": "yRzgDtcezIFc9THMC0GHpxMti_KCCRAx1ZOIVX__8-Q", "secretkey": "QNHZaW4jK3IwuUvLldMAjus_c4l_9mcORus3-FDyyxw"}`
- 토큰 발급 성공 확인됨

### 시세 조회 (아직 500 에러, 장시간에만 작동 가능)
- 엔드포인트 후보:
  - `/trt/api/v1/mock/quote/price`
  - `/quote/price`
  - `/uapi/domestic-stock/v1/quotations/inquire-price`

### 모의투자 계좌
- 계좌번호: 81260473
- 기간: 2026-05-11 ~ 2026-06-11

### 수집 종목 (9개)
| 코드 | 종목명 | 업종 |
|------|--------|------|
| 005930 | 삼성전자 | 반도체 |
| 000660 | SK하이닉스 | 반도체 |
| 005380 | 현대차 | 자동차 |
| 035420 | NAVER | IT/플랫폼 |
| 051910 | LG화학 | 화학/배터리 |
| 006400 | 삼성SDI | 배터리 |
| 028260 | 삼성물산 | 건설/무역 |
| 068270 | 셀트리온 | 바이오 |
| 207940 | 삼성바이오로직스 | 바이오/CDMO |

### 수정된 수집 스크립트
- `/home/gray/stock_data/kiwoom_collector_fixed.py`에 이미 수정된 스크립트가 있음
- 이 스크립트를 기반으로 작업할 것

### 학습 자료 (이미 수집 완료)
- Mac Studio: `/Users/gray/stock_data/manuals/` — 25개 파일, 93MB
- Linux: `/home/gray/stock_data/manuals/` — 콘솔이 수집한 파일들
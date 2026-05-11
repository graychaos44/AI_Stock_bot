#!/bin/bash
# 키움증권 데이터 수집 스크립트
# 실행: ./start_collection.sh

echo "========================================"
echo "   키움증권 데이터 수집 시작"
echo "   2026-05-11 21:37:10"
echo "========================================"
echo ""

# 설정
export RUST_LOG=info
export KIWOOM_APP_KEY="yRzgDtcezIFc9THMC0GHpxMti_KCCRAx1ZOIVX__8-Q"
export KIWOOM_SECRET_KEY="QNHZaW4jK3IwuUvLldMAjus_c4l_9mcORus3-FDyyxw"
export MOCK_ACCOUNT="81260473"

# 9개 종목 설정
codes=("005930" "000660" "005380" "006400" "012330" "015760" "010120" "272210" "207940")
names=("삼성전자" "SK하이닉스" "현대차" "삼성SDI" "현대모비스" "한국전력" "LS ELECTRIC" "한화시스템" "삼성바이오로직스")

echo "[수집 대상 종목]"
for i in "${!codes[@]}"; do
    echo "  $((i+1). ${names[i]} (${codes[i]})"
done
echo ""

# 디렉토리 설정
mkdir -p ~/stock-trader/data/{raw,processed,logs}
log_file="~/stock-trader/data/logs/collection_$(date +%Y%m%d_%H%M%S).log"

echo "[시작] 데이터 수집을 시작합니다..."
echo "  로그 파일: $log_file"
echo ""

# Rust 프로그램 실행 (백그라운드)
cd ~/stock-trader
cargo run --release --bin collector 2>> \$log_file\ &
COLLECTOR_PID=$!

echo "  PID: $COLLECTOR_PID"
echo ""
echo "데이터 수집 중... (종료하려면 Ctrl+C)"
echo ""

# 15:30까지 대기 (장 마감)
target_time=$(date -v+15H -v+30M +%s 2>/dev/null || date -d "today 15:30" +%s)
now=$(date +%s)
wait_seconds=$((target_time - now))

if [ $wait_seconds -gt 0 ]; then
    echo "장 마감(15:30)까지 $((wait_seconds/60))분 대기..."
    sleep $wait_seconds
fi

# 종료
echo ""
echo "[종료] 장 마강 - 데이터 수집 종료"
kill $COLLECTOR_PID 2>/dev/null
echo "========================================"
echo "   데이터 수집 완료"
echo "   2026-05-11 21:37:10"
echo "========================================"

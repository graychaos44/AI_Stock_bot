# MLX 모델 학습 및 변환 메뉴얼 (Jarvis용)

## 목표
Ollama에서 다운로드한 Llama 3.1 8B 모델을 MLX 형식으로 변환하고, 주식 데이터로 파인튜닝하여 맥미니에 배포

## 실행 환경
- **장비**: M4 Max Studio (48GB)
- **OS**: macOS
- **사용자**: gray (SSH 접근 가능)
- **대상**: M4 Mac Mini (192.168.50.33) - 추론용

---

## Phase 1: 환경 설정

### 1.1 필수 패키지 설치
```bash
# Python 환경 확인
python3 --version  # 3.10+ 필요

# MLX 설치
pip install mlx-lm
pip install mlx

# 기타 도구
pip install torch transformers datasets
```

### 1.2 Ollama 모델 위치 확인
```bash
# Ollama 모델 캐시 위치
ls ~/.ollama/models/

# llama3.1:8b 확인
ls ~/.ollama/models/manifests/registry.ollama.ai/library/llama3.1/
```

---

## Phase 2: GGUF 추출

### 2.1 Ollama 모델 다운로드 (이미 되어 있음)
```bash
# 모델 확인
ollama list | grep llama3.1

# 없으면 다운로드
ollama pull llama3.1:8b
```

### 2.2 GGUF 파일 추출
```bash
# Ollama 모델 캐시 디렉토리
OLLAMA_DIR="$HOME/.ollama/models"

# manifests에서 blob 파일 찾기
MANIFEST=$(cat "$OLLAMA_DIR/manifests/registry.ollama.ai/library/llama3.1/8b" 2>/dev/null || echo "")

# blob 파일 복사
mkdir -p ~/mlx_training
find "$OLLAMA_DIR/blobs" -name "sha256-*" -size +4G | head -1 | xargs -I {} cp {} ~/mlx_training/model.gguf

echo "✓ GGUF 파일 추출 완료"
ls -lh ~/mlx_training/model.gguf
```

---

## Phase 3: GGUF → MLX 변환

### 3.1 MLX 형식으로 변환
```bash
cd ~/mlx_training

# mlx_lm.convert로 변환
python -m mlx_lm.convert \
  --gguf-model ./model.gguf \
  --mlx-model ./llama3.1-8b-mlx \
  --q-bits 4

echo "✓ MLX 변환 완료"
ls -lh ./llama3.1-8b-mlx/
```

### 3.2 변환 결과 확인
```bash
# 변환된 파일 목록
ls -lh ~/mlx_training/llama3.1-8b-mlx/

# 예상 파일:
# - weights.npz
# - config.json
# - tokenizer.json
```

---

## Phase 4: 주식 데이터 준비

### 4.1 학습 데이터 형식 (JSONL)
```json
{"instruction": "삼성전자 현재 RSI가 30이고 MACD 골든크로스 발생. 분석해줘.", "response": "과매도 구간 진입 및 추세 반전 신호. 단기 반등 가능성 높음. 매수 관점 접근 권장."}
{"instruction": "현대차 거래량이 평소 대비 200% 급증했고 주가는 5% 상승. 어떻게 해석?", "response": "거래량 동반 상승은 강한 매수세 신호. 단, 과열 가능성 확인 필요."}
```

### 4.2 데이터 파일 생성
```bash
mkdir -p ~/mlx_training/data

# 데이터 파일 생성 (예시 - 실제로는 더 많은 데이터 필요)
cat > ~/mlx_training/data/stock_training.jsonl << 'EOF'
{"instruction": "KOSPI 200의 현재 RSI가 30 이하이고 MACD 골든크로스가 발생했습니다. 분석해줘.", "response": "과매도 구간 진입 및 추세 반전 신호가 포착되었습니다. 단기 반등 가능성이 높으므로 매수 관점으로 접근을 권장합니다."}
{"instruction": "삼성전자 거래량이 평소 대비 3배 급증했습니다. 주가는 상승 중입니다. 어떻게 해석하나요?", "response": "거래량 동반 상승은 강한 매수세를 의미합니다. 단, 과열 가능성도 있어 추가 확인이 필요합니다."}
{"instruction": "현대차 2차 전지 관련주로 편입되었습니다. 장기 투자 관점에서 어떤가요?", "response": "EV 전환 트렌드 수혜 기대 가능. 단, 밸류에이션과 업황 확인 후 진입 권장."}
EOF

echo "✓ 학습 데이터 준비 완료"
wc -l ~/mlx_training/data/stock_training.jsonl
```

---

## Phase 5: QLoRA 파인튜닝

### 5.1 학습 실행
```bash
cd ~/mlx_training

# QLoRA 학습
python -m mlx_lm.lora \
  --model ./llama3.1-8b-mlx \
  --data ./data/stock_training.jsonl \
  --iters 1000 \
  --batch-size 4 \
  --learning-rate 1e-5 \
  --lora-r 8 \
  --lora-alpha 16 \
  --lora-dropout 0.1 \
  --save-every 100 \
  --adapter-path ./adapters

echo "✓ 학습 완료"
ls -lh ./adapters/
```

### 5.2 학습 모니터링
```bash
# 학습 로그 확인
tail -f ~/mlx_training/training.log

# GPU 사용률 확인
while true; do
  echo "=== $(date) ==="
  top -l 1 | grep -E "(python|mlx)"
  sleep 60
done
```

---

## Phase 6: 모델 병합 및 내보내기

### 6.1 어댑터 병합
```bash
cd ~/mlx_training

# 기본 모델 + 어댑터 병합
python -m mlx_lm.fuse \
  --model ./llama3.1-8b-mlx \
  --adapter-path ./adapters \
  --save-path ./stock-expert-mlx

echo "✓ 모델 병합 완료"
ls -lh ./stock-expert-mlx/
```

### 6.2 GGUF 변환 (맥미니용)
```bash
# MLX → GGUF 변환 (4-bit)
python -m mlx_lm.convert \
  --mlx-model ./stock-expert-mlx \
  --gguf-model ./stock-expert.gguf \
  --q-bits 4

echo "✓ GGUF 변환 완료"
ls -lh ./stock-expert.gguf
```

---

## Phase 7: 맥미니로 전송

### 7.1 파일 전송
```bash
# GGUF 파일을 맥미니로 전송
scp -P 22 \
  ~/mlx_training/stock-expert.gguf \
  gray@192.168.50.33:~/models/

echo "✓ 맥미니로 전송 완료"
```

### 7.2 맥미니에서 Ollama 등록
```bash
# 맥미니 SSH 접속 후
ssh gray@192.168.50.33

# Modelfile 생성
cat > ~/models/Modelfile << 'EOF'
FROM ./stock-expert.gguf

SYSTEM """당신은 주식 투자 분석 전문가입니다.
기술적 지표와 시장 데이터를 바탕으로 객관적인 분석을 제공합니다.
투자 결정은 사용자가 직접 내리며, 당신은 분석가로서 참고 자료를 제공합니다."""

PARAMETER temperature 0.7
PARAMETER top_p 0.9
PARAMETER top_k 40
PARAMETER num_ctx 4096
EOF

# Ollama에 모델 등록
ollama create stock-expert -f ~/models/Modelfile

# 등록 확인
ollama list | grep stock-expert

echo "✓ Ollama 등록 완료"
```

---

## Phase 8: 테스트

### 8.1 모델 테스트
```bash
# 맥미니에서 테스트
ollama run stock-expert

# 테스트 입력
>>> 삼성전자 RSI가 25이고 MACD 골든크로스가 발생했습니다. 어떻게 분석하나요?
```

### 8.2 예상 출력
```
과매도 구간(RSI 25)에서 MACD 골든크로스가 발생했습니다.
이는 단기 반등 가능성이 높은 신호입니다.
추가 확인 사항: 거래량 동반 여부, 지지선 확인
분석가 의견: 매수 관점으로 모니터링 권장
```

---

## 체크리스트

- [ ] MLX 설치 완료
- [ ] Ollama 모델 다운로드 확인
- [ ] GGUF 추출 완료
- [ ] MLX 변환 완료
- [ ] 학습 데이터 준비 (최소 1000개 샘플 권장)
- [ ] QLoRA 학습 완료
- [ ] 어댑터 병합 완료
- [ ] GGUF 변환 완료
- [ ] 맥미니 전송 완료
- [ ] Ollama 등록 완료
- [ ] 테스트 완료

---

## 문제 해결

### 문제: 메모리 부족
```bash
# 해결: 더 작은 배치 사이즈
python -m mlx_lm.lora \
  --model ./llama3.1-8b-mlx \
  --data ./data/stock_training.jsonl \
  --iters 1000 \
  --batch-size 1  # 감소 \
  --learning-rate 1e-5
```

### 문제: 학습 시간 너무 오래 걸림
```bash
# 해결: iterations 감소
python -m mlx_lm.lora \
  --model ./llama3.1-8b-mlx \
  --data ./data/stock_training.jsonl \
  --iters 500  # 감소
```

### 문제: 변환 실패
```bash
# 해결: 직접 변환 스크립트 사용
python << 'EOF'
from mlx_lm import convert
convert.convert_llama_to_mlx(
    gguf_path="./model.gguf",
    mlx_path="./llama3.1-8b-mlx",
    quantize=True,
    q_bits=4
)
EOF
```

---

## 참고 자료

- MLX 공식 문서: https://ml-explore.github.io/mlx/
- mlx-lm GitHub: https://github.com/ml-explore/mlx-examples
- Ollama 문서: https://github.com/ollama/ollama

---

**작성일**: 2026-05-11
**작성자**: OpenClaw
**버전**: 1.0
**대상**: Jarvis (M4 Max Studio 관리자)

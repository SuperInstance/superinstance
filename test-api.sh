#!/bin/bash
# Test SuperInstance API endpoints

set -e

BASE_URL="http://localhost:3001"

echo "🧪 Testing SuperInstance API..."
echo ""

# Test 1: List breeds
echo "1️⃣ Testing GET /api/breeds"
curl -s "$BASE_URL/api/breeds" | jq '.' || echo "Failed"
echo ""
echo ""

# Test 2: Get specific breed
echo "2️⃣ Testing GET /api/breeds/cattle"
curl -s "$BASE_URL/api/breeds/cattle" | jq '.' || echo "Failed"
echo ""
echo ""

# Test 3: Chat endpoint (main test)
echo "3️⃣ Testing POST /api/chat"
curl -s -X POST "$BASE_URL/api/chat" \
  -H "Content-Type: application/json" \
  -d '{"prompt": "Hello, can you help me?"}' | jq '.' || echo "Failed"
echo ""
echo ""

# Test 4: Cattle endpoint
echo "4️⃣ Testing POST /api/cattle"
curl -s -X POST "$BASE_URL/api/cattle" \
  -H "Content-Type: application/json" \
  -d '{"message": "What is the weather today?"}' | jq '.' || echo "Failed"
echo ""
echo ""

# Test 5: Collie endpoint
echo "5️⃣ Testing POST /api/collie"
curl -s -X POST "$BASE_URL/api/collie" \
  -H "Content-Type: application/json" \
  -d '{"message": "Tell me about the ranch"}' | jq '.' || echo "Failed"
echo ""
echo ""

# Test 6: Evolution trigger
echo "6️⃣ Testing POST /api/evolution"
curl -s -X POST "$BASE_URL/api/evolution" | jq '.' || echo "Failed"
echo ""
echo ""

# Test 7: Pasture sync
echo "7️⃣ Testing POST /api/sync"
curl -s -X POST "$BASE_URL/api/sync" \
  -H "Content-Type: application/json" \
  -d '{
    "pasture_id": "cattle-v1",
    "update": "",
    "timestamp": 0
  }' | jq '.' || echo "Failed"
echo ""

echo "✅ API tests complete!"

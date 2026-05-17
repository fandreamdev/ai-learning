#!/usr/bin/env bash
# =====================================================
# ProjectAlpha 联调走查脚本（spec §15.4 + plan §7.4）
#
# 假设：
#   - 后端运行在 :8000
#   - 前端运行在 :5173（用于验证 Vite 代理）
#
# 用法：
#   bash week1/tools/integration-check.sh
#
# 退出码：0 全过；非 0 = 失败用例数
# =====================================================
set -uo pipefail

BACKEND_URL="${BACKEND_URL:-http://127.0.0.1:8000}"
FRONTEND_URL="${FRONTEND_URL:-http://localhost:5173}"

PASS=0
FAIL=0
SECTION=""

# 临时响应文件：在 Windows Git Bash 下 /tmp 不存在，用当前目录下的文件
TMP_RESP="${TMPDIR:-.}/_integration_resp_$$.json"
trap 'rm -f "$TMP_RESP"' EXIT

section() {
  SECTION="$*"
  echo ""
  echo "=== $SECTION ==="
}

pass() { echo "  ✅ $*"; PASS=$((PASS + 1)); }
fail() { echo "  ❌ $*"; FAIL=$((FAIL + 1)); }

# 提取 JSON 字段：jq 不一定可用，用 python 兜底
jget() {
  python -c "import sys, json; d=json.load(sys.stdin); print($1, end='')"
}

# ---------------------------------------------------------------------------
# 1) /healthz 与 /readyz
# ---------------------------------------------------------------------------
test_health() {
  section "1. Health & Readiness"

  local body
  body=$(curl -s "$BACKEND_URL/healthz")
  if echo "$body" | grep -q '"code":0' && echo "$body" | grep -q '"data":"ok"'; then
    pass "/healthz returns unified success"
  else
    fail "/healthz unexpected: $body"
  fi

  body=$(curl -s "$BACKEND_URL/readyz")
  if echo "$body" | grep -q '"code":0' && echo "$body" | grep -q '"data":"ready"'; then
    pass "/readyz reports ready (DB ping ok)"
  else
    fail "/readyz unexpected: $body"
  fi
}

# ---------------------------------------------------------------------------
# 2) Vite 代理透传
# ---------------------------------------------------------------------------
test_proxy() {
  section "2. Vite proxy"

  local code
  code=$(curl -s -o /dev/null -w "%{http_code}" "$FRONTEND_URL/api/v1/tickets?page_size=1")
  if [[ "$code" == "200" ]]; then
    pass "GET /api/v1/tickets via 5173 -> 200"
  else
    fail "Proxy GET unexpected HTTP $code"
  fi

  local html
  html=$(curl -s -o /dev/null -w "%{http_code}" "$FRONTEND_URL/")
  if [[ "$html" == "200" ]]; then
    pass "Frontend root / -> 200"
  else
    fail "Frontend root unexpected HTTP $html"
  fi
}

# ---------------------------------------------------------------------------
# 3) 列表接口字段集 + 时间格式
# ---------------------------------------------------------------------------
test_list_shape_and_time() {
  section "3. List response shape & ISO time"

  local body
  body=$(curl -s "$BACKEND_URL/api/v1/tickets?page_size=1")
  local missing
  missing=$(echo "$body" | python -c '
import json, sys
d = json.load(sys.stdin)
required_top = {"code", "message", "data"}
required_data = {"items", "total", "page", "page_size"}
missing = []
if set(d.keys()) < required_top:
    missing.append(f"top-level missing: {required_top - set(d.keys())}")
data = d["data"]
if set(data.keys()) < required_data:
    missing.append(f"data missing: {required_data - set(data.keys())}")
if data["items"]:
    item = data["items"][0]
    item_required = {"id","title","description","status","priority","assignee","tags","created_at","updated_at"}
    if set(item.keys()) < item_required:
        missing.append(f"item missing: {item_required - set(item.keys())}")
print("|".join(missing))
')
  if [[ -z "$missing" ]]; then
    pass "List response has all required fields"
  else
    fail "Field shape mismatch: $missing"
  fi

  local time_check
  time_check=$(echo "$body" | python -c '
import json, sys, re
d = json.load(sys.stdin)
items = d["data"]["items"]
if not items:
    print("no_items")
else:
    t = items[0]["created_at"]
    pattern = r"^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}"
    print("ok" if re.match(pattern, t) else f"bad:{t}")
')
  case "$time_check" in
    ok) pass "created_at uses ISO 8601 format" ;;
    no_items) pass "(no items to verify time format; skipping)" ;;
    *) fail "Time format unexpected: $time_check" ;;
  esac
}

# ---------------------------------------------------------------------------
# 4) 多状态筛选：CSV vs 重复参数等价
# ---------------------------------------------------------------------------
test_multi_status_equivalence() {
  section "4. Multi-status: CSV vs repeated params"

  local csv repeat
  csv=$(curl -s "$BACKEND_URL/api/v1/tickets?status=open,in_progress&page_size=100" | python -c 'import json,sys; print(json.load(sys.stdin)["data"]["total"])')
  repeat=$(curl -s "$BACKEND_URL/api/v1/tickets?status=open&status=in_progress&page_size=100" | python -c 'import json,sys; print(json.load(sys.stdin)["data"]["total"])')
  if [[ "$csv" == "$repeat" ]]; then
    pass "CSV vs repeated yield same total ($csv)"
  else
    fail "Mismatch: csv=$csv, repeat=$repeat"
  fi
}

# ---------------------------------------------------------------------------
# 5) 状态/优先级枚举与文档对齐
# ---------------------------------------------------------------------------
test_enum_alignment() {
  section "5. Enum alignment with spec"

  local statuses priorities
  statuses=$(curl -s "$BACKEND_URL/api/v1/tickets?page_size=100" | python -c '
import json,sys
items = json.load(sys.stdin)["data"]["items"]
print(",".join(sorted({i["status"] for i in items})))
')
  priorities=$(curl -s "$BACKEND_URL/api/v1/tickets?page_size=100" | python -c '
import json,sys
items = json.load(sys.stdin)["data"]["items"]
print(",".join(sorted({i["priority"] for i in items})))
')
  expected_statuses="closed,done,in_progress,open"
  expected_priorities="high,low,medium,urgent"
  if [[ "$statuses" == "$expected_statuses" ]]; then
    pass "Status enum: $statuses"
  else
    fail "Status enum mismatch: got $statuses (expect $expected_statuses)"
  fi
  if [[ "$priorities" == "$expected_priorities" ]]; then
    pass "Priority enum: $priorities"
  else
    fail "Priority enum mismatch: got $priorities (expect $expected_priorities)"
  fi
}

# ---------------------------------------------------------------------------
# 6) tags 自动 lower / 去重 / 空数组默认
# ---------------------------------------------------------------------------
test_tags_normalize() {
  section "6. Tags lower-cased, deduped, default to []"

  local body id tags_check
  body=$(curl -s -X POST "$BACKEND_URL/api/v1/tickets" \
    -H 'Content-Type: application/json' \
    --data '{"title":"tags norm test","tags":["Bug","bug","BUG","Frontend"]}')
  id=$(echo "$body" | python -c 'import json,sys; print(json.load(sys.stdin)["data"]["id"])')
  tags_check=$(echo "$body" | python -c '
import json, sys
tags = json.load(sys.stdin)["data"]["tags"]
print("ok:" + ",".join(tags) if tags == ["bug", "frontend"] else "bad:" + repr(tags))
')
  if [[ "$tags_check" == ok:* ]]; then
    pass "Tags normalized: ${tags_check#ok:}"
  else
    fail "Tags unexpected: ${tags_check#bad:}"
  fi

  # 空 tags
  local body2 tags_empty id2
  body2=$(curl -s -X POST "$BACKEND_URL/api/v1/tickets" \
    -H 'Content-Type: application/json' \
    --data '{"title":"no tags"}')
  tags_empty=$(echo "$body2" | python -c '
import json, sys
tags = json.load(sys.stdin)["data"]["tags"]
print("empty" if tags == [] else "bad:" + repr(tags))
')
  id2=$(echo "$body2" | python -c 'import json,sys; print(json.load(sys.stdin)["data"]["id"])')
  if [[ "$tags_empty" == "empty" ]]; then
    pass "Default tags is [] (not null)"
  else
    fail "Default tags unexpected: ${tags_empty#bad:}"
  fi

  # 清理
  curl -s -X DELETE "$BACKEND_URL/api/v1/tickets/$id" > /dev/null
  curl -s -X DELETE "$BACKEND_URL/api/v1/tickets/$id2" > /dev/null
}

# ---------------------------------------------------------------------------
# 7) 状态非法流转 -> 40002
# ---------------------------------------------------------------------------
test_illegal_status_transition() {
  section "7. Illegal status transition -> 40002"

  local body id code msg http
  body=$(curl -s -X POST "$BACKEND_URL/api/v1/tickets" \
    -H 'Content-Type: application/json' --data '{"title":"trans test"}')
  id=$(echo "$body" | python -c 'import json,sys; print(json.load(sys.stdin)["data"]["id"])')
  curl -s -X PATCH "$BACKEND_URL/api/v1/tickets/$id/status" \
    -H 'Content-Type: application/json' --data '{"status":"in_progress"}' > /dev/null

  http=$(curl -s -o "$TMP_RESP" -w "%{http_code}" \
    -X PATCH "$BACKEND_URL/api/v1/tickets/$id/status" \
    -H 'Content-Type: application/json' --data '{"status":"open"}')
  code=$(python -c 'import json,sys; print(json.load(sys.stdin)["code"])' < "$TMP_RESP")
  msg=$(python -c 'import json,sys; print(json.load(sys.stdin)["message"])' < "$TMP_RESP")
  if [[ "$http" == "400" && "$code" == "40002" ]]; then
    pass "Illegal transition returns 400/40002: $msg"
  else
    fail "Illegal transition unexpected http=$http code=$code msg=$msg"
  fi

  curl -s -X DELETE "$BACKEND_URL/api/v1/tickets/$id" > /dev/null
}

# ---------------------------------------------------------------------------
# 8) page_size > 100 -> 40001
# ---------------------------------------------------------------------------
test_page_size_limit() {
  section "8. page_size > 100 -> 40001"

  local http code
  http=$(curl -s -o "$TMP_RESP" -w "%{http_code}" "$BACKEND_URL/api/v1/tickets?page_size=200")
  code=$(python -c 'import json,sys; print(json.load(sys.stdin).get("code"))' < "$TMP_RESP")
  if [[ "$http" == "400" && "$code" == "40001" ]]; then
    pass "page_size=200 -> 400/40001"
  else
    fail "page_size=200 unexpected http=$http code=$code"
  fi
}

# ---------------------------------------------------------------------------
# 9) 删除后 GET -> 40401
# ---------------------------------------------------------------------------
test_delete_then_404() {
  section "9. Delete then GET -> 40401"

  local body id http code
  body=$(curl -s -X POST "$BACKEND_URL/api/v1/tickets" \
    -H 'Content-Type: application/json' --data '{"title":"delete test"}')
  id=$(echo "$body" | python -c 'import json,sys; print(json.load(sys.stdin)["data"]["id"])')
  curl -s -X DELETE "$BACKEND_URL/api/v1/tickets/$id" > /dev/null
  http=$(curl -s -o "$TMP_RESP" -w "%{http_code}" "$BACKEND_URL/api/v1/tickets/$id")
  code=$(python -c 'import json,sys; print(json.load(sys.stdin)["code"])' < "$TMP_RESP")
  if [[ "$http" == "404" && "$code" == "40401" ]]; then
    pass "Deleted ticket returns 404/40401"
  else
    fail "Unexpected http=$http code=$code"
  fi
}

# ---------------------------------------------------------------------------
# 10) 关键字命中描述
# ---------------------------------------------------------------------------
test_keyword_in_description() {
  section "10. Keyword matches description (LIKE)"

  local total
  total=$(curl -s "$BACKEND_URL/api/v1/tickets?keyword=captcha&page_size=100" | python -c 'import json,sys; print(json.load(sys.stdin)["data"]["total"])')
  if [[ "$total" -ge 1 ]]; then
    pass "keyword=captcha matched $total ticket(s)"
  else
    fail "keyword=captcha matched 0; expected ≥ 1 (seed has it in description)"
  fi
}

# ---------------------------------------------------------------------------
# 11) keyword < 2 字符 -> 400
# ---------------------------------------------------------------------------
test_keyword_min_length() {
  section "11. keyword length < 2 -> 400"

  local http
  http=$(curl -s -o /dev/null -w "%{http_code}" "$BACKEND_URL/api/v1/tickets?keyword=a")
  if [[ "$http" == "400" ]]; then
    pass "keyword=a -> 400"
  else
    fail "keyword=a unexpected http=$http"
  fi
}

# ---------------------------------------------------------------------------
# 12) 聚合接口去重排序
# ---------------------------------------------------------------------------
test_aggregations() {
  section "12. /tags and /assignees: deduped and sorted"

  local tags assignees ok
  tags=$(curl -s "$BACKEND_URL/api/v1/tags" | python -c 'import json,sys; print(json.load(sys.stdin)["data"])')
  assignees=$(curl -s "$BACKEND_URL/api/v1/assignees" | python -c 'import json,sys; print(json.load(sys.stdin)["data"])')
  ok=$(echo "$tags" | python -c 'import sys; t=eval(sys.stdin.read()); import sys; print("ok" if (t==sorted(t) and len(set(t))==len(t)) else "bad")')
  if [[ "$ok" == "ok" ]]; then
    pass "/tags is sorted and deduped: $tags"
  else
    fail "/tags not sorted/deduped: $tags"
  fi
  ok=$(echo "$assignees" | python -c 'import sys; t=eval(sys.stdin.read()); print("ok" if (t==sorted(t) and len(set(t))==len(t)) else "bad")')
  if [[ "$ok" == "ok" ]]; then
    pass "/assignees is sorted and deduped: $assignees"
  else
    fail "/assignees not sorted/deduped: $assignees"
  fi
}

# ---------------------------------------------------------------------------
# 13) 分页边界
# ---------------------------------------------------------------------------
test_pagination_boundary() {
  section "13. Pagination boundaries"

  local total page1_len last_page last_len overshoot_len
  total=$(curl -s "$BACKEND_URL/api/v1/tickets?page_size=1" | python -c 'import json,sys; print(json.load(sys.stdin)["data"]["total"])')
  if [[ "$total" -lt 5 ]]; then
    fail "Need ≥ 5 tickets to test pagination (have $total)"
    return
  fi
  page1_len=$(curl -s "$BACKEND_URL/api/v1/tickets?page=1&page_size=2" | python -c 'import json,sys; print(len(json.load(sys.stdin)["data"]["items"]))')
  last_page=$(( (total + 1) / 2 ))
  last_len=$(curl -s "$BACKEND_URL/api/v1/tickets?page=$last_page&page_size=2" | python -c 'import json,sys; print(len(json.load(sys.stdin)["data"]["items"]))')
  overshoot_len=$(curl -s "$BACKEND_URL/api/v1/tickets?page=$((last_page + 5))&page_size=2" | python -c 'import json,sys; print(len(json.load(sys.stdin)["data"]["items"]))')
  if [[ "$page1_len" == "2" && "$last_len" -ge "1" && "$overshoot_len" == "0" ]]; then
    pass "Pagination boundaries OK (total=$total, last_page=$last_page)"
  else
    fail "Pagination unexpected: page1=$page1_len, last=$last_len, overshoot=$overshoot_len"
  fi
}

# ---------------------------------------------------------------------------
# 14) 创建后列表 total 增加（重新拉取）
# ---------------------------------------------------------------------------
test_create_then_list_refresh() {
  section "14. Create -> list total increases"

  local before after id body
  before=$(curl -s "$BACKEND_URL/api/v1/tickets?page_size=1" | python -c 'import json,sys; print(json.load(sys.stdin)["data"]["total"])')
  body=$(curl -s -X POST "$BACKEND_URL/api/v1/tickets" -H 'Content-Type: application/json' --data '{"title":"refresh test"}')
  id=$(echo "$body" | python -c 'import json,sys; print(json.load(sys.stdin)["data"]["id"])')
  after=$(curl -s "$BACKEND_URL/api/v1/tickets?page_size=1" | python -c 'import json,sys; print(json.load(sys.stdin)["data"]["total"])')
  if [[ $((after - before)) == "1" ]]; then
    pass "Total increased $before -> $after"
  else
    fail "Total mismatch: $before -> $after"
  fi
  curl -s -X DELETE "$BACKEND_URL/api/v1/tickets/$id" > /dev/null
}

# ---------------------------------------------------------------------------
# 15) 500 错误统一格式（无显式触发点；通过非法路由 + 5xx 校验，跳过）
#     这里用：未定义路由 -> 404 但走统一格式（模拟错误处理覆盖）
# ---------------------------------------------------------------------------
test_unified_error_format() {
  section "15. Unified error envelope on 4xx"

  local body code
  body=$(curl -s "$BACKEND_URL/api/v1/no-such-route")
  code=$(echo "$body" | python -c 'import json,sys; print(json.load(sys.stdin)["code"])')
  if [[ "$code" =~ ^[0-9]+$ && "$code" != "0" ]]; then
    pass "Error envelope present: $body"
  else
    fail "Error envelope missing: $body"
  fi
}

# ---------------------------------------------------------------------------
# main
# ---------------------------------------------------------------------------
main() {
  test_health
  test_proxy
  test_list_shape_and_time
  test_multi_status_equivalence
  test_enum_alignment
  test_tags_normalize
  test_illegal_status_transition
  test_page_size_limit
  test_delete_then_404
  test_keyword_in_description
  test_keyword_min_length
  test_aggregations
  test_pagination_boundary
  test_create_then_list_refresh
  test_unified_error_format

  echo ""
  echo "============================================="
  echo " Total: ✅ PASS=$PASS   ❌ FAIL=$FAIL"
  echo "============================================="

  if [[ "$FAIL" -ne 0 ]]; then
    exit 1
  fi
}

main "$@"

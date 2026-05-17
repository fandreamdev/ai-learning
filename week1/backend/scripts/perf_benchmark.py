"""NF01 性能基准。

在业务数据库中插入 1000 条 ticket，连续请求 ``GET /api/v1/tickets``
50 次取 p50 / p95 / max，断言 p50 < 500 ms（spec NF01）。

用法（需先启动后端 ``uvicorn app.main:app --port 8000``）::

    cd week1/backend
    uv run python -m scripts.perf_benchmark

退出码：``0`` 通过；``1`` 性能未达标；``2`` 后端未启动。
"""

from __future__ import annotations

import statistics
import sys
import time
import urllib.error
import urllib.request

from app.core.database import SessionLocal
from app.models.ticket import Ticket
from app.schemas.ticket import TicketCreate, TicketPriority
from app.services import ticket_service

# spec NF01：列表接口 < 500ms（1000 条数据量级下）
THRESHOLD_P50_MS = 500
THRESHOLD_P95_MS = 1000

URL = "http://127.0.0.1:8000/api/v1/tickets?page_size=20"
SEED_COUNT = 1000
SAMPLES = 50
WARMUP = 5

PRIORITIES: list[TicketPriority] = [
    TicketPriority.low,
    TicketPriority.medium,
    TicketPriority.high,
    TicketPriority.urgent,
]


def seed(n: int = SEED_COUNT) -> None:
    """清空 tickets 后批量写入 ``n`` 条。"""
    print(f"[seed] inserting {n} tickets ...")
    with SessionLocal() as db:
        db.execute(Ticket.__table__.delete())
        db.commit()
        for i in range(n):
            ticket_service.create_ticket(
                db,
                TicketCreate(
                    title=f"perf-{i:04d}",
                    description=f"benchmark ticket #{i}",
                    priority=PRIORITIES[i % 4],
                    assignee=f"user-{i % 5}",
                    tags=[f"tag-{i % 10}"],
                ),
            )
    print("[seed] done")


def cleanup() -> None:
    print("[cleanup] truncating tickets ...")
    with SessionLocal() as db:
        db.execute(Ticket.__table__.delete())
        db.commit()


def hit(url: str) -> float:
    """发送一次请求，返回毫秒。"""
    t0 = time.perf_counter()
    with urllib.request.urlopen(url) as resp:
        resp.read()
    return (time.perf_counter() - t0) * 1000.0


def main() -> int:
    # 后端连通性
    try:
        urllib.request.urlopen("http://127.0.0.1:8000/healthz", timeout=2.0)
    except (urllib.error.URLError, TimeoutError) as exc:
        print(f"[fatal] backend not reachable on :8000: {exc}", file=sys.stderr)
        return 2

    seed()

    # 预热
    for _ in range(WARMUP):
        hit(URL)

    samples = [hit(URL) for _ in range(SAMPLES)]
    samples.sort()
    p50 = statistics.median(samples)
    p95 = samples[int(0.95 * len(samples)) - 1]
    p99 = samples[int(0.99 * len(samples)) - 1]
    avg = statistics.fmean(samples)
    mx = max(samples)

    print()
    print("==== NF01 perf benchmark ====")
    print(f"  url        : {URL}")
    print(f"  samples    : {SAMPLES} (warmup {WARMUP})")
    print(f"  p50 (ms)   : {p50:7.1f}    threshold {THRESHOLD_P50_MS}")
    print(f"  p95 (ms)   : {p95:7.1f}    threshold {THRESHOLD_P95_MS}")
    print(f"  p99 (ms)   : {p99:7.1f}")
    print(f"  avg (ms)   : {avg:7.1f}")
    print(f"  max (ms)   : {mx:7.1f}")
    print("=============================")

    cleanup()

    failed = []
    if p50 >= THRESHOLD_P50_MS:
        failed.append(f"p50 {p50:.1f}ms ≥ {THRESHOLD_P50_MS}ms")
    if p95 >= THRESHOLD_P95_MS:
        failed.append(f"p95 {p95:.1f}ms ≥ {THRESHOLD_P95_MS}ms")

    if failed:
        for f in failed:
            print(f"FAIL: {f}", file=sys.stderr)
        return 1

    print("PASS: NF01 performance threshold met")
    return 0


if __name__ == "__main__":
    sys.exit(main())

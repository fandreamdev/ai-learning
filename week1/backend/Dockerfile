# =====================================================
# ProjectAlpha Backend Dockerfile
#
# Multi-stage build:
#   1. builder: install dependencies via uv into /app/.venv
#   2. runtime: copy .venv + app code, run alembic + uvicorn
#
# Build:    docker build -t pa-backend .
# Run:      docker run -p 8000:8000 --env-file ../.env.docker pa-backend
# Compose:  see ../docker-compose.yml
#
# Note: 默认基础镜像使用 docker.m.daocloud.io mirror（中国大陆访问 Docker Hub
# 经常超时）。国际网络环境可改回 python:3.13-slim。
# =====================================================

# ---------- builder ----------
FROM docker.m.daocloud.io/library/python:3.13-slim AS builder

ENV PYTHONUNBUFFERED=1 \
    PIP_NO_CACHE_DIR=1 \
    PIP_DISABLE_PIP_VERSION_CHECK=1 \
    PIP_INDEX_URL=https://pypi.tuna.tsinghua.edu.cn/simple \
    UV_INDEX_URL=https://pypi.tuna.tsinghua.edu.cn/simple

WORKDIR /app

# Install uv (deterministic, ~5MB)
RUN pip install --no-cache-dir uv==0.9.17

# Cache layer: only re-install when lockfile changes
COPY pyproject.toml uv.lock ./
RUN uv sync --frozen --no-dev

# ---------- runtime ----------
FROM docker.m.daocloud.io/library/python:3.13-slim

ENV PYTHONUNBUFFERED=1 \
    PYTHONDONTWRITEBYTECODE=1 \
    PATH="/app/.venv/bin:$PATH"

WORKDIR /app

# Copy the prebuilt virtualenv
COPY --from=builder /app/.venv /app/.venv

# Application code (only what we need at runtime)
COPY app ./app
COPY alembic ./alembic
COPY alembic.ini ./
COPY docker-entrypoint.sh ./
RUN chmod +x docker-entrypoint.sh

EXPOSE 8000

ENTRYPOINT ["./docker-entrypoint.sh"]

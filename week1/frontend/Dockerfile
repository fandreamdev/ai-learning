# =====================================================
# ProjectAlpha Frontend Dockerfile
#
# Multi-stage build:
#   1. builder: node:22-alpine -> npm ci + npm run build -> dist/
#   2. runtime: nginx:1.27-alpine serving dist + reverse-proxying /api
#
# Build:    docker build -t pa-frontend .
# Run:      docker run -p 80:80 pa-frontend
# Compose:  see ../docker-compose.yml
# =====================================================

# ---------- builder ----------
# 默认使用 docker.m.daocloud.io mirror（中国大陆访问 Docker Hub 经常超时）；
# 国际网络环境可改回 node:22-alpine
FROM docker.m.daocloud.io/library/node:22-alpine AS builder

WORKDIR /app

# Use China-friendly npm registry mirror inside the build container
RUN npm config set registry https://registry.npmmirror.com

# Cache layer: install deps before copying full source
COPY package.json package-lock.json ./
RUN npm ci --no-audit --no-fund

# Copy source. .dockerignore prunes node_modules / dist / e2e / etc.
COPY . .

# Build — outputs dist/
# build:docker 跳过 tsc -b project references；vite (esbuild) 负责 TS 转译。
# 类型检查在 dev 环境的 npm run typecheck / CI 中已经覆盖。
RUN npm run build:docker

# ---------- runtime ----------
FROM docker.m.daocloud.io/library/nginx:1.27-alpine

# Drop default config, install ours
RUN rm /etc/nginx/conf.d/default.conf
COPY nginx.conf /etc/nginx/conf.d/default.conf

# Static assets
COPY --from=builder /app/dist /usr/share/nginx/html

EXPOSE 80

# Default nginx CMD already runs in foreground; keep explicit for clarity
CMD ["nginx", "-g", "daemon off;"]

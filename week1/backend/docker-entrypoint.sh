#!/bin/sh
# ProjectAlpha backend container entrypoint:
#   1. wait for MySQL to accept connections (up to 30s)
#   2. ensure target DB exists (compose creates project_alpha; create test DB if missing)
#   3. run alembic upgrade head
#   4. exec uvicorn
set -e

DB_HOST="${DB_HOST:-mysql}"
DB_PORT="${DB_PORT:-3306}"
DB_USER="${DB_USER:-root}"
DB_NAME="${DB_NAME:-project_alpha}"
DB_NAME_TEST="${DB_NAME_TEST:-project_alpha_test}"

echo "[entrypoint] waiting for mysql at ${DB_HOST}:${DB_PORT}..."
i=0
until python -c "
import os, sys, pymysql
try:
    pymysql.connect(
        host=os.environ.get('DB_HOST', 'mysql'),
        user=os.environ['DB_USER'],
        password=os.environ['DB_PASSWORD'],
        port=int(os.environ.get('DB_PORT', '3306')),
        connect_timeout=2,
    ).close()
    sys.exit(0)
except Exception as e:
    sys.exit(1)
" 2>/dev/null; do
    i=$((i + 1))
    if [ "$i" -ge 60 ]; then
        echo "[entrypoint] mysql not ready after 60 attempts; aborting" >&2
        exit 1
    fi
    sleep 1
done
echo "[entrypoint] mysql is up"

# Ensure both business and test databases exist (idempotent)
echo "[entrypoint] ensuring databases exist..."
python -c "
import os, pymysql
conn = pymysql.connect(
    host=os.environ.get('DB_HOST', 'mysql'),
    user=os.environ['DB_USER'],
    password=os.environ['DB_PASSWORD'],
    port=int(os.environ.get('DB_PORT', '3306')),
)
with conn.cursor() as cur:
    for db in [os.environ.get('DB_NAME', 'project_alpha'),
               os.environ.get('DB_NAME_TEST', 'project_alpha_test')]:
        cur.execute(f'CREATE DATABASE IF NOT EXISTS \`{db}\` DEFAULT CHARACTER SET utf8mb4 DEFAULT COLLATE utf8mb4_unicode_ci')
conn.commit()
conn.close()
"

echo "[entrypoint] running alembic upgrade head..."
alembic upgrade head

echo "[entrypoint] starting uvicorn..."
exec uvicorn app.main:app --host 0.0.0.0 --port 8000

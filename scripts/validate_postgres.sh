#!/usr/bin/env bash
set -euo pipefail

# Usage:
#   CHACRAB_POSTGRES_TEST_URL=postgres://user:pass@localhost:5432/chacrab_test \
#   CHACRAB_TEST_MASTER_PASSWORD=testpass123 \
#   ./scripts/validate_postgres.sh
#
# Optional:
#   CHACRAB_BIN=./target/debug/chacrab ./scripts/validate_postgres.sh

DATABASE_URL="${CHACRAB_POSTGRES_TEST_URL:-}"
MASTER_PASSWORD="${CHACRAB_TEST_MASTER_PASSWORD:-testpass123}"
CHACRAB_BIN="${CHACRAB_BIN:-cargo run --quiet --}"

if [[ -z "${DATABASE_URL}" ]]; then
  echo "❌ CHACRAB_POSTGRES_TEST_URL is required"
  echo "   Example: export CHACRAB_POSTGRES_TEST_URL='postgres://user:pass@localhost:5432/chacrab_test'"
  exit 1
fi

if [[ "${DATABASE_URL}" != postgres://* && "${DATABASE_URL}" != postgresql://* ]]; then
  echo "❌ CHACRAB_POSTGRES_TEST_URL must start with postgres:// or postgresql://"
  exit 1
fi

if command -v sha256sum >/dev/null 2>&1; then
  ns_hash="$(printf "%s" "${DATABASE_URL}" | sha256sum | awk '{print $1}' | cut -c1-12)"
else
  ns_hash="$(printf "%s" "${DATABASE_URL}" | cksum | awk '{print $1}')"
fi

export CHACRAB_TEST_MODE=1
export CHACRAB_MASTER_PASSWORD="${MASTER_PASSWORD}"
export CHACRAB_CURRENT_PASSWORD="${MASTER_PASSWORD}"
export CHACRAB_NEW_PASSWORD="${MASTER_PASSWORD}"
export CHACRAB_IMPORT_DUPLICATE=skip
export CHACRAB_ALLOW_PLAINTEXT_EXPORT=true
export CHACRAB_KEYRING_SERVICE="chacrab-pg-validate-${ns_hash}"
export CHACRAB_KEYRING_USERNAME="validator"

label_suffix="$(date +%s)"
label="PgValidation${label_suffix}"
export_file="$(mktemp /tmp/chacrab-pg-export-XXXXXX.json)"

cleanup() {
  rm -f "${export_file}" || true
}
trap cleanup EXIT

run_cmd() {
  # shellcheck disable=SC2086
  ${CHACRAB_BIN} --database "${DATABASE_URL}" "$@"
}

echo "🔎 ChaCrab PostgreSQL validation"
echo "   Database: ${DATABASE_URL}"
echo "   Label: ${label}"

echo "\n[1/10] init (idempotent)"
set +e
init_output="$(run_cmd init 2>&1)"
init_status=$?
set -e
if [[ ${init_status} -ne 0 ]]; then
  if grep -qi "already initialized" <<<"${init_output}"; then
    echo "   ↪ Vault already initialized, continuing"
  else
    echo "❌ init failed"
    echo "${init_output}"
    exit 1
  fi
fi

echo "[2/10] login"
run_cmd login >/dev/null

echo "[3/10] add"
run_cmd add --label "${label}" --username pg_user --password pg_pass_123 --url https://postgres.example >/dev/null

echo "[4/10] list verify"
list_output="$(run_cmd list)"
grep -q "${label}" <<<"${list_output}" || { echo "❌ label not found in list output"; exit 1; }

echo "[5/10] update"
run_cmd update --label "${label}" --password pg_pass_456 >/dev/null

echo "[6/10] export encrypted"
run_cmd export --output "${export_file}" --format encrypted >/dev/null
[[ -s "${export_file}" ]] || { echo "❌ export file missing/empty"; exit 1; }

echo "[7/10] delete"
run_cmd delete --label "${label}" >/dev/null

echo "[8/10] import"
run_cmd import --input "${export_file}" >/dev/null

echo "[9/10] list verify re-import"
list_output="$(run_cmd list)"
grep -q "${label}" <<<"${list_output}" || { echo "❌ label not found after import"; exit 1; }

echo "[10/10] logout"
run_cmd logout >/dev/null

echo "✅ PostgreSQL validation PASSED"

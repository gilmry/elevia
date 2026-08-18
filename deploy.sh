#!/usr/bin/env bash
# Sur le serveur cible (Debian/Ubuntu), une fois : ./deploy.sh
#   -> installe docker, docker compose, git, cron, crée .env si absent,
#      programme un cron qui appelle ce même script avec --run.
# Le cron appelle ensuite : ./deploy.sh --run
#   -> pull + redeploy prod si origin/main a bougé, sinon ne fait rien.
# Idempotent : les deux modes peuvent être relancés sans risque.
set -euo pipefail

REPO_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CRON_SCHEDULE="${CRON_SCHEDULE:-*/5 * * * *}"
CRON_MARKER="elevia-auto-deploy"
LOG_FILE="$REPO_DIR/deploy.log"
LOCK_FILE="$REPO_DIR/.deploy.lock"

log() { echo "[$(date -u +%Y-%m-%dT%H:%M:%SZ)] $*" >> "$LOG_FILE"; }

run_deploy() {
  cd "$REPO_DIR"
  exec 9>"$LOCK_FILE"
  flock -n 9 || exit 0

  git fetch origin main --quiet

  local_rev="$(git rev-parse main)"
  remote_rev="$(git rev-parse origin/main)"
  running="$(docker compose --profile prod ps --status running -q 2>/dev/null)"

  # Rien à faire seulement si main n'a pas bougé ET que prod tourne déjà -
  # sinon (premier run après clone, ou prod arrêtée manuellement) on déploie
  # même sans nouveau commit, pour que l'état voulu (prod up) soit garanti.
  if [ "$local_rev" = "$remote_rev" ] && [ -n "$running" ]; then
    exit 0
  fi

  if [ "$local_rev" = "$remote_rev" ]; then
    log "prod non démarrée, déploiement initial ($remote_rev)"
  else
    log "nouveau commit sur main ($local_rev -> $remote_rev), déploiement"
  fi

  if ! git checkout main --quiet || ! git merge --ff-only origin/main --quiet; then
    log "échec du fast-forward vers origin/main, déploiement annulé"
    exit 1
  fi

  if docker compose --profile prod up -d --build >> "$LOG_FILE" 2>&1; then
    log "déploiement réussi ($remote_rev)"
    docker image prune -f >> "$LOG_FILE" 2>&1
  else
    log "échec du déploiement ($remote_rev), voir logs ci-dessus"
    exit 1
  fi
}

bootstrap() {
  if [ "$(id -u)" -ne 0 ] && ! command -v sudo >/dev/null 2>&1; then
    echo "root ou sudo requis pour installer les paquets système" >&2
    exit 1
  fi
  local sudo=""
  [ "$(id -u)" -ne 0 ] && sudo="sudo"

  if ! command -v apt-get >/dev/null 2>&1; then
    echo "ce script suppose une distribution basée sur apt (Debian/Ubuntu)" >&2
    exit 1
  fi

  echo "==> installation des dépendances système"
  $sudo apt-get update -qq
  $sudo apt-get install -y -qq ca-certificates curl git cron >/dev/null

  if ! command -v docker >/dev/null 2>&1; then
    echo "==> installation de Docker (script officiel get.docker.com)"
    curl -fsSL https://get.docker.com | $sudo sh
  fi

  if ! docker compose version >/dev/null 2>&1; then
    echo "docker compose (plugin v2) introuvable après installation de Docker" >&2
    exit 1
  fi

  if [ ! -f "$REPO_DIR/.env" ]; then
    echo "==> création de .env depuis .env.example (à éditer avant le premier déploiement)"
    cp "$REPO_DIR/.env.example" "$REPO_DIR/.env"
  fi

  touch "$LOG_FILE"

  echo "==> programmation du déploiement auto (cron, toutes les $CRON_SCHEDULE)"
  local cron_line="$CRON_SCHEDULE $REPO_DIR/deploy.sh --run # $CRON_MARKER"
  local existing_crontab
  existing_crontab="$(crontab -l 2>/dev/null || true)"
  local new_crontab
  if echo "$existing_crontab" | grep -qF "$CRON_MARKER"; then
    new_crontab="$(echo "$existing_crontab" | grep -vF "$CRON_MARKER")"
  else
    new_crontab="$existing_crontab"
  fi
  { echo "$new_crontab"; echo "$cron_line"; } | grep -v '^$' | crontab -

  $sudo systemctl enable --now cron >/dev/null 2>&1 || true

  cat <<EOF

Bootstrap terminé.
- Dépendances installées : docker, docker compose plugin, git, cron
- Cron installé : $cron_line
- Logs de déploiement : $LOG_FILE

Avant le premier déploiement, éditer $REPO_DIR/.env (DOMAIN, API_DOMAIN,
ACME_EMAIL, JWT_SECRET, mots de passe Postgres, ADMIN_EMAIL/PASSWORD), puis
soit attendre le prochain tick cron, soit lancer manuellement :
  $REPO_DIR/deploy.sh --run
EOF
}

if [ "${1:-}" = "--run" ]; then
  run_deploy
else
  bootstrap
fi

#!/usr/bin/env bash
# Montée en charge par paliers de concurrence croissante contre un endpoint,
# pour trouver le point de rupture (latence qui explose ou erreurs qui
# apparaissent). Utilise `hey` si disponible (recommandé, bien plus fiable),
# sinon retombe sur une boucle curl en parallèle (ordre de grandeur seulement).
#
# Usage :
#   ./test-montee-charge.sh <url> [paliers="5 10 20 50 100"] [requetes_par_palier=50]
#
# ATTENTION : ne jamais lancer ce script contre un environnement de
# production réel sans prévenir le propriétaire du service et sans avoir
# choisi une fenêtre de trafic creuse. Un test de charge non maîtrisé peut
# rendre le service indisponible pour de vrais utilisateurs.
set -euo pipefail

URL="${1:?usage: $0 <url> [paliers] [requetes_par_palier]}"
PALIERS="${2:-5 10 20 50 100}"
REQ_PER_PALIER="${3:-50}"

echo "Cible : $URL"
echo "Paliers de concurrence testés : $PALIERS"
echo

if command -v hey >/dev/null 2>&1; then
  for c in $PALIERS; do
    echo "=== Palier : $c requêtes simultanées ==="
    hey -n "$REQ_PER_PALIER" -c "$c" "$URL" | grep -E "Success rate|Requests/sec|Total:|50%|95%|99%|\[.*\]\s+responses"
    echo
  done
  echo "Lis les taux d'erreur et la latence p95/p99 palier par palier : le"
  echo "point de rupture est le premier palier où le taux d'erreur augmente"
  echo "nettement ou où la latence p95 décroche brutalement de la tendance."
  exit 0
fi

echo "hey non trouvé, utilisation d'une boucle curl parallèle (moins précis,"
echo "donne un ordre de grandeur, pas des percentiles de latence)."
echo

for c in $PALIERS; do
  echo "=== Palier : $c requêtes simultanées ==="
  start=$(date +%s.%N)
  success=0
  fail=0
  for _ in $(seq 1 "$c"); do
    (
      code="$(curl -s -o /dev/null -w '%{http_code}' --max-time 10 "$URL" || echo "000")"
      echo "$code"
    ) &
  done | while read -r code; do
    if [ "$code" -ge 200 ] && [ "$code" -lt 300 ]; then
      success=$((success + 1))
    else
      fail=$((fail + 1))
    fi
  done
  wait
  end=$(date +%s.%N)
  echo "Durée du palier : $(awk "BEGIN{printf \"%.2f\", $end-$start}")s"
  echo
done

echo "Pour des percentiles de latence fiables (p50/p95/p99) et un taux"
echo "d'erreur précis, installe 'hey' (go install github.com/rakyll/hey@latest)"
echo "ou 'wrk'/'ab' et relance ce script."

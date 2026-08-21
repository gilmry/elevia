#!/usr/bin/env bash
# Mesure le temps CPU réel consommé par un conteneur Docker pour traiter N
# requêtes séquentielles, via le delta de usage_usec dans cpu.stat (cgroup v2)
# avant/après. Soustrait le bruit de fond mesuré sur une fenêtre calme.
#
# Usage :
#   ./mesure-cpu-cgroup.sh <nom_conteneur> <url_endpoint> [N=100] [duree_bruit_fond_s=20]
#
# Exemple :
#   ./mesure-cpu-cgroup.sh mon-backend https://api.example.org/health 100 20
#
# Prérequis : cgroup v2, curl, jq facultatif (non utilisé ici, awk suffit).
# Le script doit tourner sur l'hôte qui fait tourner le conteneur (pas dedans).
set -euo pipefail

CONTAINER="${1:?usage: $0 <conteneur> <url> [N] [duree_bruit_fond_s]}"
URL="${2:?usage: $0 <conteneur> <url> [N] [duree_bruit_fond_s]}"
N="${3:-100}"
IDLE_WINDOW="${4:-20}"

cgroup_path() {
  local id
  id="$(docker inspect --format '{{.Id}}' "$CONTAINER")"
  # Cherche le cpu.stat correspondant, quel que soit le driver cgroup
  # (systemd ou cgroupfs) et la profondeur d'imbrication.
  local found
  found="$(find /sys/fs/cgroup -name cpu.stat -path "*${id}*" 2>/dev/null | head -1)"
  if [ -z "$found" ]; then
    echo "Impossible de localiser le cgroup du conteneur $CONTAINER (id $id)." >&2
    echo "Vérifie que cgroup v2 est actif (cat /sys/fs/cgroup/cgroup.controllers)." >&2
    exit 1
  fi
  echo "$found"
}

usage_usec() {
  awk '/^usage_usec/ {print $2}' "$1"
}

CPU_STAT="$(cgroup_path)"
echo "Cgroup : $CPU_STAT"

echo "--- Mesure du bruit de fond (${IDLE_WINDOW}s sans trafic généré par ce script) ---"
before_idle="$(usage_usec "$CPU_STAT")"
sleep "$IDLE_WINDOW"
after_idle="$(usage_usec "$CPU_STAT")"
idle_usec_per_sec=$(( (after_idle - before_idle) / IDLE_WINDOW ))
echo "Bruit de fond : ${idle_usec_per_sec} usec/s"

echo "--- Salve de $N requêtes séquentielles vers $URL ---"
before="$(usage_usec "$CPU_STAT")"
start_ts=$(date +%s)
for _ in $(seq 1 "$N"); do
  curl -s -o /dev/null "$URL"
done
end_ts=$(date +%s)
after="$(usage_usec "$CPU_STAT")"

elapsed=$(( end_ts - start_ts ))
[ "$elapsed" -lt 1 ] && elapsed=1
delta_usec=$(( after - before ))
idle_contribution=$(( idle_usec_per_sec * elapsed ))
net_usec=$(( delta_usec - idle_contribution ))
[ "$net_usec" -lt 0 ] && net_usec=0
per_request_usec=$(( net_usec / N ))

cat <<EOF

=== Résultat ===
Conteneur          : $CONTAINER
Endpoint            : $URL
Requêtes            : $N
Durée de la salve   : ${elapsed}s
Delta CPU brut       : ${delta_usec} usec
Contribution bruit de fond estimée : ${idle_contribution} usec
Delta CPU net (hors bruit de fond) : ${net_usec} usec
Coût CPU moyen par requête (net)   : ${per_request_usec} usec (~$(awk "BEGIN{printf \"%.3f\", $per_request_usec/1000}") ms)

Rappel : ce chiffre n'inclut que ce conteneur. Additionne-le à celui des
autres conteneurs traversés (proxy, DB...) pour le coût total de la requête.
Refais cette mesure une deuxième fois à un autre moment pour valider (viser
moins de ~10% d'écart entre les deux passes).
EOF

---
name: empreinte-serveur
description: This skill should be used when the user asks to "mesure l'empreinte serveur", "calcule le CO2 par requête", "combien coûte cette stack en CPU/RAM", "fais un audit de charge", "trouve le point de rupture", "quelle est l'empreinte carbone de l'appli", "mesure le vrai coût d'une requête", "combien d'utilisateurs simultanés supporte le serveur", "audit CPU/RAM/réseau/CO2", "compare notre stack à Node/Python/React en empreinte", wants an honest, measured (not guessed) footprint report for a web application running in Docker containers on a Linux server, or wants to back up a "stack sobre" / "green IT" claim with real numbers before writing a report, a blog post, or a client-facing case study.
version: 1.0.0
license: CC-BY-SA-4.0
source: https://github.com/gilmry/elevia (méthodologie appliquée en premier lors de l'audit CO2/charge d'Elevia, août 2026)
---

# Empreinte serveur — mesure honnête (pas estimée)

Skill pour produire un rapport chiffré et vérifiable du coût réel (CPU, RAM, réseau, CO2, capacité) d'une application web en production, plutôt qu'une affirmation non prouvée de type "notre stack est sobre". Dérivé de l'audit réalisé sur Elevia (Rust/Actix + Astro/Svelte + PostgreSQL, OVH 1 vCPU/2 Go) et publié comme cas client sur ecosolva.com.

## Règle absolue — mesuré vs estimé, jamais confondu

Chaque chiffre du rapport final doit être étiqueté sans ambiguïté :

- **Mesuré** : obtenu directement sur la machine cible (cgroup, `docker stats`, `curl`, test de charge réel). C'est la majorité du rapport.
- **Estimé / littérature** : dérivé de coefficients publics (Green Software Foundation SCI, Cloud Carbon Footprint) ou de benchmarks tiers (TechEmpower) quand une mesure directe est impossible (pas de wattmètre physique, pas de reconstruction de la même appli dans une autre stack).

Ne jamais présenter un chiffre de la seconde catégorie comme s'il appartenait à la première. La section "Limites de la mesure" en fin de rapport est **obligatoire**, pas optionnelle.

## Prérequis

- Accès SSH (ou exec local) à l'hôte qui fait tourner les conteneurs Docker de l'application.
- cgroup v2 disponible (`cat /sys/fs/cgroup/cgroup.controllers` doit lister `cpu`).
- `curl`, `docker`, et si possible un outil de charge (`hey`, `wrk`, `ab` — sinon une boucle bash en parallèle suffit pour un premier ordre de grandeur).
- L'autorisation du propriétaire du serveur avant tout test de montée en charge sur un environnement réellement en production (voir anti-patterns).

## Méthodologie, étape par étape

### 1. Cartographie

`docker compose ps` (ou `docker ps`) pour lister les conteneurs impliqués dans une requête typique : reverse proxy (Traefik/nginx), backend applicatif, base de données, éventuellement cache. Noter les noms exacts — ils serviront à toutes les mesures suivantes.

### 2. Coût CPU réel par requête (le cœur de la méthode)

Utiliser `scripts/mesure-cpu-cgroup.sh` (voir plus bas) qui automatise ce protocole :

1. Pour chaque conteneur concerné, lire `usage_usec` dans `cpu.stat` du cgroup du conteneur **avant** la salve de requêtes.
2. Envoyer N requêtes **séquentielles** (100 est un bon compromis signal/durée) contre l'endpoint cible via `curl`.
3. Relire `usage_usec` **après**. Le delta / N = temps CPU moyen par requête pour ce conteneur.
4. Additionner les deltas de tous les conteneurs traversés par la requête (proxy + backend + DB) = coût CPU total de la requête.
5. Mesurer un **bruit de fond** : le même delta sur une fenêtre calme sans trafic (20-60s), pour soustraire l'activité de fond (autovacuum Postgres, polling du proxy, etc.) plutôt que de l'attribuer à tort à la requête.
6. Refaire une **deuxième passe indépendante** à un autre moment. Un écart de plus de ~10% entre les deux passes signale une mesure à refaire, pas une moyenne à accepter telle quelle.

Piège classique à documenter, pas à ignorer : chaque `curl` sans `--keepalive` ouvre une connexion TLS neuve, et le handshake TLS domine très souvent le coût CPU total d'une requête simple — bien plus que le code applicatif. Séparer explicitement "coût TLS" et "coût applicatif pur" dans le rapport, et préciser qu'un utilisateur réel en session persistante ne paie ce coût qu'une fois.

Tester au minimum : une lecture simple (health check), une lecture avec accès base de données, une page statique, et — si l'appli en a une — une opération volontairement coûteuse comme un hash de mot de passe (bcrypt/argon2) à la connexion. Ce dernier cas révèle souvent le vrai goulot d'étranglement, très différent du reste.

### 3. Capacité — point de rupture en charge

Montée en charge progressive (paliers de concurrence croissants : 5, 10, 20, 50, 100...) contre chaque type d'endpoint testé à l'étape 2, en observant la latence P95 et le taux d'erreur. Le point de rupture = palier où la latence explose ou où des erreurs apparaissent. Tester séparément un endpoint de lecture et un endpoint d'authentification (hash de mot de passe) : ils ont typiquement des points de rupture qui diffèrent d'un facteur 10 ou plus, et c'est souvent l'authentification, pas la lecture, qui fixe le vrai plafond de la machine.

Sur un environnement réellement en production, prévenir avant de tester et privilégier une fenêtre creuse (voir anti-patterns).

### 4. Empreinte mémoire serveur

`docker stats --no-stream` sur une fenêtre calme, pour chaque conteneur. Chiffre direct, pas de calcul nécessaire. Comparer avec des ordres de grandeur publics pour d'autres stacks (ex : un service Node.js équivalent tourne typiquement à 40-100 Mo au repos, Python/Django à 60-150 Mo par worker) — en étiquetant clairement que ces chiffres de comparaison ne sont pas mesurés sur la même application.

### 5. Payload réseau

Sur les fichiers réellement construits/servis : `curl -sI --compressed <url> | grep -i "content-length\|content-encoding"` pour chaque asset clé (page HTML, bundle JS principal, plus grosses dépendances tierces). Noter la taille brute et compressée. Vérifier que `content-encoding: gzip` (ou `br`) apparaît bien — l'absence de compression est souvent la première correction rapide et à fort impact trouvée pendant l'audit. Comparer avec des tailles publiques documentées pour d'autres frameworks (runtime React/Vue vs Svelte compilé, etc.), toujours étiqueté comme littérature.

### 6. Empreinte mémoire navigateur (optionnel)

Via Chrome DevTools Protocol (`Performance.enable` + `Performance.getMetrics`, lire `JSHeapUsedSize`) ou manuellement dans l'onglet Performance/Memory des DevTools, à différentes étapes du parcours utilisateur (premier chargement, après connexion, page la plus lourde).

### 7. Conversion en CO2

Formule :

```
Énergie (kWh) = temps_CPU_total (h) × puissance_par_vCPU (kW) × PUE
CO2 (g)        = Énergie (kWh) × intensité_carbone_du_réseau_électrique (gCO2eq/kWh)
```

Coefficients par défaut si aucune donnée plus précise n'est disponible (Green Software Foundation SCI, Cloud Carbon Footprint) : ~5 W par vCPU à pleine charge, PUE ~1.2 pour un datacenter moderne.

**Point critique, à ne jamais sauter** : aller chercher l'intensité carbone réelle du réseau électrique du pays d'hébergement — pas réutiliser un chiffre d'un autre pays. Sources fiables : RTE éCO2mix pour la France, electricitymaps.com pour les autres pays. L'écart entre un réseau très décarboné (France, ~50 gCO2eq/kWh grâce au nucléaire) et un réseau charbon-dépendant peut atteindre un facteur 10, ce qui change complètement la conclusion.

Toujours préciser dans le rapport : "pas de wattmètre physique sur cette machine, conversion basée sur des coefficients publiés, pas une mesure directe d'énergie."

### 8. Comparaison avec d'autres stacks (optionnel)

Si la même application n'a pas été reconstruite dans une autre stack pour comparaison directe, s'appuyer sur des benchmarks tiers reconnus (TechEmpower Framework Benchmarks) pour situer un ordre de grandeur relatif (ex : Rust vs Node.js vs Python sur du JSON/DB équivalent). Présenter ces ratios comme littérature, appliqués au temps CPU applicatif *mesuré* sur la stack étudiée — jamais comme une mesure de la stack comparée.

### 9. Coût de la machine

Relever le prix catalogue réel du fournisseur cloud utilisé (avec date de consultation), et recommander explicitement de le confirmer sur la facture réelle plutôt que de le citer comme définitif.

## Format du rapport de sortie

Un rapport suit toujours cette structure (voir `references/gabarit-rapport.md` pour un exemple complet rempli avec les chiffres Elevia) :

1. Méthode (N requêtes, deltas cgroup, bruit de fond soustrait, nombre de passes)
2. Ce qui coûte cher en CPU (tableau requête / temps CPU / ce qui domine)
3. Capacité — combien d'utilisateurs simultanés, point de rupture par type d'endpoint
4. Empreinte mémoire serveur (tableau par conteneur)
5. Payload réseau (tableau par asset, brut vs compressé)
6. Empreinte mémoire navigateur (si mesurée)
7. Conversion CO2 (formule + coefficients + résultat par requête)
8. Comparaison avec d'autres stacks (si applicable, clairement étiqueté littérature)
9. Coût de la machine
10. Conclusion : quel est le vrai levier (généralement le multiplicateur d'architecture à volume, pas le CO2 par requête isolée, qui est presque toujours négligeable en absolu)
11. **Limites de la mesure** (section obligatoire, honnête sur tout ce qui n'a pas pu être mesuré directement)

## Anti-patterns à refuser

- Présenter un coefficient CO2 estimé (SCI, Cloud Carbon Footprint) comme une mesure directe d'énergie.
- Réutiliser l'intensité carbone d'un pays pour un serveur hébergé dans un autre pays.
- Lancer un test de montée en charge jusqu'au point de rupture sur un environnement de production réel sans prévenir le propriétaire du service, ou sans fenêtre creuse — un test de charge non maîtrisé peut littéralement mettre le service hors service pour de vrais utilisateurs.
- Faire une seule passe de mesure et l'accepter sans deuxième passe de contrôle.
- Ignorer le coût TLS/handshake dans une mesure au `curl` sans le signaler — ça peut faire passer un coût réseau/protocolaire pour un coût applicatif.
- Publier un chiffre de comparaison avec une autre stack (Node/Python/React...) sans préciser clairement qu'il vient de la littérature et pas d'une reconstruction mesurée.
- Omettre la section "Limites de la mesure" du rapport final.

## Ressources

- `scripts/mesure-cpu-cgroup.sh` — script bash prêt à l'emploi : mesure le delta `cpu.stat` d'un conteneur avant/après une salve de N requêtes `curl` sur un endpoint donné, avec soustraction du bruit de fond.
- `scripts/test-montee-charge.sh` — script bash de montée en charge par paliers de concurrence croissante, avec mesure de latence et taux d'erreur par palier.
- `references/gabarit-rapport.md` — gabarit de rapport complet, rempli avec l'exemple réel Elevia (structure à réutiliser telle quelle).
- Référence d'exécution d'origine : audit CO2/charge Elevia, août 2026, cas client publié sur ecosolva.com/fr/blog/cas-elevia-delegation-cout-reel.

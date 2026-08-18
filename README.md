# Elevia

Outil de suivi des coûts et de la production pour une coopérative agricole
(exploitations membres, intrants, coûts mensuels, production, dashboards
individuels et agrégés coopérative). Projet de démonstration / dogfooding,
pensé open-source et réutilisable par d'autres coopératives.

Stack : Rust/Actix-web (architecture hexagonale) + PostgreSQL côté backend,
Svelte + Astro (PWA installable, offline-first) côté frontend, orchestrés
en local via Docker Compose.

## Statut

🚧 Scaffold en cours. Ce commit pose la structure de dossiers, le
docker-compose.yml et un squelette minimal buildable (endpoint `/health`
côté backend, page d'accueil + manifest PWA côté frontend). Le modèle de
données, les handlers métier et les tests BDD d'isolation arrivent dans un
second temps.

## Structure du projet

```
elevia/
├── docker-compose.yml       # orchestration (traefik, postgres, backend, frontend)
├── .env.example              # variables d'environnement partagées
├── backend/                   # Rust / Actix-web, architecture hexagonale
│   ├── src/
│   │   ├── domain/            # entités et logique métier pures
│   │   ├── application/       # ports (traits repository) + use cases + DTOs
│   │   └── infrastructure/    # implémentations SQLx, handlers HTTP, routes
│   ├── migrations/            # migrations SQL (SQLx)
│   └── tests/                 # tests BDD (cucumber-rs), dont le test d'isolation
└── frontend/                   # Svelte + Astro, PWA offline-first
    ├── src/
    │   ├── pages/              # écrans (saisie coûts, production, dashboards, admin)
    │   ├── components/         # composants Svelte
    │   └── lib/                # client API, gestion de la queue offline
    └── public/                 # manifest PWA, service worker
```

## Démarrer en local (développement)

Le profil par défaut (`docker compose up`) est le profil **production** :
il passe par Traefik et demande un certificat Let's Encrypt pour de vrais
noms de domaine (voir [Déploiement](#déploiement-production)), donc il ne
fonctionne pas tel quel en local sans domaine. Pour développer en local,
utiliser le profil `dev`, qui expose les services directement sans Traefik :

1. Copier le fichier d'environnement :

   ```bash
   cp .env.example .env
   ```

2. Lancer la stack en mode développement, avec hot-reload (cargo-watch côté
   backend, serveur de dev Astro côté frontend) :

   ```bash
   docker compose --profile dev up --build
   ```

   - Backend : http://localhost:8080 (healthcheck : `GET /health`)
   - Frontend : http://localhost:4321

   Les migrations SQLx s'exécutent automatiquement au démarrage du backend
   (`sqlx::migrate!` dans `main.rs`).

## Déploiement production

Le profil par défaut ajoute [Traefik](https://traefik.io/) devant `backend`
et `frontend`, avec TLS automatique via Let's Encrypt (challenge HTTP) pour
deux sous-domaines :

- `DOMAIN` (par défaut `elevia.ecosolva.com`) → frontend (PWA)
- `API_DOMAIN` (par défaut `api.elevia.ecosolva.com`) → backend (API)

Étapes sur le serveur cible :

1. Pointer les enregistrements DNS `DOMAIN` et `API_DOMAIN` (A records) vers
   l'IP du serveur.
2. `cp .env.example .env` puis ajuster `DOMAIN`, `API_DOMAIN`, `ACME_EMAIL`,
   `JWT_SECRET` et les identifiants Postgres.
3. Ouvrir les ports 80 et 443 sur le serveur (le 80 sert au challenge ACME
   et à la redirection vers HTTPS).
4. `docker compose up -d --build`

Traefik demande les certificats au premier démarrage ; les logs
(`docker compose logs traefik`) indiquent si le challenge ACME échoue (DNS
pas encore propagé, port 80 fermé, etc.).

## Démo vidéo

Pour une démo locale complète : `docker compose --profile dev up --build`,
attendre que le healthcheck Postgres passe au vert, puis ouvrir
http://localhost:4321. Le backend expose son état sur
http://localhost:8080/health.

## Licence

Voir [LICENSE](./LICENSE).

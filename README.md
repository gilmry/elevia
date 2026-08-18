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
├── docker-compose.yml       # orchestration locale (postgres, backend, frontend)
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

## Démarrer en local

1. Copier le fichier d'environnement :

   ```bash
   cp .env.example .env
   ```

2. Lancer la stack (build + démarrage) :

   ```bash
   docker compose up --build
   ```

   - Backend : http://localhost:8080 (healthcheck : `GET /health`)
   - Frontend : http://localhost:4321

   Les migrations SQLx s'exécutent automatiquement au démarrage du backend
   (`sqlx::migrate!` dans `main.rs`).

3. Mode développement avec hot-reload (cargo-watch côté backend, serveur de
   dev Astro côté frontend) :

   ```bash
   docker compose --profile dev up --build
   ```

## Démo vidéo

Pour une démo locale complète : `docker compose up --build`, attendre que
le healthcheck Postgres passe au vert, puis ouvrir http://localhost:4321.
Le backend expose son état sur http://localhost:8080/health.

## Licence

Voir [LICENSE](./LICENSE).

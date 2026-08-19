# Elevia

Outil de suivi des coûts et de la production pour une coopérative agricole
(exploitations membres, intrants, coûts mensuels, production, dashboards
individuels et agrégés coopérative). Projet de démonstration / dogfooding,
pensé open-source et réutilisable par d'autres coopératives.

Stack : Rust/Actix-web (architecture hexagonale) + PostgreSQL côté backend,
Svelte + Astro (PWA installable, offline-first) côté frontend, orchestrés
en local via Docker Compose.

## Statut

En production sur https://elevia.ecosolva.org, déployée automatiquement à
chaque merge sur `main` (voir [Déploiement automatique](#déploiement-automatique-gitops)).

- **Backend** : modèle de données, endpoints v1, isolation par exploitation
  (test BDD dédié), changement de mot de passe (self-service + reset admin),
  serveur MCP en lecture seule, serveur OAuth 2.1 + PKCE auto-hébergé pour
  les clients MCP.
- **Frontend** : connexion, saisie coûts/production, dashboards individuels
  et coopérative, backoffice admin (exploitations, produits, reset mot de
  passe), page « Mon compte », queue offline (PWA installable), mentions
  légales.
- **Tests** : suite Playwright de bout en bout (voir [Tests end-to-end](#tests-end-to-end))
  tournant en CI à chaque push sur `main` ; test BDD d'isolation par
  exploitation (`cargo test --test bdd`, nécessite Docker/testcontainers).

## Structure du projet

```
elevia/
├── docker-compose.yml       # orchestration (traefik, postgres, backend, frontend)
├── .env.example              # variables d'environnement partagées
├── backend/                   # Rust / Actix-web, architecture hexagonale
│   ├── src/
│   │   ├── domain/            # entités et logique métier pures
│   │   ├── application/       # ports (traits repository) + use cases + DTOs
│   │   └── infrastructure/    # SQLx, handlers HTTP/routes, serveur MCP (web/mcp.rs)
│   │                          # et serveur OAuth 2.1 + PKCE (web/oauth.rs)
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

`dev` et `prod` sont deux profils Compose exclusifs (seul `postgres` n'a pas
de profil et tourne dans les deux). Le profil `prod` passe par Traefik et
demande un certificat Let's Encrypt pour de vrais noms de domaine (voir
[Déploiement](#déploiement-production)), donc il ne fonctionne pas tel quel
en local sans domaine. Pour développer en local, utiliser le profil `dev`,
qui expose les services directement sans Traefik :

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

### Premier login

Il n'existe aucun mot de passe par défaut codé en dur, et aucune route
publique ne permet de créer le premier compte admin (toutes les routes
`/admin/...` exigent déjà un token admin). Au tout premier démarrage, si
aucun admin n'existe encore en base, le backend en crée un à partir des
variables d'environnement `ADMIN_EMAIL` / `ADMIN_PASSWORD` (voir
`.env.example` - à changer avant tout déploiement réel). Une fois connecté
avec ce compte, créer les comptes des exploitations via le backoffice admin
(`POST /admin/exploitations`) plutôt que de réutiliser ces identifiants.

### Mots de passe

- **Self-service** : chaque compte (admin ou exploitation) change son propre
  mot de passe depuis la page « Mon compte » (ancien mot de passe requis),
  `POST /auth/change-password`.
- **Compte oublié** : un admin réinitialise le mot de passe d'une
  exploitation depuis la liste des exploitations, sans connaître l'ancien -
  `POST /admin/exploitations/{id}/reset-password`.

Minimum 8 caractères des deux côtés (comptés, pas en octets).

## Connecter Claude (MCP)

Le backend expose `POST /mcp`, un serveur [MCP](https://modelcontextprotocol.io/)
(JSON-RPC 2.0, transport "Streamable HTTP") permettant à un admin ou une
exploitation de brancher Claude directement sur son compte Elevia, en tant
que connecteur distant.

Deux façons de s'authentifier :

- **OAuth 2.1 + PKCE (recommandé)** : le backend fait aussi office de
  serveur d'autorisation auto-hébergé (`/.well-known/oauth-authorization-server`,
  `/oauth/register`, `/oauth/authorize`, `/oauth/token`). Claude s'enregistre
  et se reconnecte tout seul, l'utilisateur voit juste un écran de connexion
  (email/mot de passe) puis reste connecté indéfiniment : l'access token
  (1h) se renouvelle automatiquement via le refresh token (30 jours,
  révoqué et remplacé à chaque usage). Aucune configuration manuelle
  requise côté client au-delà de coller l'URL `/mcp`.
- **Bearer JWT direct** : coller le jeton obtenu via `POST /auth/login`
  dans l'en-tête `Authorization`. Plus simple pour un script ou un test,
  mais le jeton expire au bout de 12h sans possibilité de renouvellement -
  à réserver à un usage ponctuel.

Les deux méthodes produisent le même type de jeton (mêmes claims JWT), donc
`/mcp` et le reste de l'API REST ne font aucune différence entre les deux.

Outils disponibles, selon le rôle du compte :

| Outil | Admin | Exploitation |
|---|---|---|
| `list_products` | oui | oui |
| `get_coop_dashboard` | oui | oui |
| `list_my_entries` | - | oui (ses propres coûts) |
| `get_my_dashboard` | - | oui (son propre dashboard) |
| `list_exploitations` | oui | - |

Lecture seule pour l'instant : aucun outil ne modifie de données. La liste
retournée par `tools/list` est déjà filtrée par rôle ; `tools/call`
revérifie la même règle côté serveur (défense en profondeur, comme pour
les endpoints REST équivalents).

## Tests end-to-end

Les parcours utilisateurs (admin comme exploitation) sont couverts par une
suite Playwright dans `frontend/e2e/`, qui sert de documentation vivante :
chaque run pilote un vrai navigateur contre la stack `dev` et produit un
rapport HTML avec la vidéo de chaque parcours. Voir
[frontend/e2e/README.md](./frontend/e2e/README.md) pour la liste des
parcours couverts et comment lancer/consulter les tests.

Cette suite tourne aussi en CI ([`.github/workflows/e2e-living-docs.yml`](./.github/workflows/e2e-living-docs.yml))
à chaque push sur `main`, et le rapport (vidéos incluses) est publié sur
**https://gilmry.github.io/elevia/** : c'est la référence toujours à jour
du comportement réel de l'app, sans avoir à relancer les tests en local.

## Déploiement production

Le profil `prod` ajoute [Traefik](https://traefik.io/) devant `backend`
et `frontend`, avec TLS automatique via Let's Encrypt (challenge HTTP) pour
deux sous-domaines :

- `DOMAIN` (par défaut `elevia.ecosolva.org`) → frontend (PWA)
- `API_DOMAIN` (par défaut `api.elevia.ecosolva.org`) → backend (API)

Étapes sur le serveur cible :

1. Pointer les enregistrements DNS `DOMAIN` et `API_DOMAIN` (A records) vers
   l'IP du serveur.
2. `cp .env.example .env` puis ajuster `DOMAIN`, `API_DOMAIN`, `ACME_EMAIL`,
   `JWT_SECRET`, `PUBLIC_API_URL` (`https://<API_DOMAIN>`) et les identifiants
   Postgres.
3. Ouvrir les ports 80 et 443 sur le serveur (le 80 sert au challenge ACME
   et à la redirection vers HTTPS).
4. `docker compose --profile prod up -d --build`

Traefik demande les certificats au premier démarrage ; les logs
(`docker compose logs traefik`) indiquent si le challenge ACME échoue (DNS
pas encore propagé, port 80 fermé, etc.).

### Déploiement automatique (gitops)

`./deploy.sh` installe les dépendances (docker, docker compose, git, cron),
crée `.env` depuis `.env.example` si absent, et programme un cron (toutes
les 5 min par défaut, réglable via `CRON_SCHEDULE`) qui poll `origin/main` :
tout nouveau commit sur `main` déclenche automatiquement `git pull` +
`docker compose --profile prod up -d --build`. Éditer `.env` avant le
premier déploiement (voir étape 2 ci-dessus). Logs dans `deploy.log`.

## Démo vidéo

Pour une démo locale complète : `docker compose --profile dev up --build`,
attendre que le healthcheck Postgres passe au vert, puis ouvrir
http://localhost:4321. Le backend expose son état sur
http://localhost:8080/health.

## Licence

Voir [LICENSE](./LICENSE).

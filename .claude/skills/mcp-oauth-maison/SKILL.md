---
name: mcp-oauth-maison
description: This skill should be used when the user asks to "crée un serveur MCP avec OAuth", "ajoute OAuth 2.1 PKCE pour un client MCP", "connecte Claude directement à mon appli", "serveur MCP maison", "authorization server fait maison sans provider externe", "dynamic client registration RFC 7591", "PKCE S256", "expose mes données via MCP avec un vrai bouton Connect", "remote MCP server self-hosted", or wants to let an MCP client (Claude Code, Claude Desktop, claude.ai) connect to a self-hosted backend with a real OAuth "Connect" flow instead of a manually pasted, expiring token — reusing the application's existing user/password store rather than an external identity provider (Auth0, Okta, Keycloak...).
version: 1.0.0
license: CC-BY-SA-4.0
source: https://github.com/gilmry/elevia (backend/src/infrastructure/web/oauth.rs, mcp.rs, application/use_cases/oauth_use_cases.rs, migrations/0002_oauth.sql)
---

# Serveur MCP avec OAuth 2.1 + PKCE fait maison

Skill pour exposer les données d'une application (déjà dotée d'une table
utilisateurs email/mot de passe) à un client MCP — Claude Code, Claude
Desktop, claude.ai — via un vrai flow "Connect" OAuth, entièrement
self-hosted, sans fournisseur d'identité externe (pas d'Auth0/Okta/Keycloak).
Dérivé de l'implémentation Rust/Actix-web d'Elevia ; le pattern (architecture,
décisions de sécurité, schéma de données) est indépendant du langage, seul le
code de référence en `references/` est Rust.

## Pourquoi fait maison plutôt qu'un provider externe

Un client MCP qui demande "Connect" attend un vrai flow OAuth 2.1 avec PKCE,
pas un token collé à la main qui expire au bout de quelques heures. Brancher
un provider externe (Auth0, Keycloak...) pour ça ajoute une dépendance, un
compte tiers et souvent un coût récurrent — alors que l'application a déjà
tout ce qu'il faut : une table utilisateurs et une vérification de mot de
passe. Le serveur d'autorisation fait maison **délègue la vérification des
identifiants au code d'authentification existant** (même table, même hash),
et n'émet que la couche OAuth par-dessus.

Les jetons d'accès émis sont le **même JWT** que celui déjà utilisé par
l'API REST classique (juste avec une durée de vie plus courte, rafraîchissable) :
aucune des routes métier existantes n'a besoin d'un traitement spécial pour
accepter un token venu du flow MCP plutôt que d'un `/auth/login` classique.

## Architecture — 5 pièces

### 1. Discovery — `GET /.well-known/oauth-authorization-server`

Document JSON (RFC 8414) qui annonce `issuer`, `authorization_endpoint`,
`token_endpoint`, `registration_endpoint`, `scopes_supported`,
`response_types_supported`, `grant_types_supported`,
`code_challenge_methods_supported: ["S256"]`,
`token_endpoint_auth_methods_supported: ["none"]`. C'est ce document qui
permet à un client MCP de se configurer **automatiquement**, sans qu'un
humain ne remplisse un formulaire d'admin quelque part.

### 2. Enregistrement dynamique de client — `POST /oauth/register` (RFC 7591)

Le client MCP s'enregistre lui-même à la première connexion : il envoie ses
`redirect_uris`, reçoit un `client_id` généré côté serveur. **Clients publics
uniquement** : pas de secret client (`token_endpoint_auth_method: "none"`),
la sécurité repose entièrement sur PKCE. Valider strictement que chaque
`redirect_uri` est une URI absolue en `https://`, ou `http://localhost` pour
le développement local d'un client — jamais autre chose.

### 3. Authorization endpoint — `GET` puis `POST /oauth/authorize`

`GET` : valide `client_id`/`redirect_uri`/`response_type=code`/
`code_challenge_method=S256` **avant** d'afficher quoi que ce soit — c'est la
seule vérification qui doit avoir lieu avant de faire confiance à
`redirect_uri` assez pour y renvoyer une réponse (succès ou erreur). Si elle
échoue, on affiche une erreur générique en page, jamais une redirection vers
un `redirect_uri` non vérifié. Si elle réussit, on affiche un formulaire de
connexion HTML classique (email + mot de passe) avec les paramètres OAuth en
champs cachés.

`POST` : revalide `client_id`/`redirect_uri`/`code_challenge_method` **une
deuxième fois** (les champs cachés du formulaire sont fournis par le client,
pas une source de vérité, même s'ils sont invisibles à l'utilisateur), vérifie
email/mot de passe contre la table utilisateurs existante, puis émet un
**code d'autorisation à usage unique**, court (10 minutes), lié explicitement
à `client_id` + `redirect_uri` + `code_challenge`. Redirige vers
`redirect_uri?code=...&state=...`.

### 4. Token endpoint — `POST /oauth/token`

Deux `grant_type` supportés :

- `authorization_code` : consomme le code (usage unique — une deuxième
  tentative avec le même code doit échouer), vérifie qu'il n'est pas expiré,
  que `client_id`/`redirect_uri` correspondent exactement à ceux de
  l'autorisation, et vérifie PKCE (`SHA256(code_verifier) == code_challenge`,
  encodé en base64url sans padding). Émet alors un access token (JWT existant,
  courte durée, ex. 1h) et un refresh token (secret opaque, longue durée,
  ex. 30 jours).
- `refresh_token` : vérifie le hash du refresh token présenté contre celui
  stocké, vérifie qu'il correspond au bon `client_id` et n'est pas révoqué.
  **Rotation obligatoire et inconditionnelle** : le refresh token présenté
  est révoqué que le reste de l'échange réussisse ou non, pour qu'un token
  volé puis rejoué après qu'un client légitime l'a déjà utilisé pour se
  rafraîchir soit automatiquement bloqué.

### 5. Le serveur MCP lui-même — `POST /mcp`

JSON-RPC 2.0 sur le transport MCP "Streamable HTTP", **stateless** : chaque
requête se ré-authentifie via le même header `Authorization: Bearer <JWT>`
que le reste de l'API — pas de session serveur à protéger séparément.
Méthodes minimales : `initialize` (annonce version de protocole et
capacités), `tools/list` (liste des outils, filtrée selon le rôle de
l'utilisateur authentifié — contrôle la **découvrabilité**, pas
l'autorisation), `tools/call` (exécute un outil, **revérifie le rôle
indépendamment** de ce que `tools/list` a montré). Chaque outil MCP est un
adaptateur fin vers un cas d'usage métier qui existe déjà et qui est déjà
testé par l'API REST — **aucune nouvelle logique métier n'est écrite
spécialement pour MCP**, ce qui évite un chemin d'exécution parallèle non
testé.

## Décisions de sécurité à ne jamais sauter

- PKCE **S256 uniquement**, jamais "plain" — c'est une exigence d'OAuth 2.1, pas une option.
- L'access token est le JWT applicatif existant : jamais stocké côté serveur, juste signé/vérifié à la volée.
- Le refresh token est un secret opaque ; **seul son hash SHA-256 est persisté**, jamais le token en clair — exactement le même principe qu'un hash de mot de passe.
- Le code d'autorisation est à usage unique, court (10 min typiquement), et explicitement lié à `client_id` + `redirect_uri` + `code_challenge` — ces trois valeurs sont revérifiées à l'échange, pas seulement à l'émission.
- `redirect_uri` n'est jamais fait confiance sans validation stricte contre la liste enregistrée pour ce `client_id`, et cette validation a lieu **avant tout rendu ou toute redirection** — sinon le endpoint devient un redirecteur ouvert.
- Rotation inconditionnelle du refresh token à chaque utilisation, même en cas d'échec du reste de l'échange.
- `tools/list` ne doit **jamais** être la seule barrière d'autorisation : `tools/call` revalide indépendamment le rôle/la portée pour chaque appel.
- Le endpoint `/mcp` doit rester strictement en lecture seule (ou séparer clairement lecture/écriture avec des scopes explicites) tant que le client MCP n'a pas de mécanisme de confirmation utilisateur fiable pour les actions destructives.

## Schéma de données minimal (3 tables)

Voir `references/migration-oauth.sql` pour le script complet commenté. Résumé :

- `oauth_clients` (client_id, client_name, redirect_uris[], created_at)
- `oauth_authorization_codes` (code, client_id, user_id, redirect_uri, code_challenge, expires_at, used) — index sur `expires_at` pour le nettoyage périodique des codes expirés
- `oauth_refresh_tokens` (**token_hash** en clé primaire, jamais le token, client_id, user_id, expires_at, revoked) — index sur `user_id`

## Checklist d'implémentation

1. Table(s) OAuth (migration, voir référence).
2. Endpoint discovery `/.well-known/oauth-authorization-server`.
3. Endpoint `POST /oauth/register` (enregistrement dynamique, clients publics uniquement).
4. Endpoint `GET /oauth/authorize` (formulaire de connexion HTML minimal) + `POST /oauth/authorize` (vérification credentials + émission du code).
5. Endpoint `POST /oauth/token` (échange code→tokens avec vérif PKCE, et refresh avec rotation).
6. Réutiliser le mécanisme d'émission de JWT déjà existant pour l'access token — ne pas en inventer un second.
7. Endpoint `POST /mcp` (JSON-RPC : `initialize`, `tools/list`, `tools/call`), qui réutilise l'extracteur d'authentification Bearer déjà existant pour l'API REST.
8. Chaque outil MCP = un appel direct à un cas d'usage métier déjà existant et testé.

## Vérification manuelle du flow complet

```bash
# 1. Discovery
curl -s https://api.example.org/.well-known/oauth-authorization-server | jq

# 2. Enregistrement d'un client de test
curl -s -X POST https://api.example.org/oauth/register \
  -H 'Content-Type: application/json' \
  -d '{"redirect_uris": ["http://localhost:8765/callback"], "client_name": "test"}'
# -> récupérer client_id

# 3. Générer un couple PKCE
CODE_VERIFIER=$(openssl rand -base64 32 | tr -d '=+/')
CODE_CHALLENGE=$(printf '%s' "$CODE_VERIFIER" | openssl dgst -sha256 -binary | openssl base64 | tr '+/' '-_' | tr -d '=')

# 4. Ouvrir dans un navigateur (ou curl -L -d pour simuler le POST du formulaire) :
#    GET /oauth/authorize?client_id=...&redirect_uri=...&response_type=code
#      &state=xyz&code_challenge=$CODE_CHALLENGE&code_challenge_method=S256
# -> se connecter, récupérer le `code` depuis la redirection

# 5. Échanger le code contre des tokens
curl -s -X POST https://api.example.org/oauth/token \
  -d "grant_type=authorization_code&code=$CODE&client_id=$CLIENT_ID" \
  -d "redirect_uri=http://localhost:8765/callback&code_verifier=$CODE_VERIFIER"

# 6. Vérifier qu'un mauvais code_verifier est bien rejeté (PKCE effectif)
# 7. Vérifier qu'un refresh_token déjà utilisé une fois est bien rejeté la deuxième fois (rotation effective)
```

## Anti-patterns à refuser

- Accepter `code_challenge_method=plain` (OAuth 2.1 l'interdit).
- Stocker le refresh token en clair au lieu de son hash.
- Faire confiance à `redirect_uri` avant d'avoir vérifié qu'il appartient bien à la liste enregistrée pour ce `client_id`.
- Réutiliser un code d'autorisation déjà consommé sans le rejeter explicitement.
- Laisser un refresh token valide indéfiniment après avoir servi une fois (pas de rotation).
- Écrire de la logique métier nouvelle et non testée directement dans les handlers `tools/call`, au lieu de réutiliser des cas d'usage existants.
- Faire reposer l'autorisation d'un outil MCP uniquement sur le fait qu'il n'apparaît pas dans `tools/list` pour cet utilisateur.
- Exposer des outils MCP en écriture/destructifs sans réflexion spécifique sur la confirmation utilisateur côté client MCP.

## Ressources

- `references/migration-oauth.sql` — schéma SQL complet et commenté des 3 tables.
- `references/oauth-entities.rs` — entités domaine (`OAuthClient`, `AuthorizationCode`, `RefreshToken`).
- `references/oauth-use-cases.rs` — logique métier complète : enregistrement, validation de la requête d'autorisation, échange de code (PKCE), refresh avec rotation.
- `references/oauth-endpoints.rs` — implémentation de référence des endpoints HTTP discovery/register/authorize/token (Rust/Actix-web, tirée d'Elevia telle quelle), avec les commentaires de sécurité en contexte.
- `references/mcp-endpoint.rs` — implémentation de référence de l'endpoint `/mcp` (JSON-RPC, tools/list filtré par rôle, tools/call qui revalide).
- Référence d'exécution d'origine : Elevia, `backend/src/infrastructure/web/oauth.rs` et `mcp.rs`, `application/use_cases/oauth_use_cases.rs`, `migrations/0002_oauth.sql`.

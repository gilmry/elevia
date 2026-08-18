# Prompt Claude Code — Stack coopérative agricole (Rust + PWA + Docker Compose)

Contexte à copier-coller tel quel dans Claude Code.

---

```
Je veux scaffolder un projet full-stack pour un outil de suivi des coûts 
et de la production destiné à une coopérative agricole (15 exploitations 
au démarrage, potentiellement plus). C'est un projet de démonstration/
dogfooding, pensé pour être open-source et réutilisable par d'autres 
coopératives.

## Stack et architecture

- Backend : Rust, Actix-web, architecture hexagonale (ports & adapters), 
  même conventions que mon projet KoproGo (github.com/gilmry/koprogo) — 
  séparation domain / application / infrastructure, tests BDD (cucumber-rs 
  ou équivalent)
- Base de données : PostgreSQL
- Frontend : PWA (installable, fonctionnement offline avec sync différée), 
  Svelte + Astro si possible pour rester cohérent avec KoproGo, sinon 
  Svelte seul
- Orchestration locale : docker-compose avec un service par brique 
  (postgres, backend, frontend), volumes pour la persistance, variables 
  d'environnement via .env

## Modèle de données (base à créer via migrations SQLx ou diesel)

- exploitations (id, nom, contact, créé_le)
- utilisateurs (id, exploitation_id nullable, email, rôle: admin|exploitation, 
  hash mot de passe)
- products (id, nom, unité, catégorie) -- ex: maïs, soja, provende, 
  vétérinaire, électricité, eau, main d'œuvre
- entries (id, exploitation_id, product_id, mois, quantité, coût, créé_le)
- production (id, exploitation_id, mois, quantité_produite, unité)

## Endpoints à implémenter (v1, minimal viable)

Auth :
- POST /auth/login (email + mot de passe, retourne un JWT)

Côté exploitation (authentification requise, scope = sa propre 
exploitation uniquement) :
- POST /exploitations/{id}/entries — soumettre les coûts du mois
- GET /exploitations/{id}/entries — historique de sa propre exploitation
- POST /exploitations/{id}/production — déclarer la quantité produite du mois
- GET /exploitations/{id}/dashboard — ses propres stats : coût de revient 
  par unité, évolution mensuelle, marge estimée

Côté admin (backoffice, rôle admin requis) :
- POST /admin/exploitations — créer une exploitation + son compte utilisateur
- GET /admin/exploitations — liste des exploitations avec statut de saisie 
  du mois en cours (qui a soumis, qui n'a pas encore)
- POST /admin/products — créer/gérer les intrants et postes de coûts trackés
- PUT /admin/products/{id} — modifier un produit

Partagé (accessible aux exploitations ET aux admins, données agrégées 
uniquement, jamais le détail d'une autre exploitation) :
- GET /coop/dashboard — besoins totaux en intrants (pour les négociations 
  groupées), marge moyenne de la coopérative, écarts anonymisés entre 
  exploitations (quartiles plutôt que valeurs nominatives)

## Contrainte de sécurité non négociable

L'isolation des données entre exploitations doit être imposée au niveau 
du backend (vérification exploitation_id du token JWT vs exploitation_id 
de la ressource demandée sur CHAQUE endpoint concerné), pas seulement 
côté frontend. Écris un test BDD spécifique qui vérifie qu'une 
exploitation A ne peut jamais lire les entries ou le dashboard détaillé 
d'une exploitation B, même en modifiant l'ID dans l'URL.

## PWA — spécificités

- Manifest + service worker pour installation depuis le navigateur mobile
- Mode offline : la saisie des coûts doit fonctionner sans réseau, avec 
  synchronisation automatique dès que la connexion revient (queue locale 
  IndexedDB, retry en arrière-plan)
- Interface simple, pensée pour un usage au champ sur smartphone (gros 
  boutons, formulaire de saisie en 2-3 champs max par écran)

## Docker Compose

Fournis un docker-compose.yml avec :
- service postgres (image officielle, volume nommé pour la persistance, 
  variables d'env pour user/password/db)
- service backend (build depuis le Dockerfile Rust, dépend de postgres 
  en healthcheck, variable DATABASE_URL)
- service frontend (build depuis Dockerfile, sert le build de prod de 
  la PWA, ou mode dev avec hot-reload si je précise un profil "dev")
- Un README avec les commandes pour lancer en local (docker-compose up), 
  et comment lancer les migrations au premier démarrage

## Ce que je veux en sortie

1. Structure de dossiers du projet (backend/ et frontend/ séparés)
2. Le docker-compose.yml commenté
3. Le schéma de migration SQL initial
4. Les handlers Actix-web pour les endpoints listés ci-dessus, avec la 
   vérification d'isolation des données
5. Au moins un test BDD (feature Gherkin) qui couvre le scénario 
   d'isolation décrit plus haut
6. Un README expliquant comment démarrer le projet en local pour une 
   démo vidéo

Commence par me proposer la structure de dossiers et le docker-compose.yml, 
je validerai avant qu'on attaque le code des handlers.
```

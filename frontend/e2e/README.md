# Tests end-to-end - documentation vivante

Ces specs Playwright ne sont pas qu'une suite de non-régression : elles
**sont** la documentation des parcours utilisateurs d'Elevia. Une doc écrite
à la main décrit ce que l'app *devrait* faire et pourrit avec le temps ;
ici, chaque exécution pilote un vrai navigateur contre la vraie stack
(Postgres + backend Actix + frontend Astro/Svelte) et produit un rapport
HTML avec le détail étape par étape et la vidéo du parcours - donc toujours
synchronisé avec le comportement réel de l'app.

## Lancer les tests

Prérequis : la stack tourne en local (`docker compose --profile dev up
--build`, voir le README principal).

```bash
cd frontend
npm install
npm run test:e2e:install   # une fois, télécharge Chromium pour Playwright
npm run test:e2e
```

## Consulter la documentation vivante

```bash
npm run test:e2e:report
```

Ouvre le rapport HTML : chaque test y apparaît décomposé en étapes nommées
(`test.step`), avec captures d'écran et, pour chaque test, **la vidéo
complète du parcours**. C'est la référence à jour de "comment se comporte
l'app" - à consulter avant de faire confiance à une description écrite du
comportement.

`playwright-report/` et `test-results/` sont regénérés à chaque run et
ignorés par git (voir `.gitignore`) : ne pas les committer, juste relancer
`npm run test:e2e` pour les reproduire.

Autres commandes utiles :

- `npm run test:e2e:headed` - voir le navigateur en direct pendant le run
- `npm run test:e2e:ui` - mode interactif Playwright (time-travel debugging)

## Parcours couverts

| Fichier | Parcours |
|---|---|
| `journeys/coop-lifecycle.spec.ts` | Chemin nominal complet : l'admin ajoute un produit, corrige sa catégorie, enregistre une exploitation membre ; le membre se connecte, déclare un coût et une production, consulte son dashboard ; l'admin revoit le statut de saisie et le dashboard coopérative agrégé. |
| `journeys/auth-access-control.spec.ts` | Mauvais mot de passe (message d'erreur, pas de navigation) ; visiteur anonyme redirigé vers `/login` ; admin redirigé hors d'une page réservée aux exploitations ; exploitation redirigée hors du backoffice admin. |
| `journeys/offline-entry.spec.ts` | Saisie d'un coût sans réseau (file d'attente IndexedDB locale, message "hors ligne"), puis synchronisation automatique à la reconnexion. |

## Notes de conception

- **Pas de reset de la base entre les runs** : chaque test génère ses
  propres noms/emails uniques (`support/env.ts::unique()`) pour rester
  indépendant des données laissées par les runs précédents. Ne pas
  réintroduire de données fixes ("Ferme Test", `test@elevia.local`, ...)
  sous peine de collisions entre exécutions.
- **`data-testid`** : utilisé uniquement là où un sélecteur par texte se
  casse la figure - typiquement `AdminProductList.svelte`, où le nom du
  produit disparaît du texte affiché (il ne reste que dans la value d'un
  input) dès qu'on passe en mode édition. Ailleurs, les sélecteurs
  accessibles (`getByRole`, `getByLabel`) suffisent et vieillissent mieux.
- **Comptes** : l'admin utilise `ADMIN_EMAIL`/`ADMIN_PASSWORD` (mêmes
  variables que le backend, cf. `.env.example`) ; surchargeables via les
  mêmes noms si votre `.env` local diverge.
- **`workers: 1`** dans `playwright.config.ts` : les specs partagent la
  même base Postgres non réinitialisée: exécution séquentielle par choix,
  pas par limitation technique.

# Gabarit de rapport — empreinte serveur

Ce gabarit reprend la structure exacte utilisée pour l'audit Elevia (Rust/Actix +
Astro/Svelte + PostgreSQL, OVH 1 vCPU/2 Go, Gravelines). Réutilise-le tel quel,
en remplaçant les chiffres par ceux mesurés sur ta propre stack. Les chiffres
ci-dessous sont donnés à titre d'exemple concret, pas comme référence à
reproduire — chaque stack a son propre profil.

---

## Méthode

100 appels séquentiels par type de requête, delta de temps CPU cumulé
(`cpu.stat`, `usage_usec`) avant/après sur chaque conteneur traversé, moins le
bruit de fond mesuré sur une fenêtre calme. Deux passes indépendantes,
cohérentes à ~X% près.

## Ce qui coûte cher en CPU

| Requête | Temps CPU total | Ce qui domine |
|---|---|---|
| Lecture simple (`/health`) | ~X ms | ... |
| Lecture + requête DB | ~X ms | ... |
| Page statique | ~X ms | ... |
| Connexion (hash mot de passe) | ~X ms | ... |

*Exemple Elevia : le handshake TLS dominait toutes les requêtes sauf la
connexion, où bcrypt écrasait le reste d'un facteur 15-20x. Une fois TLS et
hash mis de côté, le code applicatif pur coûtait 0,3-3 ms.*

## Capacité — combien d'utilisateurs simultanés

| Type d'endpoint | Point de rupture (requêtes simultanées) |
|---|---|
| Lecture / dashboard | ~X |
| Connexion (hash coûteux) | ~X |

## Empreinte mémoire serveur

| Conteneur | RAM (au repos) |
|---|---|
| Backend | ~X Mo |
| Frontend | ~X Mo |
| Base de données | ~X Mo |
| Reverse proxy | ~X Mo |
| **Total** | **~X Mo**, X% de la machine |

## Payload réseau

| Asset | Brut | Compressé |
|---|---|---|
| Page HTML | X Ko | X Ko |
| Runtime framework | X Ko | X Ko |
| Plus grosse dépendance tierce | X Ko | X Ko |

## Empreinte mémoire navigateur (si mesurée)

| Étape | Mémoire (tas JS) |
|---|---|
| Premier chargement | X Mo |
| Après connexion | X Mo |
| Page la plus lourde | X Mo |

## Conversion en CO2

Hypothèses : ~5 W/vCPU à pleine charge, PUE ~1.2, intensité carbone du réseau
électrique local = **[à remplacer par la vraie valeur du pays d'hébergement,
source RTE éCO2mix ou electricitymaps.com — ne pas réutiliser 50 gCO2/kWh
(France) pour un autre pays]**.

- Requête typique : ~X µg de CO2
- Requête coûteuse (hash) : ~X µg de CO2

## Comparaison avec d'autres stacks (si applicable)

*Étiqueter explicitement comme littérature, pas mesure, si l'appli n'a pas été
reconstruite dans l'autre stack pour comparaison directe.*

| Stack | Temps CPU applicatif | Ratio |
|---|---|---|
| Stack étudiée (mesurée) | ~X ms | 1x |
| Stack comparée (littérature, ex. TechEmpower) | ~X ms | Xx |

## Coût de la machine

[Fournisseur, gamme, specs] : ~X €/mois selon le catalogue public consulté le
[date] — à confirmer sur la facture réelle.

## Conclusion

Le CO2 par requête individuelle est presque toujours négligeable en absolu,
quelle que soit la stack. Le vrai levier est le multiplicateur entre choix
d'architecture, qui se répercute sur tout le reste à mesure que le volume
grandit (nombre de machines, électricité, matériel). Formuler la conclusion
autour de ce multiplicateur, pas autour du chiffre CO2 isolé.

## Limites de la mesure

*Section obligatoire. Exemple de formulations à adapter :*

- Pas de wattmètre physique : la conversion CPU→énergie repose sur des
  coefficients publiés, pas une mesure directe.
- Chaque requête de test ouvrait une connexion TLS neuve (curl sans
  keep-alive) — un utilisateur en session persistante paierait ce coût une
  seule fois.
- La comparaison avec d'autres stacks vient de benchmarks tiers, pas d'une
  reconstruction de l'appli dans ces stacks.
- Le prix de la machine vient d'un catalogue public à une date donnée, pas de
  la facture réelle.
- Mémoire navigateur mesurée sur une seule session — ordre de grandeur, pas
  valeur certifiée.

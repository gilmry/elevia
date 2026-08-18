Feature: Isolation des données entre exploitations

  En tant qu'exploitation authentifiée, je ne dois jamais pouvoir lire ou modifier
  les entries, la production ou le dashboard détaillé d'une autre exploitation,
  même en modifiant l'identifiant dans l'URL. Cette isolation doit être imposée
  par le backend (vérification exploitation_id du token vs exploitation_id de la
  ressource demandée), pas seulement côté frontend.

  Background:
    Given un compte administrateur existe
    And l'exploitation "Ferme A" existe avec un compte utilisateur
    And l'exploitation "Ferme B" existe avec un compte utilisateur
    And "Ferme B" a soumis des coûts et une production pour le mois courant

  Scenario: Une exploitation ne peut pas lire les entries d'une autre exploitation
    Given je suis authentifié en tant qu'utilisateur de "Ferme A"
    When je demande les entries de "Ferme B" en modifiant l'identifiant dans l'URL
    Then la réponse est refusée avec un statut 403

  Scenario: Une exploitation ne peut pas lire le dashboard d'une autre exploitation
    Given je suis authentifié en tant qu'utilisateur de "Ferme A"
    When je demande le dashboard de "Ferme B" en modifiant l'identifiant dans l'URL
    Then la réponse est refusée avec un statut 403

  Scenario: Une exploitation ne peut pas soumettre des coûts pour une autre exploitation
    Given je suis authentifié en tant qu'utilisateur de "Ferme A"
    When je soumets des coûts pour "Ferme B" en modifiant l'identifiant dans l'URL
    Then la réponse est refusée avec un statut 403

  Scenario: Une exploitation peut lire ses propres entries
    Given je suis authentifié en tant qu'utilisateur de "Ferme B"
    When je demande mes propres entries
    Then la réponse est acceptée avec un statut 200

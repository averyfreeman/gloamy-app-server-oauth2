# Application macOS : exploitation, sécurité et récupération d'association

Dernière vérification : **11 novembre 2026**.

L'application Tauri réutilise l'interface Vue existante. Elle démarre le daemon Gloamy intégré uniquement lorsqu'aucun daemon local sain n'est disponible. Si un daemon sain existe, l'application s'y connecte sans en prendre la gestion. La WebView se connecte à `127.0.0.1` ou `::1` ; une adresse non locale spécifique n'est pas prise en charge dans cette version POC.

## Cycle de vie du daemon

L'option « Garder le daemon actif après la fermeture » est enregistrée dans les préférences de l'application et désactivée par défaut. Désactivée, l'application demande l'arrêt normal uniquement si le daemon indique qu'il est géré par l'application de bureau. Activée, ce daemon reste disponible pour l'interface web. À la fermeture, l'application n'arrête jamais un daemon sans cette marque, y compris un ancien daemon détecté par son API HTTP.

## Récupération d'association

Depuis Authentification ou Paramètres, demandez un nouveau code à usage unique et terminez l'association via le flux `/pair` existant. Le code transite par un socket Unix réservé au même utilisateur, reste en mémoire et n'est écrit ni dans la configuration ni dans les journaux. La nouvelle association ajoute un hachage de jeton sans invalider les appareils déjà associés. La demande est refusée si l'association est désactivée. Aucun endpoint HTTP ne génère de code.

## Contrôle local et sécurité

Le socket se trouve dans `$TMPDIR/gloamy-<uid>/control.sock` (ou dans le répertoire temporaire équivalent du système). Le répertoire utilise le mode `0700` et le socket le mode `0600`. Le protocole autorise uniquement la consultation du statut, la demande d'un code et l'arrêt normal. Il n'offre ni commande shell générique ni écriture de configuration. Il protège contre les autres utilisateurs locaux, mais pas contre les processus du même utilisateur. La vérification existante des bearer tokens reste active.

## Dépannage

- **Adresse déjà utilisée :** arrêtez le service qui utilise l'hôte et le port configurés, ou modifiez `gateway.host` / `gateway.port` dans `~/.gloamy/config.toml`. L'application ne démarre pas de daemon en double.
- **Récupération indisponible :** vérifiez que l'association est activée et demandez le code depuis l'application de bureau. Un navigateur ne peut pas accéder au socket local.
- **Daemon existant inaccessible :** vérifiez sa disponibilité à l'adresse locale configurée. Un daemon non géré continue de fonctionner après la fermeture de l'application.

Voir le contrat complet dans [Sécurité du socket de contrôle desktop](../../security/desktop-control-socket.md).

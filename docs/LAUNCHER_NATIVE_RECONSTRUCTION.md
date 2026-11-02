# Reconstruction native de RappelzCmdLauncher

État des preuves : 2 novembre 2026.

`RappelzCmdLauncher.exe` reçoit en premier argument le programme à démarrer, prépare deux marqueurs d’environnement associés à un événement Windows héritable, puis appelle `CreateProcessA`. La cible est **`argv[1]`**. La chaîne retrouvée `Launcher.exe` est une valeur de marqueur ; elle ne désigne pas un exécutable de jeu imposé par ce lanceur.

Ce document présente un contrat rédigé à partir des octets et d’analyses statiques. Il ne reproduit pas du pseudocode original, ne constitue pas le source d’origine et ne rapporte aucune exécution du lanceur ou du client. Les résultats du processus d’analyse sont distingués de ceux que le programme étudié produirait en session.

## Artefact et versions établies

| Élément | Identification |
| --- | --- |
| Fichier étudié | `Rappelz NoLimits/RappelzCmdLauncher.exe` |
| Taille | 34 304 octets |
| SHA-256 | `6230684e0b1eb096d39564f742ecf0d36f5ec8b2986917ceaab8db74cf75f1f6` |
| Format | PE32, x86, sous-système GUI |
| Base / entrée | `0x400000` / `0x40181b` |
| Ghidra | 12.1.4 PUBLIC |
| Java | 21.0.12.1+1-LTS, Eclipse Adoptium HotSpot, selon le fichier `release` du JDK épinglé |
| Profil processeur / compilateur d’analyse | `x86:LE:32:default` / `windows` |
| Décodage indépendant des instructions | Capstone 5.0.7 |

Le dossier `Rappelz NoLimits`, les noms de fonctions reconnus et les ressemblances avec des bibliothèques CRT ne suffisent pas à identifier une version commerciale du client, un compilateur exact ou un Epic. **Aucun Epic n’est déduit de ces preuves.** Les profils et le hash du PE permettent de rattacher les conclusions au fichier effectivement étudié. [S1, S8, S10]

## Entrée, arguments et cible

L’entrée `0x40181b` initialise le cookie de sécurité, puis rejoint le démarrage CRT `0x4016ae`. Le CRT obtient la ligne ANSI, construit les arguments via `0x40301c` et `0x402e82`, initialise l’environnement et les callbacks, puis appelle la fonction d’application `0x4011d0`.

Le parseur traite les guillemets, espaces et tabulations hors guillemets, les suites d’antislashs devant un guillemet et les caractères multioctets. Il ne s’agit donc pas d’un simple découpage sur les espaces. Sa signature imprimée était incomplète : les appels originaux utilisent aussi EDX et EDI pour l’entrée et le comptage.

Les initialiseurs `0x405700` et `0x405710` relient les globaux du CRT aux globaux utilisés par l’application : pointeur `argv` de `0x409b50` vers `0x409b40`, puis `argc` de `0x409b4c` vers `0x409b44`. Ils sont référencés par la table `[0x4060fc, 0x406108)` parcourue par `__cinit`.

Avec moins de deux arguments, aucun enfant n’est créé. Sinon, `argv[1]` est transmis comme `lpApplicationName` à `CreateProcessA`, au site `0x40129d`. Aucun analyseur de switches propre au jeu, chemin de jeu fixe ou appel à un shell n’est observé sur ce parcours. [S1]

La revue bornée des 146 textes et des 61 imports ne montre pas d’accès propre au lanceur à un fichier de configuration, au registre ou à un endpoint réseau. L’appel CRT à `WriteFile` appartient à la sortie d’erreur. Les résolutions et chargements dynamiques restent une limite : ce constat ne ferme pas tous les comportements indirects ou ceux des dépendances. [S1]

## Environnement et événement héritable

La préparation `0x401080` cherche le dernier **antislash** de `argv[1]` pour en extraire le nom de base ; sans antislash, elle emploie l’argument entier. Aucun traitement équivalent des slashs ordinaires ni contrôle d’existence de la cible n’est observé.

À `0x40110a`, elle crée un événement anonyme, à réinitialisation automatique, initialement non signalé, avec un handle héritable. Le décodeur `0x401000` permet ensuite de reconstituer les chaînes suivantes à partir des octets originaux, sans exécuter ce code :

| Utilisation | Valeur établie |
| --- | --- |
| Premier format décodé | ` %s_PARENT`, avec un espace initial |
| Format réellement passé au formateur | `%s_PARENT`, car le pointeur avance d’un octet |
| Variable correspondante | `<nom-de-base>_PARENT=Launcher.exe` |
| Second format | `%s_RUNNER` |
| Variable correspondante | `<nom-de-base>_RUNNER=<handle en décimal non signé>` |

Les deux écritures d’environnement se trouvent à `0x401161` et `0x4011a8`. Le programme ne vérifie pas leurs résultats ni celui de `CreateEventA`. Le helper renvoie le handle, mais l’application ignore ce retour. Aucune fermeture explicite de cet événement n’est observée dans ces deux fonctions.

`CreateProcessA` reçoit `bInheritHandles=TRUE`, un environnement nul et un dossier courant nul. Selon le contrat normal de l’API, cela permet à l’enfant d’hériter de l’environnement modifié, du dossier courant et du handle héritable. **L’usage de ces variables et la signalisation de l’événement par le client ne sont pas établis ici.** [S1]

## Reconstruction de la ligne de commande

L’application commence par un tampon ANSI vide de 1 024 octets. Pour les arguments d’indice 1 à `argc-1`, elle utilise le format exact `%s "%s"`, avec le tampon comme contenu précédent et comme destination. Elle transmet ensuite le tampon avancé d’un octet pour retirer l’espace initial.

Pour des arguments simples, la ligne obtenue correspond donc à `"argv[1]" "argv[2]" ...`. Le nom du lanceur, `argv[0]`, est omis. Les arguments ont déjà été analysés par le CRT avant d’être entourés de nouveaux guillemets ; la ligne d’origine n’est pas copiée telle quelle.

Les attributs de processus et de thread sont nuls, les flags de création valent zéro, et `STARTUPINFOA` est initialisé à zéro avec `cb=0x44`. Après création réussie, le programme tente de fermer les trois handles standard restés à zéro dans cette structure, puis les handles de processus et de thread. Il ne vérifie pas ces fermetures et n’attend pas la fin de l’enfant.

La netteté de ce contrat se limite aux opérations observées : les formats non bornés, les tampons de 1 024 et 128 octets, le chevauchement source/destination et l’absence de nouvel échappement complet ne permettent pas de certifier les arguments longs, les guillemets internes ou les antislashs terminaux. Aucun essai d’exécution de ces cas n’a été effectué. [S1]

## Quatre entrées absentes du premier inventaire

Le premier inventaire comprenait 146 fonctions internes et 61 imports externes. Trois lots ont produit 146 textes sans erreur. La lecture des tables et des octets a pourtant établi quatre entrées supplémentaires :

| Entrée / dernier octet inclus | Mot de table | Comportement établi |
| --- | --- | --- |
| `0x4019e5..0x4019f4` | `0x406110` | Test de capacité processeur, stockage du résultat, retour zéro |
| `0x402562..0x402592` | `0x406114` | Allocation et initialisation d’une table de callbacks, retour zéro ou 24 |
| `0x402bef..0x402bfc` | `0x40611c` | Installation du filtre d’exception CRT, retour zéro |
| `0x405710..0x40571a` | `0x406104` | Copie d’`argc`, sans retour sémantique établi |

Les trois premières appartiennent à la table `[0x40610c, 0x406120)`, consommée par `__initterm_e`. Celui-ci appelle le pointeur à `0x401439`, teste EAX au tour suivant et transmet son résultat à `__cinit`, qui le teste à `0x401483`.

La première récupération réussie (V1) emploie un **profil Ghidra distinct et isolé**. Les quatre entrées et corps y ont réellement été créés et observés : 90 octets et 34 instructions correspondent à l’original. Le compteur y passe de **146 à 150 fonctions internes**, soit de 207 à 211 avec les externes. Quatre nouveaux textes sont alors produits. Ce profil ne prétend pas reproduire exactement la base REA initiale. [S2]

La V2 reprend les mêmes quatre entrées. Elle aligne les métadonnées des trois initialiseurs sur un retour de quatre octets dans EAX ; leurs textes imprimaient déjà des retours cohérents. Le copieur d’`argc` reste non forcé. **Les passes ne s’additionnent pas : il ne s’agit pas de quatre nouvelles fonctions à chaque essai.** Les V3/V4 emploient ensuite un profil ciblé qui ne recrée pas ces quatre entrées ; leur compteur de 207 reste stable, sans constituer une régression du travail précédent. [S2, S3]

## Retours de main et stockage custom ECX/EAX

Les octets originaux montrent trois résultats entrants dans EAX, avant les appels au helper `0x401308` :

| Condition | Production du résultat | Appel au helper | Mot retourné si le cookie est valide |
| --- | --- | --- | --- |
| `argc < 2` | `0x4011ed` | `0x4011f9` | `0xffffffff` |
| `CreateProcessA` réussit | `0x4012f8` | `0x4012fa` | `0x00000000` |
| `CreateProcessA` échoue | `GetLastError` à `0x4012a7` | `0x4012b7` | Les mêmes 32 bits que ce résultat |

La signedness du type source n’est pas établie. Le mot `0xffffffff` vaut −1 s’il est interprété comme un entier signé de 32 bits. Le zéro décrit la création du processus, pas la réussite du programme enfant.

Le helper exact `0x401308..0x401316` compare ECX au cookie global. Si les valeurs correspondent, son retour rapide ne modifie pas EAX. Les trois queues de main conservent également EAX jusqu’au `RET 0x10`. La composition de ces opérations établit les résultats du tableau **uniquement sur ce chemin de cookie valide**.

La V3 a tenté un inlining analytique du helper. Le résultat tout à un est devenu visible sur la première branche, mais deux autres sites n’ont pas pu être inlinés. Le zéro et le résultat d’erreur n’étaient donc pas restitués sur ces deux sites. [S4]

La V4 conserve l’inlining désactivé et explicite l’état des registres du seul helper :

| Élément analytique | Stockage observé |
| --- | --- |
| Cookie entrant | `ECX:4` |
| Résultat entrant conservé | `EAX:4` |
| Retour | `EAX:4` |

Les deux entrées ne se chevauchent pas ; le partage d’EAX entre entrée et sortie est intentionnel. Le corps analysé restitue le résultat entrant lorsque le cookie correspond. Les trois appels de main montrent désormais les valeurs tout à un, zéro et le résultat affecté de `GetLastError`. La convention et la purge du helper sont préservées ; les métadonnées de main et de la destination d’échec ne changent pas pendant ce modèle. Les 589 octets gardés restent identiques. [S5, S6, S7]

### Pourquoi le C imprimé n’est pas directement compilable

Le nom `__fastcall` demeure affiché, mais `custom_storage=true` impose une liaison supplémentaire que ce nom ne décrit pas. Le profil Windows ordinaire utilise ECX puis EDX pour les deux premiers arguments entiers ; le modèle de la V4 utilise **ECX puis EAX**. Le deuxième « paramètre » rend explicite un registre préservé et ne prouve pas l’existence d’une déclaration originale à deux paramètres.

Compiler directement la déclaration imprimée avec un `__fastcall` ordinaire ne reproduirait donc pas cette ABI. Une future implémentation compatible nécessiterait une prise en charge délibérée des registres, ou une reconstruction du comportement à un autre niveau. Aucune implémentation n’est fournie ici. Le choix éventuel d’`int` par l’imprimeur pour le cookie ne démontre pas non plus sa signedness d’origine : les métadonnées fixes restent `undefined4`. [S5, S9]

## Voie d’échec et limites de restauration

Si le cookie ne correspond pas, le helper rejoint `0x401825`. Cette destination conserve un attribut `noreturn` importé. Ses octets contiennent toutefois un appel à `TerminateProcess` à `0x401923`, suivi de `LEAVE/RET` à `0x401929/0x40192a`. L’analyse présente ne démontre pas que cette voie ne puisse jamais revenir et n’y garantit pas la conservation du résultat. Aucun attribut d’échec n’a été forcé pour obtenir les résultats normaux. [S4, S5]

La V3 nommait initialement un booléen de manière trop forte : le retour de `endTransaction(false)` attestait une **demande** de rollback, pas une restauration effective. Le probe immédiat montrait encore le nouveau type de main et le drapeau inline du helper. Une transaction extérieure est une explication possible, sans avoir été prouvée.

La V4 distingue explicitement la demande API et l’observation : la restauration immédiate reste fausse. Après sortie du processus sous `-readOnly`, les fichiers `.gpr/.rep` du projet isolé sont réellement absents ; l’original et sa copie sont inchangés. Le discard observé du projet et la conservation des octets ne sont pas assimilés à une restauration des métadonnées pendant le script. [S4, S5, S7]

## Ce qui est validé, et ce qui reste à faire

Les quatre textes supplémentaires prouvent que quatre entrées et corps justifiés ont été intégrés à un profil d’analyse. Ils ne prouvent ni l’exhaustivité des fonctions du PE, ni la justesse de chaque signature imprimée, ni le fonctionnement de l’application. Les difficultés de retour, l’inlining partiel et le probe non restauré en donnent des exemples concrets.

La composition normale de la V4 est établie conjointement par les octets, stockages et textes. Le zéro réellement observé à la sortie de l’outil appartient au **processus d’analyse**. Le lanceur, ses commandes reconstituées et le client n’ont pas été exécutés ; aucune session serveur ou connexion à deux clients n’est validée par ce travail.

Restent notamment ouverts : la voie de cookie invalide, l’usage réel des marqueurs/événement par l’enfant, les arguments complexes ou trop longs, les erreurs de préparation ignorées et la validation d’une future implémentation. Aucune fonction supplémentaire n’est comptée pour les modèles V3/V4, aucune donnée de jeu ni configuration installée n’est modifiée.

## Registre des preuves

Les chemins sont relatifs à `A = client-data/audit/2026-11-02-module-rappelzcmdlauncher-89c21a84/`. Les contenus locaux ne sont pas reproduits dans ce document. Les hashes SHA-256 permettent d’identifier les dossiers de preuve utilisés.

| Référence | Fichier sous A | SHA-256 |
| --- | --- | --- |
| S1 | `launcher-behavior-contract.local.json` | `1099629ef124476877ca15506a74c94e21373fbf19ac09cae60f7b1320ef3b62` |
| S2 | `launcher-entry-recovery-fresh-02/independent-review.local.json` | `4b67eb9fdab025c279e98762c752c4310baa137114dcc934a1e1be18d2cb4299` |
| S3 | `launcher-entry-recovery-fresh-03/independent-review.local.json` | `0b74b2b2cd629b6350b4c5c30719db6cfe181b2a954aadba5361e5aa5b674276` |
| S4 | `launcher-cookie-model-fresh-01/independent-review.local.json` | `a83da4d168289237a38c1ff8dcb8db1a3ec924ae73e617313d22d763263b46d5` |
| S5 | `launcher-cookie-storage-fresh-01/independent-review.local.json` | `759e0c66604ae433812554bc3fd14af8ee6311db3524345437f17f44cb7c2a8d` |
| S6 | `launcher-normal-return-contract-v4.local.json` | `1551df488f76ec2c3dd0b831c274b17db0cac6b36a1475a0e9cb13a23123f648` |
| S7 | `launcher-cookie-storage-fresh-01/post.local.json` | `1c353cbf459d338e98e41248b618533358cddc3c86f424d660041dd394837f38` |
| S8 | `launcher-cookie-storage-fresh-01/run-profile.local.json` | `d9f5b8458075ad0a334e38144b5be120180720c07b90c3990c1055638c3b6b06` |
| S9 | Dans Ghidra 12.1.4 : `Ghidra/Processors/x86/data/languages/x86win.cspec`, prototype `__fastcall`, ligne 113 | `5145482b1c9aa6a9c6ad4d5554f98a35fb11b82a13e226df96018a171ea94988` |
| S10 | Fichier `release` du JDK sélectionné par `java_home` dans S8 | `07117c72ce033949c14878e07fbf2fe23f59a1f8c90a6d1351b5c89099847ce7` |

Les 146 textes initiaux proviennent des trois lots `global-pseudo-00000001.json` à `global-pseudo-00000003.json`, dont les identifiants Evidence et hashes sont conservés dans S1. Les items 0, 1, 2, 3 et 22 du premier lot décrivent respectivement le décodeur, la préparation, main, le helper cookie et sa destination d’échec. Les expériences Ghidra locales possèdent leurs propres profils et ne deviennent pas, par renommage, de nouveaux Evidence publics REA ou des succès d’exécution du jeu.

Le manifeste local `A/launcher-native-reconstruction-publication.local.json` épingle cette version du document et ses sources. Les contrats et revues antérieurs restent conservés ; leurs formulations de rollback doivent être lues avec la correction d’observation explicitée ci-dessus.

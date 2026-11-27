# Groupes — client Rappelz NoLimits (SFrame.exe)

Analyse **statique** (Ghidra 12.1.4 via REA) de `Rappelz NoLimits\SFrame.exe`, SHA-256
`27f42fc8f6b08df8c239640b938d601aeaebb7b3db0c89fc47755325a98a410d`. SFrame.exe n'a jamais été exécuté. Conventions de
`summon_protocol.md` : [O] observé dans l'assembleur ou les octets, [D] déduit. Constats d'un sous-agent ; les points relus
sur le binaire portent leur Evidence ID.

Le client **n'a pas de paquet de groupe** : tout passe par le chat, dans les deux sens.

## 1. Serveur → client : TS_SC_CHAT (22), type 100

- [O] TS_SC_CHAT (22) : expéditeur char[21] à +7, longueur u16 à +0x1c, type u8 à +0x1e, texte à +0x1f (FUN_00a26e60). Le texte
  est reposté en message interne SMSG_CHATTING.
- [O] FUN_006cac50 aiguille le **type 100** (`cmp …, 0x64`) vers FUN_006ca2c0. Celui-ci découpe le texte sur **`|`** (chaîne
  0xd31b5c, relue : `ev_c7c2ad593afc118f705f108f5aae9d182b8847fb0b60eca006a51f2d29266804`), puis passe la main à trois gestionnaires
  (noms RTTI) : CPartyListMgr (FUN_006f2fb0), SPartyMgr (FUN_006f5360) et SMessengerMgr (FUN_006f4e90).
- [O] Seul le type est testé. L'affichage du chat masque l'expéditeur **`@PARTY`** avec le type 100 (FUN_0074bd30 ; chaîne 0xd369e8,
  `ev_c685391d952886e9a5ecca65325ac5c185d11cf7da27cd81f1f26b0d20b9bb0f`). Le serveur envoie donc `@PARTY`, type 100.
- Les nombres sont lus avec atoi, les handles avec strtoul en base 10. Identifiant de groupe et clé doivent rester des i32 positifs.

| Commande | Champs (après la commande) | Effet côté client |
|---|---|---|
| `CREATE` | groupe, chef[, type] | crée le groupe ; type 0 = normal, 1/2/4 = raid ou équipe d'attaque, 3 = cas à part |
| `PINFO` | id, groupe, chef, partage, A, B, type, puis 9 champs par membre | état complet (§2) |
| `INVITE` | inviteur, groupe, id, clé | boîte « RequestPartyJoinAccept » ; ignoré si déjà en groupe |
| `JOIN` | groupe, membre | ajout d'un membre |
| `LEAVE` | membre, drapeau | départ ; drapeau 1 affiche en plus le texte 0x984 |
| `KICK` | groupe, membre | exclusion |
| `DESTROY` | groupe, drapeau | ferme la fenêtre de groupe |
| `PROMOTE` | groupe, nouveau chef | le client ne contrôle que 2 champs mais lit le 3e : en envoyer 3 |
| `LOGIN` / `LOGOUT` | groupe, membre / membre | membre en ligne / hors ligne (handle remis à 0) |
| `MINFO` | 9 champs membre | ajout ou mise à jour d'un membre |
| `CHANGE_NAME` / `CHANGE_LEVEL` / `CHANGE_JOB` | membre, valeur | nom, niveau (+0x24), métier (+0x20) |
| `MODE` | n | mode de partage du butin |
| `LIST` | x, mode, A, B | mêmes champs que PINFO 4 à 6 |
| `PPOS` | (handle, x, y) répétés | positions des membres |
| `ASSIST` | handle | cible d'assistance |

Non analysés : `RMINFO`, `SINFO`, `MSINFO` (raid), et les invitations d'arène.

Le client ne vide pas son groupe sur un `LEAVE` ou un `KICK` qui le nomme (FUN_006fc500 retire seulement le membre). Le serveur
envoie donc `DESTROY` au partant ou à l'exclu, et `LEAVE` ou `KICK` aux autres [D].

## 2. PINFO

FUN_006fa0e0 exige au moins 8 champs :

`PINFO|id|groupe|chef|partage|A|B|type` puis, pour chaque membre, `handle|nom|niveau|métier|f4|f5|x|y|f8`.

- Les champs 1 à 6 ne sont lus qu'au premier PINFO d'un groupe ; le type l'est à chaque fois.
- Un handle 0 marque un membre hors ligne. Le chef est reconnu par comparaison avec son nom.
- [O] La fenêtre affiche au plus **8 membres** (FUN_008d4760).
- [D] f4 et f5 (membre +0x58 et +0x5c) sont probablement les PV et PM en pourcentage : RappelzCore les envoie ainsi.
- Inconnus, envoyés à 0 : A, B et f8.

## 3. Client → serveur : TS_CS_CHAT_REQUEST (20), type 0

- [O] FUN_006d77a0(type, cible, texte) : cible char[21] à +7, u8 0 à +0x1c, longueur u8 à +0x1d (170 au plus), type u8 à +0x1e,
  texte à +0x1f. C'est la disposition déjà décodée par `CsChatRequest`.
- [O] Fenêtre et menu de groupe (FUN_006d2e60) : `/pcreate <nom>`, `/pinvite <joueur>`, `/pkick <joueur>`, `/pdestroy`, `/pleave`,
  `/ppromote <joueur>`, `/pshare <mode>`, tous en type 0. Sauf `/pcreate` et `/pleave`, le client les réserve au chef (sinon il
  affiche lui-même le texte 0x1e0).
- [O] Réponse à INVITE (FUN_00798260) : accepter envoie `/pjoin <id> <clé>`, refuser envoie `/pjoin <id>`. Formats `%s %d %d` et
  `%s %d` relus, `ev_442f25e360b94eebaeab124e2e40b0e39347ccfc5f55e791fa26285eecb85a30` et
  `ev_a59b31a203f27bec4636e4f7a8c397ef8dccb21938eaaad30f7b3b514cbc1947`. Rien n'est envoyé si l'id vaut −1.
- [O] Le message interne de cette réponse est transmis par FUN_006d2e60 (cas 0x3b), qui prend le type à l'octet +0x29. FUN_00798260
  met cet octet à 0 : la réponse part donc en type 0, sans cible.
- [O] Les boîtes de confirmation (FUN_00797f10) envoient `/party_kick <joueur>`, `/party_disband` ou `/party_leave`. Le serveur
  accepte donc les deux orthographes.

## 4. RappelzCore

- **Protocole** : `rz_proto::party_text` construit les messages du §1 et reconnaît les commandes du §3 (`Command::parse`).
- **Groupes** : `worldserver/src/party.rs`. Un groupe est attaché aux personnages (guid) et survit à la déconnexion, comme dans
  AzerothCore. Les membres hors ligne restent affichés, avec un handle 0.
- **Base** : tables AzerothCore inchangées `groups` (`leaderGuid` = chef, `lootMethod` = mode de partage, `groupType` = 0) et
  `group_member`, via `rz_database::groups`. Le nom du groupe n'a pas de colonne : après un redémarrage, il redevient le nom du chef.
- **Règles du serveur [D], le client n'en donne aucune** :
  - 8 membres au plus, la limite de la fenêtre.
  - Invitation valable 60 s, avec une clé aléatoire vérifiée par `/pjoin`.
  - On peut inviter tout joueur connecté.
  - Le chef qui part cède sa place au membre suivant.
  - Un groupe d'un seul membre reste valide, comme après `/pcreate` ; il disparaît avec son dernier membre.
  - Un personnage supprimé quitte son groupe.
- **Victoire en groupe** : règle d'AzerothCore (`xp_in_group_rate`).
  - Membres concernés : ceux qui sont vivants, dans la couche du tueur et vus par lui.
  - Part de chacun : `total × coefficient × niveau / somme des niveaux`, pour l'EXP comme pour les JP. Coefficient : 1,0 jusqu'à
    2 membres, 1,166 à 3, 1,3 à 4, 1,4 au-delà.
  - Chacun reçoit le crédit de quête ; les rupees et l'EXP de créature restent au tueur.
  - Le mode de partage (`/pshare`) est enregistré et diffusé (`MODE`), mais le butin au sol reste au tueur : les valeurs des modes
    ne sont pas connues.
- **Messages envoyés** :

| Événement | Destinataires et messages |
|---|---|
| création (`/pcreate`, ou première invitation sans groupe) | au chef : `CREATE`, puis `PINFO` |
| invitation | à l'invité : `INVITE` |
| acceptation | aux autres membres : `JOIN` ; à tous : `PINFO` |
| départ ou exclusion | au partant : `DESTROY` ; aux autres : `LEAVE` ou `KICK`, `PROMOTE` si le chef change, puis `PINFO` |
| changement de chef | à tous : `PROMOTE`, puis `PINFO` |
| dissolution | à tous : `DESTROY` |
| mode de partage | à tous : `MODE` |
| connexion d'un membre | aux autres : `LOGIN` ; à tous : `PINFO` |
| déconnexion d'un membre | aux autres : `LOGOUT` |
| montée de niveau | à tous : `CHANGE_LEVEL` |
| toutes les 2 s, dès 2 membres connectés | à tous : `PPOS` |

- **Extension RappelzRust** : le client RappelzRust garde son extension versionnée (62000/62001) et ne reçoit pas les textes
  `@PARTY`. Elle n'affiche un groupe qu'à partir de deux membres connectés ; si le chef est hors ligne, le premier membre connecté
  apparaît comme chef.
- **Commande `.party`** : create, invite, accept, decline, leave, kick, leader, share, disband. Elle passe par les mêmes actions.

## 5. Essai réel (MariaDB 3310, rz-bot)

Rcore (niveau 20) crée « Les Fiers » et invite Rtwo (niveau 1), qui accepte par `/pjoin 1 <clé>`. Rcore tue un Louveteau Affamé,
passe le partage à 1 puis cède la tête à Rtwo.

- **Messages reçus** : Rtwo reçoit `INVITE`, `PINFO` (2 membres), `MODE|1`, `PROMOTE`, 8 `PPOS`, puis `LOGOUT|Rcore` au départ de Rcore.
- **Partage** : Rcore gagne 69 EXP et 12 JP, Rtwo 3 EXP et 0 JP ; les rupees vont à Rcore seul. La base passe de 365 à 434 EXP pour
  Rcore, de 0 à 3 pour Rtwo.
- **Base** : `groups` (1, chef 2, partage 1) et deux lignes `group_member`.
- **Après un redémarrage** : le groupe est rechargé. Rtwo, seul en ligne, reçoit un `PINFO` où Rcore a le handle 0. `/pkick Rcore`
  donne `KICK`, `/party_disband` donne `DESTROY`, et les tables se vident.
- Le client graphique d'origine n'a pas été essayé : la chaîne `CREATE`/`PINFO` d'un nouveau groupe est déduite des gestionnaires.

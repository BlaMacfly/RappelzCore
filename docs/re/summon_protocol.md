# Invocations (créatures du Summoner) — client Rappelz NoLimits (SFrame.exe)

Analyse **statique** (Ghidra 12.1.4 via REA) de `Rappelz NoLimits\SFrame.exe`, PE x86,
SHA-256 `27f42fc8f6b08df8c239640b938d601aeaebb7b3db0c89fc47755325a98a410d`. SFrame.exe n'a jamais été exécuté.
Complète `network_protocol.md` §8 (ENTER) et §9 (gameplay) ; conventions reprises de `quest_protocol.md`.

Conventions :
- **Offsets = offsets paquet** (en-tête TS_MESSAGE de 7 octets inclus : `u32 size, u16 id, u8 checksum`).
- Objets internes `SMSG_*` / `SIMSG_*` : « kind » à objet+4, **offset SMSG = offset paquet + 0x0c** (les champs
  utiles commencent à SMSG+0x13 = paquet+7).
- `[O]` = lu dans le code (adresse citée) ou mesuré sur le fichier ; `[D]` = déduit (nom, sens, enchaînement non
  observé directement) ; `[?]` = inconnu.
- **Taille** d'un paquet S→C = dernier octet lu par le parseur + 1 (le parseur FUN_00a2fed0 ne contrôle pas la
  taille annoncée) ; taille C→S = valeur écrite dans `size` par le constructeur.
- `rec+xx` = offset dans l'enregistrement **db_creature.rdb** (942 o., §7) ; `slot+xx` = offset dans le
  gestionnaire **SCreatureSlotMgr** (UIInstance+0x260, voir `../SFRAME_STAT_ROUTING_RE.md`).
- Handles : `card_handle` = handle de l'**objet carte** (item) dans l'inventaire ; `summon_handle` = handle de la
  **créature** (objet monde de type 4), attribué par le serveur et connu du client avant l'invocation.

Evidence IDs REA principaux :

| Contenu | Evidence ID |
|---|---|
| Parseur S→C FUN_00a2fed0 (cases 0x12d–0x142, 0x1fd, 0x202, 0x3ea, 0x3eb, 0x11f) | `ev_a43ed3815bdf077a8526f96971dbd62efd47c6f218cf1ed61715c0d55aa81686` |
| Assembleur du gestionnaire jeu FUN_006d2e60 (envois 303 / 323 / 324 / 452) | `ev_205114af9075fad59a94fbc8062556a54df0ae8918a8bce6f8d30dbfb5d086c3` |
| Envois isolés FUN_006c9420 (303), FUN_006d09a0 (323), FUN_006d0940 (452) | `ev_c536c5de47854d20eaf4184be7c2565ded368583d2ac29f3b85eff4c8af20c2f` |
| ENTER summon/pet FUN_006a9430 / FUN_006aab90, monture FUN_006afa10 / FUN_006b0e70, SCreatureSlotMgr FUN_0070ffd0 / FUN_007102f0 / FUN_00711b30 | `ev_0113bd92f0b6ccd0ecac6146c9cd1b4bb7ab043c63d582c367eaa35933f91330` |
| ENTER synthétique d'invocation FUN_006ae3a0, fenêtre invocation simple FUN_00773cf0 | `ev_926da516eb77810b2f4a9af2655ad65899ca4c3d559e7693b5bb9f734a4162c8` |
| Carte sélectionnée FUN_00711bc0, lookup créature FUN_006acb90, slots FUN_00710820 / FUN_007108f0 | `ev_4cd8487a0de4aa5eb00fa2bebc3be83015751e2707da204998b0eae1022fbe2a` |
| Prédicats / mises à jour FUN_00710be0, FUN_00710ab0, FUN_00710a00, FUN_00711250, FUN_00710f50 | `ev_20e34a35519eaad22923d3d19d3135a41be2464b002883b809f50004a512801d` |
| Action skill 4001/4002 FUN_0079c590, contrôle FUN_006d1cc0 | `ev_f81b1d7a1adb358e0b6a2ea64975f30105198012e18b3ad9edb7f468233e61f3` |
| Construction de l'action FUN_006d8d40 | `ev_0a158c2161599cbf56ef4b1be2456dbbc1376a91abc12593b8cbe14a76891c11` |
| Ctors SGameLocalCreature FUN_00969b90 / SGameOtherCreature FUN_00974ed0 | `ev_c0a867c2594b54ee7b38f2c9b081c2b4e08889af6834c8856b5fa44d72f5f11c` |
| Ctor SGameCreature FUN_00963e10 (vitesses, échelle) | `ev_8814dfc4710e2de7128c26dc52970d5a6d40393b1af4391570e8c94ddf9e5535` |
| Chargeurs db_creature FUN_00660a40 et db_SummonExp (×4) | `ev_0905d27e2673c70f262b5ce45a52c14081fccc9ecbd0b834d2e1e51c7a08aa7b` |
| Chaînes « db_creature » / « db_summon » ; xrefs | `ev_5221277f977e5e9a4db71e9259d0e78c3b513a07f7a7b9a1e84f1d7a6c2588bf` / `ev_c9a97399c601fd8b1712624d103d267149ca0c3e9cbfa77a4bd639a301dd89d2` / `ev_7fbc24e18d9e2c4b2f8c96c1824b9f3d02dcd42dbba0c23cbb827c515e490fa1` / `ev_c9bef05c806bea6fe0ce911f1977bdf7b4c6c085ae5e398491ca20f5850eaa0e` |
| Appelants de FUN_006acb90 (68) | `ev_952b643e4055a693e6f3cc89c9fd612788115905d393c95a07e03a5a8e68863a` |
| Lecteurs de champs créature (lot 1 : icônes, rang, évolution, vitesses, modèle…) | `ev_b574855273ba5d03fbb9ff4e59157952662457cdcddaebdf1b4039cecf53f09f` |
| Lecteurs de champs créature (lot 2 : fiche carte, info-bulle, espèce, carte…) | `ev_106e464e87367950f78a8d938086b5bf2178984b5b89087a7bffbc411038fb5d` |
| SummonExp : getter FUN_00648260 / niveau depuis EXP FUN_006481b0 (assembleur) | `ev_feb3e89350cd7d21a20a14850558fe2281e415226bc99b01ffdf669b8e3acbf6` / `ev_cb4603a5e73b775b1c42ecbf5db17160bb7000954f765807a153c71c549791ee` |
| Ferme à créatures FUN_0088ee30 (plafonds de niveau par stade) | `ev_967665c125f065a4f19d10118e328bb056ecf496b67d6fd575b7b3cfd9715d09` |
| UseOrEquipItem FUN_0079b9a0 (ACT_CREATURE_USE = objet utilisé sur une créature) | `ev_31f5e0d9fa5fe9d80e6c1eb414bce8dd08b4b99d7472eb62ce5ea86cbf1a641b` |

## 1. Vue d'ensemble des paquets

| Paquet | ID | Dir | Taille | Corps (offsets paquet) | Fonction / effet client |
|---|---|---|---|---|---|
| TS_SC_ADD_SUMMON_INFO | 301 | S→C | **46** | +7 u32 card_handle ; +0xb u32 summon_handle ; +0xf char[19] name ; +0x22 i32 code ; +0x26 i32 level ; +0x2a i32 sp | kind 0x23 → FUN_006d2e60 + FUN_0070ffd0 : crée/met à jour la fiche de créature (§2) [O] |
| TS_SC_REMOVE_SUMMON_INFO | 302 | S→C | 11 | +7 u32 card_handle | kind 0x24 : retire l'entrée et la fiche dont card_handle correspond (FUN_007102f0) [O] |
| TM_EQUIP_SUMMON | 303 | S→C | 32 | +7 u8 open_dialog [D nom] ; +8 u32 card_handle[6] | kind 0x25 → FUN_00711b30 : copie les 6 handles dans slot+0x1c..+0x33 (formation) ; l'octet +7 n'est pas lu par ce consommateur [O] |
| TM_EQUIP_SUMMON | 303 | C→S | 32 | +7 u8 = 0 (toujours) ; +8 u32 card_handle[6] | FUN_006d2e60 kind 0x419 (0x6d55c0) et FUN_006c9420 : nouvelle formation [O] |
| TS_CS_SUMMON | 304 | C→S | — | — | **aucun constructeur trouvé** (§5) : l'invocation passe par TS_CS_SKILL 4001 [O absence, voir limites] |
| TS_SC_UNSUMMON | 305 | S→C | 11 | +7 u32 summon_handle | kind 0x27 (monde FUN_006b2130) : si l'objet existe et est de type 4, FX 0x13cb, marque obj+0x5e4 = 1 [O] |
| TS_SC_UNSUMMON_NOTICE | 306 | S→C | 15 | +7 u32 summon_handle [D] ; +0xb u32 (délai ?) [D] | kind 0x28 ; consommateur non identifié [O parse] |
| TS_SC_SUMMON_EVOLUTION | 307 | S→C | **38** | +7 u32 card_handle [D] ; +0xb u32 summon_handle ; +0xf char[19] name ; +0x22 i32 code (nouvelle forme) | kind 0x29 (monde) : FX 0x13cb sur l'objet summon_handle [O] ; mise à jour de la fiche non localisée [?] |
| TS_SC_TAMING_INFO | 310 | S→C | 16 | +7 u8 mode ; +8 u32 tamer_handle [D] ; +0xc u32 target_handle | kind 0x2e (monde) : mode 0 = début (drapeau monde+0x128), 1, 2/3 = fin (FX + IHM) [O branches, D sens] |
| (sans nom TM_) | 311 | S→C | 11 | +7 u32 | écrit directement dans `[DAT_00ed0998]+0x11f4` (pas d'objet SMSG) [O] ; sens [?] |
| TS_SC_MOUNT_SUMMON | 320 | S→C | **24** | +7 u32 handle (cavalier) ; +0xb u32 summon_handle ; +0xf f32 x ; +0x13 f32 y ; +0x17 u8 success | kind 0x4d → FUN_006afa10 (monde) + message 0x168 si joueur local [O] |
| TS_SC_UNMOUNT_SUMMON | 321 | S→C | 16 | +7 u32 handle ; +0xb u32 summon_handle ; +0xf u8 flag | kind 0x4e → FUN_006b0e70 + message 0x169 si joueur local [O] ; `flag` non lu par ces deux handlers |
| TS_SC_SHOW_SUMMON_NAME_CHANGE | 322 | S→C | 11 | +7 u32 handle | kind 0x67 ; consommateur non identifié (ouverture de la saisie de nom [D]) |
| TS_CS_CHANGE_SUMMON_NAME | 323 | C→S | **26** | +7 char[19] name (sans handle) | FUN_006d2e60 kind 0x46d (0x6d634c) et FUN_006d09a0 [O] |
| TS_CS_GET_SUMMON_SETUP_INFO | 324 | C→S | **8** | +7 u8 = 1 si la fenêtre 0x2c n'est pas ouverte (FUN_009b68a0(0x2c) == 0) | FUN_006d2e60 kind 0x3a, chaîne `req_summon_formation` (0x6d3601) [O] ; réponse attendue : 303 [D] |
| TS_CS_SUMMON_CARD_SKILL_LIST | 452 | C→S | 11 | +7 u32 handle (carte [D]) | FUN_006d2e60 kind 0x4e1 (0x6d6490) et FUN_006d0940 [O] |

Le **corps de 301 et 307 est identique jusqu'à +0x25** (le parseur réutilise le même schéma de copie,
16 + 2 + 1 octets pour le nom). La chaîne `name` est copiée telle quelle (19 octets, NUL inclus) [O].

## 2. Fiches de créature : SCreatureSlotMgr [O]

Le gestionnaire (UIInstance+0x260) tient **deux structures alimentées par 301** :
- liste `mgr+0xc..+0x10` d'entrées de 0x28 o. : `{card_handle, summon_handle, name[20], code, level, sp}`
  (FUN_006d2e60 case 0x23) ;
- vecteur `slot+0x3c..+0x40` de fiches de 0x180 o. (FUN_0070ffd0) : copie du SMSG (fiche[0]), puis
  `+0x20 level`, `+0x48 sp`, `+0x4c name` (SimpleString), `+0x64 code`, `+0x68 card_handle`.
  **Clé = summon_handle** (SMSG+0x17 = paquet+0xb) : un second 301 avec le même summon_handle met la fiche à jour.
- `slot+0x1c..+0x33` : `card_handle[6]` de la **formation** (303) ; FUN_007108f0(i) les rend, FUN_00710820(i)
  rend le summon_handle de la fiche dont card_handle = formation[i].
- `slot+0x8` : **carte sélectionnée** ; `slot+0xc` / `+0x10` : son summon_handle (FUN_00711bc0, SIMSG kind 0x41a).
- `slot+0x34`, `slot+0x38` : deux emplacements « invoqué » ; FUN_00710a00 rend leur nombre (0..2) [O] ;
  écrivain non identifié [?].

Mises à jour appliquées **par summon_handle** (FUN_00710be0 = « handle connu dans les fiches ») dans
FUN_006d2e60 [O] ; ces paquets n'exigent donc pas que la créature soit invoquée :

| Paquet | ID | Champs utilisés pour une créature | Fonction |
|---|---|---|---|
| TS_SC_LEVEL_UPDATE | 1002 | +7 handle ; +0xb level (fiche+0x20, puis recalcul EXP) ; +0xf job_level ignoré | FUN_00711250 → FUN_00710d30 |
| TS_SC_EXP_UPDATE | 1003 | +7 handle ; +0xb i64 exp ; +0x13 i64 jp | FUN_00710ef0 / FUN_007111f0 |
| TS_SC_HPMP | 509 | +7 handle ; +0xf hp (fiche+0x30) ; +0x13 max_hp ; +0x1b mp ; +0x1f max_mp | FUN_00710f50 / FUN_00711300 / FUN_007112b0 / FUN_00711350 |
| TS_SC_SP | 514 | +7 handle ; +0xb i16 sp | FUN_00711400 |
| TS_SC_ITEM_WEAR_INFO | 287 | +7 item_handle ; +0xb wear_position ; +0xd target_handle ≠ joueur → équipement de créature | FUN_00711120 |
| TS_SC_STAT_INFO | 1000 | appliqué aux données de l'objet `handle` (FUN_00940f90, tout type) ; fenêtres créature 0x31/0x32 seulement si FUN_00710ab0 : fiche trouvée **et** card_handle présent dans la formation | voir `../SFRAME_STAT_ROUTING_RE.md` |

Barre d'EXP (FUN_00710d30) : `base = SummonExp(level−1)`, `next = SummonExp(level)`,
fiche+0x50 = base, fiche+0x58 = next − base, fiche+0x60 = exp − base (§8) [O]. Le stade d'évolution `rec+0x5e`
est passé au getter mais **ignoré** (RET 8, second argument inutilisé) [O].

## 3. TS_SC_ENTER (3) d'une invocation — obj_type 4, 96 octets [O]

Aiguillage : `type` (+7) = **1** et `obj_type` (+0x19) = **4** → FUN_006a9430 (`network_protocol.md` §8.1).
Taille copiée par le parseur : 96 (pet obj_type 7 : 95). Layout confirmé par deux sources indépendantes :
les lectures de FUN_006a9430 et le **SMSG_ENTER synthétique** que FUN_006ae3a0 fabrique (0x6c o. = 0x60 + 0xc,
`obj_type` 4, nom `"summon"`) pour la prévisualisation locale.

| Offset | Type | Champ | Lecture / effet |
|---|---|---|---|
| +0x07 | u8 | type = 1 | |
| +0x08 | u32 | handle (= summon_handle) | clé objet ; `[mgr+0xf4]->vtbl+0x1a4(handle)` ≠ 0 → **créature du joueur local** |
| +0x0c / +0x10 / +0x14 | f32 | x, y, z | z recalculé sur le terrain |
| +0x18 | u8 | layer | obj+0x50 |
| +0x19 | u8 | obj_type = 4 | FUN_0095d510 |
| +0x1a..+0x3f | bloc commun (38 o.) | status, face (+0x1e → obj[7]), hp (+0x22 → vtbl+0x208), max_hp, mp, max_mp, level (+0x32), race, skin_color, is_first_enter, energy (+0x3c, > 0 → FUN_00959280) | FUN_009597e0 (`network_protocol.md` §8.2) |
| +0x40 | u32 | **master_handle** (maître) | autre créature : vtbl+0x22c(master) ; puis rattachement au maître (ci-dessous) |
| +0x44 | EncodedId (8 o.) | **code** = `db_creature.id` | décodé (lo = w3+2(w0+w2), hi = w1+2(w0−w2)) et cherché dans la base créature (FUN_006acb90) |
| +0x4c | char[19] | name | FUN_0095d4d0 (nom affiché) |
| +0x5f | u8 | ? | 3ᵉ argument du ctor SGameCreature (FUN_00963e10 → octet obj+0x1214) ; sens [?], **mettre 0** [D] |

Effets [O] :
- Créature **du joueur** (fiche connue) : objet `SGameLocalCreature` (0x1268 o., FUN_00969b90), niveau pris dans la
  fiche (`FUN_0095cfc0(fiche+0x20)`), IHM `refresh_durability`, message système 0x166 avec le nom.
  Sinon `SGameOtherCreature` (0x1224 o., FUN_00974ed0).
- Champs de la base appliqués : `rec+0x42` → vtbl+0x31c ; `rec+0x4e == 1` → FUN_0094ecd0 ; vitesses `rec+0x31` /
  `rec+0x35` (÷ 7) et échelle `rec+0x29` dans le ctor (FUN_00963e10).
- Maître : si `master_handle` ≠ 0 et l'objet maître existe (FUN_006b6440), le client appelle
  `maître->vtbl+0x220(summon_handle, 1)` ; si le maître est monté (état 0x11/0x12 ou mode 4) → FUN_006ab3f0. Sinon
  la paire `(master_handle, summon)` est mise en attente (vecteur monde+0xdc) jusqu'à l'ENTER du maître.
- Si le handle existe déjà, l'ENTER summon **met à jour** l'objet au lieu d'être ignoré (`network_protocol.md` §8.1).
- Le nom affiché vient du paquet (+0x4c) ; la base fournit modèle, icône, vitesses.

## 4. Lien carte ↔ créature [O sauf mention]

- La carte est un objet d'inventaire normal (TS_ITEM_INFO, 331 o.) dont l'ItemResource a `type` (+0xc) = **13**
  (testé `== 0xd` par l'icône FUN_0081f940 et l'info-bulle FUN_00783360).
- **`summon_code` (ITEM_INFO+0x13d, objet item+0x145) = `db_creature.id`** de la créature portée par la carte :
  l'icône de la carte l'encode et cherche la créature (FUN_006acb90) pour dessiner visage (`rec+0x186`), cadre de
  rang (`rec+0x20`, `game_panel_creaturecard_rank_0..5`, avec ITEM_INFO+0x20 enhance et +0x18), marque
  d'évolution (`rec+0x5e` → `game_mark_creature_evolution_01..03`).
- L'info-bulle suit la chaîne `rec+0x62` (forme suivante) pour lister les noms d'évolution (`rec+0x8`).
- `own_summon_handle` (ITEM_INFO+0x143) ≠ 0 indique un **objet porté par une créature** (équipement de créature),
  pas une carte (`network_protocol.md` §9.2) [O] ; l'équipement porté est signalé par 287 avec target_handle.
- La relation carte → créature côté réseau est **301** : `card_handle` = handle de l'item carte, `summon_handle`
  = handle réservé de la créature, `code` = même id que `summon_code` [D : cohérence attendue, non vérifiée par
  le client].
- **303 = formation** : 6 card_handle (pas des summon_handle) ; le slot 0 est la créature « principale » de
  l'IHM `formation_main`, 1..5 `formation_sub00..04` [D d'après les noms de contrôles].
- `rec+0x286` (card_id) = id d'ItemResource de la carte vierge associée à la créature (lookup FUN_00939600) [O
  lookup, D sens].

## 5. Invoquer / renvoyer : skills 4001 et 4002 [O sauf mention]

Aucun constructeur de TS_CS_SUMMON (304) n'existe : balayage des immédiats `0x130` (`mov r32`, `mov m16`,
`mov m32` avec id+checksum, `push`) sans résultat hors faux positifs (`cmp [reg+0x130]`). L'invocation est une
**compétence** :

| Skill | Nom (db_string) | delay_cast | Coût | Cible |
|---|---|---|---|---|
| 4001 | « Invocation de Créature » | 8,8 s − 0,8 s/niveau | MP 60 + 20/niveau | carte sélectionnée |
| 4002 | « Rappel de Créature » | 0,5 s | MP 5 | carte sélectionnée |

(valeurs lues dans `Resource\db_skill.rdb` avec `tools/rdb_dump.js` [O mesure]).

Chemin client :
1. La sélection d'une carte (fenêtre formation / invocation simple) émet SIMSG kind 0x41a → FUN_00711bc0 :
   `slot+8 = card_handle`, `slot+0xc = slot+0x10 = summon_handle` de la fiche.
2. FUN_0079c590 (action de compétence) : anti-répétition (≥ 20 unités d'horloge client depuis la dernière action) ;
   pour 4001 exige `FUN_00710a00() < 3` ; construit la requête `{skill_id, skill_level, target = slot+8 (card_handle),
   caster = handle du joueur}`.
3. FUN_006d1cc0 : skill inconnu → message 0x184 ; **4001/4002 avec target = 0 → message 0x17a** (aucune carte) ;
   puis FUN_006d8d40 : pour 4001/4002, `FUN_00968560(skill_id, level, target = card_handle, …)` (pas de choix de
   cible automatique) → file d'actions du joueur (vtbl+0x138).
4. Envoi **TS_CS_SKILL (400, 31 o.)** `skill_id = 4001|4002`, `target = card_handle` [D : dernier maillon
   FUN_0096b840 non retracé pour ces deux ids].

Réponse serveur attendue [D] : TS_SC_SKILL (401) incantation puis tir ; pour 4001, ENTER (obj_type 4, `handle` =
summon_handle de la fiche, `master_handle` = joueur) ; pour 4002, UNSUMMON (305) puis LEAVE (9) [D].

## 6. Monture [O sauf mention]

- Commandes de discussion `/mount_creature` et `/unmount_creature` (chaînes 0xd62c40 / 0xd62c50) ; chemin
  d'envoi non suivi [?].
- 320 : si cavalier **et** créature existent → FUN_006ac060 (lien), et si `success` : les deux objets sont posés en
  `(x, y, z_terrain)` (FUN_00a69100, état 0x8c), animation vtbl+0x178, puis vtbl+0x284(msg) ; si l'un manque →
  FUN_006ab3f0(cavalier, créature, 1) (lien différé). Joueur local + success → message 0x168.
- 321 : lien défait (FUN_006ac060), animation vtbl+0x180 sauf mode particulier, vtbl+0x284 ; objets absents →
  FUN_006ab3f0(…, 0). Joueur local → message 0x169.

## 7. db_creature.rdb — 942 octets [O]

Chargeur **FUN_00660a40** : `FUN_005c01c0("db_creature.rdb")`, lecture de 0x84 (en-tête + count), puis par
enregistrement `read(rec, 0x3ae)` dans une structure de 0x3bc o. Fichier (archive data.003) :
**3 337 638 o., count 3 543 → (3 337 638 − 132) / 3 543 = 942 = 0x3ae** [O mesure].
Traitements au chargement [O] :
- filtre région : enregistrement **ignoré si `rec+0x3aa & code_pays ≠ 0`** (FUN_0098d5c0) ;
- `".jpg"` ajouté à la chaîne `rec+0x28a` ;
- clé = `rec+0x0` encodée (masques aléatoires en +0x3ae / +0x3b2, valeur en +0x3b0 / +0x3b4) ;
- après chargement, pour chaque `rec+0x62 > 0`, la cible reçoit `mem+0x3b8 = id` (forme précédente, en mémoire
  seulement).

| Off. | Type | Champ | Preuve / valeurs mesurées |
|---|---|---|---|
| 0x000 | i32 | **id** (= code de créature, clé) | [O] ; 60..10560, unique |
| 0x004 | i32 | model_id | [D] ; 333 valeurs |
| 0x008 | i32 | **name_id** | [O] texte affiché (info-bulle, évolutions) ; = id + 20000 pour 2 982/3 543 |
| 0x00c | i32 | ? | [?] 0..9 |
| 0x010 | i32 | type (clé de la table d'icônes FUN_0064ee80) | [O lecture, D sens] 0..20 |
| 0x014 | i32 | **species_id** | [O] même usage que `db_monster.species_id` (+0x34) dans la cible (FUN_0093abc0) ; 1..160 |
| 0x018 | i32 | ? | [?] 0..13 |
| 0x01c | i32 | ? | [O lecture FUN_006ad2d0, ? sens] 0..6 |
| 0x020 | u8 | **rank** (grade de carte) | [O] cadres `rank_0..5` ; 0..5 |
| 0x021 | i32 | stat_id | [D] = id pour 2 915/3 543 |
| 0x025 | f32 | size | [O lecture FUN_00964530, D sens] |
| 0x029 | f32 | **scale** | [O] FUN_00963e10 |
| 0x02d | f32 | target_fx_size | [O lecture FUN_0094e9d0, D sens] |
| 0x031 | i32 | **walk_speed** (÷ 7) | [O] FUN_00963e10 ; 40 le plus souvent |
| 0x035 | i32 | **run_speed** (÷ 7) | [O] FUN_00963e10 ; 80 le plus souvent |
| 0x039 / 0x03d | i32 | vitesses de monture ? | [D] 50 / 120 le plus souvent |
| 0x041 | i8 | is_riding_only ? | [O] fiche carte FUN_0075d920 n'affiche rien si < 0 ou == 1 ; 0:3507, 1:23, 2:13 |
| 0x042 | i32 | riding_motion_type ? | [O] → vtbl+0x31c à l'ENTER ; 0..6 |
| 0x046 | i32 | attack_range ? | [D] 20..2000 |
| 0x04a | i32 | walk_type ? | [O lecture FUN_00954810] |
| 0x04e | i32 | slant_type | [O] == 1 → FUN_0094ecd0 ; 0/1 |
| 0x052 | i32 | material | [O lecture FUN_00954730] |
| 0x056 | i32 | weapon_type | [O lecture FUN_009545b0] ; 101 / 106 / 111 |
| 0x05a | i32 | attack_motion_speed | [O] FUN_00953890 : 0 → −50 %, 1 → −40 %, 2 → −30 %, sinon inchangé |
| 0x05e | i32 | **form** (stade d'évolution 1..3) | [O] marques d'évolution ; id du stade 1 = id − form + 1 (FUN_0075d920) |
| 0x062 | i32 | **evolve_target** (id de la forme suivante, 0 = aucune) | [O] ; 2 310 > 0, toutes existantes |
| 0x066 / 0x06a / 0x06e | i32 | camera_x / y / z | [D] |
| 0x072 / 0x076 / 0x07a | f32 | target_x / y / z | [D] (−0,3 / −0,06 / 14,86 partout) |
| 0x07e | char[256] | **model** | [O] FUN_009539e0 |
| 0x17e | i32 | motion_file_id | [O lecture FUN_009530c0] |
| 0x182 | i32 | face_id | [D] toujours 0 |
| 0x186 | char[256] | **face_file_name** (icône) | [O] icônes de carte |
| 0x286 | i32 | **card_id** (ItemResource de la carte) | [O lookup FUN_00939600] ; 540200..540203 surtout |
| 0x28a | char[256] | **illust_file_name** (+ « .jpg ») | [O] FUN_0075b5a0 |
| 0x38a | i32 | text_feature_id ? | [D] 3002601… (ids de texte) |
| 0x38e | i32 | ? | [?] toujours 0 |
| 0x392..0x3a2 | i32 ×5 | ? (liste d'ids) | [?] 46..137 surtout, 0x3a2 toujours 0 |
| 0x3a6 | i32 | texture_group | [O lecture FUN_00965fc0] ; −1 le plus souvent |
| 0x3aa | u32 | **local_flag** (masque de région) | [O] filtre au chargement ; 0 partout |

Les noms marqués « ? » reprennent l'ordre usuel des colonnes `SummonResource` (serveurs epic) ; seuls les
champs [O] ont un usage client établi.

## 8. db_summonexp.rdb — 24 octets [O]

Quatre chargeurs identiques (FUN_00647c90, FUN_006482f0 singleton, FUN_00648510, FUN_006486b0) :
`read(rec, 0x18)` par enregistrement. Fichier (data.007) : **4 932 o., count 200 → 24** [O mesure].

| Off. | Type | Champ | Preuve |
|---|---|---|---|
| 0x00 | i64 | **exp cumulée** pour passer du niveau `i+1` au niveau `i+2` (enregistrement i) | [O] FUN_00648260(n) = rec[n−1].exp ; FUN_006481b0(exp) = premier i tel que rec[i] ≤ exp < rec[i+1], niveau = i + 2 |
| 0x08 | i64 | ? | [?] non lu par le client ; 0 à partir de l'enregistrement 80 |
| 0x10 | i64 | ? | [?] non lu par le client ; 0 à partir de l'enregistrement 100 |

Exemples : rec[0] = (5, 1068, 21798), rec[1] = (17, 2808, 49728), rec[199] = (57 566 988 928 792, 0, 0).
La ferme à créatures (FUN_0088ee30) plafonne le niveau affiché par stade : **60 / 115 / 150** pour form 1 / 2 / 3 [O].

Autres tables mesurées (non analysées côté client) [O mesure] : db_creatureenhance 6 × 12, db_creaturefarm
108 × 4, db_monster_creature 396 × 48, db_pet 160 × 344, db_summonrandomskill 152 × 68 (chargeur
`SSummonRandomSkillDB`).

## 9. Séquences [D sauf ids, tailles et formats marqués O]

- **Connexion avec cartes** : … TS_SC_INVENTORY (207) contenant les cartes (type 13, `summon_code` = id créature)
  → pour chaque carte liée : **301** `card_handle, summon_handle, name, code, level, sp` → STAT_INFO (1000),
  HPMP (509), LEVEL_UPDATE (1002), EXP_UPDATE (1003) avec `handle = summon_handle` → **303** formation (6
  card_handle, 0 = vide). Le client n'a besoin d'aucun ENTER tant que rien n'est invoqué.
- **Ouvrir la formation** : C→S 324 (+7 = 1 si la fenêtre n'est pas ouverte) → S→C 303 (open_dialog). Modifier :
  C→S 303 (6 card_handle) → S→C 303 confirmé.
- **Invoquer** : sélection de carte (local) → C→S TS_CS_SKILL 4001, target = card_handle → S→C TS_SC_SKILL
  (incantation 8,8 − 0,8·lv s, puis tir) → **ENTER obj_type 4** (`handle` = summon_handle, `master_handle` =
  joueur, `code` encodé, `name`) → STAT_INFO / HPMP pour summon_handle.
- **Renvoyer** : C→S TS_CS_SKILL 4002, target = card_handle → S→C SKILL → **305** summon_handle → LEAVE (9).
  Le serveur peut prévenir par 306 (summon_handle, délai) avant un renvoi forcé.
- **Monture** : `/mount_creature` → S→C **320** (cavalier, créature, x, y, success = 1) ; descente → **321**.
- **Montée de niveau** : EXP_UPDATE (1003) puis LEVEL_UPDATE (1002) avec summon_handle ; barre d'EXP recalculée
  depuis db_summonexp (§8). **Évolution** : S→C **307** (card_handle, summon_handle, nouveau name/code) ;
  envoyer aussi 301 avec le nouveau code pour mettre la fiche à jour, et un nouvel ENTER si invoquée.
- **Renommer** : S→C 322 (handle) → C→S 323 (`name[19]`) → S→C 301 avec le nouveau nom.
- **Suppression d'une carte** : S→C 302 (card_handle) puis retrait d'inventaire.

## 10. Limites / inconnues

- **TS_CS_SUMMON (304)** : absent du client (recherche par motifs d'immédiats, non exhaustive pour une construction
  indirecte). Le dernier maillon « action → TS_CS_SKILL » pour 4001/4002 n'a pas été retracé jusqu'à FUN_0096b840.
- Écrivain de `slot+0x34` / `+0x38` (créatures invoquées, max 2) et usage du test `FUN_00710a00() < 3` : non trouvés.
- Consommateurs de 306 (kind 0x28), 322 (kind 0x67) et de l'octet `open_dialog` de 303 : non identifiés (IHM).
  Mise à jour de la fiche par 307 non localisée.
- Paquet 311 (hors table TM_) : seule l'écriture `[DAT_00ed0998]+0x11f4` est observée.
- ENTER +0x5f (octet du ctor SGameCreature) et `db_creature` +0x0c, +0x18, +0x1c, +0x392..+0x3a2 : sens inconnu.
- db_summonexp colonnes +0x08 / +0x10 : jamais lues par le client.
- TS_SC_TAMING_INFO : correspondance des modes 1/2/3 avec TAMING_START/SUCCESS/FAILED non établie.
- Tailles S→C = étendue lue par le parseur ; un serveur officiel peut envoyer des octets supplémentaires ignorés.
- Pets (350–354, ENTER obj_type 7 de 95 o.) : seulement effleurés (FUN_006aab90 lit master +0x40, code +0x44,
  name +0x4c, sans l'octet +0x5f).

## 11. Adresses

| Adresse | Rôle |
|---|---|
| 0xa2fed0 | parseur S→C (cases 0x12d–0x142) |
| 0x6d2e60 | gestionnaire jeu : kinds 0x23/0x24/0x25 (301/302/303), 0x3a (324), 0x419 (303 C→S), 0x41a, 0x46d (323), 0x4e1 (452), 0x21/0x22/0x37/0x48/0x12 (mises à jour de fiche), 0x4d/0x4e (messages monture) |
| 0x6b2130 | monde : kinds 0x27 (305), 0x29 (307), 0x2e (310), 0x4d → 0x6afa10, 0x4e → 0x6b0e70 |
| 0x6a9430 / 0x6aab90 | ENTER summon / pet |
| 0x6ae3a0 | ENTER synthétique (prévisualisation d'invocation) |
| 0x969b90 / 0x974ed0 / 0x963e10 | ctors SGameLocalCreature / SGameOtherCreature / SGameCreature |
| 0x70ffd0 / 0x7102f0 / 0x711b30 / 0x711bc0 | fiche : ajout (301) / retrait (302) / formation (303) / carte sélectionnée |
| 0x710be0 / 0x710ab0 / 0x710a00 / 0x710820 / 0x7108f0 | fiche connue / fiche en formation / nb invoquées / summon_handle du slot i / card_handle du slot i |
| 0x711250 / 0x710ef0 / 0x7111f0 / 0x710f50 / 0x711400 / 0x711120 | level / exp / jp / hp / sp / équipement de créature |
| 0x710d30 | barre d'EXP de créature (db_summonexp) |
| 0x79c590 / 0x6d1cc0 / 0x6d8d40 | action de skill (4001/4002 → carte sélectionnée) / contrôles / construction de l'action |
| 0x6c9420 / 0x6d09a0 / 0x6d0940 | envois 303 / 323 / 452 |
| 0x660a40 / 0x6608a0 / 0x6acb90 | chargeur db_creature / singleton / lookup par EncodedId |
| 0x647c90 / 0x6482f0 / 0x648510 / 0x6486b0 | chargeurs db_SummonExp |
| 0x648260 / 0x6481b0 | SummonExp : exp(n) / niveau depuis exp |
| 0x81f940 / 0x81d390 / 0x81d530 / 0x783360 | icône de carte / cadre de rang / marque d'évolution / info-bulle de carte |
| 0x88ee30 | ferme à créatures (plafonds 60/115/150) |
| 0x773cf0 | fenêtre « simple summon » (6 slots) |

## 12. Implémentation RappelzCore (2026-11-24) — socle

Choix du serveur, **[D]** sauf formats [O] ci-dessus :

| Élément | Choix |
|---|---|
| Données | `db_creature.rdb` → `Creature.dbc` (48 colonnes, 942 octets, schéma de `tools/rdb_dump.js`) ; `rz_rappelz::creatures::CreatureBook` |
| Contrôle du schéma | fin de lecture = fin du fichier ; 3 543 `name_id` sur 3 543 résolus dans `String.dbc` ; 3 485 `card_id` > 0, tous des objets existants ; 2 310 `evolve_target`, tous égaux à id + 1 et sur la même carte |
| Carte → créature | `Item.summon_id` des objets du groupe 13 (132 cartes sur 138 ; 6 vierges). `card_id` ne suffit pas : 53 cartes pour 3 485 créatures, dont 4 cartes génériques qui en couvrent 3 318 |
| Persistance | table AzerothCore `character_pet` inchangée : `id` = guid de l'objet carte, `entry` = id de créature, `slot` = place de formation (0..5) ou 100 ; ligne créée au niveau 1 à la première possession |
| Handles | 0x3000_0000 + guid de la carte |
| Connexion | inventaire (`summon_code` rempli), puis par carte 301, 1002, 1003, 509, puis 303 |
| Formation | 303 reçu : cartes du sac, connues et sans doublon ; confirmé par 303 (la formation en vigueur si la demande est refusée) ; 324 → 303 |
| Invoquer / renvoyer | compétences 4001 / 4002 **apprises**, carte dans le sac, PM (60 + 20/niveau, 5) ; une créature par joueur ; réponse TS_SC_RESULT(400) ; renvoi : 305 puis LEAVE |
| Suivi | rejoint le maître au-delà de 40, s'arrête à 20 ; téléportation, changement de couche ou écart > 300 : réapparition près du maître ; départ du maître : renvoi |
| Échange | la créature suit sa carte (nouveau propriétaire, hors formation) ; carte d'une créature invoquée : ni échange ni entrepôt |
| Pertes évitées | carte liée : ni vente ni dépôt au sol (l'objet y perdrait son guid) ; la destruction volontaire de la carte supprime la ligne |
| GM | `.learn <id> [niveau]`, comme AzerothCore |

Validation (MariaDB de test isolée, `rz-bot --summon` / `--unsummon`) : carte 540035 → fiche « Yéti » (1801),
formation enregistrée, ENTER de type 4, renvoi (305 puis LEAVE) ; dépôt au sol refusé ; échange de la carte entre
deux joueurs : 302 chez l'un, 301 chez l'autre, `character_pet.owner` modifié. **Non vérifié : client graphique
d'origine.**

Restent à faire : combat et EXP des créatures, statistiques réelles (PV/PM provisoires tirés du niveau, pleins à
chaque invocation), évolution (307), renommage (322/323), monture (320/321), équipement de créature, apprivoisement
(310), familiers (350–354), double invocation. L'incantation et TS_SC_SKILL de 4001/4002 sont faits :
`skill_protocol.md` §14.

## 13. Créature pilotée par le client — combat (2026-11-26)

### 13.1 Constats (sous-agent ; envoi d'attaque relu en FUN_0096b6c0, `ev_c95175f56f039cf256ddda1cce5c6d35fab882d5f97e2a0f64508c919b9c85ec`)

- **La créature locale a sa propre IA côté client** (SGameLocalCreature, vtable 0xd61844 ; machine SLocalCreatureStateMachine,
  vtable 0xd60004 ; script `CreatureState_Script.lua` appelé toutes les 1 000 ms). Elle envoie elle-même, **avec son propre handle** :

| Paquet | Taille | Corps |
|---|---|---|
| TS_CS_ATTACK_REQUEST (100) | 15 | +7 handle de la créature [O relu] ; +0xb cible |
| TS_CS_MOVE_REQUEST | — | handle de la créature (constructeur 0x9efe50) ; suivi derrière le maître (0x96bbf0) |
| TS_CS_SKILL (400) | 31 | +9 caster = handle de la créature ; fiche db_skillstage exigée, sinon rien n'est envoyé |
| TS_CS_CANCEL_ACTION (150) | 11 | +7 handle de la créature ; envoyé aussi après TS_SC_CANT_ATTACK, sauf raisons 5 et 6 |
| TS_CS_UPDATE (503) | 19 | +7 handle de la créature, toutes les 4 000 ms |

- Ordres du joueur (FUN_006d0eb0) : 0x1d « attaque créature » (sélection, sinon cible automatique à 360), 0x1f « joueur et créatures »,
  0x1e « rappel ». Aucun ordre « rester » trouvé. Aucune riposte autonome : l'attaque reprend après un ordre ou un ATTACK_EVENT.
- Réception : ATTACK_EVENT et SKILL dont l'attaquant ou la cible est la créature sont traités comme pour un monstre ; handles inconnus →
  TS_CS_QUERY. Kinds de fiche (FUN_006d2e60) : 0x21 = 1002, 0x22 = 1003, 0x37 = 509, 0x48 = 514 (SP), 0x12 = 287, avec le handle de la
  créature, sans ENTER. 403 avec target = créature remplit sa liste de compétences.
- Mort : PV < 1 → animation ; **aucun renvoi automatique côté client** ; au bout de 5 s, boîte de résurrection seulement si un état de
  type 109 l'autorise.

### 13.2 Défauts corrigés

L'ATTACK_REQUEST et le CANCEL_ACTION d'une créature étaient appliqués **au joueur** (le champ handle était ignoré) ; les déplacements de
la créature étaient rejetés.

### 13.3 Implémentation RappelzCore

| Élément | Choix |
|---|---|
| Commandes | ATTACK_REQUEST, MOVE_REQUEST et CANCEL_ACTION avec le handle d'une créature : acceptés pour la créature **du joueur**, vivante ; départ d'un déplacement à 60 au plus de la position serveur |
| Conduite | le premier déplacement envoyé par le client désactive le suivi et la poursuite par le serveur (gardés pour un client sans IA de créature, comme `rz-bot`) |
| Coups | règle provisoire du joueur : attaque et défense selon le niveau, ± 10 %, un coup toutes les 1,5 s, états compris ; portée `attack_range` de db_creature × 12 / 100, bornée à 20..80 **[D]** |
| Riposte | le monstre frappé vise la créature ; ses coups lui retirent des PV (509 aussi envoyé au maître pour la fiche) |
| Mort | plus d'attaque ni de cible ; renvoi (305 puis LEAVE) au bout de 5 s **[D]** ; PV pleins à la prochaine invocation |
| Récompense | au maître **[D]** ; **la créature ne gagne pas encore d'EXP** |
| Compétences de créature | refusées (TS_SC_RESULT 400, code non nul : la machine du client se libère) |

Validation (MariaDB de test isolée, `rz-bot --summon --summon-attack`) : Yéti niveau 1 invoqué, attaque avec son handle, deux coups de 5,
riposte du monstre (52 puis 49 : PV 100 → 48 → 0), renvoi 5 s plus tard ; le joueur ne part pas au combat. Tests unitaires : ordres
d'un autre joueur ou contre une cible invalide refusés, riposte, mort, renvoi, déplacement du client sans suivi imposé.
**Non vérifié : client graphique d'origine** (son IA de créature n'a jamais tourné contre ce serveur).

# Protocole des quêtes — client Rappelz NoLimits (SFrame.exe)

Analyse **statique** (Ghidra 12.1.4 via REA) de `Rappelz NoLimits\SFrame.exe`, PE x86,
SHA-256 `27f42fc8f6b08df8c239640b938d601aeaebb7b3db0c89fc47755325a98a410d`. SFrame.exe n'a jamais été exécuté.
Complète `network_protocol.md` §9 (dont le style et les conventions sont repris).

Conventions :
- **Offsets = offsets paquet** (en-tête TS_MESSAGE de 7 octets inclus : `u32 size, u16 id, u8 checksum`).
- Dans les objets internes `SMSG_*` / `SIMSG_*`, le « kind » est à objet+4 ; les champs utiles commencent à +0x13.
- `[O]` = lu dans le code (adresse citée) ; `[D]` = déduit (nom, sens, ou comportement non observé directement).
- `e+xx` = offset dans une entrée de la liste de quêtes du client (`SQuestMgr`, 0x48 octets, voir §2).
- `res+xx` = offset dans l'enregistrement QuestResource (db_quest.rdb) tel que lu par le client (§5).

Evidence IDs REA principaux (pour retrouver les sorties) :

| Contenu | Evidence ID |
|---|---|
| FUN_00a29430 (parse TS_SC_QUEST_LIST) | `ev_24311e55bd86b7c46335622387e4ffc8db7a86bb201d8157f206b2d47a6954c7` |
| FUN_00a29780 (parse TS_SC_QUEST_STATUS) | `ev_9f00b511d01818d4e82a0285a8a2b07c8d08451aef7893e025829dca93ebc3cc` |
| FUN_00709110 (ajout d'entrée) | `ev_f8f8e332f464757f4cb544f2df5fdea030d10cbad696145e701c69b75ca97f4c` |
| FUN_00708b50 (mise à jour statut) | `ev_e361c7aa78afb9edfce040f33f01e172dcaac54d98ea95ede0759401e1d616ef` |
| FUN_00709200 (quête présente ?) | `ev_e6f0e34189eef9ad0016dde58315a1f3347620b26dba400dbc4b6c109ab33e06` |
| FUN_00708c50 (singleton SQuestMgr) | `ev_bc8e1caade01935da57f4179fbe18e132c6600b5e3ceb3dea847e4b9050db6b4` |
| FUN_006d2e60 (gestionnaire jeu, envois 603/604/605/3001) | `ev_fcbea5235e2a5efd24c35551e8aef307f4d76cbd22393a2b6ddcc8f4c383e1b3` |
| FUN_006cd120 (envoi 605 isolé) / ses xrefs (vides) | `ev_c4c755c784e01de57a807cb97094c1639e5b520707227561f2eac0ff184f8e43` / `ev_0e2113ba971ee545415f157e0f3e51bff9f4c9be940388aaf1a7a630d2be2490` |
| Lot : appelants de SQuestMgr (FUN_009d2130 msgbox, FUN_008b7b50 liste, FUN_008b5d80, FUN_008b2ba0, FUN_008b8510, FUN_00a2da30, FUN_00a653b0…) | `ev_6909078b2424bbd109ca2cac2886028f427fab553bbcec2ce6a53bb96b6c074b` |
| Lot : FUN_00720ef0, FUN_007e60f0 (fenêtre PNJ), FUN_008b2650, FUN_008bc860, FUN_008b69b0, FUN_00a25670 (TS_SC_DIALOG) | `ev_e41733cdefc710a86c01f5e994c567e1db15de5edf73af0b611c4a4431cc7324` |
| Lot : FUN_007ca300, FUN_008b1c80, FUN_006682e0, FUN_0065edf0, FUN_00708e80/e20/de0, FUN_0070b610 | `ev_c6112384fdc552565ee92a1a72158ce6d8d96d8ed41a4361b3376a45063b56ca` |
| Lot : FUN_00708ef0, FUN_00708fd0, FUN_00709040, FUN_00709260, FUN_0070a400, FUN_0070a450 | `ev_71aea3f71ca5e5b640ff4e4e348dd52eed54678d35eae0ede59a401560bfb741` |
| Lot : FUN_00a654b0 (macros de texte), FUN_00709dd0, FUN_00709e30 | `ev_bba2495591034544e00636a76b1189cf8d8fa345ab2d716ae83fb8382359e1d9` |
| Lot : FUN_0070a6b0 (objectifs atteints), FUN_0070a2f0 | `ev_c77810a4a7dbf8aa2b3a4835e49fea8cc4bb1d3563a8f34bd6fbd500289e6eed` |
| Lot : FUN_008bc740 / FUN_008b7680 (« select_reward »), FUN_00c7264b | `ev_1b92a73ba65c0a3c915a4283fda216f90d8e7b49ad7db8e7044e6dd2935ed14f` |
| FUN_006ce940 (consommateur SMSG_QUEST_UPDATA) | `ev_6e4ae0edd2b7c06738adc935a607f32e7d6d1d58e09fce05846f54843eb84e6a` |
| FUN_00613c30 (horloge client) | `ev_6b626bd86751e34426946a40ac95fd5dbd452238cb19e2817122dc773d3a261d` |
| Chaînes « quest » / « #@…@# » | `ev_65ffb9cae3946370107b1996a0af9a2dc233447c0c42cc693e4bd3ecc0857602` / `ev_4e9a9c2a58f4fec2b633dbf17af31bbaf7ff6add716329d29f9fdd7f7815d820` |
| Table msgbox (index 0x11 → « msgboxGiveupQuest ») | `ev_ccf5060d93c90a0a86d6eadb9332214b060d8988745aa874d9cf6a34e37eb799` |

## 1. Vue d'ensemble des paquets

| Paquet | ID | Dir | Taille | Corps (offsets paquet) | Fonction / déclencheur |
|---|---|---|---|---|---|
| TS_SC_QUEST_LIST | 600 | S→C | 11 + 61·a + 8·p | +7 u16 a ; +9 u16 p ; +0xb QuestInfo[a] (61 o., §2) ; puis {i32 code, i32 start_id}[p] | FUN_00a29430 [O] — **remplace toute la liste** |
| TS_SC_QUEST_STATUS | 601 | S→C | **36** | +7 i32 code ; +0xb i32 status[6] ; +0x23 u8 progress | FUN_00a29780 → SMSG_QUEST_UPDATA (kind 0x6c) [O] |
| TS_SC_QUEST_INFOMATION | 602 | S→C | — | **non parsé** par FUN_00a2fed0 | aucun [O] |
| TS_CS_DROP_QUEST | 603 | C→S | 11 | +7 i32 code | FUN_006d2e60 case kind 0x3a (« drop_quest ») [O] |
| TS_CS_QUEST_INFO | 604 | C→S | 11 | +7 i32 code | FUN_006d2e60 case kind 0xa5 (SIMSG_QUEST_INFO_REQUEST) [O] |
| TS_CS_END_QUEST | 605 | C→S | 12 | +7 i32 code ; +0xb u8 flag [D sens] | FUN_006d2e60 case kind 0xb4 et FUN_006cd120 — **aucun déclencheur trouvé** [O] |
| TS_CS_DIALOG (accepter / rendre) | 3001 | C→S | 9 + n | +7 u16 n ; +9 trigger[n] (sans NUL) — `start_quest…` / `end_quest( %d, %d )` | FUN_006d2e60 case kind 0x40f [O] |
| TS_SC_DIALOG (proposer / rendre) | 3000 | S→C | 21+t+x+m | voir `network_protocol.md` §9.3 ; texte `QUEST|code|text_id`, menus `QUEST|…`, START/REJECT/OK/END/REWARD | FUN_00a25670 [O] |

## 2. TS_SC_QUEST_LIST (600) — FUN_00a29430 [O]

Déroulé (0xa29430) :
1. `thunk_FUN_0070b2a0` mémorise les drapeaux d'IHM par code (e+0x40/e+0x44), puis **FUN_00708bd0 vide les
   listes** (end = begin) [O]. La liste n'est donc jamais fusionnée : chaque 600 décrit l'état complet.
2. Pour chaque QuestInfo actif : `FUN_00709110(code, &rec[0x20], &rec[0x08], start_id, time_limit, progress, 0)` [O].
3. Pour chaque entrée « pending » : `FUN_00709110(code, zéros, zéros, start_id, 0, 0, 1)` [O].
4. FUN_0070b1d0 restaure les drapeaux et marque « nouvelle » (0x100 en e+0x40) toute quête absente avant, sauf
   au premier envoi (DAT_00ed067c) [O mécanique, D sens « nouvelle »].
5. Trois SIMSG_UI_SEND_DATA « quest_list_updated » vers les fenêtres 0x47, 3 et 0x42 [O]
   (fenêtre liste FUN_008b7680 : rafraîchit, rouvre 0x48, envoie « set_quest_code »).

La capacité du vecteur est réservée à **20** entrées (FUN_00709c50(0x14)) et FUN_008b4440 teste `count < 20` [O] ;
limite de quêtes actives côté IHM = 20 [D].

### 2.1 QuestInfo — 61 octets (offsets relatifs à l'enregistrement)

| Off. | Type | Champ | Destination client | Preuve |
|---|---|---|---|---|
| 0x00 | i32 | **code** (id QuestResource) | e+0x00 (clé de recherche partout) | [O] |
| 0x04 | i32 | **start_id** | e+0x04 | [O copie] ; sens « id de départ (PNJ/objet) » [D], aucun lecteur trouvé |
| 0x08 | i32[6] | **value[6]** (« valeurs aléatoires ») | e+0x20..0x37 | [O] lu par FUN_00708ef0(code, i<6) et par FUN_00709e30 ; utilisé pour les macros `#@x:rN@#` (N-1) et quand `res.valueK < 0` (§4) |
| 0x20 | i32[6] | **status[6]** (compteurs de progression) | e+0x08..0x1f | [O] lu par FUN_00708fd0(code, i<6) ; remplacé par 601 |
| 0x38 | u8 | **progress** | e+0x3d | [O] lu par FUN_00708e80 ; remplacé par 601 |
| 0x39 | i32 | **time_limit** (échéance absolue) | e+0x38 | [O] lu par FUN_00709040 |

⚠ L'ordre est **value puis status** : le bloc copié dans e+0x08 (celui que 601 met à jour) est `rec+0x20`
(passé en 2ᵉ argument de FUN_00709110), le bloc `rec+0x08` va en e+0x20 [O]. La struct actuelle
`QuestInfo { code, start_id, value[6], status[6], progress, time_limit }` (`shared/rz-proto/src/msg_game.rs`)
est **correcte**.

Entrée de liste client (`SQuestMgr`, 0x48 o., FUN_00709110) : e+0 code, e+4 start_id, e+8 status[6],
e+0x20 value[6], e+0x38 time_limit, e+0x3c u8 0, e+0x3d progress, **e+0x3e is_pending** (0 actif / 1 pending),
e+0x40 u16 / e+0x44 u32 drapeaux IHM [O].

### 2.2 Entrée « pending » — 8 octets
`{ i32 code ; i32 start_id }` [O] → entrée avec status/value à 0, progress 0, time_limit 0, **e+0x3e = 1**.
Les entrées pending sont stockées dans le même vecteur que les actives [O] ; la liste (FUN_008b8510) n'affiche
les objectifs que si `FUN_00709260(code) == 0` (non pending) et `progress == 1` [O]. Sens exact de « pending »
(quête suspendue / en attente de condition) : [D].

### 2.3 Valeurs de `progress` [O valeurs, D sens]
- Marque PNJ (FUN_008b5d80, `mark_status_quest_01`) : 0 → `common_mark_npc_quest01_big`, 1 → `…quest02_big`,
  2 ou **100** → `…quest03_big`.
- Marqueurs de cible (FUN_008f3160 / FUN_008f3230) : progress 0 → rien ; **1** → marque les PNJ « en cours » de
  QuestLink pour les types 201, 301, 401, 501, 601, 701 ; **2** → marque les PNJ de fin (QuestLink+0x1c).
- Minuterie et objectifs affichés seulement pour **progress == 1** (FUN_008b2ba0, FUN_008b8510).
- Interprétation [D] : 0 = non commencée/inactive, **1 = en cours**, **2 = objectifs remplis (à rendre)**,
  100 = terminée/échue (même icône que 2).

### 2.4 `time_limit` [O formule, D base]
Affiché seulement si `res+0x44` (time_limit de QuestResource) > 0 : `reste = e+0x38 − FUN_00613c30()`,
puis `/100 % 60` = secondes, `/6000 % 60` = minutes, `/360000` = heures (FUN_008b2ba0, FUN_008b8510) [O].
FUN_00613c30 = `ms_horloge/10 + DAT_00ee9060` [O]. → **échéance absolue en centièmes de seconde** dans la base
de temps synchronisée du client (temps « ar » du serveur, offset DAT_00ee9060 réglé par la synchro) [D].
Envoyer 0 si pas de limite.

## 3. TS_SC_QUEST_STATUS (601) — FUN_00a29780 [O]

| Off. | Type | Champ |
|---|---|---|
| +0x07 | i32 | code |
| +0x0b | i32[6] | status[0..5] (copiés en 4×u32 + 1×u64 : +0xb, +0xf, +0x13, +0x17, +0x1b..+0x22) |
| +0x23 | u8 | progress |

Taille **36 octets** (dernier octet lu : +0x23) [O lecture ; aucun contrôle de taille observé dans le handler].

Déroulé :
1. Si la quête existe (FUN_00709200 : recherche du code dans le vecteur principal) : création de
   **SMSG_QUEST_UPDATA** (kind 0x6c, 0x48 o.) : +0x13 code, +0x17..+0x2e status[6], +0x47 progress,
   +0x2f chaîne = texte `res.text_id_status` (res+0xc) **formaté avec l'ancien état** (FUN_00a2ca80) ; mise en file
   (FUN_00a35160) [O].
2. **FUN_00708b50(code, &pkt[0xb], progress)** : remplace e+0x08..0x1f (status) et e+0x3d (progress) de l'entrée,
   dans le vecteur principal et dans le vecteur secondaire (+0x28) [O]. N'ajoute **jamais** de quête.
3. Deux SIMSG_UI_SEND_DATA « quest_status_updated » vers fenêtres 0x47 et 0x48 [O] (la liste FUN_008b7680
   relaie vers 0x42 et 3 et redessine).

Consommateur de SMSG_QUEST_UPDATA : FUN_006d2e60 case 0x6c → **FUN_006ce940** [O] : reformate
`res.text_id_status` avec l'état courant (nouveau, puisque l'objet est traité après l'étape 2 [D, file
locale cf. network_protocol.md §12]), découpe ancien et nouveau texte sur `<br>`/`<BR>`, et si le nombre de lignes
est identique, affiche les lignes **modifiées** en `<#00FFFC>` dans la fenêtre 0x59 (« questupdate », notice
de progression) [O].

→ Le texte de progression affiché n'est pas calculé par le client à partir de constantes : il provient de la
chaîne `text_id_status` de la QuestResource avec macros (§4).

## 4. Calcul et affichage de la progression [O sauf mention]

### 4.1 Macros de texte (FUN_00a654b0, FUN_00a2da30, FUN_00a653b0, FUN_00a65380)
Appliquées aux textes de quête (summary/status/dialogue) :

| Macro | Valeur |
|---|---|
| `#@status:N@#` | `status[N-1]` (1-indexé, borné à ≥ 0), via getter FUN_00708fd0 |
| `#@status_N=V:texte@#` | `texte` si `status[N] == V` (**index N sans −1**, tel que lu dans FUN_00a654b0) |
| `#@num:N@#` / `#@num:rN@#` | `res.value[N]` (= `res+0x48+4N`, donc N=1 → value1, N=0 → type) / `value[N-1]` aléatoire du paquet |
| `#@m_name:N@#`, `#@m_location:N@#` | nom / lieu du monstre d'id `res.value[N]` (ou `rN` → value[] paquet) |
| `#@i_name:N@#`, `#@s_name:N@#`, `#@t_name:N@#` | nom d'objet / de compétence / texte, même indexation |
| `#@quest_name@#`, `#@quest_easy@#`… | nom / difficulté |

### 4.2 Objectifs (status[i] ↔ QuestResource)
FUN_0070a6b0 calcule pour i = 0..5 un booléen « objectif atteint » :
`status[i] != 0 && status[i] == cible_i`, avec `cible_i = res.value(2i+2)` (res+0x50+8i : value2, value4, …,
value12) ; pour les types **901 / 902** la cible est `value[2i+1]` du paquet (bloc aléatoire, FUN_00708ef0) [O].

→ **status[i] = compteur courant de l'objectif i**, dont l'id cible est `value(2i+1)` (value1, value3, value5…)
et le nombre requis `value(2i+2)` (value2, value4, value6…) [O lecture ; D pour l'appariement id/cible côté serveur].

Ids cibles collectés pour les marqueurs (FUN_0070a450 / FUN_00709dd0 / FUN_0070a2f0) [O] :

| type (res+0x48) | Ids lus | Remarque |
|---|---|---|
| 101 (0x65), 102 (0x66) | value1, value3, value5 | monstres ; si valueK < 0 → valeur aléatoire value[K-1] du paquet |
| 103 (0x67) | value1, 3, 5, 7, 9, 11 | objets (FUN_0070a2f0) |
| 106 (0x6a) | value7 … value12 | monstres porteurs |
| 901 (0x385), 902 (0x386) | value[0], value[2], value[4] du **paquet** | quêtes aléatoires |
| 201, 301, 401, 501, 601, 701 | — | marqueur « PNJ en cours » si progress == 1 |

Noms des types [D, mémoire rzemu] : 101 kill total, 102 kill individuel, 103 collecte, 106 chasse d'objet,
201 compétence, 301 amélioration, 401 contact, 501 niveau de job, 601 paramètre, 701 script, 901/902 aléatoires.

## 5. Champs de QuestResource lus par le client [O]

L'enregistrement en mémoire suit l'ordre du fichier db_quest.rdb (`rdb_schemas.rs`, « quest ») :

| res+ | Champ (schéma) | Lecteur |
|---|---|---|
| 0x00 | id | clé (FUN_0065edf0, FUN_00a2da30) |
| 0x0c | text_id_status | texte de progression (FUN_00a29780, FUN_006ce940, FUN_00a2da30) |
| 0x44 | time_limit | > 0 → afficher la minuterie |
| 0x48 | type | §4 |
| 0x4c..0x78 | value1..value12 | §4 |
| 0xa4..0xeb | optional_reward 1..6 (id, level, qty) | FUN_008b1c80 : nombre d'options = dernier id ≠ 0 (res+0xe0 en arrière par 12) |
| **0xf9** | **i8 show_target_type** (1/2/3) | FUN_0070a400, FUN_00707a30 |
| 0xfc | show_target_id | idem |
| 0x100 | mark_hide (bool) | FUN_0070a400 |

⚠ Le schéma actuel place `unknown4` en 0xf9 et `show_target_type` en 0xfa ; **le client lit le type de cible
à 0xf9** [O]. À vérifier sur le fichier réel avant de corriger le schéma (aucune modification faite ici).

## 6. Dialogues PNJ de quête (TS_SC_DIALOG 3000 / TS_CS_DIALOG 3001) — FUN_00a25670 [O]

### 6.1 Texte
Si le texte commence par `QUEST|` (FUN_00c96800 = strncmp 6) : découpage sur `|` :
`QUEST|<quest_code>|<text_id>` → SMSG_NPC_DIALOG+0x1b = quest_code, +0x1f = text_id ; texte affiché =
chaîne `text_id` formatée avec les données de `quest_code` (FUN_00a2ca80) ; voix `V%08d.mp3` jouée avec
`text_id` [O]. Sinon, texte `@…` : id de voix = premier jeton [O].

### 6.2 Menu (types de dialogue 0, 3, 10 seulement)
Menu découpé sur `\t` en paires **(libellé, trigger)** [O]. Libellé `NULL` → entrée masquée [O].
- Libellé `QUEST|<text_id>|<état>` : libellé remplacé par la chaîne `text_id` colorée — état 0 → `<#FFAA33>`
  (icône 1), 2 → `<#AAAAFF>` (icône 3), autre → `<#DDDD77>` (icône 2) [O]. Sens [D] : 0 = disponible,
  1 = en cours, 2 = à rendre. ⚠ Le 2ᵉ jeton est passé directement à la table de chaînes (FUN_006682e0) :
  c'est un **id de texte** (ex. nom de la quête), pas le code de quête [O].
- Uniquement si le texte est un `QUEST|…` : libellés-mots-clés (comparaison de préfixe) remplacés par des
  chaînes localisées : `START` → chaîne 0x13, `REJECT` → 0x14, `OK` → 0xb, `END` → 10 [O].
  `REWARD` → le **trigger** est lu comme entier base 10 (FUN_00c7264b) et envoyé dans SIMSG_UI_SEND_DATA
  « select_reward » à la fenêtre **0x49** (récompenses), qui s'ouvre (FUN_006c47d0(0x49,1,0)) [O] ;
  la fenêtre récompense (FUN_008bc740) mémorise ce nombre comme **code de quête** (+0x44c) [O].
- Le clic sur une entrée (fenêtre PNJ FUN_007e60f0) crée SIMSG_UI_ACT_NPCDIALOG (kind 0x40f, FUN_007ca300) avec
  le trigger → **TS_CS_DIALOG** si `len > 1` [O]. Son joué : `ui_click_accept01.wav` si le trigger contient
  `start_quest`, `ui_click_complete01.wav` s'il contient `end_quest` (FUN_00c5f830) [O].

### 6.3 Rendre une quête (choix de récompense)
Fenêtres FUN_008bc860 (récompense) et FUN_008b2650 (info/récompense) : sélection `quest_selectitem_item_%02d`
(0..5, borné par le nombre d'options, FUN_008b1c80) puis `quest_button_confirm` :
si options présentes et aucune choisie → message (chaîne 0x200), sinon index = −1 sans option ; le client
**formate lui-même** `end_quest( <code>, <index> )` (format exact, espaces compris, chaîne 0xd55448) et l'envoie
via TS_CS_DIALOG (kind 0x40f) [O] ; FUN_008b2650 poste ensuite SIMSG_SHOW_UIWINDOW 0x48 (masquage [D]).

## 7. Paquets C→S dédiés (603 / 604 / 605)

### 7.1 TS_CS_DROP_QUEST (603) — 11 octets [O]
`+7 i32 code`. Chaîne : bouton `quest_button_giveup_01` (FUN_008b2650) → SIMSG_REQ_OPEN_MSGBOX (index 0x11 =
« msgboxGiveupQuest », texte 0x321) → réponse msgbox FUN_009d2130 case 0x11, si bouton 0 (Oui [D]) →
SMSG_SEND_DATA kind 0x3a (FUN_00720ef0 : chaîne « drop_quest » en +0x13, entier en **+0x43** = `SQuestMgr+4`,
quête sélectionnée par FUN_008bac70) → FUN_006d2e60 case 0x3a compare « drop_quest » → envoi (0x6d3597) [O].

### 7.2 TS_CS_QUEST_INFO (604) — 11 octets [O]
`+7 i32 code`. Fenêtre liste FUN_008b7b50 : bouton `quest_info_button`, ou sélection d'une quête (case 0x70/0x71)
quand la fenêtre info 0x48 est ouverte → SIMSG_QUEST_INFO_REQUEST (kind 0xa5, ctor FUN_008b69b0, code en +0x13)
→ FUN_006d2e60 case 0xa5 → envoi (0x6d3a94) ; la fenêtre 0x48 est ouverte **localement** en même temps [O].
Aucune réponse n'est lue (602 non parsé) [O] → réponse serveur facultative [D].

### 7.3 TS_CS_END_QUEST (605) — 12 octets [O layout]
`+7 i32 code ; +0xb u8` (booléen). Deux constructeurs : FUN_006d2e60 case kind 0xb4 (0x6d4297, depuis msg+0x13 /
msg+0x17) et FUN_006cd120 (0x6cd130, même corps, envoi gestionnaire). **Aucun créateur du kind 0xb4** trouvé
(balayages `mov [r+4], 0xb4` / `mov r, 0xb4`) et **FUN_006cd120 n'a aucune référence** [O] → probablement code
mort dans ce client ; la fin de quête passe par TS_CS_DIALOG `end_quest(…)` [D].

### 7.4 Accepter une quête
**Aucun paquet dédié** [O : seuls 603/604/605 et 3001 existent côté émission]. L'acceptation est un
**TS_CS_DIALOG** portant le trigger fourni par le serveur dans le menu (le client ne fabrique pas
`start_quest`, il ne fait que le reconnaître pour le son) [O]. Format du trigger libre, convention
`start_quest(<code>, …)` [D].

## 8. Autres éléments liés [O sauf mention]
- Aucune gestion spécifique de TS_SC_RESULT pour `request_msg_id` 603/604/605 trouvée (balayage des `cmp`
  immédiats 0x258..0x25d : seuls des types de quête 601/701 et un code d'effet) [O balayage, D exhaustivité].
- Fenêtres : 0x42 / 3 / 0x47 (liste, « quest_list_updated », « set_quest_code »), 0x48 (info quête), 0x49
  (récompense, « select_reward »), 0x59 (notice « questupdate ») [O].
- `quest_list_update_new` (fenêtre liste) déclenche un rafraîchissement ; émetteur non identifié [O/?].

## 9. Séquences [D sauf ids, tailles et formats marqués O]

- **Connexion** : … → TS_SC_QUEST_LIST (600) complet (actives + pending).
- **Proposer** : C→S CONTACT (3002) → S→C DIALOG (3000) type 0, texte `QUEST|<code>|<text_id>`, menu
  `\tSTART\tstart_quest(<code>, …)\tREJECT\t<trigger_refus>` (libellés START/REJECT localisés par le client [O]).
  Les quêtes du PNJ peuvent aussi être listées en menu : `\tQUEST|<name_text_id>|<état>\t<trigger>` [O format].
- **Accepter** : C→S DIALOG (3001) `start_quest(…)` → serveur : ajoute la quête puis **renvoie TS_SC_QUEST_LIST
  complet** (seul paquet qui ajoute une entrée [O]) ; ferme ou remplace le dialogue.
- **Progresser** : à chaque changement de compteur → TS_SC_QUEST_STATUS (601) `code, status[6], progress`
  (progress = 2 quand tous les `status[i] == value(2i+2)`) ; le client affiche les lignes modifiées du texte
  `text_id_status`.
- **Rendre** : CONTACT → DIALOG texte `QUEST|<code>|<text_id_fin>`, menu `\tREWARD\t<code>` (ouvre la fenêtre de
  récompense avec ce code [O]) et/ou `\tEND\tend_quest( <code>, -1 )` → C→S DIALOG `end_quest( <code>, <index> )`
  (index d'option 0..5, −1 si aucune [O]) → serveur : récompenses (EXP, objets, or…) puis **TS_SC_QUEST_LIST**
  sans la quête.
- **Abandonner** : C→S DROP_QUEST (603) `code` → serveur : retire la quête et renvoie TS_SC_QUEST_LIST.
- **Consulter** : C→S QUEST_INFO (604) `code` → aucune réponse requise (fenêtre locale).

## 10. Limites / inconnues
- Sens précis de `start_id`, de « pending » et des valeurs de `progress` 0 / 100 : seulement leurs effets
  d'affichage sont observés.
- Triggers `start_quest` / refus : format libre décidé par le serveur ; seul `end_quest( %d, %d )` est imposé
  par le client.
- TS_CS_END_QUEST (605) : layout observé mais aucun chemin d'appel ; sens de l'octet +0xb inconnu.
- TS_SC_QUEST_INFOMATION (602) : non parsé ; ce client ne l'utilise pas.
- Pas de vérification de taille observée dans les handlers 600/601 (la taille paquet est contrôlée en amont
  par le parseur générique, non revérifié ici).
- `#@status_N=…@#` utilise l'index N sans −1 alors que `#@status:N@#` utilise N−1 : rapporté tel que lu.
- Offset 0xf9 vs schéma (show_target_type) : **vérifié sur les données** (2026-11-23). Dans les enregistrements du fichier, l'octet 0xf9 vaut toujours 55 et l'octet 0xfa vaut 1 exactement pour les 42 quêtes dont `show_target_id` ≠ 0 (955 autres : 0). Le schéma (`show_target_type` en 0xfa) décrit donc bien le fichier ; FUN_0070a400 lit 0xf9 dans la structure **en mémoire** (`MOVSX ECX, byte ptr [EDX+0xf9]`, ev_b5455956081460dafd1e84b5666e98d00b3843077806f4aa14073cca49650694). Le chargeur de QuestResource qui passe de l'un à l'autre n'a pas été suivi ; schéma inchangé.
- Le dialogue de quête (fenêtre `quest_dialog`) et l'usage de SMSG_NPC_DIALOG+0x1b/+0x1f par l'IHM n'ont pas été
  suivis au-delà du parseur.

## 11. Adresses

| Adresse | Rôle |
|---|---|
| 0xa29430 / 0xa29780 | parse 600 / 601 |
| 0x708c50 | singleton SQuestMgr (vtable « SQuestMgr », entrées 0x48 o.) |
| 0x709110 / 0x708b50 / 0x708bd0 | ajout entrée / maj statut+progress / vidage |
| 0x709200 / 0x708e80 / 0x709260 / 0x709040 | présence / progress (e+0x3d) / pending (e+0x3e) / time_limit (e+0x38) |
| 0x708fd0 / 0x708ef0 | status[i] (e+8) / value[i] (e+0x20) |
| 0x70a6b0 / 0x70a450 / 0x70a2f0 / 0x70a400 | objectifs atteints / ids cibles / ids objets (103) / cible de marqueur |
| 0xa654b0 / 0xa2da30 | macros `#@…@#` de quête |
| 0x6ce940 | consommateur SMSG_QUEST_UPDATA (notice « questupdate ») |
| 0x6d2e60 | gestionnaire jeu : cases 0x3a (603), 0xa5 (604), 0xb4 (605), 0x40f (3001), 0x6c |
| 0x6cd120 | envoi 605 sans référence |
| 0x720ef0 / 0x8b69b0 / 0x7ca300 | ctor SMSG_SEND_DATA kind 0x3a / SIMSG_QUEST_INFO_REQUEST 0xa5 / SIMSG_UI_ACT_NPCDIALOG 0x40f |
| 0xa25670 | parse TS_SC_DIALOG (QUEST|, START/REJECT/OK/END/REWARD) |
| 0x7e60f0 | fenêtre PNJ (clic menu, sons start_quest/end_quest) |
| 0x8b7b50 / 0x8b7680 / 0x8b8510 | fenêtre liste (604, rafraîchissements, affichage) |
| 0x8b2650 / 0x8bc860 / 0x8bc740 / 0x8b1c80 | fenêtres info/récompense (`end_quest( %d, %d )`, abandon, select_reward) |
| 0x9d2130 | résultats msgbox (case 0x11 → drop_quest) |
| 0x8b5d80 / 0x8f3160 / 0x8f3230 | marques PNJ / marqueurs selon progress |

## 12. Implémentation RappelzCore (2026-11-23)

Choix du serveur, **[D]** sauf ce qui reprend un format [O] ci-dessus :

| Élément | Choix |
|---|---|
| Données | `Quest.dbc` + `QuestLink.dbc` (`rz_rappelz::quests::QuestBook`) ; liens par `NpcResource.id` |
| Persistance | tables AzerothCore inchangées : `character_queststatus` (`status` 3 en cours / 1 à rendre, `mobcount1..4` + `itemcount1..2` = `status[6]`) et `character_queststatus_rewarded` (`rz_database::quests`) |
| Menu PNJ | `QUEST|<text_id_quest>|<0 proposée, 1 en cours, 2 à rendre>` → trigger `quest_show|<code>` |
| Dialogue | texte `QUEST|<code>|<text_id_summary>` ; menu `START` → `start_quest( <code> )`, `REJECT` → `close`, `OK`, ou `REWARD` → `<code>` (le client renvoie `end_quest( <code>, <index> )`) |
| Liste | 600 complet après chaque ajout / rendu / abandon ; `start_id` = 0, `value[6]` = 0, `time_limit` = 0 |
| Progression | 601 à chaque changement de compteur ; position i ↔ paire (value(2i+1), value(2i+2)) conservée même si une paire est vide |
| Types pris en charge | 101 (compteur unique, monstres value1/3/5), 102, 103/107/109 (objets possédés, recalculés à chaque changement de sac), 106 (objet value1 donné à chaque monstre value7..12 tué), 401 (complète dès l'acceptation), 501 |
| Non pris en charge | 201, 301, 302, 601, 701, 901/902 et toute quête à `valueK < 0` (tirage aléatoire) : jamais proposées |
| Rendu | objectifs revérifiés, objets d'objectif consommés, EXP/JP/rupees + récompense fixe + récompense optionnelle choisie ; personnage et passage en « rendue » validés dans **une seule transaction** |
| 603 / 604 / 605 | abandon / ignoré (aucune réponse lue) / ignoré (aucun chemin client établi) |
| GM | `.quest add <id>` (sans conditions) et `.quest remove <id>`, comme AzerothCore |

Validation (MariaDB de test isolée, `rz-bot --talk --quest <id>`) : quête 1193 acceptée chez Andariel (PNJ 1001),
compteur 10/10 depuis le sac, rendue (+1 720 EXP, +430 JP, +13 600 rupees, 10 objets consommés, ligne
`character_queststatus_rewarded`) ; quête 1034 : un Louveteau Affamé tué → 601 `[1,0,0,0,0,0]` persisté en
`mobcount1` ; abandon → 600 vide. Le rendu dans le client graphique d'origine n'a pas été vérifié.

Restent à faire : minuterie (`time_limit`, base d'horloge à confirmer), quêtes aléatoires (bloc `value[]`),
types 201/301/601/701, butin de quête au sol, sens de `start_id` / « pending ».

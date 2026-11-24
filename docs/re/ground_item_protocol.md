# Protocole des objets au sol — client Rappelz NoLimits (SFrame.exe)

Analyse **statique** (Ghidra 12.1.4 via REA) de `Rappelz NoLimits\SFrame.exe`, PE x86,
SHA-256 `27f42fc8f6b08df8c239640b938d601aeaebb7b3db0c89fc47755325a98a410d`. SFrame.exe n'a jamais été exécuté.
Complète `network_protocol.md` §8.1–8.6 et §9.3 (dont le style et les conventions sont repris) ; ce qui y est
déjà établi n'est rappelé que pour la lisibilité.

Conventions :
- **Offsets = offsets paquet** (en-tête TS_MESSAGE de 7 octets inclus : `u32 size, u16 id, u8 checksum`).
- Objets internes `SMSG_*` : « kind » à objet+4 ; **offset SMSG = offset paquet + 0x0c** (champ paquet +7 = SMSG+0x13).
- `[O]` = lu dans le code (adresse citée) ; `[D]` = déduit (nom, sens, ou comportement non observé directement).
- `item+xx` = offset dans l'objet client `SGameItem` (0x1260 octets, vtable 0xd613e4, ctor FUN_00968cf0).
- `res+xx` = offset dans l'enregistrement ItemResource tel que lu par le client (noms de champs non établis).
- Horloge client : FUN_00613c30 = `ms/10 + DAT_00ee9060`, en **centièmes de seconde**, base synchronisée
  (voir `quest_protocol.md` §2.4).

Evidence IDs REA principaux :

| Contenu | Evidence ID |
|---|---|
| Parseur S→C FUN_00a2fed0 (cases 3, 9, 0xcd, 0xd2, 0x11a) | `ev_a43ed3815bdf077a8526f96971dbd62efd47c6f218cf1ed61715c0d55aa81686` |
| Lot : FUN_006b2130 (monde, kinds 0x13 / 0x49), FUN_006aa0d0 (ENTER objet), FUN_00968cf0, FUN_009697c0, FUN_0096bb80 | `ev_1f8ea780ee29f3ad808c5ab37910875b51ac49a63f16b08e68a58630e719ec98` |
| Lot : FUN_009597e0, FUN_0094c1f0, FUN_009694b0, FUN_0095d510, FUN_00a68e30, FUN_006b5950 | `ev_ae1c79fefb36d090899d5d1da420204d57329aa55a359ba492b46915ec03bb78` |
| FUN_00940f90 (données créature, case kind 0xb) / FUN_009415c0 | `ev_10819c0714b63b59c09ad6be19e8db413f9a767c7b5a31a2ac647e92996bec2d` / `ev_91d1a1f959cc987bcbfbd9188013df26ec4f0fe16483f4bbfd19c80bc819a201` |
| FUN_006aec50 (aiguillage ENTER) | `ev_c680873eca48a924ada94971b56df45462e38518c2400a61dc67b1d7ed30b958` |
| Assembleur du ctor SGameItem (vtable 0xd613e4) / octets de la vtable + chaînes qui suivent | `ev_0ddd0ec171bf8437f2c5184e61e82719a23f613bf10a05f77fa2ae6a24d0df7f` / `ev_7da67bfed57437eab25cb5726bd50166979ed664715a102ab50e4105766d4d66` |
| Lot : méthodes SGameItem (FUN_00969100 modèle, FUN_00968f00 son, FUN_00969510 Process, FUN_00968e10 / FUN_00968eb0 afficher / masquer…) | `ev_da9aff0c6dca875ebfc5625ea588698ef506f598c2562e3b1fb21350ee06a58e` |
| Lot : FUN_0096c470 (ctor TAKE_ITEM), FUN_00969460 (droit de ramassage), FUN_0096cb70, FUN_00703e80, FUN_00969fd0 | `ev_4099feacccbc5c93b167313697e3b989ffde4814773c5fcce9e071693a2b9b77` |
| Lot : FUN_0096d420 (SGameLocalPet::ReqTakeItem), FUN_006cf6b0 (TS_SC_RESULT côté jeu) / assembleur 0x96d420 | `ev_df0e4082112977cb008b688694ef189005847fdf2b205554cdb2c012d3cff14f` / `ev_9045e0434639d54212ed5e4f1f7f163f06d4ee7ccdd0c44e950b9c3f9dabde67` |
| Lot : FUN_006d8090 (clic), FUN_0096c640 (ramassage auto du familier), FUN_006d0eb0 (actions clavier) | `ev_6ffa78bf7df1bb4620ba7d5b98b0f6be98c328c96438b76169a921ff3d8054e1` |
| FUN_00968610 (ctor SInputTakeItem) / FUN_009892d0 (entrée 9 → commande 0xb, portée 20.0) | `ev_8bb2aab0087a7c0ae725faa00ffe29737d2dd40ae030f2aaabdc016f3548bbcb` / `ev_88616a67d2916fbe2b0f658255374f60a305526699627afa6f2633c486e8928c` |
| Lot : FUN_0098ef70, FUN_00989ac0 (« SendServerReq », 0xb → TakeItem) | `ev_d2a4c39a975fd723f12507ffdbf512d521cc4f85939846662d91ab39645ee2ab` |
| Lot : FUN_00968b10, FUN_00983260 (SWorkPickUp::Process) | `ev_3085cf1773fc36cbb48014044ef28ed69214c5e0d7b760ceebe2dd719fa09fe0` |
| FUN_009457f0 (machine d'états, entrées réseau) | `ev_7337f12b022cc02fa4b8ffb8ac75c42af99e289aae5c84c17647881e77adda92` |
| Lot : FUN_006ac810 (libération d'un objet en attente), FUN_006dce30 | `ev_2bd2f632ba776265de2bbc25420575404cd2eee085155feb7a2a315a206731f6` |
| FUN_00971180 (SGameMob : libère ses objets) / FUN_009f2d90 / FUN_00951810 (mort) | `ev_a6acab957ceab88609144bda2e7c96dbb0676f161d377bd08df5ef657f76a1ba` / `ev_c6001c6eaac518c35888855cd0e41393a94d8af906d16a41c77ebe6f4e07e054` / `ev_4a7d03de79111e36d67ec9d5c229a91b2de154c20e282d76589489d48fb0cf37` |
| Ctor SGameMob (vtable 0xd62c68) / FUN_009732b0 (ajout d'objet au monstre) / sa xref (slot 0x274) | `ev_89ad20c7399be633f60508375f29e78a5f3728a15cfb30885565cebd5b5136f3` / `ev_151afdff1132ad0521b67f691dee17df29fef54355565a0feaf3fc46e16a6976` / `ev_518eab2661568dcfc6bff3300be1a5e595236014316cc0f69000470fb9c740e8` |
| FUN_006af4a0 (TS_SC_LEAVE, monde) | `ev_345e6577e124eded4518c5eb0cae9739df89777d1ebd16c7ccc6be76c62763b4` |
| FUN_00a47720 (OnMessage de la scène, ordre des consommateurs) | `ev_3d9496c5301246de385898acff00af9dfe367176476c851da07255875c7a4e47` |
| FUN_006d2e60 (gestionnaire jeu, kind 0x13) | `ev_fcbea5235e2a5efd24c35551e8aef307f4d76cbd22393a2b6ddcc8f4c383e1b3` (cf. quest_protocol.md) |
| Chaînes « pick / loot / take_item » / RTTI `SMSG_ITEM_DROP*` / xrefs du TypeDescriptor SMSG_ITEM_DROP_INFO | `ev_d8e7fb353a7ba7346b2230852645226ef9bb6f733864a270f76b082e5b3e3f1c` / `ev_45b8158c1f0034d605f50488f9728eb86173b33578866dfef51da7abd8c41055` / `ev_fc625e6de75ccfbf4daf7bccf6a0e2f5309ec640df945cc3119c4cc779247cb7` |
| xrefs : 0x96bb80 → vtable 0xd61acc ; 0x96d420 → 0xd62630 ; 0xa3a0b0 → 0xd787c8 ; appelants de 0x6ac810 | `ev_34667cf087e91c8b545011088d634668c168a227f4695e46d59c0a04c201e7bc` / `ev_ec6adc9a97d712433ea19939c4024e9327e62680ecec47e2b53cfa9459e30455` / `ev_273b5ff902eb84c05d0eb78c2a7f0597d165cfd0c6ad9b791896e3a1afc41ec9` / `ev_f4f865f06ecce8dab0959d91f86f66038594ce5e1e46f176d607ac0d20b34265` |

Les appels virtuels `call [reg+slot]` ont été retrouvés par un balayage Node des octets de `.text`
(`FF 9x <slot>`) : slot 0x288 (TakeItem) → 0x989bcf, 0x98efbc… ; slot 0x174 (son de chute) → 0x6ac841 ;
slot 0x360 (libération des objets du monstre) → 0x95183f ; slot 0xc0 → 0x9711ad.

## 1. Vue d'ensemble des paquets

| Paquet | ID | Dir | Taille | Corps (offsets paquet) | Kind / fonction |
|---|---|---|---|---|---|
| TS_SC_ENTER (objet au sol) | 3 | S→C | **70** | voir §2 (`type` +7 = 2, `obj_type` +0x19 = 2) | kind 0xb, FUN_006aec50 → FUN_006aa0d0 [O] |
| TS_SC_ITEM_DROP_INFO | 282 | S→C | 15 | +7 u32 (monstre [D]) ; +0xb u32 (objet [D]) | kind 0x14, `SMSG_ITEM_DROP_INFO` ; consommateur non localisé (§3) [O parse] |
| TS_CS_TAKE_ITEM | 204 | C→S | 15 | +7 u32 taker (joueur **ou familier**) ; +0xb u32 item_handle | FUN_0096bb80 / FUN_0096d420 [O] |
| TS_SC_TAKE_ITEM_RESULT | 210 | S→C | 15 | +7 u32 item_handle ; +0xb u32 taker | kind 0x49, `SMSG_TAKE_ITEM_RESULT`, FUN_006b2130 [O] |
| TS_SC_RESULT (refus/succès) | 0 | S→C | 15 | +7 u16 request = **204** ; +9 u16 result ; +0xb i32 value (= item_handle [D]) | kind 4, FUN_006cf6b0 [O] |
| TS_CS_DROP_ITEM | 203 | C→S | 15 | +7 u32 item_handle (sac) ; +0xb u32 count | FUN_006d7cc0 [O] (`network_protocol.md` §9.3) |
| TS_SC_DROP_RESULT | 205 | S→C | 12 | +7 u32 item_handle ; +0xb u8 success | kind 0x13, `SMSG_ITEM_DROP` (défaut success = 1) ; FUN_006d2e60 + FUN_006b2130 [O] |
| TS_SC_LEAVE | 9 | S→C | 11 | +7 u32 handle | kind 10, FUN_006af4a0 [O] |
| TS_SC_INVENTORY / UPDATE_ITEM_COUNT / DESTROY_ITEM / GOLD_UPDATE | 207 / 255 / 254 / 1001 | S→C | — | voir `network_protocol.md` §9.3 | mise à jour du sac après ramassage / dépôt [D] |

Aucun autre paquet d'objet au sol n'existe dans le parseur : ni ENTER groupé, ni ITEM_DROP_INFO groupé,
ni paquet « tout ramasser » côté client [O : parseur, chaînes `TM_` / RTTI `SMSG_*` limitées à ITEM_DROP,
ITEM_DROP_INFO, TAKE_ITEM_RESULT].

## 2. TS_SC_ENTER type 2 / obj_type 2 — 70 octets — FUN_006aa0d0 [O]

Le parseur (FUN_00a2f340) recopie le paquet brut ; FUN_006aec50 n'appelle FUN_006aa0d0 que si aucun objet de
même `handle` n'existe (FUN_006b6440) et si `type` = 2 et `obj_type` = 2.

| Offset | Type | Champ | Utilisation client [O sauf mention] |
|---|---|---|---|
| +0x07 | u8 | type = **2** | recopié en item+0x08 |
| +0x08 | u32 | handle | item+0x0c, clé de la table des objets du monde (FUN_006b5950) |
| +0x0c | f32 | x | item+0x10 |
| +0x10 | f32 | y | item+0x14 |
| +0x14 | f32 | z | **ignoré** : z recalculé par le terrain (scène vtable+0x34 sur x, y) ; si le terrain n'est pas prêt, item+0x11fc = 1 et Process (FUN_00969510) réessaie à chaque image |
| +0x18 | u8 | layer | item+0x50 |
| +0x19 | u8 | obj_type = **2** | données créature (FUN_0095d510, FUN_00940f90 case 2 : aucune autre lecture) |
| +0x1a | EncodedId (8 o.) | **code ItemResource encodé** | passé au ctor (FUN_00968cf0 → FUN_0094c1f0 → FUN_00940a30), décodé par FUN_00969100 / FUN_00968f00 (`lo = w3 + 2(w0+w2)`, `hi = w1 + 2(w0−w2)`) |
| +0x22 | i64 | count [D] | **aucune lecture trouvée** dans FUN_006aa0d0, FUN_009597e0, FUN_00940f90, FUN_009415c0, FUN_009697c0 ni dans les méthodes SGameItem |
| +0x2a | u32 | drop_time | item+0x1240 ; comparé à l'horloge client FUN_00613c30 (centièmes de s) par FUN_00969510 |
| +0x2e | u32[3] | hPlayer[3] | item+0x1244..+0x124f |
| +0x3a | i32[3] | party_id[3] | item+0x1250..+0x125b ; fin du paquet à +0x45 |

### 2.1 Code d'objet, modèle et son [O]
- **Code 0 = or (rupees)** : modèle `drop_etc_money.nx3` (FUN_00969100) et son `item_drop_rupi.wav`
  (FUN_00968f00). Le montant n'est lu nulle part côté ENTER ; `count` = montant est [D].
- Code ≠ 0 : recherche dans l'ItemResource (FUN_00643210 / FUN_006e78e0, entrée par défaut db+0x34 si absent) ;
  modèle = chaîne `res+0x194` + `.nx3`. Un second modèle `drop_itemeffect.nx3` (halo) est toujours chargé.
  Les deux modèles jouent l'animation `default` (FUN_009694b0(0, 1, 4.8f)) ; Process fait tourner le modèle
  dans le temps (matrice item+0x1200) [O calcul, D « rotation »].
- Son de chute (vtable+0x174, FUN_00968f00) selon l'ItemResource : `res+0xbe` ∈ {1, 2} → `item_drop_potion.wav` ;
  sinon `res+0x10` = 0 → `item_drop_etc.wav` ; `res+0x0c` < 0x71 → `item_drop_weapon.wav` ; `res+0x10` = 0xc9 →
  `item_drop_heave_equip.wav` ; `res+0x0c` < 0xcd → `item_drop_light_equip.wav` ; < 0x132 →
  `item_drop_accessary.wav` ; `res+0x10` = 0x132 → `item_drop_cube.wav` ; 0x191 → `soulstonedrop.wav` ; sinon etc.
- Le **nom** affiché vient de l'ItemResource (le paquet ne porte aucun texte) [D : survol non tracé].
- En fin d'ENTER, vtable+0x10 (FUN_00968e10 → FUN_0094e900, obj+0x60 = 1) rend l'objet **visible
  immédiatement** ; le son de chute n'est **pas** joué par l'ENTER.

### 2.2 Droits de ramassage (hPlayer / party_id / drop_time) [O mécanique, D sens]
- FUN_009697c0 : si `hPlayer[0] == 0 && party_id[0] == 0` → état item+0x125c = **3 (libre)**, sinon 0.
- FUN_00969510 (Process, si état ≠ 3) : `âge = FUN_00613c30() − drop_time` (non signé) ;
  âge < 3001 → état 0 ; < 4001 → 1 ; < 5001 → 2 ; sinon 3. Soit **30 s / 40 s / 50 s** en centièmes.
- FUN_00969460(handle, party) : vrai si état = 3, sinon vrai si `handle == hPlayer[i]` ou `party == party_id[i]`
  pour i < état. L'état est donc le **nombre de rangs autorisés** ; à l'état 0 personne n'est autorisé.
- Ce test ne sert qu'au **ramassage automatique du familier** (FUN_0096c640, FUN_0096d420). Le clic et la touche
  du joueur ne consultent ni l'état ni les propriétaires : **le serveur décide seul** ; aucune durée
  d'exclusivité n'est affichée (aucun lecteur IHM trouvé) [O pour les chemins examinés].
- Conséquence [D] : avec `drop_time` = heure serveur du drop (centièmes, même base que TS_TIMESYNC / GAME_TIME),
  le familier n'aspire rien pendant 30 s, puis les objets de hPlayer[0] / party_id[0], puis [1] à 40 s, [2] à 50 s,
  puis tout. Un `drop_time` ancien (ou hPlayer[0] = party_id[0] = 0) rend l'objet libre tout de suite.

## 3. TS_SC_ITEM_DROP_INFO (282) [O parse, D consommateur]

Parse (case 0x11a) : SMSG+0x13 ← paquet+7, SMSG+0x17 ← paquet+0xb (deux u32), kind 0x14. **Aucun consommateur
de kind 0x14 n'a été trouvé** dans FUN_006d2e60, FUN_006b2130 ni FUN_00a47720, et le TypeDescriptor
`SMSG_ITEM_DROP_INFO` n'est référencé que par les structures RTTI (pas de dynamic_cast). Le message finit dans
la file de l'IHM (scène+0x230, vtable+0x10).

Mécanisme de « chute depuis le monstre » observé, que ce paquet alimente très probablement [D] :
1. SGameMob garde un vecteur de handles d'objets (mob+0x11f4..+0x11fc) rempli par la méthode virtuelle slot
   0x274 (FUN_009732b0, push_back) ; le monde garde une liste d'objets « en attente » (monde+0xc0, entrées de
   8 o. dont le premier u32 est le handle).
2. À la mort du monstre (FUN_00951810, appelée par 0x96ac30 / 0x972990, type de créature 3), slot 0x360
   (FUN_00971180) : pour chaque handle → propriétaire (obj+0x5c) slot 0xc0 → FUN_00a3a0b0 → **FUN_006ac810** :
   objet vtable+0x10 (afficher) **+ vtable+0x174 (son de chute)**, puis retrait de monde+0xc0 [O].
3. Tant que le handle est dans monde+0xc0, un DROP_RESULT (kind 0x13) pour ce handle est ignoré par le monde [O].

Sens proposé : +7 = handle du monstre qui lâche, +0xb = handle de l'objet [D]. Ordre conseillé :
**ENTER objet puis ITEM_DROP_INFO, avant ou avec la mort du monstre** [D]. Sans ce paquet l'objet apparaît
dès l'ENTER, sans son de chute (FUN_006ac810 est le seul appelant objet du slot 0x174 trouvé).

## 4. Ramassage côté client [O sauf mention]

### 4.1 Déclencheurs
- **Clic** sur l'objet : FUN_006d8090(handle) (appelé par 0xa46420 / 0xa49e20, gestion souris de la scène) ;
  refusé si l'état d'animation du joueur vaut 0x21 [D : assis] ; crée `SInputTakeItem` (type 9, handle, distance).
- **Action clavier** (FUN_006d0eb0, action 2 et action 0x1b) : objet le plus proche dans un rayon de **60.0**
  (scène vtable+0xd4, filtre −1 ou 2 [D]) → `SInputTakeItem`. Touche exacte non tracée [D].
- La machine d'états (FUN_009892d0) traduit l'entrée 9 en commande 0xb avec une **portée de 20.0** : le joueur
  marche jusqu'à 20 unités puis `SendServerReq(0xb)` (FUN_00989ac0) → vtable+0x288 du personnage local
  (SGameLocalCreature, FUN_0096bb80) → **TS_CS_TAKE_ITEM** (taker = handle du joueur, +0xb = objet).
  Aucune autre vérification (propriété, poids, place) n'est faite côté client.

### 4.2 Familier (« pet ») : ramassage automatique
- SGameLocalPet::Process (FUN_0096c640), si le ramassage auto est actif (pet+0x1234) : cherche l'objet le plus
  proche dans le rayon pet+0x1238 (probablement l'option `game_pet_pickitemdist`, lue dans FUN_00a3def0 [D]),
  ignore ceux déjà demandés, exige FUN_00969460(handle du maître, party du maître) puis pousse `SInputTakeItem`.
- SGameLocalPet::ReqTakeItem (FUN_0096d420, slot 0x288 surchargé, log `ReqTakeItem`) envoie
  TS_CS_TAKE_ITEM **avec taker = handle du familier**, puis envoie un TS_CS_TAKE_ITEM supplémentaire pour chaque
  autre objet à moins de **20.0** du familier qui passe FUN_00969460 et n'est pas déjà dans sa file
  (pet+0x1250, deque ; entrée {pet+0x1248, 0, item, 0}). C'est le seul « tout ramasser » du client.
- Le serveur doit donc accepter un taker égal au handle d'un familier possédé, et peut recevoir une rafale de 204.

## 5. TS_SC_TAKE_ITEM_RESULT (210, kind 0x49) — FUN_006b2130 [O]

1. Objet = table(+7) ; si absent ou si son type ≠ 2 (FUN_00954980) → rien.
2. Taker = table(+0xb) ; s'il existe et que son contrôleur (taker+0x370) n'est pas à l'état 4 → création de
   `SWorkPickUp` (FUN_0097e1d0) posé comme travail courant : animation de ramassage, fin forcée après 10 s
   (FUN_00983260, `0x2711`).
3. FUN_006e1160 / FUN_006e11f0 (rôle non établi), puis si un familier local existe (FUN_00703e80()+0x20),
   FUN_0096cb70 retire l'objet de sa file d'attente [D : argument perdu par le décompilateur].

Le paquet **ne retire pas l'objet** et **ne touche pas au sac** : le serveur doit ensuite envoyer
TS_SC_LEAVE (9) de l'objet à tous les voisins, et au preneur TS_SC_INVENTORY (207, nouvel objet) ou
TS_SC_UPDATE_ITEM_COUNT (255, pile existante), ou TS_SC_GOLD_UPDATE (1001) pour l'or [D].
Il est diffusable à tous les joueurs qui voient la scène (animation du preneur) [D].

## 6. TS_SC_RESULT pour la requête 204 — FUN_006cf6b0 [O]

| result (+9) | Effet client |
|---|---|
| 0 | son `effect_item_pickup.wav` + log ; si familier local, FUN_0096cb70(value +0xb) le retire de la file du familier |
| 6 | message système chaîne **0xdf** |
| 0xb | message système chaîne **0x20e** |
| autre | log uniquement (aucun affichage) |

Les textes des chaînes 0xdf / 0x20e ne sont pas résolus ; d'après les codes TS_RESULT usuels, 6 = accès refusé
(objet d'un autre) et 0xb = trop lourd [D]. En parallèle, la machine d'états (FUN_009457f0) repasse à l'état 1
un personnage dont le contrôleur était à l'état 4 quand elle reçoit un RESULT 204 [O mécanique, D sens].
Envoyer `value` = handle de l'objet demandé.

## 7. Dépôt depuis le sac [O sauf mention]

- C→S TS_CS_DROP_ITEM (203) : `+7 item_handle`, `+0xb u32 count` (FUN_006d7cc0).
- S→C TS_SC_DROP_RESULT (205, kind 0x13) :
  - gestionnaire jeu (FUN_006d2e60) : **success ≠ 0 → aucune action** ; **success = 0** → message système :
    objet absent du sac → 0x393 ; ItemResource `res+0x66` bit 13 → 0xe5 ; `res+0x0c` = 0x0d → 0x106 ; sinon 0x393 ;
  - monde (FUN_006b2130) : si le handle est dans monde+0xc0 → rien ; sinon, si un objet du monde porte ce handle,
    vtable+0x10 (rendu visible).
- La mise à jour du sac n'est pas faite par DROP_RESULT : envoyer TS_SC_DESTROY_ITEM (254) ou
  TS_SC_UPDATE_ITEM_COUNT (255) (dépôt partiel), puis TS_SC_ENTER de l'objet au sol aux voisins [D].
  Le traitement monde du kind 0x13 suggère que l'objet au sol garde le **handle de l'objet du sac** [D].

## 8. Disparition — TS_SC_LEAVE (9) — FUN_006af4a0 [O]

Retire le handle de monde+0xc0 s'il y est, puis supprime l'objet (FUN_00a69050, FUN_006b64e0(handle, 1)).
Aucun traitement spécifique aux objets ; aucune expiration locale : l'objet reste affiché jusqu'au LEAVE.

## 9. Séquences [D sauf ids, tailles et formats marqués O]

- **Butin d'un monstre** : S→C ATTACK_EVENT / HPMP (hp 0) … ; pour chaque objet : TS_SC_ENTER type 2
  (code encodé, count, drop_time = heure serveur en centièmes, hPlayer[0] = tueur, party_id[0] = groupe du tueur)
  puis TS_SC_ITEM_DROP_INFO (+7 monstre, +0xb objet) ; mort du monstre → l'objet apparaît avec le son de chute ;
  LEAVE du monstre plus tard. Or : code 0, count = montant.
- **Ramassage réussi** : C→S TAKE_ITEM (204, taker, objet) → S→C TAKE_ITEM_RESULT (210, objet, taker) diffusé
  → LEAVE (9, objet) diffusé → au preneur INVENTORY (207) / UPDATE_ITEM_COUNT (255) ou GOLD_UPDATE (1001)
  → RESULT (0, 204, 0, objet) pour le son et la file du familier [O effets client].
- **Ramassage refusé** : C→S TAKE_ITEM → S→C RESULT (0, request 204, result 6 ou 0xb, value = objet) ; l'objet
  reste au sol. Ne pas envoyer TAKE_ITEM_RESULT.
- **Dépôt depuis le sac** : C→S DROP_ITEM (203, handle, count) → S→C DROP_RESULT (205, handle, 1)
  + DESTROY_ITEM (254) ou UPDATE_ITEM_COUNT (255) + ENTER type 2 aux voisins (hPlayer[0] = 0, party_id[0] = 0
  pour un objet libre) ; refus : DROP_RESULT (205, handle, 0) seul.
- **Disparition** (expiration serveur, ramassage par un autre) : LEAVE (9, objet) à tous ceux qui le voient.

## 10. Limites / inconnues

- Consommateur du kind 0x14 (ITEM_DROP_INFO) introuvable statiquement, ainsi que l'appelant du slot 0x274
  (FUN_009732b0) : le lien ITEM_DROP_INFO → liste du monstre / monde+0xc0 est déduit.
- `count` (+0x22) n'est lu par aucun code examiné : usage pour l'or et l'infobulle non prouvé.
- Sémantique serveur de `drop_time` et des rangs (état 0 = personne pendant 30 s) : seule la mécanique client
  est observée ; la durée d'exclusivité réelle est un choix serveur.
- Textes des messages 0xdf, 0x20e, 0x393, 0xe5, 0x106 non résolus ; sens exact de `res+0x0c`, `res+0x10`,
  `res+0x66`, `res+0xbe`, `res+0x194`.
- Touche clavier liée aux actions 2 / 0x1b, filtre du « plus proche objet » (−1 / 2), rôle de
  FUN_006e1160 / FUN_006e11f0 et argument de FUN_0096cb70 dans le kind 0x49.
- Unité des distances (20.0 approche, 60.0 recherche, 20.0 chaîne du familier) = unités monde du client [D].

## 11. Adresses

| Adresse | Rôle |
|---|---|
| 0x6aec50 / 0x6aa0d0 | aiguillage ENTER / ENTER objet au sol |
| 0x968cf0 / 0xd613e4 | ctor SGameItem / sa vtable |
| 0x9697c0 / 0x969460 / 0x969510 | propriétaires + état libre / test de droit de ramassage / Process (état, terrain, rotation) |
| 0x969100 / 0x968f00 | chargement du modèle (or : `drop_etc_money.nx3`) / son de chute (slot 0x174) |
| 0x968e10 / 0x968eb0 | afficher (slot 0x10) / masquer (slot 0x14) |
| 0x6b2130 | monde : kinds 0x13 (DROP_RESULT), 0x49 (TAKE_ITEM_RESULT) |
| 0x6d2e60 / 0x6cf6b0 | gestionnaire jeu : kind 0x13 / TS_SC_RESULT (204) |
| 0x6af4a0 | TS_SC_LEAVE (monde) |
| 0x6ac810 / 0xa3a0b0 | libération d'un objet en attente (affichage + son) / relais scène (slot 0xc0) |
| 0x9732b0 / 0x971180 / 0x951810 | SGameMob : ajout d'objet (slot 0x274) / libération (slot 0x360) / mort |
| 0x6d8090 / 0x6d0eb0 | clic sur objet / actions clavier 2 et 0x1b |
| 0x968610 / 0x9892d0 / 0x989ac0 | ctor SInputTakeItem / entrée 9 → commande 0xb (20.0) / SendServerReq |
| 0x96bb80 / 0x96c470 | envoi TS_CS_TAKE_ITEM (SGameLocalCreature, slot 0x288) / ctor du paquet |
| 0x96d420 / 0x96c640 / 0x96cb70 | familier : ReqTakeItem en chaîne / ramassage auto / retrait de la file |
| 0x97e1d0 / 0x983260 | SWorkPickUp : ctor / Process (animation, 10 s) |

## Implémentation RappelzCore (2026-11-23)

Choix du serveur, **[D]** sauf formats [O] ci-dessus :

| Élément | Choix |
|---|---|
| Handles | plage 0x5000_0000 (monstres 0x2000…, PNJ 0x4000…, objets de sac 0x6000…) |
| Butin | `creature_loot_template` (Entry = id de monstre) + `reference_loot_template` ; règles AzerothCore : ligne `GroupId` 0 tirée selon `Chance`, un seul élément par groupe, références imbriquées (profondeur ≤ 8) |
| DropGroup | `rz-dbimport --only loot` : DropGroup.dbc → `reference_loot_template` (Entry = \|id\|, `GroupId` 1, `Chance` = drop_percentage × 100 ; sous-groupe : `Item` = `Reference` = \|id\|) — 31 657 lignes |
| Butin par monstre | **aucune donnée dans le client** (`drop_table_link_id` de Monster.dbc vise la table serveur officielle) : `creature_loot_template` est à remplir par l'administrateur |
| Mort d'un monstre | objets posés en éventail (rayon 12) autour du corps, `owners[0]` = tueur, `drop_time` = horloge du jeu ; ENTER via la visibilité puis 282 (monstre, objet) |
| Droits | exclusivité du tueur 30 s, puis libre ; objet jeté par un joueur : libre |
| Ramassage (204) | distance ≤ 60, même couche, droits, place dans le sac (sauf rupees, code 0) ; succès : 210 + LEAVE à tous ceux qui le voyaient, 207 au preneur, RESULT(204, 0, objet) ; refus : RESULT(204, 6, objet) ; échec SQL : l'objet retourne au sol |
| Dépôt (203) | pile entière ou partielle (`count` 0 = tout) ; la base est modifiée d'abord, puis 254/255, 205(1), objet posé libre sous le joueur ; objet inconnu : 205(0) |
| Disparition | 180 s, LEAVE à tous ceux qui le voyaient |
| Rupees | toujours versées directement à la mort (récompense), pas encore posées au sol |
| Familier | ramassage par le familier non géré (taker ≠ joueur ignoré) |

Validation (MariaDB de test isolée, `rz-bot --attack --loot`, `--drop`) : avec une entrée de test pour le
Louveteau Affamé (Potion Bleue 100 % + référence 510000), la mort produit deux objets réservés au tueur, deux
TS_SC_ITEM_DROP_INFO, puis deux ramassages réussis (210, RESULT 204 → 0) ; les objets sont en base. Une potion
jetée (pile 4 → 3) apparaît libre au sol puis est ramassée. Non vérifié : client graphique d'origine,
exclusivité face à un second joueur (couverte par un test unitaire), disparition après 180 s en conditions réelles.

**Correctif du 2026-11-24** : les objets au sol n'étaient montrés qu'à la couche 0, sans comparer leur propre couche ;
ils ne sont désormais visibles que sur leur couche (test `ground_items_are_only_visible_on_their_own_layer`).

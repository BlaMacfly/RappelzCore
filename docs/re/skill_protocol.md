# Compétences, états et recharges — client Rappelz NoLimits (SFrame.exe)

Analyse **statique** (Ghidra 12.1.4 via REA, complétée par un balayage d'octets du PE) de `Rappelz NoLimits\SFrame.exe`,
PE x86, SHA-256 `27f42fc8f6b08df8c239640b938d601aeaebb7b3db0c89fc47755325a98a410d`. SFrame.exe n'a jamais été exécuté.
Complète `network_protocol.md` §9.3, `rdb_format.md` §5.3 (db_skill) et db_tenacity, `summon_protocol.md` §5 (4001/4002),
`../SKILL_CASTING_VALIDATION.md` et `../SKILL_CORPSE_VALIDATION.md` (sous-ensemble restauration, non refait ici).

Conventions :
- **Offsets = offsets paquet** (en-tête TS_MESSAGE de 7 octets inclus). Objets internes : kind à objet+4,
  **offset SMSG = offset paquet + 0x0c** (sauf la queue de TS_SC_SKILL, recopiée à SMSG+0x56…).
- `[O]` = lu dans le code (adresse citée) ou mesuré sur le fichier ; `[D]` = déduit ; `[?]` = inconnu.
- Un impact (« hit ») fait 45 octets (0x2d) en mémoire ; ses offsets sont **relatifs à l'octet `type`** de l'entrée
  et identiques dans le paquet et dans l'objet SMSG_DAMAGE (kind 0xf) à partir de SMSG_DAMAGE+0x7a.
- `rec+xx` = offset fichier dans **db_skill.rdb** (827 o.), copié tel quel en mémoire ; `st+xx` = **db_tenacity.rdb** (552 o.).
- Temps réseau : **centièmes de seconde** (cs), horloge « arTime » de TS_TIMESYNC ; le client multiplie par 10 (ms).

Evidence IDs REA principaux :

| Contenu | Evidence ID |
|---|---|
| Parseur S→C FUN_00a2fed0 (0x191, 0x193, 0x194, 0x196, 0x197, 0x1c3, 0x1f9) | `ev_a43ed3815bdf077a8526f96971dbd62efd47c6f218cf1ed61715c0d55aa81686` |
| Monde FUN_006b2130, skill FUN_006b06d0, état FUN_006b0940, recharges FUN_006cff80, 403 FUN_006d27c0 / FUN_00a2ae90, action FUN_006d1cc0 / FUN_006d8d40 / FUN_0079c590 | `ev_cbc2b616a08c9addaffc0e4a683a0a33b2125ebbf585ae887db6c55c42e851a1` |
| db_skill FUN_00673640, évaluateur FUN_00776780, info-bulle FUN_00787fd0, créature/monstre FUN_00971200 / FUN_00988710, ONSKILLDAMAGE FUN_00a3d5c0 | `ev_047b136090d6d4ac836c1ec03d9c785d086cf3dc7408eb8c8407c124376754b4` |
| Macros d'info-bulle FUN_006eb300–FUN_006ee990, emplacements FUN_0070cfa0–FUN_0070d840, cible FUN_006c6f40, armes FUN_006c8860, états portés FUN_009549e0 / FUN_009597e0 | `ev_64b54e7620996241da3f52daf158e4bd8fcd9d94aa5e1e51f893ff9a6897d6a3` |
| Texte de formule FUN_006eb020, états d'une compétence FUN_006ed580, passifs FUN_006ee110, effet d'état FUN_006edc60 | `ev_5bb3284f206efe275e79071317e75546c7daeb725490b1d06dd53110d262aca4` |
| SSkillSlotMgr / SCreatureSlotMgr (vtables 0xd33e00 / 0xd33e7c) | `ev_ec478f42517da3fb8e4935abe742620d0b1cb0774597d0ee27bab43442c7ad50` |
| Chargeur db_tenacity FUN_00637840, effets d'état FUN_00788890 / FUN_007889f0, passifs FUN_006edb00 / FUN_006edbb0 | `ev_34ef83e89abb4817444b6c79c0d6ac0e97edc6cea52404ba72d3e08102226f13` |
| Ajout/retrait d'état FUN_0094eff0 / FUN_00951a50, 406 FUN_006d0490 | `ev_f1c01033523666efc1cbd4cfaf00422298262a3479540fa9d78b83a567490f3a` |
| Machine d'états joueur FUN_009457f0, objets SNewSkill / SWorkingSkill | `ev_8542eebf896f0141716268a5f9f552955244e87bd725155845b7cf693c608025` |
| Messages d'impact FUN_007ac540, nombres flottants FUN_009df3b0, SMSG_DAMAGE FUN_006c6430 / FUN_00981190 | `ev_efd570cd9d16f06b51e39c5273e7fa21dfaf9204e62ccd03f31b63485c5ac00e` |
| File PV/PM FUN_006da6a0, fenêtres compétence/barre rapide, IHM FUN_009d3db0 | `ev_30a50d621661dca934e7bbd04d0434094d1b3b45454a275a11aca04b94a3eaf9` |
| Application PV/PM FUN_0094f830 / FUN_006e18d0 / FUN_006e1440 / FUN_006e19f0 ; FUN_006e1730, FUN_00691880 | `ev_02c43ce3f43169c6ffc16eccf1ac50b7cbcd5db15375dc6ae3c829fed020e805` / `ev_5ca4a3d1ed2d99733732eceea671174175b47a021234f4e8123c5b0ffdd509b8` |
| Erreurs d'incantation FUN_009e30f0, jauge FUN_00746150 / FUN_00747320 ; jauge FUN_00744f90, approche FUN_0098c130 | `ev_8d094e989d3f6dbe838a12dd26c53a8bf3a657e468dc609daba74b87c0aeae7e` / `ev_8d93837a34fdf0fc8293713db4281816bb81e4537f1e1bb00243064e7cfec9cc` |
| Envoi joueur FUN_0096f260, portée FUN_0098b690 / FUN_0098b4c0 | `ev_2f4c3cb732a08a0254892fc82cdae5e16eaaae19f6665a162e5e860c9285f1e6` |
| Actions du joueur FUN_00978cf0, objet de compétence FUN_006d15f0 ; machine FUN_0098b7c0 | `ev_8caa2a147e767c32cd9111d2b7677ee5e4b8e21d2390ca2ddc0294d5714926f5` / `ev_84784cd59fa6ccdb15d5a6afdcab6b36c61ee3032df2a5a31898bfde2fecf882` |
| Durées d'état FUN_00705bd0, dégâts périodiques FUN_0077d470… ; états types 1/2/3/4 FUN_00793bd0 / FUN_00793730 / FUN_00794040 | `ev_c837d18fe8eb487f6867f555fb88ca931f94917ec478efa1c703a0dabd4c8270` / `ev_9c3e7c94a92982ba73c11bd014f5bbafd124a9a0a141a88f60e9cf8cb7f1fd59` |
| Chaînes « skill » / « #@ » | `ev_99caaa3e3d57cacdf04d6384e444a744d1c829d5087f1726a560bebbf412567a` / `ev_3838fad613f1f52281c17b795855ea8bc0447d8ab229e842a72094e66bd3b916` |

## 1. Vue d'ensemble des paquets

| Paquet | ID | Dir | Taille | Corps | Effet client |
|---|---|---|---|---|---|
| TS_CS_SKILL | 400 | C→S | **31** | +7 u16 skill_id ; +9 u32 caster ; +0xd u32 target ; +0x11 f32 x, +0x15 y, +0x19 z ; +0x1d u8 layer (**toujours 0**) ; +0x1e u8 niveau | joueur : **FUN_0096f260** (vtable SGameLocalPlayer+0x294) ; créature : FUN_0096b840 [O] (§5) |
| TS_SC_SKILL | 401 | S→C | 48 + queue | §2, §3 | kind 0x2a [O] |
| TS_SC_SKILL_LIST | 403 | S→C | 14 + 14n | +7 u32 target ; +0xb u16 n ; +0xd u8 modification_type ; n × {i32 id, u8 base_lv, u8 cur_lv, u32 total_cool, u32 remain_cool} | kind 0x2d [O] (§6) |
| TS_SC_ADDED_SKILL_LIST | 404 | S→C | 13 + 6n | +7 u32 target ; +0xb u16 n ; n × {i32 id, u8, u8} | kind 0xa6, FUN_006c8d20 : vide puis remplit la liste de bonus (vtable +0x10 puis +0x20) [O] ; sens des 2 octets [D] |
| TM_STATE_DAMAGE / STATE_RESULT | 406 | S→C | **36** | +7 u32 lanceur [D] ; +0xb u32 cible ; +0xf u32 code d'état ; +0x13 u16 niveau ; +0x15 u16 type ; +0x17 i32 valeur ; +0x1b i32 valeur finale ; +0x1f u8 fin ; +0x20 i32 total | kind 0x58 [O] (§4.4) |
| TS_SC_AURA | 407 | S→C | **14** | +7 u32 handle ; +0xb u16 skill_id ; +0xd u8 actif | kind 0x61 : bascule de la barre rapide (§4.5) [O] |
| TS_SC_SKILL_LEVEL_LIST | 451 | S→C | 9 + 5n | +7 u16 n ; n × 5 octets bruts ({i32 id, u8 niveau} [D]) | kind 0xb2 transmis à une fenêtre ; consommateur non suivi [?] |
| TS_SC_STATE | 505 | S→C | 31 | §4 | kind 0x34 [O] |
| TS_SC_HPMP | 509 | S→C | 36 | voir `network_protocol.md` | kind 0x37 [O] |

## 2. TS_SC_SKILL (401)

En-tête commun (48 octets, FUN_00a2a7f0 [O]) : +7 u16 skill_id ; +9 u8 niveau ; +0xa u32 caster ; +0xe u32 target ;
+0x12 f32 x ; +0x16 f32 y ; +0x1a f32 z ; +0x1e u8 layer ; **+0x1f u8 type** ; +0x20 i32 hp_cost ; +0x24 i32 mp_cost ;
+0x28 i32 caster_hp ; +0x2c i32 caster_mp. Les valeurs **−1** de caster_hp / caster_mp signifient « ne pas toucher »
(FUN_006e18d0 compare à −1) [O]. hp_cost / mp_cost ne servent qu'au calcul intermédiaire (§3.3).

| type | Nom (chaîne de débogage) | Queue lue | Effet client [O] |
|---|---|---|---|
| 0 | FIRE | +0x30 u8 multiple ; +0x31 f32 range ; +0x35 u8 target_count ; +0x36 u8 fire_count ; +0x37 u16 n ; +0x39 impacts[n] | action de tir + impacts ; ferme la jauge ; démarre la recharge locale |
| 1 | CAST (CASTING) | +0x30 u32 durée (cs) ; +0x34 u16 erreur | erreur 0 : jauge, animation, PV/PM du lanceur appliqués ; erreur ≠ 0 : message et abandon |
| 2 | CASTING_UPDATE | +0x30 u32 durée (cs) ; +0x34 u16 | jauge mise à `durée × 10` ms (FUN_00746150), rien d'autre |
| 3 | CANCEL | aucune (target interne := caster) | ferme la jauge, abandonne l'action (FUN_00978cf0 cas 0xd), libère l'emplacement |
| 4 | REGION_FIRE | identique à FIRE | effets + impacts **sans** changement d'état du lanceur |
| 5 | COMPLETE | aucune (target interne := caster) | machine d'états → 8 (fin de compétence) |
| autre | — | — | ignoré |

Tailles lues : CASTING / UPDATE 54, CANCEL / COMPLETE 48, FIRE / REGION_FIRE 57 + impacts. Des octets de bourrage
supplémentaires sont sans effet (le parseur ne contrôle pas la taille).

### 2.1 Aiguillage [O]

- FUN_006b06d0 (monde) : FIRE → cherche un impact **RUSH (30)** réussi et déplace le lanceur
  (FUN_00a69100) ; crée les effets (FUN_006a2060, sauf si le lanceur est un monstre) ; appelle `lanceur->vtbl+0x284`.
  CASTING → vtbl+0x284 puis orientation vers (x, y, z) (vtbl+0x1f8). UPDATE, REGION_FIRE, COMPLETE, CANCEL → vtbl+0x284.
  **Lanceur inconnu : paquet ignoré** (pas de TS_CS_QUERY, contrairement à ATTACK_EVENT).
- FUN_009457f0 (machine du joueur ; monstres FUN_00988710, créatures FUN_009486c0) :

| Reçu | État de la machine | Détail |
|---|---|---|
| FIRE | 7 | copie multiple / range / target_count / fire_count / impacts ; impact 30 → fin d'approche |
| CASTING, erreur 0 | 5 | position cible = (x, y, z) ; durée → animation d'incantation |
| CASTING, erreur **2** (TOO_FAR) sur la compétence en attente | — | **l'action est remise en file** : le client se rapproche puis renvoie 400 |
| CASTING, autre erreur | 8 | fin |
| CANCEL | 0xd | abandon des actions en cours |
| COMPLETE | 8 | si la compétence est physique (`rec+0xb` = 0) et nuisible, un drapeau est levé (reprise de l'attaque automatique [D]) |
| TS_SC_RESULT(400, code ≠ 0) | 8 | **libère la machine, sans message** dans le gestionnaire d'IHM examiné |

- Sortie des états 7 et 8 (FUN_0098b7c0) : dès que l'objet peut agir (vtbl+0x2c0) **et** qu'aucune action n'est en cours
  (FUN_00955cb0). Un tir est de plus borné à 4 000 ms (FUN_00986da0). **COMPLETE n'est donc pas nécessaire pour débloquer
  le joueur** ; FIRE termine lui-même l'incantation en cours (appel vtbl+0x18 des actions, même méthode que COMPLETE).
- **FIRE seul** (sans CASTING) : admis, l'action de tir est créée directement. FIRE avec **n = 0** : les actions en cours
  sont arrêtées et seul le mouvement de fin de db_skillstage est joué (s'il existe) ; PV/PM du lanceur appliqués aussitôt.
- **db_skillstage sans ligne pour la compétence** : le joueur n'envoie pas 400 (FUN_0096f260) et n'anime pas le tir (FUN_00978cf0) ;
  les changements PV/PM restent alors en file jusqu'à un 509 ou un CASTING du lanceur (§3.3) [D].
- Ni FIRE ni CANCEL après CASTING : la jauge reste affichée et l'état 5 dure (aucune expiration locale trouvée) [O lecture, D].
- COMPLETE utile : compétences tenues (objets SWorkingSkill) et reprise de l'attaque automatique (FUN_0098b7c0, `+0x2c8`).

### 2.2 Erreurs d'incantation affichées (FUN_009e30f0, joueur ou créature du joueur) [O]

| Code RESULT | Nom (table 0xe98fe8) | Texte db_string |
|---|---|---|
| 5 | NOT_ACTABLE | 307 |
| 20 | NOT_ENOUGH_HP | 306 |
| 21 | NOT_ENOUGH_MP | 305 |
| 22 | COOL_TIME | 335 |
| 23 | LIMIT_WEAPON | 317 |
| 24 / 25 / 26 | LIMIT_RACE / LIMIT_JOB / LIMIT_TARGET | 280 |
| 31 | NOT_ENOUGH_ENERGY | 383 |
| 32 | NOT_ENOUGH_BULLET | 334 |
| 70, 90–94 | apprivoisement | 7925, 7922, 743, 7923, 7926, 7927 |
| 2 (TOO_FAR), autres | — | aucun texte ; 2 relance l'approche |

Un refus **visible** passe donc par 401 CASTING avec `erreur`, pas par TS_SC_RESULT(400).

## 3. Impacts

### 3.1 Dispositions

Tailles lues par FUN_00a2a7f0 [O]. Tous commencent par `+0 u8 type ; +1 u32 target`.

| type | Nom usuel [D] | Taille | Champs (offsets relatifs à l'entrée) |
|---|---|---|---|
| 0 / 1 | DAMAGE / MAGIC_DAMAGE | 32 | +5 i32 target_hp ; +9 u8 damage_type ; **+0xa i32 damage** ; **+0xe u32 flag** ; +0x12 u16 élément[7] |
| 2 | DAMAGE_WITH_KNOCK_BACK | 45 | 0..0x1f comme 0 ; +0x20 f32 x ; +0x24 f32 y ; +0x28 u8 vitesse ; +0x29 u32 [durée, D, non lue] |
| 10 | RESULT | 10 | +5 u8 succès ; +6 i32 genre |
| 20 | ADD_HP | 13 | +5 i32 target_hp ; +9 i32 montant |
| 21 | ADD_MP | 13 | +5 i32 target_mp ; +9 i32 montant |
| 22 | ADD_HP_MP_SP | 25 | +5 i32 target_hp ; +9 i32 PV ; +0xd i32 PM ; +0x11 i32 SP ; +0x15 i32 target_mp |
| 23 | REBIRTH | 23 | +5 i32 target_hp ; +9 i32 PV ; +0xd i32 PM ; +0x11 i32 ? ; **+0x15 i16 target_mp** |
| 30 | RUSH | 19 | +5 u8 succès ; +6 f32 x ; +0xa f32 y ; +0xe f32 orientation [D] ; +0x12 u8 vitesse |
| 40 / 41 | CHAIN_DAMAGE / CHAIN_MAGIC_DAMAGE | 36 | 0..0x1f comme 0 ; +0x20 u32 handle d'origine [D] |
| 42 | CHAIN_HEAL | 17 | +5 i32 target_hp ; +9 i32 montant ; +0xd u32 handle d'origine [D] |
| autre | — | — | **le pointeur n'avance pas** : toutes les entrées suivantes sont relues au même endroit, donc perdues |

Champs confirmés par trois lecteurs indépendants : FUN_009df3b0 (nombres), FUN_007ac540 (messages), FUN_006da6a0 (PV/PM).
`élément[7]` = dégâts **supplémentaires inclus dans damage** : nombre principal = `damage − Σ élément`, second nombre
« + Σ élément » [O]. Ordre des éléments : normal, feu, eau, vent, terre, lumière, ombre (ordre de `rec+9`, FUN_006eb540) [D].

`flag` (+0xe), testé dans cet ordre [O] : **bit 0 = critique**, bit 2 (4) = blocage, bit 3 (8) = blocage parfait,
bit 1 (2) = raté (« n'a eu aucun effet » ou « le sort a échoué » selon `rec+0xb`). La réaction de la cible n'est jouée que si
`flag & 0x0a == 0` (FUN_00693210). Le bit 2 est le blocage (guard), pas le raté : ne pas confondre avec ATTACK_INFO,
dont les bits diffèrent (1 blocage parfait, 2 blocage, 4 raté, 8 critique dans le même code).

`genre` de RESULT réussi : **10** état ajouté, **11** état retiré, **21** activé, **22** désactivé ; autre = pas de message.
Échec (+5 = 0) : message d'échec et marque « résisté ». RESULT est **purement visuel** : il n'ajoute ni ne retire d'état.

### 3.2 Effet de chaque impact [O]

| type | Nombre affiché | PV/PM modifiés par FIRE seul | Message système |
|---|---|---|---|
| 0, 1, 2, 40, 41 | dégâts, critique, blocage, raté | **oui** : PV = target_hp | oui (331, 374, 498, 308, 330…) + buff/débuff si `rec+0xcf` > 0 |
| 10 | « résisté » si échec | non | oui |
| 20, 42 | vert, si montant > 0 | **oui** : PV = target_hp | oui |
| 21 | bleu | **oui** : PM = target_mp | oui |
| 22 | PV vert, PM bleu, SP vert | **PV et PM oui** ; SP non (TS_SC_SP 514) | oui |
| 23 | PV vert, PM bleu | **oui** (PM sur 16 bits signés) | non (pas de cas dans FUN_007ac540) |
| 30 | — | **non, et bloque tout le paquet** (§3.3) | non |

Type 2 réussi : la cible en vie est déplacée à (x, y) à la vitesse donnée (FUN_009f3650). REBIRTH ne change pas le statut
« mort » par lui-même dans le code examiné : envoyer aussi STATUS_CHANGE / HPMP [D].

### 3.3 File PV/PM (FUN_006da6a0 → FUN_0094f830 → FUN_006e18d0) [O]

- **CASTING réussi** : caster_hp / caster_mp appliqués **immédiatement**. Ils doivent être exacts.
- **FIRE / REGION_FIRE** : un enregistrement {lanceur, caster_hp, caster_mp, hp_cost, mp_cost, impacts} est mis en attente sur le
  lanceur, puis appliqué au premier impact joué (FUN_006e1730, appelé par ONSKILLDAMAGE FUN_00a3d5c0). Pour chaque cible :
  valeur finale = `target_hp` / `target_mp` ; s'il reste d'autres changements en attente sur la cible, le client retranche
  `damage` (ou ajoute le montant) à titre intermédiaire. **La valeur finale fait foi ; PV finaux < 1 → mort (vtbl+0x2f4)**.
- **n = 0** : appliqué tout de suite (PV/PM du lanceur).
- **Un impact RUSH dans la liste annule la mise en file de tout le paquet** (PV/PM du lanceur et des autres impacts) :
  envoyer HPMP (509) dans ce cas.
- **Exception** : FIRE d'une compétence dont db_skillstage.type = 604 (0x25c) ne met rien en file.
- Un HPMP (509) ou un CASTING suivant pour le même handle vide d'abord la file (FUN_006e1400 / FUN_006e19f0).
  Un HPMP n'est requis pour aucun impact 0–23, 40–42 ; il reste utile pour max_hp / max_mp, SP et resynchronisation.
- Regen : HPMP avec `display` (+0x23) ≠ 0 affiche un nombre ; TS_SC_REGEN_HPMP (516) met à jour sans nombre.

## 4. États

### 4.1 TS_SC_STATE (505) — 31 octets [O]

| Offset | Type | Champ | Remarque |
|---|---|---|---|
| +7 | u32 | handle porteur | inconnu → ignoré (à envoyer **après** l'ENTER) |
| +0xb | u16 | state_handle | partie de la clé |
| +0xd | u16 | niveau | **0 = retrait** |
| +0xf | u32 | code = db_tenacity.state_id | partie de la clé |
| +0x13 | u32 | **end_time** (arTime, cs) ; 0xffffffff = permanent | ce n'est **pas** une durée restante |
| +0x17 | u32 | start_time (arTime, cs) | |
| +0x1b | i32 | valeur | pour le code 9006, le client retranche 100 |

Durée affichée = `(end_time − start_time) × 10` ms, départ = `(start_time − décalage d'horloge) × 10` ms (FUN_00705bd0, seul
endroit où les deux champs sont combinés) [O]. Le décalage vient de TS_TIMESYNC : sans synchronisation, les durées sont fausses.

### 4.2 Règle d'ajout / mise à jour / retrait [O]

Clé = **(handle, state_handle, code)** (FUN_009549e0 / FUN_00954e70 : les trois doivent être égaux).

| Cas | Condition | Effet |
|---|---|---|
| Retrait | niveau = 0 | FUN_00951a50 : l'entrée de même clé est ôtée (inconnue → rien) ; IHM « retirer » ; joueur local : message 312 |
| Mise à jour | niveau > 0, clé connue | niveau, start, end, valeur remplacés ; message seulement si le **niveau change** |
| Ajout | niveau > 0, clé inconnue | nouvelle entrée, liste bonus ou malus selon `st+0xc` (is_harmful) ; joueur local : message 310 (bonus) ou 381 (malus) |
| Code absent de db_tenacity | — | rien n'est affiché |

Le retrait exige donc **le même state_handle et le même code**. Rien n'expire côté client : le serveur doit envoyer le niveau 0.
Les codes 9004, 9005, 9006 vont dans une liste spéciale. « Nouveau » (FX, message) = niveau différent de celui déjà connu.

### 4.3 Effets propres au client (FUN_006b0940, FUN_009597e0) [O]

| `st+0x48` effect_type ou code | Effet local |
|---|---|
| type 104 (0x68) et 114 (0x72) | transformation d'apparence (`value_0` / champ valeur) ; 114 change aussi le nom affiché (SIMSG_UI_CHANGE_DISPLAY_NAME) |
| type 109 (0x6d) | drapeau objet 0x10 |
| type 111 (0x6f) | drapeau objet 8, rayon = value_0 × 12 ; au retrait, rend visibles/désélectionne |
| type 200 | créature d'accompagnement (value_0) créée / retirée (FUN_006ae3a0 / FUN_006b1d80) |
| codes 6005, 6006, 6008, 6009 | drapeaux 0x20, 0x40, 0x80, 0x100 |
| codes 6012, 6019 | drapeau 2 (aspect figé [D]) |
| codes 6016, 201084 | invisibilité : opacité 0,2 pour soi, 0 pour les autres ; désélection de la cible |

Autres usages : `st+0x1fc / 0x200 / 0x208` (FX à l'incantation de `cast_skill_id`), `st+0x210 / 0x214 / 0x218` (FX à l'impact).

### 4.4 TM_STATE_DAMAGE (406) — dégâts et soins périodiques [O]

| type (+0x15) | Effet | PV/PM |
|---|---|---|
| 1 | dégâts PV + réaction de la cible | PV = valeur finale (+0x1b) |
| 2 | perte de PM | PM = valeur finale |
| 4 | soin PV | PV = valeur finale |
| 5 | soin PM | PM = valeur finale |
| autre | pas de nombre | — |

Suffisant seul (pas de 509). Si `fin` (+0x1f) ≠ 0 et `total` (+0x20) > 0, et que le joueur est lanceur ou cible, un message
récapitulatif nomme l'état et le total.

### 4.5 TS_SC_AURA (407) [O]

Kind 0x61 : ignoré sauf handle = joueur (propriétaire 1) ou créature connue (2). La barre rapide (SUIQuickSlotWnd,
FUN_007f17e0) mémorise (skill_id, handle) et l'état **actif si +0xd == 1**, inactif sinon. Aucun état n'est ajouté : les effets
d'une aura restent à envoyer par 505.

### 4.6 db_tenacity.rdb [O]

Chargeur FUN_00637840 : en-tête 0x84, enregistrements de **0x228 = 552 o.** lus d'un bloc ; fichier 1 217 844 o. = 132 + 2 206 × 552
[O mesure]. Clé = +0. Accesseurs : **nom** = db_string[`st+4`] (FUN_00637640) ; info-bulle = db_string[`st+8`] (FUN_006377b0) ;
**nuisible** = `st+0xc` (FUN_00637800) ; **icône** = chaîne `st+0xf4` (FUN_00637600). Champs aussi lus : `st+0x10` (bit 0x2000,
interdit de monture), `st+0x28` reiteration_count (> 0 : pas de message de mise à jour), `st+0x2c` base_effect_id,
`st+0x30` fire_interval, `st+0x38..0x44`, `st+0x48` effect_type, `st+0x50` value_0..19 (f64). Le schéma `tenacity` existant est
compatible.

## 5. TS_CS_SKILL (400) : contrôles locaux et envoi

### 5.1 Enchaînement [O]

IHM FUN_0079c590 (anti-répétition ; 4001 refusé s'il y a 3 créatures) → SIMSG 0x413 → FUN_006d1cc0 → FUN_006d8d40 (crée
SInputSkill) → machine d'états (approche FUN_0098c130) → état 5 → **vtbl+0x294** = FUN_0096f260 (joueur).

**Divergence** : `network_protocol.md` et `msg_game.rs` citent FUN_0096b840 ; c'est la version **créature** (vtable
SGameLocalCreature+0x294). Le format est identique, les conditions diffèrent.

### 5.2 Contrôles avant envoi [O]

| Contrôle | Source | Refus |
|---|---|---|
| compétence inconnue / `rec+8` (is_valid) = 0 | FUN_006d1cc0 | — / texte 388 |
| emplacement occupé (recharge locale en cours, slot+0x2c ≠ 0) | FUN_0070cc90 / FUN_0070cc50 | 335 |
| 4001 / 4002 sans carte | FUN_006d1cc0 | 378 |
| passive (`rec+0xa` = 0) | FUN_006d1cc0 | 391 |
| cible (`rec+0xd` = 1) | FUN_006c6f40, §5.3 | 307 ou 318 |
| `rec+0xe` (cadavre) | FUN_006d1cc0 | 392 |
| monté et état `rec+0xcf` portant le bit 0x2000 | FUN_006d1cc0 | 290 |
| arme équipée (`rec+0x88..0x97`, 0x98 bouclier, 0x99 sans condition) | FUN_006c8860 | 317 |
| portée | FUN_0098b690 | approche automatique |
| peut agir (vtbl+0x2c0), aucune action en cours non terminée, mort, enregistrement db_skillstage présent | FUN_0096f260 | silencieux |

**Non trouvés localement** : PV, PM, énergie, objets, niveau, coût en EXP/JP, recharge au moment de l'envoi. Le serveur doit
tout contrôler. En mouvement : le client s'arrête (vtbl+0x17c) si db_skillstage+5 = 0 ou +6 = 1, puis envoie.

Types d'arme (code interne → champ) : 0x5f double_axe, 0x60 double_sword, 0x62 double_dagger, 0x65 one_hand_sword,
0x66 two_hand_sword, 0x67 dagger, 0x68 spear, 0x69 axe, 0x6a one_hand_mace, 0x6b two_hand_mace, 0x6c heavybow, 0x6d lightbow,
0x6e crossbow, 0x6f / 0x70 bâtons une / deux mains, 0x71 one_hand_axe, 0xd2 bouclier. Deux armes identiques (0x65, 0x67, 0x71)
sont testées aussi comme doubles. Chaque vf_* est un booléen de même nom dans le schéma.

### 5.3 Cible [O]

- `rec+0xd` = 0 : cible = soi. = 1 : cible sélectionnée. = 2 : **cible = 0 et position choisie à la souris** pour les types d'effet
  271, 272, 303 et 153 (FUN_0079c590).
- Nuisible (`rec+0xc`) : accepté si `tf_monster` (0xc2) ou `uf_enemy` (0xbf) ; cadavre exigé si `rec+0xe`.
- Bénéfique, cible amie (groupe, vtbl+0x280…) ; sinon redirection sur soi si `tf_avatar`, sur la créature si `tf_summon` ;
  sans cible : soi si bascule (`rec+0xf`) ou `tf_avatar` / `tf_summon`. Les autres uf_* (self, party, guild, neutral, purple)
  n'ont pas de lecteur certain : le filtrage fin appartient au serveur.

### 5.4 Portée, position, niveau [O]

- `cast_range` et `valid_range` × **12** unités monde ; `cast_range` = −1 → portée d'attaque (vtbl+0x210).
- À portée si `distance_2D − (rayon_lanceur + portée + rayon_cible) < 0` (ou distance nulle) et si vtbl+0x144 ne signale pas d'obstacle ;
  rayon = vtbl+0x120 × échelle × 12 (échelle 1 pour un avatar).
- (x, y, z) = position de la demande (souris) pour une compétence de zone ; sinon contenu de la demande, sans garantie.
- Niveau : si `rec+0x4e` (cost_energy_per_skl) = **1**, le client envoie **le nombre de points d'énergie** du joueur comme niveau.
- 4001 / 4002 : `target` = handle de la carte (voir `summon_protocol.md`).

## 6. Recharges et listes de compétences

- **403** : `modification_type` 1 = vider puis remplir (connexion) ; 0 = mise à jour par id ; autre = ignoré. Cible = joueur,
  créature connue ou familier connu, sinon ignoré [O]. `base_lv` sert au plafond, `cur_lv` au niveau affiché [D].
- total_cool / remain_cool en **cs** (× 10 → ms). Le parseur force `total ≥ remain`. Début de recharge =
  `maintenant − (total − remain) × 10`, active si `remain ≠ 0` [O]. En mise à jour, si `remain ≠ total`, l'ancien total est gardé.
- **FIRE et REGION_FIRE démarrent seuls la recharge** avec le **dernier total connu** (FUN_0070d840) pour **toutes** les compétences
  du joueur de même `cool_time_group_id` (`rec+0xb6` ≠ 0, FUN_0070cfa0), puis marquent l'emplacement occupé. Si total = 0, aucune
  recharge n'est visible : **envoyer total_cool dans 403 dès la connexion**, et à chaque changement.
- CASTING marque l'emplacement « en cours » ; CANCEL l'efface (et le groupe) sans recharge ; COMPLETE ne touche pas la recharge.
- 404 : niveaux ajoutés (objets, états) ; liste remplacée entièrement à chaque paquet. 451 : non suivi.

## 7. db_skill : sens des champs lus par le client

| Champ | Lecture [O] |
|---|---|
| is_passive (0xa) | **0 = passive** (texte 391) : nom inversé |
| is_physical_act (0xb) | **1 = magique** : texte d'échec de sort, pas de reprise d'attaque. Mesure : types 231/232/261–271 à 1 pour 189/201, types 300xx à 0 pour 252/264. Nom **inversé** |
| is_harmful (0xc), is_need_target (0xd), is_corpse (0xe), is_toggle (0xf) | §5 |
| elemental (9) | 0 normal, 1 feu, 2 eau, 3 vent, 4 terre, 5 lumière, 6 ombre |
| casting_type / casting_level (0x10 / 0x11) | **aucun lecteur trouvé** ; valeurs 0/1/2 : 2253/376/113 |
| cast_range / valid_range (0x16 / 0x1a) | × 12 |
| cost_hp, cost_mp | `base + per_skl × niveau` (info-bulle) ; `cost_mp_per_enhance × ep` affiché à part |
| delay_cast (0x9a) | `(base + per_skl × niveau) / 100` s ; `delay_cast_mode_per_enhance` = fraction : réduction `(base + per_skl×lv)/100 × ep × mode` |
| delay_common (0xa6) | cs, utilisé pour la vitesse d'animation de tir des créatures (FUN_00971200) |
| delay_cooltime (0xaa) | `base/100 + per_skl/100 × niveau` s ; réduction `(base / 100 entier) × ep × mode` |
| cool_time_group_id (0xb6) | recharge commune si ≠ 0 |
| skill_lvup_limit (0xc3) | 0 : bonus génériques d'objets par famille (FUN_0070d070) |
| target (0xc7) | animation : 1, 4, 31 sur soi → mouvement 0x5a ; sinon 0x5d (soi) / 0x5b (bénéfique). Zone : serveur |
| effect_type (0xc9) | formules §8, choix souris, menace (501–503, 505–507, 511, 512 « soin ») |
| skill_enchant_link_id (0xcb) | compétence liée : sa formule est ajoutée avec `vN = v1N × niveau + v2N` |
| state_id (0xcf) | état posé ; d'autres états viennent des variables selon effect_type (§8.4) |
| niveau d'état | `state_level_base + int(per_skl × niveau) + int(per_enhance × ep)` |
| durée d'état | `(state_second + per_level × niveau) / 100` s, plus `per_enhance × ep / 100` ; bascule ou 0 → texte sans durée |
| probabilité | `probability_on_hit + probability_inc_by_slv × niveau` (%) |
| hit_bonus, hate, critical | `hit_bonus_per_enhance × ep` ; `int(hate_basic + hate_per_skl × niveau)` + `hate_per_enhance × ep` ; `critical_bonus + per_skl × niveau` (%) |
| toggle_group, cost_havoc, need_*, percentage, hate_mod, is_projectile | pas de lecteur certain dans cette passe [?] |

### 7.1 Surcharges codées dans l'exécutable [O]

Après chargement, FUN_00673640 écrase **100 valeurs** (tables 0xe91128, 78 entrées, puis 0xe91608, 22 entrées ; entrées de 16 o. :
id, champ, ancienne valeur, nouvelle valeur). Champ 1..20 = var ; 21 = delay_cooltime (s) ; 22 = state_second_per_level (inutilisé).
L'ancienne valeur n'est pas lue par le code ; elle ne coïncide pas toujours avec le fichier libre (voir §13). **Le fichier seul ne donne donc pas
les valeurs affichées.** Valeurs finales (87 compétences) :

3151 cool=40 ; 21102 v9=0,045 ; 21103 v1=1 ; 21111 v3=0,04 v6=0,03 ; 21302 cool=15 ; 21304 v2=0,2 ; 21311 cool=15 ;
40047 v1=23 ; 40049 v5=75 ; 40053 v5=7,5 ; 40054 v1=6 ; 40056 v2=75 ; 40057 v2=75 ; 40370 v2=0,1 ; 40371 v2=0,09 ;
40372 v2=0,075 ; 40373 v2=0,12 ; 40376 v2=0,1 ; 40377 v2=0,09 ; 40378 v2=0,08 ; 40721 v1=0,04 ; 40722 v1=0,8 ; 40723 v1=0,9 ;
40724 v1=0,1 ; 40763 v5=0,1 ; 41001, 41002, 41003 v2=v7=v12=0,075 ; 41311 cool=20 ; 41333 v3=0,01 ; 45101 v1=0,4 ; 45104 v1=0,5 ;
45106 v1=0,5 ; 45107 v1=0,5 ; 45111 v1=0,6 ; 45116 v1=1,5 ; 45117 v1=0,6 cool=60 ; 45120 v1=0,5 ; 45121 v1=1,2 ; 45125 v1=1 ;
45127 v1=1,2 ; 45135 v1=0,5 ; 45203 v1=1,2 ; 45206 v1=1,2 ; 45208 cool=25 ; 45210 v1=1,3 ; 45211 v1=1,3 ; 45217 v1=1,3 ;
45221 v1=1,4 ; 45222 v1=1,3 ; 45223 cool=30 ; 45225 v1=1,3 ; 45227 v1=1,4 ; 45228 cool=15 ; 45315 v1=0,4 ; 45404 v1=0,9 ;
45416 v1=1,1 ; 45418 v1=1,2 ; 45419 v1=1 ; 45423 v1=0,9 ; 45426 v1=0,6 ; 45428 v1=1,2 ; 45505 v1=1,7 ; 45515 v1=2 ; 45516 v1=2 ;
45517 v1=1,8 ; 45518 v1=1,7 ; 45519 v1=1,8 ; 45521 v1=0,3 ; 45530 v2=0,15 v3=0,04 ; 47016 v1=2,7 ; 47021 v1=2,5 ; 47023 v1=0,4 ;
48203 cool=300 ; 48208 v1=0,35 v2=0,05 ; 48209 v1=1,2 ; 48210 v3=30 ; 48211 v1=0,6 ; 48212 v1=0,6 ; 48213 v1=0,6 ; 48214 v1=0,5 ;
48218 v1=500 v2=0 ; 48245 v1=0,15 ; 48248 v1=57 ; 48258 v1=0,9 ; 48259 v1=0,439 ; 61002 v2=4,5.

## 8. Info-bulles : formules réellement évaluées

### 8.1 Évaluateur [O]

Le client **compile et évalue des expressions** (`rp::expr<float>`, FUN_00776780 ; compilation FUN_006ead30). Le texte source est
**db_string[2 000 000 000 + effect_type]** (FUN_006eb020 ; bonus d'amélioration seul : 2 100 000 000 + effect_type, FUN_007ae300).
Mesure : 145 formules de la première plage, 60 de la seconde, présentes dans `db_string(windows-1252).rdb` (fichier libre).

- Opérateurs : `( ) + - * /`, `&` concaténation, `$n` texte db_string n, `,`.
- Variables : `fx.v1..fx.v20` (var1..20 **après surcharges**), `slv` niveau, `ep` amélioration (carte), `att`, `mag`, `cooltime` (s),
  `castingdelay` (s), `mp` (coût), `sv.ca.basehp`, `sv.ca.slvhp`, `maxmp`, `curmp`.
- `att` / `mag` = **banque A + banque B** des statistiques dérivées, dword +4 / +0x14 du bloc de 82 o. (0x6ee990, 0x6ee9b0 ; voir
  `../SFRAME_STAT_BANKS_RE.md`) : attaque et attaque magique totales [O lecture, D noms].
- Fonctions : int, absint, signedint, float(x, n), absfloat, signedfloat, if, signedif, onesignedif, unsignedif, rep,
  skilldb.nameid, tenacitydb.nameid, itemdb.item / nameid, racename, attributename, descentname, bitsetname_a / _b.

Ce sont des **valeurs d'affichage** avant défense, résistance, élément, critique et niveau de la cible. Elles ne prouvent pas la
formule du serveur, mais ce sont les seules formules présentes dans le client. Aucune statistique de la cible n'intervient.

### 8.2 Compétences actives (nombre de lignes db_skill, fichier libre de 2 742)

Notation : vN = fx.vN, L = slv ; « +ep: » = supplément par point d'amélioration.

| effect_type (n) | Expression affichée |
|---|---|
| 30001 (61), 30013 (12), 30004 (9) | dégâts `att·(v1+L·v2)+v3+L·v4` ; +ep: 30001 `att·v11+v12`, 30013 `v12` ; 30013 rayon v5 |
| 30002 (5) | idem 30001 ; vol PV 100·v5 %, PM 100·v6 % |
| 30006 (5), 30008 (1) | idem ; 30006 recul `v9+L·v10`, immobilisé `v11+L·v12` s, +ep `att·v5+v6` ; 30008 recul `v10+L·v11`, `v12+L·v13` s, rayon v5 |
| 30009 (11), 30016 (11), 30017 (2) | par coup, coups `v5+L·v6` ; 30009/30016 +ep `att·v11+v12` ; 30017 recul `v9+L·v10`, `v11+L·v12` s |
| 30011 (85), 30012 (43), 30015 (2), 30018 (10) | zone : +ep `att·v11+v12` (30015 `v12`), rayon v5 (sauf 30018) ; coups 30012 et 30018 `v8+L·v9`, 30015 `v10+L·v11` |
| 30020 (1) | `att·((v1+L·v2+ep·v11)+(v3+L·v4+ep·v12)) + mag·((v13+L·v14+…)+(v16+L·v17+ep·v18))` ; rayon v5 (texte fautif : « ep.fx.v15 ») |
| 32021 (6) | mag : `mag·(v1+L·v2)+v4+L·v5` **et** att : `att·(v7+L·v8)+v9+L·v10` |
| 32251 (1) | `att·(v1+L·v2)+v3+L·v4+curmp·(v5+L·v6)+maxmp·(v9+L·v10)` ; consomme 100·(v7+L·v8) % des PM actuels et 100·(v11+L·v12) % des PM max |
| 101 (14), 111 (22), 113 (6) | `+ v1+L·v2`, +ep v5 ; 111 rayon v3 (+ep v8), max v12 cibles ; 113 rayon v9, max v4 |
| 102 (8) / 103 (2) | `v1+L·v2` par coup (+ep v6), coups `v3+L·v4` / `v1+L·v2` % (+ep v5) |
| 117 (1) / 151 (6), 152 (4) | `v1+L·v2`, interruption `v3+L·v4` % / 151 : `v1+L·v2` (+ep v10), 152 : `att+v1+L·v2` (+ep v5) ; recul `v6+L·v7` (+ep v8), immobilisé `v9+L·v10` s (+ep v11) |
| 201 (3), 211 (1) | `mag+v1+L·v2`, +ep v3 / v6 |
| 231 (56), 261 (60), 265 (2) | `mag·(v1+L·v2)+v4+L·v5`, +ep `mag·v3+v6` |
| 232 (17), 233 (2) | idem par coup, coups `v7+L·v8` ; 233 coût PV de créature `v10+L·v11` |
| 234 (4) | `mag·(v1+L·v2+ep·v3)` ; cible à plus de `v10+L·v17` % et moins de `v11+L·v18` % PV : mort immédiate à `v7+L·v8` % |
| 235 (3) | idem 231 ; vol PV 100·(v7+L·v8) %, PM 100·(v10+L·v11) % |
| 237 (6) | dégâts PM `v1+L·v2`, PV `v4+L·v5` |
| 262 (14), 263 (18) | zone rayon v10, `mag·(v1+L·v2)+v4+L·v5` ; 262 +ep `mag·v3` ; 263 +ep `mag·v3+v6`, coups `v7+L·v8` |
| 267 (1) | dégâts = 100·(v1+L·v2) % des PV max (de qui : non précisé) (+ep 100·v3) |
| 271 (36), 273 (2) | au sol : 271 `mag·(v1+L·v2)+v4+L·v5`, durée `v7+L·v8` s, rayon v10, retour 100·v11 % ; 273 soin `mag·(v1+L·v2)+v4+L·v5`, dégâts `mag·(v7+L·v8)+v10+L·v11`, durée `v13+L·v14`, rayon v16 |
| 381 (1), 382 (6), 384 (2) | piège : déclenchement v1, dégâts `v2+L·v3` (381 physiques), durée `v4+L·v5` s, rayon v6 ; 384 délai `v8+L·v9` s |
| 1001 (3) | `v1+L·v2`, rayon v3 |
| 32062 (2) / 32063 (4) | chaîne dégâts / soin `mag·(v1+L·v2+ep·v3)+ep·v4` ; bonds `v5+L·v6`, distance v10, variation 100·(v7+L·v8) % par bond, cumul si v11 |
| 353 (6) | zone de soin PV `v1+L·v2`, PM `v3+L·v4`, SP `v5+L·v6`, durée `v8+L·v9` s, rayon v10 |
| **501 (42), 521 (10)** | soin PV `mag·(v1+L·v2)+v3+L·v4` (+ep v7) **plus** 100·(v5+L·v6) % (+ep 100·v8) ; 521 rayon v11 |
| 502 (2) | soin PM même forme ; `v5+L·v6` % des PM max |
| 504 (6), 30501 (4) | résurrection : 504 PV `L·v1` %, PM `L·v2` %, EXP `L·v3` % ; 30501 PV 100·(v1+L·v2) %, PM 100·(v3+L·v4) %, EXP 100·(v5+L·v6) % (+ep 100·v10/v11/v12) |
| 505 (5), 507 (3), 508 (2) | PV `mag·(v1+L·v2)+L·v3[+v11]` + 100·L·v4 % ; PM `mag·(v6+L·v7)+L·v8[+v12]` + 100·L·v9 % ; 507 % = 100·(v13+L·v4) et 100·(v14+L·v9) ; 508 sans v11/v12, rayon v11 |
| 509 (17) / 510 (9) | PV / PM `mag·(v1+L·v2)+v3+L·v4` (+ep v7) et `v5+L·v6` % (+ep v8, **sans** ×100) |
| 511 (6) | PV `v1+L·v2`, PM `v3+L·v4` (+ep v5 / v6) : cohérent avec la règle du cadavre déjà implantée |
| 512 (2) | coûte à la créature PV `v1+L·v2`, PM `v3+L·v4` ; rend × v7 |
| 302 (173) / 304 (3) / 307 (3) / 310 (1) | rayon v6 / interruption `v2+L·v3` % / éléments v7..v13 / état v1 pendant `v16+L·v17+ep·v18` s |
| 401 (2), 402 (2) | dissipation des états v1 (402 : v1, v5..v9 ; tous si v9) jusqu'au niveau `v2+L·v3` (402 `L·v2`) |
| 314 (2) | consomme l'objet v1, quantité v2..v20 par niveau |
| 701 (177), 702 (4) | bascule / aura : `v1+L·v2` PM toutes les 5 s (+ep v3) |
| 900 (2), 903 (1), 904 (1) | menace `v1+L·v2` (903 en %), réussite `v6+L·v7` % ; 903 rayon v3 ; 904 taux réduit de `v1+v2` %, durée `v16+L·v17` s |
| 9001 (73) | crée l'objet `v1+L·v2` en quantité `v3+L·v4` |
| 30601 (3) | réussite `v1+L·v2` %, nombre d'états retirés `v4+L·v5` |
| 30701 (2) | coût en rupy v1 ; multiplicateurs v5 / v8 / v11 à v3 / v6 / v9 % |

**Sans formule de valeur** (texte seul ou aucune) : **301 (612 lignes, état simple)**, 104, 112, 121, 131, 236, 239–241, 268, 311, 313, 503,
522, 601–605, 901, 9501–9503, 30019, 30022, 32261 : le client ne donne aucune indication de calcul pour elles.

### 8.3 Passifs

| effect_type (n) | Valeur affichée |
|---|---|
| **3 (31) / 4 (32)** | code en dur FUN_006ee110 : six triplets `(masque, base, par niveau)` = v1–3, v4–6 (banque A), v7–9, v10–12 (banque B), v13–15, v16–18 (banque A) ; valeur `base + L·par niveau`, **× 100 %** pour le type 4 |
| 10001 (15) | ATT `v1+L·v2`, vit. att. v3/v4, précision v5/v6, MATT v7/v8, critique v9/v10, ATT % 100·(v11+L·v12) |
| 10006 (36) | 4 couples élément vk, `v(k+1)+L·v(k+2)` % |
| 10008 (83) / 10011 (29) | L·v1..L·v10 = ATT, DEF, MATT, MDEF, vit. att., déplacement, précision, préc. mag., esquive, rés. mag. ; 10011 × 100 % |
| 10009 (37) / 31003 (3) | L·v1..L·v12 = regen PV, recup. PV, regen PM, recup. PM, blocage %, blocage, critique, force critique, vit. incantation, recharge, interruption, distance ; 31003 × 100 % |
| 10010 (25) | vit. att. 100·(v2+L·v3), critique v4/v5, PV max v6/v7, PM max v12/v13, regen PV v8/v9, regen PM v10/v11 (% ×100) |
| 10013 (8) | par type v1 / v6 / v11 : dégâts infligés / subis 100·(…) % |
| 10015 (49) / 10042 (50) | FOR, VIT, AGI, DEX, INT, SAG, CHA = v(2k−1)+L·v(2k) ; 10042 × 100 % |
| 10021 (15) / 10032 (4) / 10045 (6) | PV max v1/v2, PM max v3/v4, récup. PV v7/v8, PM v9/v10 ; 10032 % (récup. v7..v10) ; 10045 PV/PM max % |
| 10043 (57) / 10044 (4) / 10037 (3) | dix couples v(2k−1)+L·v(2k) (ordre de 10008 / 10009) ; 10043 et 10044 × 100 % |
| 31001 (9) | ATT, vit. att., précision, MATT, critique, vit. incantation : 100·(…) % |
| 10048–10051 (26), 10052–10058 (17) | déclenchement de l'état v1 : 10048–10051 chance `v7+L·v8` %, durée `v5+L·v6` s ; 10052+ conditions PV v15..v18 (%), PM v20 |
| 10063–10065, 32281 | recharges : toutes `v2+L·v3` s, compétence v4 `v5+L·v6`, v7 `v8+L·v9` |
| 32001, 32011, 32032, 32171, 32183, 32192, 32202, 32211, 32262–32264, 32271, 32292, 32301–32304 | bonus de créature, modification de compétence (cooltime, mp, castingdelay), résistances d'état, conversions de statistique (100·(v2+L·v3) % de la statistique v13/v14 vers v1, etc.) |

Masques banque A (bit → statistique) : 0 FOR, 1 VIT, 2 AGI, 3 DEX, 4 INT, 5 SAG, 6 CHA, 7 ATT, 8 MATT, 9 DEF, 10 MDEF, 11 vit. att.,
12 vit. incantation, 13 déplacement, 14 précision, 15 préc. mag., 16 critique, 17 blocage %, 18 blocage, 19 esquive, 20 rés. mag.,
21 PV max, 22 PM max, 23 SP max, 24/25/26 récup. PV/PM/SP, 27/28 regen PV/PM, 30 poids max. Banque B : 0–6 résistances (normal, feu,
eau, vent, terre, lumière, ombre), 14–20 dégâts élémentaires, 21–27 dégâts supplémentaires, 28 force critique [O FUN_0078b050 / FUN_0078c5d0].
Bits 2/3 : l'ordre AGI puis DEX est celui des textes 7103/7104 lus par ces fonctions.

### 8.4 États posés par une compétence (FUN_006ed580) [O]

`state_id`, plus selon effect_type : 301, 302, 307, 605 → v1..v5 ; 304 → v5 (niveau `v6+L·v7`) ; 308 → v1, v2, v3 (niveaux
`v4+L·v5`, `v8+L·v9`, `v10+L·v11`) ; 309 → v1 (niveau `v2+L·v3`) ; 311 → v1..v10 ; 701 → v4, v5 ; 702 → v4..v8 ; 904 → v11 (niveau
`v12+L·v13`) et v5 ; 10048–10062 → v1 ; 30601 → v13..v15. Un état de type 88 ajoute l'état `value_0`. À niveau non précisé :
règle de `state_level_*` (§7).

### 8.5 Effets d'état affichés (FUN_00788890 / FUN_007889f0), lv = niveau d'état

| Champ | Valeur affichée [O] |
|---|---|
| base_effect_id 1, 2 | dégâts toutes les `fire_interval` (unité brute affichée en s) : `(amplify_base + amplify_per_skl·lv) × att + add_damage_base + add_damage_per_skl·lv`, élément `st+0x34` |
| base_effect_id 3, 4 | même forme avec `mag` |
| base_effect_id 11 / 12 | soin PV / PM périodique, même forme avec `mag` |
| effect_type 1 (999) / 2 (419) | six triplets (masque, base, par niveau) : value 0–2, 3–5, 12–14, 15–17 (banque A), 6–8, 9–11 (banque B) ; type 2 × 100 % |
| 3, 4 | comme 2, autre texte d'introduction |
| 8 | masque value_0, `value_1 + lv·value_2` |
| 21 | attaque supplémentaire : `value_0 + lv·value_1` %, armes value_8..11 |
| 22 / 23 / 24 / 25 | dégâts supplémentaires `value_0 + lv·value_1` (23 et 25 × 100 %) avec probabilité `value_6 + lv·value_7` % ; 24/25 pour les compétences |
| 26 | pose l'état value_0 avec probabilité `value_6 + lv·value_7` |
| 27 / 34 | à l'attaque / en critique : PV `v0+lv·v1`, PM `v2+lv·v3`, SP `v4+lv·v5` (27 ; probabilité v6+lv·v7 ; 34 : probabilité v4+lv·v5) |
| 35 | taux de récupération PV 100·(v0+lv·v1) % + (v4+lv·v5), PM 100·(v2+lv·v3) % + (v6+lv·v7) |
| 41 | vol PV 100·(v0+lv·v1) %, PM 100·(v2+lv·v3) %, probabilité `v6+lv·v7` |
| 44 | renvoi `v0+lv·v1`, probabilité `v6+lv·v7` |
| 49, 86 / 83, 91, 111 | 100·(v0+lv·v1) / `v0+lv·v1` |
| 36–40, 42, 43, 46, 61, 64, 73, 74, 81, 82, 92, 93, 109, 3xxx | fonctions dédiées, **non transcrites** |

## 9. Séquences [D sauf formats]

- **Connexion** : 403 `modification_type` 1 avec total_cool de chaque compétence (et remain si en recharge) → 404 → 505 par état
  (après l'ENTER du joueur et TS_TIMESYNC) → 407 par aura active. Pour un observateur : ENTER puis 505 de chaque état visible.
- **Instantanée** : 400 → FIRE (caster_hp/mp après coût, impacts avec target_hp finaux) → 403 `modification_type` 0 (total, remain) au
  lanceur ; COMPLETE facultatif (reprise d'attaque automatique).
- **Avec incantation** : 400 → CASTING (durée cs, erreur 0, PV/PM **actuels**) → [UPDATE si la durée change] → FIRE → 403.
- **De zone / au sol** : 400 avec target 0 et x, y, z → FIRE : x, y, z, `range`, `target_count`, une entrée par cible. Impacts
  suivants (pluie, zone persistante) : REGION_FIRE.
- **À état** : FIRE avec RESULT (succès 1, genre 10) par cible **puis** 505 par cible (state_handle unique par porteur,
  start = maintenant, end = maintenant + durée). Échec : RESULT succès 0. Dégâts + état : impact 0/1 puis 505.
- **Périodique** : 406 à chaque tic (type 1/2/4/5), dernier tic avec fin = 1 et total ; puis 505 niveau 0.
- **Soin** : FIRE avec 20 / 21 / 22 (montant réel, valeur finale après plafonnement) ; 509 facultatif.
- **Ruée** : FIRE avec 30 en plus des dégâts, **puis 509** pour le lanceur et les cibles (§3.3).
- **Résurrection** : FIRE avec 23, STATUS_CHANGE / 509, EXP_UPDATE.
- **Annulation** (mouvement, 150, dégâts) : CANCEL ; ni coût ni recharge. **Refus** : CASTING avec erreur (message), ou
  TS_SC_RESULT(400, code) sans message.
- **Bascule / aura** : FIRE + RESULT genre 21 / 22, 407, 505 (niveau > 0 / 0), coût PM périodique par 509.

## 10. Divergences avec l'existant

| Existant | Constat |
|---|---|
| `ScState.remain_time` | +0x13 est une **heure de fin** (arTime cs), −1 = permanent |
| `skill` : `is_physical_act` | vaut 1 pour les compétences **magiques** (comme `is_passive`, déjà corrigé) |
| TS_CS_SKILL ← FUN_0096b840 | version créature ; joueur = FUN_0096f260 ; `layer` toujours 0, niveau = énergie si `rec+0x4e` = 1 |
| « CANCEL = 9 octets nuls » | aucun octet de queue lu ; inoffensif |
| impacts DAMAGE marqués [D] | confirmés : damage +0xa, flag +0xe, éléments +0x12 **inclus** dans damage |
| HPMP après chaque FIRE | non requis (sauf RUSH, SP, maxima) |
| recharge via 403 seulement | FIRE démarre la recharge locale ; 403 doit surtout fournir **total_cool** à l'avance |
| valeurs db_skill | 100 surcharges dans l'exécutable (§7.1) |
| REBIRTH, RUSH, CHAIN_* | dispositions ci-dessus ; noms tirés de sources publiques [D] |

## 11. Limites / inconnues

- Aucune formule de dégâts **finale** (défense, résistance, précision, critique, niveau) n'existe côté client ; les 612 compétences de
  type 301 et les types sans formule restent à définir serveur.
- `casting_type`, `casting_level`, `toggle_group`, `percentage`, `hate_mod`, uf_* (hors uf_enemy), sens de `target` pour le choix des
  cibles : non établis. Sens exact de multiple / fire_count : copiés, usage graphique non suivi.
- Champs +0x11 de REBIRTH, +0x29 de KNOCK_BACK, +0x20 / +0xd des chaînes, octets de 404 et entrées de 451 : [D] ou [?].
- Expiration locale d'un CASTING sans suite : aucune trouvée, absence non prouvée.
- TS_SC_RESULT(400) : message éventuel dans une fenêtre non examinée.
- Formules : lues dans le fichier **libre** db_string (windows-1252) et db_skill libre ; une archive plus ancienne existe et la priorité
  à l'exécution n'est pas prouvée. La priorité d'opérateurs de `rp::expr` n'a pas été désassemblée (les formules sont
  entièrement parenthésées ou sans ambiguïté). Les libellés d'états « fonctions dédiées » (§8.5) ne sont pas transcrits.
- Unité de `fire_interval` : affichée telle quelle ; `delay_*` et `state_second*` sont en centièmes (÷ 100) dans le client.
- La cohérence de `att` / `mag` avec les champs réseau de TS_SC_STAT_INFO repose sur `../SFRAME_STAT_BANKS_RE.md`.

## 12. Adresses

| Adresse | Rôle |
|---|---|
| 0xa2fed0 / 0xa2a7f0 / 0xa2ae90 | parseur S→C / 401 / 403 |
| 0x6b2130 / 0x6b06d0 / 0x6b0940 | monde / 401 / 505 |
| 0x6d2e60 / 0x6cff80 / 0x6d27c0 / 0x6c8d20 / 0x6d0490 | gestionnaire jeu / 401 (recharges) / 403 / 404 / 406 |
| 0x6da6a0 / 0x94f830 / 0x6e1400 / 0x6e1730 / 0x6e18d0 / 0x6e19f0 | file PV/PM : ajout / enregistrement / marquage / impact / application / vidage |
| 0xa3d5c0 / 0x7ac540 / 0x9df3b0 | impact joué / messages / nombres flottants |
| 0x9457f0 / 0x98b7c0 / 0x98c130 / 0x978cf0 | machine du joueur : réception / tic / demande ; actions du joueur |
| 0x988710 / 0x971200 / 0x9486c0 | monstre : réception / actions ; créature |
| 0x9e30f0 / 0x744f90 / 0x746150 / 0x747320 | erreurs d'incantation / jauge : ouverture, mise à jour, fermeture |
| 0x6d1cc0 / 0x6d8d40 / 0x6c6f40 / 0x6c8860 / 0x79c590 / 0x96f260 / 0x96b840 | contrôles, cible, armes, IHM, envoi joueur / créature |
| 0x98b690 / 0x98b4c0 | portée / rayon |
| 0x70c9d0 / 0x70cfa0 / 0x70d840 / 0x70d7e0 / 0x70d780 | emplacement : 403 / groupe / FIRE / CASTING / CANCEL |
| 0x9549e0 / 0x94eff0 / 0x951a50 / 0x705bd0 | état : recherche / ajout, mise à jour / retrait / durées |
| 0x637840 / 0x637640 / 0x6377b0 / 0x637600 / 0x637800 | db_tenacity : chargeur / nom / info-bulle / icône / nuisible |
| 0x673640 / 0xe91128 / 0xe91608 | db_skill : chargeur / surcharges |
| 0x787fd0 / 0x776780 / 0x6eb020 / 0x6ebb10 / 0x6ed580 / 0x6ee110 / 0x788890 / 0x7889f0 | info-bulle, évaluateur, formules, états, passifs, effets d'état |

## 13. Priorité des fichiers et contrôles indépendants (2026-11-24)

| Point | Constat |
|---|---|
| Priorité fichier libre / archive | **tranchée [O]** : `KFileManager::CreateStreamFromResource` (FUN_005c23c0, `ev_c4daa0c4a4a8dcfd5edfe852c953994a1f055db876273659d6aa76fb5eaf02d3`) parcourt les chargeurs dans l'ordre et prend le premier qui répond ; FUN_005c0690 les ajoute en fin de liste (`ev_06495ea8392aa79fa0b4d4feeb180819c4510a2a9ad30aaae0ba311263d9b1e9`) ; FUN_00b87450 enregistre `secondres`, `country`, `./Resource/`, puis `data.000` en dernier (`ev_3fad4ea3f5da740c003b4a95befc59459dba30194c92e48a4ed1abbd808d94e9`). Le fichier libre l'emporte |
| Surcharges (§7.1) | relues dans l'exécutable : 100 entrées, 87 compétences. FUN_00673640 **écrase sans comparer** (`ev_4af7d5c54f7189a1b28dca6353a5a1e2c2b47d9ed16d46fdf92e30ae6aceef79`) : le 3ᵉ champ (« ancienne valeur ») n'est jamais lu |
| Rectification de §7.1 | l'« ancienne valeur » ne coïncide **pas** toujours avec le fichier libre : 54 sur 100 (45117 : 1,4 dans le fichier, 0,7 dans la table). Elle correspond à l'archive pour 87 sur 100. `db_skill` existe en deux versions : 2 742 lignes (libre), 3 130 (archive). Les valeurs finales restent valables |
| Formules (§8) | relues dans `String.dbc` pour 231, 261, 501, 701, 302, 30001, 30009, 30011, 30012 : conformes ; aucune pour 301 |
| `is_physical_act` | confirmé inversé : 0 pour 59/61 compétences 30001 et 84/84 de 30011, 1 pour 55/56 de 231 |
| Soin en % (501) | libellé du client : « Augmente la récupération de HP de …% » ; l'assiette (PV max ?) n'est **pas** établie |

## 14. Implémentation RappelzCore (2026-11-24) — premier lot

| Élément | Choix |
|---|---|
| Périmètre | cible unique seulement : 30001 (physique), 231 (magique), 501 (soin), 601 / 602 (invocation, `summon_protocol.md`) ; 47 compétences. Écartées : zones (30011, 261…), états (301, 302…), bascules, passifs, compétences liées, coûts ou conditions particuliers |
| Valeur | formule d'info-bulle (§8), surcharges de l'exécutable (§7.1) comprises ; attaque physique et magique = celles envoyées dans STAT_INFO ; aucun bonus d'énergie (`ep` = 0) |
| Défense | **provisoire** : valeur − défense / 2, ± 10 %, au moins 1 (règle du combat de base ; rien côté client) |
| Soin en % (501) | appliqué aux PV max de la cible **[D]** : la 9550 (v5 = 1, reste nul) n'aurait sinon aucun effet |
| Déroulement | RESULT(400) ; CASTING (durée, erreur 0) si incantation ; FIRE : coût, effet, recharge du groupe ; 403 pour les compétences apprises du groupe ; mouvement, annulation ou condition perdue : CANCEL sans coût ni recharge |
| Observateurs | FIRE à ceux qui voient le lanceur ; 509 à ceux qui ne voient que la cible |
| Contrôles serveur | apprise, niveau, vivant, PM, recharge, immobile si incantation, cible visible et à portée (`cast_range` × 12 + marge) ; au tir : mêmes contrôles |
| Non contrôlé | arme requise (`vf_*`), `uf_*` / `tf_*` fins, obstacles, cadavre |
| Recharges | en mémoire du serveur seulement ; total annoncé dès 403 de connexion |

Validation (MariaDB de test isolée, `rz-bot --cast`, `--interrupt`) : 3011 (CASTING 50 cs puis FIRE, 12 PM par
tir, 538 PV → 0 en 11 tirs, mort récompensée) ; 2001 (FIRE immédiat, seconde demande refusée : recharge) ; mouvement
pendant l'incantation : CANCEL, PM intacts ; refus : non apprise, en mouvement, hors de portée ; 4001 (800 cs) puis
4002 (50 cs), et 4001 interrompue. Tests unitaires : dégâts, mort, recharge, annulation, soin plafonné, refus sur
joueur. **Non vérifié : client graphique d'origine** ; `modification_type` 1 à la connexion (le client RappelzRust
n'accepte que 0).

# Format des bases client Rappelz `.rdb` (+ `db_item.ref`)

Client analysé : `Rappelz NoLimits\SFrame.exe` (PE x86, sha256 `27f42fc8f6b08df8c239640b938d601aeaebb7b3db0c89fc47755325a98a410d`,
chemins de build `source9.5.2`), avec Ghidra 12.1.4 via REA. Fichiers étudiés :
`Rappelz NoLimits\Resource\*.rdb` et `db_item.ref`, ouverts **en lecture seule**.

Légende (même convention que `data_archive.md`) :
**[OBS]** lu dans le code du client ET vérifié sur les vrais fichiers ;
**[DON]** vérifié sur les données seulement (tailles exactes, ids de chaînes résolus, valeurs cohérentes) ;
**[DED]** déduit (nom repris de RappelzRDBTool, ou interprétation plausible non prouvée) ;
**[INC]** inconnu.

Script de validation (Node.js, Python n'étant pas installé sur ce poste) :
`C:\Users\Marty\Desktop\RappelzCore\docs\re\tools\rdb_dump.js`

```text
node rdb_dump.js "C:/Users/Marty/Desktop/RappelzCore/Rappelz NoLimits/Resource" all 2     # toutes les tables, 2 exemples chacune
node rdb_dump.js "<Resource>" item 5                                                    # une table
```

Il parse les 29 fichiers du dossier `Resource`, vérifie que chaque fichier est consommé
**exactement jusqu'au dernier octet**, résout les `*_id` texte dans `db_string(windows-1252).rdb` et
affiche des enregistrements. Résultat actuel : **29 OK, 0 FAIL**.

---

## 1. Sources de preuve

### 1.1 Chargeurs dans `SFrame.exe` [OBS]

Tous suivent le même schéma : `FUN_005c01c0(nom)` + `FUN_005c23c0(nom)` ouvrent un flux, puis
`memset(buf,0,0x80)` et **une seule lecture de 0x84 octets** (en-tête 0x80 + compteur), puis
`count` lectures de `sizeof(record)` (méthode `vtable+4` = `read(dst, len)`).

| Fichier | Fonction | Lecture par enregistrement | Remarques tirées du code |
|---|---|---|---|
| `db_item.rdb` + `db_item.ref` | `FUN_00644200` | `0x178` (376) | Lit d'abord `db_item.ref` ; copie l'enregistrement champ par champ dans une structure `ItemBaseEx` de `0x1854` octets ; message de debug du client patché : `sizeof(RDB_ITEM_95)=%u (cible 376)` et `item[%d]: id=%d name_id=%d type=%d class=%d wear=%d level=%d icon_id=%d tooltip=%d` |
| `db_monster.rdb` | `FUN_0064f990` | `0x228` (552) | `name_id` = `+0x8`, `model` = `+0xFC` (`char[256]`) ; debug `load key(uid)=%d name_id=%d model=[%s]` |
| `db_skill.rdb` | `FUN_00673640` | `0x33B` (827) | Structure **packée** ; patch post-chargement qui écrit `float` à `+0x10D + 4*n` (n = 1..20 → `var1..var20` à `0x111`), un `int` ×100 à `+0xAA`, un `int` à `+0xE3` |
| `db_job.rdb` | `FUN_00648920` | `0x124` (292) | `+0x20` u32, `+0x24` `char[256]` |
| `db_NPCResource.rdb` | `FUN_0064bbd0` | `0x3E5` (997) | Lit aussi `db_NPCEventPeriodResource.rdb` (16 o/enr.) ; teste `+0x24 & masque_région` ; `+0x34` `char[256]`, `+0x1E0` `char[0x200]` |
| `db_Quest.rdb` | `FUN_0065ee80` | `0x10C` (268) | indexé par `id` (`+0`) |
| `db_WorldLocation.rdb` | `FUN_00639f00` | `0x477` (1143) | Constructeur : `+0x41A = 100` par défaut (volume) ; zone `char` de `0x338` à `+0xDE` |
| `db_skilltree.rdb` | `FUN_00673e70` | `i32 n` puis `n × 0x40` | Format groupé (voir §4.3) ; message `[SSkillTreeDB] Create Stream Failed - [db_skilltree.rdb]` |
| `db_String(<cp>).rdb` | `FUN_006693d0` | variable | Voir §4.1 ; nom construit `"db_String" + "(" + codepage + ")" + ".rdb"`, repli sur `db_String.rdb` ; ensuite surcharges optionnelles `string_resource.txt` et `string_table.txt` (CSV) |

Dans plusieurs chargeurs (item, monster, NPC), l'`id` lu est ensuite **masqué en mémoire**
(clés aléatoires `FUN_005fc940`, champs ajoutés après l'enregistrement). Cela ne concerne pas le
fichier : la clé recalculée est égale à la valeur lue sur disque.

### 1.2 Définitions RappelzRDBTool [DED → confirmées DON]

L'en-tête des fichiers dit `RDB Created by RappelzRDBTool from Glandu2`. Les définitions de
champs de cet outil (dépôt public GitHub `glandu2/RappelzRDBTool`, dossier `Databases/lua/`,
GPLv2) ont été récupérées et **recalculées octet par octet** : pour les 23 tables fixes du dossier,
la taille calculée est égale à `(taille_fichier − 132) / count`, et le script relit chaque fichier
jusqu'au dernier octet. Les ids de texte (`name_id`, `text_id`, `tooltip_id`…) se résolvent dans
`db_string` à ~100 %. Les noms de champs ci-dessous viennent de ces définitions (fautes
d'orthographe d'origine conservées : `stati_id`, `sexsual_id`, `standard_walk_spped`…), sauf mention.

Correspondance version : `Item94` (format NoLimits/9.4+ avec `db_item.ref`), `Job91`, `Monster91`,
`Skill81`, `Npc83`, `Quest81`, `WorldLocation81`, `State81` (= `db_tenacity`), `Mix81` (=
`db_combineres`, **corrigé** : il manque `sub02_type_05` dans la définition, sinon 432 ≠ 436), etc.

### 1.3 Recoupement code ↔ définitions [OBS]

- **db_item** : la copie `RDB_ITEM_95 → ItemBaseEx` dans `FUN_00644200` place les champs aux
  offsets exacts de l'ancien format `Item91` (packé) : `+0x1C→rank`, `+0x38→range (EX+0x42)`,
  `+0x3C→weight`, `+0x50→wear_type (EX+0x5A)`, `+0x54 (u16)→bits limit_* (EX+0x5E)`,
  `+0x68→base_type_0 (EX+0x76)`, floats → doubles, `+0xD8 (u16)→cool_time_group`, `+0xDA (u8)→decrease_type`,
  `+0xDC (u8)→job_depth`, `+0xE0..0xE5 (6×i8)→model_type_*`, `+0xE8!=0→deco_model_change`,
  les 18 `i16` à `+0xF0` + `+0x114/+0x116/+0x11A` → **indices dans `db_item.ref`** recopiés en
  `char[256]` (`model_00..17`, `texture_filename`, `drop_type`, `icon_file_name`), `+0x118 (i16)→icon_id`,
  `+0x11C→tooltip_id (EX+0x184C)`, `+0x120→group_bit_set (EX+0x1850)`.
- **db_skill** : `var1` à `0x111` (patch `+0x10D+4n`), `delay_cooltime` à `0xAA`, `state_second_per_level`
  à `0xE3`, `tooltip_id 0x22A`, `icon_id 0x22E`, `icon_file_name 0x232`, `is_projectile 0x332`,
  `projectile_speed 0x333`, `projectile_acceleration 0x337` : tous identiques entre le constructeur du
  client et la définition `Skill81`.
- **db_npcresource** : `local_flag (0x24)` testé contre le masque de région, `model_file (0x34)`,
  `ai_script + contact_script (0x1E0, 2×256)`.
- **db_quest** : seul `+0x28` (champ de bits `limit_*`) est initialisé à part : cohérent.
- **db_job** : `unknown_value` (0x1E) vaut **163** dans les 42 lignes. RappelzRDBTool force cette même
  constante, ce qui confirme la correspondance.

---

## 2. Chiffrement / obfuscation

- **Les `.rdb` et `db_item.ref` du dossier `Resource\` sont en clair** [OBS] : en-tête ASCII lisible,
  chaînes lisibles, aucune transformation dans les chargeurs (lecture brute dans la structure).
- Les `.rdb` rangés **dans l'archive `data.00x`** passent par le flux XOR de l'archive (voir
  `data_archive.md`) : c'est une couche d'archive, pas une propriété du format RDB. [OBS, d'après data_archive.md]
- **Exception : `db_monster.id`** est stocké **bit-permuté** [DON+DED]. On le décode avec la permutation
  de RappelzRDBTool (`Monster91.lua`, `compute_bit_swapping`) :
  ```text
  enc[i] = i ; j = 3 ; pour i = 0..31 : swap(enc[i], enc[j]) ; j = (j + i + 3) % 32
  enc = [3,6,9,15,1,8,21,19,2,12,14,16,22,24,23,25,17,7,18,11,4,29,0,28,31,26,10,13,30,27,20,5]
  stocké = OR des (1 << enc[i]) pour chaque bit i de l'id réel ; décodage : dec[enc[i]] = i
  ```
  Vérification : les 7940 ids font l'aller-retour, et `name_id == id + 30000` pour 7911/7940
  (ex. stocké 8 → id 1 « Dragon Blanc Niv.1 », 64 → 2). Les ids de monstres cités par `db_quest`
  (1003, 1004…) n'existent qu'une fois décodés. Le client utilise la valeur stockée telle quelle comme clé.
- Les chaînes sont en **code page Windows** (le suffixe du nom de fichier `db_string(windows-125x)`
  l'indique). Il faut décoder avec la bonne code page, pas en UTF-8 (en Rust : `encoding_rs::WINDOWS_1252`).

---

## 3. En-tête commun (132 octets) [OBS]

Tout est **little-endian** ; les enregistrements sont **packés** (`#pragma pack(1)`, aucun alignement).

```text
date:    [u8; 8]    (0x00)  // ASCII "YYYYMMDD", ex. "20260729" (pas de NUL)
comment: [u8; 120]  (0x08)  // libre : zéros + texte, ex. à 0x10 "RDB Created by RappelzRDBTool from Glandu2"
                            //   ou "RDB Written with Daedalus v1.1.1 by iSmokeDrow." (0x08..0x0F = '.' dans ce cas)
count:   i32        (0x80)  // nombre d'enregistrements (db_skilltree : nombre de GROUPES)
records:            (0x84)  // count × sizeof(record), sans bourrage ni pied de fichier
```

Le client ignore les 128 premiers octets (il les lit dans un tampon jetable).

Conventions de type dans les listes ci-dessous :
`i8/i16/i32/f32/f64` = types Rust directs (signés) ; `i32 /*décimal ×100*/` = valeur ×100 (ex. `range` 400 = 4,00) ;
`[u8; N]` = chaîne NUL-terminée dans N octets (code page Windows) ; `flags_0xNN: u8` = octet de bits,
bit 0 = poids faible. Pour Rust : `#[repr(C, packed)]` + `zerocopy`/`bytemuck`, ou lecture champ par champ.

---

## 4. Formats non fixes

### 4.1 `db_string(windows-125x).rdb` : enregistrements de longueur variable [OBS]

En-tête standard (count à 0x80), puis pour chaque enregistrement :

```text
name_len:  i32           // longueur de name en octets (NUL de fin compris en général ; ici souvent 1 = "\0")
value_len: i32           // longueur de value en octets (souvent avec NUL et/ou bourrage de NUL)
name:      [u8; name_len]
value:     [u8; value_len]   // texte (balises <#rrggbb>, <br>, #@macro@#…)
code:      i32           // id de la chaîne (clé de recherche ; c'est lui que référencent name_id/text_id/tooltip_id…)
group_id:  i32           // 0 dans 99 % des cas (99, 1..7, −1 observés)
reserved:  [i32; 4]      // 0 sur disque (le client y range des pointeurs)
```

Le chargeur (`FUN_006693d0`) lit les deux longueurs, alloue `len+1`, ajoute lui-même le NUL et
indexe par `code`. **À faire en Rust : couper au premier NUL.**
Observé (`windows-1252`) : 150 632 chaînes, fichier consommé exactement (14 803 699 o). Les 5 variantes
(1250/1252/1254/1256/1257) ont le même format (147 637 à 150 632 chaînes). Exemples :
`10244201 → "Manteau de Cuir"`, `20244201 → "@#auto_tooltip#@Gaia Armure Invocateur"`, `10100 → "Vagabond"`,
`30001 → "Dragon Blanc Niv.1"`.

### 4.2 `db_item.ref` : table de chaînes de db_item [OBS]

Pas d'en-tête RDB :

```text
count:  u32 (0x0)   // 8989
strlen: u32 (0x4)   // 52
strings: [[u8; strlen]; count] (0x8)   // index 0..count-1, NUL-terminées
```

8 + 8989×52 = 467 436 = taille du fichier. Le chargeur refuse si `strlen-1 ≥ 0xFFF` ou `count-1 ≥ 999999`.
Les champs `i16` de `db_item` (`model_00..17`, `texture_filename`, `drop_type`, `icon_file_name`) sont des
**indices** dans cette table (index hors bornes ou négatif → chaîne vide).
Ex. : `2 = "icon_secrute_RefreshScroll"`, `448 = "wand_prominencerod"`, `704 = "dem_body_under001"`.

### 4.3 `db_skilltree.rdb` : format groupé [OBS]

`count` (0x80) = **nombre de groupes** (285). Puis, pour chaque groupe : `n: i32` suivi de `n`
enregistrements de 64 octets (voir §5). Observé : 9275 lignes, 132 + 285×4 + 9275×64 = 594 872 = taille du fichier.

---

## 5. Tables fixes

### 5.0 Récapitulatif (observé sur les fichiers)

| Fichier | count | taille enr. | 132 + count×taille | Définition | Statut |
|---|---:|---:|---|---|---|
| db_item.rdb | 34 682 | 376 | = 13 040 564 | Item94 | OBS |
| db_monster.rdb | 7 940 | 552 | = 4 383 012 | Monster91 | OBS (id bit-permuté) |
| db_skill.rdb | 2 742 | 827 | = 2 267 766 | Skill81 | OBS |
| db_job.rdb | 42 | 292 | = 12 396 | Job91 | OBS |
| db_npcresource.rdb | 1 497 | 997 | = 1 492 641 | Npc83 | OBS |
| db_quest.rdb | 997 | 268 | = 267 328 | Quest81 | OBS |
| db_worldlocation.rdb | 6 689 | 1 143 | = 7 645 659 | WorldLocation81 | OBS |
| db_string(windows-1252).rdb | 150 632 | variable | 14 803 699 | String81 | OBS |
| db_skilltree.rdb | 285 groupes / 9 275 lignes | 64 (+4 par groupe) | = 594 872 | SkillTree91 | OBS |
| db_dropgroup.rdb | 6 366 | 84 | = 534 876 | DropGroup81 | DON |
| db_skillstage.rdb | 1 469 | 160 | = 235 172 | SkillStage81 | DON |
| db_skilltreedisplay.rdb | 379 | 28 | = 10 744 | SkillTreeDisplay81 | DON |
| db_tenacity.rdb (= états/buffs) | 2 206 | 552 | = 1 217 844 | State81 | DON |
| db_combineres.rdb | 4 526 | 436 | = 1 973 468 | Mix81 corrigé | DON |
| db_itemeffectresource.rdb | 4 778 | 181 | = 864 950 | ItemEffect81 | DON |
| db_enhanceeffect.rdb | 35 988 | 208 | = 7 485 636 | EnhanceEffect91 | DON |
| db_fieldpropresource.rdb | 826 | 657 | = 542 814 | FieldProp81 | DON |
| db_questlink.rdb | 1 306 | 11 | = 14 498 | QuestLink81 | DON |
| db_monstermotionset.rdb | 352 | 100 | = 35 332 | MonsterMotionSet81 | DON |
| db_monsterspeciesresource.rdb | 162 | 12 | = 2 076 | MonsterSpecies81 | DON |
| db_motionfxset.rdb | 694 | 68 | = 47 324 | MotionFxSet81 | DON |
| db_soundresource.rdb | 699 | 260 | = 181 872 | Sound81 | DON |
| db_titleresource.rdb | 276 | 114 | = 31 596 | Title81 | DON |
| db_creatureenhance.rdb | 6 | 12 | = 204 | CreatureEnhance81 | DON |

### 5.1 `db_item.rdb` : 376 octets [OBS]

Les offsets confirmés par `FUN_00644200` sont listés en §1.3. `name_id` et `tooltip_id` se résolvent
à 34 682/34 682. Exemple : `id 244201, name_id 10244201 "Manteau de Cuir", type 1, group 2, class 204,
wear_type 2, weight 132.0, price 19400, base_type_0=15/var1 46/var2 4.8, icon "icon_clothe_gaia_summoner201",
model_00 "dem_body_under001", drop_type "drop_wear_robe", limit_gaia+limit_summoner`.

Taille calculée : **376 octets**.

```text
id: i32 (0x0)
name_id: i32 (0x4)
type: i32 (0x8)
group: i32 (0xc)
class: i32 (0x10)
set_id: i32 (0x14)
set_part_flag: i32 (0x18)
rank: i32 (0x1c)
level: i32 (0x20)
enhance: i32 (0x24)
socket: i32 (0x28)
status_flag: i32 (0x2c)
use_min_level: i32 (0x30)
use_max_level: i32 (0x34)
range: i32 /*décimal ×100*/ (0x38)
weight: f32 (0x3c)
price: i32 (0x40)
huntaholic_point: i32 (0x44)
ethereal_durability: i32 (0x48)
endurance: i32 (0x4c)
wear_type: i32 (0x50)
flags_0x54: u8 (0x54)  // bits LSB→MSB : 0=nv0, 1=nv1, 2=limit_deva, 3=limit_asura, 4=limit_gaia, 5=nv2, 6=nv3, 7=nv4
flags_0x55: u8 (0x55)  // bits LSB→MSB : 0=nv5, 1=nv6, 2=limit_fighter, 3=limit_hunter, 4=limit_magician, 5=limit_summoner, 6=nv7, 7=nv8
nv9: i16 (0x56)
item_use_flag: i32 (0x58)
material: i32 (0x5c)
summon_id: i32 (0x60)
throw_range: i32 (0x64)
base_type_0: i16 (0x68)
base_type_1: i16 (0x6a)
base_type_2: i16 (0x6c)
base_type_3: i16 (0x6e)
base_var1_0: f32 (0x70)
base_var1_1: f32 (0x74)
base_var1_2: f32 (0x78)
base_var1_3: f32 (0x7c)
base_var2_0: f32 (0x80)
base_var2_1: f32 (0x84)
base_var2_2: f32 (0x88)
base_var2_3: f32 (0x8c)
opt_type_0: i16 (0x90)
opt_type_1: i16 (0x92)
opt_type_2: i16 (0x94)
opt_type_3: i16 (0x96)
opt_var1_0: f32 (0x98)
opt_var1_1: f32 (0x9c)
opt_var1_2: f32 (0xa0)
opt_var1_3: f32 (0xa4)
opt_var2_0: f32 (0xa8)
opt_var2_1: f32 (0xac)
opt_var2_2: f32 (0xb0)
opt_var2_3: f32 (0xb4)
effect_id: i32 (0xb8)
enhance_id: i32 (0xbc)
skill_id: i32 (0xc0)
state_id: i32 (0xc4)
state_level: i32 (0xc8)
state_time: i32 (0xcc)
cool_time: i32 (0xd0)
available_period: i32 (0xd4)
cool_time_group: i16 (0xd8)
decrease_type: i16 (0xda)
job_depth: i16 (0xdc)
unknownValue1: i16 (0xde)
model_type_dem: i8 (0xe0)
model_type_def: i8 (0xe1)
model_type_asm: i8 (0xe2)
model_type_asf: i8 (0xe3)
model_type_gam: i8 (0xe4)
model_type_gaf: i8 (0xe5)
unknownValue2: i16 (0xe6)
deco_model_change: i32 (0xe8)
unknownValue3: i32 (0xec)
model_00: i16 (0xf0)
model_01: i16 (0xf2)
model_02: i16 (0xf4)
model_03: i16 (0xf6)
model_04: i16 (0xf8)
model_05: i16 (0xfa)
model_06: i16 (0xfc)
model_07: i16 (0xfe)
model_08: i16 (0x100)
model_09: i16 (0x102)
model_10: i16 (0x104)
model_11: i16 (0x106)
model_12: i16 (0x108)
model_13: i16 (0x10a)
model_14: i16 (0x10c)
model_15: i16 (0x10e)
model_16: i16 (0x110)
model_17: i16 (0x112)
texture_filename: i16 (0x114)
drop_type: i16 (0x116)
icon_id: i16 (0x118)
icon_file_name: i16 (0x11a)
tooltip_id: i32 (0x11c)
group_bit_set: i32 (0x120)
effect_id_01: i16 (0x124)
effect_id_02: i16 (0x126)
value_01: f32 (0x128)
value_02: f32 (0x12c)
value_03: f32 (0x130)
value_04: f32 (0x134)
value_05: f32 (0x138)
value_06: f32 (0x13c)
value_07: f32 (0x140)
value_08: f32 (0x144)
value_09: f32 (0x148)
value_10: f32 (0x14c)
value_11: f32 (0x150)
value_12: f32 (0x154)
value_13: f32 (0x158)
value_14: f32 (0x15c)
value_15: f32 (0x160)
value_16: f32 (0x164)
value_17: f32 (0x168)
value_18: f32 (0x16c)
value_19: f32 (0x170)
value_20: f32 (0x174)
```

Notes :
- [DED] `rank` (0..7), `level` (1/6/10), `enhance` (max d'enchantement : 0..25), `socket` (0..4) : noms
  RappelzRDBTool ; le debug du client appelle `level` le champ `0x20`.
- [OBS] `cool_time_group` est lu en u16 (0xD8), `decrease_type` en **u8** (0xDA), `job_depth` en **u8** (0xDC,
  masque de profondeurs, 15 = toutes) ; les octets 0xD9/0xDB/0xDD sont nuls dans toutes les lignes.
- [INC] `unknownValue1` (0xDE, 0..9), `unknownValue2` (0xE6, double des model_type), `unknownValue3`, `nv*`.
  `nv9` (0x56) est toujours 0.
- [DED] La fin `effect_id_01/02 + value_01..20` (0x124..0x177, 2 × 10 floats) n'est **pas lue** par le
  chargeur NoLimits. D'après les données, ce sont les bonus par niveau d'enchantement (ex. 2.2/2.6/3.1/3.9/3.9).
- `range` est un décimal ×100 (400 → 4,00 m).

### 5.2 `db_monster.rdb` : 552 octets [OBS]

`name_id` résolu 7940/7940. **`id` à décoder (§2).** `model` à 0xFC [OBS].

Taille calculée : **552 octets**.

```text
id: i32 (0x0)
monster_group: i32 (0x4)
name_id: i32 (0x8)
location_id: i32 (0xc)
transform_level: i32 (0x10)
walk_type: i32 (0x14)
slant_type: i32 (0x18)
size: f32 (0x1c)
scale: f32 (0x20)
target_fx_size: f32 (0x24)
level: i32 (0x28)
grp: i32 (0x2c)
affiliation: i32 (0x30)
species_id: i32 (0x34)
magic_type: i32 (0x38)
race: i32 (0x3c)
visible_range: i32 (0x40)
chase_range: i32 (0x44)
monster_type: i32 (0x48)
monster_grade_icon: i32 (0x4c)
stat_id: i32 (0x50)
monster_skill_link_id: i32 (0x54)
ability: i32 (0x58)
standard_walk_spped: i32 (0x5c)
standard_run_spped: i32 (0x60)
walk_speed: i32 (0x64)
run_speed: i32 (0x68)
attack_range: i32 /*décimal ×100*/ (0x6c)
hidesense_range: i32 /*décimal ×100*/ (0x70)
hp: i32 (0x74)
mp: i32 (0x78)
attack_point: i32 (0x7c)
magic_point: i32 (0x80)
defence: i32 (0x84)
magic_defence: i32 (0x88)
attack_speed: i32 (0x8c)
magic_speed: i32 (0x90)
accuracy: i32 (0x94)
avoid: i32 (0x98)
magic_accuracy: i32 (0x9c)
magic_avoid: i32 (0xa0)
taming_id: i32 (0xa4)
creature_taming_code: i32 (0xa8)
taming_percentage: f32 (0xac)
taming_exp_mod: f32 (0xb0)
exp: i32 (0xb4)
jp: i32 (0xb8)
gold_drop_percentage: i32 (0xbc)
gold_min: i32 (0xc0)
gold_max: i32 (0xc4)
chaos_drop_percentage: i32 (0xc8)
chaos_min: i32 (0xcc)
chaos_max: i32 (0xd0)
exp_2: i32 (0xd4)
jp_2: i32 (0xd8)
gold_min_2: i32 (0xdc)
gold_max_2: i32 (0xe0)
chaos_min_2: i32 (0xe4)
chaos_max_2: i32 (0xe8)
drop_table_link_id: i32 (0xec)
texture_group: i32 (0xf0)
local_flag: i32 (0xf4)
flags_0xf8: u8 (0xf8)  // bits LSB→MSB : 0=f_fisrt_attack, 1=f_group_first_attack, 2=f_response_casting, 3=f_response_race, 4=f_response_battle
padding0: i8 (0xf9)
padding1: i16 (0xfa)
model: [u8; 256] (0xfc)
motion_file_id: i32 (0x1fc)
camera_x: i32 (0x200)
camera_y: i32 (0x204)
camera_z: i32 (0x208)
target_x: f32 (0x20c)
target_y: f32 (0x210)
target_z: f32 (0x214)
fight_type: i32 (0x218)
material: i32 (0x21c)
weapon_type: i32 (0x220)
attack_motion_speed: i32 (0x224)
```

[DED] Les noms viennent de `Monster91`. `attack_range`/`hidesense_range` sont des décimaux ×100.
`motion_file_id` renvoie vers `db_monstermotionset.character_id` (352 lignes) et `species_id` vers
`db_monsterspeciesresource` [DED]. `drop_table_link_id` (toujours ≥ 0 ici) → table de drop côté serveur [DED] ; les ids de `db_dropgroup` sont, eux, négatifs (−510000…).

### 5.3 `db_skill.rdb` : 827 octets, packé [OBS]

`text_id` 2742/2742, `tooltip_id` 2742/2742. `desc_id` n'est pas stocké dans le RDB.

Taille calculée : **827 octets**.

```text
id: i32 (0x0)
text_id: i32 (0x4)
is_valid: i8 (0x8)
elemental: i8 (0x9)
is_passive: i8 (0xa)
is_physical_act: i8 (0xb)
is_harmful: i8 (0xc)
is_need_target: i8 (0xd)
is_corpse: i8 (0xe)
is_toggle: i8 (0xf)
casting_type: i8 (0x10)
casting_level: i8 (0x11)
toggle_group: i32 (0x12)
cast_range: i32 (0x16)
valid_range: i32 (0x1a)
cost_hp: i32 (0x1e)
cost_hp_per_skl: i32 (0x22)
cost_mp: i32 (0x26)
cost_mp_per_skl: i32 (0x2a)
cost_mp_per_enhance: i32 (0x2e)
cost_hp_per: f32 (0x32)
cost_hp_per_skl_per: f32 (0x36)
cost_mp_per: f32 (0x3a)
cost_mp_per_skl_per: f32 (0x3e)
cost_havoc: i32 (0x42)
cost_havoc_per_skl: i32 (0x46)
cost_energy: i32 (0x4a)
cost_energy_per_skl: i32 (0x4e)
cost_exp: i32 (0x52)
cost_exp_per_enhance: i32 (0x56)
cost_jp: i32 (0x5a)
cost_jp_per_enhance: i32 (0x5e)
cost_item: i32 (0x62)
cost_item_count: i32 (0x66)
cost_item_count_per_skl: i32 (0x6a)
need_level: i32 (0x6e)
need_hp: i32 (0x72)
need_mp: i32 (0x76)
need_havoc: i32 (0x7a)
need_havoc_burst: i32 (0x7e)
need_state_id: i32 (0x82)
need_state_level: i8 (0x86)
need_state_exhaust: i8 (0x87)
vf_one_hand_sword: i8 (0x88)
vf_two_hand_sword: i8 (0x89)
vf_double_sword: i8 (0x8a)
vf_dagger: i8 (0x8b)
vf_double_dagger: i8 (0x8c)
vf_spear: i8 (0x8d)
vf_axe: i8 (0x8e)
vf_one_hand_axe: i8 (0x8f)
vf_double_axe: i8 (0x90)
vf_one_hand_mace: i8 (0x91)
vf_two_hand_mace: i8 (0x92)
vf_lightbow: i8 (0x93)
vf_heavybow: i8 (0x94)
vf_crossbow: i8 (0x95)
vf_one_hand_staff: i8 (0x96)
vf_two_hand_staff: i8 (0x97)
vf_shield_only: i8 (0x98)
vf_is_not_need_weapon: i8 (0x99)
delay_cast: i32 /*décimal ×100*/ (0x9a)
delay_cast_per_skl: i32 /*décimal ×100*/ (0x9e)
delay_cast_mode_per_enhance: f32 (0xa2)
delay_common: i32 /*décimal ×100*/ (0xa6)
delay_cooltime: i32 /*décimal ×100*/ (0xaa)
delay_cooltime_per_skl: i32 /*décimal ×100*/ (0xae)
delay_cooltime_mode_per_enhance: f32 (0xb2)
cool_time_group_id: i32 (0xb6)
uf_self: i8 (0xba)
uf_party: i8 (0xbb)
uf_guild: i8 (0xbc)
uf_neutral: i8 (0xbd)
uf_purple: i8 (0xbe)
uf_enemy: i8 (0xbf)
tf_avatar: i8 (0xc0)
tf_summon: i8 (0xc1)
tf_monster: i8 (0xc2)
skill_lvup_limit: i32 (0xc3)
target: i16 (0xc7)
effect_type: i16 (0xc9)
skill_enchant_link_id: i32 (0xcb)
state_id: i32 (0xcf)
state_level_base: i32 (0xd3)
state_level_per_skl: f32 (0xd7)
state_level_per_enhance: f32 (0xdb)
state_second: i32 /*décimal ×100*/ (0xdf)
state_second_per_level: i32 /*décimal ×100*/ (0xe3)
state_second_per_enhance: i32 /*décimal ×100*/ (0xe7)
probability_on_hit: i32 (0xeb)
probability_inc_by_slv: i32 (0xef)
hit_bonus: i16 (0xf3)
hit_bonus_per_enhance: i16 (0xf5)
percentage: i16 (0xf7)
hate_mod: f32 (0xf9)
hate_basic: i32 (0xfd)
hate_per_skl: f32 (0x101)
hate_per_enhance: f32 (0x105)
critical_bonus: i32 (0x109)
critical_bonus_per_skl: i32 (0x10d)
var1: f32 (0x111)
var2: f32 (0x115)
var3: f32 (0x119)
var4: f32 (0x11d)
var5: f32 (0x121)
var6: f32 (0x125)
var7: f32 (0x129)
var8: f32 (0x12d)
var9: f32 (0x131)
var10: f32 (0x135)
var11: f32 (0x139)
var12: f32 (0x13d)
var13: f32 (0x141)
var14: f32 (0x145)
var15: f32 (0x149)
var16: f32 (0x14d)
var17: f32 (0x151)
var18: f32 (0x155)
var19: f32 (0x159)
var20: f32 (0x15d)
jp_01: i32 (0x161)
jp_02: i32 (0x165)
jp_03: i32 (0x169)
jp_04: i32 (0x16d)
jp_05: i32 (0x171)
jp_06: i32 (0x175)
jp_07: i32 (0x179)
jp_08: i32 (0x17d)
jp_09: i32 (0x181)
jp_10: i32 (0x185)
jp_11: i32 (0x189)
jp_12: i32 (0x18d)
jp_13: i32 (0x191)
jp_14: i32 (0x195)
jp_15: i32 (0x199)
jp_16: i32 (0x19d)
jp_17: i32 (0x1a1)
jp_18: i32 (0x1a5)
jp_19: i32 (0x1a9)
jp_20: i32 (0x1ad)
jp_21: i32 (0x1b1)
jp_22: i32 (0x1b5)
jp_23: i32 (0x1b9)
jp_24: i32 (0x1bd)
jp_25: i32 (0x1c1)
jp_26: i32 (0x1c5)
jp_27: i32 (0x1c9)
jp_28: i32 (0x1cd)
jp_29: i32 (0x1d1)
jp_30: i32 (0x1d5)
jp_31: i32 (0x1d9)
jp_32: i32 (0x1dd)
jp_33: i32 (0x1e1)
jp_34: i32 (0x1e5)
jp_35: i32 (0x1e9)
jp_36: i32 (0x1ed)
jp_37: i32 (0x1f1)
jp_38: i32 (0x1f5)
jp_39: i32 (0x1f9)
jp_40: i32 (0x1fd)
jp_41: i32 (0x201)
jp_42: i32 (0x205)
jp_43: i32 (0x209)
jp_44: i32 (0x20d)
jp_45: i32 (0x211)
jp_46: i32 (0x215)
jp_47: i32 (0x219)
jp_48: i32 (0x21d)
jp_49: i32 (0x221)
jp_50: i32 (0x225)
unknownValue0: i8 (0x229)
tooltip_id: i32 (0x22a)
icon_id: i32 (0x22e)
icon_file_name: [u8; 256] (0x232)
is_projectile: i8 (0x332)
projectile_speed: f32 (0x333)
projectile_acceleration: f32 (0x337)
```

Notes : [INC] `critical_bonus` (0x109) vaut **26995** (= octets `"si"`) dans les 2742 lignes : probablement
un résidu de l'outil plutôt qu'une vraie valeur. Les `DECIMAL` (`delay_cast`, `delay_cooltime`, `state_second`…)
sont des valeurs ×100 : `delay_cooltime` vaut surtout 6000/1500/500, soit 60 s/15 s/5 s (le client y écrit lui-même `(int)(x*100)`) [DED]. Le client écrase ensuite certains champs avec une table
interne (`DAT_00e91134..0x00e91774`) : `var1..20`, `delay_cooltime`, `state_second_per_level` [OBS].

### 5.4 `db_job.rdb` : 292 octets [OBS]

`text_id` 42/42 (ex. 100 → "Vagabond", 101 → "Mercenaire", 102 → "Mystique").
`job_class` : 1..4 (combattant/chasseur/mage/invocateur) [DED] ; `job_depth` 0..3 ; `avable_job_*` = métiers
suivants (−1 = aucun) ; `skill_tree_id` → `db_skilltree.skill_tree_id`.

Taille calculée : **292 octets**.

```text
id: i32 (0x0)
text_id: i32 (0x4)
stati_id: i32 (0x8)
skill_tree_id: i32 (0xc)
job_class: i8 (0x10)
job_depth: i8 (0x11)
up_lv: i16 (0x12)
up_jlv: i16 (0x14)
avable_job_0: i16 (0x16)
avable_job_1: i16 (0x18)
avable_job_2: i16 (0x1a)
avable_job_3: i16 (0x1c)
unknown_value: i16 (0x1e)
icon_id: i32 (0x20)
icon_file_name: [u8; 256] (0x24)
```

### 5.5 `db_npcresource.rdb` : 997 octets [OBS]

`text_id` 1495/1495, `name_text_id` 1497/1497. Ex. `id 1001, name "Andariel", text "Breeder", model "npc_def_common004", x 8330, y 7167`.

Taille calculée : **997 octets**.

```text
id: i32 (0x0)
text_id: i32 (0x4)
name_text_id: i32 (0x8)
race_id: i32 (0xc)
sexsual_id: i32 (0x10)
x: i32 (0x14)
y: i32 (0x18)
z: i32 (0x1c)
face: i32 (0x20)
local_flag: i32 (0x24)
face_x: i32 (0x28)
face_y: i32 (0x2c)
face_z: i32 (0x30)
model_file: [u8; 256] (0x34)
hair_id: i32 (0x134)
face_id: i32 (0x138)
body_id: i32 (0x13c)
weapon_item_id: i32 (0x140)
shield_item_id: i32 (0x144)
clothes_item_id: i32 (0x148)
helm_item_id: i32 (0x14c)
gloves_item_id: i32 (0x150)
boots_item_id: i32 (0x154)
belt_item_id: i32 (0x158)
mantle_item_id: i32 (0x15c)
necklace_item_id: i32 (0x160)
earring_item_id: i32 (0x164)
ring1_item_id: i32 (0x168)
ring2_item_id: i32 (0x16c)
motion_id: i32 (0x170)
is_roam: i32 (0x174)
roaming_id: i32 (0x178)
standard_walk_speed: i32 (0x17c)
standard_run_speed: i32 (0x180)
walk_speed: i32 (0x184)
run_speed: i32 (0x188)
attackable: i32 (0x18c)
offensive_type: i32 (0x190)
spawn_type: i32 (0x194)
chase_range: i32 (0x198)
regen_time: i32 (0x19c)
level: i32 (0x1a0)
stat_id: i32 (0x1a4)
attack_range: i32 (0x1a8)
attack_speed_type: i32 (0x1ac)
hp: i32 (0x1b0)
mp: i32 (0x1b4)
attack_point: i32 (0x1b8)
magic_point: i32 (0x1bc)
defence: i32 (0x1c0)
magic_defence: i32 (0x1c4)
attack_speed: i32 (0x1c8)
magic_speed: i32 (0x1cc)
accuracy: i32 (0x1d0)
avoid: i32 (0x1d4)
magic_accuracy: i32 (0x1d8)
magic_avoid: i32 (0x1dc)
ai_script: [u8; 256] (0x1e0)
contact_script: [u8; 256] (0x2e0)
texture_group: i32 (0x3e0)
type: i8 (0x3e4)
```

[DED] `x/y/z` en unités monde entières ; `local_flag` = masque de régions de service (le client
l'ignore si `local_flag & masque_région == 0`) [OBS] ; `*_item_id` → `db_item.id`.

### 5.6 `db_quest.rdb` : 268 octets [OBS]

`text_id_quest` 997/997, `text_id_summary` 997/997. Ex. `1005 "Entraînement Deva" : "Chassez 3 Fulmars…", type 101, value1 1003 (monstre décodé)`.

Taille calculée : **268 octets**.

```text
id: i32 (0x0)
text_id_quest: i32 (0x4)
text_id_summary: i32 (0x8)
text_id_status: i32 (0xc)
limit_begin_time: i32 (0x10)
limit_end_time: i32 (0x14)
limit_level: i32 (0x18)
limit_job_level: i32 (0x1c)
limit_max_level: i32 (0x20)
limit_max_job_level: i32 (0x24)
flags_0x28: u8 (0x28)  // bits LSB→MSB : 0=nv, 1=limit_deva, 2=limit_asura, 3=limit_gaia, 4=limit_fighter, 5=limit_hunter, 6=limit_magician, 7=limit_summoner
padding0: i8 (0x29)
padding1: i16 (0x2a)
limit_job: i32 (0x2c)
limit_favor: i32 (0x30)
repeatable: i8 (0x34)
unknown1: i8 (0x35)
unknown2: i16 (0x36)
invoke_condition: i32 (0x38)
invoke_value: i32 (0x3c)
time_limit_type: i16 (0x40)
job_depth: i16 (0x42)
time_limit: i32 (0x44)
type: i32 (0x48)
value1: i32 (0x4c)
value2: i32 (0x50)
value3: i32 (0x54)
value4: i32 (0x58)
value5: i32 (0x5c)
value6: i32 (0x60)
value7: i32 (0x64)
value8: i32 (0x68)
value9: i32 (0x6c)
value10: i32 (0x70)
value11: i32 (0x74)
value12: i32 (0x78)
drop_group_id: i32 (0x7c)
quest_difficulty: i32 (0x80)
favor: i32 (0x84)
exp: i32 (0x88)
jp: i32 (0x8c)
holicpoint: i32 (0x90)
gold: i32 (0x94)
default_reward_id: i32 (0x98)
default_reward_level: i32 (0x9c)
default_reward_quantity: i32 (0xa0)
optional_reward_id1: i32 (0xa4)
optional_reward_level1: i32 (0xa8)
optional_reward_quantity1: i32 (0xac)
optional_reward_id2: i32 (0xb0)
optional_reward_level2: i32 (0xb4)
optional_reward_quantity2: i32 (0xb8)
optional_reward_id3: i32 (0xbc)
optional_reward_level3: i32 (0xc0)
optional_reward_quantity3: i32 (0xc4)
optional_reward_id4: i32 (0xc8)
optional_reward_level4: i32 (0xcc)
optional_reward_quantity4: i32 (0xd0)
optional_reward_id5: i32 (0xd4)
optional_reward_level5: i32 (0xd8)
optional_reward_quantity5: i32 (0xdc)
optional_reward_id6: i32 (0xe0)
optional_reward_level6: i32 (0xe4)
optional_reward_quantity6: i32 (0xe8)
forequest1: i32 (0xec)
forequest2: i32 (0xf0)
forequest3: i32 (0xf4)
or_flag: i8 (0xf8)
unknown4: i8 (0xf9)
show_target_type: i8 (0xfa)
unknownPadding: i8 (0xfb)
show_target_id: i32 (0xfc)
mark_hide: i8 (0x100)
unknown5: i8 (0x101)
unknown6: i16 (0x102)
cool_time: i32 (0x104)
accept_cool_time: i32 (0x108)
```

[DED] Les champs `RDBIGNORE` de RappelzRDBTool (`limit_favor_group_id`, `favor_group_id`, `hate_group_id`,
`script_*_text`) n'existent pas dans le fichier. `value1..12` dépendent de `type` (101 = chasse : value1 = id
monstre décodé, value2 = nombre ; ex. quête 1005 : 1003, 3, 2003) [DED].

### 5.7 `db_worldlocation.rdb` : 1143 octets, packé [OBS]

`text_id` 6689/6689 (ex. 10 → "Laksy"). Plusieurs lignes peuvent avoir le même `id` (le client les regroupe
dans une liste par id : `FUN_006391e0`) [OBS].

Taille calculée : **1143 octets**.

```text
id: i32 (0x0)
text_id: i32 (0x4)
x: i32 (0x8)
y: i32 (0xc)
preloading_distance: i32 (0x10)
apply_location_name: i32 (0x14)
apply_light: i32 (0x18)
apply_bgm: i32 (0x1c)
location_type: i32 (0x20)
fog_application: i32 (0x24)
time_id: i32 (0x28)
weather_id: i32 (0x2c)
cloud_ratio: i32 (0x30)
weather_change_time: i32 (0x34)
weather_ratio: i32 (0x38)
sky_start_a: i32 (0x3c)
sky_start_r: i32 (0x40)
sky_start_g: i32 (0x44)
sky_start_b: i32 (0x48)
sky_mid_a: i32 (0x4c)
sky_mid_r: i32 (0x50)
sky_mid_g: i32 (0x54)
sky_mid_b: i32 (0x58)
sky_end_a: i32 (0x5c)
sky_end_r: i32 (0x60)
sky_end_g: i32 (0x64)
sky_end_b: i32 (0x68)
sky_mid_rate: f32 (0x6c)
diffuse_a: i32 (0x70)
diffuse_r: i32 (0x74)
diffuse_g: i32 (0x78)
diffuse_b: i32 (0x7c)
ambient_a: i32 (0x80)
ambient_r: i32 (0x84)
ambient_g: i32 (0x88)
ambient_b: i32 (0x8c)
specular_a: i32 (0x90)
specular_r: i32 (0x94)
specular_g: i32 (0x98)
specular_b: i32 (0x9c)
fog_a: i32 (0xa0)
fog_r: i32 (0xa4)
fog_g: i32 (0xa8)
fog_b: i32 (0xac)
v_start: f32 (0xb0)
v_end: f32 (0xb4)
h_start: f32 (0xb8)
h_end: f32 (0xbc)
fixed_terrain_distance: i16 (0xc0)
cloud_a: i32 (0xc2)
cloud_r: i32 (0xc6)
cloud_g: i32 (0xca)
cloud_b: i32 (0xce)
land_specular: f32 (0xd2)
character_diffuse: f32 (0xd6)
character_ambient: f32 (0xda)
sun_texture_file: [u8; 256] (0xde)
moon_texture_file: [u8; 256] (0x1de)
cloud_texture_file: [u8; 256] (0x2de)
bgm1_ID: i32 (0x3de)
bgm2_ID: i32 (0x3e2)
bgm3_ID: i32 (0x3e6)
bgm1_ratio: i32 (0x3ea)
bgm2_ratio: i32 (0x3ee)
bgm3_ratio: i32 (0x3f2)
environmental_sound1_ID: i32 (0x3f6)
environmental_sound2_ID: i32 (0x3fa)
environmental_sound3_ID: i32 (0x3fe)
environmental_sound4_ID: i32 (0x402)
environmental_sound1_ratio: i32 (0x406)
environmental_sound2_ratio: i32 (0x40a)
environmental_sound3_ratio: i32 (0x40e)
environmental_sound4_ratio: i32 (0x412)
environmental_sound_reverb: i32 (0x416)
environmental_sound_volume: i32 (0x41a)
environmental_sound_equalizer: i32 (0x41e)
minimap_ani_file_ID: i16 (0x422)
worldmap_ani_file_ID: i16 (0x424)
unknownvalue0: i8 (0x426)
zoom_map_type: i32 (0x427)
display_positoin_x: i32 (0x42b)
display_positoin_y: i32 (0x42f)
zoom_x: i32 (0x433)
offset_x: i32 (0x437)
offset_y: i32 (0x43b)
size_x: i32 (0x43f)
size_y: i32 (0x443)
blank_resource_ID: i32 (0x447)
blank_a: i32 (0x44b)
blank_r: i32 (0x44f)
blank_g: i32 (0x453)
blank_b: i32 (0x457)
zoom_minimap: i32 (0x45b)
precipitation_min: i32 (0x45f)
precipitation_max: i32 (0x463)
aurora_t: f32 (0x467)
aurora_a: f32 (0x46b)
aurora_s: f32 (0x46f)
aurora_h: f32 (0x473)
```

[DED] `x/y` = coordonnées de la case de zone. [INC] Le constructeur du client initialise des `u32` à 0x41F,
0x423… (décalage d'un octet après 0x41E). Ça colle avec `minimap_ani_file_ID`/`worldmap_ani_file_ID`/`unknownvalue0`
(2+2+1 octets) suivis de `zoom_map_type` à 0x427, mais l'interprétation exacte de 0x422..0x426 n'est pas prouvée.

### 5.8 `db_skilltree.rdb` : lignes de 64 octets (format groupé §4.3) [OBS]

Taille calculée : **64 octets**.

```text
skill_tree_id: i32 (0x0)
skill_id: i32 (0x4)
skill_group_id: i32 (0x8)
min_skill_lv: i32 (0xc)
max_skill_lv: i32 (0x10)
lv: i32 (0x14)
job_lv: i32 (0x18)
jp_ratio: f32 (0x1c)
need_skill_id_1: i32 (0x20)
need_skill_id_2: i32 (0x24)
need_skill_id_3: i32 (0x28)
need_skill_lv_1: i32 (0x2c)
need_skill_lv_2: i32 (0x30)
need_skill_lv_3: i32 (0x34)
cenhance_min: i32 (0x38)
cenhance_max: i32 (0x3c)
```

### 5.9 Autres tables [DON]

`db_dropgroup.rdb` (84) : les `drop_min/max_count_*` de l'outil sont `RDBIGNORE`, ils ne sont pas dans le fichier.

Taille calculée : **84 octets**.

```text
id: i32 (0x0)
drop_item_id_00: i32 (0x4)
drop_item_id_01: i32 (0x8)
drop_item_id_02: i32 (0xc)
drop_item_id_03: i32 (0x10)
drop_item_id_04: i32 (0x14)
drop_item_id_05: i32 (0x18)
drop_item_id_06: i32 (0x1c)
drop_item_id_07: i32 (0x20)
drop_item_id_08: i32 (0x24)
drop_item_id_09: i32 (0x28)
drop_percentage_00: f32 (0x2c)
drop_percentage_01: f32 (0x30)
drop_percentage_02: f32 (0x34)
drop_percentage_03: f32 (0x38)
drop_percentage_04: f32 (0x3c)
drop_percentage_05: f32 (0x40)
drop_percentage_06: f32 (0x44)
drop_percentage_07: f32 (0x48)
drop_percentage_08: f32 (0x4c)
drop_percentage_09: f32 (0x50)
```

`db_tenacity.rdb` (552) = **table des états (buffs/debuffs)**, définition `State81` : `text_id`/`tooltip_id` 2205/2206 (ex. 1001 "Gain Max HP").

Taille calculée : **552 octets**.

```text
state_id: i32 (0x0)
text_id: i32 (0x4)
tooltip_id: i32 (0x8)
is_harmful: i8 (0xc)
padding0: i8 (0xd)
padding1: i16 (0xe)
state_time_type: i32 (0x10)
state_group: i32 (0x14)
duplicate_group_1: i32 (0x18)
duplicate_group_2: i32 (0x1c)
duplicate_group_3: i32 (0x20)
uf_avatar: i8 (0x24)
uf_summon: i8 (0x25)
uf_monster: i8 (0x26)
unknown0: i8 (0x27)
reiteration_count: i8 (0x28)
unknown1: i8 (0x29)
unknown2: i16 (0x2a)
base_effect_id: i32 (0x2c)
fire_interval: i32 (0x30)
elemental_type: i32 (0x34)
amplify_base: f32 (0x38)
amplify_per_skl: f32 (0x3c)
add_damage_base: i32 (0x40)
add_damage_per_skl: i32 (0x44)
effect_type: i32 (0x48)
unknown3: i32 (0x4c)
value_0: f64 (0x50)
value_1: f64 (0x58)
value_2: f64 (0x60)
value_3: f64 (0x68)
value_4: f64 (0x70)
value_5: f64 (0x78)
value_6: f64 (0x80)
value_7: f64 (0x88)
value_8: f64 (0x90)
value_9: f64 (0x98)
value_10: f64 (0xa0)
value_11: f64 (0xa8)
value_12: f64 (0xb0)
value_13: f64 (0xb8)
value_14: f64 (0xc0)
value_15: f64 (0xc8)
value_16: f64 (0xd0)
value_17: f64 (0xd8)
value_18: f64 (0xe0)
value_19: f64 (0xe8)
icon_id: i32 (0xf0)
icon_file_name: [u8; 256] (0xf4)
fx_id: i32 (0x1f4)
pos_id: i32 (0x1f8)
cast_fx_id: i32 (0x1fc)
cast_fx_pos_id: i32 (0x200)
cast_skill_id: i32 (0x204)
hit_fx_id: i32 (0x208)
hit_fx_pos_id: i32 (0x20c)
special_output_timing_id: i32 (0x210)
special_output_fx_id: i32 (0x214)
special_output_fx_pos_id: i32 (0x218)
special_output_fx_delay: i32 (0x21c)
state_fx_id: i32 (0x220)
state_fx_pos_id: i32 (0x224)
```

`db_skillstage.rdb` (160). [INC] `unknown0` = 163 et `is_creature_skill` = −31232 sur la plupart des lignes (bourrage ?).

Taille calculée : **160 octets**.

```text
id: i32 (0x0)
elemental: i8 (0x4)
type: i8 (0x5)
deal_damage: i8 (0x6)
has_target: i8 (0x7)
corpse_target: i8 (0x8)
is_toggle: i8 (0x9)
unknown0: i16 (0xa)
casting_time: f32 (0xc)
is_creature_skill: i16 (0x10)
unknown1: i16 (0x12)
stage_type_id: i32 (0x14)
casting_type_id: i32 (0x18)
casting_start_motion_id: i32 (0x1c)
casting_middle_motion_id: i32 (0x20)
casting_fx_set_id: i32 (0x24)
fire_motion_id: i32 (0x28)
fire_fx_set_id: i32 (0x2c)
stage_data_01: f32 (0x30)
stage_data_02: f32 (0x34)
stage_data_03: f32 (0x38)
stage_data_04: f32 (0x3c)
stage_data_05: f32 (0x40)
stage_data_06: f32 (0x44)
stage_data_07: f32 (0x48)
stage_data_08: f32 (0x4c)
stage_data_09: f32 (0x50)
stage_data_10: f32 (0x54)
stage_data_11: f32 (0x58)
stage_data_12: f32 (0x5c)
stage_data_13: f32 (0x60)
stage_data_14: f32 (0x64)
stage_data_15: f32 (0x68)
stage_data_16: f32 (0x6c)
stage_data_17: f32 (0x70)
stage_data_18: f32 (0x74)
stage_data_19: f32 (0x78)
stage_data_20: f32 (0x7c)
sub_stage_on_hit_id: i32 (0x80)
sub_stage_on_hit_continuous: i32 (0x84)
hit_fx_output_delay: i32 (0x88)
hit_fx_set_id: i32 (0x8c)
hit_fx_set_position_id: i32 (0x90)
fire_fx_output_delay: i32 (0x94)
sub_fire_fx_set_id: i32 (0x98)
sub_fire_fx_position: i32 (0x9c)
```

`db_skilltreedisplay.rdb` (28)

Taille calculée : **28 octets**.

```text
job_id: i32 (0x0)
slot_v: i32 (0x4)
skill1_id: i32 (0x8)
skill2_id: i32 (0xc)
skill3_id: i32 (0x10)
skill4_id: i32 (0x14)
skill5_id: i32 (0x18)
```

`db_questlink.rdb` (11)

Taille calculée : **11 octets**.

```text
npc_id: i32 (0x0)
quest_id: i32 (0x4)
flag_start: i8 (0x8)
flag_progress: i8 (0x9)
flag_end: i8 (0xa)
```

`db_combineres.rdb` (436) : 9 blocs `subNN` à `0x4C + (NN−1)×0x28` (5 types puis 5 valeurs) ; types 3 = objet (valeur = id objet) [DED].

Taille calculée : **436 octets**.

```text
id: i32 (0x0)
mix_type: i32 (0x4)
mix_value_01: i32 (0x8)
mix_value_02: i32 (0xc)
mix_value_03: i32 (0x10)
mix_value_04: i32 (0x14)
mix_value_05: i32 (0x18)
mix_value_06: i32 (0x1c)
sub_material_count: i32 (0x20)
main_type_01: i32 (0x24)
main_value_01: i32 (0x28)
main_type_02: i32 (0x2c)
main_value_02: i32 (0x30)
main_type_03: i32 (0x34)
main_value_03: i32 (0x38)
main_type_04: i32 (0x3c)
main_value_04: i32 (0x40)
main_type_05: i32 (0x44)
main_value_05: i32 (0x48)
sub01_type_01: i32 (0x4c)
sub01_type_02: i32 (0x50)
sub01_type_03: i32 (0x54)
sub01_type_04: i32 (0x58)
sub01_type_05: i32 (0x5c)
sub01_value_01: i32 (0x60)
sub01_value_02: i32 (0x64)
sub01_value_03: i32 (0x68)
sub01_value_04: i32 (0x6c)
sub01_value_05: i32 (0x70)
sub02_type_01: i32 (0x74)
sub02_type_02: i32 (0x78)
sub02_type_03: i32 (0x7c)
sub02_type_04: i32 (0x80)
sub02_type_05: i32 (0x84)
sub02_value_01: i32 (0x88)
sub02_value_02: i32 (0x8c)
sub02_value_03: i32 (0x90)
sub02_value_04: i32 (0x94)
sub02_value_05: i32 (0x98)
sub03_type_01: i32 (0x9c)
sub03_type_02: i32 (0xa0)
sub03_type_03: i32 (0xa4)
sub03_type_04: i32 (0xa8)
sub03_type_05: i32 (0xac)
sub03_value_01: i32 (0xb0)
sub03_value_02: i32 (0xb4)
sub03_value_03: i32 (0xb8)
sub03_value_04: i32 (0xbc)
sub03_value_05: i32 (0xc0)
sub04_type_01: i32 (0xc4)
sub04_type_02: i32 (0xc8)
sub04_type_03: i32 (0xcc)
sub04_type_04: i32 (0xd0)
sub04_type_05: i32 (0xd4)
sub04_value_01: i32 (0xd8)
sub04_value_02: i32 (0xdc)
sub04_value_03: i32 (0xe0)
sub04_value_04: i32 (0xe4)
sub04_value_05: i32 (0xe8)
sub05_type_01: i32 (0xec)
sub05_type_02: i32 (0xf0)
sub05_type_03: i32 (0xf4)
sub05_type_04: i32 (0xf8)
sub05_type_05: i32 (0xfc)
sub05_value_01: i32 (0x100)
sub05_value_02: i32 (0x104)
sub05_value_03: i32 (0x108)
sub05_value_04: i32 (0x10c)
sub05_value_05: i32 (0x110)
sub06_type_01: i32 (0x114)
sub06_type_02: i32 (0x118)
sub06_type_03: i32 (0x11c)
sub06_type_04: i32 (0x120)
sub06_type_05: i32 (0x124)
sub06_value_01: i32 (0x128)
sub06_value_02: i32 (0x12c)
sub06_value_03: i32 (0x130)
sub06_value_04: i32 (0x134)
sub06_value_05: i32 (0x138)
sub07_type_01: i32 (0x13c)
sub07_type_02: i32 (0x140)
sub07_type_03: i32 (0x144)
sub07_type_04: i32 (0x148)
sub07_type_05: i32 (0x14c)
sub07_value_01: i32 (0x150)
sub07_value_02: i32 (0x154)
sub07_value_03: i32 (0x158)
sub07_value_04: i32 (0x15c)
sub07_value_05: i32 (0x160)
sub08_type_01: i32 (0x164)
sub08_type_02: i32 (0x168)
sub08_type_03: i32 (0x16c)
sub08_type_04: i32 (0x170)
sub08_type_05: i32 (0x174)
sub08_value_01: i32 (0x178)
sub08_value_02: i32 (0x17c)
sub08_value_03: i32 (0x180)
sub08_value_04: i32 (0x184)
sub08_value_05: i32 (0x188)
sub09_type_01: i32 (0x18c)
sub09_type_02: i32 (0x190)
sub09_type_03: i32 (0x194)
sub09_type_04: i32 (0x198)
sub09_type_05: i32 (0x19c)
sub09_value_01: i32 (0x1a0)
sub09_value_02: i32 (0x1a4)
sub09_value_03: i32 (0x1a8)
sub09_value_04: i32 (0x1ac)
sub09_value_05: i32 (0x1b0)
```

`db_itemeffectresource.rdb` (181)

Taille calculée : **181 octets**.

```text
id: i32 (0x0)
ordinal_id: i32 (0x4)
tooltip_id: i32 (0x8)
effect_type: i8 (0xc)
effect_id: i16 (0xd)
effect_level: i16 (0xf)
value_0: f64 (0x11)
value_1: f64 (0x19)
value_2: f64 (0x21)
value_3: f64 (0x29)
value_4: f64 (0x31)
value_5: f64 (0x39)
value_6: f64 (0x41)
value_7: f64 (0x49)
value_8: f64 (0x51)
value_9: f64 (0x59)
value_10: f64 (0x61)
value_11: f64 (0x69)
value_12: f64 (0x71)
value_13: f64 (0x79)
value_14: f64 (0x81)
value_15: f64 (0x89)
value_16: f64 (0x91)
value_17: f64 (0x99)
value_18: f64 (0xa1)
value_19: f64 (0xa9)
unknown: i32 (0xb1)
```

`db_enhanceeffect.rdb` (208)

Taille calculée : **208 octets**.

```text
sid: i32 (0x0)
sub_id: i16 (0x4)
effect_id: i16 (0x6)
value_01: f64 (0x8)
value_02: f64 (0x10)
value_03: f64 (0x18)
value_04: f64 (0x20)
value_05: f64 (0x28)
value_06: f64 (0x30)
value_07: f64 (0x38)
value_08: f64 (0x40)
value_09: f64 (0x48)
value_10: f64 (0x50)
value_11: f64 (0x58)
value_12: f64 (0x60)
value_13: f64 (0x68)
value_14: f64 (0x70)
value_15: f64 (0x78)
value_16: f64 (0x80)
value_17: f64 (0x88)
value_18: f64 (0x90)
value_19: f64 (0x98)
value_20: f64 (0xa0)
value_21: f64 (0xa8)
value_22: f64 (0xb0)
value_23: f64 (0xb8)
value_24: f64 (0xc0)
value_25: f64 (0xc8)
```

`db_fieldpropresource.rdb` (657)

Taille calculée : **657 octets**.

```text
id: i32 (0x0)
text_id: i32 (0x4)
tooltip_id: i32 (0x8)
type: i32 (0xc)
local_flag: i32 (0x10)
casting_time: i32 (0x14)
use_count: i32 (0x18)
regen_time: i32 (0x1c)
life_time: i32 (0x20)
casting_range: f32 (0x24)
target_fx_size: f32 (0x28)
limit_min_level: i32 (0x2c)
limit_max_level: i32 (0x30)
limit_deva: i8 (0x34)
limit_asura: i8 (0x35)
limit_gaia: i8 (0x36)
limit_fighter: i8 (0x37)
limit_hunter: i8 (0x38)
limit_magician: i8 (0x39)
limit_summoner: i8 (0x3a)
limit_job: i32 (0x3b)
activation_condition: i32 (0x3f)
activation_value1: i32 (0x43)
activation_value2: i32 (0x47)
activation2_condition: i32 (0x4b)
activation2_value1: i32 (0x4f)
activation2_value2: i32 (0x53)
activate_id: i32 (0x57)
drop_item_id_00: i32 (0x5b)
drop_item_ratio_00: i32 (0x5f)
drop_item_count_min_00: i32 (0x63)
drop_item_count_max_00: i32 (0x67)
drop_item_level_min_00: i32 (0x6b)
drop_item_level_max_00: i32 (0x6f)
drop_item_id_01: i32 (0x73)
drop_item_ratio_01: i32 (0x77)
drop_item_count_min_01: i32 (0x7b)
drop_item_count_max_01: i32 (0x7f)
drop_item_level_min_01: i32 (0x83)
drop_item_level_max_01: i32 (0x87)
script_text: [u8; 256] (0x8b)
file_name: [u8; 256] (0x18b)
terrain_check: i32 (0x28b)
worldmap_check_type: i16 (0x28f)
```

`db_monstermotionset.rdb` (100)

Taille calculée : **100 octets**.

```text
character_id: i32 (0x0)
def: i32 (0x4)
battle: i32 (0x8)
idle: i32 (0xc)
walk: i32 (0x10)
run: i32 (0x14)
be_attack: i32 (0x18)
be_attack2: i32 (0x1c)
dead: i32 (0x20)
flying_dead: i32 (0x24)
take: i32 (0x28)
attack_1: i32 (0x2c)
attack_2: i32 (0x30)
attack_3: i32 (0x34)
draw_bow: i32 (0x38)
target_bow: i32 (0x3c)
fire_bow: i32 (0x40)
casting_1: i32 (0x44)
casting_2: i32 (0x48)
casting_3: i32 (0x4c)
melee_1: i32 (0x50)
melee_2: i32 (0x54)
melee_3: i32 (0x58)
mount: i32 (0x5c)
unmount: i32 (0x60)
```

`db_monsterspeciesresource.rdb` (12) : `string_id` 162/162 (ex. 8160 "Ent").

Taille calculée : **12 octets**.

```text
sid: i32 (0x0)
species_id: i32 (0x4)
string_id: i32 (0x8)
```

`db_motionfxset.rdb` (68)

Taille calculée : **68 octets**.

```text
id: i32 (0x0)
fx_type: i32 (0x4)
add_time: i32 (0x8)
following: i32 (0xc)
graphic_effect_start_file_ID: i32 (0x10)
graphic_effect_middle_file_ID: i32 (0x14)
graphic_effect_end_file_ID: i32 (0x18)
graphic_effect_direction: i32 (0x1c)
is_effect0: i8 (0x20)
is_effect1: i8 (0x21)
is_effect2: i8 (0x22)
unknown0: i8 (0x23)
sound_start_file_ID: i32 (0x24)
sound_middle_file_ID: i32 (0x28)
sound_end_file_ID: i32 (0x2c)
sound_play_probability: i32 (0x30)
sound_volumn: i32 (0x34)
is_sound0: i8 (0x38)
is_sound1: i8 (0x39)
is_sound2: i8 (0x3a)
sound_reverb_apply: i8 (0x3b)
sound_filter_apply: i8 (0x3c)
sound_stereo_apply: i8 (0x3d)
sound_option0: i8 (0x3e)
sound_option1: i8 (0x3f)
sound_option2: i8 (0x40)
kagValue: [u8; 3] (0x41)
```

`db_soundresource.rdb` (260)

Taille calculée : **260 octets**.

```text
sound_ID: i32 (0x0)
sound_Name: [u8; 256] (0x4)
```

`db_titleresource.rdb` (114)

Taille calculée : **114 octets**.

```text
id: i32 (0x0)
name_id: i32 (0x4)
tooltip_id: i32 (0x8)
status_id: i32 (0xc)
rate: i8 (0x10)
opt_type_0: i16 (0x11)
opt_type_1: i16 (0x13)
opt_type_2: i16 (0x15)
opt_type_3: i16 (0x17)
opt_type_4: i16 (0x19)
opt_type_5: i16 (0x1b)
opt_type_6: i16 (0x1d)
opt_type_7: i16 (0x1f)
opt_var1_0: f32 (0x21)
opt_var1_1: f32 (0x25)
opt_var1_2: f32 (0x29)
opt_var1_3: f32 (0x2d)
opt_var1_4: f32 (0x31)
opt_var1_5: f32 (0x35)
opt_var1_6: f32 (0x39)
opt_var1_7: f32 (0x3d)
opt_var2_0: f32 (0x41)
opt_var2_1: f32 (0x45)
opt_var2_2: f32 (0x49)
opt_var2_3: f32 (0x4d)
opt_var2_4: f32 (0x51)
opt_var2_5: f32 (0x55)
opt_var2_6: f32 (0x59)
opt_var2_7: f32 (0x5d)
effect_id: i32 (0x61)
local_flag: i32 (0x65)
is_periodic: i8 (0x69)
begin_of_period: i32 (0x6a)
end_of_period: i32 (0x6e)
```

`db_creatureenhance.rdb` (12)

Taille calculée : **12 octets**.

```text
enhance_level: i16 (0x0)
stat_amplify: f32 (0x2)
card_durability: i16 (0x6)
slot_amount: i16 (0x8)
jp_addition: i16 (0xa)
```

---

## 6. Inconnu / à vérifier

- [INC] Sémantique fine de nombreux champs (énumérations `type`/`group`/`class` des objets, `effect_type`
  des compétences, `value_*` des états). Les noms sont fiables pour la position, pas forcément pour le sens.
- [INC] Ordre de priorité entre `Resource\` (en clair) et la même base dans `data.00x` (XOR) : non vérifié.
- [INC] Pourquoi `name_len` vaut 1 (nom vide) dans `db_string` : le champ `name` (64 octets max côté SQL)
  semble vidé par l'outil Daedalus. Le client indexe uniquement par `code`.
- [INC] Les tables présentes dans le client mais absentes de `Resource\` (db_state, db_effect…) n'ont pas été traitées.

## 7. Sources

- `SFrame.exe` (chargeurs listés en §1.1).
- Définitions RappelzRDBTool : https://github.com/glandu2/RappelzRDBTool (`Databases/lua/*.lua`, `Base/include/DataType.h`).

## Tables de progression retrouvées dans les archives, 31/10/2026

La [validation de progression](../JOB_PROGRESSION_VALIDATION.md) retrouve `db_basicstat.rdb` (3923×16octets, huit i16), `db_exp.rdb` (300×40octets, cinq i64, schémaExp91) et `db_summonexp.rdb` (200×24octets, trois i64). Les niveaux EXP sont implicites, index+1 ; les anciens schémas Exp24octets ne correspondent pas. Les42 `Job.stati_id` trouvent leur base. Le lecteur dédié `progression::ProgressionBook` et les compagnons `Progression/*.dbc` stockent les i64 en deux cellules32bits low/high, sans float ni conversion générique tronquée. Tous les champs sont comparés aux ressources brutes, dans les installations serveur/client. EXP300=`272233208401492000` est conservée exactement. Les propriétés cumulatives et bonus effectifs ne sont pas déduits de la seule forme des tables : les références primaires et limites sont explicites dans le rapport. Les anciennes observations des29 fichiers loose restent historiques.

# Cartes Rappelz (moteur nFlavor) : `.nfm` et fichiers associés

Client analysé : `Rappelz NoLimits\SFrame.exe` (même binaire que `data_archive.md`), Ghidra 12.1.4 via REA.
Données : les 136 `.nfm` et les ~800 fichiers `mXXX_YYY.*` de l'archive `data.000-008`
(extraction : `node docs/re/tools/rappelz_data.js "Rappelz NoLimits" extract '^m\d+_\d+\.' <dossier>`).
Implémentation : crate `shared/rz-terrain` (`cargo test -p rz-terrain` relit les vrais fichiers).

Légende : **[O]** observé dans le binaire et/ou vérifié octet par octet sur les fichiers,
**[D]** déduit (cohérent avec les données, non lu dans le code), **[I]** inconnu.
Tous les entiers et réels sont little-endian ; `f32` = IEEE 754.

---

## 1. Fonctions du client

| Adresse | Rôle |
|---|---|
| `FUN_00b1f920` | Lit `TerrainSeamlessWorld.cfg` (`TILE_LENGTH=`, `TILECOUNT_PER_SEGMENT=`, `SEGMENTCOUNT_PER_MAP=`, `SETFOV=`, `MAPLAYER=`, `MAPSIZE=`, `MAPFILE=`). En mode vérification, lit 0x3C octets de chaque `.nfm` et exige `"nFlavor Map"`, version `0x16`, et les mêmes paramètres de grille. |
| `FUN_00997bd0`, `FUN_009984b0` | Pour x = 0..12, y = 0..9 : `sprintf("m%03d_%03d.nfm", x, y)` ; si la carte existe, ouvre `m%03d_%03d.nfs`, contrôle `"nFlavor Script"` et la version (1 → `FUN_009961c0`, 2 → `FUN_00996570`). |
| `FUN_00996570` / `FUN_009961c0` | `.nfs` v2 / v1 : rectangles + noms, puis événements (`FUN_00995fd0`). |
| `FUN_0099cc60` | `.qpf` : en-tête 0x1A octets, enregistrements de 0x31 octets ; position + `x·16128` (marqueurs de quête de la carte du monde). |
| `FUN_00b00b00` | Crée la tâche `CTerrainSegmentLoadThread` (vtable `0x00d877dc`) d'un segment : origine monde du segment, nom des 9 cartes (segment + 8 voisins). |
| `FUN_00b09e70` | Exécution de la tâche : `FUN_00af9220` puis `CTerrainMapEngine::CreateSegment` (`FUN_00b01030`). |
| `FUN_00af9220` | Constructeur `CTerrainInfoForSegment` (vtable `0x00d87728`) : tampon de (6+3)² sommets × **23** octets, valeurs par défaut (brouillard 1024/4096, ciel `0xF8FFCCDE`), appelle `FUN_00afa350`, calcule les normales (`FUN_00afadb0`). |
| `FUN_00afa350` | Lit le relief du segment **et de ses 8 voisins** (1 colonne/ligne avant, 2 après) : `seek(dwTerrainSegmentOffset)`, `seek((n²·23 + 10)·index_segment)`, en-tête de 10 octets (seul celui du segment central est gardé), puis lignes de `n` sommets. |
| `FUN_00afa7a0` | Lit l'en-tête (0x3C), `seek(dwMapPropertiesOffset)` → `FUN_00afaa90` + 1 octet, `seek(dwPropOffset + 4·index)` → décalage du bloc, `u32 n` + `n·0x31` octets d'objets, puis **si version == 0x16** les groupes d'herbe. |
| `FUN_00afaa90` | Propriétés de carte (couleurs lues R, G, B → `D3DLIGHT9`). |
| `FUN_00b01030` | `CTerrainMapEngine::CreateSegment` : objets (`FUN_00b00de0`), herbe `CTerrainSpeedGrassForGame` (x, y + origine segment, z = hauteur du sol). |
| `FUN_00b00de0` | Un `CTerrainPropForGame` par objet : id = `(index & 0xFFFFF) \| (num_carte << 20)`, position (x, y relatifs au segment + origine, z = `FUN_00b24510` hauteur du sol), décalage z, rotation, échelle, `u16` numéro, verrouillage de hauteur. |
| `FUN_00b04da0` | `CTerrainMapEngine::IsBlockedAttribute` → vtable +0x20 du segment. |
| vtable `0x00d87728` | `+0x04 FUN_00af9d70` poids 2 bits d'une couche · `+0x08 FUN_00af9e40` hauteur d'un sommet · `+0x0C FUN_00af98b0` attribut u64 · `+0x10 FUN_00af9e80` texture dominante · `+0x14 FUN_00afa0e0` position d'un sommet · `+0x18 FUN_00afa170` normale · `+0x1C FUN_00afa150` alpha = poids × 0x55 · `+0x20 FUN_00af98f0` sous-case bloquée · `+0x24 FUN_00afa200` hauteur interpolée · `FUN_00af9960` couleur éclairée. |

Le client n'ouvre par leur nom que `.nfm`, `.nfs`, `.qpf` et `.pvs` **[O]** (seules chaînes
`m%03d_%03d.*` du binaire). `.nfa`, `.nfc`, `.nfe`, `.nfw`, `.nfl`, `.nfp` sont donc destinés au
serveur ou aux outils **[D]** (le `.nfm` contient lui-même des copies de `.nfa`, `.nfw`, `.nfe`).

## 2. Système de coordonnées **[O]**

* `TerrainSeamlessWorld.cfg` : `TILE_LENGTH=42`, `TILECOUNT_PER_SEGMENT=6`, `SEGMENTCOUNT_PER_MAP=64`,
  `MAPSIZE=14,16`, puis `MAPFILE=X,Y,couche,Mxxx_yyy,id` (109 cartes). Le commentaire coréen : origine
  (0,0) en **bas à gauche**, Y vers le haut, X vers la droite.
* Carte = 64 × 64 segments, segment = 6 × 6 cases, case = 42 unités → **16128** unités de côté.
* `mXXX_YYY` : `XXX` = X, `YYY` = Y (`sprintf(.., "m%03d_%03d", x, y)`). Origine monde = `(X·16128, Y·16128)`.
  Vérifications sur les données :
  * relief continu entre voisins (70 paires sur 91 « informatives » ont un écart au raccord < 1/3 de
    l'écart au bord opposé ; ex. m009_004 → m010_004 : 24 contre 697 ; axes inversés : > 1000) ;
  * PNJ de `db_npcresource` : 4 % seulement dans un polygone bloquant `.nfa` (qui couvrent ~70 % des cartes),
    contre 62 % si l'axe Y est retourné et 44 % si X/Y sont échangés ; 11 % sur une sous-case d'attribut bloquée ;
  * `.nfs` : les scripts d'objets (`init_npc(1002, …)`) donnent exactement la position de l'objet du `.nfm`.
* Unités : la hauteur est dans la même unité que x, y. Rotations en radians, z vertical.
* Les **positions locales** (relatives à la carte) sont la règle dans les fichiers : objets `.nfm`
  (relatifs au *segment*), `.qpf`, `.nfw`, `.nfc` (centre), scripts d'objets `.nfs`.
  Les polygones `.nfa`/`.nfc`/`.nfe` sont en **unités d'attribut** = 1/8 de case (5,25 unités, 0..3072).
  Les rectangles `.nfs` sont en **cases** (0..384).

## 3. `.nfm` — « nFlavor Map »

### 3.1 En-tête **[O]**

| Offset | Type | v22 (`0x16`, 122 fichiers) | v16 (14 fichiers, dont M002_002 qui est dans le .cfg) |
|---|---|---|---|
| 0x00 | `char[16]` | `"nFlavor Map"` + `\0` | idem |
| 0x10 | u32 | version = 22 | 16 |
| 0x14 | u32 | `dwMapPropertiesOffset` (= 0x3C) | 0x34 |
| 0x18 | u32 | `dwTerrainSegmentOffset` | idem |
| 0x1C | u32 | `dwPropOffset` | idem |
| 0x20 | u32 | `dwVectorAttrOffset` | idem |
| 0x24 | u32 | `dwWaterOffset` | idem |
| 0x28 | u32 | `dwGrassColonyOffset` | — (v16 : `TILECOUNT_PER_SEGMENT`) |
| 0x2C | u32 | `dwEventAreaOffset` | — |
| 0x30 / 0x28 | i32 | `TILECOUNT_PER_SEGMENT` = 6 | |
| 0x34 / 0x2C | i32 | `SEGMENTCOUNT_PER_MAP` = 64 | |
| 0x38 / 0x30 | f32 | `TILE_LENGTH` = 42.0 | |

Les sections se suivent sans trou dans l'ordre des décalages ; la dernière va jusqu'à la fin du fichier
(vérifié sur les 136 fichiers).

### 3.2 Propriétés de carte (55 octets) **[O]**

```
rgb  primary_diffuse;  rgb primary_ambient;  f32 primary_dir[3];     // 1re lumière (D3DLIGHT9 +4 / +0x24 / +0x40)
rgb  secondary_diffuse; rgb secondary_ambient; f32 secondary_dir[3]; // 2e lumière
rgb  sky_color; rgb fog_color;  f32 fog_near; f32 fog_far;  u32 sky_type;
u8   show_terrain;                                                   // lu à part vers +0x128 ; nom [I]
```
Couleurs en octets **R, G, B** (lus dans cet ordre). Ex. m000_000 : lumières blanches, direction (1/3, 2/3, −2/3),
ciel `CC DE F8`, brouillard 500 → 5500, `sky_type` 1.

### 3.3 Relief **[O]**

64 × 64 segments de `10 + 36·23` = **838 octets**, segment `s = sy·64 + sx` (ligne par ligne, X d'abord) :
```
u32 version;            // 0 partout [I]
u16 tile[3];            // textures : base, couche 1, couche 2 (identifiants TerrainTextureInfo.cfg)
KVertex v[6][6];        // v[vy][vx], X d'abord
KVertex {               // 23 octets, sans alignement
  f32 height;
  u32 fill_bits[2];     // poids des couches 1 et 2
  u64 attribute;        // blocage
  u8  color[3];         // ordre B, G, R (FUN_00af9960) ; gris ffffff le plus souvent
}
```
* Sommet global `(i, j) = (sx·6 + vx, sy·6 + vy)`, position locale `(i·42, j·42)` → grille 384 × 384.
  La case (383, j) est fermée par le sommet 0 de la carte voisine (le client lit 2 sommets « après »
  chez le voisin). Hauteurs observées −1775 … 5724.
* **Hauteur interpolée** (`FUN_00afa200`) : case coupée selon la diagonale (i+1, j)–(i, j+1) ;
  `u + v < 1` → plan (h00, h10, h01), sinon plan (h10, h01, h11).
* **Poids de texture** (`FUN_00af9d70`, `FUN_00afa150`) : la case du sommet est divisée en 4 × 4 ;
  la sous-case (sx, sy) vaut `(fill_bits[k] >> 2·(sy·4 + sx)) & 3`, alpha = valeur × 0x55.
  Couche 0 = `tile[0]` opaque, couche 1 = `tile[1]` × alpha(`fill_bits[0]`), couche 2 = `tile[2]` × alpha(`fill_bits[1]`).
  « Texture dominante » (`FUN_00af9e80`) : `tile[2]` si `fill_bits[1] ≠ 0`, sinon `tile[1]` si `fill_bits[0] ≠ 0`, sinon `tile[0]`.
  Les 3 textures sont communes aux 36 cases du segment.
* **Attribut** (`FUN_00af98f0`) : case divisée en 8 × 8, bit `sy·8 + sx` de `attribute`, 1 = bloqué
  (`IsBlockedAttribute` renvoie 1 par défaut). Coordonnées passées en 1/8 de case.
* **Couleur éclairée** (`FUN_00af9960`) : `couleur_sommet × min(1, diffuse·max(0, −n·dir) + ambiante)`
  par composante.

`TerrainTextureInfo.cfg` : `TEXTURE=id,fichier.dds`, `DETAIL=` (répétition : 1 texture pour DETAIL cases)
et `CATEGORY=` s'appliquent aux lignes suivantes ; 398 textures. 96 % des couples (carte, texture) se
résolvent ; quelques identifiants (24467…, 44337…, 50006…) sont absents du fichier **[O]**.

### 3.4 Objets placés et herbe (`dwPropOffset`) **[O]**

```
u32 block_offset[64·64];         // décalage ABSOLU dans le fichier du bloc du segment s
bloc :
  u32 prop_count;
  PropRecord props[prop_count];   // 49 octets
  si version == 22 :
    u32 speedgrass_count;
    { u32 grass_id; u32 n; { f32 x, y; f32 nx, ny, nz; } [n] } [speedgrass_count]
PropRecord {                      // 49 octets, sans alignement
  u32 index;                      // 20 bits utiles ; id global = index | ((map_y·14 + map_x) << 20)
  f32 x, y;                       // relatifs au SEGMENT (0..252)
  f32 z_offset;                   // ajouté à la hauteur du sol (0, −18.5, … : la hauteur du sol est recalculée)
  f32 rot_x, rot_y, rot_z;        // radians ; rot_z = lacet (le plus souvent seul non nul)
  f32 scale_x, scale_y, scale_z;
  u16 prop_num;                   // PROPNAME de TerrainPropInfo.cfg, modulo 65536 (voir ci-dessous)
  u8  height_locked;              // si ≠ 0, f32 suivant copié dans l'objet (+0x24) [D : hauteur absolue]
  f32 locked_height;
  i16 texture_group;              // −1 le plus souvent
}
```
* Les blocs sont contigus et couvrent exactement la section dans les 106 cartes du `.cfg`.
  Dans 4 cartes absentes du `.cfg` (m004_010, m004_013, m005_010, m006_010) la table de décalages
  est périmée (blocs illisibles) : la crate ignore ces blocs et le signale dans `warnings`.
* `z_offset` : le client passe `pfVar8[2]` à part et prend `z = hauteur du sol` (`FUN_00b24510`) ;
  99 % des valeurs sont dans ±50 alors que le sol monte à plus de 1000 → décalage **[D]**.
* **Modèle** : `TerrainPropInfo.cfg`, `PROPNAME=num,fichier.cob` (3458 entrées, dont 1165 > 65535) ;
  `CATEGORY`, `RENDERTYPE=general|building`, `LIGHTMAPTYPE=normal|collision|none`,
  `VISIBLE_RATIO` s'appliquent aux lignes suivantes. Le `u16` du fichier ne contient que les 16 bits
  bas : 18819 désigne `PROPNAME=215427` (18819 + 3·65536). Avec une table indexée sur 16 bits
  (dernière définition gagnante), **99,8 %** des 87 372 objets des cartes du monde ont un modèle
  (82 % seulement sans ce repli) **[D]**.
* Herbe « SpeedGrass » : `x, y` relatifs au segment, 3 réels ≈ vecteur unitaire (normale) **[D]**,
  `grass_id` [I]. 7 cartes du monde en contiennent (m006_003 : 123 070 touffes, m007_008 : 136 390, m010_003 : 89 371…) ; ce sont les mêmes cartes qui ont des zones d'herbe (section 6).

### 3.5 Sections annexes

| Section | Contenu | Fichier équivalent |
|---|---|---|
| `dwVectorAttrOffset` | polygones bloquants **[O]** | identique au `.nfa` dans la plupart des cartes |
| `dwWaterOffset` | eau, format `.nfw` **[O]** | **vide (u32 0) dans toutes les cartes du monde** ; les `.nfw` des cartes du `.cfg` totalisent 69 zones |
| `dwGrassColonyOffset` | zones d'herbe procédurale (v22) **[O]** | aucun (≠ `.nfp`) |
| `dwEventAreaOffset` | zones d'événement **[O]** | identique au `.nfe` |

Zone d'herbe (section 6) **[O]** structure, **[I]** sens des réels :
```
u32 count; {
  u32 id; f32 a[5]; u32 color_argb; f32 b[6];   // ex. 7,16,10,0.1,0.3 | ff859564 | 0.6,1,0.03,0.05,50,0.2
  u32 len; char texture[len];                    // "grass_mix2.dds\0" (peut être vide)
  u32 polygon_count; { u32 n; i32 x, y [n]; }    // unités monde locales (≤ 16070)
} [count]
```

## 4. Fichiers par carte

### `.nfa` — polygones bloquants (serveur) **[O]**
`u32 count; { u32 n; i32 x, y [n]; } [count]` en 1/8 de case. Fichier vide possible (m007_001).
119/121 fichiers conformes ; m006_009 (absente du `.cfg`) est incohérent.

### `.nfw` — eau **[O]** structure, **[D]** sens
`u32 count; WaterArea[count]` de 44 octets :
`f32 corner_a[3]; f32 corner_b[3]; f32 center[3]; u32 flags (0/1) [I]; u32 water_id`.
Rectangle aligné sur les axes, coordonnées locales, z = niveau (identique dans les 3 points).
Ex. m000_001 : (0, −36) – (16182, 16128), niveau 102, type 1012. Types 1000–1037 [I] (probablement
`water.cfg` / `db_LowQualityWater.rdb`). 86 fichiers, tous conformes (m005_010, absente du `.cfg`, a une section eau corrompue dans son `.nfm`).

### `.nfe` — zones d'événement **[O]**
`u32 count; { u32 id; u32 polygon_count; { u32 n; i32 x, y [n]; } } [count]` (1/8 de case).
Ex. m001_001 : 33 zones 10101, 10102…

### `.nfc` — zones nommées / régions **[O]** structure
```
u32 count; {
  u32 priority;            // 1..4 [D]
  f32 x, y, z;             // centre, local
  f32 radius;              // 0 partout [D]
  u32 len; char name[len]; // coréen EUC-KR, \0 inclus (peut être vide)
  u32 len; char script[len]; // "call_lc_In( 10100 )" → id WorldLocation [D]
  u32 polygon_count; { u32 n; i32 x, y [n]; }   // 1/8 de case
} [count]
```
100/109 conformes ; les 9 autres (toutes absentes du `.cfg`) ont un compteur périmé.

### `.nfp` **[O]** structure, **[I]** rôle
`u32 count; { u32 id; u32 polygon_count; { u32 n; f32 x, y, z [n]; }; u32 len; char name[len] ("M010_003|") } [count]`.
33 fichiers, la plupart à 0 entrée.

### `.nfl` — lumières ponctuelles **[D]**
24 octets d'en-tête (`0,0,0, ffffffff, ffffffff, 00ff00ff` [I]), `u32 count`, puis 44 octets :
`f32 pos[3]; f32 params[3] (portée/atténuation ?); f32 value; u32 color[3] (ARGB); u32 kind (3)`.
40 fichiers, tous en 44 octets sauf m005_001 (32 octets, absente du `.cfg`).

### `.qpf` — « nFlavor QuestProp » **[O]**
`char[18] "nFlavor QuestProp\0"; u32 version = 3; u32 count; PropRecord[count]` (49 octets, comme le `.nfm`),
mais `x, y` relatifs à la **carte**. `index` = identifiant `db_fieldpropresource` (3618/3622 trouvés ;
le modèle est son champ `file_name` [D]) ; `prop_num` en est le plus souvent une copie. 108 fichiers conformes.

### `.nfs` — « nFlavor Script » **[O]**
```
char[16] "nFlavor Script"; u32 version (= 2 partout); u32 off_areas (0x20), off_events, off_props;
u32 n; { i32 left, top, right, bottom; u32 len; char name[len]; } [n]   // en cases (×42 → unités)
u32 n; { u32 area_index; u32 f; { u32 trigger; u32 len; char script[len]; } [f] } [n]
u32 n; { u32 prop_id; f32 x, y; u16 prop_num; u32 f; { u32 trigger; u32 len; char script[len]; } [f] } [n]
```
* v2 : le client convertit les rectangles par `(v + 384·carte)·42` ; v1 : `v + 384·carte` (pas de ×42).
* `prop_id` = id global de l'objet `.nfm` (`index | (map_y·14 + map_x) << 20`), position locale ; trigger 0 =
  initialisation (`init_npc(...)`, `mob(...)`), 1 = contact (`NPC_..._contact()`). Les 3 sections du client
  ne lisent que les deux premières. 119/122 conformes (3 absentes du `.cfg` ont des octets en trop).

### Autres
* `.pvs` : `"RAPPELZ PVS"`, ensembles de visibilité (`m%03d_%03d.pvs` lu par le client) **[I]**.
* `.gci` (7 fichiers), `.bmp`, `.tif` : non analysés **[I]**.
* `mXXX_YYY_<l>_<c>.jpg` : 16 × 16 morceaux de la minicarte.

## 5. Où sont les objets 3D

1. **`.nfm`, section `dwPropOffset`** : tous les objets du décor (arbres, rochers, bâtiments, ponts,
   PNJ « objets » comme `npc_dem_common002.cob`) → modèle via `TerrainPropInfo.cfg`.
2. **`.qpf`** : objets de quête (coffres, plantes à récolter) → `db_fieldpropresource`.
3. **`.nfs` section 3** : scripts attachés à des objets du `.nfm`.
4. **SpeedGrass** (blocs d'objets v22) et **zones d'herbe** (section 6) : végétation générée.

## 6. Restes inconnus

* `SegmentInfo.version`, `show_terrain`, `sky_type` : sens exact.
* Ordre d'application des rotations des objets ; interprétation exacte de `height_locked`.
* Eau : d'où le client tire-t-il l'eau, la section du `.nfm` étant vide ? (`water.cfg`, `db_LowQualityWater.rdb` ?)
* Paramètres des zones d'herbe, `grass_id`, rôle du `.nfp`, en-tête et paramètres du `.nfl`, `.pvs`, `.gci`.

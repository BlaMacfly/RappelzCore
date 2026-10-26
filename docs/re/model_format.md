# Modèles et animations Rappelz (moteur nFlavor) : `.nx3`, `.naf`, `.cob`

Client analysé : `Rappelz NoLimits\SFrame.exe` (v1.3.60, sha256 `27f42fc8…a410d`), Ghidra 12.1.4 via REA.
Le binaire contient les chemins de sources (`c:\users\administrator\downloads\source9.5.2\client\game\engine\renderer\scobmanager.cpp`).
Implémentation : crate `shared/rz-model` (`cargo test -p rz-model`, balayage complet :
`cargo test -p rz-model --release -- --ignored --nocapture`).

Légende : **[O]** observé (binaire et/ou vérifié sur les vrais fichiers), **[D]** déduit, **[I]** inconnu.

Résultat du balayage complet [O] : 22 061 / 22 072 `.nx3`, 20 825 / 20 825 `.naf`, 4 311 / 4 312 `.cob`
lus et vérifiés (index < nb de sommets, UV/positions finies, poids normalisés, parents d'os valides,
quaternions unitaires). Les 11 + 1 restants sont inexploitables aussi pour le client : 9 nx3 utilisent
un gabarit absent des `.tml` (GUID `{A35E3135-98DC-4215-A06F-F5594213CBEF}`, exception « unknown template GUID »
dans FUN_005d09b0 : `npc_dyuel_*`, `npc_juena_*`, `ilfx_chekpoint_on_start_fx01`…), 2 nx3 sont vides
(`novice_builddustbeil_cacol/height.nx3`), 1 cob est tronqué à 12 octets (`w004_ron_v001_p_armory_ea005.cob`).

---

## 1. Fonctions du client

| Adresse | Rôle |
|---|---|
| `FUN_005c9450` | Lit 4 octets, recule, renvoie `== 'BTRF'` (0x46525442) |
| `FUN_00ac8330` | Chargement d'un `.nx3` : BTRF → `FUN_005d07d0` + `FUN_00a7c6b0`, sinon `KBinaryFiler` (`FUN_005db010`/`FUN_005db1e0`) + `FUN_00a78520`. Trace « `NX3 Load : %s (%dms) (%dKB)` » |
| `FUN_005d07d0` | Document BTRF : magie, en-tête, puis table d'ids, table de chaînes, objets ; drapeau « v1.0 » si `u16@8 == 1 && u16@10 == 0` |
| `FUN_005d09b0` | Table d'identifiants (GUID + nombre de membres) ; erreurs « idtable size error! », « unknown template GUID » |
| `FUN_005cafa0` | Recherche d'un gabarit par GUID (table de hachage `(d4lo + d4hi>>16 + d1) % n`) |
| `FUN_005d0be0` | Tables de chaînes (ASCII puis UTF-16) ; « stringtable size error! », « wstr table count error » |
| `FUN_005d0570` | Liste des objets de premier niveau |
| `FUN_005d0010` | Découpe des membres d'un objet (taille fixe du gabarit ou préfixe u32) |
| `FUN_005c6480` | Taille d'un code de type : 1→1, 2→2, 3/5/7/8→4, 4/6→8 |
| `FUN_005db1e0` | Lecteur KNIB : en-tête `DIFK` `NIBK` (0x80 octets), blocs `KLBK` de 0x20 octets |
| `FUN_005daf70` | `KTemplateDataObject` (objet KNIB) |
| `FUN_00a7c6b0` / `FUN_00a78520` | `KNX3ChannelControl::LoadTRF` / `::Load` : aiguillage par nom de gabarit (`nx3_version_header`, `nx3_property`, `nx3_mtl_header`, `nx3_mesh_header`, `nx3_new_mesh_header`, `nx3_light_header`, `nx3_camera`, `nx3_bone_ani_header`) |
| `FUN_00a7f040` | Matériaux (`mtl_array` → `sub_mtl_block_array`) |
| `FUN_00a7e210` | En-tête de maillages : `mesh_array`, puis `mesh_tm_array` (struct de 0x80 : nom[64] + matrice[16]) |
| `FUN_00a7dc30` | Nœud de maillage : `mesh_name`, `material_id`, blocs, `fx_array` (notes `UVTRANSFORM`, `UVANI`), enfants (récursif) |
| `FUN_00a7e5a0` | Bloc : préfixes `ep_` (point d'effet) / `ev_` (événement), `texture_index`, `lightmap_index`, index u16 |
| `FUN_00a7ed20` | Image de maillage : `time_value`, tableaux, `mesh_tm`, `bone_block` |
| `FUN_00a80440` | Tampon de sommets de 64 octets : position, normale, couleur, UV0, UV1 (si `texel_array` = 4 flottants/sommet), poids ; sphère et boîte englobantes |
| `FUN_00a7ba20` | Animation d'os : `bone_count`, `channel_name/flag/time_span/frame_rate`, `bone_ani_array`, liens parent/enfants |
| `FUN_00a7aee0` | Un os animé : `szName`[64], `parent_Index`, `base_tm`, `flag`, clés ; réduit les clés (position : écart < 0,005 ; rotation : `|xyz| < 0,002` et `w > 0,99`) |
| `FUN_00af0920` | `COBManager::load` (cache par nom) ; `FUN_00af0170` charge un cob : BTRF → `FUN_00aedee0`, sinon `FUN_00aecd70` |
| `FUN_00643a30` | Chargement de `db_DefaultModelRes.rdb` |
| `FUN_00986b10` | Lecture d'animation : vitesse passée au contrôleur = `vitesse × 4.8` (ticks par ms) |

Chaînes utiles : `0x00d6006c` `_biped.naf` … `_weaponr.naf` (11 suffixes), `0x00d60110` noms de parties,
`0x00d52c88` `%s.cob`, `0x00d64d00` `gam.nx3`…`asf.nx3` [I : usage non suivi].

## 2. Gabarits (`.tml`) [O]

Les définitions de gabarits sont des fichiers texte de l'archive : `nx3.tml`, `nobj.tml` (cob),
`pfx.tml`, `sprite.tml`, `qjtv.tml`. Syntaxe des gabarits DirectX `.x` :

```text
template nx3_mesh_block
{C817F7B0-E4E7-40fb-97B3-2B97CC000521}
{
    dword          texture_index;
    dword          mesh_frame_size;
    nx3_mesh_frame mesh_frame_array[mesh_frame_size];
    dword          index_buffer_size;
    word           index_buffer_array[index_buffer_size];
    dword          lightmap_index;      // ajouté plus tard : les vieux fichiers ont 3 membres stockés
}
```

Types : `char` (1), `word` (2), `dword`, `float`, `string`, `dict` (bloc), ou un autre gabarit ;
tableaux de taille fixe `[16]` ou variable `[membre]`. Le texte est repris dans `rz_model::tml::SCHEMA_TML`.

**GUID** : texte → octets comme un GUID Windows (Data1/2/3 little-endian, Data4 tel quel). Certains GUID
de `nobj.tml` ne sont pas hexadécimaux (`{G2E978BB-18F4-5357-CGC9-7547FE667886}`) : le moteur calcule
chaque chiffre `0-9`, `A-F` normalement et toute autre lettre comme `toupper(c) - '0'` (G = 23), en
accumulant `v = v*16 + chiffre` tronqué à la taille du champ. Vérifié : `cob_header_v13` = `bb78e972 f418 5753 d7c9 7547fe667886`.

## 3. Conteneurs

### 3.1 BTRF (19 459 nx3, 6 126 naf, 2 978 cob) [O]

```text
"BTRF"                      4
u32  ext_size               (= 4)
u8   ext[ext_size]          u16 major = 1, u16 minor = 1 (154 naf et 127 nx3 en 1.0)
u32  id_size                ; N = id_size / 18
     GUID[N]                16 octets chacun
     u16 count[N]           nombre de membres STOCKÉS du gabarit (versionnement)
     (bourrage jusqu'à id_size, octets « 12 »)
u32  str_size               ; bloc [p+4, p+4+str_size)
     u32 n ; n chaînes C ; u32 nw ; nw chaînes UTF-16 (vides dans nx3/naf/cob)
u32  data_size
     u32 nobj
     nobj × { en-tête 4 octets (5 en v1.0), type = dernier octet ;
              type 1..8 : primitif de taille fixe, sinon u32 taille ; contenu }
```

Objet (type 9) : `u16 index` (1-based dans la table d'ids) puis les membres stockés (`count` premiers
de la liste des membres **sans les compteurs de tableaux** : `dword x_size` qui dimensionne un tableau
n'est pas écrit). Chaque membre :

- primitif scalaire ou tableau de taille fixe : taille fixe, pas de préfixe (`string` = u32 index
  1-based dans la table de chaînes, 0 = vide) ;
- sinon `u32 taille` puis :
  - tableau primitif variable : `u8 code` (1 char, 2 word, 3 dword, 5 float, 7 string) puis les éléments ;
    taille 0 = tableau vide (pas de code) ;
  - tableau d'objets : `u16 index de gabarit`, `u32 n`, puis n éléments ; chacun précédé d'un
    `u32 taille` sauf si tous les membres stockés du gabarit sont de taille fixe ;
  - objet scalaire : `u16 index` puis ses membres.

### 3.2 KNIB / `DIFKNIBK` (2 611 nx3, 14 699 naf, 1 333 cob) [O]

```text
"DIFKNIBK" i32 version(1) i32 (0) i32 nblocs ... (en-tête 0x80 octets)
nblocs × { "KLBK" u32 taille 24 octets ; objet balisé }
```

Balises : `0x1f` objet = `GUID, u32 n, n membres balisés` (tous les membres, compteurs compris, dans
l'ordre du gabarit) ; `0x28` word ; `0x29` dword ; `0x2a` float ; `0x2c` char ; `0x33` chaîne
(`u32 longueur avec NUL` + octets) ; `0x34` tableau = `u8 balise d'élément [+ GUID si 0x1f], u32 n,
chaîne nom du compteur` (vide pour une taille fixe) puis les éléments (bruts, ou objets complets `0x1f…`).

Les deux conteneurs donnent le même arbre (`rz_model::container::Document`). Chaînes : octets bruts
(chemins d'outils en CP949) lus en Latin-1.

## 4. `.nx3` : modèles

Objets de premier niveau : `nx3_version_header {version = 0x10000}` (optionnel), `nx3_mtl_header`,
`nx3_new_mesh_header` (ou `nx3_mesh_header`/`old_nx3_mesh_header`), parfois `nx3_camera`, `nx3_light_header`.

- **Matériaux** [O] : `mtl_array[material_id].sub_mtl_block_array[texture_index]` (`mtl_name`,
  `texture_name`, `mtl_id`, `channel_id`, `power`, `self_illumi`, `smoothing`, couleurs ARGB).
  `texture_name` est un chemin d'outil (`H:\Graphic\…\asm_common001.dds`, parfois `.tga`/`.psd`) ; le
  fichier à charger est le **nom de base en minuscules avec l'extension `.dds`** [D, vérifié : 99 %
  trouvés dans l'archive contre 48 % sans remplacement d'extension].
- **Nœuds** : `mesh_array` puis `mesh_children_array` (récursif). Chaque `nx3_mesh_block` est un
  sous-maillage : `texture_index`, `mesh_frame_array` (image 0 = géométrie ; aucune image
  supplémentaire observée), `index_buffer_array` (u16, liste de triangles ; quelques fichiers ont un
  reste non multiple de 3, ignoré), `lightmap_index`.
- **Image** `nx3_mesh_frame` [O] : `vertex_array` (xyz), `normal_array`, `texel_array` (uv, ou 2 jeux
  si 4 flottants/sommet : UV0 puis UV lightmap), `color_array` (vide partout), `bone_block`, `mesh_tm[16]`.
- **Os** [O] : `mesh_tm_array` = `{name, tm[16]}` = **matrice de liaison inverse** (modèle → os) de
  chaque os, alignée sur les positions de `bone_block`. Le nx3 ne contient **aucune hiérarchie**.
- **Poids** [O] : `bone_block[i]` (un par os, `NotUse` si inutilisé) : `weight_array` = paires
  `(index de sommet en flottant, poids)`, `offset_vector_array` = position du sommet dans le repère de
  l'os, **égale à `sommet × tm`** (écart max 3·10⁻⁶ sur `asm_body_common001`). Somme des poids = 1.
- **Animation de nœud** [O/D] : `ani_time_array` + `ani_matrix_array` (12 flottants = 4 lignes de 3,
  translation en dernier) ou `ani_block_array` (ancien). Les sommets d'un maillage non skinné sont dans
  le repère local du nœud : position monde = `v × clé(t)` (ex. `zone005_prop_rail_02` : la clé fait
  passer de Y-haut à Z-haut). Sans clé : `mesh_tm` [D]. Pour les maillages skinnés, `mesh_tm` et les
  clés valent l'identité ou une échelle (0,947 pour `desertwildcat_body`) ; le skinning ne l'applique
  pas [I : rôle exact de cette échelle].
- `visi_time_array`/`visi_value_array` : visibilité animée ; `fx_array` : notes (`UVTRANSFORM`, `UVANI`…).
- Noms `ep_*` (points d'effet) et `ev_*` (événements) : non rendus (FUN_00a7e5a0).

### Conventions [O]

- Repère 3ds Max : **Z vers le haut**, personnages tournés vers **−Y**, unité ≈ 10 cm (asura 18,5,
  déva femme 16,4 unités en pose d'attente). Certaines pièces (cheveux `*_newhair_*`, décors) sont en
  Y-haut dans leur repère de liaison : le skinning (ou la clé de nœud) les remet en place.
- Matrices en lignes, vecteurs lignes (`p' = p · M`, translation en dernière ligne), comme D3DX.
- Triangles en **sens horaire** vus de face (1640 contre 4 sur `asm_body_common001`).
- UV au sens D3D, copiés tels quels dans le tampon de sommets (FUN_00a80440) ; les V sont souvent
  négatifs (échantillonnage WRAP). Pour OpenGL : `(u, −v)`.

## 5. `.naf` : animations [O]

Un objet `nx3_bone_ani_header {bone_count, channel_array}` (parfois précédé de `nx3_version_header`).
Tous les fichiers observés ont un seul canal :
`nx3_bone_ani_channel {channel_name, channel_flag, channel_time_span, channel_frame_rate (30), bone_ani_array}`.
Os : `szName`, `parent_Index` (−1 = racine), `base_tm[16]`, `flag` (0x101), `pos_time_array` +
`pos_key_array` (xyz), `rot_time_array` + `rot_key_array` (quaternion **x, y, z, w**), `child_Index_array`.

- **Temps en ticks 3ds Max, 4800/s** : `time_span` 3841 pour la course asura (0,8 s), 6401 pour une
  attaque (1,33 s), clés tous les 100 ticks ; le client avance de `vitesse × 4.8` par ms (FUN_00986b10).
- **Clés absolues dans le repère racine du fichier, pas relatives au parent** : position de `Body2`
  − position de `Pelvis` = 1,473 = distance de liaison. `base_tm` = inverse de la pose à t = 0
  (`R(q)` puis translation, conjugaison/translation vérifiées).
- `*_biped.naf` : repère **modèle**. Parties (`*_hair.naf`, `*_weaponr.naf`, `*_mantle.naf`…) : racine
  `Dummy_<partie>_<os>` (`Dummy_hair_Head`, `Dummy_weaponR_Bip01 R Hand`) et clés dans le repère de
  l'os d'attache du squelette (`Head`, `Bip01 R Hand`) [D, vérifié visuellement] : monde = `clé × monde(os d'attache)`.
- Noms d'os : `Bip01 R Hand` (naf) ≡ `Bip01_R_Hand` (nx3), `@HideBip01` ≡ `_HideBip01` : comparer
  après normalisation (minuscules, espace et `@` → `_`).

**Skinning** [O, vérifié visuellement] : `M_os(t) = tm_os (liaison inverse du nx3) × monde_os(t)`,
sommet = `Σ wᵢ · (v × M_osᵢ)`. Chaque nx3 a ses propres liaisons inverses, ce qui recolle des pièces
exportées dans des orientations différentes.

## 6. `.cob` : objets composés [O]

Un objet `cob_header_v10…v13` **par partie d'animation** (`obj_partindex` 0..10 = biped, hair,
necklace, earring, mantle, fabric, longskirt, middleskirt, shortskirt, weaponl, weaponr). Champs :
`obj_clan` (race : 3 gaïa, 4 déva, 5 asura ; 1 monstre…), `obj_sex` (1 femme, 2 homme, 0),
`render_type`, `prop_category`, `shadow_type`, boîtes `sel/dead/visible/collide_cube[6]`,
`effpos_list` (`asm_top.nx3`…), `lod_list`, `height_file_name`, `camera_collision_file_name`
(`empty` = aucun), `mesh_list` (liste plate, décors), `mesh_part_list` (emplacement → fichiers nx3),
`ani_list` (naf de la partie), `ani_event_list`, `mesh_texture_list` (v13 : séquences de textures).

Emplacements (`mesh_part_num`) observés : 0 visage, 1 cheveux, 2 corps, 3 pieds, 4 mains, 6/7 jupes,
13 tissu (`def_fabric_*`) [D d'après les noms].

## 7. Assemblage des apparences

### Joueurs

- Préfixe race/sexe [O] : `gam`/`gaf` (3), `dem`/`def` (4), `asm`/`asf` (5) ; sexe 2 = homme, 1 = femme
  (cf. `obj_clan`/`obj_sex` de `asm.cob` = 5/2). Le cob de race (`asm.cob`) liste les maillages
  possibles par emplacement et toutes les animations de chaque partie (≈ 300).
- `db_DefaultModelRes.rdb` [O] : en-tête 0x80, `u32 n` (44), enregistrements de 1552 octets :
  `i32 id, i32 type, i32 sous-type, char[256] × 6 (dem, def, asm, asf, gam, gaf), i32 index`.
  Types : 14 cheveux (101-106, 701-705, 10101-10106 → `*_newhair_common00N`), 15 visage (201-211 →
  `*_newface_common001`), 16 corps nu (301 → `*_body_under001`), 5 mains (401 `hand_under001`,
  400 sous-type 2), 6 pieds (501/500), 2 corps (600-604), 100 visages de PNJ (10201-10206).
- **Personnage nu par défaut** (`default_character`) : `<p>_newface_common001`, `<p>_newhair_common001`,
  `<p>_body_under001`, `<p>_hand_under001`, `<p>_foot_under001` (.nx3) ; attente sans arme
  `<p>_una_default01_biped.naf` + `<p>_una_default01_hair.naf`. Animations :
  `<p>_<arme>_<mouvement>_<partie>.naf`, arme ∈ una, swo, swt, bwa, cba, dgo, sfa, spa, swa, axa.
- `LOBBY_CHARACTER_INFO.model_id[5]` : ids de `db_DefaultModelRes` ; le serveur range [0] cheveux,
  [1] visage [D] ; [2..4] probablement corps/mains/pieds (301/401/501) [I] : `character_meshes`
  résout chaque id par la table, sans supposer l'ordre. Variantes de visage 201-211 : même maillage,
  différence portée ailleurs (texture ?) [I].
- Équipement : `db_item.model_00..17` (chaînes de `db_item.ref`) = 6 colonnes race/sexe × 3 variantes
  (`item_model_file`) ; les 3 variantes sont identiques pour 99 % des objets, règle de choix [I] ;
  l'emplacement remplacé (corps, mains…) se lit dans le nom/`wear_type` [D].

### PNJ et monstres [O]

`NpcResource.model_file` et `Monster.model` → `"%s.cob"` (en minuscules ; quelques valeurs ont déjà
`.cob`) : 103/105 noms de PNJ distincts non vides et 467/479 noms de monstres (dont 6
`monstertest0N`) existent dans l'archive. Le cob donne les nx3
(un par emplacement, + `mesh_list`) et les animations par partie ; l'attente est `*_default01_*`
(`def_una_default01_biped.naf`, `desertwildcat_una_default01_biped.naf`), `*_idle01_*` étant une
variante. Le monstre est skinné par son seul `*_biped.naf` (squelette `Bip01`).

## 8. Inconnus

- Rôle de `flag`/`channel_flag` des naf, des 3 octets nuls de l'en-tête d'objet BTRF, de
  `channel_id`, `power`, `self_illumi` dans le rendu.
- Échelle portée par `mesh_tm` / clés de nœud des maillages skinnés de monstres.
- Ordre exact de `model_id[5]`, variantes 1-2 de `db_item.model_*`, variantes de visage 201-211.
- Gabarit `{A35E3135-…}` (9 fichiers d'un autre exporteur, illisibles par le client).
- Usage des noms `gam.nx3`…`asf.nx3` du binaire (aucun fichier de ce nom dans l'archive).

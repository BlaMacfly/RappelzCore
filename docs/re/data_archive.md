# Archive de ressources Rappelz : `data.000` + `data.001`..`data.008`

Client analysé : `Rappelz NoLimits\SFrame.exe` (PE x86, sha256 `27f42fc8f6b08df8c239640b938d601aeaebb7b3db0c89fc47755325a98a410d`),
avec Ghidra 12.1.4 via REA. Les fichiers du client n'ont été ouverts qu'en lecture.

Légende : **[OBS]** observé dans le binaire ET vérifié sur les vrais fichiers, **[BIN]** lu dans
le binaire mais pas testé sur les fichiers, **[DED]** déduit, **[INC]** inconnu.

Script de validation qui fonctionne (Node.js, car Python n'est pas installé sur ce poste) :
`C:\Users\Marty\Desktop\RappelzCore\docs\re\tools\rappelz_data.js`
(`node rappelz_data.js "<dossier client>" selftest [dossier_sortie]`, puis `stats`, `list [regex]`, `extract <regex> <dossier>`).

---

## 1. Fonctions analysées

| Adresse | Rôle |
|---|---|
| `FUN_00b87450` | Initialisation : `FUN_005dfeb0("data.000", …)` puis `FUN_005ddd40(&DAT_00ede970, &PTR_00e99b40, 1, 1)` |
| `FUN_005dfeb0` | Ouvre le paquet (chemin et nom de l'index), appelle `FUN_005e2170` (chargement de l'index) puis `FUN_005e2660` |
| `FUN_005e2170` | Boucle de lecture de `data.000` jusqu'à EOF ; insère chaque entrée dans une table de hachage (`h31(lower(nom)) % nb_seaux`) |
| `FUN_005df680` | **Lit une entrée** d'index et la déchiffre (XOR avec clé roulante) ; calcule le n° de fichier data via `FUN_005e3620` |
| `FUN_005e3620` | **Hash → n° du fichier data** (`abs(h31) % N`) |
| `FUN_005e2660` | Construit les listes d'espace libre de chaque data.00N ; écrit `corrupted.txt` si deux entrées se chevauchent |
| `FUN_005e2a80` | Réécrit l'index (`data.000`) : même format, même chiffrement (sert au patcher) |
| `FUN_005e1080` | Ajoute ou remplace un fichier dans le paquet (patch) |
| `FUN_005e1f50` | Supprime une entrée (recherche par nom encodé) |
| `FUN_005c2b90` / `FUN_005c3560` | Clé de recherche : copie de chaîne + `h = h*31 + tolower(c)` |
| `FUN_005ddd40` | Configure le système de fichiers : `+8` = paquet, `+0xC` = encodeur de noms (objet `0x00e99b40`), `+4` = chiffrement actif (1), `+5` = liste d'exclusion active (1) |
| `FUN_005ddf20` | **Ouverture d'un fichier par son nom** : passe le nom en minuscules, l'encode (vtable+4 de l'encodeur), l'ouvre, puis emballe le flux dans un flux XOR sauf pour les extensions exclues |
| `FUN_00b8fe70` → `FUN_005cf720` | **Encodage du nom de fichier** |
| `FUN_005d8740` | Constructeur du flux de déchiffrement (vtable `0x00dfcbac`) |
| `FUN_005d87d0` | `read()` du flux de déchiffrement : XOR du contenu avec la même table de 256 octets |
| `FUN_005d8e80` | Ouvre un fichier du paquet (objet `XFile`, position relative à 0) |

## 2. Format de `data.000` (index) **[OBS]**

Tout le fichier est chiffré par un XOR avec une clé de 256 octets **à index roulant sur
l'ensemble du fichier** : l'octet n° `i` du fichier (en partant de 0) est XORé avec `KEY[i & 0xFF]`.
Le compteur n'est **pas** remis à zéro entre deux entrées. Déchiffrer tout le fichier d'un coup
revient donc au même.

Après déchiffrement, le fichier est une simple suite d'entrées, sans en-tête, sans compteur et
sans terminateur :

```
struct IndexEntry {          // taille = 1 + name_len + 8
    u8   name_len;           // longueur du nom ENCODÉ (= longueur du nom clair + 2)
    u8   name[name_len];     // nom encodé, ASCII, sans '\0'
    u32  offset;             // little-endian : position dans data.00N
    u32  size;               // little-endian : taille en octets
}
```

D'après le code, le 1er u32 va dans `entry+0x1C` et le 2e dans `entry+0x18`. `FUN_005e2660`
teste `entry[+0x1C] + entry[+0x18] > next[+0x1C]`, ce qui confirme que le 1er u32 est l'offset
et le 2e la taille. Les fichiers le confirment aussi (0 chevauchement, 0 trou).
Le tampon de nom fait 260 octets (contrôle `0x103`) : `name_len` ≤ 255.
Dans le jeu NoLimits, le plus long nom clair fait 65 caractères.

Le n° du fichier data **n'est pas stocké** dans l'index : il est recalculé à partir du nom encodé (§4).
Si un nom apparaît deux fois, l'entrée chargée en dernier met à jour la précédente (`FUN_005e49b0`).
Le jeu actuel ne contient aucun doublon.

### Clé XOR (index ET contenu) : `SFrame.exe` `0x00dfb1a8`, 256 octets **[OBS]**

Le binaire en contient une copie identique en `0x00dfb0a8`, que `FUN_005d87d0` utilise quand il
traite les données par mots de 32 bits (index `(pos>>2)&0x3F`, ce qui revient au même).

```
77 e8 5e ec b7 4e c1 87 4f e6 f5 3c 1f b3 15 43 6a 49 30 a6 bf 53 a8 35 5b e5 9e 0e 41 ec 22 b8
d4 80 a4 8c ce 65 13 1d 4b 08 5a 6a bb 6f ad 25 b8 dd cc 77 30 74 ac 8c 5a 4a 9a 9b 36 bc 53 0a
3c f8 96 0b 5d aa 28 a9 b2 82 13 6e f1 c1 93 a9 9e 5f 20 cf d4 cc 5b 2e 16 f5 c9 4c b2 1c 57 ee
14 ed f9 72 97 22 1b 4a a4 2e b8 96 ef 4b 3f 8e ab 60 5d 7f 2c b8 ad 43 ad 76 8f 5f 92 e6 4e a7
d4 47 19 6b 69 34 b5 0e 62 6d a4 52 b9 e3 e0 64 43 3d e3 70 f5 90 b3 a2 06 42 02 98 29 50 3f fd
97 58 68 01 8c 1e 0f ef 8b b3 41 44 96 21 a8 da 5e 8b 4a 53 1b fd f5 21 3f f7 ba 68 47 f9 65 df
52 ce e0 de ec ef cd 77 a2 0e bc 38 2f 64 12 8d f0 5c e0 0b 59 d6 2d 99 cd e7 01 15 e0 67 f4 32
35 d4 11 21 c3 de 98 65 ed 54 9d 1c b9 b0 aa a9 0c 8a b4 66 60 e1 ff 2e c8 00 43 a9 67 37 db 9c
```

## 3. Encodage des noms de fichiers (`FUN_005cf720`) **[OBS]**

Les noms sont plats : aucun dossier, mais des espaces sont possibles (`beast_desert wildcat.cob`).
Ils sont passés en minuscules avant l'encodage.
Le nom encodé fait `L + 2` caractères : `préfixe` + corps transformé (L) + `suffixe`.

### Tables

`NAME_ENC` : `SFrame.exe` `0x00dfb3a0`, 128 octets, indexée par un `char` **signé**. Les index 0..31 valent 0.
```
00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00
67 20 00 26 77 2c 6c 4e 58 4f 00 37 2e 25 65 00 38 5f 5d 23 50 31 2d 24 56 5b 00 59 00 5e 00 00
4b 7d 6a 30 40 47 53 29 41 78 79 36 39 45 46 7b 57 62 3d 52 76 74 68 32 34 4d 28 6b 00 6d 61 2b
7e 44 27 43 21 4a 49 64 42 55 60 71 66 70 48 51 33 4c 6e 6f 5a 69 72 73 75 3b 7a 63 00 54 35 00
```

`NAME_DEC` : `SFrame.exe` `0x00dfb320`, 128 octets. C'est l'inverse exact de `NAME_ENC`
(vérifié sur tous les caractères 0x20..0x7F que `NAME_ENC` associe à une valeur non nulle).
Le client ne s'en sert pas pour lire, mais elle est dans le binaire :
```
00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00
21 64 00 33 37 2d 23 62 5a 47 00 5f 25 36 2c 00 43 35 57 70 58 7e 4b 2b 30 4c 00 79 00 52 00 00
44 48 68 63 61 4d 4e 45 6e 66 65 40 71 59 27 29 34 6f 53 46 7d 69 38 50 28 3b 74 39 00 32 3d 31
6a 5e 51 7b 67 2e 6c 20 56 75 42 5b 26 5d 72 73 6d 6b 76 77 55 78 54 24 49 4a 7a 4f 00 41 60 00
```

`CHARSET` : `SFrame.exe` `0x00eaa4c0`, chaîne C de 84 caractères :
```
^&T_Nsd{xo5v`rOYV+,iIU#kCJq8$'~L0P]FeBn-Au(pXHZhwDy2}agWG7K=bQ;SRt)46l@jE%9!c1[3fmMz
```
(hex : `5e26545f4e73647b786f357660724f59562b2c694955236b434a713824277e4c30505d4665426e2d4175287058485a68774479327d61675747374b3d62513b5352742934366c406a4525392163315b33666d4d7a`)

### Encodage (pseudo-code exact)

```
encode(plain):
    s = tolower(plain); L = len(s)
    seed = (Σ (s[i]*17 + 1) + L) mod 32 ; if seed == 0: seed = 32     // reste signé, toujours >0 en ASCII
    seed0 = seed
    for i in 0..L-1:
        o = s[i]; c = o
        repeat seed times: c = NAME_ENC[c]
        s[i] = c
        seed = (o*17 + 1 + seed) mod 32 ; if seed == 0: seed = 32     // o = caractère CLAIR
    prefix = CHARSET[(Σ s[i]) mod 84]                                 // somme des caractères encodés
    i0 = (int)((float)L * 0.66) ; i1 = (int)((float)L * 0.33)
    swap(s[0], s[i0]) ; swap(s[1], s[i1])
    return prefix + s + CHARSET[seed0]
```

### Décodage (inverse)

```
decode(e):
    L = len(e) - 2 ; s = e[1 .. L]
    seed = indexOf(CHARSET, e[L+1])           // 1..32 (les 33 premiers caractères de CHARSET sont distincts)
    swap(s[1], s[i1]) ; swap(s[0], s[i0])     // échanges annulés dans l'ordre inverse
    for i in 0..L-1:
        c = s[i]; repeat seed times: c = NAME_DEC[c]
        s[i] = c
        seed = (c*17 + 1 + seed) mod 32 ; if seed == 0: seed = 32
    return s                                   // le préfixe ne sert qu'au contrôle
```

Vérification : `encode(decode(x)) == x` est vrai pour les **125 282 / 125 282** entrées de `data.000`.

## 4. Choix du fichier `data.00N` (`FUN_005e3620`) **[OBS]**

On hache le **nom encodé** (celui qui est stocké dans l'index) :

```
h: i32 = 0
for b in nom_encodé:
    c = (i8)b                    // char signé
    if 'A' <= c <= 'Z': c += 32
    h = h*31 + c                 // arithmétique 32 bits avec débordement
id = abs(h) % N                  // abs C : (h ^ (h>>31)) - (h>>31) ; reste signé
fichier = "data.00" + (id + 1)   // N = 8 -> data.001 .. data.008
```

`N` vaut `*(this+4)`, le nombre de fichiers de données. Il vaut 8 dans ce client **[DED]** :
avec N = 8, les 125 282 entrées tiennent dans leur fichier, sans chevauchement ni trou, et la
somme des tailles égale exactement la taille de chaque fichier. L'endroit où `this+4` est
initialisé n'a pas été tracé **[INC]**.

L'autre hachage (`FUN_005c3560`, même formule `h*31 + tolower(c)` mais sur un `u32`, sans abs)
ne sert qu'aux seaux de la table de hachage en mémoire. Un extracteur n'en a pas besoin.

## 5. Chiffrement du contenu (`FUN_005ddf20` + `FUN_005d87d0`) **[OBS]**

Quand `+4` (chiffrement) vaut 1, ce qui est le cas pour ce client, chaque fichier est XORé avec
la **même clé de 256 octets** que l'index. L'index de clé est la **position dans la ressource**,
qui démarre à 0 au début de chaque fichier : `plain[i] = raw[i] ^ KEY[i & 0xFF]`.

Seules exceptions (`+5` = 1) : les fichiers dont les **4 derniers caractères** du nom sont l'une
des extensions ci-dessous (table en `0x00dfccb4`, entrées de 8 octets). Ils sont stockés **en clair** :

```
.dds  .tga  .naf  .nx3  .cob  .itc  .itm  .itb  .nfm
```

Tous les autres sont XORés : jpg, png, bmp, wav, lua, cfg, txt, xml, rdb, nui, spr, spt, fx,
jtv, nf*, ttf, etc. Vérifié sur les vrais fichiers :

| Fichier | Stockage | Premiers octets après traitement |
|---|---|---|
| `10.dds` | clair | `44 44 53 20` « DDS  » |
| `1.tga` | clair | type d'image 0x0A (RLE truecolor) |
| `ailes6084168.jpg` | XOR | `ff d8 ff e0 … JFIF` |
| `1.png` | XOR | `89 50 4e 47 0d 0a 1a 0a` |
| `game_panel_image_worldmap_over_all.bmp` | XOR | `42 4d` « BM » |
| `ain_ancientknight_honor.wav` | XOR | `RIFF….WAVE` |
| `creaturestate_script.lua` | XOR | texte Lua lisible jusqu'à la fin |
| `npcinfo.cfg`, `abuse_list.txt`, `company_logo.xml`, `db_aniinfo.rdb`, `bg_asura.jtv` (`BTRF`) | XOR | lisibles |

Le contenu n'est **pas compressé** : la taille de l'index est la taille finale. Quand
`FUN_005ddf20` compare l'extension, `param_1` est le nom tel qu'il a été demandé. Si cette
comparaison (`FUN_00c8f010`) est sensible à la casse, `X.DDS` serait déchiffré par erreur
**[INC]**. Un extracteur peut passer le nom en minuscules (tous les noms de l'index le sont).

## 6. Statistiques (jeu NoLimits actuel) **[OBS]**

- `data.000` : 5 271 103 octets, **125 282 entrées**, aucun doublon, aucun nom illisible.
- Fichier le plus gros : 56 231 468 octets. 4 entrées font 0 octet.

| Fichier | Entrées | Octets référencés = taille du fichier |
|---|---|---|
| data.001 | 15 557 | 838 710 866 |
| data.002 | 15 648 | 744 873 777 |
| data.003 | 15 631 | 836 773 867 |
| data.004 | 15 559 | 820 777 489 |
| data.005 | 15 644 | 734 324 335 |
| data.006 | 15 885 | 789 093 249 |
| data.007 | 15 575 | 799 986 208 |
| data.008 | 15 783 | 882 738 889 |

Les fichiers sont parfaitement compacts : 0 chevauchement, 0 octet perdu.

Répartition par extension :
```
dds:40285 jpg:27519 nx3:22072 naf:20825 cob:4312 tga:4213 png:2898 spt:901 wav:765 nui:485
nfm:136 nfs:122 nfa:121 nfc:109 qpf:108 nfw:86 rdb:69 txt:45 nfl:40 nfp:33 spr:17 jtv:15
pvs:15 nfe:13 fx:9 xml:8 gci:7 lua:7 m4v:6 bmp:5 cfg:5 tml:5 lst:4 tif:4 ttf:4 gc2:3 nfk:3
otf:3 sdb:3 ini:1 ref:1
```

Les 10 premières entrées de l'index :
`game_panel_image_worldmap_over_all.bmp, m002_001.bmp, m003_000.bmp, m003_001.bmp,
waterbump.bmp, npcinfo.cfg, terrainpropinfo.cfg, terrainseamlessworld.cfg,
terraintextureinfo.cfg, water.cfg`. La liste continue avec `ain_*.cob`.

## 7. Notes pour l'extracteur Rust

- Lire `data.000` en entier, faire `buf[i] ^= KEY[i & 0xFF]`, puis parcourir `u8 len | name | u32 off | u32 size`.
- `file = abs_i32(h31_signed_lower(nom_encodé)) % 8 + 1`. Utiliser `wrapping_mul`/`wrapping_add` sur `i32`.
  `i32::MIN.unsigned_abs() % 8 == 0`, ce qui donne le même résultat qu'en C.
- `decode_name` : voir §3. Pour les indices d'échange, faire `((L as f32) as f64 * 0.66) as i32`
  (même résultat que le client pour toutes les longueurs rencontrées).
- Contenu : `seek(off)`, `read(size)`. Si l'extension n'est pas dans la liste d'exclusion, appliquer `buf[i] ^= KEY[i & 0xFF]` en partant de i = 0.
- Pour écrire ou patcher (sens inverse) : utiliser `encode_name`. Le n° de fichier découle du nom
  encodé. L'index se réécrit dans le même format et avec la même clé roulante (`FUN_005e2a80`).

## 8. Inconnu ou non tracé

- **[INC]** L'endroit où `N = *(this+4) = 8` est initialisé (probablement une constante ou un
  comptage des `data.00N`). La valeur 8 est confirmée par les fichiers.
- **[INC]** La comparaison d'extension est-elle sensible à la casse (`FUN_00c8f010`, probablement `strcmp`) ?
- **[INC]** Comportement avec des caractères ≥ 0x80 dans un nom : l'index `char` signé tomberait
  avant la table. Ce cas n'existe pas dans le jeu actuel.

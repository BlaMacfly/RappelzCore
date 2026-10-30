# Interface du client Rappelz : `.spr`, `.nui`, légendes, règles de dessin

Client analysé : `Rappelz NoLimits\SFrame.exe` (v1.3.60, sha256 `27f42fc8…a410d`), Ghidra 12.1.4 via REA.
Le client n'a été ouvert qu'en lecture. Le chemin des sources internes visible dans l'exe est
`source9.5.2\client\game\engine\renderer\kresourcemanager.cpp`. Les classes du moteur d'interface
s'appellent `KUI*` (`KUIGenWnd`, `KUIControlButton`, `KUIControlCaptionButton`…).
Implémentation : crate `shared/rz-ui` (voir § 9).

Légende : **[O]** observé dans le code du client (et, sauf mention, vérifié sur les fichiers) ;
**[D]** déduit des données ou des rendus ; **[I]** inconnu.

---

## 1. Fonctions du client

| Adresse | Rôle |
|---|---|
| `FUN_00b87450` | Initialisation ; charge `ui_frame.spr` (puis `ui_frame_patch.spr`, absent de NoLimits) |
| `FUN_00ac2820` | `LoadSpriteSet` : précharge `01_equip`, `02_item`, `03_skill`, `04_event`, `05_ui`, `06_live` `.spr` (table `0x00e98444`) puis le fichier demandé |
| `FUN_00ac2a60` | Fusion de deux jeux : un nom déjà présent est **ignoré**, sauf préfixe `static_common_rank` (supprimé puis ré-ajouté, `FUN_00ac0bd0`) |
| `FUN_00abfcd0` / `FUN_00ac5140` | `AddResSpriteAni` : dans un même fichier, une animation redéfinie **remplace** la précédente |
| `FUN_00addb80` | Parseur `.spr` texte (découpe CRLF, séparateur `0x00dfb944`, `atoi` `FUN_00c729b0`) |
| `FUN_00ade650` | Création d'une animation (images à l'origine 0,0, échelle 1) |
| `FUN_00ac18f0` | `GetSpriteSet(nom)` : cache, chargement paresseux du `.spr` nommé par la clé `spr` |
| `FUN_00ae5100` | Chargement des textures d'animation : nom **exact** (`FUN_005c23c0`), sinon « Texture Resource Not Found : %s » et animation abandonnée ; empaquetage dans des atlas A8R8G8B8 (dump debug `c:\ui_pool%02d.tga`) |
| `FUN_00ab6410` / `FUN_00ab6420` / `FUN_00ab6470` | Largeur / hauteur d'une image / image n° i (`NULL` hors de [0,N)) |
| `FUN_00b3ab30` | Tables des noms `KSTYLE_*` (`DAT_00edd564`), `KFLAG_*` (`DAT_00edd56c`), `KANCHOR_*` (`DAT_00edd574`) |
| `FUN_00b39f10` / `FUN_00b39c00` | Parseur `.nui` (`KUIWndParseObject`) : mots-clés, puis valeurs ; erreurs « Parsing error at line %d. %s » (`0x00d89c1c`) |
| `FUN_00b3a4f0` | `end` : le contrôle est ajouté au genwnd dans l'ordre du fichier |
| `FUN_00b3ec90` / `FUN_00b3ead0` | Fabrique type → créateur (table de hachage remplie par des initialiseurs statiques `push nom ; push fn`) ; appel de `Create(parseobj)` (vfunc+8) |
| `FUN_00b3c510` | `Create` de base : flag→+0x10, style→+0x14, anchor→+0x18, rect→+0x108..+0x114 (brut), ani→+0xc4, spr→+0xdc, frame→+0xf4, caption→+0x94, info→+0x7c |
| `FUN_009a6d50` | Rendu de base d'un contrôle : images, enfants (ordre de la liste), texture de légende |
| `FUN_009a9480` / `FUN_009a9050` | Découpage générique (1 / 3 / 9 images) |
| `FUN_009aa090` | Constructeur `KUIControlButton` (vtable `0x00d66598`) : 12 sprites (4 × 3) |
| `FUN_009ad1a0` / `FUN_009ad7b0` | Mise en page bouton horizontale / verticale (`KSTYLE_BUTTON_VERTICAL`) |
| `FUN_009aba00` | Rendu bouton : état +0x121c, désactivé (+0x294) → état 3 |
| `FUN_009aaf80` | Bouton activé : retire les `<#rrggbb>` de la légende et préfixe la couleur mémorisée en +0x298 |
| `FUN_009ac840` | `captionbutton` : image de légende centrée |
| `FUN_00b35070` / `FUN_00b35d90` / `FUN_00b376a0` | `KUIGenWnd` : constructeur (9 animations par défaut, table `0x00e98b7c`), Create, Render |
| `FUN_00b38530` / `FUN_00b386d0` | Fond (`window_tile`) / cadre 9-slice (`outframe`) du genwnd |
| `FUN_00b2d640` | Parseur de légendes à balises (table `0x00d891a0..0x00d8926c`) |
| `FUN_009a7bd0` / `FUN_009a0f20` | Rendu du texte dans une texture de la taille du rect ; `KFLAG_SINGLE_LINE` → coupe avec « ... » |
| `FUN_006682e0` | Chaîne n° id (table de chaînes) |
| `FUN_009c8170` / `FUN_009e4330` | Table des 234 noms de fenêtres (`0x00e97688`, indice = n° de fenêtre) → création vfunc+0xd0 |

Créateurs enregistrés (extrait) **[O]** : `button`→`0x009aaa30`, `captionbutton`→`0x009aaa90`,
`KUIControlCaptionCheck`→`0x009aab80`, `KUIControlButtonCheck`→`0x009aac70`, `simplebutton`→`0x009abdc0`,
`static`→`0x00b67110`, `iconstatic`→`0x00b671b0`, `multi_icon`→`0x00b67210`.

---

## 2. `.spr` texte (14 des 17 fichiers)

```
<nom d'animation>\r\n
<N>\r\n
N × {
  <fichier image>\r\n      ex. button_common_new_middle01.png
  <w,h>\r\n                IGNORÉ par le client [O]
  <0>\r\n                  IGNORÉ par le client [O] (toujours 0 dans NoLimits)
}
```

- **[O]** Seule la 1re ligne de chaque triplet est lue. La taille d'une image est sa taille réelle
  après chargement ; « 0,0 » n'a aucun effet (c'était la taille de la 1re image notée par l'outil).
- **[O]** Nom d'image ouvert **tel quel**, sans repli d'extension. Vérifié : les 17 262 images des
  animations référencées par les fenêtres existent sous leur nom exact ; 0 n'existe qu'avec une
  autre extension [O]. (`rz-ui` garde un repli `.dds/.tga/.png/.jpg/.bmp`, jamais utilisé.)
- **[O]** Une image manquante fait abandonner toute l'animation : le contrôle ne dessine rien.
- `N = 0` existe (`New Sprite 39`).

### Ordre de chargement et doublons

Pour une fenêtre `spr = ui_frame.spr;` **[O]** : `01_equip.spr`, `02_item.spr`, `03_skill.spr`,
`04_event.spr`, `05_ui.spr`, `06_live.spr`, puis `ui_frame.spr` (`ui_frame_patch.spr` si présent).
Entre fichiers, la **première** définition gagne (fusion) ; dans un fichier, la **dernière**.
Conséquence vérifiée **[D]** : `button_common_middle_new` est défini dans `05_ui.spr`
(`button_common_new_middle01.png`, 19×30) et dans `ui_frame.spr` (`…middle_new01.tga`, 19×28) ;
c'est la version 05_ui qui s'applique, et les rects des boutons des `.nui` font bien 30 px de haut.
De même `button_common_closewindow` → `*_copy.png` 21×17 = taille des rects.

## 3. `.spr` binaires (`minimap.spr`, `ui_icon.spr`, `worldmap.spr`) [O données / D sens]

Sérialisation à balises des gabarits de **`sprite.tml`** (gabarits DirectX `.x` : `sprite_type`
{1ACDB25A-…}, `animation_type`, `frame_type`, `bound_box`, `track_type`, `library_type` {E1BDE3E7-…}).

```
0x00  "DIFKNIBK"  u32 1  u32 0  u32 1 …            en-tête 0x80 octets
0x80  "KLBK"  u32 taille_des_données  …            en-tête 0x20 octets
0xA0  0x1F GUID(16) u32 nb_membres  membres…       structure racine (sprite_type)
membre :
  0x29 u32                                          dword
  0x2A f32                                          float
  0x33 u32 len  octets[len] (\0 inclus)             chaîne
  0x34 0x29|0x2A  u32 n  u32 0  n×4 octets           tableau de dword / float
  0x34 0x1F GUID  u32 n  u32 len nom[len]  n × (0x1F GUID u32 nb_membres membres…)
                                                     tableau de structures (nom = champ taille)
```

- `sprite_type` = {animation_size, animation_array, library_size, library_array}.
- `animation_type` = {nom, frame_time, frame_size, frame_array, track_size, track_array}.
- `frame_type` (9 membres dans ces fichiers) = delay, image_origin[2], attr[8] (1,1,1,0…),
  attrlock[8], bound_box_size, bound_box_array, library_number, flags, **id de planche** (chaîne,
  ex. `ID_ui_icon.TGA_SUB_464`).
- `library_type` = {library_name (= id), library_path (ex. `ui_icon.TGA`), library_rect x1,y1,x2,y2}.
  Une image d'animation est donc un **sous-rectangle** d'une planche (x2/y2 exclus [D]).
- Lien avec le code client : [I] (non localisé) ; ces fichiers ne sont référencés par aucune
  fenêtre `window_*`.

## 4. `.sdb` et `.tml`

- `.sdb` (`04_event`, `06_live`, `sprite_icon`) : base source de l'outil de sprites,
  `nom / N / N × { chemin complet d'origine / 0 }` (chemins `\rappelz_UI\…`, `\Documents and Settings\…`).
  **Aucune chaîne « .sdb » dans l'exe [O]** : non lu par le client.
- `.tml` : `sprite.tml` décrit les gabarits des `.spr` binaires (§ 3) ; `nx3.tml`, `nobj.tml`,
  `pfx.tml`, `qjtv.tml` concernent la 3D, les effets et les vidéos du lobby. Le client ne
  référence que `QJTV.tml`, `NX3.tml`, `NObj.tml` [O].
- Les noms `ani` introuvables (`botton_lobbycommon_01`, `editbox_lobbycommon_01`,
  `tile_lobbycommon_background`, `background_creatureinfo_name`…) ne sont dans **aucun** fichier
  (ni `.spr`, ni `.sdb`, ni nom de fichier de l'archive) et dans aucune chaîne de l'exe ; les noms
  d'animation ne sont pas transformés par le code [O]. Ils appartiennent aux **62 anciens `.nui`**
  (sans préfixe `window_`), dont 2/768 animations seulement se résolvent : fenêtres obsolètes [D].

---

## 5. `.nui` : grammaire

```
// commentaire (ligne commençant par //)
begin <type>
<clé> = <valeur>;
…
end
```
- **[O]** Mots-clés : `begin`, `end`, `id`, `ani`, `spr`, `frame` (entier), `rect` (4 entiers),
  `pos` (2 entiers), `caption` (chaîne entre guillemets), `info`, `style`, `flag`, `anchor`
  (OU de noms, `|` final toléré), `add_button`, `genwnd`.
- Le 1er bloc est `genwnd` (la fenêtre, `rect = 0,0,largeur,hauteur`) ; les suivants sont ses
  contrôles, **créés et dessinés dans l'ordre du fichier** [O].
- **`rect = x1,y1,x2,y2`** brut, **x2/y2 exclus** : largeur = x2−x1 [O] (calculs de mise en page) ;
  coordonnées relatives au coin haut-gauche de la fenêtre [D] (le décalage parent n'a pas été localisé).
- **`pos = x,y`** sans rect : le client met x2 = y2 = −1 et prend la taille de l'image `frame` [O].
- **`ani`** : nom d'animation ; vide (`ani = ;`) = rien. Composite (vu dans les données [O],
  sens [D]) : barres de défilement `fond/curseur/haut/bas` ou `fond/curseur/début/haut/fin/bas` ;
  onglets `bouton/tab_inventory<#a0a0a0>` (2e partie : groupe d'onglets ? [I]).
- `spr` : jeu de sprites ; toutes les fenêtres `window_*` utilisent `ui_frame.spr`.
- Fichiers en CP949 (commentaires et textes de secours coréens).

### Types de contrôles (nombre dans les 485 fichiers)

`static` 8226, `button` 902, `iconstatic` 878, `simplebutton` 636, `multi_icon` 516, `genwnd` 483,
`number` 242, `check` 207, `quick_effect` 109, `gauge` 88, `edit` 80, `captionbutton` 62,
`v_scroll` 53, `quest_info_mgr` 29, `v_scrollSmallEx` 28, `simple_tab` 26, `combo_box` 25,
`tab_simple_head` 23, `h_slider` 10, `3dstatic` 9, `tab_sheet` 6, `tab_head` 6, `colorbox` 5,
`qjtv_static` 4, `dynamictexture_static` 4, `minimap_static` 3, `detail_tab` 3, `v_scrollEx` 2,
`list^item_string^h` 2, `list^item_icon_string^h` 2, `buttoncheck` 2, `statusbar` 1, `chaos_gauge` 1.

### Styles, drapeaux, ancrages [O] (`FUN_00b3ab30`)

| KSTYLE_ | valeur | KSTYLE_ | valeur |
|---|---|---|---|
| NOCLOSE | 0x2 | MOVE_BY_TITLEBAR | 0x2000 |
| NOMINIMIZE | 0x4 | MOVE_BY_TOPFRAME | 0x4000 |
| NORESIZE | 0x8 | MOVE_BY_ALL | 0x8000 |
| NOTITLE | 0x10 | MOVE_BY_CUSTOM | 0x10000 |
| NOSTATUSBAR | 0x20 | NO_CLIPPING | 0x20000 |
| VSCROLL / HSCROLL | 0x40 / 0x80 | VERTICAL_REPEAT | 0x40000 |
| NOFRAME | 0x100 | HORIZONTAL_REPEAT | 0x80000 |
| NOTOPFRAME | 0x200 | SPR_CAPTION | 0x100000 |
| NOBACKGROUND | 0x400 | ALPHA_SHOW | 0x200000 |
| RESIZE_LEFT / RESIZE_TOP | 0x800 / 0x1000 | | |

Styles propres à un type (réutilisent les bits bas) : BUTTON_LEFTSIDE 2, BUTTON_RIGHTSIDE 4,
BUTTON_CANDRAG 8, BUTTON_VERTICAL 0x10, CHECK_CAPTION_RIGHT 2, SB_NOLEFTSIDE 2, SB_NORIGHTSIDE 4,
TAB_VERTICAL 2, NUMBER_LEFT/CENTER/RIGHT 2/4/8, GAUGE_WITH_GRADUATION 2, MSG_VERTICAL_REPEAT 1,
MSG_HORIZONTAL_REPEAT 2. KFLAG_ : NO_GET_MESSAGE 2, GET_PASS_MESSAGE 4, CAN_DRAG 8,
SINGLE_LINE 0x10, NO_GET_FOCUS 0x20. KANCHOR_ : LEFT 2, RIGHT 4, TOP 8, BOTTOM 0x10.

---

## 6. Règles de dessin

W×H = taille réelle de l'**image 0** de l'animation. Les morceaux étirés le sont par mise à
l'échelle (pas de répétition), sauf mention.

| Contrôle / nb d'images | Règle | Preuve |
|---|---|---|
| tout contrôle en mode `pos` | image `frame` (0) à sa taille en (x, y) | [O] |
| générique, 1 image | étirée sur tout le rect ; `VERTICAL_REPEAT`/`HORIZONTAL_REPEAT` : tuiles de H/W, la dernière coupée | [O] genwnd, [D] static |
| générique, 3 images | horizontal : gauche [x1, x1+W), milieu [x1+W, x2−W) étiré, droite [x2−W, x2) ; **hauteur H** (pas celle du rect). Vertical (haut/milieu/bas, largeur du rect) si `style & 2` pour un static ; toujours vertical pour `v_scroll*` | [O] static, [D] scroll |
| générique, 9 images | 9-slice, indice = ligne×3 + colonne ; coins W×H, bords et centre étirés (les coins bas utilisent aussi W×H de l'image 0) | [O] |
| générique, autre N | rien (« Error!!! : Frame », `0x00d66418`) | [O] |
| `button` 12 images | indice = **état×3 + morceau** (gauche, milieu, droite). Gauche W₀×H₀ en x1, milieu de largeur w−W₀−W₂ étiré, droite en x2−W₂ ; hauteur = H de l'image ; `style&2` sans gauche, `&4` sans droite, `&0x10` vertical | [O] |
| états | 0 normal, 1 survol, 2 appuyé, 3 désactivé | 3 [O], 0-2 [D] (rendus cohérents : 1 = rouge) |
| `button`/`buttoncheck` 24 images | 2 jeux de 12 : 0-11 coché/actif (`…stretch`), 12-23 non coché (`…off_stretch`) | [D] noms |
| `simplebutton` 4 images | une image par état, étirée sur le rect (images = taille du rect) | [D] |
| `check` 4 images | indice = coché×2 + survol (`checkbox_01,01,02,02`) | [D] |
| `captionbutton` | fond = `ani` comme `button` ; `caption` = **nom d'animation** (1 image par état) centrée : x = x1 + max(0,(w−Wimg)/2) | [O] |
| `gauge` 9 images | 3 couches × 3 morceaux (`_01_xx`, `_02_xx`, `_03_xx`) : couche 3 (jpg opaque) = fond sur tout le rect, couche 1 = remplissage, largeur × valeur ; couche 2 non utilisée | [D] rendu |
| barre de défilement composite | fond (3 images verticales), flèches haut/bas à leur taille centrées, curseur sous la flèche haute | [D] |
| `genwnd` | fond `window_tile` (sauf NOBACKGROUND) → contrôles → `window_topframe` en (x1, y1+2) sur la largeur (sauf NOTOPFRAME) → boutons → cadre `outframe` 9-slice **par-dessus** (sauf NOFRAME). Ces animations n'existent pas dans NoLimits : les fenêtres `window_*` dessinent leur cadre avec des `static` | [O] |
| légende texte | rendue dans une texture de la taille du rect (coupée au rect) ; `KFLAG_SINGLE_LINE` → une ligne + « ... » | [O] |

Vérification visuelle (PNG ci-dessous) : bords 9-slice nets, coins non déformés, milieux étirés,
boutons 3-slice à toutes les largeurs (108, 140, 180 px), état survol, jauges partielles.

---

## 7. Légendes, chaînes et polices

- Balises [O] (`FUN_00b2d640`, comparaison en majuscules) : `FONT:`, `SIZE:`, `P`, `$L`, `B` `/B`,
  `U` `/U`, `STRIKE`, `INV`, `SHD`, `%%`, `LEFT` `TOP` `RIGHT` `BOTTOM` `HCENTER` `VCENTER`, `OUT`,
  `GLOW`, `SHADOW`, `2XGLOW` (et leurs `/…`), `OFFSETY:`, `OFFSET:`, `CARET`, `WB`, `^` ;
  émoticônes en ligne. Vus dans les données mais absents de la table : `<#rrggbb>` (couleur du
  texte), `<@rrggbb>` (couleur du contour [D]), `<br>` (saut de ligne [D]), `<center>`, `<bold>`.
- **`<$N:texte>`** [O] : chaîne n° N de la table de chaînes (`FUN_006682e0`, message « ID %d … »
  si absente) = colonne `code` de `db_string(<codepage>).rdb`. Le client lit `Resource\` en
  priorité : la copie de l'archive est tronquée, celle de `Resource\db_string(windows-1252).rdb`
  (150 596 chaînes) donne le français (« ID », « Mot de passe », « Connexion »…). `texte` = secours
  (coréen CP949).
- Les chaînes traduites contiennent souvent leurs propres balises (ex. 6115 =
  `<hcenter><vcenter><size:10><#242424>Connexion`) qui s'appliquent après celles du `.nui`.
  Pour un bouton activé, le client retire les `<#rrggbb>` de la légende et préfixe la couleur
  mémorisée en +0x298 (`FUN_009aaf80`) [O] ; valeur de +0x298 [I].
- Police : `localeinfo.ini` → `FONT_DEFAULT = FONT_01 = FONT_02 = FONT_03 = tahoma.ttf`
  (`<font:font_default|font_01…>`), hinting 0 ; police GDI nommée « Default » [O]. Unité de
  `<size:N>` : points supposés (N×4/3 px) [D].
- Taille et couleur par défaut sans balise : [I] (rz-ui : 10, blanc).

---

## 8. Fenêtres par écran

Table des 234 noms de fenêtres du code : `0x00e97688` [O]. `localeinfo.ini` nomme aussi
`window_lobby_login_logo.nui`, `window_lobby_login_logo_subtitle.nui`,
`window_lobby_copyright_logo.nui`, `window_lobby_published_logo.nui`, `window_lobby_loading_*`.
Seules les fenêtres **`window_*`** (423 fichiers, 99,9 % d'animations résolues) sont actuelles ;
`login.nui`, `lobby_*.nui`, `mainframe.nui`… sont obsolètes.

| Écran | Fenêtres | Contrôles importants |
|---|---|---|
| Connexion | `window_lobby_login` (376×184), `window_lobby_login_logo`, `window_lobby_login_sub`, `window_lobby_login_text` | `accountedit`, `passwordedit` (edit), `idok` (« Connexion »), `button_quitgame` ; sub : `button_credits`, `button_intro`, `button_id_pw_find`, `button_member` |
| Serveurs | `window_lobby_server_list` (296×463), `window_lobby_server_sub`, `_server_notice`, `_server_screenshot` | lignes `server_list_text_NN` sur fonds `common_selectclickbackground_NN`, surbrillance `server_select_sample_NN`, état `static_condition_check_NN` (`static_condition_check_020…100`), `severform_scroll`, `button_choice` (6109), `button_back` (6104) ; sub : `button_notice`, `button_screenshot` |
| Personnages | `window_lobby_character_list` (296×500), `window_lobby_character_admin`, `window_lobby_character_name_tag`, `window_lobby_character_choice_text` | 8 lignes N=00..07 : `character_select_0N` (surbrillance), `static_job_icon0N`, `select_jobname_text0N`, `select_lv_number0N`, `select_jlv_number0N`, `select_name_text0N` ; `entergamebtn`, `selectserverbtn` ; admin : `button_character_create`, `button_character_delete` |
| Création | `window_lobby_character_create(_2)`, `_create01(_2)`, `_character_customizing`, `window_lobby_race_select`, `window_lobby_clanselect`, `window_lobby_clan_*` | (non détaillé) |
| Barre de vie | `window_main_player` (238×78) | jauges `mainframe_hpbar`, `mainframe_mpbar`, `mainframe_staminabar`, `mainframe_expbar`, `icon_job` |
| Raccourcis | `window_main_quickslot` (865×167) | `icon_slot00…` (multi_icon) |
| Menu / barre des tâches | `window_main_menu`, `window_main_menu_sub`, `window_main_taskbar` (1024×24) | `button_mainmenu`… |
| Chat | `window_main_chat` (434×149), `window_main_chat_sub` | `chat_edit`, `chat_tab`, `chat_scroll`, `chat_line00…`, `button_set`, `button_Type` |
| Minicarte | `window_main_minimap` (256×297), `window_fulldown_minimap` | `zoomin`, `zoomout`, `map_zoomin`, `map_zoomout` |
| Inventaire | `window_main_inventory_bag` (477×541), `window_main_inventory_equip` (342×485) | `tab_inventory`, `button_tab_01…10`, `icon_item00`, `scrollbar_list_01`, `edit_search` ; équipement `icon_helm`, `icon_armor`, `icon_weapon`… |
| Fiche personnage | `window_player_character_info_main` (+ `_sub_basic`, `_sub_added`) | `gauge_hp/mp/exp`, `tab_basic_01`, `tab_added_01` |
| Dialogue PNJ | `window_npc_talk` (900×760) | `npc_job_name`, `npc_name`, `npc_story`, `npc_face` (3dstatic), `button_prev`, `npc_button_next`, `button_hide`, `npc_select`, `outframe_choice` |
| Cible | `window_main_target` (+ `_target_player/_npc/_creature_monster`) | |

Animations encore introuvables dans les `window_*` : `common_panel_outframe_copper_gold2`
(minicarte), `static_ending_pubilshed`, et 2 images (`common_layout_titanium_dotline_dungeon_target.png`,
`game_panel_titanium_arena_status_line_gray_a.png`).

---

## 9. Implémentation `shared/rz-ui`

- `SpriteSet::load_ui_set(read, "ui_frame.spr")` (ordre et fusion du § 2), `merge_spr(_bytes)`,
  `animation(nom) -> Option<&Animation { name, frames: Vec<Frame { image, spr_size, spr_extra,
  sub, file, w, h }>, source }>`, `resolve_document(&doc, lookup)` / `resolve_animation` ;
  `image_candidates(nom)` ; `parse_spr`, `parse_spr_bytes` (texte ou `DIFKNIBK`), `parse_sdb`,
  `kbin::parse_tree`.
- `parse_nui(&str) -> Result<NuiDocument, UiError>` ; `NuiControl { kind, type_name, id, rect,
  pos, ani, spr, caption, frame, style, flags, anchor, info, unknown_bits, attrs, line }`,
  `ani_parts()`, `main_ani()` ; `decode_text` (CP949).
- `parse_caption(&str) -> Caption` (`runs`, `halign`, `valign`), `Caption::resolve(|id| …)`.
- `layout(&doc, &sprites, origin, &UiState) -> Vec<DrawCmd>` avec
  `DrawCmd::Image(ImageCmd { image, src, dst, control })` / `DrawCmd::Text(TextCmd { caption, rect,
  single_line, control })` ; `UiState` : état par défaut et par contrôle, cochés, masqués, jauges,
  légendes remplacées.
- Tests : `cargo test -p rz-ui` (17 `.spr` dont 3 binaires, 15 915 animations ; 485 `.nui` ;
  résolution `window_*` 3871/3873 = 99,9 %, anciens `.nui` 2/768).
- Rendu : `cargo run -p rz-ui --release --example render_nui -- "Rappelz NoLimits" out.png
  [--canvas 1024x768] [--hover id] [--hide a,b*] [--set id=texte] [--gauge id=0.5] [--zoom 3]
  window_lobby_login.nui@324,400 …`. PNG de vérification dans `client-data/ui-re/` :
  `screen_login.png`, `screen_server_list.png`, `screen_character_select.png`,
  `screen_mainframe.png`, `screen_inventory_npc.png`, `zoom_player.png`.

## 10. Inconnu / à faire

- Couleur de légende des boutons (+0x298) et couleurs par état ; taille/couleur de texte par défaut.
- Rendu propre de `simplebutton`, `check`, `edit`, `gauge`, `tab_*`, `combo_box`, `h_slider`,
  `number` (police d'image ? toutes leurs `ani` manquent), `multi_icon`, `quick_effect`.
- Sens de la 2e partie des `ani` d'onglets, de la couche 2 des jauges, de `KSTYLE_SPR_CAPTION`.
- Décalage parent des rects (relatif au genwnd supposé), positions écran des fenêtres (code).
- Quels contrôles le code masque/affiche au départ (surbrillances, lignes vides, exemples).
- Lecture des `.spr` binaires par le client (fonction non localisée).

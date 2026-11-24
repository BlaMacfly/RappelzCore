# Protocole entrepôt et échange — client Rappelz NoLimits (SFrame.exe)

Analyse **statique** (Ghidra 12.1.4 via REA) de `Rappelz NoLimits\SFrame.exe`, PE x86,
SHA-256 `27f42fc8f6b08df8c239640b938d601aeaebb7b3db0c89fc47755325a98a410d`. SFrame.exe n'a jamais été exécuté.
Complète `network_protocol.md` §9 et reprend les conventions de `quest_protocol.md`.

Conventions :
- **Offsets = offsets paquet** (en-tête TS_MESSAGE de 7 octets inclus : `u32 size, u16 id, u8 checksum`).
- Objets internes `SMSG_*` / `SIMSG_*` : « kind » à objet+4, champs utiles à partir de +0x13 (`m+xx`).
- `[O]` = lu dans le code (adresse citée) ; `[D]` = déduit (nom, sens, ou enchaînement non observé directement).
- `item+xx` = offset dans TS_ITEM_INFO (331 o., `network_protocol.md` §9.2).
- `mgr` = objet du gestionnaire jeu FUN_006d2e60 (`this`) ; `pinfo` = objet joueur local `mgr+0x5c`
  (handle du joueur en `pinfo+0x204`, partenaire d'échange en `pinfo+0x20c`).

Evidence IDs REA principaux :

| Contenu | Evidence ID |
|---|---|
| Lot : FUN_006ca220 (envoi 212), FUN_006cdbf0 (218), FUN_006cef50 (219), FUN_006ddc00 (demande d'échange), FUN_00718fa0 (STradeMgr::ProcTradeMsg → 280), FUN_008d5fd0 (« select_member_request_trade ») | `ev_59c823bc34431da489464f4088039e63b1eb3b0fbbce3566e167fd8435453beb` |
| Lot : FUN_006cd320 (TS_SC_INVENTORY), FUN_006d0bb0 (consommateur SMSG_TRADE), FUN_00704ca0 (ChangeState), FUN_00718b60 (copie ITEM_INFO), FUN_006e1310 (envoi) | `ev_40f814c25010cc8f3a3b2093b419688454bf658f804d0e003e332155f1c10bdb` |
| Lot : FUN_0070e320 / 0070e470 / 0070e560 / 0070e6c0 (liste de l'entrepôt) | `ev_6f361fb0cd7241e18642556cbef1de928d427d9b9172c961f7edcf3c4eff4611` |
| FUN_00798670 (fabrique SMSG_CHANGE_STORAGE, kind 0x1d) / xref de sa vtable | `ev_6c8b606e637d52c09dffd39b95027efc016f8dfe3ececced124f4a17f27c45a0` / `ev_e8fb47fe9101d860b9c03f942188c8d9cefff0f141fccb9bf2cb74b5e2ea2f7e` |
| FUN_00933ff0 (fenêtre entrepôt, PumpUpMessage) | `ev_60c28f4dbb4b2576ac560b7857b9806e4d09680bb7a985bf3e397840c70d0945` |
| Lot : FUN_0077ef70 (dépôt/retrait d'objet), FUN_00931d60 (affichage/masquage fenêtre entrepôt) | `ev_6623053e81895bfcbc33eaffc53de67af060d325eaccc1468c80aac045703486` |
| FUN_006cdd70 (récepteur TS_SC_PROPERTY du joueur) | `ev_cb5ffd7cef0665789aab6491d2831bbc895fa51622e3ddeea97a2a96afeb3c3f` |
| FUN_006cf6b0 (TS_SC_RESULT côté jeu) | `ev_8b911f3535cc40690039a9a11c75640127c75fa175aa126abe43e21628ff9d93` |
| Lot : FUN_0092d560 (SUITradeWnd::PumpUpMessage), FUN_0092e2f0 (affichage de l'or) | `ev_d1662596f1e770228823eb6457f6de0e7b5dcca94d0e4a59c12b6cb83f046c91` |
| Lot : FUN_0092c4a0 (fenêtre d'échange, réception), FUN_0092df30 (mode 10) | `ev_2490dbf22076ad00a3225885df1c73e0ece081a0188e00b64282ab7184b7d6b4` |
| Lot : FUN_0092dd70 (mode 5), FUN_0092ec30, FUN_0092ed20, FUN_007959f0 (refus automatique) | `ev_7002ec1d028bf00f2571a6573790845b26e27a4ae7c1a59586c805073a76c68f` |
| Lot : FUN_00790790 (accepter/refuser), FUN_0092de50 (mode 11), FUN_0092e0f0 | `ev_8f8a5b3b5a06e6c14994bfb219b4300a56fe254b505175ee8f4247b6199f90b3` |
| FUN_00799dc0 (mode 7) / appelants du ctor SMSG_TRADE FUN_007766e0 | `ev_a89acc8a901c1ea00e8ad2731e773b88f8284c96d21187bdc796b9333548b4b3` / `ev_2f1b4f600c25a4457398fcf380c9b7da0a5a3b0405a0e4b0fde360b0c1672c65` |
| Octets 0x719078 (cible de FUN_00718b60 = paquet+0xc) / 0xe9694c (limite 10 objets) | `ev_0404057ad636b9b34837fe166f099c0bb0a8bc4f354716076ddd5b8bc776dae8` / `ev_0fbc74f0983cf763f80a385db0ded7c6d3e1e99a498bb0eb19bf164fa6814351` |
| Chaînes « trade » / « storage » | `ev_433041d886f77b4c80cd9f605dad38f0f0004b83c8a88b7683ab548d7d865d7b` / `ev_1b3b87e2794c5745645a2f1bc63fa6954926ac9e2cfe073697d9fcfb5f096ba1` |
| xref « storage_gold » (table de propriétés FUN_00a1f9f0) / xref « open_storage() » (fenêtre PNJ 0x7e6394) | `ev_ebfa44c2fe75f6bec468b96b7ee2a8ff1456cc9c463830231839dc8d3f3d21fe` / `ev_e000d20431a6499d2b10ae86b91efcbd8f27c4228d2333b18ea52fdda5679ef3` |
| FUN_006d2e60 (gestionnaire jeu, cases 0x19 / 0x1c / 0x1d / 0x20 / 0x47d / 0x491) | `ev_fcbea5235e2a5efd24c35551e8aef307f4d76cbd22393a2b6ddcc8f4c383e1b3` (cf. quest_protocol.md) |

Le parseur S→C FUN_00a2fed0 (cases 0xcf, 0xd3, 0x118, 0x3e9) provient d'une extraction antérieure de la même
session ; les constructions C→S ont été retrouvées par le balayage `mov eax, id` / `mov word [x+4], ax`
(adaptation de `client-data/proto-re/scan2b.js`) : 212 → 0x6ca230, 0x6d4314 ; 218 → 0x6cdbf9, 0x6d5da8 ;
219 → 0x6cef59, 0x6d61da ; 280 → 0x6ddcb2, 0x718fc0, 0x8d609b. Aucune construction C→S de 211, 213, 1001, 3003.

## 1. Vue d'ensemble

| Paquet | ID | Dir | Taille | Corps (offsets paquet) | Fonction |
|---|---|---|---|---|---|
| TS_SC_OPEN_STORAGE | 211 | S→C | 11 | +7 u32 (sens [D], cf. §2.1) | parseur case 0xd3 → SMSG_OPEN_STORAGE kind 0x1c ; FUN_006d2e60 case 0x1c [O] |
| TS_CS_STORAGE | 212 | C→S | **20** | +7 u32 item_handle ; +0xb u8 mode ; +0xc i64 count | FUN_006d2e60 case 0x1d (SMSG_CHANGE_STORAGE) / FUN_006ca220 [O] |
| TS_SC_INVENTORY (objets d'entrepôt) | 207 | S→C | 9+331n | identique à l'inventaire, **`item+0x141` wear_position = −2** | FUN_006cd320 → liste entrepôt FUN_0070e320 [O] |
| TS_SC_PROPERTY `storage_gold` | 507 | S→C | 36 | +7 handle joueur ; +0xb is_number=1 ; +0xc name=`storage_gold` ; +0x1c i64 valeur | FUN_006cdd70 index 0x27 → `pinfo+0x94` [O] |
| TS_SC_GOLD_UPDATE | 1001 | S→C | 19 | +7 i64 or de l'inventaire ; +0xf u32 (non lu par FUN_006d2e60) | kind 0x20 → `pinfo->vtbl+4(i64)` [O] |
| TS_CS_CHANGE_ITEM_POSITION | 218 | C→S | 16 | +7 u8 is_storage ; +8 u32 handle1 ; +0xc u32 handle2 | FUN_006d2e60 case 0x47d / FUN_006cdbf0 [O] |
| TS_CS_ARRANGE_ITEM | 219 | C→S | 8 | +7 u8 is_storage | FUN_006d2e60 case 0x491 / FUN_006cef50 [O] |
| TS_TRADE | 280 | C→S et S→C | **343** | +7 u32 handle ; +0xb u8 mode ; +0xc TS_ITEM_INFO (331 o.) | parseur case 0x118 → SMSG_TRADE kind 0x19 ; envoi FUN_00718fa0 [O] |
| TS_CS_DIALOG (`open_storage()`) | 3001 | C→S | 9+n | trigger choisi dans le menu PNJ | fenêtre PNJ FUN_007e60f0 [O] |

## 2. Entrepôt

### 2.1 Ouverture
- **Déclencheur côté client** [O] : dans la fenêtre PNJ (FUN_007e60f0, 0x7e6390), si le trigger du menu commence
  par `open_storage()` (comparaison de préfixe sur 14 caractères, FUN_005f5130), le client vérifie d'abord que
  les fenêtres 0x45, 0x46, 0x1f et 0x2d ne sont pas ouvertes ; sinon le trigger part normalement dans
  **TS_CS_DIALOG (3001)**. Le menu PNJ (TS_SC_DIALOG 3000) doit donc proposer un trigger `open_storage()` [D :
  la chaîne exacte attendue par le serveur est libre, c'est le préfixe qui conditionne le contrôle client].
- **TS_SC_OPEN_STORAGE (211, 11 o.)** [O] : le parseur ne lit qu'un u32 en +7 (→ `m+0x13`). FUN_006d2e60 case 0x1c :
  `mgr+0x90 = 2` (état « entrepôt »), `[mgr+0x54]+0x24 = 1`, `[mgr+0x54]+0x28 = u32(+7)`, puis
  `FUN_00704ca0(1)` (SPlayerInfoMgr::ChangeState, état 1). Le sens du u32 n'est pas établi [D : handle du PNJ
  probable ; aucun lecteur de `[mgr+0x54]+0x28` retrouvé].
- TS_SC_SHOW_WINDOW (3003) n'intervient pas : il est parsé par FUN_00a2a4a0 (non analysé), l'ouverture de
  l'entrepôt passe par l'id dédié 211 [O pour 211 ; 3003 non suivi].
- Une seconde fabrique de SMSG_OPEN_STORAGE existe (FUN_00a285b0, sans appelant direct résolu) [O, usage inconnu].

### 2.2 Contenu de l'entrepôt : TS_SC_INVENTORY (207) avec wear_position = −2 [O]
FUN_006cd320 (kind 0x16) traite chaque TS_ITEM_INFO :
- `item+0x141 == -2` → **FUN_0070e320** : ajout/mise à jour dans la liste de l'entrepôt (vecteur d'objets de
  0x170 o. ; si un objet de même handle existe il est mis à jour, sinon créé) ; recherche ultérieure par
  FUN_0070e560 (utilisée par la fenêtre entrepôt) ;
- sinon → FUN_006e3640 (inventaire), comme décrit dans `network_protocol.md` §9.4.
- Le tri/rafraîchissement final dépend du **premier** objet du paquet : si `items[0].wear_position == -2` →
  FUN_0070e6c0 (tri de l'entrepôt), sinon FUN_006e7400 (inventaire). **Ne pas mélanger** objets d'entrepôt et
  d'inventaire dans un même TS_SC_INVENTORY [D, conséquence du test sur l'objet 0].
- Aucune limite de capacité dans la liste cliente (vecteur dynamique) [O] ; la fenêtre affiche `+0x49c`
  emplacements par page avec défilement et des onglets de catégories `tab_warehouse` 0..8 (filtre local) [O].
  La capacité réelle est donc **décidée par le serveur** [D].
- TS_SC_DESTROY_ITEM (254, kind 0x17) appelle **FUN_0070e470** (retrait de la liste entrepôt par handle) puis
  FUN_006e4280 (inventaire) ; TS_SC_UPDATE_ITEM_COUNT (255, kind 0x18) appelle FUN_0070e6f0 (entrepôt) puis
  FUN_006e74d0 (inventaire) [O]. Ces deux paquets agissent donc sur l'entrepôt comme sur l'inventaire :
  **les handles doivent être uniques sur l'ensemble inventaire + entrepôt** [D].

### 2.3 TS_CS_STORAGE (212, 20 o.) [O]
Corps : `+7 u32 item_handle ; +0xb u8 mode ; +0xc i64 count` (FUN_006d2e60 case 0x1d copie `m+0x13`, `m+0x1f`,
`m+0x17..0x1e` ; doublon FUN_006ca220). Construit via FUN_00798670(handle, count_lo, count_hi, mode).

| mode | Sens | Émetteur (adresse d'appel de FUN_00798670) | handle / count |
|---|---|---|---|
| 0 | inventaire → entrepôt (dépôt d'objet) [O] | FUN_0077ef70 (objet trouvé dans l'inventaire), 0x77f0cf ; fenêtre inventaire 0x86aa5f / 0x86aa8c | handle d'inventaire ; count = 1 si non empilable, saisie numérique sinon, 10 (maj-clic) ou pile entière (ctrl-clic) [O] |
| 1 | entrepôt → inventaire (retrait d'objet) [O] | FUN_0077ef70 (objet trouvé dans l'entrepôt), 0x77f207 / 0x77f29f ; 0x867cd3 / 0x867cfa | handle d'entrepôt ; même logique de quantité [O] |
| 2 | dépôt d'or (bouton `storage_button_receipts`) [O] | fenêtre entrepôt : opération 2, saisie bornée à l'or de l'inventaire (`pinfo+0x58/0x5c`), envoi 0x9330f6 / 0x933236 | count = montant ; handle = valeur résiduelle de l'opération précédente (non remise à 0) [O] — à ignorer |
| 3 | retrait d'or (bouton `storage_button_payingout`) [O] | idem, saisie bornée à `fenêtre+0x490/0x494` [D : or de l'entrepôt] | count = montant ; handle à ignorer |
| 4 | fermeture de l'entrepôt [O] | FUN_00931d60 quand la fenêtre est masquée (bouton `button_close` → masque la fenêtre 0x38) | handle 0, count 0 |

Effets locaux [O] : à l'envoi du mode 4, FUN_006d2e60 remet `mgr+0x90 = 0`, `[mgr+0x54]+0x24 = 0` et appelle
`FUN_00704ca0(-1)`. Après un dépôt d'objet équipé (`+0x149 != -1` dans l'objet client), FUN_0077ef70 appelle en
plus FUN_0079b9a0 [D : déséquipement local].

### 2.4 Or de l'entrepôt [O sauf mention]
- Il n'est **pas** dans TS_SC_GOLD_UPDATE : ce paquet (19 o.) porte `+7 i64` = or de l'inventaire (passé à
  `pinfo->vtbl+4`, même setter que la propriété « gold » index 7) ; le u32 en +0xf est copié en `m+0x1b` mais
  ignoré par FUN_006d2e60 [O] (chaos [D]).
- L'or de l'entrepôt passe par **TS_SC_PROPERTY (507)** nommé `storage_gold` : la table de noms (FUN_00a1f9f0)
  l'associe à l'index **0x27** ; le récepteur FUN_006cdd70, si `+7 == handle du joueur`, convertit la chaîne
  `%I64d` avec FUN_00c729b0 et la range en `pinfo+0x94` (u32) [O]. FUN_00c729b0 est l'entier 32 bits, l'or
  (index 7) utilisant FUN_00c7264b 64 bits [D par contraste] : rester sous 2^31.
- Le lien `pinfo+0x94` → `fenêtre+0x490/0x494` (borne du retrait) n'a pas été suivi [D].

### 2.5 Déplacement / rangement dans l'entrepôt [O]
- **TS_CS_CHANGE_ITEM_POSITION (218, 16 o.)** : `+7 u8 is_storage ; +8 u32 handle1 ; +0xc u32 handle2`.
  SIMSG kind 0x47d : `m+0x13` octet, `m+0x14`, `m+0x18`. Les constructions de la fenêtre entrepôt (0x9326f4,
  0x935427) écrivent **is_storage = 1**, celles de l'inventaire (0x861ac7, 0x867a8c, 0x86e997) **0** [O].
  handle1 = objet déplacé, handle2 = objet de l'emplacement cible [D].
- **TS_CS_ARRANGE_ITEM (219, 8 o.)** : `+7 u8 is_storage`. Bouton `button_arrange` de l'entrepôt (0x934176,
  0x936b1a) → 1 ; inventaire (0x86c6af, 0x87041a) → 0 [O]. Réponse attendue : TS_SC_INVENTORY complet de la
  zone concernée [D].

### 2.6 Résultats
FUN_006cf6b0 (TS_SC_RESULT côté jeu, `m+0x13` = request_id u16, `m+0x15` = result u16) ne traite que 0xcc, 0xfd,
0x102, 400, 800, 0x321, 0x109a et 0x2715 (ce dernier ouvre `/cstorage`, entrepôt payant) [O] : **aucun
traitement spécifique** pour 212, 218, 219 ou 280. Les codes `RESULT_NOT_ACTABLE_WHILE_USING_STORAGE` et
`RESULT_TARGET_IS_USING_STORAGE` existent dans la table de chaînes [O] ; leur affichage générique n'a pas été
suivi. Un refus côté serveur peut se contenter de ne rien renvoyer ou d'un TS_SC_RESULT générique [D].

## 3. Échange entre joueurs — TS_TRADE (280)

### 3.1 Format (identique dans les deux sens) [O]
| Off. | Type | Champ |
|---|---|---|
| +0x00 | en-tête | size = **343**, id = 280 |
| +0x07 | u32 | handle : C→S = **partenaire** (`pinfo+0x20c`, ou cible pour la demande) ; S→C = joueur **concerné** par l'action |
| +0x0b | u8 | mode (0..11) |
| +0x0c | TS_ITEM_INFO (331 o.) | renseigné seulement pour les modes 5, 6, 10, 11 ; zéros sinon |

- Parseur (case 0x118) : SMSG_TRADE de 0x181 o., `m+0x17 = u32(+7)`, `m+0x35 = mode`, `m+0x36 = copie des 331 o.`,
  `m+0x34 = 1`, `m+0x13 = 0` (= « reçu du serveur ») ; une chaîne `m+0x1b` (nom) est initialisée vide [O].
- Envoi : FUN_00718fa0 (STradeMgr::ProcTradeMsg) n'est appelé que si `m+0x13 != 0` (message produit par l'IHM)
  et pour mode ≤ 11 ; il recopie handle et mode et, pour 5/6/10/11, l'ITEM_INFO `m+0x36` en paquet+0xc
  (FUN_00718b60 avec ECX = paquet+0xc, octets 0x719078) [O].
- Le consommateur « monde » FUN_006b2130 (case 0x19) remplace `m+0x1b` par le nom de l'objet de handle `m+0x17`
  s'il est connu [O] : **le joueur concerné doit être visible (TS_SC_ENTER) pour que son nom s'affiche**.

### 3.2 Champs C→S par mode [O]
| mode | Nom [D] | Déclencheur client | handle (+7) | ITEM_INFO (+0xc) |
|---|---|---|---|---|
| 0 | REQUEST | FUN_006ddc00 (action « échanger » sur la cible ; refus local si cible = soi) ; FUN_008d5fd0 (menu de groupe `select_member_request_trade`) ; 0x79b311 (handle `pinfo+0x208`) | joueur visé | — |
| 1 | ACCEPT | FUN_00790790(1) (réponse oui à la boîte `msgboxTradeJoinCheck`) | demandeur (`pinfo+0x20c`) | — |
| 3 | CANCEL | boutons `button_close` / `trade_selfcancel` ; FUN_00704ca0 quand l'état « échange » est quitté | partenaire | — |
| 4 | REJECT | FUN_00790790(0) (réponse non) ; FUN_007959f0 (refus automatique si l'option d'échange est désactivée) | demandeur | — |
| 5 | ADD_ITEM | FUN_0092dd70 : glisser un objet d'inventaire dans la fenêtre (SIMSG 0x497, saisie de quantité si besoin) | partenaire | `+0x00` handle, `+0x08` i64 (champ client +0x10, uid [D]), `+0x10` i64 count |
| 6 | SET_GOLD | bouton `trade_rpinput_button01` → saisie (bornée à l'or de l'inventaire, > 0) | partenaire | `+0x10` i64 = **montant d'or** ; reste à zéro |
| 7 | FREEZE | FUN_00799dc0 : bouton `trade_selfok` → boîte 0x127 (`msgboxTradeFreezeCheck` [D]) → oui | partenaire | — |
| 8 | CONFIRM | bouton `trade_finalconfirm` | partenaire | — |
| 9 | PROCESS | **automatique** : quand le client a reçu un mode 8 pour lui **et** pour le partenaire (FUN_0092c4a0 case 8, FUN_0092e0f0) | partenaire | — |
| 10 | REMOVE_ITEM | FUN_0092df30 : retrait d'un de ses objets de la fenêtre (quantité saisie ou pile entière) | partenaire | handle, `+0x08`, `+0x10` count retiré |
| 11 | MODIFY_COUNT | FUN_0092de50 (aucun appelant direct résolu) | partenaire | handle, `+0x08`, `+0x10` nouveau count |

Le mode 2 n'est jamais produit par l'IHM [O : aucun ctor SMSG_TRADE avec mode 2 parmi les 12 appels de
FUN_007766e0].

### 3.3 Champs S→C lus et effets [O sauf mention]
Les messages reçus passent par FUN_006d0bb0 (gestionnaire jeu) puis par la fenêtre FUN_0092c4a0 ; seuls `+7`,
`+0xb` et, pour 5/6/10/11, l'ITEM_INFO sont lus.

| mode | Lecture / effet côté client |
|---|---|
| 0 | `+7` = demandeur. Si inconnu localement (ou FUN_00954980 vrai) → message système 0x118. Fenêtre : refus automatique (mode 4) si l'option l'interdit, sinon boîte `msgboxTradeJoinCheck` (texte 0x125, `#@user_name@#` = nom résolu) ; mémorise le partenaire (FUN_00705c10). |
| 1 | aucun traitement (ni FUN_006d0bb0 ni fenêtre). |
| 2 | ouverture : `mgr+0x90 = 1`, fenêtres 0xb (échange) et 0x2d ouvertes, `panel_category_name_01` = nom du partenaire (`+7`), `_02` = propre nom, `FUN_00704ca0(0)` (état « échange »), message 0x253e. À envoyer **aux deux joueurs**, `+7` = l'autre joueur [D]. |
| 3 | annulation : `mgr+0x90 = 0`, fermeture des fenêtres 0xb, 0x2d, 0x19, `FUN_00704ca0(-1)`, boîte texte 0x126. |
| 4 | refus : message système 0x116 (0x31b si `m+0x33` non nul — jamais posé par le parseur [D]) ; FUN_00705c10(`+7`). |
| 5 | ajoute l'ITEM_INFO complet à la colonne du joueur `+7` (soi : `pinfo+0x204` ; partenaire : `pinfo+0x20c`), au plus **10** objets (DAT_00e9694c = 10). |
| 6 | or affiché `trade_selfgold` ou `trade_hisgold` selon `+7` ; montant = `item+0x10` (i64, paquet+0x1c). |
| 7 | verrou de `+7` : soi → `trade_selfok` désactivé, barre 01, message 0x2540 ; partenaire → barre 02. Quand les deux sont verrouillés : message 0x2541 et bouton d'or désactivé. |
| 8 | confirmation de `+7` (soi → `trade_finalconfirm` désactivé). Quand les deux ont confirmé, le client **envoie le mode 9**. |
| 9 | fin : `mgr+0x90 = 0`, message système 0x117, fermeture des fenêtres 0xb, 0x2d, 0x19, `FUN_00704ca0(-1)`, boîte 0x117. |
| 10 | retire de la colonne `+7` l'objet de même handle (FUN_0092dce0 [D : comparaison par handle]). |
| 11 | colonne `+7` : l'objet de handle `item+0x00` prend le count `item+0x10` (FUN_0092ed20). |

Les nombres 0x115…0x2541 sont des identifiants de chaînes localisées (FUN_006a7a10 / FUN_00668490) ; leur texte
n'a pas été résolu.

### 3.4 États locaux [O]
- `mgr+0x90` : 0 libre, 1 échange (S→C mode 2), 2 entrepôt (S→C 211). Remis à 0 par les modes 3, 9 et par
  l'envoi de TS_CS_STORAGE mode 4.
- `FUN_00704ca0` (SPlayerInfoMgr::ChangeState, état en `+0x200`) : 0 = échange, 1 = entrepôt, −1 = aucun.
  Passer de 0 à −1 **envoie automatiquement un mode 3** au partenaire `+0x20c` [O] ; le serveur peut donc recevoir
  un CANCEL après un mode 9 ou un mode 3 qu'il a lui-même émis [D] — à ignorer si l'échange n'existe plus.

## 4. Séquences [D sauf formats marqués O]

- **Ouvrir l'entrepôt** : C→S CONTACT (3002) → S→C DIALOG (3000) avec un menu contenant `open_storage()` → C→S
  DIALOG (3001) `open_storage()` → S→C **OPEN_STORAGE (211)** → **INVENTORY (207)** listant tous les objets
  stockés avec `wear_position = -2` (paquet dédié, ne contenant que des objets d'entrepôt) → **PROPERTY (507)
  `storage_gold`**. Le client passe en état entrepôt (`mgr+0x90 = 2`).
- **Déposer un objet** : C→S STORAGE (212) mode 0 `handle, count` → serveur : vérifie que l'entrepôt est ouvert, la
  capacité et la quantité ; côté inventaire DESTROY_ITEM (254) si la pile part entière, UPDATE_ITEM_COUNT (255)
  sinon ; côté entrepôt INVENTORY (207, `wear_position = -2`) pour la pile créée ou complétée. Si le handle est
  conservé, envoyer **DESTROY_ITEM avant INVENTORY** (DESTROY retire le handle des deux listes, §2.2).
- **Retirer un objet** : STORAGE mode 1 → symétrique (DESTROY/UPDATE côté entrepôt, INVENTORY `wear_position = -1`).
- **Or** : STORAGE mode 2 `count = montant` → GOLD_UPDATE (1001, or de l'inventaire diminué) + PROPERTY
  `storage_gold` ; mode 3 → l'inverse. Ignorer `item_handle`.
- **Ranger / déplacer** : ARRANGE_ITEM (219) `is_storage = 1` ou CHANGE_ITEM_POSITION (218) `is_storage = 1` →
  INVENTORY (207, −2) des objets concernés.
- **Fermer** : C→S STORAGE mode 4 (envoyé dès que la fenêtre est masquée) → aucune réponse requise.
- **Échange complet** (A demande à B) :
  1. A → S : TRADE mode 0, `+7 = B`. S → B : mode 0, `+7 = A`.
  2. B → S : mode 1, `+7 = A` (ou mode 4 → S → A : mode 4, `+7 = B`, fin).
  3. S → A : mode 2, `+7 = B` ; S → B : mode 2, `+7 = A` (ouverture des fenêtres).
  4. Ajouts : X → S mode 5 (handle, count) ; S → A **et** B : mode 5, `+7 = X`, ITEM_INFO complet (count proposé).
     Retrait : mode 10 → S → A et B mode 10 ; quantité : mode 11. Or : X → S mode 6 (`item+0x10` = montant) →
     S → A et B mode 6, `+7 = X`, `item+0x10` = montant total proposé.
  5. Verrou : X → S mode 7 → S → A et B mode 7, `+7 = X`. Toute modification après verrou devrait être refusée.
  6. Confirmation : X → S mode 8 → S → A et B mode 8, `+7 = X`. Quand chaque client a vu les deux mode 8, il envoie
     mode 9 ; le serveur exécute l'échange une seule fois (au premier mode 9 valide, ou après les deux).
  7. S → A et B : mode 9 (`+7` = partenaire), puis mises à jour d'inventaire (DESTROY_ITEM / UPDATE_ITEM_COUNT /
     INVENTORY) et GOLD_UPDATE.
- **Annulation** : X → S mode 3 (bouton, fermeture, changement d'état) → S → l'autre joueur (et X) mode 3 ; aucun
  transfert. Le serveur doit aussi annuler sur déconnexion, mort, éloignement ou ouverture d'entrepôt et envoyer
  mode 3 aux deux.

## 5. Limites / inconnues
- **TS_SC_OPEN_STORAGE** : seul le u32 en +7 est lu ; son sens (handle PNJ ? capacité ?) et l'ouverture effective
  de la fenêtre 0x38 depuis le kind 0x1c n'ont pas été suivis (aucun lecteur de `[mgr+0x54]+0x28` trouvé).
- **Capacité de l'entrepôt** : aucune limite côté client ; valeur serveur à définir.
- **storage_gold** : stocké en 32 bits côté client (FUN_00c729b0 [D]) ; lien avec la borne de retrait de la
  fenêtre non vérifié.
- **GOLD_UPDATE +0xf** : copié mais non lu par FUN_006d2e60 ; consommateur scène FUN_00a47720 non vérifié.
- **CHANGE_ITEM_POSITION** : rôle exact handle1/handle2 (source / destination, 0 pour un emplacement vide ?) non
  établi.
- **TRADE** : `item+0x08` envoyé par le client vient du champ +0x10 de l'objet client (uid probable) ; le lien
  boîte de dialogue → FUN_00790790 / FUN_00799dc0 est déduit (fonctions observées, câblage des boîtes non suivi) ;
  déclencheur de FUN_0092de50 (mode 11) inconnu ; le mode 1 reçu n'a aucun effet ; protection anti-double
  exécution du mode 9 à faire côté serveur.
- **Résultats** : aucun TS_SC_RESULT spécifique à 212/218/219/280 dans FUN_006cf6b0 ; affichage générique non suivi.
- Textes des chaînes 0x115–0x118, 0x125–0x127, 0x31b, 0x253e, 0x2540, 0x2541 non résolus.
- TS_SC_SHOW_WINDOW (3003, FUN_00a2a4a0) et l'entrepôt payant (10002–10004, `/cstorage`) hors périmètre.

## 6. Adresses

| Adresse | Rôle |
|---|---|
| 0xa2fed0 | parseur S→C : cases 0xcf (207), 0xd3 (211), 0x118 (280), 0x3e9 (1001) |
| 0x6d2e60 | gestionnaire jeu : cases 0x16, 0x17, 0x18, 0x19, 0x1c, 0x1d (→212), 0x20, 0x47d (→218), 0x491 (→219) |
| 0x6ca220 / 0x6cdbf0 / 0x6cef50 | envois 212 / 218 / 219 (doublons sans appelant) |
| 0x798670 | fabrique SMSG_CHANGE_STORAGE (kind 0x1d) `(handle, count, mode)` |
| 0x77ef70 | dépôt (mode 0) / retrait (mode 1) d'objet, saisie de quantité |
| 0x933ff0 / 0x931d60 | fenêtre entrepôt : boutons (or, rangement, emplacements) / masquage → mode 4 |
| 0x6cd320 | TS_SC_INVENTORY (wear_position −2 → entrepôt) |
| 0x70e320 / 0x70e470 / 0x70e560 / 0x70e6c0 / 0x70e6f0 | liste entrepôt : ajout / retrait / recherche / tri / count [D, non décompilé] |
| 0x6cdd70 | récepteur TS_SC_PROPERTY (index 0x27 `storage_gold` → `pinfo+0x94`) |
| 0x6cf6b0 | TS_SC_RESULT côté jeu |
| 0x6d0bb0 | consommateur SMSG_TRADE (états `mgr+0x90`, messages système) |
| 0x718fa0 / 0x718b60 | STradeMgr::ProcTradeMsg (envoi 280) / copie ITEM_INFO |
| 0x7766e0 | ctor SMSG_TRADE (12 appelants) |
| 0x704ca0 | SPlayerInfoMgr::ChangeState (0 échange, 1 entrepôt, −1 aucun ; envoi mode 3) |
| 0x6ddc00 / 0x8d5fd0 | demande d'échange (cible / menu de groupe) |
| 0x790790 / 0x7959f0 / 0x799dc0 | accepter-refuser / refus automatique / verrou |
| 0x92c4a0 / 0x92d560 | fenêtre d'échange : réception / boutons |
| 0x92dd70 / 0x92df30 / 0x92de50 / 0x92e0f0 | modes 5 / 10 / 11 / 9 |
| 0x92ec30 / 0x92ed20 / 0x92e2f0 | colonnes d'objets / mise à jour du count / affichage de l'or |
| 0x7e60f0 (0x7e6390) | fenêtre PNJ : contrôle du trigger `open_storage()` |

## 7. Implémentation RappelzCore (2026-11-23)

Choix du serveur, **[D]** sauf formats [O] ci-dessus :

| Élément | Choix |
|---|---|
| PNJ d'entrepôt | `creature_template.npcflag` & `UNIT_NPC_FLAG_BANKER` (0x20000, valeur AzerothCore) ; menu « Entrepôt » → trigger `open_storage()` |
| 211 | `+7` = handle du PNJ |
| Objets stockés | `character_inventory.slot` 150..249 (100 places), `bag` 0 ; envoyés seulement à l'ouverture, dans un 207 dédié (`wear_position` −2, `index` = slot − 150) |
| Rupees stockées | extra `storage_gold` du personnage (`character_settings`, source `rappelz`) ; propriété `storage_gold` (507) ; plafond u32 |
| 212 | modes 0/1 : pile entière (handle conservé : DESTROY_ITEM puis 207) ou partielle (UPDATE_ITEM_COUNT + nouvel objet) ; 2/3 : GOLD_UPDATE + `storage_gold` ; 4 : fermeture. Refusé si l'entrepôt n'est pas ouvert ou si le PNJ n'est plus visible |
| Échange — négociation | état dans le monde (demande cible → demandeur, puis un côté par joueur) ; modes 5/6/10/11 rediffusés aux deux avec `+7` = auteur ; toute modification annule les confirmations ; refusée après le verrou (7) ; 10 objets max |
| Échange — exécution | quand les deux ont envoyé 9 : **une seule transaction** (`rz_database::trade::exchange`) qui revérifie propriétaire, emplacement de sac et quantité de chaque objet, la place libre, puis transfère (`item_instance.owner_guid`, `character_inventory`) et les rupees ; sinon annulation (3) pour les deux. Chaque session applique sa part (DESTROY / UPDATE_ITEM_COUNT / 207 / GOLD_UPDATE) puis 9 est envoyé aux deux |
| Pendant un échange | équiper/déséquiper, jeter, détruire, acheter, vendre et l'entrepôt sont refusés ; la déconnexion annule l'échange |

Validation (MariaDB de test isolée, `rz-bot --talk --storage`, `--trade-with` / `--trade-accept`) : entrepôt ouvert
chez Andariel (flag posé en base de test), objet 101101 déposé (slot 150) et 100 rupees, contenu renvoyé seulement
à l'ouverture après reconnexion ; échange Aria → Brann d'une Potion Bleue (pile de 5 → 4 / 1) et de 50 rupees,
modes 0, 2, 5, 6, 7, 8, 9 observés et résultat persisté. Non vérifié : client graphique d'origine, annulation en
cours de négociation, refus (4), objets d'échange côté Brann (bot sans journal d'inventaire pour 255).

Restent ouverts : capacité réelle de l'entrepôt, entrepôt par compte ou par personnage, sens de 218/219 côté
entrepôt, déclencheur du mode 11.

### Correctif du 2026-11-24 : duplication d'or

**Défaut présent depuis `8185dcf`.** L'échange mettait la base à jour, puis chaque session corrigeait son or en
mémoire par un événement. Un joueur qui coupait sa connexion juste après avoir validé partait sans traiter cet
événement ; sa sauvegarde de départ réécrivait l'ancien montant. Reproduit : 10 échanges sur 10 dupliqués.

| Étape | Règle désormais appliquée |
|---|---|
| Exécution | les deux personnages sont marqués « en règlement » sous le verrou qui retire l'échange ; la marque tombe une fois les événements déposés, même si la tâche est abandonnée |
| Départ | annuler ses échanges ouverts, attendre la fin du règlement, traiter les événements reçus, puis seulement sauvegarder |
| Reconnexion | retirer l'ancienne session, attendre le règlement, puis lire le personnage en base |
| Confirmation (8) | le personnage est sauvegardé ; la transaction exige que l'or offert soit en base et que le receveur ne déborde pas, sinon rien n'est modifié |
| Issue SQL incertaine | les deux sessions partent sans sauvegarder |

Validation : 17 échanges exécutés dont 13 avec coupure brutale (`rz-bot --trade-with … --vanish`), somme de l'or
inchangée ; trois tests unitaires. Fenêtre restante, non traitée : un arrêt brutal du **serveur** dans les quelques
millisecondes entre la validation SQL et le traitement de l'événement, si une sauvegarde périodique y tombe aussi.

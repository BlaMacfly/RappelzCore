# Rappelz NoLimits — protocole réseau client ↔ serveur (SFrame.exe)

Rétro-ingénierie **statique** de `Rappelz NoLimits\SFrame.exe` (PE x86, sha256
`27f42fc8f6b08df8c239640b938d601aeaebb7b3db0c89fc47755325a98a410d`), avec Ghidra 12.1.4 via REA.
Le client n'a été ni exécuté, ni connecté au serveur.

Légende : **[O]** = observé directement dans le code ou les octets ; **[D]** = déduit (nom de champ,
sémantique, recoupement avec les sources publiques Rappelz/rzemu) ; **[?]** = non vérifié.

## 0. Contexte important

- Le binaire est une **recompilation des sources « 9.5.2 »** (chemins de debug
  `c:\users\administrator\downloads\source9.5.2\library\internal\source\network\xsyncstreamconnection.cpp`,
  date de build `Jul 31 2026 17:53:58`), avec des ajouts privés : logs en français
  (`[ENTER] CharacterLogin ... envoi TS_LOGIN`, `CONNEXION FERMEE`, `WAIT id=%u need_size=%u ... framing incomplet`,
  `UNHANDLED id=%u`, `CHARLIST ...`), un fichier `C:\Temp\net_debug.txt`, etc. **[O]**
- Version client envoyée : chaîne **`"201507080"`** (0xd70be4, pointée par 0xe97de4 et 0xe97de8). **[O]**
  (`version.txt` dit 1.3.60, mais ce n'est pas ce qui part sur le réseau.)
- **Les IDs ne correspondent pas tous à la table de noms `TM_*`** embarquée (FUN_00a1f9f0, table de debug) :
  `TS_CS_LOGIN` = **62** (table : 1), `TS_CS_MOVE_REQUEST` = **63** (table : 5),
  `TS_SC_LOGIN_RESULT` = **64** (table : 4). Les autres IDs vérifiés concordent avec la table.
  Pour l'émulateur, **seuls les IDs de la section 5 (observés dans les constructeurs/parseur) font foi.** **[O]**

## 1. Ligne de commande / options

### 1.1 Parsing (FUN_00607d10, appelé au stade `startup` de WinMain FUN_00b90cc0) [O]
- `GetCommandLineA()` complet stocké dans l'env sous `app.cmdline`.
- Découpage sur `' '` ; le token 0 (exe) est ignoré ; chaque token commençant par `/` :
  - `/clé:valeur` ou `/clé=valeur` → `env[clé] = valeur` (chaîne) ;
  - `/clé` seul → drapeau `env[clé] = 1`.
- Les valeurs sont stockées dans un `XEnvStruct` global (FUN_00606a20 = accès singleton,
  `IsExist` FUN_00606bd0, `GetString` FUN_00606350, `GetInt` FUN_00606640, `GetFloat` FUN_006066e0).
- Pas de guillemets gérés : une valeur ne peut pas contenir d'espace.

### 1.2 Clés lues (observées) [O]
`auth_ip`, `auth_port` (lobby, connexion auth), `port` (défaut port game, FUN_009ee0e0), `locale`,
`res_locale`, `country`, `use_nprotect` (défaut 1), `use_nprotect_diagnostic_mode`,
`use_nprotect_soft_detect_1013`, `use_nprotect_ignore_1013`, `notenc`, `tserver`, `chat_mode`,
`secondres`, `winfps`, `string_log`, `font_ratio`, `help_url_w/h`, `cash_url_w/h`, `cash_add_url_w/h`,
`merge_uiframe`, `allow_double_exec`, `check_digital_signer`, `ImmTerminate`, `main_server`,
`imbclogin`, `account`, `password`, `gamexp_sid`, `web_login`, `noautologin`,
`guild_icon_upload.ip`, `guild_icon_upload.port`.

`launcher_config.json` (auth_ip 185.234.72.20, auth_port 4615, locale windows-1250, country CN,
use_nprotect 0, notenc true, layout_dir 6…) est traduit par le launcher en arguments, vraisemblablement
`SFrame.exe /auth_ip:185.234.72.20 /auth_port:4615 /locale:windows-1250 /country:CN /use_nprotect:0 /notenc ...` **[D]**

### 1.3 `country` → code pays (FUN_0098d5c0) [O]
`<NULL>`/inconnu=1 (KR), HK=2, US=4, DE=8, JP=0x10, TW=0x20, **CN=0x40**, FR=0x80, RU=0x100, MASG=0x200,
PH=0x400, VN=0x800, TL=0x1000, ME=0x2000, TR=0x4000, PL=0x8000, IT=0x10000, BR=0x20000, ES=0x40000, ID=0x80000.
Impact réseau : TS_CA_ACCOUNT reçoit une extension (iovation si US=4, adresse MAC si DE/FR/TR/PL/IT,
FUN_0098e740) — **avec CN (NoLimits) : paquet standard de 149 octets, sans extension.**

### 1.4 `notenc` (FUN_00b87450, init_resource) [O]+[D]
- Si `/notenc` présent : le répertoire `./Resource/` est monté comme répertoire **en clair**
  (objet DAT_00ede86c, FUN_005db660 seul).
- Sinon : `./Resource/` est monté via un lecteur supplémentaire (FUN_005ddce0 + FUN_005ddd40(..., 0, 0))
  — c.-à-d. fichiers loose **chiffrés** attendus. [D]
- Les archives `data.000…` sont toujours montées (FUN_005dfeb0 + FUN_005ddd40(..., 1, 1)).
- **`notenc` n'a aucun effet sur le réseau** : les 7 constructions de `XSyncStreamConnection`
  (FUN_009ed9a0, appelées en 0x9ed82d, 0x9edf27, 0xa01bcf, 0xa07770, 0xa07e92, 0xa0b337, 0xa114f1, + 0x854290)
  passent toutes `encrypt=1, threaded=1` en dur (`6A 01 6A 01`). **[O]**

### 1.5 Contrôle « lancé par le launcher » (FUN_00b8dca0) [O]
Lit la variable d'environnement `<nom process>_RUNNER` (handle d'event numérique), fait `SetEvent` +
`CloseHandle` ; si la ligne de commande est vide ou un contrôle échoue → *« Game client is must executed
by launcher! »* (sauf débogueur attaché). `RappelzCmdLauncher.exe` (imports `CreateEventA`,
`SetEnvironmentVariableA`, `CreateProcessA`, format `%s "%s"`) prépare cela. Option
`check_digital_signer` (défaut 0) : vérification de signature « Gala Lab Corp. » + variable `_PARENT`.

## 2. Couche transport

- TCP, socket `XSyncStreamConnection` (vtable 0xd4dd90), thread réseau `FUN_005f69e0` →
  `FUN_005f62a0` (select/recv/send toutes les 10 ms). **[O]**
- Connexion : FUN_005f5f50. Écriture : FUN_005f5d60. Lecture : FUN_005f5c50.
- Objets de connexion (FUN_005f6680) : `+0x434` cipher d'**envoi**, `+0x438` cipher de **réception**,
  `+0x43c` file d'envoi, `+0x440` file de réception. **[O]**
- **Tout le flux** (en-têtes compris) est chiffré : à l'écriture, le buffer complet est chiffré par le
  cipher d'envoi puis mis en file ; à la réception, chaque bloc `recv()` est déchiffré par le cipher de
  réception avant parsing. Deux états RC4 indépendants par connexion (un par sens), réinitialisés à chaque
  nouvelle connexion (auth et game : même clé). **[O]**

## 3. En-tête de paquet (TS_MESSAGE) [O]

| Offset | Type | Champ |
|---|---|---|
| 0 | u32 LE | `size` — taille **totale** du paquet, en-tête inclus |
| 4 | u16 LE | `id` |
| 6 | u8 | `msg_check_sum` |
| 7 | … | corps |

**Checksum** : somme sur 8 bits des 6 premiers octets (les 4 octets de `size` + les 2 octets de `id`) :
```rust
fn checksum(size: u32, id: u16) -> u8 {
    size.to_le_bytes().iter().chain(id.to_le_bytes().iter()).fold(0u8, |a, b| a.wrapping_add(*b))
}
```
Vu dans tous les constructeurs (ex. FUN_00a02130, FUN_00a02180, FUN_009ed8f0, 0x9e96ee). Exemple :
`1b 00 00 00 11 27` → `0x1b+0x11+0x27 = 0x53`.

**Réception (FUN_00a2fed0)** : lit ≤ 0x400 octets par appel, accumule, tant qu'il reste ≥ 7 octets : lit
`size`/`id`, attend si `size` > octets disponibles, sinon dispatch. **Le client ne vérifie pas le checksum
en réception** [O] (le serveur doit quand même le calculer correctement ; le serveur officiel le vérifie [D]).

### 3.1 Paquets « leurres » (id 9999 = TM_NONE) envoyés par le client [O]
Fonctions d'envoi : FUN_009edf60 (lobby/auth) et FUN_009ee9a0 (game) — « ConnPendMessage ».
1. Si un reliquat `usRestDummy` (conn+0x100) ≠ 0 : envoie d'abord `usRestDummy` octets **copiés depuis le
   début du paquet réel suivant** (bourrage qui termine le leurre précédent), puis remet à 0.
2. Avec 10 % de chance : envoie un leurre complet `{size = 7..13 (rand%7+7), id = 9999, checksum}`
   suivi de `size-7` octets de contenu indéterminé (pile).
3. Envoie le paquet réel.
4. Avec 30 % de chance : prépare un leurre `{size = 8..14, id = 9999}` dont seulement
   `size - rest` octets sont envoyés (`rest = rand % (size-7)`), le reste étant envoyé au prochain envoi (étape 1).

→ **Côté serveur : tout paquet d'id 9999 doit être ignoré (sauter `size` octets).** Le cadrage par
`size` reste toujours cohérent ; un leurre peut rester « incomplet » jusqu'au prochain paquet du client.

## 4. Chiffrement

### 4.1 Flux TCP : RC4 « XRC4Cipher » (vtable 0xdffdac) [O]
- **Clé (identique auth et game, envoi et réception)** : chaîne ASCII de 17 octets à 0xdfe098
  `}h79q~B%al;k'y $E` = `7d 68 37 39 71 7e 42 25 61 6c 3b 6b 27 79 20 24 45`
  (posée dans FUN_005f6680 et FUN_0062d000 ; le constructeur FUN_0062ce30 initialise d'abord avec
  `"Neat & Simple"` puis l'objet est ré-initialisé avec la vraie clé).
- Init (FUN_0062ca20) — **attention, KSA non standard** (vérifié en assembleur 0x62ca9d–0x62cad4) :
```rust
fn rc4_init(key: &[u8]) -> ([u8;256], u8, u8) {
    let mut s = [0u8;256]; for i in 0..256 { s[i] = i as u8; }
    let k: Vec<u8> = (0..256).map(|i| key[i % key.len()]).collect();
    let (mut j, mut t) = (0usize, s[0]);
    for i in 0..256 {
        j = (t as usize + k[j] as usize + j) & 0xff;   // k indexé par j, et t = ancien s[i-1]
        t = s[i]; s[i] = s[j]; s[j] = t;              // swap ; t garde l'ancien s[i]
    }
    // drop 1013 (0x3f5) octets de keystream
    let (mut x, mut y) = (0u8, 0u8);
    for _ in 0..1013 { x = x.wrapping_add(1); let a = s[x as usize]; y = y.wrapping_add(a);
                       s[x as usize] = s[y as usize]; s[y as usize] = a; }
    (s, x, y)
}
```
- Chiffrement/déchiffrement (FUN_0062d080, slots vtable +4/+8 ; 4ᵉ argument « peek » = sauvegarde/restaure
  l'état, non utilisé sur le réseau) : PRGA RC4 **standard** :
  `x+=1; a=S[x]; y+=a; swap(S[x],S[y]); out = in ^ S[(S[x]+S[y]) & 0xff]`.
- Vecteurs de test (calculés avec l'algorithme ci-dessus par un script Node temporaire) :
  - 32 premiers octets de keystream après init+drop :
    `5d0c41012b250cfbab96c97300634a583548d5177d7a4d48e7219f62dca00380`
  - (pour comparaison, RC4 standard-drop1013 avec la même clé : `1d1780eb6f6be24a…` — **différent**)
  - TS_CA_VERSION en clair `1b0000001127533230313530373038300000000000000000000000`
    → chiffré (si c'est le 1ᵉʳ paquet du flux) `460c41013a025fc99ba7fc43375372683548d5177d7a4d48e7219f`.
  Si l'interopérabilité échoue avec une capture réelle, tester en second choix le KSA standard (drop 1013).

### 4.2 Mot de passe : RSA-1024 + AES-128-CBC (OpenSSL 1.0.1h lié statiquement) [O]
Pas de DES/« MERONG » dans ce binaire (aucune occurrence).
1. Après connexion auth et envoi de TS_CA_VERSION, le client génère une paire **RSA 1024 bits, e = 65537**
   (FUN_005b7e60 : `RSA_generate_key_ex(rsa, 0x400, BN(0x10001))`), exporte la clé publique en PEM
   **SubjectPublicKeyInfo** `-----BEGIN PUBLIC KEY-----` (FUN_005b7fd0 → `PEM_ASN1_write_bio(i2d_RSA_PUBKEY, "PUBLIC KEY")`)
   et l'envoie dans **TS_CA_RSA_PUBLIC_KEY (71)**.
2. Le serveur répond **TS_AC_AES_KEY_IV (72)** : `RSA_public_encrypt(PKCS#1 v1.5)` de 32 octets
   `aes_key[16] || aes_iv[16]`. Le client fait `RSA_private_decrypt(..., RSA_PKCS1_PADDING)`
   (FUN_005b8120 → FUN_0052a0c0 = `rsa->meth->rsa_priv_dec`), puis `EVP_EncryptInit_ex` /
   `EVP_DecryptInit_ex` avec **EVP_aes_128_cbc** (FUN_00528140 → EVP_CIPHER nid 419, bloc/clé/IV 16,
   FUN_005b8060), clé = 16 premiers octets, IV = 16 suivants.
3. Chaque chiffrement (FUN_005b8180) / déchiffrement (FUN_005b8220) ré-initialise le contexte avec
   l'IV d'origine (`EVP_*Init_ex(ctx, NULL, NULL, NULL, NULL)`), padding **PKCS#7** (taille sortie =
   `(len & ~15) + 16`).
4. Utilisations : mot de passe dans TS_CA_ACCOUNT (chiffré, ≤ 77 octets) ; **clé one-time** dans
   TS_AC_SELECT_SERVER (chiffrée par le serveur, déchiffrée par le client, 8 premiers octets = u64).

## 5. Flux de connexion et paquets

Fonctions clés : parseur unique S→C **FUN_00a2fed0** (switch sur l'id → objets `SMSG_*` avec un « kind »
interne, puis file FUN_00a35160) ; dispatch lobby **FUN_00a09b50** (SGameRenewalLobby, vtable 0xd732a0,
slot +0x1cc) ; process game **FUN_009ef310** (SGameManager) ; envoi game FUN_009ee9a0, envoi lobby
FUN_009edf60 (via FUN_00a03010 / FUN_00a15520).

### 5.1 Séquence
```
AUTH (auth_ip:auth_port)
 C→S TS_CA_VERSION (10001)            — sans réponse
 C→S TS_CA_RSA_PUBLIC_KEY (71)
 S→C TS_AC_AES_KEY_IV (72)
 C→S TS_CA_ACCOUNT (10010)
 S→C TS_AC_RESULT (10000) {request=10010, result=0}
 C→S TS_CA_SERVER_LIST (10021)
 S→C TS_AC_SERVER_LIST (10022)
 C→S TS_CA_SELECT_SERVER (10023)
 S→C TS_AC_SELECT_SERVER (10024)      — le client ferme la connexion auth
GAME (ip:port de l'entrée choisie)
 C→S TS_CS_VERSION (51)  [+ éventuellement TS_CS_REPORT (8000)]
 C→S TS_CS_ACCOUNT_WITH_AUTH (2005)
 S→C TS_SC_RESULT (0) {request=2005, result=0}
 C→S TS_CS_CHARACTER_LIST (2001)
 S→C TS_SC_CHARACTER_LIST (2004)
 (option) TS_CS_CHECK_CHARACTER_NAME (2006) / TS_CS_CREATE_CHARACTER (2002) / TS_CS_DELETE_CHARACTER (2003)
          → TS_SC_RESULT (0) {request=2006/2002/2003}
 C→S TS_CS_LOGIN (62)
 S→C TS_SC_LOGIN_RESULT (64), puis TS_SC_STAT_INFO, TS_SC_PROPERTY, TS_SC_ENTER, … ; TS_TIMESYNC (2)
 jeu : TS_CS_MOVE_REQUEST (63) ↔ TS_SC_MOVE (8) ; TS_CS_CHAT_REQUEST (20) ↔ TS_SC_CHAT (22)/CHAT_LOCAL (21)
```

### 5.2 Auth — client → serveur

| Paquet | ID | Taille | Corps (offsets depuis le début du paquet) | Source |
|---|---|---|---|---|
| TS_CA_VERSION | 10001 (0x2711) | 27 | +7 `char version[20]` = `"201507080"` | FUN_00a02180, 0xa0c088.. [O] |
| TS_CA_RSA_PUBLIC_KEY | 71 (0x47) | 11 + n | +7 `u32 key_size` (n) ; +11 `u8 key[n]` = texte PEM « PUBLIC KEY » | FUN_00a021f0 + FUN_005b7fd0 [O] |
| TS_CA_ACCOUNT | 10010 (0x271a) | 149 | +7 `char account[61]` (strncpy 60, +0x43 = 0) ; +0x44 `u32 password_size` ; +0x48 `u8 password[77]` (AES-128-CBC/PKCS7 du mot de passe) | 0xa09e82–0xa09f06 [O] |
| TS_CA_SERVER_LIST | 10021 (0x2725) | 7 | — | FUN_00a02130 [O] |
| TS_CA_SELECT_SERVER | 10023 (0x2727) | 9 | +7 `u16 server_idx` | 0xa0c832 [O] |

Variantes de compte (type de login lobby+0x250) [O] : 10011 (0x271b, 136 o., compte strncpy 60),
10012 (0x271c, 136 o., compte strncpy 55) = `account[61] + u32 size + u8 pwd[64]` [D] ;
10013 (0x271d, 87 o., login web/SID `gamexp_sid`). Avec country US/DE/FR/TR/PL/IT, FUN_00a15520 ajoute
`u8 flag` (+0x95), `u16 len` (+0x96), `data[len]` (+0x98) et `size += len+3`. Non utilisé par NoLimits (CN).

### 5.3 Auth — serveur → client

| Paquet | ID | Taille | Corps | Kind / Source |
|---|---|---|---|---|
| TS_AC_RESULT | 10000 (0x2710) | 15 | +7 `u16 request_msg_id` ; +9 `u16 result` ; +0xb `i32 login_flag` | kind 1, handler FUN_00a07260 [O] |
| TS_AC_RESULT_WITH_STRING | 10002 (0x2712) | 19 + n | +7 u16 request ; +9 u16 result ; +0xb i32 login_flag ; +0xf `i32 n` ; +0x13 `char str[n]` | kind 0xb2 [O] |
| TS_AC_AES_KEY_IV | 72 (0x48) | 11 + n | +7 `u32 n` ; +0xb `u8 data[n]` (RSA PKCS#1 → 32 o. clé+IV ; n = 128 pour RSA-1024) | kind 0xbe [O] |
| TS_AC_SERVER_LIST | 10022 (0x2726) | 11 + 302·c | +7 `u16 last_login_server_idx` [D] ; +9 `u16 count` ; +0xb `SERVER_INFO[count]` (302 o. chacun) | kind 2 [O] |
| TS_AC_SELECT_SERVER | 10024 (0x2728) | 41 | +7 `u16 result` ; +9 `i32 encrypted_size` (16) ; +0xd `u8 encrypted_data[16]` ; +0x1d `u64` (copié, non utilisé par le lobby — clé en clair ? [D]) ; +0x25 `u32 pending_time` | kind 3, 0xa0a145 [O] |
| TS_AC_UPDATE_PENDING_TIME | 10025 (0x2729) | 11 | +7 `u32 pending_time` | kind 0x6e [O] |
| TS_AC_ACCOUNT_INFO (?) | 10014 (0x271e) | ≥ 0x48 | +7 chaîne ; +0x44 u32 | kind 0xbf, SMSG_ACCOUNT_INFO [O] |

`SERVER_INFO` (302 o.) [taille O, champs D] : +0 `u16 server_idx` ; +2 `char name[21]` ;
+0x17 `u8 is_adult` ; +0x18 `char screenshot_url[256]` ; +0x118 `char ip[16]` ; +0x128 `i32 port` ;
+0x12c `u16 user_ratio`.

`TS_AC_RESULT` (FUN_00a07260) [O] : n'est traité que si `request_msg_id == 10010` et état lobby = 3.
`result == 0` → succès ; `login_flag & 2` → message 0x3b1 et arrêt ; sinon envoi automatique de
TS_CA_SERVER_LIST. Codes d'erreur gérés : 1, 5, 6, 9, 10, 0x16, 0x26, 0x27 (temps de compte, `login_flag`),
0x28, 0x3c (IP bloquée), 0x59, 0x66, 0x67 (EULA) ; après une erreur la connexion auth est recréée.

`TS_AC_SELECT_SERVER` côté serveur : `encrypted_data = AES-128-CBC(key, iv, one_time_key.to_le_bytes())`
→ 16 octets ; mettre `encrypted_size = 16`, `pending_time = 0` (si ≠ 0 le client affiche une file d'attente
de `pending_time*10`).

### 5.4 Game — client → serveur

| Paquet | ID | Taille | Corps | Source |
|---|---|---|---|---|
| TS_CS_VERSION | 51 (0x33) | 27 | +7 `char version[20]` = `"201507080"` | FUN_009ee0e0 (0x9ee20e) [O] |
| TS_CS_REPORT | 8000 (0x1f40) | 15 + n | +7 `u16 len` (= n+6) ; +9 `u8 mac[6]` ; +0xf infos système (texte) | FUN_009ee0e0 (2ᵉ branche) [O] |
| TS_CS_ACCOUNT_WITH_AUTH | 2005 (0x7d5) | 76 | +7 `char account[61]` ; +0x44 `u64 one_time_key` | FUN_009e9c10 [O] |
| TS_CS_CHARACTER_LIST | 2001 (0x7d1) | 68 | +7 `char account[61]` | 0xa0f896 [O] |
| TS_CS_CHECK_CHARACTER_NAME | 2006 (0x7d6) | 26 | +7 `char name[19]` | a09b50 case 0x3fb [O] |
| TS_CS_CREATE_CHARACTER | 2002 (0x7d2) | 639 | +7 `LOBBY_CHARACTER_INFO` (632 o., level=1, nom à +0xc0) | FUN_009e9760 [O] |
| TS_CS_DELETE_CHARACTER | 2003 (0x7d3) | 45 | +7 `char name[19]` ; +0x1a `char security_no[19]` [D] | FUN_009e98f0 [O] |
| **TS_CS_LOGIN** | **62 (0x3e)** | **287** | +7 `char name[24]` (strncpy 23) ; +0x1f `u8 reserved[256]` (zéro) | FUN_009e9690 [O] |
| TS_TIMESYNC (réponse) | 2 | 11 | +7 `u32 time` (renvoie la valeur reçue, max 4 fois) | FUN_009ef310 [O] |
| **TS_CS_MOVE_REQUEST** | **63 (0x3f)** | 26 + 8·c | +7 `u32 handle` ; +0xb `f32 x` ; +0xf `f32 y` ; +0x13 `u32 cur_time` ; +0x17 `u8 speed_sync` ; +0x18 `u16 count` ; +0x1a `{f32 tx, f32 ty}[count]` | FUN_009efe50 [O] |
| TS_CS_CHAT_REQUEST | 20 (0x14) | 31 + len | +7 `char target[21]` ; +0x1c `u8 request_id` (0) ; +0x1d `u8 len` (≤170) ; +0x1e `u8 type` ; +0x1f `char msg[len]` | FUN_006d77a0 [O] |
| TS_CS_QUERY | 13 (0xd) | 11 | +7 `u32 handle` | scan d'en-têtes [O] |
| TS_CS_TARGETING | 511 | 11 | +7 `u32 target` [D] | [O] |
| TS_CS_RETURN_LOBBY / REQUEST_RETURN_LOBBY / LOGOUT | 23 / 25 / 27 | 7 | — | 0x6d56e3, 0x6d4e77, 0x6d694b [O] |
| TS_CS_CHANGE_ALIAS | 31 | 26 | +7 `char name[19]` | 0x90771b [O] |
| TS_CS_SECURITY_NO | 9005 | 30 | +7 `i32 mode` ; +0xb `char security_no[19]` [D] | [O] |
| TS_CS_CHECK_ILLEGAL_USER | 57 (0x39) | 15 | +7 `"MRFS"` ; +0xb `u32 fnv1a32(SFrame.exe)` — toutes les 30 s | FUN_009ed8f0, FUN_009ef310 [O] |
| TS_CS_GAME_GUARD_AUTH_ANSWER | 56 | 11 | nProtect (ignorable) | [O] |

### 5.5 Game — serveur → client

| Paquet | ID | Taille | Corps | Kind / Source |
|---|---|---|---|---|
| TS_SC_RESULT | 0 | 15 | +7 `u16 request_msg_id` ; +9 `u16 result` ; +0xb `i32 value` | kind 4, FUN_00a29bf0 [O] |
| TS_TIMESYNC | 2 | 11 | +7 `u32 time` (horloge serveur, centièmes de s [D]) | kind 0x2712 [O] |
| TS_SC_ENTER | 3 | variable | voir ci-dessous | kind 0xb, FUN_00a2f340 [O] |
| TS_SC_MOVE | 8 | 19 + 8·c | +7 `u32 start_time` ; +0xb `u32 handle` ; +0xf `u8 tlayer` ; +0x10 `u8 speed` ; +0x11 `u16 count` ; +0x13 `{f32 x, f32 y}[count]` | kind 6 [O] |
| TS_SC_LEAVE | 9 | 11 | +7 `u32 handle` | kind 10 [O] |
| TS_SC_SET_TIME | 10 | 11 | +7 `i32 gap` (ajouté à l'horloge de jeu) | [O] |
| TS_SC_REGION_ACK | 11 | 15 | +7 `u32 rx` ; +0xb `u32 ry` [D] | kind 8 [O] |
| TS_SC_WARP | 12 | 20 | +7 `f32 x` ; +0xb `f32 y` ; +0xf `f32 z` ; +0x13 `u8 layer` | kind 30001 [O] |
| TS_SC_CHAT_LOCAL | 21 | 13 + len | +7 `u32 handle` ; +0xb `u8 len` ; +0xc `u8 type` ; +0xd `char msg[len]` | kind 0x30, FUN_00a277f0 [O] |
| TS_SC_CHAT | 22 | 31 + len | +7 `char sender[21]` ; +0x1c `u16 len` ; +0x1e `u8 type` ; +0x1f `char msg[len]` (sender `@NOTICE`, `@GUILD`, … = canaux spéciaux) | kind 0x30, FUN_00a26e60 [O] |
| TS_SC_CHAT_RESULT | 24 | 17 | +7 u8 ; +8 u8 ; +9 i32 ; +0xd i32 | kind 0x5a [O] |
| TS_SC_DISCONNECT_DESC | 28 | 8 | +7 `u8 code` → MessageBox « DISCONNECT ID=%d » et sortie | [O] |
| **TS_SC_LOGIN_RESULT** | **64 (0x40)** | 97 | voir ci-dessous | kind 9, FUN_00a2f800 [O] |
| TS_SC_PROPERTY | 507 | 36 (+chaîne) | +7 `u32 handle` ; +0xb `u8 is_number` ; +0xc `char name[16]` ; +0x1c `i64 value` ; +0x24 `char str[]` si !is_number | kind 0x36, FUN_00a29150 [O] |
| TS_SC_STAT_INFO | 1000 | 110 | +7 `u32 handle` ; +0xb..+0x6d stats/attributs (98 o.) ; +0x6d `u8 type` [D] | kind 31 [O taille] |
| TS_SC_GOLD/LEVEL/EXP_UPDATE | 1001/1002/1003 | — | LEVEL : +7,+0xb,+0xf u32 ; EXP : +7..+0x17 (5×u32) | kinds 32/33/34 [O] |
| TS_SC_CHARACTER_LIST | 2004 (0x7d4) | 15 + 632·c | +7 `u32 current_server_time` ; +0xb `u16 last_login_index` ; +0xd `u16 count` ; +0xf `LOBBY_CHARACTER_INFO[count]` (max 8 lus pour log) | kind 5, FUN_00a2ef90 [O] |

**TS_SC_LOGIN_RESULT (64), 97 octets** [offsets O, noms D] : +7 `u16 result` (0 = OK ; 1/0x45 → msg 0x93,
8 → msg 0x92) ; +9 `u32 handle` ; +0xd `f32 x` ; +0x11 `f32 y` ; +0x15 `f32 z` ; +0x19 `u8 layer` ;
+0x1a `f32 face_direction` ; +0x1e `i32 region_size` ; +0x22 `i32 hp` ; +0x26 `i32 mp` ; +0x2a `i32 max_hp` ;
+0x2e `i32 max_mp` ; +0x32 `i32 havoc` ; +0x36 `i32 max_havoc` ; +0x3a `i32 sex` ; +0x3e `i32 race` ;
+0x42 `u32 skin_color` ; +0x46 `char name[19]` ; +0x59 `i32 cell_size` ; +0x5d `i32 guild_id`.

**LOBBY_CHARACTER_INFO (632 o.)** [taille et offsets level/job/name/wear O, reste D] :
+0 `i32 sex` ; +4 `i32 race` ; +8 `i32 model_id[5]` ; +0x1c `i32 hair_color_index` ; +0x20 `u32 hair_color_rgb` ;
+0x24 `u32 hide_equip_flag` ; +0x28 `i32 texture_id` ; +0x2c `i32 wear_info[28]` ; +0x9c `i32 level` ;
+0xa0 `i32 job` ; +0xa4 `i32 job_level` ; +0xa8 `i32 exp_percentage` ; +0xac `i32 hp` ; +0xb0 `i32 mp` ;
+0xb4 `i32 permission` ; +0xb8 `u8 is_banned` ; +0xb9 `char name[19]` ; +0xcc `u32 skin_color` ;
+0xd0 `char create_time[30]` ; +0xee `char delete_time[30]` ; +0x10c `i32 wear_item_enhance_info[28]` ;
+0x17c `i32 wear_item_level_info[28]` ; +0x1ec `u8 wear_item_elemental_type[28]` ; +0x208 `i32 wear_appearance_code[28]`.

**TS_SC_ENTER (3)** [O] : +7 `u8 type` ; +8 `u32 handle` ; +0xc `f32 x` ; +0x10 `f32 y` ; +0x14 `f32 z` ;
+0x18 `u8 layer` ; +0x19 `u8 obj_type` ; +0x1a données selon `obj_type`. Tailles totales copiées :
joueur (0) **122**, NPC (1) 72, item (2) 70, monstre (3) 73, summon (4) 96, skill prop (5) 38,
field prop (6) 63, pet (7) 95. Joueur [offsets O pour sex/race/name, autres D] : +0x1a `u32 status` ;
+0x1e `f32 face_direction` ; +0x22 hp ; +0x26 max_hp ; +0x2a mp ; +0x2e max_mp ; +0x32 level ;
+0x36 `u8 race` ; +0x37 `u32 skin_color` ; +0x3b `u8 is_first_enter` ; +0x3c `i32 energy` ; +0x40 `u8 sex` ;
+0x41..+0x58 (face, texture, hair, couleurs) ; +0x59 `char name[19]` ; +0x6c `u16 job_id` ;
+0x6e `u32 ride_handle` ; +0x72 `u32 guild_id` ; +0x76 `i32 title_id`.
⚠ Le client **jette** un ENTER joueur si `sex ∉ {1,2}` ou `race ∉ [3..8]` (log « Enter : Name[...] Sex Race »).

### 5.6 Résultats TS_SC_RESULT gérés par le lobby (kind 4) [O]
- request 2005 : 0 → suite (FUN_00a12870, liste des persos) ; 1/6/0x12 → erreur et retour état 6 ; 0x10 → msg.
- request 2001 : result ≠ 0 → erreur.
- request 2003 : 0 → OK ; 0x36 → compteur (5 essais) ; autres → msg.
- request 2006 : 0 → nom libre (puis création) ; 9 / 0x13 → nom pris / invalide.
- request 2002 : 0 → perso créé ; 9 / 0x13 / autre → erreurs.
- request 2000 / 9005 : flux sécurité/compte.

### 5.7 Ping / maintien
Pas de paquet ping dédié observé. Maintien : TS_TIMESYNC (2) initié par le serveur (le client répond
jusqu'à 4 fois), TS_CS_CHECK_ILLEGAL_USER (57) toutes les 30 s, et les paquets leurres 9999. [O]

## 6. Adresses utiles
| Adresse | Rôle |
|---|---|
| 0xb90cc0 | WinMain (stades startup/parse_cmdline/init_resource/…) |
| 0x607d10 | parsing ligne de commande → XEnv |
| 0xb87450 | init ressources (`notenc`, data.000) |
| 0x5f6680 | init connexion : ciphers RC4, files, verrous |
| 0x62ca20 / 0x62d080 | RC4 init (KSA + drop 1013) / crypt |
| 0x5f5c50 / 0x5f5d60 / 0x5f62a0 / 0x5f5f50 | Read / Write / Select-thread / Connect |
| 0x9edf60 / 0x9ee9a0 | envoi lobby / game (avec leurres 9999) |
| 0xa2fed0 | parseur S→C (switch id → SMSG kind) |
| 0xa09b50 / 0xa07260 | dispatch lobby / handler TS_AC_RESULT |
| 0x9ef310 / 0x9eed60 | SGameManager::Process / OnMessage |
| 0x5b7e60 / 0x5b7fd0 / 0x5b8120 / 0x5b8060 / 0x5b8180 / 0x5b8220 | RSA gen / export PEM / RSA dec / AES init / AES enc / AES dec |
| 0x9e9690 / 0x9efe50 / 0x6d77a0 | TS_CS_LOGIN / TS_CS_MOVE_REQUEST / TS_CS_CHAT_REQUEST |
| 0xa2f800 / 0xa2f340 / 0xa2ef90 | parse LOGIN_RESULT / ENTER / CHARACTER_LIST |
| 0xa1f9f0 | table de noms `TM_*` (debug, IDs partiellement obsolètes) |

## 7. Limites / à vérifier
- Noms de champs [D] issus du recoupement avec les sources publiques (epic 9) ; offsets et tailles [O].
- KSA RC4 non standard lu en assembleur ; à confirmer sur une capture réelle si possible.
- TS_CS_REGION_UPDATE, TS_CS_UPDATE, TS_CS_GAME_TIME, TS_SC_STAT_INFO (détail) non disséqués.
- L'u64 à +0x1d de TS_AC_SELECT_SERVER est copié mais non utilisé par le code lobby observé.

## 8. Compléments (TS_SC_ENTER NPC/monstre/item, entrée en jeu, TS_SC_STAT_INFO)

Analyse statique complémentaire (Ghidra via REA, même binaire). Convention : dans les objets `SMSG_*`
construits par le parseur, **offset SMSG = offset paquet + 0x0c** (en-tête TS_MESSAGE recopié à SMSG+0x0c) ;
tous les offsets ci-dessous sont des **offsets paquet** (depuis le début du paquet, en-tête inclus).

### 8.1 TS_SC_ENTER (3) — octet `type` et aiguillage [O]
FUN_00a2f340 recopie les octets **bruts** (aucun décodage au parse). Le handler monde FUN_006b2130
(kind 0xb) → FUN_006aec50 aiguille sur **`type` (+7)** puis **`obj_type` (+0x19)** :

| `type` (+7) | `obj_type` (+0x19) | Handler | Classe |
|---|---|---|---|
| 0 | 0 | FUN_006ab010 | joueur (SGameOtherPlayer) |
| 1 | 1 | FUN_006aa570 | NPC (SGameNpc, ctor FUN_009736c0) |
| 1 | 3 | FUN_006aa290 | monstre (SGameMob, ctor FUN_009704f0) |
| 1 | 4 / 7 | FUN_006a9430 / FUN_006aab90 | summon / pet |
| 2 | 2 | FUN_006aa0d0 | item au sol (SGameItem, ctor FUN_00968cf0) |
| 2 | 5 / 6 | (inline) / FUN_006a9e80 | skill prop / field prop |

⚠ **`type` n'est pas libre** : un monstre ou un NPC envoyé avec `type` ≠ 1, ou un item avec `type` ≠ 2, est ignoré.
Si le `handle` existe déjà (FUN_006b6440), seuls summon (4) et pet (7) sont mis à jour ; les autres ENTER sont ignorés.

### 8.2 Bloc « creature info » commun (obj_type 0, 1, 3, 4, 7) [O offsets, D noms]
Lu par FUN_00940f90 (case kind 0xb) et les handlers ci-dessus :
`+0x1a u32 status` (→ data+0xec) ; `+0x1e f32 face_direction` ; `+0x22 i32 hp` ; `+0x26 i32 max_hp` ;
`+0x2a i32 mp` ; `+0x2e i32 max_mp` ; `+0x32 i32 level` ; `+0x36 u8 race` ; `+0x37 u32 skin_color` ;
`+0x3b u8 is_first_enter` ; `+0x3c i32 energy` → fin à **+0x40** (38 octets, confirmé).
Joueur (case 0) : en plus `+0x40 u8 sex`, `+0x41/+0x45/+0x49 i32` (apparence, → data+0xf0/f4/f8), nom `+0x59`, `+0x6c u16 job`.

### 8.3 NPC (obj_type 1) — 72 octets [O]
| Offset | Type | Champ |
|---|---|---|
| +0x07 | u8 | type = **1** |
| +0x08..+0x19 | — | handle, x, y, z, layer, obj_type = 1 |
| +0x1a..+0x3f | creature info (38 o.) | hp/mp/level lus mais nom/stats affichés viennent de la NpcResource |
| **+0x40** | **`EncodedId` (8 o.)** | **npc_id encodé** (voir 8.6) |

Le NPC est recherché dans la base NPC (FUN_0064ba40 + FUN_006b62d0) par la clé décodée [O].
Si un NPC de même handle est en cache, le client compare les ids décodés pour le réutiliser (0x6aa633–0x6aa684) [O].

### 8.4 Monstre (obj_type 3) — 73 octets [O]
| Offset | Type | Champ |
|---|---|---|
| +0x07 | u8 | type = **1** |
| +0x1a..+0x3f | creature info | status, face, hp (+0x22, aussi passé à vtable+0x208), max_hp, mp, max_mp, level ; `race` (+0x36) stocké en mob+0x1260 |
| **+0x40** | **`EncodedId` (8 o.)** | **monster_id encodé** — passé tel quel au ctor SGameMob (SMSG `[esi+0x4c]`, `[esi+0x50]`) |
| +0x48 | u8 | is_tamed [D — recopié, lecteur non identifié] |

Le nom affiché vient de la MonsterResource (name_id) ; « Unknown Name » si l'id n'est pas trouvé.
Lookup : table de hachage du MonsterDB (FUN_0064f880 / FUN_006a1840), seau = `(i16)(hi*2) % n` [O].

### 8.5 Item au sol (obj_type 2) — 70 octets [O offsets, D noms]
| Offset | Type | Champ |
|---|---|---|
| +0x07 | u8 | type = **2** |
| **+0x1a** | **`EncodedId` (8 o.)** | **item code encodé** (même schéma) → ctor SGameItem FUN_00968cf0 |
| +0x22 | i64 | count [D] |
| +0x2a | u32 | drop_time [D] (→ item+0x1240) |
| +0x2e | u32[3] | hPlayer[3] (droits de ramassage) |
| +0x3a | i32[3] | party_id[3] |

Si `hPlayer[0] == 0 && party_id[0] == 0` → état « libre » (item+0x125c = 3), sinon 0 (FUN_009697c0) [O].

### 8.6 Encodage des ids (`EncodedId`, 8 octets) [O]
**Ce n'est PAS la permutation de bits de db_monster.rdb** : c'est un brouillage additif 16 bits avec deux
masques aléatoires (« secure int » anti-cheat mémoire). Fonctions : encodage FUN_0064a330, ré-encodage
FUN_00941960, lecture ContentID FUN_00953c80 / InnContentID FUN_009544c0, décodage inline (0x6aa471, 0x6aa633,
0x64fb2c). Le ctor des données créature (FUN_00940a30) range la même clé dans ContentID et InnContentID.

Structure : 4 × u16 LE `w0, w1, w2, w3`.
```rust
fn decode(w: [u16; 4]) -> u32 {                      // client
    let lo = w[3].wrapping_add(w[0].wrapping_add(w[2]).wrapping_mul(2));
    let hi = w[1].wrapping_add(w[0].wrapping_sub(w[2]).wrapping_mul(2));
    (lo as u32) | ((hi as u32) << 16)
}
fn encode(id: u32, r1: u16, r2: u16) -> [u16; 4] {  // serveur (r1, r2 quelconques, 0 autorisé)
    let (lo, hi) = (id as u16, (id >> 16) as u16);
    [r1, hi.wrapping_sub(r1.wrapping_sub(r2).wrapping_mul(2)),
     r2, lo.wrapping_sub(r1.wrapping_add(r2).wrapping_mul(2))]
}
```
Vecteurs de test (octets dans l'ordre du paquet, calculés par script Node) :
- id 100001 (0x000186A1), r1 = r2 = 0 → `00 00 01 00 00 00 a1 86` (forme la plus simple : `u16 0, u16 hi, u16 0, u16 lo`)
- id 100001, r1 = 0x1234, r2 = 0x5678 → `34 12 89 88 78 56 49 b5`
- id 123125 (0x0001E0F5), r1 = 0xBEEF, r2 = 0x0042 → `ef be a7 82 42 00 93 62`

⚠ Envoyer l'id comme `u32 id + u32 0` est **faux** (décodé en `2·lo | (hi + 2·lo) << 16`).

**Valeur à encoder** : le client compare la valeur décodée **au u32 brut lu à l'offset 0 de l'enregistrement
db_monster.rdb** (FUN_0064f990 : `uVar13 = *record`, lecture par flux générique, aucune transformation).
Aucune permutation de bits n'est appliquée côté client, ni au chargement ni dans ENTER.
→ si le fichier stocke l'id permuté, **le serveur doit envoyer cette même valeur permutée** (le champ tel qu'il est
dans le .rdb), puis l'encoder avec `encode()`. Même logique attendue pour NpcResource / ItemResource [D].
Aide au débogage : le client écrit `C:\Temp\mob_load.txt` (5 premières clés chargées, « load key(uid)=… ») et
`C:\Temp\mob_spawn.txt` (8 premiers spawns, « spawn ContentID=%d InnContentID=%d found=%d ») [O].

### 8.7 Entrée en jeu après TS_SC_LOGIN_RESULT [O sauf mention]
Machine à états de la scène de jeu : champ +0x224 (« GameRef », setter FUN_009b7620, log `!!!!! SetGameRef : %d`),
traitée dans FUN_00a47ef0 (Process), FUN_00a47720 (OnMessage), FUN_00a49470 (Render).

1. Lobby : envoi TS_CS_LOGIN, scène lobby → 0xd. SGameManager::Process (FUN_009ef310) voit 0xd :
   lance le thread `SGameLoad::LoadGame` (FUN_009e7970, préchargement), champ scène+0x228 = 6 puis 4.
2. Réception **TS_SC_LOGIN_RESULT (kind 9)** : si `result == 0`, le manager le met en cache (+0x104) et lève +0x108.
3. Quand `+0x108 && scène+0x228 == 4` : « GAMECREATE » → création de la scène de jeu, puis le LOGIN_RESULT
   en cache lui est transmis : création du joueur local (FUN_006af6d0), chargement terrain / caméra de départ
   (FUN_00a382b0, une seule fois, flag +0x2d0) et, si l'état vaut 6, **passage direct à 0xb (en jeu)**.
4. Transitions internes sans paquet serveur : 1 → 7 quand **tous les objets ENTER du monde ont fini de charger
   leur modèle** (FUN_006ade00 : obj+0x60 ou +0x11a5) ; 7 (fondu ~1 s) → 6 ou 0xb ; WARP : 2 → 3 → 9 → 4 → 5,
   puis 0xc → « WarpComplete » (FUN_00a372c0) → 7. En état 9, les messages réseau (kind < 1001 ou > 10007)
   sont **mis en file** et rejoués en état 5 (FUN_00a47720 / FUN_00a47ef0).

→ **Aucun paquet « fin de chargement » n'est attendu** : TS_SC_LOGIN_RESULT (result 0) suffit pour entrer en jeu ;
les autres paquets (STAT_INFO, PROPERTY, INVENTORY, ENTER, GAME_TIME…) sont utiles mais non bloquants [D].
Risque [D] : un objet ENTER dont le modèle ne se charge jamais peut bloquer l'état 1 (écran de chargement).

Paquets envoyés par le client pendant / après l'entrée :

| Paquet | ID | Taille | Corps | Déclencheur / source |
|---|---|---|---|---|
| TS_CS_GAME_TIME | 1100 (0x44c) | 7 | — (en-tête seul) | à la réception du LOGIN_RESULT, FUN_006cd910 [O] ; réponse attendue TS_SC_GAME_TIME (1101, kind 82) [D] |
| TS_CS_GET_REGION_INFO | 550 (0x226) | 15 | +7 `f32 x` ; +0xb `f32 y` | quand la cellule de région du joueur change (FUN_00a47ef0, log `region x: %d y: %d`) [O] |
| TS_CS_REGION_UPDATE | **67 (0x43)** (table TM_ : 7) | 24 | +7 `u32 update_time` ; +0xb `f32 x` ; +0xf `f32 y` ; +0x13 `f32 z` ; +0x17 `u8 is_stop` (= mgr+0x9c == 0) | FUN_009efff0, slot +4 de la vtable SGameManager (0xd70c0c) ; appelant indirect non résolu [O layout, D déclencheur] |
| TS_CS_UPDATE | 503 (0x1f7) | 19 | +7 `u32 handle` ; +0xb `u32 game_time` ; +0xf `u32 time()` | expiration d'états (buffs), max 1/s, FUN_0095e5c0 [O] |
| TS_TIMESYNC / TS_CS_CHECK_ILLEGAL_USER | 2 / 57 | 11 / 15 | voir 5.4 | [O] |

### 8.8 TS_SC_STAT_INFO (1000, 110 octets, kind 31) [O tailles/offsets, D noms]
Parse = copie brute (parseur 0xa2fed0), stockage FUN_00940f90 case 0x1f : `pkt+0x0b..+0x6c` → data+0x80..+0xe1.
L'IHM fiche perso (FUN_0089d9b0) recopie séparément data+0x90..+0xe1 (attributs) et lit les champs aux largeurs
ci-dessous (i32 utilisés tels quels, i16 via `(short)`) ; getter FUN_00940db0 : `+0x47 / 100` et `× 12 / 100` (portée).

| Offset | Type | Champ | Offset | Type | Champ |
|---|---|---|---|---|---|
| +0x07 | u32 | handle | +0x3d | i16 | avoid |
| +0x0b | i16 | stat_id | +0x3f | i16 | magic_avoid |
| +0x0d | i16 | strength | +0x41 | i16 | block_chance |
| +0x0f | i16 | vital | +0x43 | i16 | move_speed |
| +0x11 | i16 | dexterity | +0x45 | i16 | attack_speed |
| +0x13 | i16 | agility | +0x47 | i16 | attack_range [O getter] |
| +0x15 | i16 | intelligence | +0x49 | i16 | max_weight |
| +0x17 | i16 | mentality | +0x4b | i16 | casting_speed |
| +0x19 | i16 | luck | +0x4d | i16 | cool_time_speed |
| +0x1b | i16 | critical | +0x4f | i16 | item_chance |
| +0x1d | i16 | critical_power | +0x51 | i16 | hp_regen_percentage |
| **+0x1f** | **i32** | attack_point_right | +0x53 | i16 | hp_regen_point |
| **+0x23** | **i32** | attack_point_left | +0x55 | i16 | mp_regen_percentage |
| **+0x27** | **i32** | defence | +0x57 | i16 | mp_regen_point |
| **+0x2b** | **i32** | block_defence | +0x59 | i16 | perfect_block |
| **+0x2f** | **i32** | magic_point | +0x5b | i16 | magic_penetration |
| **+0x33** | **i32** | magic_defence | +0x5d | i16 | physical_penetration |
| +0x37 | i16 | accuracy_right | +0x5f..+0x6c | 14 o. | inconnus (7 × i16 ?) [?] — mettre 0 |
| +0x39 | i16 | accuracy_left | **+0x6d** | **u8** | **type** (0 = total, 1 = bonus/équipement) [D] |
| +0x3b | i16 | magic_accuracy | | | |

Les six champs i32 (+0x1f..+0x36) sont confirmés par la largeur des lectures de l'IHM (`static_basic01_value` =
attaque droite, `static_basic01_left_value` = attaque gauche, `basic03` = défense, `basic08` = magie,
`basic10` = défense magique, `basic14` = block_defence) ; les i16 par les lectures `(short)`. Les noms suivent
l'ordre rzemu / epic 9 ; l'ordre des 8 stats (+0x0b..+0x1a, 16 o. copiés en bloc) n'est pas vérifié individuellement.

### 8.9 Adresses ajoutées
| Adresse | Rôle |
|---|---|
| 0x6b2130 / 0x6aec50 | handler monde (switch kind) / ENTER (aiguillage type/obj_type) |
| 0x6aa290 / 0x6aa570 / 0x6aa0d0 / 0x6ab010 | ENTER monstre / NPC / item / joueur |
| 0x940a30 / 0x940f90 | ctor « creature data » (ids sécurisés) / son OnMessage (LOGIN_RESULT, ENTER, STAT_INFO) |
| 0x64a330 / 0x941960 / 0x953c80 / 0x9544c0 | encode EncodedId / ré-encodage / get ContentID / get InnContentID |
| 0x64f990 | chargement db_monster.rdb (+ log C:\Temp\mob_load.txt) |
| 0x9b7620 | SetGameRef (état scène +0x224) |
| 0xa47ef0 / 0xa47720 / 0xa49470 | scène de jeu : Process / OnMessage / Render |
| 0xa382b0 / 0xa387e0 / 0xa372c0 | LOGIN_RESULT (→ 0xb) / WARP (→ 2) / WarpComplete (→ 7) |
| 0x9efff0 / 0x6cd910 / 0x95e5c0 | envoi TS_CS_REGION_UPDATE / TS_CS_GAME_TIME / TS_CS_UPDATE |
| 0x89d9b0 | IHM fiche personnage (lecture des attributs STAT_INFO) |

## 9. Gameplay (inventaire, PNJ, personnage, combat, divers)

Analyse statique (Ghidra via REA, même binaire). Offsets = **offsets paquet** (en-tête inclus).
Implémentation : `shared/rz-proto/src/msg_game.rs` (tests de taille et d'aller-retour).
Méthode : parseur S→C FUN_00a2fed0 (case id → objet `SMSG_*`, « kind » à SMSG+4), consommateurs
FUN_006d2e60 (gestionnaire jeu, switch sur kind) / FUN_006b2130 (monde) / FUN_00a47720 (scène) ;
messages C→S trouvés par balayage des constructions d'en-tête (`mov eax, id` + `mov word [x+4], ax`,
taille à `[x]` ; scripts Node `client-data/proto-re/scan2b.js`), puis décompilation. Envoi joueur :
`(*this->vtbl+0xc4)(pkt)` ; envoi gestionnaire : `(*mgr[0xa4]->vtbl+0xc4)(pkt)`.

### 9.1 Table TM_ (FUN_00a1f9f0) [O]
263 noms extraits (`client-data/proto-re/tm.txt`). Pour le gameplay, la table concorde avec les
constructeurs/parseur (100–1202, 3000–3004, 9000/9001…), hors ids déjà connus (LOGIN, MOVE,
REGION_UPDATE, LOGIN_RESULT) ; **TS_SC_ERASE_ITEM 209** existe dans le parseur mais pas dans la table.

### 9.2 Structure d'objet TS_ITEM_INFO (331 o. = 0x14b) [O taille]
Offsets relatifs à l'enregistrement. [O] = lu par FUN_006cd320 / FUN_006e3640 ou initialisé par le
ctor FUN_0070e980 ; le reste suit l'ordre epic 9.5 [D].

| Off. | Type | Champ | Off. | Type | Champ |
|---|---|---|---|---|---|
| 0x00 | u32 | handle [O] | 0x60 | u32[10] | random_type [O bloc] |
| 0x04 | u32 | code ItemResource, **en clair** [O] | 0x88 | i64[10] | random_value_1 [O bloc] |
| 0x08 | i64 | uid [D] | 0xd8 | i64[10] | random_value_2 [O bloc] |
| 0x10 | i64 | count [O] | 0x128 | i32 | remain_time (> 0 : liste « durée limitée ») [O] |
| 0x18 | u32 | ethereal_durability [D] | 0x12c | u8 | elemental_effect_type [D] |
| 0x1c | u32 | endurance (÷ 100 000 côté client) [O usage] | 0x12d | i32 ×3 | elemental remain_time / attack / magic [D] |
| 0x20 | u8 | enhance [D] | 0x139 | i32 | appearance_code [D] |
| 0x21 | u8 | level [D] | 0x13d | u32 | summon_code [D] |
| 0x22 | u16 | enchant [D] | **0x141** | i16 | **wear_position** (-1 sac ; -2 traité à part) [O] |
| 0x24 | u32 | flag (copié en item+0x164) [O] | 0x143 | u32 | own_summon_handle (porteur ≠ joueur) [O] |
| 0x28 | i32[4] | socket [O] | 0x147 | i32 | index [D] |
| 0x38 | u32[5] + i32[5] | awaken value / data [O bloc] | | | |

### 9.3 Messages

| Paquet | ID | Dir | Taille | Corps | Kind / fonction |
|---|---|---|---|---|---|
| TS_SC_INVENTORY | 207 | S→C | 9+331n | +7 u16 count ; +9 ITEM_INFO[n] | 0x16, FUN_006cd320 [O] |
| TS_SC_WEAR_INFO | 202 | S→C | 487 | +7 handle ; +0xb i32 wear[28] ; +0x7b i32 enhance[28] ; +0xeb i32 level[28] ; +0x15b u8 elem[28] ; +0x177 i32 appearance[28] | 0x11 [O] |
| TS_SC_ITEM_WEAR_INFO | 287 | S→C | 26 | +7 item_handle ; +0xb i16 wear_position ; +0xd target_handle ; +0x11 i32 enhance [D, non lu] ; +0x15 u8 elem [D] ; +0x16 i32 appearance [D] | 0x12, FUN_00711120 [O] |
| TS_CS_PUTON_ITEM | 200 | C→S | 16 | +7 u8 position ; +8 item_handle ; +0xc target_handle | FUN_006d7e30 [O] |
| TS_CS_PUTOFF_ITEM | 201 | C→S | 12 | +7 u8 position ; +8 target_handle | FUN_006d7e30 [O] |
| TS_CS_DROP_ITEM | 203 | C→S | 15 | +7 item_handle ; +0xb u32 count | FUN_006d7cc0 [O] |
| TS_CS_TAKE_ITEM | 204 | C→S | 15 | +7 taker (soi) ; +0xb item_handle | FUN_0096bb80 [O] |
| TS_SC_DROP_RESULT | 205 | S→C | 12 | +7 item_handle ; +0xb u8 success | 0x13 [O] |
| TS_SC_TAKE_ITEM_RESULT | 210 | S→C | 15 | +7 item_handle ; +0xb taker | 0x49 [O] |
| TS_CS_ERASE_ITEM | 208 | C→S | 8+13n | +7 u8 n ; {u32 handle, i64 count, u8}[n] [ordre D] | FUN_006d7bf0 [O taille] |
| TS_SC_ERASE_ITEM | 209 | S→C | 8+12n | +7 u8 n ; {u32 handle, i64 count}[n] [D] | 0xaa [O taille] |
| TS_CS_CHANGE_ITEM_POSITION | 218 | C→S | 16 | +7 u8 is_storage ; +8 handle1 ; +0xc handle2 [D] | [O taille] |
| TS_CS_USE_ITEM | 253 | C→S | 47 | +7 item_handle ; +0xb target ; +0xf char param[32] | FUN_0096f9f0 [O] |
| TS_SC_USE_ITEM_RESULT | 283 | S→C | 15 | +7 item_handle ; +0xb target [D noms] | 0x1a [O] |
| TS_SC_DESTROY_ITEM | 254 | S→C | 11 | +7 item_handle | 0x17 [O] |
| TS_SC_UPDATE_ITEM_COUNT | 255 | S→C | 19 | +7 item_handle ; +0xb i64 count | 0x18, FUN_0070e6f0 [O] |
| TS_SC_ITEM_DROP_INFO | 282 | S→C | 15 | +7 monster ; +0xb item [D noms] | 0x14 [O] |
| TS_SC_BELT_SLOT_INFO | 216 | S→C | 39 | +7 u32 handle[8] | 0x55 [O] |
| TS_SC_ITEM_COOL_TIME | 217 | S→C | 167 | +7 i32[40] | 0x57 [O] |
| TS_SC_GOLD_UPDATE | 1001 | S→C | 19 | +7 i64 gold ; +0xf i32 chaos [D] | 0x20 [O taille] |
| TS_CS_CONTACT | 3002 | C→S | 11 | +7 npc_handle | FUN_006d7dd0 [O] |
| TS_SC_DIALOG | 3000 | S→C | 21+t+x+m | +7 i32 type ; +0xb npc_handle ; +0xf u16 title_len ; +0x11 u16 text_len ; +0x13 u16 menu_len ; +0x15 title, text, menu (sans NUL) | SMSG_NPC_DIALOG, FUN_00a25670 [O] |
| TS_CS_DIALOG | 3001 | C→S | 9+n | +7 u16 n ; +9 trigger[n] | mgr kind 0x40f [O] |
| TS_SC_GENERAL_MESSAGE_BOX | 3004 | S→C | 9+n | +7 u16 n ; +9 texte | 0xb5 [O] |
| TS_SC_MARKET | 250 | S→C | 13+20n | +7 npc_handle ; +0xb u16 n ; {i32 code, i64 price, i32 huntaholic, i32 arena}[n] [D champs] | 0x3c [O] |
| TS_CS_BUY_ITEM | 251 | C→S | 13 | +7 i32 item_code ; +0xb u16 count | mgr kind 0x416 [O] |
| TS_CS_SELL_ITEM | 252 | C→S | 19 | +7 item_handle ; +0xb i64 count | mgr kind 0x418 [O] |
| TS_SC_LEVEL_UPDATE | 1002 | S→C | 19 | +7 handle ; +0xb i32 level ; +0xf i32 job_level | 0x21 [O] |
| TS_SC_EXP_UPDATE | 1003 | S→C | 27 | +7 handle ; +0xb i64 exp ; +0x13 i64 jp [D] | 0x22 [O taille] |
| TS_SC_HPMP | 509 | S→C | 36 | +7 handle ; +0xb add_hp ; +0xf hp ; +0x13 max_hp ; +0x17 add_mp ; +0x1b mp ; +0x1f max_mp ; +0x23 u8 display [D noms] | 0x37 [O] |
| TS_SC_REGEN_HPMP | 516 | S→C | 27 | +7 handle ; 4 × i32 hp_regen, mp_regen, hp, mp [D noms] | 0x56 [O] |
| TS_SC_STATUS_CHANGE | 500 | S→C | 15 | +7 handle ; +0xb u32 status | 0x31 [O] |
| TS_SC_STATE | 505 | S→C | 31 | +7 handle ; +0xb u16 state_handle ; +0xd u16 level ; +0xf u32 code ; +0x13 i32 remain (-1 = permanent) ; +0x17 u32 start ; +0x1b i32 value | 0x34 [O] |
| TS_SC_SP / TS_SC_ENERGY | 514 / 515 | S→C | 15 / 13 | +7 handle ; +0xb i16 sp ; +0xd i16 max_sp / +0xb i16 energy | 0x48 / 0x54 [O] |
| TS_SC_SKILL_LIST | 403 | S→C | 14+14n | +7 target ; +0xb u16 n ; +0xd u8 modification_type ; {i32 id, u8 base_lv, u8 cur_lv, u32 total_cool, u32 remain_cool}[n] | 0x2d, FUN_00a2ae90 [O] |
| TS_SC_ADDED_SKILL_LIST | 404 | S→C | 13+6n | +7 target ; +0xb u16 n ; {i32 id, u8, u8}[n] | [O] (non implémenté) |
| TS_SC_QUEST_LIST | 600 | S→C | 11+61a+8p | +7 u16 a ; +9 u16 p ; {i32 code, i32 start_id, i32 value[6], i32 status[6], u8 progress, i32 time_limit}[a] ; {i32 code, i32 start_id}[p] | FUN_00a29430 [O] |
| TS_CS_ATTACK_REQUEST | 100 | C→S | 15 | +7 handle (soi) ; +0xb target | ctor FUN_00969de0, envoi FUN_0096b6c0 [O] |
| TS_SC_ATTACK_EVENT | 101 | S→C | 22+61n | +7 attacker ; +0xb target ; +0xf u16 speed ; +0x11 u16 delay ; +0x13 u8 action ; +0x14 u8 flag ; +0x15 u8 n ; ATTACK_INFO[n] (61 o., champs [D]) | 0xc, FUN_006ae2b0 [O] |
| TS_SC_CANT_ATTACK | 102 | S→C | 19 | +7 attacker ; +0xb target ; +0xf i32 reason | 0xe [O] |
| TS_CS_CANCEL_ACTION | 150 | C→S | 11 | +7 handle (soi) | FUN_0096b7f0 [O] |
| TS_CS_SKILL | 400 | C→S | 31 | +7 u16 skill_id ; +9 caster ; +0xd target ; +0x11 x, y, z f32 ; +0x1d u8 layer ; +0x1e u8 level | FUN_0096b840 [O] |
| TS_SC_SKILL | 401 | S→C | 48+… | +7 u16 id ; +9 u8 lv ; +0xa caster ; +0xe target ; +0x12 x, y, z ; +0x1e u8 layer ; +0x1f u8 type (0 = FIRE) ; +0x20 hp_cost ; +0x24 mp_cost ; +0x28 caster_hp ; +0x2c caster_mp ; FIRE : +0x30 u8 multiple ; +0x31 f32 range ; +0x35 u8 target_count ; +0x36 u8 fire_count ; +0x37 u16 n ; +0x39 hits | SMSG_SKILL_EVENT, FUN_00a2a7f0 [O] |
| TS_CS_LEARN_SKILL | 402 | C→S | 21 | +7, +0xb, +0xf u32 ; +0x13 u16 [D] | FUN_006d8010 [O taille] |
| TS_CS_TARGETING | 511 | C→S | 11 | +7 target | ctor FUN_006c4d00 [O] |
| TS_SC_TARGET | 512 | S→C | 11 | +7 handle | 0x38 [O] |
| TS_CS_RESURRECTION | 513 | C→S | 12 | +7 u32 handle ; +0xb u8 type (0..3) | mgr kind 0x47b [O taille] |
| TS_SC_DEAD | 504 | S→C | — | **ignoré par le parseur** (`id == 0x1f8` → saut) | [O] |
| TS_SC_EMOTION / TS_CS_EMOTION | 1201 / 1202 | S→C / C→S | 15 / 11 | +7 handle ; +0xb i32 emotion / +7 i32 emotion | 0x51 [O] |
| TS_CS_QUERY | 13 | C→S | 11 | +7 handle inconnu | ctor FUN_006a7b90 [O] |
| TS_CS_GAME_TIME / TS_SC_GAME_TIME | 1100 / 1101 | C→S / S→C | 7 / 15 | — / +7 u32 t ; +0xb u32 game_time (stockés par la scène) | 0x52 [O] |
| TS_SC_URL_LIST | 9001 | S→C | 9+n | +7 u16 n ; +9 « clé, valeur, clé, valeur… » séparés par des barres verticales → environnement client | [O] |
| TS_SC_OPEN_URL | 9000 | S→C | 18+n | +7 u8 [D] ; +8 u32 width ; +0xc u32 height ; +0x10 u16 n ; +0x12 url | kind 100 [O] |

Hits de TS_SC_SKILL (FIRE), taille de chaque entrée selon l'octet `type` en tête [O] : 0/1 DAMAGE → 32,
2 → 45, 10 → 10, 20/21 → 13, 22 → 25, 23 → 23, 30 → 19, 40/41 → 36, 42 → 17. Entrée DAMAGE [D] :
`u8 type, u32 target, i32 target_hp, u8 damage_type, i32 damage, i32 flag, u16 elemental[7]`.

### 9.4 Comportements notables [O]
- **TS_SC_DIALOG** : menus lus seulement pour `type` 0, 3, 10 ; menu découpé sur `'\t'`
  (`"\t<texte>\t<trigger>"` répété, appariement [D]) ; texte commençant par `QUEST|` = dialogue de quête
  (mots-clés START / REJECT / OK / END / REWARD) ; couleurs `<#RRGGBB>` ; `V%8.8d.mp3` joué si un id de voix est présent.
- **TS_SC_ATTACK_EVENT** : si l'attaquant ou la cible est inconnu, le client envoie **TS_CS_QUERY (13)** pour ce
  handle (FUN_006ae2b0) — le serveur doit répondre par un TS_SC_ENTER.
- **TS_SC_INVENTORY** : objets indexés par `handle` ; `wear_position` ≥ 0 (< 0x25) les place dans l'équipement ;
  `own_summon_handle` ≠ 0 les attribue à une invocation.
- Les envois déclenchés par l'IHM passent par FUN_006d2e60 (kinds SIMSG_* ≥ 0x400).

### 9.5 Séquences attendues [D sauf ids/tailles]
- **Inventaire après connexion** : S→C LOGIN_RESULT → STAT_INFO → **INVENTORY (207)** (tous les objets, portés
  inclus, `wear_position` renseigné) → **WEAR_INFO (202)** du joueur → GOLD_UPDATE (1001) → BELT_SLOT_INFO (216)
  → SKILL_LIST (403) → QUEST_LIST (600). L'ouverture de la fenêtre est locale (aucun paquet).
- **Équiper** : C→S PUTON_ITEM (200) → S→C ITEM_WEAR_INFO (287, nouvelle position) + WEAR_INFO (202, diffusé)
  + STAT_INFO (1000). Déséquiper : PUTOFF_ITEM (201) → ITEM_WEAR_INFO (position -1) + WEAR_INFO.
- **PNJ** : C→S CONTACT (3002) → S→C DIALOG (3000, menu) → C→S DIALOG (3001, trigger choisi) → S→C MARKET (250)
  → C→S BUY_ITEM (251) / SELL_ITEM (252) → S→C INVENTORY (objet ajouté) ou UPDATE_ITEM_COUNT (255) /
  DESTROY_ITEM (254) + GOLD_UPDATE (1001) ; TS_SC_RESULT (0) en cas d'erreur.
- **Attaque** : C→S TARGETING (511) → ATTACK_REQUEST (100) → S→C ATTACK_EVENT (101) répétés + HPMP (509) ;
  arrêt : CANCEL_ACTION (150) ; refus : CANT_ATTACK (102). Mort : TS_SC_DEAD étant ignoré, envoyer HPMP (hp 0)
  / STATUS_CHANGE puis LEAVE (9) ; butin : ENTER item (§8.5) + ITEM_DROP_INFO (282) ; ramassage TAKE_ITEM (204)
  → TAKE_ITEM_RESULT (210) + INVENTORY.
- **Compétence** : C→S SKILL (400) → S→C SKILL (401, type CASTING si incantation) puis SKILL (401, FIRE + hits)
  + HPMP (509) ; recharge via SKILL_LIST (403).

### 9.6 Limites
- ATTACK_INFO (61 o.) : seule la taille est observée ; le découpage proposé (i32 elemental[7], i64 hp) est [D].
- EXP_UPDATE (i64 exp / i64 jp), GOLD_UPDATE (chaos), ERASE_ITEM (ordre), LEARN_SKILL, SHOW_WINDOW (3003),
  QUEST_STATUS (601) : [D] ou non disséqués. Aucun paquet « quick slot » dans le parseur (raccourcis
  probablement stockés via TS_SC_PROPERTY [D]).
- Les noms de champs [D] suivent l'ordre public epic 9.5 ; les offsets marqués [O] font foi.

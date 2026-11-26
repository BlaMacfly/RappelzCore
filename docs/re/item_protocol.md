# Utilisation des objets — client Rappelz NoLimits (SFrame.exe)

Analyse **statique** (Ghidra 12.1.4 via REA) de `Rappelz NoLimits\SFrame.exe`, PE x86,
SHA-256 `27f42fc8f6b08df8c239640b938d601aeaebb7b3db0c89fc47755325a98a410d`. SFrame.exe n'a jamais été exécuté.
Conventions de `summon_protocol.md` : offsets paquet (en-tête de 7 octets inclus), `[O]` lu, `[D]` déduit, `[?]` inconnu.
Sauf mention, les constats viennent d'un sous-agent ; le point relu est signalé avec son Evidence ID.

## 1. Paquets

| Paquet | ID | Dir | Taille | Corps |
|---|---|---|---|---|
| TS_CS_USE_ITEM | 253 | C→S | 47 | +7 u32 objet ; +0xb u32 cible (**0** pour une potion) ; +0xf char[32] paramètre (zéros) |
| TS_SC_USE_ITEM_RESULT | 283 | S→C | 15 | +7 objet ; +0xb cible [D noms] ; **aucun traitement trouvé** dans le gestionnaire de jeu |
| TS_SC_RESULT, request 253 | 0 | S→C | 15 | seul le code 0x54 affiche un texte (9245, « donjon ») ; tout autre code est silencieux |
| TS_SC_ITEM_COOL_TIME | 217 | S→C | 167 | +7 i32[40] : **indice = cool_time_group − 1**, valeur = reste en centièmes |
| TS_SC_UPDATE_ITEM_COUNT / TS_SC_DESTROY_ITEM | 255 / 254 | S→C | 19 / 11 | nouveau total de la pile / retrait |

## 2. Côté client

- Chemin : UseOrEquipItem FUN_0079b9a0 → FUN_006c9500 → FUN_006e6420 (type 3 seulement) → FUN_006d15f0 → envoi direct de 253 pour
  les potions (`opt_type_0` 1 ou 2, ou sans compétence) ; sinon une entrée SInputUseItem passe par la machine du joueur.
- Refus locaux : fenêtres ouvertes, boutique personnelle ouverte, compétence d'objet (5) en monture, objet interdit assis (bit 18 de
  `item_use_flag`). **Ni niveau, ni recharge, ni mort ne sont contrôlés** : le serveur doit tout faire. Aucun verrou n'attend la réponse.
- Bit 7 de `item_use_flag` : utilisation ciblée par la compétence `opt_var1_0`. « Sur une créature » (act 4) envoie un équipement (200),
  pas 253.
- **Recharges** : le client **n'en démarre aucune seul**. FUN_006e6fa0 (relu, `ev_0503b03ba3b973460b4a13028ffeacfbff688920425119f47d39f82c2c86de83`)
  n'accepte que les indices 0..39 et **n'écrit que si l'emplacement est libre** : un 217 n'écourte ni ne prolonge une recharge en cours.
- Effets d'`opt_type` (données et info-bulles) : 1 « Restaure N HP » (var1), 2 « N MP », 5 compétence var1 au niveau var2, 6 état de
  l'objet (`state_id`, `state_level`, `state_time` s), 94 paquet de var2 exemplaires de l'objet var1. Les autres (82, 101–104, 122 coloration…)
  ne sont pas établis ; aucune fonction de texte par `opt_type` n'a été trouvée.

## 3. Implémentation RappelzCore (2026-11-27)

| Élément | Choix |
|---|---|
| Objets | `rz_rappelz::consumables` : type 3, `base_type` nuls, effets 1, 2, 6 et 94 uniquement ; **1 398** définis, **849** utilisables (états connus et sans malus) |
| Contrôles serveur | objet du sac, sur soi (cible 0 ou soi), niveau `use_min_level`..`use_max_level`, vivant, recharge, état connu, contenu existant ; refusé pendant un échange |
| Ordre | contenu d'un paquet créé d'abord (sac plein : rien n'est consommé), puis un exemplaire retiré de **la pile utilisée** (SQL puis 255 ou 254), puis effets |
| Soins | instantanés, plafonnés ; 509 avec les gains à afficher |
| Recharges | serveur, par (personnage, groupe) ; groupe 0 : propre à l'objet, non affichée ; 217 complet après chaque utilisation d'un groupe 1–40 et à la connexion ; en mémoire seulement (perdues à l'arrêt) |
| Réponse | 283 si l'objet a servi (sans effet connu chez le client) ; sinon TS_SC_RESULT(253, 6) et un message serveur |

Validation (MariaDB de test isolée, `rz-bot --use-item`) : Potion Rouge Niv.1 → PV 300 → 450, pile 3 → 2, recharge du groupe 1 (1 000 cs),
seconde utilisation 2 s plus tard refusée ; Potion Rouge Niv.4 refusée au niveau 20 (niveau 60 requis) ; Potion Bleue Niv.1 → PM 50 → 200 ;
paquet de flèches → 1 000 flèches, paquet retiré ; recharge restante annoncée à la reconnexion. Tests unitaires : soin plafonné, mort,
indices de 217.
**Non vérifié : client graphique d'origine** (affichage de la recharge sur l'icône, réaction à 283). Pas encore : compétences d'objet
(type 5), utilisation sur une cible, colorations et autres types.

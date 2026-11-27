# Mort et résurrection du joueur — client Rappelz NoLimits (SFrame.exe)

Analyse **statique** (Ghidra 12.1.4 via REA) de `Rappelz NoLimits\SFrame.exe`, SHA-256
`27f42fc8f6b08df8c239640b938d601aeaebb7b3db0c89fc47755325a98a410d`. SFrame.exe n'a jamais été exécuté. Conventions de
`summon_protocol.md`. Constats d'un sous-agent ; le point relu sur le binaire est signalé avec son Evidence ID.

## 1. Côté client

- **Mort** : seuls les PV finaux < 1 la déclenchent (509, ATTACK_EVENT ou FIRE, via la file PV/PM) : mouvement 0x16, drapeau
  objet+0x630. TS_SC_DEAD (504) est ignoré ; TS_SC_STATUS_CHANGE (500) n'a pas de bit « mort » (seuls 0x1, 0x2, 0x800 sont lus). Un
  ENTER ou un LOGIN aux PV nuls montre le personnage déjà mort.
- **Fenêtre** : 5 s après la mort (FUN_0096e560), choix selon les objets possédés (2010454 > 2902042 > 910005, puis 910004) et un état
  d'effect_type 109 : point de retour seul, + objet, + état. Dans une zone particulière (FUN_009558a0), compte à rebours de 10 s
  (texte 9202) puis envoi **automatique** du type 4.
- **TS_CS_RESURRECTION (513)**, 12 octets, construit en 0x6d624a (relu, `ev_d86d386021d4f9290bc3fc61445f6ab3c29e28ee5599f90c6aa84697c5cca4d2`) :
  +7 u32 handle, +0xb u8 type.

| Type | Origine | handle |
|---|---|---|
| 0 | « point de retour », ou réponse à la boîte simple | 0 |
| 1 | « état » (type 109) | le joueur lui-même |
| 2 | « objet » — le code de l'objet **n'est pas envoyé** | 0 |
| 3 | fenêtre de combat entre joueurs | 0 |
| 4 | fin du compte à rebours, sans clic | 0 |

- **Retour à la vie** : un 509 (ou des PV finaux) **positifs** relève le personnage et ferme les fenêtres ; aucun autre chemin trouvé.
  Les autres joueurs voient la même chose, sans LEAVE ni ENTER.
- **Perte d'EXP** : un EXP_UPDATE dont l'EXP baisse affiche 206 « Vous avez perdu … EXP ». Montant et conditions (6823 « pas perdu
  d'expérience ») non établis.

## 2. Implémentation RappelzCore (2026-11-27)

| Élément | Choix |
|---|---|
| Mort | le joueur **reste mort** (PV 0) ; plus d'attaque, de déplacement, de compétence ni d'objet ; les états tombent |
| 513 | handle 0 ou le joueur ; type 2 : premier objet possédé dans l'ordre du client, consommé, résurrection **sur place** ; types 0, 1 (aucun état 109 pris en charge), 3 et 4, ou type 2 sans objet : point de retour de la race |
| Résurrection | PV et PM pleins **[D]** ; 509 à tous ceux qui voient le joueur, puis WARP pour le point de retour |
| Client sans fenêtre | `Rappelz.AutoResurrectSeconds` (60 par défaut, 0 = jamais) : point de retour d'office ; RappelzRust n'envoie pas 513 |
| Perte d'EXP | **aucune** : règle inconnue |

Validation (MariaDB de test isolée, `rz-bot --attack --resurrect`) : personnage à 1 PV tué par le monstre, reste mort, 513 type 0 →
1 980 PV au point de retour ; type 2 avec l'objet 910005 → ressuscité sur place, objet consommé ; sans demande → point de retour
au bout de 12 s (réglage de l'essai). Test unitaire : mort durable, résurrection d'office une seule fois, sur place.
**Non vérifié : client graphique d'origine.**

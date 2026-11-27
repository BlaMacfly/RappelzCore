//! Groupe du client SFrame : il n'a aucun paquet dédié et passe par le chat
//! (docs/re/party_protocol.md).
//!
//! - Serveur → client : TS_SC_CHAT (22), expéditeur `@PARTY`, type 100, texte `COMMANDE|champ|…`.
//! - Client → serveur : TS_CS_CHAT_REQUEST (20) de type 0, commandes `/pcreate`, `/pinvite`,
//!   `/pjoin`… et leurs variantes `/party_…` des boîtes de confirmation.

use crate::msg::ScChat;

/// Expéditeur masqué par l'affichage du chat pour le type 100 (FUN_0074bd30).
pub const SENDER: &str = "@PARTY";
/// Type de chat aiguillé vers le gestionnaire de groupe (FUN_006cac50, `cmp 0x64`).
pub const CHAT_TYPE: u8 = 100;
/// La fenêtre de groupe affiche au plus 8 membres (FUN_008d4760).
pub const MAX_MEMBERS: usize = 8;
/// Longueur maximale retenue pour un nom de groupe (choix du serveur).
pub const MAX_NAME: usize = 20;

/// Message système de groupe prêt à envoyer.
pub fn system(text: String) -> ScChat {
    ScChat { sender: SENDER.into(), chat_type: CHAT_TYPE, message: text }
}

/// Un champ ne doit jamais contenir le séparateur `|`.
fn field(s: &str) -> String {
    s.replace('|', "")
}

/// Nom de groupe acceptable : 1 à 20 caractères imprimables, sans `|`.
pub fn valid_name(name: &str) -> bool {
    let n = name.chars().count();
    (1..=MAX_NAME).contains(&n) && !name.contains('|') && name.chars().all(|c| !c.is_control())
}

/// Bloc membre de PINFO et MINFO : 9 champs (FUN_006fa0e0, FUN_006fd680).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct MemberLine {
    /// 0 = hors ligne.
    pub handle: u32,
    pub name: String,
    pub level: i32,
    pub job: i32,
    /// Champs +0x58 et +0x5c du membre, sens déduit : PV et PM en pourcentage.
    pub hp_percent: i32,
    pub mp_percent: i32,
    pub x: i32,
    pub y: i32,
}

impl MemberLine {
    fn push(&self, out: &mut Vec<String>) {
        out.extend([
            self.handle.to_string(),
            field(&self.name),
            self.level.to_string(),
            self.job.to_string(),
            self.hp_percent.to_string(),
            self.mp_percent.to_string(),
            self.x.to_string(),
            self.y.to_string(),
            // Champ +0x68 : sens inconnu.
            "0".into(),
        ]);
    }
}

/// `CREATE|groupe|chef|type` ; type 0 = groupe normal.
pub fn create(party: &str, leader: &str) -> String {
    format!("CREATE|{}|{}|0", field(party), field(leader))
}

/// `PINFO|id|groupe|chef|partage|A|B|type` puis un bloc de 9 champs par membre.
/// A et B sont inconnus (envoyés à 0).
pub fn pinfo(id: u32, party: &str, leader: &str, share: u8, members: &[MemberLine]) -> String {
    let mut out = vec![
        "PINFO".to_string(),
        id.to_string(),
        field(party),
        field(leader),
        share.to_string(),
        "0".into(),
        "0".into(),
        "0".into(),
    ];
    for m in members {
        m.push(&mut out);
    }
    out.join("|")
}

/// `INVITE|inviteur|groupe|id|clé` ; la réponse du client est `/pjoin id clé` ou `/pjoin id`.
pub fn invite(inviter: &str, party: &str, id: u32, key: u32) -> String {
    format!("INVITE|{}|{}|{id}|{key}", field(inviter), field(party))
}

/// `JOIN|groupe|membre`, aux membres déjà présents.
pub fn join(party: &str, member: &str) -> String {
    format!("JOIN|{}|{}", field(party), field(member))
}

/// `LEAVE|membre|0`, aux membres restants (1 afficherait le texte 0x984).
pub fn leave(member: &str) -> String {
    format!("LEAVE|{}|0", field(member))
}

/// `KICK|groupe|membre`, aux membres restants.
pub fn kick(party: &str, member: &str) -> String {
    format!("KICK|{}|{}", field(party), field(member))
}

/// `DESTROY|groupe|0` : le client ferme sa fenêtre de groupe.
pub fn destroy(party: &str) -> String {
    format!("DESTROY|{}|0", field(party))
}

/// `PROMOTE|groupe|nouveau_chef` (le client lit le 3e champ).
pub fn promote(party: &str, leader: &str) -> String {
    format!("PROMOTE|{}|{}", field(party), field(leader))
}

/// `LOGIN|groupe|membre`.
pub fn login(party: &str, member: &str) -> String {
    format!("LOGIN|{}|{}", field(party), field(member))
}

/// `LOGOUT|membre`.
pub fn logout(member: &str) -> String {
    format!("LOGOUT|{}", field(member))
}

/// `CHANGE_LEVEL|membre|niveau`.
pub fn change_level(member: &str, level: i32) -> String {
    format!("CHANGE_LEVEL|{}|{level}", field(member))
}

/// `MODE|n` : mode de partage du butin.
pub fn mode(share: u8) -> String {
    format!("MODE|{share}")
}

/// `PPOS|handle|x|y|…` : positions des membres en ligne.
pub fn ppos(points: &[(u32, i32, i32)]) -> String {
    let mut out = String::from("PPOS");
    for (h, x, y) in points {
        out.push_str(&format!("|{h}|{x}|{y}"));
    }
    out
}

/// Commande de groupe envoyée par le client.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Command {
    Create(String),
    Invite(String),
    /// Réponse à INVITE : `key` absent = refus.
    Join { party: u32, key: Option<u32> },
    Kick(String),
    Leave,
    Destroy,
    Promote(String),
    Share(u8),
}

impl Command {
    /// Reconnaît une commande de groupe ; `None` pour tout autre texte.
    pub fn parse(text: &str) -> Option<Self> {
        let text = text.trim();
        let rest = text.strip_prefix('/')?;
        let (verb, arg) = match rest.split_once(char::is_whitespace) {
            Some((v, a)) => (v, a.trim()),
            None => (rest, ""),
        };
        let name = || (!arg.is_empty()).then(|| arg.to_string());
        Some(match verb.to_ascii_lowercase().as_str() {
            "pcreate" => Self::Create(name()?),
            "pinvite" => Self::Invite(name()?),
            "pkick" | "party_kick" => Self::Kick(name()?),
            "pleave" | "party_leave" => Self::Leave,
            "pdestroy" | "party_disband" => Self::Destroy,
            "ppromote" => Self::Promote(name()?),
            "pshare" => Self::Share(arg.parse().ok()?),
            "pjoin" => {
                let mut it = arg.split_whitespace();
                let party = it.next()?.parse().ok()?;
                let key = match it.next() {
                    Some(k) => Some(k.parse().ok()?),
                    None => None,
                };
                if it.next().is_some() {
                    return None;
                }
                Self::Join { party, key }
            }
            _ => return None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Message;

    #[test]
    fn server_texts_match_the_client_field_layout() {
        let m = MemberLine { handle: 7, name: "A|b".into(), level: 20, job: 100, hp_percent: 50, mp_percent: 100, x: 8400, y: 7180 };
        let p = pinfo(3, "Fiers", "Ab", 0, &[m]);
        let fields: Vec<_> = p.split('|').collect();
        // 8 champs d'en-tête puis 9 par membre, comme l'exige FUN_006fa0e0.
        assert_eq!(fields.len(), 8 + 9);
        assert_eq!(&fields[..8], ["PINFO", "3", "Fiers", "Ab", "0", "0", "0", "0"]);
        assert_eq!(&fields[8..], ["7", "Ab", "20", "100", "50", "100", "8400", "7180", "0"]);
        assert_eq!(invite("Ab", "Fiers", 3, 99), "INVITE|Ab|Fiers|3|99");
        assert_eq!(promote("Fiers", "Cd").split('|').count(), 3);
        assert_eq!(ppos(&[(1, 2, 3), (4, 5, 6)]), "PPOS|1|2|3|4|5|6");
        let chat = system(destroy("Fiers"));
        let back = ScChat::from_packet(&chat.to_packet()).unwrap();
        assert_eq!((back.sender.as_str(), back.chat_type, back.message.as_str()), ("@PARTY", 100, "DESTROY|Fiers|0"));
    }

    #[test]
    fn client_commands_are_recognised_in_both_spellings() {
        assert_eq!(Command::parse("/pcreate Les Fiers"), Some(Command::Create("Les Fiers".into())));
        assert_eq!(Command::parse("/pinvite Rtwo"), Some(Command::Invite("Rtwo".into())));
        assert_eq!(Command::parse("/pjoin 3 99"), Some(Command::Join { party: 3, key: Some(99) }));
        assert_eq!(Command::parse("/pjoin 3"), Some(Command::Join { party: 3, key: None }));
        assert_eq!(Command::parse("/party_kick Rtwo"), Some(Command::Kick("Rtwo".into())));
        assert_eq!(Command::parse("/party_disband"), Some(Command::Destroy));
        assert_eq!(Command::parse("/party_leave"), Some(Command::Leave));
        assert_eq!(Command::parse("/pshare 2"), Some(Command::Share(2)));
        for bad in ["/pinvite", "/pjoin", "/pjoin x", "/pjoin 1 2 3", "/pshare z", "bonjour", "/gps", ".party leave"] {
            assert_eq!(Command::parse(bad), None, "{bad}");
        }
        assert!(valid_name("Les Fiers") && !valid_name("") && !valid_name("a|b") && !valid_name(&"x".repeat(21)));
    }
}

//! Groupes (docs/re/party_protocol.md). Le client SFrame les pilote par le chat : commandes
//! `/pcreate`, `/pinvite`, `/pjoin`… et messages `@PARTY` de type 100. Le client RappelzRust
//! garde son extension versionnée (`rz_proto::party`, paquets 62000/62001).
//!
//! Les groupes sont attachés aux personnages (guid) et survivent à la déconnexion, comme dans
//! AzerothCore ; ils sont enregistrés dans `groups` / `group_member`.

use std::collections::HashMap;

use anyhow::Result;
use rz_database::groups::{self as db_groups, StoredGroup};
use rz_proto::party::{PartyInvite, PartyMember, ScPartyState, VERSION};
use rz_proto::party_text::{self as text, Command, MemberLine, MAX_MEMBERS};
use rz_proto::Message;
use rz_database::sqlx::MySqlPool;

use crate::session::Session;
use crate::world::World;

pub const INVITE_SECONDS: u64 = 60;
/// Positions des membres (PPOS), en centièmes de seconde.
const POSITIONS_INTERVAL: u32 = 200;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Member {
    pub guid: u32,
    pub name: String,
    pub level: i32,
    pub job: i32,
}

#[derive(Clone, Debug)]
pub struct Group {
    pub id: u32,
    pub name: String,
    /// Guid du chef.
    pub leader: u32,
    pub members: Vec<Member>,
    pub share: u8,
}

impl Group {
    pub fn leader_name(&self) -> &str {
        self.members.iter().find(|m| m.guid == self.leader).map_or("", |m| m.name.as_str())
    }

    fn named(&self, name: &str) -> Option<&Member> {
        self.members.iter().find(|m| m.name.eq_ignore_ascii_case(name))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Invitation {
    pub party: u32,
    /// Clé renvoyée par `/pjoin id clé`.
    pub key: u32,
    pub inviter: u32,
    pub expires: u64,
}

/// Écriture en base à reporter après l'action.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Store {
    Create { id: u32, leader: u32, share: u8 },
    Join { id: u32, member: u32 },
    Leave { member: u32 },
    Leader { id: u32, leader: u32 },
    Share { id: u32, share: u8 },
    Delete { id: u32 },
}

/// Départ d'un membre.
#[derive(Clone, Debug)]
pub struct Departure {
    pub id: u32,
    pub party: String,
    pub new_leader: Option<String>,
    pub deleted: bool,
    pub stores: Vec<Store>,
}

/// Action demandée par un joueur, depuis le client SFrame ou la commande `.party`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Action {
    Create(String),
    Invite(String),
    /// `/pjoin id clé` (accepter) ou `/pjoin id` (refuser).
    Respond { party: u32, key: Option<u32> },
    Accept,
    Decline,
    Leave,
    Kick(String),
    Promote(String),
    Disband,
    Share(u8),
}

impl From<Command> for Action {
    fn from(c: Command) -> Self {
        match c {
            Command::Create(n) => Self::Create(n),
            Command::Invite(n) => Self::Invite(n),
            Command::Join { party, key } => Self::Respond { party, key },
            Command::Kick(n) => Self::Kick(n),
            Command::Leave => Self::Leave,
            Command::Destroy => Self::Disband,
            Command::Promote(n) => Self::Promote(n),
            Command::Share(s) => Self::Share(s),
        }
    }
}

impl Action {
    pub const USAGE: &'static str =
        "Usage : .party create <nom> | invite <joueur> | accept | decline | leave | kick <joueur> | leader <joueur> | share <n> | disband";

    /// Arguments de la commande `.party`.
    pub fn from_args(args: &[&str]) -> Option<Self> {
        let one = || (args.len() == 2).then(|| args[1].to_string());
        Some(match (*args.first()?, args.len()) {
            ("create", n) if n >= 2 => Self::Create(args[1..].join(" ")),
            ("invite", _) => Self::Invite(one()?),
            ("accept", 1) => Self::Accept,
            ("decline", 1) => Self::Decline,
            ("leave", 1) => Self::Leave,
            ("disband", 1) => Self::Disband,
            ("kick", _) => Self::Kick(one()?),
            ("leader", _) => Self::Promote(one()?),
            ("share", 2) => Self::Share(args[1].parse().ok()?),
            _ => return None,
        })
    }
}

type Outcome<T> = std::result::Result<T, &'static str>;

#[derive(Default)]
pub struct Parties {
    groups: HashMap<u32, Group>,
    /// Guid du personnage → groupe.
    membership: HashMap<u32, u32>,
    /// Guid de l'invité → invitation.
    invites: HashMap<u32, Invitation>,
    next_id: u32,
    last_positions: u32,
}

impl Parties {
    /// Groupes enregistrés. Un chef disparu est remplacé par le premier membre restant.
    pub fn load(&mut self, stored: Vec<StoredGroup>, max_id: u32) {
        self.next_id = self.next_id.max(max_id);
        for g in stored {
            let members: Vec<Member> = g
                .members
                .into_iter()
                .filter(|m| !self.membership.contains_key(&m.guid))
                .take(MAX_MEMBERS)
                .map(|m| Member { guid: m.guid, name: m.name, level: i32::from(m.level), job: 0 })
                .collect();
            let Some(first) = members.first() else { continue };
            let leader = if members.iter().any(|m| m.guid == g.leader) { g.leader } else { first.guid };
            let name = members.iter().find(|m| m.guid == leader).unwrap().name.clone();
            for m in &members {
                self.membership.insert(m.guid, g.id);
            }
            self.next_id = self.next_id.max(g.id);
            self.groups.insert(g.id, Group { id: g.id, name, leader, members, share: g.share });
        }
    }

    pub fn group(&self, guid: u32) -> Option<&Group> {
        self.groups.get(self.membership.get(&guid)?)
    }

    pub fn by_id(&self, id: u32) -> Option<&Group> {
        self.groups.get(&id)
    }

    fn group_mut(&mut self, guid: u32) -> Option<&mut Group> {
        let id = *self.membership.get(&guid)?;
        self.groups.get_mut(&id)
    }

    pub fn invitation(&self, target: u32, now: u64) -> Option<&Invitation> {
        self.invites.get(&target).filter(|i| i.expires > now)
    }

    pub fn expire(&mut self, now: u64) {
        self.invites.retain(|_, i| i.expires > now);
    }

    fn led_by(&self, actor: u32) -> Outcome<u32> {
        let g = self.group(actor).ok_or("Vous n'êtes pas dans un groupe.")?;
        if g.leader != actor {
            return Err("Seul le chef du groupe peut effectuer cette action.");
        }
        Ok(g.id)
    }

    pub fn create(&mut self, actor: Member, name: &str) -> Outcome<(u32, Vec<Store>)> {
        if self.group(actor.guid).is_some() {
            return Err("Vous êtes déjà dans un groupe.");
        }
        if !text::valid_name(name) {
            return Err("Nom de groupe invalide (1 à 20 caractères, sans « | »).");
        }
        // Le client lit l'identifiant avec atoi : il doit rester un i32 positif.
        let id = self.next_id.checked_add(1).filter(|i| *i <= i32::MAX as u32).ok_or("Identifiants de groupe épuisés.")?;
        self.next_id = id;
        self.invites.remove(&actor.guid);
        let leader = actor.guid;
        self.membership.insert(leader, id);
        self.groups.insert(id, Group { id, name: name.to_string(), leader, members: vec![actor], share: 0 });
        Ok((id, vec![Store::Create { id, leader, share: 0 }]))
    }

    pub fn invite(&mut self, actor: u32, target: u32, now: u64, key: u32) -> Outcome<Invitation> {
        self.expire(now);
        let id = self.led_by(actor)?;
        if actor == target {
            return Err("Vous ne pouvez pas vous inviter.");
        }
        if self.groups[&id].members.len() >= MAX_MEMBERS {
            return Err("Le groupe est complet.");
        }
        if self.group(target).is_some() {
            return Err("Ce joueur est déjà dans un groupe.");
        }
        if self.invites.get(&target).is_some_and(|i| i.party != id) {
            return Err("Ce joueur a déjà une invitation en attente.");
        }
        if self.invites.values().filter(|i| i.inviter == actor && i.party == id).count() >= MAX_MEMBERS {
            return Err("Trop d'invitations en attente.");
        }
        let invitation = Invitation { party: id, key, inviter: actor, expires: now + INVITE_SECONDS };
        self.invites.insert(target, invitation.clone());
        Ok(invitation)
    }

    /// Refus : renvoie l'inviteur.
    pub fn decline(&mut self, target: u32, party: Option<u32>) -> Outcome<u32> {
        match self.invites.get(&target) {
            Some(i) if party.is_none_or(|p| p == i.party) => Ok(self.invites.remove(&target).unwrap().inviter),
            _ => Err("Aucune invitation en attente."),
        }
    }

    /// Acceptation. `key` est vérifiée quand le client la fournit (`/pjoin id clé`).
    pub fn join(&mut self, target: Member, party: Option<u32>, key: Option<u32>, now: u64) -> Outcome<(u32, Vec<Store>)> {
        let i = self.invites.get(&target.guid).ok_or("Aucune invitation en attente.")?;
        if party.is_some_and(|p| p != i.party) || key.is_some_and(|k| k != i.key) {
            return Err("Invitation inconnue.");
        }
        let i = self.invites.remove(&target.guid).unwrap();
        if i.expires <= now {
            return Err("L'invitation a expiré.");
        }
        if self.group(target.guid).is_some() {
            return Err("Vous êtes déjà dans un groupe.");
        }
        let g = self.groups.get_mut(&i.party).ok_or("Ce groupe n'existe plus.")?;
        if g.members.len() >= MAX_MEMBERS {
            return Err("Le groupe est complet.");
        }
        let member = target.guid;
        g.members.push(target);
        self.membership.insert(member, i.party);
        Ok((i.party, vec![Store::Join { id: i.party, member }]))
    }

    /// Départ. Le groupe disparaît avec son dernier membre ; un chef qui part cède sa place au
    /// membre suivant (un groupe d'un seul membre reste valide, comme après `/pcreate`).
    pub fn leave(&mut self, member: u32) -> Outcome<Departure> {
        let id = self.membership.remove(&member).ok_or("Vous n'êtes pas dans un groupe.")?;
        self.invites.retain(|_, i| i.inviter != member);
        let g = self.groups.get_mut(&id).unwrap();
        g.members.retain(|m| m.guid != member);
        let mut d = Departure { id, party: g.name.clone(), new_leader: None, deleted: false, stores: vec![Store::Leave { member }] };
        if g.members.is_empty() {
            self.groups.remove(&id);
            self.invites.retain(|_, i| i.party != id);
            d.deleted = true;
            d.stores = vec![Store::Delete { id }];
        } else if g.leader == member {
            g.leader = g.members[0].guid;
            d.new_leader = Some(g.members[0].name.clone());
            d.stores.push(Store::Leader { id, leader: g.leader });
        }
        Ok(d)
    }

    /// Exclusion par le chef, en ligne ou non.
    pub fn kick(&mut self, actor: u32, name: &str) -> Outcome<(Member, Departure)> {
        self.led_by(actor)?;
        let m = self.group(actor).unwrap().named(name).cloned().ok_or("Ce joueur n'est pas dans votre groupe.")?;
        if m.guid == actor {
            return Err("Utilisez la commande de départ pour quitter le groupe.");
        }
        let d = self.leave(m.guid)?;
        Ok((m, d))
    }

    pub fn promote(&mut self, actor: u32, name: &str) -> Outcome<(u32, Vec<Store>)> {
        let id = self.led_by(actor)?;
        let leader = self.groups[&id].named(name).ok_or("Ce joueur n'est pas dans votre groupe.")?.guid;
        self.groups.get_mut(&id).unwrap().leader = leader;
        if leader != actor {
            self.invites.retain(|_, i| i.inviter != actor);
        }
        Ok((id, vec![Store::Leader { id, leader }]))
    }

    pub fn disband(&mut self, actor: u32) -> Outcome<Group> {
        let id = self.led_by(actor)?;
        let g = self.groups.remove(&id).unwrap();
        for m in &g.members {
            self.membership.remove(&m.guid);
        }
        self.invites.retain(|_, i| i.party != id);
        Ok(g)
    }

    pub fn set_share(&mut self, actor: u32, share: u8) -> Outcome<u32> {
        let id = self.led_by(actor)?;
        self.groups.get_mut(&id).unwrap().share = share;
        Ok(id)
    }

    /// Déconnexion : les invitations tombent, l'appartenance reste.
    pub fn forget_invites(&mut self, guid: u32) {
        self.invites.retain(|target, i| *target != guid && i.inviter != guid);
    }
}

/// Coefficient d'EXP de groupe d'AzerothCore (`Acore::XP::xp_in_group_rate`, groupe normal).
pub fn group_rate(count: usize) -> f64 {
    match count {
        0..=2 => 1.0,
        3 => 1.166,
        4 => 1.3,
        _ => 1.4,
    }
}

impl World {
    fn online(&self, guid: u32) -> Option<&crate::world::PlayerEntity> {
        self.players.values().find(|p| p.guid == guid)
    }

    fn member_of(&self, handle: u32) -> Option<Member> {
        let p = self.players.get(&handle)?;
        Some(Member { guid: p.guid, name: p.name.clone(), level: p.enter.creature.level, job: i32::from(p.enter.job_id) })
    }

    fn party_line(&self, m: &Member) -> MemberLine {
        let Some(p) = self.online(m.guid) else {
            return MemberLine { name: m.name.clone(), level: m.level, job: m.job, ..Default::default() };
        };
        let percent = |v: i32, max: i32| (i64::from(v.max(0)) * 100 / i64::from(max.max(1))).clamp(0, 100) as i32;
        let (x, y) = p.motion.position(self.now());
        MemberLine {
            handle: p.handle,
            name: p.name.clone(),
            level: p.enter.creature.level,
            job: i32::from(p.enter.job_id),
            hp_percent: percent(p.hp, p.max_hp),
            mp_percent: percent(p.mp, p.max_mp),
            x: x as i32,
            y: y as i32,
        }
    }

    /// Message `@PARTY` pour un membre en ligne utilisant le client SFrame.
    fn party_say(&self, guid: u32, line: &str) {
        if let Some(p) = self.online(guid).filter(|p| !self.party_clients.contains(&p.handle)) {
            let _ = p.tx.send(text::system(line.to_string()).to_packet());
        }
    }

    fn party_tell(&self, id: u32, line: &str, except: Option<u32>) {
        let Some(g) = self.parties.by_id(id) else { return };
        for m in g.members.iter().filter(|m| Some(m.guid) != except) {
            self.party_say(m.guid, line);
        }
    }

    fn party_info(&self, id: u32) {
        let Some(g) = self.parties.by_id(id) else { return };
        let lines: Vec<MemberLine> = g.members.iter().map(|m| self.party_line(m)).collect();
        let info = text::pinfo(g.id, &g.name, g.leader_name(), g.share, &lines);
        self.party_tell(id, &info, None);
    }

    /// Action de groupe d'un joueur ; renvoie les écritures à faire en base.
    pub fn party_action(&mut self, actor: u32, action: Action) -> Vec<Store> {
        let Some(me) = self.member_of(actor) else { return Vec::new() };
        let result = self.try_party_action(actor, me, action);
        self.publish_parties();
        result.unwrap_or_else(|why| {
            self.notify(actor, why);
            Vec::new()
        })
    }

    fn try_party_action(&mut self, actor: u32, me: Member, action: Action) -> Outcome<Vec<Store>> {
        let now = self.uptime_secs();
        let guid = me.guid;
        match action {
            Action::Create(name) => {
                let (id, stores) = self.parties.create(me.clone(), name.trim())?;
                self.party_say(guid, &text::create(name.trim(), &me.name));
                self.party_info(id);
                Ok(stores)
            }
            Action::Invite(name) => {
                let target = self.find_by_name(&name).map(|p| (p.guid, p.handle)).ok_or("Ce joueur n'est pas connecté.")?;
                let mut stores = Vec::new();
                // Sans groupe, l'invitation en crée un au nom de l'inviteur (commande `.party`).
                if self.parties.group(guid).is_none() {
                    let (id, s) = self.parties.create(me.clone(), &me.name)?;
                    stores = s;
                    self.party_say(guid, &text::create(&me.name, &me.name));
                    self.party_info(id);
                }
                let key = rand::random::<u32>() % (i32::MAX as u32) + 1;
                let i = match self.parties.invite(guid, target.0, now, key) {
                    Ok(i) => i,
                    Err(why) => {
                        self.notify(actor, why);
                        return Ok(stores);
                    }
                };
                let party = self.parties.by_id(i.party).unwrap().name.clone();
                self.party_say(target.0, &text::invite(&me.name, &party, i.party, i.key));
                self.notify(target.1, &format!("{} vous invite dans le groupe « {party} » (.party accept / decline).", me.name));
                self.notify(actor, &format!("Invitation envoyée à {name}."));
                Ok(stores)
            }
            Action::Respond { party, key: None } => self.party_decline(&me, Some(party)),
            Action::Decline => self.party_decline(&me, None),
            Action::Respond { party, key: Some(key) } => self.party_join(me, Some(party), Some(key), now),
            Action::Accept => self.party_join(me, None, None, now),
            Action::Leave => {
                let d = self.parties.leave(guid)?;
                self.party_say(guid, &text::destroy(&d.party));
                self.party_departed(&d, &text::leave(&me.name));
                Ok(d.stores)
            }
            Action::Kick(name) => {
                let (m, d) = self.parties.kick(guid, &name)?;
                self.party_say(m.guid, &text::destroy(&d.party));
                if let Some(h) = self.online(m.guid).map(|p| p.handle) {
                    self.notify(h, "Vous avez été exclu du groupe.");
                }
                self.party_departed(&d, &text::kick(&d.party, &m.name));
                Ok(d.stores)
            }
            Action::Promote(name) => {
                let (id, stores) = self.parties.promote(guid, &name)?;
                let g = self.parties.by_id(id).unwrap();
                let line = text::promote(&g.name, g.leader_name());
                self.party_tell(id, &line, None);
                self.party_info(id);
                Ok(stores)
            }
            Action::Disband => {
                let g = self.parties.disband(guid)?;
                for m in &g.members {
                    self.party_say(m.guid, &text::destroy(&g.name));
                    if let Some(h) = self.online(m.guid).map(|p| p.handle) {
                        self.notify(h, "Le groupe est dissous.");
                    }
                }
                Ok(vec![Store::Delete { id: g.id }])
            }
            Action::Share(share) => {
                let id = self.parties.set_share(guid, share)?;
                self.party_tell(id, &text::mode(share), None);
                Ok(vec![Store::Share { id, share }])
            }
        }
    }

    fn party_decline(&mut self, me: &Member, party: Option<u32>) -> Outcome<Vec<Store>> {
        let inviter = self.parties.decline(me.guid, party)?;
        if let Some(h) = self.online(inviter).map(|p| p.handle) {
            self.notify(h, &format!("{} a refusé votre invitation.", me.name));
        }
        Ok(Vec::new())
    }

    fn party_join(&mut self, me: Member, party: Option<u32>, key: Option<u32>, now: u64) -> Outcome<Vec<Store>> {
        let (guid, name) = (me.guid, me.name.clone());
        let (id, stores) = self.parties.join(me, party, key, now)?;
        let g = self.parties.by_id(id).unwrap();
        self.party_tell(id, &text::join(&g.name, &name), Some(guid));
        self.party_info(id);
        Ok(stores)
    }

    /// Prévient les membres restants d'un départ (LEAVE ou KICK), puis du nouveau chef.
    fn party_departed(&self, d: &Departure, line: &str) {
        if d.deleted {
            return;
        }
        self.party_tell(d.id, line, None);
        if let Some(leader) = &d.new_leader {
            self.party_tell(d.id, &text::promote(&d.party, leader), None);
        }
        self.party_info(d.id);
    }

    /// Entrée en jeu d'un membre : les autres le voient en ligne, tous reçoivent PINFO.
    pub(crate) fn party_login(&mut self, handle: u32) {
        let Some(me) = self.member_of(handle) else { return };
        let Some(g) = self.parties.group_mut(me.guid) else { return };
        if let Some(m) = g.members.iter_mut().find(|m| m.guid == me.guid) {
            (m.name, m.level, m.job) = (me.name.clone(), me.level, me.job);
        }
        let (id, line) = (g.id, text::login(&g.name, &me.name));
        self.party_tell(id, &line, Some(me.guid));
        self.party_info(id);
    }

    /// Sortie du monde : le membre reste dans son groupe, hors ligne.
    pub(crate) fn party_logout(&mut self, guid: u32, name: &str) {
        self.parties.forget_invites(guid);
        if let Some(id) = self.parties.group(guid).map(|g| g.id) {
            self.party_tell(id, &text::logout(name), Some(guid));
        }
    }

    pub(crate) fn party_level(&mut self, handle: u32) {
        let Some(me) = self.member_of(handle) else { return };
        let Some(g) = self.parties.group_mut(me.guid) else { return };
        if let Some(m) = g.members.iter_mut().find(|m| m.guid == me.guid) {
            m.level = me.level;
        }
        let id = g.id;
        self.party_tell(id, &text::change_level(&me.name, me.level), None);
    }

    /// Personnage supprimé : il quitte son groupe.
    pub(crate) fn party_forget(&mut self, guid: u32) -> Vec<Store> {
        self.parties.forget_invites(guid);
        let Some(name) = self.parties.group(guid).and_then(|g| g.members.iter().find(|m| m.guid == guid)).map(|m| m.name.clone()) else {
            return Vec::new();
        };
        let Ok(d) = self.parties.leave(guid) else { return Vec::new() };
        self.party_departed(&d, &text::leave(&name));
        self.publish_parties();
        d.stores
    }

    /// PPOS : positions des membres en ligne, pour les groupes d'au moins deux joueurs connectés.
    pub(crate) fn tick_party_positions(&mut self, now: u32) {
        if now.saturating_sub(self.parties.last_positions) < POSITIONS_INTERVAL {
            return;
        }
        self.parties.last_positions = now;
        for g in self.parties.groups.values() {
            let points: Vec<(u32, i32, i32)> = g
                .members
                .iter()
                .filter_map(|m| self.online(m.guid))
                .map(|p| {
                    let (x, y) = p.motion.position(now);
                    (p.handle, x as i32, y as i32)
                })
                .collect();
            if points.len() >= 2 {
                let line = text::ppos(&points);
                for m in &g.members {
                    self.party_say(m.guid, &line);
                }
            }
        }
    }

    /// Partage d'une victoire : membres vivants dans la couche du tueur et vus par lui.
    /// Chacun reçoit `total × rate × niveau / somme des niveaux` (règle d'AzerothCore).
    pub(crate) fn party_shares(&self, killer: u32, exp: i64, jp: i64) -> Vec<(u32, i64, i64)> {
        let Some(k) = self.players.get(&killer) else { return Vec::new() };
        let eligible: Vec<&crate::world::PlayerEntity> = match self.parties.group(k.guid) {
            Some(g) => g
                .members
                .iter()
                .filter_map(|m| self.online(m.guid))
                .filter(|p| p.handle == killer || (p.hp > 0 && p.layer == k.layer && k.visible.contains(&p.handle)))
                .collect(),
            None => vec![k],
        };
        if eligible.len() <= 1 {
            return vec![(killer, exp, jp)];
        }
        let rate = group_rate(eligible.len());
        let levels: i64 = eligible.iter().map(|p| i64::from(p.enter.creature.level.max(1))).sum();
        eligible
            .iter()
            .map(|p| {
                let share = |v: i64| ((v.max(0) as f64) * rate * p.enter.creature.level.max(1) as f64 / levels as f64) as i64;
                (p.handle, share(exp).max(1), share(jp))
            })
            .collect()
    }

    pub fn enable_party(&mut self, handle: u32, version: u16) {
        if version != VERSION || !self.players.contains_key(&handle) {
            return;
        }
        self.party_clients.insert(handle);
        self.publish_parties();
    }

    /// Extension RappelzRust : listes bornées recalculées depuis les entités en ligne.
    /// Un groupe n'y apparaît qu'avec au moins deux membres connectés ; un chef hors ligne y
    /// est remplacé par le premier membre connecté.
    pub fn publish_parties(&mut self) {
        let now = self.uptime_secs();
        self.parties.expire(now);
        let mut updates = Vec::new();
        for &handle in &self.party_clients {
            let Some(guid) = self.players.get(&handle).map(|p| p.guid) else { continue };
            let mut state = ScPartyState::default();
            if let Some(g) = self.parties.group(guid) {
                let members: Vec<PartyMember> = g
                    .members
                    .iter()
                    .filter_map(|m| self.online(m.guid))
                    .map(|p| PartyMember {
                        handle: p.handle,
                        name: p.name.clone(),
                        level: p.enter.creature.level,
                        layer: p.layer,
                        hp: p.hp.clamp(0, p.max_hp.max(1)),
                        max_hp: p.max_hp.max(1),
                        mp: p.mp.clamp(0, p.max_mp.max(1)),
                        max_mp: p.max_mp.max(1),
                    })
                    .collect();
                if members.len() >= 2 {
                    state.party_id = g.id;
                    state.leader = self.online(g.leader).map_or(members[0].handle, |p| p.handle);
                    state.members = members;
                }
            } else if let Some(i) = self.parties.invitation(guid, now) {
                if let Some(p) = self.online(i.inviter) {
                    state.invite = Some(PartyInvite {
                        inviter: p.handle,
                        name: p.name.clone(),
                        seconds_left: (i.expires - now).min(u16::MAX as u64) as u16,
                    });
                }
            }
            if self.party_sent.get(&handle) != Some(&state) {
                updates.push((handle, state));
            }
        }
        for (handle, state) in updates {
            self.send_to(handle, state.to_packet());
            self.party_sent.insert(handle, state);
        }
    }

    pub fn party_chat(&self, actor: u32, text: &str) {
        let Some(p) = self.players.get(&actor) else { return };
        let Some(g) = self.parties.group(p.guid) else {
            self.notify(actor, "Vous n'êtes pas dans un groupe.");
            return;
        };
        let packet = rz_proto::msg::ScChat { sender: p.name.clone(), chat_type: rz_proto::msg::chat_type::PARTY, message: text.into() }.to_packet();
        for m in &g.members {
            if let Some(other) = self.online(m.guid) {
                let _ = other.tx.send(packet.clone());
            }
        }
    }
}

/// Reporte les écritures d'une action. Une erreur SQL est journalisée sans annuler l'état en
/// mémoire, qui reste la référence jusqu'au prochain redémarrage.
pub(crate) async fn store(db: &MySqlPool, stores: &[Store]) {
    for s in stores {
        let r = match *s {
            Store::Create { id, leader, share } => db_groups::create(db, id, leader, share).await,
            Store::Join { id, member } => db_groups::add_member(db, id, member).await,
            Store::Leave { member } => db_groups::remove_member(db, member).await,
            Store::Leader { id, leader } => db_groups::set_leader(db, id, leader).await,
            Store::Share { id, share } => db_groups::set_share(db, id, share).await,
            Store::Delete { id } => db_groups::delete(db, id).await,
        };
        if let Err(e) = r {
            tracing::warn!("groupe : écriture {s:?} impossible : {e:#}");
        }
    }
}

impl Session {
    /// Action de groupe du joueur, puis écriture en base.
    pub(crate) async fn party(&mut self, action: Action) -> Result<()> {
        let Some(g) = &self.in_game else { return Ok(()) };
        let stores = self.ctx.world.lock().unwrap().party_action(g.handle, action);
        store(&self.ctx.chars, &stores).await;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn m(guid: u32) -> Member {
        Member { guid, name: format!("P{guid}"), level: guid as i32, job: 0 }
    }

    #[test]
    fn create_invite_and_join_check_the_key_and_the_party() {
        let mut p = Parties::default();
        assert!(p.invite(1, 2, 0, 7).is_err(), "sans groupe");
        let (id, stores) = p.create(m(1), "Fiers").unwrap();
        assert_eq!(stores, vec![Store::Create { id, leader: 1, share: 0 }]);
        assert!(p.create(m(1), "Encore").is_err());
        assert!(p.create(m(9), "a|b").is_err());
        assert!(p.invite(1, 1, 0, 7).is_err());
        p.invite(1, 2, 0, 7).unwrap();
        assert!(p.join(m(2), Some(id), Some(8), 1).is_err(), "mauvaise clé");
        // La mauvaise clé consomme l'invitation ? Non : elle reste.
        assert!(p.invitation(2, 1).is_none_or(|i| i.key == 7));
        p.invite(1, 2, 1, 7).unwrap();
        assert!(p.join(m(2), Some(id + 1), Some(7), 2).is_err(), "autre groupe");
        p.invite(1, 2, 2, 7).unwrap();
        assert_eq!(p.join(m(2), Some(id), Some(7), 3).unwrap().1, vec![Store::Join { id, member: 2 }]);
        assert_eq!(p.group(2).unwrap().members.len(), 2);
        // Seul le chef invite.
        assert!(p.invite(2, 3, 3, 1).is_err());
        // Refus et expiration.
        p.invite(1, 3, 3, 5).unwrap();
        assert_eq!(p.decline(3, Some(id)).unwrap(), 1);
        p.invite(1, 3, 3, 5).unwrap();
        assert!(p.join(m(3), None, None, 3 + INVITE_SECONDS).is_err());
    }

    #[test]
    fn departures_pass_leadership_and_the_last_one_deletes() {
        let mut p = Parties::default();
        let (id, _) = p.create(m(1), "Fiers").unwrap();
        for g in 2..=3 {
            p.invite(1, g, 0, 1).unwrap();
            p.join(m(g), None, None, 1).unwrap();
        }
        let d = p.leave(1).unwrap();
        assert_eq!((d.new_leader.as_deref(), d.deleted), (Some("P2"), false));
        assert_eq!(d.stores, vec![Store::Leave { member: 1 }, Store::Leader { id, leader: 2 }]);
        assert!(p.kick(3, "P2").is_err(), "pas chef");
        let (kicked, d) = p.kick(2, "p3").unwrap();
        assert_eq!((kicked.guid, d.deleted), (3, false));
        assert_eq!(p.group(2).unwrap().members.len(), 1);
        let d = p.leave(2).unwrap();
        assert!(d.deleted && d.stores == vec![Store::Delete { id }]);
        assert!(p.by_id(id).is_none() && p.group(2).is_none());
        // Les identifiants ne sont pas réutilisés.
        assert!(p.create(m(1), "Autre").unwrap().0 > id);
    }

    #[test]
    fn capacity_promotion_share_and_reload() {
        let mut p = Parties::default();
        let (id, _) = p.create(m(1), "Fiers").unwrap();
        for g in 2..=8 {
            p.invite(1, g, 0, 1).unwrap();
            p.join(m(g), None, None, 1).unwrap();
        }
        assert_eq!(p.invite(1, 9, 1, 1), Err("Le groupe est complet."));
        assert!(p.promote(2, "P3").is_err());
        assert_eq!(p.promote(1, "P4").unwrap().1, vec![Store::Leader { id, leader: 4 }]);
        assert!(p.set_share(1, 2).is_err());
        assert_eq!(p.set_share(4, 2), Ok(id));
        let g = p.disband(4).unwrap();
        assert_eq!(g.members.len(), 8);
        assert!(p.group(1).is_none());
        // Rechargement : chef disparu remplacé, identifiants suivants au-delà du maximum.
        let mut q = Parties::default();
        q.load(
            vec![StoredGroup {
                id: 40,
                leader: 99,
                share: 1,
                members: vec![
                    db_groups::StoredMember { guid: 5, name: "P5".into(), level: 3 },
                    db_groups::StoredMember { guid: 6, name: "P6".into(), level: 4 },
                ],
            }],
            41,
        );
        let g = q.group(6).unwrap();
        assert_eq!((g.id, g.leader, g.name.as_str(), g.share), (40, 5, "P5", 1));
        assert_eq!(q.create(m(1), "Nouveau").unwrap().0, 42);
    }

    #[test]
    fn group_rate_follows_azerothcore() {
        assert_eq!([1, 2, 3, 4, 5, 8].map(group_rate), [1.0, 1.0, 1.166, 1.3, 1.4, 1.4]);
        assert_eq!(Action::from_args(&["create", "Les", "Fiers"]), Some(Action::Create("Les Fiers".into())));
        assert_eq!(Action::from_args(&["leader", "P2"]), Some(Action::Promote("P2".into())));
        assert_eq!(Action::from_args(&["kick"]), None);
        assert_eq!(Action::from(Command::Join { party: 3, key: None }), Action::Respond { party: 3, key: None });
    }
}

//! Ephemeral, server-authoritative groups. No database rows or retail party rules are assumed.
use crate::world::World;
use rz_proto::party::{PartyInvite, PartyMember, ScPartyState, MAX_MEMBERS, VERSION};
use rz_proto::Message;
use std::collections::HashMap;

pub const INVITE_SECONDS: u64 = 60;
#[derive(Clone, Debug)]
pub struct Group {
    pub leader: u32,
    pub members: Vec<u32>,
}
#[derive(Clone, Debug)]
pub struct Invitation {
    pub inviter: u32,
    pub expires: u64,
}
#[derive(Default)]
pub struct Parties {
    groups: HashMap<u32, Group>,
    membership: HashMap<u32, u32>,
    invites: HashMap<u32, Invitation>,
    next_id: u32,
}
type Outcome<T> = std::result::Result<T, &'static str>;
impl Parties {
    pub fn group(&self, member: u32) -> Option<(u32, &Group)> {
        let id = *self.membership.get(&member)?;
        Some((id, self.groups.get(&id)?))
    }
    pub fn invitation(&self, target: u32, now: u64) -> Option<&Invitation> {
        self.invites.get(&target).filter(|i| i.expires > now)
    }
    pub fn expire(&mut self, now: u64) {
        self.invites.retain(|_, i| i.expires > now);
    }
    fn leader_group(&self, actor: u32) -> Outcome<u32> {
        let (id, g) = self.group(actor).ok_or("Vous n'êtes pas dans un groupe.")?;
        if g.leader != actor {
            return Err("Seul le chef du groupe peut effectuer cette action.");
        }
        Ok(id)
    }
    pub fn invite(&mut self, actor: u32, target: u32, now: u64) -> Outcome<()> {
        self.expire(now);
        if actor == target {
            return Err("Vous ne pouvez pas vous inviter.");
        }
        if self.group(target).is_some() {
            return Err("Ce joueur est déjà dans un groupe.");
        }
        if let Some((_, g)) = self.group(actor) {
            self.leader_group(actor)?;
            if g.members.len() >= MAX_MEMBERS {
                return Err("Le groupe est complet.");
            }
        }
        if self.invites.contains_key(&target) {
            return Err("Ce joueur a déjà une invitation en attente.");
        }
        if self.invites.values().filter(|i| i.inviter == actor).count() >= MAX_MEMBERS - 1 {
            return Err("Trop d'invitations en attente.");
        }
        self.invites.insert(
            target,
            Invitation {
                inviter: actor,
                expires: now + INVITE_SECONDS,
            },
        );
        Ok(())
    }
    pub fn accept(&mut self, target: u32, now: u64) -> Outcome<()> {
        let invitation = self
            .invites
            .remove(&target)
            .ok_or("Aucune invitation en attente.")?;
        if invitation.expires <= now {
            return Err("L'invitation a expiré.");
        }
        if self.group(target).is_some() {
            return Err("Vous êtes déjà dans un groupe.");
        }
        let actor = invitation.inviter;
        if self.group(actor).is_some() {
            let id = self.leader_group(actor)?;
            let group = self.groups.get_mut(&id).unwrap();
            if group.members.len() >= MAX_MEMBERS {
                return Err("Le groupe est complet.");
            }
            group.members.push(target);
            self.membership.insert(target, id);
        } else {
            let id = self
                .next_id
                .checked_add(1)
                .ok_or("Identifiants de groupe épuisés.")?;
            self.next_id = id;
            self.groups.insert(
                id,
                Group {
                    leader: actor,
                    members: vec![actor, target],
                },
            );
            self.membership.insert(actor, id);
            self.membership.insert(target, id);
        }
        // Joining another group revokes invitations previously sent by the joiner.
        self.invites.remove(&actor);
        self.invites.retain(|_, i| i.inviter != target);
        Ok(())
    }
    pub fn decline(&mut self, target: u32) -> Outcome<u32> {
        Ok(self
            .invites
            .remove(&target)
            .ok_or("Aucune invitation en attente.")?
            .inviter)
    }
    pub fn leave(&mut self, member: u32) -> Outcome<()> {
        let id = self
            .membership
            .remove(&member)
            .ok_or("Vous n'êtes pas dans un groupe.")?;
        self.invites.retain(|_, i| i.inviter != member);
        let g = self.groups.get_mut(&id).unwrap();
        g.members.retain(|m| *m != member);
        if g.members.len() < 2 {
            for remaining in &g.members {
                self.membership.remove(remaining);
            }
            let remaining = g.members.clone();
            self.groups.remove(&id);
            self.invites.retain(|_, i| !remaining.contains(&i.inviter));
        } else if g.leader == member {
            g.leader = g.members[0];
        }
        Ok(())
    }
    pub fn kick(&mut self, actor: u32, target: u32) -> Outcome<()> {
        let id = self.leader_group(actor)?;
        if actor == target {
            return Err("Utilisez la commande de départ pour quitter le groupe.");
        }
        if self.membership.get(&target) != Some(&id) {
            return Err("Ce joueur n'est pas dans votre groupe.");
        }
        self.leave(target)
    }
    pub fn promote(&mut self, actor: u32, target: u32) -> Outcome<()> {
        let id = self.leader_group(actor)?;
        if self.membership.get(&target) != Some(&id) {
            return Err("Ce joueur n'est pas dans votre groupe.");
        }
        self.groups.get_mut(&id).unwrap().leader = target;
        if actor != target {
            self.invites.retain(|_, i| i.inviter != actor);
        }
        Ok(())
    }
    pub fn disband(&mut self, actor: u32) -> Outcome<()> {
        let id = self.leader_group(actor)?;
        let g = self.groups.remove(&id).unwrap();
        for member in &g.members {
            self.membership.remove(member);
        }
        self.invites.retain(|_, i| !g.members.contains(&i.inviter));
        Ok(())
    }
    pub fn disconnected(&mut self, member: u32) {
        let _ = self.leave(member);
        self.invites
            .retain(|target, i| *target != member && i.inviter != member);
    }
}

impl World {
    pub fn enable_party(&mut self, handle: u32, version: u16) {
        if version != VERSION || !self.players.contains_key(&handle) {
            return;
        }
        self.party_clients.insert(handle);
        self.publish_parties();
    }
    /// Recompute bounded rosters from live entities. Only changed states go on the wire.
    pub fn publish_parties(&mut self) {
        let now = self.uptime_secs();
        self.parties.expire(now);
        let mut updates = Vec::new();
        for &handle in &self.party_clients {
            let mut state = ScPartyState::default();
            if let Some((id, g)) = self.parties.group(handle) {
                state.party_id = id;
                state.leader = g.leader;
                state.members = g
                    .members
                    .iter()
                    .filter_map(|h| self.players.get(h))
                    .map(|p| PartyMember {
                        handle: p.handle,
                        name: p.name.clone(),
                        level: p.enter.creature.level,
                        layer: p.layer,
                        hp: p.hp.max(0),
                        max_hp: p.max_hp,
                        mp: p.mp.max(0),
                        max_mp: p.max_mp,
                    })
                    .collect();
            } else if let Some(i) = self.parties.invitation(handle, now) {
                if let Some(p) = self.players.get(&i.inviter) {
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
    pub fn party_command(&mut self, actor: u32, args: &[&str]) {
        if !self.players.contains_key(&actor) {
            return;
        }
        let action = args.first().copied().unwrap_or("");
        let target = if args.len() == 2 {
            self.find_by_name(args[1]).map(|p| p.handle)
        } else {
            None
        };
        let result=match (action,args.len()) {
            ("invite",2)=>match target {
                Some(target) if self.can_see(actor,target,self.now())=>self.parties.invite(actor,target,self.uptime_secs()),
                _=>Err("Le joueur doit être connecté et visible dans votre couche."),
            },
            ("accept",1)=>self.parties.accept(actor,self.uptime_secs()),
            ("decline",1)=>self.parties.decline(actor).map(|inviter|self.notify(inviter,"Invitation de groupe refusée.")),
            ("leave",1)=>self.parties.leave(actor),
            ("disband",1)=>self.parties.disband(actor),
            ("kick",2)=>target.ok_or("Joueur introuvable.").and_then(|t|self.parties.kick(actor,t)),
            ("leader",2)=>target.ok_or("Joueur introuvable.").and_then(|t|self.parties.promote(actor,t)),
            _=>Err("Usage : .party invite <joueur> | accept | decline | leave | kick <joueur> | leader <joueur> | disband"),
        };
        match result {
            Ok(()) => {
                self.notify(actor, "Groupe : action effectuée.");
                if action == "invite" {
                    if let Some(target) = target {
                        let name = &self.players[&actor].name;
                        self.notify(
                            target,
                            &format!(
                                "{name} vous invite dans un groupe (.party accept / decline)."
                            ),
                        );
                    }
                }
            }
            Err(message) => self.notify(actor, message),
        }
        self.publish_parties();
    }
    pub fn party_chat(&self, actor: u32, text: &str) {
        let Some((_, g)) = self.parties.group(actor) else {
            self.notify(actor, "Vous n'êtes pas dans un groupe.");
            return;
        };
        let Some(p) = self.players.get(&actor) else {
            return;
        };
        let packet = rz_proto::msg::ScChat {
            sender: p.name.clone(),
            chat_type: rz_proto::msg::chat_type::PARTY,
            message: text.into(),
        }
        .to_packet();
        for member in &g.members {
            self.send_to(*member, packet.clone());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn consent_privileges_expiry_and_incoming_invitation_are_enforced() {
        let mut p = Parties::default();
        assert!(p.accept(2, 0).is_err());
        assert!(p.invite(1, 1, 0).is_err());
        p.invite(1, 2, 0).unwrap();
        assert!(p.group(1).is_none());
        assert!(p.invite(3, 2, 0).is_err());
        assert_eq!(p.decline(2).unwrap(), 1);
        p.invite(1, 2, 0).unwrap();
        assert!(p.accept(2, 60).is_err());
        p.invite(1, 2, 61).unwrap();
        p.accept(2, 62).unwrap();
        assert_eq!(p.group(1).unwrap().1.members, [1, 2]);
        assert!(p.invite(2, 3, 62).is_err());
        assert!(p.disband(2).is_err());
        assert!(p.kick(2, 1).is_err());
        p.promote(1, 2).unwrap();
        assert!(p.disband(1).is_err());
        p.disband(2).unwrap();
        assert!(p.group(1).is_none());
        assert!(p.group(2).is_none());
    }
    #[test]
    fn capacity_is_rechecked_on_accept_and_disconnect_revokes_pending_authority() {
        let mut p = Parties::default();
        p.invite(1, 2, 0).unwrap();
        p.accept(2, 1).unwrap();
        for target in 3..=9 {
            p.invite(1, target, 1).unwrap();
        }
        for target in 3..=8 {
            p.accept(target, 2).unwrap();
        }
        assert!(p.accept(9, 2).is_err());
        assert_eq!(p.group(1).unwrap().1.members.len(), 8);
        p.disconnected(1);
        assert_eq!(p.group(2).unwrap().1.leader, 2);
        p.invite(2, 9, 3).unwrap();
        p.disconnected(2);
        assert!(p.invitation(9, 3).is_none());
        assert_eq!(p.group(3).unwrap().1.leader, 3);
        p.invite(3, 10, 4).unwrap();
        p.promote(3, 4).unwrap();
        assert!(p.invitation(10, 4).is_none());
        p.kick(4, 3).unwrap();
        assert!(p.group(3).is_none());
    }
    #[test]
    fn two_member_departure_dissolves_and_ids_are_not_reused() {
        let mut p = Parties::default();
        p.invite(1, 2, 0).unwrap();
        p.accept(2, 1).unwrap();
        let first = p.group(1).unwrap().0;
        p.leave(2).unwrap();
        assert!(p.group(1).is_none());
        p.invite(1, 2, 2).unwrap();
        p.accept(2, 3).unwrap();
        assert_ne!(p.group(1).unwrap().0, first);
        p.disconnected(2);
        assert!(p.groups.is_empty());
        assert!(p.membership.is_empty());
        p.invite(1, 2, 4).unwrap();
        p.disconnected(1);
        assert!(p.accept(2, 5).is_err());
    }
}

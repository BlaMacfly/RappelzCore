//! Versioned, opt-in Rust party extension. These are NOT recovered SFrame packets.
//! Legacy clients receive neither this state nor an unsolicited extension handshake.
use crate::packet::{PacketError, Reader, Result, Writer};
use crate::Message;

pub const CS_PARTY_HELLO: u16 = 62000;
pub const SC_PARTY_STATE: u16 = 62001;
pub const VERSION: u16 = 1;
/// Local policy, not a claim about retail party rules.
pub const MAX_MEMBERS: usize = 8;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CsPartyHello(pub u16);
impl Message for CsPartyHello {
    const ID: u16 = CS_PARTY_HELLO;
    fn write(&self, w: &mut Writer) {
        w.u16(self.0);
    }
    fn read(r: &mut Reader<'_>) -> Result<Self> {
        let version = r.u16()?;
        if r.remaining() != 0 {
            return Err(invalid(Self::ID, "trailing bytes"));
        }
        Ok(Self(version))
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PartyMember {
    pub handle: u32,
    pub name: String,
    pub level: i32,
    pub layer: u8,
    pub hp: i32,
    pub max_hp: i32,
    pub mp: i32,
    pub max_mp: i32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PartyInvite {
    pub inviter: u32,
    pub name: String,
    pub seconds_left: u16,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ScPartyState {
    pub party_id: u32,
    pub leader: u32,
    /// Includes the recipient; empty means a confirmed solo player.
    pub members: Vec<PartyMember>,
    pub invite: Option<PartyInvite>,
}

fn invalid(id: u16, field: &'static str) -> PacketError {
    PacketError::Invalid { id, field }
}

impl Message for ScPartyState {
    const ID: u16 = SC_PARTY_STATE;
    fn write(&self, w: &mut Writer) {
        w.u16(VERSION)
            .u32(self.party_id)
            .u32(self.leader)
            .u8(self.members.len() as u8);
        for m in &self.members {
            w.u32(m.handle)
                .fixed_str(&m.name, 21)
                .i32(m.level)
                .u8(m.layer)
                .i32(m.hp)
                .i32(m.max_hp)
                .i32(m.mp)
                .i32(m.max_mp);
        }
        w.bool(self.invite.is_some());
        if let Some(i) = &self.invite {
            w.u32(i.inviter).fixed_str(&i.name, 21).u16(i.seconds_left);
        }
    }
    fn read(r: &mut Reader<'_>) -> Result<Self> {
        if r.u16()? != VERSION {
            return Err(invalid(Self::ID, "version"));
        }
        let party_id = r.u32()?;
        let leader = r.u32()?;
        let count = r.u8()? as usize;
        if count > MAX_MEMBERS || count == 1 {
            return Err(invalid(Self::ID, "member count"));
        }
        let mut members = Vec::with_capacity(count);
        for _ in 0..count {
            let m = PartyMember {
                handle: r.u32()?,
                name: r.fixed_str(21)?,
                level: r.i32()?,
                layer: r.u8()?,
                hp: r.i32()?,
                max_hp: r.i32()?,
                mp: r.i32()?,
                max_mp: r.i32()?,
            };
            if m.handle == 0
                || m.name.is_empty()
                || m.level <= 0
                || m.max_hp <= 0
                || m.hp < 0
                || m.hp > m.max_hp
                || m.mp < 0
                || m.mp > m.max_mp
                || members
                    .iter()
                    .any(|other: &PartyMember| other.handle == m.handle)
            {
                return Err(invalid(Self::ID, "member"));
            }
            members.push(m);
        }
        if (count == 0 && (party_id != 0 || leader != 0))
            || (count != 0 && (party_id == 0 || !members.iter().any(|m| m.handle == leader)))
        {
            return Err(invalid(Self::ID, "party identity"));
        }
        let invite = match r.u8()? {
            0 => None,
            1 => {
                let i = PartyInvite {
                    inviter: r.u32()?,
                    name: r.fixed_str(21)?,
                    seconds_left: r.u16()?,
                };
                if i.inviter == 0 || i.name.is_empty() || i.seconds_left == 0 || count != 0 {
                    return Err(invalid(Self::ID, "invite"));
                }
                Some(i)
            }
            _ => return Err(invalid(Self::ID, "invite flag")),
        };
        if r.remaining() != 0 {
            return Err(invalid(Self::ID, "trailing bytes"));
        }
        Ok(Self {
            party_id,
            leader,
            members,
            invite,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn member(handle: u32) -> PartyMember {
        PartyMember {
            handle,
            name: format!("Player{handle}"),
            level: 1,
            hp: 100,
            max_hp: 100,
            mp: 50,
            max_mp: 50,
            ..Default::default()
        }
    }
    #[test]
    fn party_wire_roundtrips_solo_invite_and_full_roster_with_dead_member() {
        let mut full = ScPartyState {
            party_id: 1,
            leader: 1,
            members: (1..=8).map(member).collect(),
            invite: None,
        };
        full.members[1].hp = 0;
        full.members[1].layer = 1;
        for state in [
            ScPartyState::default(),
            ScPartyState {
                invite: Some(PartyInvite {
                    inviter: 1,
                    name: "Player1".into(),
                    seconds_left: 60,
                }),
                ..Default::default()
            },
            full,
        ] {
            let packet = state.to_packet();
            assert_eq!(ScPartyState::from_packet(&packet).unwrap(), state);
            for len in 0..packet.body.len() {
                let mut short = packet.clone();
                short.body.truncate(len);
                assert!(ScPartyState::from_packet(&short).is_err());
            }
        }
        assert_eq!(
            CsPartyHello::from_packet(&CsPartyHello(VERSION).to_packet())
                .unwrap()
                .0,
            VERSION
        );
    }
    #[test]
    fn party_wire_rejects_bad_version_counts_identity_duplicate_members_and_tail() {
        let valid = ScPartyState {
            party_id: 1,
            leader: 1,
            members: vec![member(1), member(2)],
            invite: None,
        };
        let mut duplicate = valid.clone();
        duplicate.members[1].handle = 1;
        assert!(ScPartyState::from_packet(&duplicate.to_packet()).is_err());
        let mut wrong = valid.clone();
        wrong.leader = 9;
        assert!(ScPartyState::from_packet(&wrong.to_packet()).is_err());
        wrong = valid.clone();
        wrong.members[0].hp = 101;
        assert!(ScPartyState::from_packet(&wrong.to_packet()).is_err());
        wrong = valid.clone();
        wrong.members.truncate(1);
        assert!(ScPartyState::from_packet(&wrong.to_packet()).is_err());
        let mut p = valid.to_packet();
        p.body[0] = 2;
        assert!(ScPartyState::from_packet(&p).is_err());
        p = valid.to_packet();
        p.body[10] = 9;
        assert!(ScPartyState::from_packet(&p).is_err());
        p = valid.to_packet();
        p.body.push(0);
        assert!(ScPartyState::from_packet(&p).is_err());
    }
}

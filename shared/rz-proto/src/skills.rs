//! Opt-in Rust learning catalog, distinct from recovered SFrame messages.
use crate::packet::{PacketError, Reader, Result, Writer};
use crate::Message;
pub const CS_SKILLS_HELLO: u16 = 62010;
pub const SC_SKILL_CATALOG: u16 = 62011;
pub const VERSION: u16 = 1;
pub const MAX_ENTRIES: usize = 512;
pub mod catalog {
    pub const READY: u8 = 0;
    pub const UNAVAILABLE: u8 = 1;
    pub const INVALID_CHARACTER: u8 = 2;
    pub const UNKNOWN_JOB: u8 = 3;
}
pub mod reason {
    pub const READY: u8 = 0;
    pub const JP: u8 = 1;
    pub const PREREQUISITE: u8 = 2;
    pub const JOB_LEVEL: u8 = 3;
    pub const PLAYER_LEVEL: u8 = 4;
    pub const MAXIMUM: u8 = 5;
    pub const UNSUPPORTED: u8 = 6;
    pub const INVALID_DATA: u8 = 7;
    pub const WRONG_JOB: u8 = 8;
    pub const UNKNOWN_SKILL: u8 = 9;
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CsSkillsHello(pub u16);
impl Message for CsSkillsHello {
    const ID: u16 = CS_SKILLS_HELLO;
    fn write(&self, w: &mut Writer) {
        w.u16(self.0);
    }
    fn read(r: &mut Reader) -> Result<Self> {
        let v = r.u16()?;
        if r.remaining() != 0 {
            return Err(PacketError::Invalid {
                id: CS_SKILLS_HELLO,
                field: "hello length",
            });
        }
        Ok(Self(v))
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SkillOffer {
    pub id: u32,
    pub learned: u8,
    pub next: u8,
    pub maximum: u8,
    pub cost: Option<i64>,
    pub player_level: u16,
    pub job_level: u16,
    pub prerequisites: [(u32, u8); 3],
    pub passive: bool,
    pub reason: u8,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScSkillCatalog {
    pub target: u32,
    pub job: i32,
    pub job_level: i32,
    pub player_level: u8,
    pub jp: i64,
    pub status: u8,
    pub entries: Vec<SkillOffer>,
}
fn invalid(field: &'static str) -> PacketError {
    PacketError::Invalid {
        id: SC_SKILL_CATALOG,
        field,
    }
}
impl Message for ScSkillCatalog {
    const ID: u16 = SC_SKILL_CATALOG;
    fn write(&self, w: &mut Writer) {
        w.u16(VERSION)
            .u32(self.target)
            .i32(self.job)
            .i32(self.job_level)
            .u8(self.player_level)
            .i64(self.jp)
            .u8(self.status)
            .u16(self.entries.len() as u16);
        for e in &self.entries {
            w.u32(e.id)
                .u8(e.learned)
                .u8(e.next)
                .u8(e.maximum)
                .i64(e.cost.unwrap_or(-1))
                .u16(e.player_level)
                .u16(e.job_level);
            for (id, level) in e.prerequisites {
                w.u32(id).u8(level);
            }
            w.bool(e.passive).u8(e.reason);
        }
    }
    fn read(r: &mut Reader) -> Result<Self> {
        if r.u16()? != VERSION {
            return Err(invalid("version"));
        }
        let mut state = Self {
            target: r.u32()?,
            job: r.i32()?,
            job_level: r.i32()?,
            player_level: r.u8()?,
            jp: r.i64()?,
            status: r.u8()?,
            entries: Vec::new(),
        };
        let count = r.u16()? as usize;
        if state.target == 0
            || state.status > catalog::UNKNOWN_JOB
            || count > MAX_ENTRIES
            || (state.status != catalog::READY && count != 0)
        {
            return Err(invalid("catalog header"));
        }
        for _ in 0..count {
            let id = r.u32()?;
            let learned = r.u8()?;
            let next = r.u8()?;
            let maximum = r.u8()?;
            let cost = r.i64()?;
            let player_level = r.u16()?;
            let job_level = r.u16()?;
            let mut prerequisites = [(0, 0); 3];
            for pair in &mut prerequisites {
                *pair = (r.u32()?, r.u8()?);
            }
            let passive = match r.u8()? {
                0 => false,
                1 => true,
                _ => return Err(invalid("passive flag")),
            };
            let reason = r.u8()?;
            if id == 0
                || state
                    .entries
                    .last()
                    .is_some_and(|e: &SkillOffer| e.id >= id)
                || learned > 50
                || maximum > 50
                || maximum < learned
                || next != if learned < 50 { learned + 1 } else { 0 }
                || cost < -1
                || reason > reason::UNKNOWN_SKILL
                || (reason == reason::READY
                    && (cost < 0 || next == 0 || next > maximum || state.jp < cost))
            {
                return Err(invalid("offer"));
            }
            state.entries.push(SkillOffer {
                id,
                learned,
                next,
                maximum,
                cost: (cost >= 0).then_some(cost),
                player_level,
                job_level,
                prerequisites,
                passive,
                reason,
            });
        }
        if r.remaining() != 0 {
            return Err(invalid("trailing bytes"));
        }
        Ok(state)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn learning_request_matches_21_byte_profile_and_keeps_signed_level_for_validation() {
        use crate::msg_game::CsLearnSkill;
        for level in [1, 50, -1] {
            let request = CsLearnSkill {
                target: 7,
                skill_id: 42,
                base_random_skill_id: 0,
                skill_level: level,
            };
            let packet = request.to_packet();
            assert_eq!(packet.to_bytes().len(), 21);
            assert_eq!(&packet.body[12..14], &level.to_le_bytes());
            assert_eq!(CsLearnSkill::from_packet(&packet).unwrap(), request);
            for len in 0..14 {
                let mut short = packet.clone();
                short.body.truncate(len);
                assert!(CsLearnSkill::from_packet(&short).is_err());
            }
            let mut extra = packet;
            extra.body.push(0);
            assert!(CsLearnSkill::from_packet(&extra).is_err());
        }
    }
    fn sample() -> ScSkillCatalog {
        ScSkillCatalog {
            target: 1,
            job: 100,
            job_level: 1,
            player_level: 1,
            jp: 10,
            status: catalog::READY,
            entries: vec![SkillOffer {
                id: 42,
                learned: 0,
                next: 1,
                maximum: 3,
                cost: Some(3),
                player_level: 1,
                job_level: 1,
                prerequisites: [(0, 0); 3],
                passive: false,
                reason: reason::READY,
            }],
        }
    }
    #[test]
    fn catalog_roundtrip_retains_zero_cost_wide_jp_and_all_truncations_fail() {
        let mut s = sample();
        s.jp = i64::MAX;
        s.entries[0].cost = Some(0);
        let packet = s.to_packet();
        assert_eq!(ScSkillCatalog::from_packet(&packet).unwrap(), s);
        for len in 0..packet.body.len() {
            let mut short = packet.clone();
            short.body.truncate(len);
            assert!(ScSkillCatalog::from_packet(&short).is_err());
        }
    }
    #[test]
    fn catalog_rejects_fabricated_ready_invalid_counts_duplicates_and_trailing_data() {
        let mut s = sample();
        s.entries[0].cost = Some(11);
        assert!(ScSkillCatalog::from_packet(&s.to_packet()).is_err());
        s = sample();
        s.entries.push(s.entries[0].clone());
        assert!(ScSkillCatalog::from_packet(&s.to_packet()).is_err());
        let mut p = sample().to_packet();
        p.body.push(0);
        assert!(ScSkillCatalog::from_packet(&p).is_err());
        s = sample();
        s.status = catalog::UNAVAILABLE;
        assert!(ScSkillCatalog::from_packet(&s.to_packet()).is_err());
        s = sample();
        s.entries[0].learned = 51;
        assert!(ScSkillCatalog::from_packet(&s.to_packet()).is_err());
    }
}

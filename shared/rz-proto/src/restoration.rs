//! Checked subset of the existing SKILL event, not a new wire extension.
use crate::{
    msg_game::{self as game, skill_type},
    packet::{PacketError, Result},
    Writer,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Phase {
    Casting(u32),
    Cancel,
    Fire { mana: i32, amount: i32 },
    /// EF511 emits ADD_HP (zero for the supported profile), then ADD_MP to the caster.
    CorpseFire { mana:i32, amount:i32 },
}
fn invalid() -> PacketError {
    PacketError::Invalid {
        id: game::id::TS_SC_SKILL,
        field: "self restoration event",
    }
}
pub fn payload(phase: &Phase) -> Vec<u8> {
    let mut w = Writer::new();
    match phase {
        Phase::Casting(duration) => {
            w.u32(*duration).u16(0).zeros(3);
        }
        Phase::Cancel => {
            w.zeros(9);
        }
        Phase::Fire { mana, amount } => {
            w.bool(false)
                .f32(0.)
                .u8(1)
                .u8(1)
                .u16(1)
                .u8(game::skill_hit::ADD_MP)
                .u32(0)
                .i32(*mana)
                .i32(*amount);
        }
        Phase::CorpseFire{mana,amount}=>{
            w.bool(false).f32(0.).u8(1).u8(1).u16(2)
                .u8(game::skill_hit::ADD_HP).u32(0).i32(0).i32(0)
                .u8(game::skill_hit::ADD_MP).u32(0).i32(*mana).i32(*amount);
        }
    }
    w.finish(0).body
}
pub fn event(
    id: u16,
    level: u8,
    caster: u32,
    position: (f32, f32, f32, u8),
    hp: i32,
    mp: i32,
    cost: i32,
    phase: Phase,
) -> game::ScSkill {
    let mut bytes = payload(&phase);
    if matches!(phase, Phase::Fire { .. }|Phase::CorpseFire{..}) {
        bytes[10..14].copy_from_slice(&caster.to_le_bytes());
        if matches!(phase,Phase::CorpseFire{..}) {
            bytes[14..18].copy_from_slice(&hp.to_le_bytes());
            bytes[23..27].copy_from_slice(&caster.to_le_bytes());
        }
    }
    game::ScSkill {
        skill_id: id,
        skill_level: level,
        caster,
        target: caster,
        x: position.0,
        y: position.1,
        z: position.2,
        layer: position.3,
        skill_type: match phase {
            Phase::Casting(_) => skill_type::CASTING,
            Phase::Cancel => skill_type::CANCEL,
            Phase::Fire { .. }|Phase::CorpseFire{..} => skill_type::FIRE,
        },
        hp_cost: 0,
        mp_cost: cost,
        caster_hp: hp,
        caster_mp: mp,
        payload: bytes,
    }
}
pub fn decode(s: &game::ScSkill) -> Result<Phase> {
    if s.skill_id == 0
        || !(1..=50).contains(&s.skill_level)
        || s.caster == 0
        || s.target==0
        || ![s.x, s.y, s.z].iter().all(|v| v.is_finite())
        || s.hp_cost != 0
        || s.mp_cost < 0
        || s.caster_hp < 0
        || s.caster_mp < 0
    {
        return Err(invalid());
    }
    if ![9, 22, 54, 35, 99].contains(&s.payload.len()) {
        return Err(invalid());
    }
    let packet = crate::RawPacket {
        id: game::id::TS_SC_SKILL,
        body: s.payload.clone(),
    };
    let mut r = packet.reader();
    let phase = match s.skill_type {
        skill_type::CASTING => {
            let duration = r.u32()?;
            if duration == 0 || duration > 360_000 || r.u16()? != 0 || r.rest() != [0; 3] {
                return Err(invalid());
            }
            return Ok(Phase::Casting(duration));
        }
        skill_type::CANCEL => {
            if s.payload != [0; 9] {
                return Err(invalid());
            }
            return Ok(Phase::Cancel);
        }
        skill_type::FIRE => {
            if r.u8()? != 0
                || r.f32()? != 0.
                || r.u8()? != 1
                || r.u8()? != 1
            {
                return Err(invalid());
            }
            let count=r.u16()?;
            if count==2 {
                if s.target==s.caster || ![35,99].contains(&s.payload.len()) || r.u8()?!=game::skill_hit::ADD_HP || r.u32()?!=s.caster || r.i32()?!=s.caster_hp || r.i32()?!=0 {return Err(invalid());}
                if s.payload.len()==99 {for _ in 0..32 {if r.u8()?!=0{return Err(invalid());}}}
            }else if count!=1 || s.target!=s.caster {return Err(invalid());}
            if r.u8()?!=game::skill_hit::ADD_MP || r.u32()?!=s.caster {return Err(invalid());}
            let mana = r.i32()?;
            let amount = r.i32()?;
            if mana < 0 || mana != s.caster_mp || amount < 0 {
                return Err(invalid());
            }
            if count==2 {Phase::CorpseFire{mana,amount}} else {Phase::Fire { mana, amount }}
        }
        _ => return Err(invalid()),
    };
    // Local constructor walks compact 13-byte entries; public profiles may pad the union to 45.
    if r.remaining() != 0 && r.rest() != [0; 32] {
        return Err(invalid());
    }
    Ok(phase)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Message;
    #[test]
    fn corpse_fire_distinguishes_selected_body_from_two_caster_recovery_hits(){
        let mut s=event(3212,1,7,(10.,20.,0.,0),100,50,5,Phase::CorpseFire{mana:50,amount:320});s.target=0x20000001;
        assert_eq!(decode(&s).unwrap(),Phase::CorpseFire{mana:50,amount:320});assert_eq!(s.to_packet().to_bytes().len(),83);
        assert_eq!(s.payload[9],20);assert_eq!(&s.payload[10..14],&7u32.to_le_bytes());assert_eq!(s.payload[22],21);assert_eq!(&s.payload[23..27],&7u32.to_le_bytes());
        let mut padded=s.clone();padded.payload.splice(22..22,[0;32]);padded.payload.extend([0;32]);assert!(decode(&padded).is_ok());
        for index in [9,10,18,22,23] {let mut bad=s.clone();bad.payload[index]=bad.payload[index].wrapping_add(1);assert!(decode(&bad).is_err(),"{index}");}
        s.target=7;assert!(decode(&s).is_err());
    }
    #[test]
    fn checked_cast_cancel_and_mana_hit_have_independent_offsets_and_reject_bad_fields() {
        let phases = [
            Phase::Casting(145),
            Phase::Cancel,
            Phase::Fire {
                mana: 50,
                amount: 3500,
            },
        ];
        for phase in phases {
            let s = event(3212, 1, 7, (10., 20., 0., 0), 100, 50, 5, phase.clone());
            assert_eq!(decode(&s).unwrap(), phase);
            assert_eq!(game::ScSkill::from_packet(&s.to_packet()).unwrap(), s);
        }
        let mut s = event(
            3212,
            1,
            7,
            (10., 20., 0., 0),
            100,
            50,
            5,
            Phase::Fire {
                mana: 50,
                amount: 3500,
            },
        );
        assert_eq!(s.to_packet().to_bytes().len(), 70);
        assert_eq!(s.payload[9], 21);
        assert_eq!(&s.payload[10..14], &7u32.to_le_bytes());
        assert_eq!(&s.payload[14..18], &50i32.to_le_bytes());
        assert_eq!(&s.payload[18..22], &3500i32.to_le_bytes());
        s.payload.extend([0; 32]);
        assert!(decode(&s).is_ok());
        s.payload.push(0);
        assert!(decode(&s).is_err());
        s.payload.truncate(21);
        assert!(decode(&s).is_err());
        s.target = 8;
        assert!(decode(&s).is_err());
        let request = game::CsSkill {
            skill_id: 3212,
            caster: 7,
            target: 7,
            skill_level: 1,
            ..Default::default()
        };
        let mut packet = request.to_packet();
        assert_eq!(packet.to_bytes().len(), 31);
        assert_eq!(&packet.to_bytes()[9..13], &7u32.to_le_bytes());
        packet.body.push(0);
        assert!(game::CsSkill::from_packet(&packet).is_err());
        packet.body.truncate(23);
        assert!(game::CsSkill::from_packet(&packet).is_err());
        assert!(game::CsSkill::from_packet(
            &game::CsSkill {
                x: f32::NAN,
                ..request
            }
            .to_packet()
        )
        .is_err());
    }
}

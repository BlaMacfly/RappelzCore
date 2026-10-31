//! Deliberately narrow cast support: unenhanced, fixed-amount MP restoration on self.
//! Field interpretations are recorded in docs/SKILL_CASTING_VALIDATION.md.
use crate::dbcstore::{NamedDbc, Row, StoreError};
use std::{collections::HashMap, path::Path};

#[derive(Clone, Debug, Default)]
pub struct Restoration {
    pub corpse: bool,
    pub range: u32,
    pub cooldown_per_level: i32,
    pub id: u16,
    pub mp_base: i32,
    pub mp_per_level: i32,
    pub cast_base: i32,
    pub cast_per_level: i32,
    pub cooldown: u32,
    pub group: u32,
    pub minimum_level: u8,
    pub minimum_hp: i32,
    pub minimum_mp: i32,
    pub amount_base: f32,
    pub amount_per_level: f32,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Quote {
    pub corpse: bool,
    pub range: u32,
    pub id: u16,
    pub level: u8,
    pub mp: i32,
    /// Durations in centiseconds, matching the game clock. No cast-speed system yet.
    pub cast: u32,
    pub cooldown: u32,
    pub group: u32,
    pub amount: i32,
    pub minimum_level: u8,
    pub minimum_hp: i32,
    pub minimum_mp: i32,
}

fn centiseconds(value: f32) -> Option<i32> {
    let v = f64::from(value) * 100.0;
    // Dec2 columns have hundredth precision; recover it before level arithmetic.
    (v.is_finite() && (-360_000.0..=360_000.0).contains(&v)).then(|| v.round() as i32)
}
impl Restoration {
    pub fn from_row(r: Row<'_>) -> Option<Self> {
        if r.int("is_valid") != 1
            || r.int("is_passive") != 1
            || r.int("effect_type") != 502
            || r.int("target") != 1
            || r.int("is_need_target") != 0
            || r.int("uf_self") != 1
            || r.int("tf_avatar") != 1
            || r.int("vf_is_not_need_weapon") != 1
            || !(0..=1).contains(&r.int("casting_type"))
        {
            return None;
        }
        // No implicit ignoring of costs, conditions, states, MAtk, percentage or AoE effects.
        for name in [
            "is_harmful",
            "is_corpse",
            "is_toggle",
            "cost_hp",
            "cost_hp_per_skl",
            "cost_havoc",
            "cost_havoc_per_skl",
            "cost_energy",
            "cost_energy_per_skl",
            "cost_exp",
            "cost_jp",
            "cost_item",
            "cost_item_count",
            "cost_item_count_per_skl",
            "need_havoc",
            "need_havoc_burst",
            "need_state_id",
            "need_state_level",
            "need_state_exhaust",
            "state_id",
            "uf_party",
            "uf_guild",
            "uf_neutral",
            "uf_purple",
            "uf_enemy",
        ] {
            if r.int(name) != 0 {
                return None;
            }
        }
        for name in [
            "cost_hp_per",
            "cost_hp_per_skl_per",
            "cost_mp_per",
            "cost_mp_per_skl_per",
            "delay_common",
            "var1",
            "var2",
            "var5",
            "var6",
            "var9",
            "var10",
            "var11",
            "var12",
            "var13",
            "var14",
            "var15",
            "var16",
            "var17",
            "var18",
            "var19",
            "var20",
        ] {
            if r.float(name) != 0.0 {
                return None;
            }
        }
        let id = u16::try_from(r.id()).ok().filter(|id| *id > 0)?;
        let group = u32::try_from(r.int("cool_time_group_id")).ok()?;
        let cooldown = u32::try_from(centiseconds(r.float("delay_cooltime"))?).ok()?;
        // This reference profile's cooldown has no per-level term.
        if r.float("delay_cooltime_per_skl") != 0.0 {
            return None;
        }
        Some(Self {
            id,
            mp_base: r.int("cost_mp"),
            mp_per_level: r.int("cost_mp_per_skl"),
            cast_base: centiseconds(r.float("delay_cast"))?,
            cast_per_level: centiseconds(r.float("delay_cast_per_skl"))?,
            cooldown,
            group,
            minimum_level: u8::try_from(r.int("need_level")).ok()?,
            minimum_hp: r.int("need_hp"),
            minimum_mp: r.int("need_mp"),
            amount_base: r.float("var3"),
            amount_per_level: r.float("var4"),
            ..Default::default()
        })
    }
    /// Corpse targeting is a separate client-stage rule; never merge the loose
    /// 502 row into the archive's 511 coefficients or permissions.
    pub fn from_corpse_rows(r:Row<'_>,stage:Row<'_>)->Option<Self>{
        if r.id()!=3212 || stage.id()!=r.id() || r.int("is_valid")!=1 || r.int("is_passive")!=1
            || r.int("effect_type")!=511 || r.int("is_need_target")!=1 || r.int("is_corpse")!=1
            || r.int("is_harmful")!=1 || r.int("target")!=1 || r.int("uf_enemy")!=1
            || r.int("tf_monster")!=1 || r.int("vf_is_not_need_weapon")!=1
            || stage.int("has_target")!=1 || stage.int("corpse_target")!=1 {return None;}
        for name in ["is_toggle","uf_self","uf_party","uf_guild","uf_neutral","uf_purple","tf_avatar","tf_summon",
            "cost_hp","cost_hp_per_skl","cost_havoc","cost_havoc_per_skl","cost_energy","cost_energy_per_skl",
            "cost_exp","cost_jp","cost_item","cost_item_count","cost_item_count_per_skl","need_havoc","need_havoc_burst",
            "need_state_id","need_state_level","need_state_exhaust","state_id"] {if r.int(name)!=0{return None;}}
        for name in ["cost_hp_per","cost_hp_per_skl_per","cost_mp_per","cost_mp_per_skl_per","delay_common",
            "var1","var2","var5","var7","var8","var9","var10","var11","var12","var13","var14","var15","var16","var17","var18","var19","var20"] {if r.float(name)!=0. {return None;}}
        let cast_range=u32::try_from(r.int("cast_range")).ok()?.checked_mul(12)?;
        let valid_range=u32::try_from(r.int("valid_range")).ok()?.checked_mul(12)?;
        if cast_range==0 || cast_range>4096 || valid_range<cast_range {return None;}
        Some(Self{corpse:true,range:cast_range,id:3212,mp_base:r.int("cost_mp"),mp_per_level:r.int("cost_mp_per_skl"),
            cast_base:centiseconds(r.float("delay_cast"))?,cast_per_level:centiseconds(r.float("delay_cast_per_skl"))?,
            cooldown:u32::try_from(centiseconds(r.float("delay_cooltime"))?).ok()?,cooldown_per_level:centiseconds(r.float("delay_cooltime_per_skl"))?,
            group:u32::try_from(r.int("cool_time_group_id")).ok()?,minimum_level:u8::try_from(r.int("need_level")).ok()?,
            minimum_hp:r.int("need_hp"),minimum_mp:r.int("need_mp"),amount_base:r.float("var3"),amount_per_level:r.float("var4")})
    }
    pub fn quote(&self, level: u8) -> Option<Quote> {
        if !(1..=50).contains(&level) || self.minimum_hp < 0 || self.minimum_mp < 0 {
            return None;
        }
        let mp = i64::from(self.mp_base) + i64::from(self.mp_per_level) * i64::from(level);
        let cast = i64::from(self.cast_base) + i64::from(self.cast_per_level) * i64::from(level);
        let amount =
            f64::from(self.amount_base) + f64::from(self.amount_per_level) * f64::from(level);
        let cooldown=i64::from(self.cooldown)+i64::from(self.cooldown_per_level)*i64::from(level);
        if !(0..=i64::from(i32::MAX)).contains(&mp)
            || !(1..=360_000).contains(&cast)
            || !(0..=360_000).contains(&cooldown)
            || !amount.is_finite()
            || !(1.0..=f64::from(i32::MAX)).contains(&amount)
        {
            return None;
        }
        Some(Quote {
            corpse:self.corpse,range:self.range,
            id: self.id,
            level,
            mp: mp as i32,
            cast: cast as u32,
            cooldown: cooldown as u32,
            group: if self.group == 0 {
                0x1_0000 + u32::from(self.id)
            } else {
                self.group
            },
            amount: amount.trunc() as i32,
            minimum_level: self.minimum_level,
            minimum_hp: self.minimum_hp,
            minimum_mp: self.minimum_mp,
        })
    }
}
#[derive(Default)]
pub struct RestorationBook(HashMap<u16, Restoration>);
impl RestorationBook {
    pub fn open(path: &Path) -> Result<Self, StoreError> {
        let path=path.join("CorpseSkill");
        let table = NamedDbc::open(&path, "skill")?;
        let stages = NamedDbc::open(&path,"skillstage")?;
        Ok(Self::from_corpse_tables(&table,&stages))
    }
    pub fn from_corpse_tables(table:&NamedDbc,stages:&NamedDbc)->Self{
        // The isolated profile must contain exactly one unambiguous row per table.
        if table.len()!=1 || stages.len()!=1 {return Self::default();}
        Self(
            table
                .rows()
                .filter_map(|row|Restoration::from_corpse_rows(row,stages.row(0)))
                .map(|d| (d.id, d))
                .collect(),
        )
    }
    pub fn quote(&self, id: u16, level: u8) -> Option<Quote> {
        self.0.get(&id)?.quote(level)
    }
    pub fn from_definition(d: Restoration) -> Self {
        Self(HashMap::from([(d.id, d)]))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn table(name:&str,values:&[(&str,f64)])->NamedDbc{
        use crate::rdb::{self,FieldType,RdbValue};
        let layout=rdb::layout(name).unwrap();
        let row=layout.fields.iter().map(|field|{
            let value=values.iter().find(|(key,_)|*key==field.name).map_or(0.,|(_,v)|*v);
            match field.ty {FieldType::F32|FieldType::F64|FieldType::Dec2=>RdbValue::Float(value),FieldType::Char(_)=>RdbValue::Str(String::new()),_=>RdbValue::Int(value as i64)}
        }).collect();
        let data=rdb::RdbTable{header:rdb::RdbHeader{date:String::new(),comment:String::new(),count:1},layout,rows:vec![row]};
        NamedDbc::from_bytes(name,&crate::to_dbc::table_to_dbc(&data)).unwrap()
    }
    #[test]
    fn archived_corpse_profile_is_separate_from_hybrid_or_missing_stage_rules(){
        let fields=[("id",3212.),("is_valid",1.),("is_passive",1.),("effect_type",511.),("is_need_target",1.),("is_corpse",1.),("is_harmful",1.),("target",1.),("uf_enemy",1.),("tf_monster",1.),("vf_is_not_need_weapon",1.),("cast_range",16.),("valid_range",16.),("delay_cast",1.5),("delay_cast_per_skl",-0.05),("delay_cooltime",60.),("delay_cooltime_per_skl",-2.),("cost_mp_per_skl",5.),("cool_time_group_id",103.),("var3",200.),("var4",120.)];
        let stage=table("skillstage",&[("id",3212.),("has_target",1.),("corpse_target",1.)]);
        let skills=table("skill",&fields);let b=RestorationBook::from_corpse_tables(&skills,&stage);let q=b.quote(3212,1).unwrap();
        assert_eq!((q.amount,q.mp,q.cast,q.cooldown,q.corpse,q.range),(320,5,145,5800,true,192));
        assert_eq!((b.quote(3212,2).unwrap().amount,b.quote(3212,2).unwrap().cooldown),(440,5600));
        for (key,value) in [("effect_type",502.),("is_corpse",0.),("is_need_target",0.),("uf_self",1.),("tf_avatar",1.),("var7",1.),("cast_range",0.)] {
            let mut changed=fields.to_vec();if let Some(item)=changed.iter_mut().find(|(name,_)|*name==key){item.1=value;}else{changed.push((key,value));}
            assert!(RestorationBook::from_corpse_tables(&table("skill",&changed),&stage).quote(3212,1).is_none(),"{key}");
        }
        let no_corpse=table("skillstage",&[("id",3212.),("has_target",1.)]);assert!(RestorationBook::from_corpse_tables(&skills,&no_corpse).quote(3212,1).is_none());
    }
    #[test]
    fn costs_durations_and_amounts_are_checked_before_any_cast() {
        let mut d = Restoration {
            id: 3212,
            mp_base: 0,
            mp_per_level: 5,
            cast_base: 150,
            cast_per_level: -5,
            cooldown: 2500,
            group: 103,
            minimum_level: 0,
            minimum_hp: 0,
            minimum_mp: 0,
            amount_base: 2000.,
            amount_per_level: 1500.,
            ..Default::default()
        };
        let q = d.quote(1).unwrap();
        assert_eq!((q.mp, q.cast, q.cooldown, q.amount), (5, 145, 2500, 3500));
        assert!(d.quote(0).is_none());
        assert!(d.quote(51).is_none());
        assert!(d.quote(31).is_none());
        d.mp_per_level = i32::MAX;
        assert!(d.quote(2).is_none());
        d.mp_per_level = -1;
        assert!(d.quote(1).is_none());
        d.mp_per_level = 5;
        d.amount_base = f32::NAN;
        assert!(d.quote(1).is_none());
        assert_eq!(centiseconds(-0.05), Some(-5));
        assert_eq!(centiseconds(f32::INFINITY), None);
    }
    #[test]
    fn available_local_data_supports_mana_restoration_but_not_passives_or_stat_heals() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../client-data/server-data/dbc");
        if !path.join("CorpseSkill/Skill.dbc").exists() {
            return;
        }
        let b = RestorationBook::open(&path).unwrap();
        let quote=b.quote(3212,1).unwrap();
        assert_eq!((quote.amount,quote.mp,quote.cast,quote.cooldown,quote.range,quote.corpse),(320,5,145,5800,192,true));
        for id in [1003, 1004, 3201, 3202, 3211, 9550] {
            assert!(b.quote(id, 1).is_none(), "{id}");
        }
    }
}

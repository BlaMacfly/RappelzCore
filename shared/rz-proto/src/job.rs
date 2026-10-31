//! Recovered 410 handle request; private native progression preview is an opt-in extension.
use crate::{Message,Reader,Writer};
use crate::packet::{PacketError,Result};
pub const CS_JOB_LEVEL_UP:u16=410;
pub const SC_JOB_PROGRESS:u16=62012;
pub const VERSION:u16=1;
pub mod status {
    pub const READY:u8=0;
    pub const UNAVAILABLE:u8=1;
    pub const INVALID_CHARACTER:u8=2;
    pub const UNKNOWN_JOB:u8=3;
    pub const MAXIMUM:u8=4;
    pub const JP:u8=5;
    pub const UNSUPPORTED:u8=6;
}
#[derive(Clone,Debug,PartialEq,Eq)]
pub struct CsJobLevelUp{pub target:u32}
impl Message for CsJobLevelUp {
    const ID:u16=CS_JOB_LEVEL_UP;
    fn write(&self,w:&mut Writer){w.u32(self.target);}
    fn read(r:&mut Reader)->Result<Self>{let target=r.u32()?;if r.remaining()!=0{return Err(invalid(Self::ID,"request length"));}Ok(Self{target})}
}
#[derive(Clone,Debug,PartialEq,Eq)]
pub struct ScJobProgress {
    pub target:u32,pub job:i32,pub job_level:i32,pub jp:i64,pub cost:Option<i64>,pub status:u8,
    /// Raw job resource base, before level/equipment/state bonuses; never effective stats.
    pub base:Option<(i32,[i16;7])>,
}
fn invalid(id:u16,field:&'static str)->PacketError{PacketError::Invalid{id,field}}
impl Message for ScJobProgress {
    const ID:u16=SC_JOB_PROGRESS;
    fn write(&self,w:&mut Writer){
        w.u16(VERSION).u32(self.target).i32(self.job).i32(self.job_level).i64(self.jp).i64(self.cost.unwrap_or(-1)).u8(self.status);
        let (id,values)=self.base.unwrap_or((-1,[0;7]));w.i32(id);for v in values{w.u16(v as u16);}
    }
    fn read(r:&mut Reader)->Result<Self>{
        if r.u16()?!=VERSION{return Err(invalid(Self::ID,"version"));}
        let target=r.u32()?;let job=r.i32()?;let job_level=r.i32()?;let jp=r.i64()?;let cost=r.i64()?;let status=r.u8()?;let base_id=r.i32()?;let mut attributes=[0;7];for v in &mut attributes{*v=r.u16()? as i16;}
        if target==0 || status>status::UNSUPPORTED || cost < -1 || cost==0 || base_id < -1 || base_id>i16::MAX as i32 || attributes.iter().any(|v|*v<0)
            || (base_id==-1 && attributes!=[0;7]) || r.remaining()!=0
            || (matches!(status,status::READY|status::JP) && (cost<1 || jp<0 || job<0 || !(1..50).contains(&job_level) || base_id<0))
            || (status==status::READY && jp<cost) || (status==status::JP && jp>=cost) {
            return Err(invalid(Self::ID,"progress fields"));
        }
        Ok(Self{target,job,job_level,jp,cost:(cost>0).then_some(cost),status,base:(base_id>=0).then_some((base_id,attributes))})
    }
}
#[cfg(test)]
mod tests{
    use super::*;
    #[test]fn recovered_request_is_eleven_bytes_and_rejects_trailing_data(){let q=CsJobLevelUp{target:0x12345678};let mut p=q.to_packet();assert_eq!(p.to_bytes().len(),11);assert_eq!(p.body,[0x78,0x56,0x34,0x12]);assert_eq!(CsJobLevelUp::from_packet(&p).unwrap(),q);p.body.push(0);assert!(CsJobLevelUp::from_packet(&p).is_err());}
    #[test]fn private_preview_roundtrips_large_exact_jp_and_rejects_invalid_states(){
        let mut s=ScJobProgress{target:7,job:302,job_level:35,jp:9_007_199_254_740_993,cost:Some(30200),status:status::READY,base:Some((123,[14,16,13,13,20,20,10]))};
        assert_eq!(ScJobProgress::from_packet(&s.to_packet()).unwrap(),s);
        for field in 0..5{let mut bad=s.clone();match field{0=>bad.target=0,1=>bad.cost=Some(0),2=>bad.jp=-1,3=>bad.job_level=50,_=>bad.base=None};assert!(ScJobProgress::from_packet(&bad.to_packet()).is_err());}
        s.status=status::JP;s.jp=100;assert_eq!(ScJobProgress::from_packet(&s.to_packet()).unwrap(),s);let mut p=s.to_packet();p.body.push(0);assert!(ScJobProgress::from_packet(&p).is_err());
    }
}

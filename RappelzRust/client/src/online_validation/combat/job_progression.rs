//! Two ordinary native clients exercise real job-level requests and durable private JP.
use super::super::{mark,read,unit};
use crate::{App,NetCmd};
use anyhow::{ensure,Context,Result};
use rz_proto::{job::{self,status},Message};
use serde_json::{json,Value};
use std::{collections::BTreeMap,path::PathBuf,time::{Duration,Instant}};

pub struct JobValidation {
    directory:PathBuf,role:String,peer:String,character:String,stage:u8,started:Instant,since:Instant,
    own:u32,initial:u32,other:u32,initial_other:u32,steps:Vec<Value>,requests:Vec<Value>,events:BTreeMap<String,usize>,messages:Vec<String>,failure:Option<String>,
}
fn quote(p:&job::ScJobProgress)->Value{json!({"target":p.target,"job":p.job,"jlv":p.job_level,"jp":p.jp,"cost":p.cost,"status":p.status,"base":p.base})}
impl JobValidation {
    pub fn new(p:&Value,app:&App)->Result<Self>{
        super::super::super::guard(app)?;let role=p["role"].as_str().context("role")?.to_owned();ensure!(role=="owner"||role=="witness","role");let peer=p["peer"].as_str().context("peer")?.to_owned();ensure!(peer.starts_with("Lua"),"dedicated peer");
        let directory=PathBuf::from(p["directory"].as_str().context("directory")?);ensure!(!directory.join(format!("{role}.json")).exists(),"fresh attempt");
        Ok(Self{directory,role,peer,character:app.auto_enter.clone().context("character")?,stage:0,started:Instant::now(),since:Instant::now(),own:0,initial:0,other:0,initial_other:0,steps:vec![],requests:vec![],events:BTreeMap::new(),messages:vec![],failure:None})
    }
    pub fn message(&mut self,line:&str){if let Some(e)=line.strip_prefix("[RappelzDebug] EVENT ").and_then(|s|s.split_whitespace().next()){*self.events.entry(e.into()).or_default()+=1;}if self.messages.len()<512{self.messages.push(line.into());}}
    fn ready(&self,key:&str)->bool{read(&self.directory.join(key)).is_some()}
    fn mark(&self,key:&str)->Result<()>{mark(&self.directory,key,json!({"handle":self.own,"elapsed_ms":self.started.elapsed().as_millis()}))}
    fn advance(&mut self,stage:u8){self.stage=stage;self.since=Instant::now();}
    fn record(&mut self,app:&App,name:&str){let w=app.world.as_ref();let s=w.map(|w|w.addon_state()).unwrap_or_default();self.steps.push(json!({"stage":name,"elapsed_ms":self.started.elapsed().as_millis(),"player":s.player.as_ref().map(unit),"peer":w.and_then(|w|w.unit(self.other)).as_ref().map(unit),"job":w.and_then(|w|w.job_progress()).map(quote),"xp_jp":w.map(|w|w.experience()),"money":s.money,"position":w.and_then(|w|w.unit_position(self.own)),"learned":s.learned_skills,"events":self.events,"display":super::super::super::AddonValidation::display(app)}));}
    fn capture(&self,label:&str)->Result<()>{let p=self.directory.join(format!("{}-{label}.png",self.role));macroquad::prelude::get_screen_data().export_png(p.to_str().context("capture")?);Ok(())}
    fn raw(&mut self,app:&App,target:u32,label:&str){self.requests.push(json!({"target":target,"label":label,"kind":"raw"}));app.send(NetCmd::Raw(job::CsJobLevelUp{target}.to_packet()));}
    fn increase(&mut self,app:&mut App,label:&str)->Result<()>{let command=app.world.as_mut().context("world")?.increase_job_level().context("production job-level helper refused")?;self.requests.push(json!({"target":self.own,"label":label,"kind":"helper"}));app.send(command);Ok(())}
    pub fn tick(&mut self,app:&mut App)->Result<bool>{
        ensure!(self.started.elapsed()<Duration::from_secs(120),"job stage {} timed out",self.stage);ensure!(app.error.is_none()&&app.addons.error.is_none()&&app.addons.runtime.errors().is_empty(),"client/addon error");
        let peer_role=if self.role=="owner"{"witness"}else{"owner"};if let Some(done)=read(&self.directory.join(format!("{peer_role}.json"))){ensure!(done["success"]==true,"peer failed");}
        if self.stage==20 {if app.world.is_none()&&self.events.get("PLAYER_LOGOUT")==Some(&1){self.record(app,"FirstLobby");self.mark(&format!("{}.saved_lobby",self.role))?;self.advance(21);}return Ok(false);}
        if self.stage==21 {if self.ready("saved-verified"){app.send(NetCmd::EnterWorld(self.character.clone()));self.advance(22);}return Ok(false);}
        if self.stage==25 {if app.world.is_none()&&self.events.get("PLAYER_LOGOUT")==Some(&2){self.record(app,"Logout");self.stage=26;return Ok(true);}return Ok(false);}
        let Some(w)=app.world.as_ref() else{return Ok(false);};if app.loading.is_some(){return Ok(false);}
        let Some(p)=w.job_progress().cloned() else{return Ok(false);};let trace=app.combat_trace.as_ref().context("trace")?;ensure!(trace.errors.is_empty(),"trace decoding error");
        let n=trace.job_results.len();let owner=self.role=="owner";
        if self.stage==0 {
            if self.own==0 {
                let Some(peer)=w.visible_player(&self.peer) else{return Ok(false);};let Some(known)=w.learned_skills() else{return Ok(false);};ensure!(known.is_empty(),"fresh learning state");
                ensure!([p.job,p.job_level,p.jp as i32,p.cost.unwrap_or(-1) as i32]==[if owner{100}else{200},1,if owner{8}else{0},3],"fixture quote");
                let assets=app.assets.as_ref().context("assets")?;let book=assets.progression.as_ref().context("lossless archive progression")?;ensure!(book.counts()==(3923,300,200),"source table counts");
                ensure!(book.experience(300).context("row300")?.exp.to_string()=="272233208401492000","wide EXP changed");
                let stat=assets.jobs.as_ref().and_then(|j|j.by_id(p.job)).context("job resource")?.int("stati_id");ensure!(p.base==book.base(stat).map(|a|(stat,a)),"server/client base mismatch");
                self.own=w.addon_state().player.context("player")?.handle;self.initial=self.own;self.other=peer.handle;self.initial_other=peer.handle;app.auto_enter=None;app.world.as_mut().unwrap().show_skills(true);self.record(app,"Initial");self.mark(&format!("{}.ready",self.role))?;
            }
            if self.ready(&format!("{peer_role}.ready")){self.raw(app,self.other,"foreign_owner");if owner{self.raw(app,u32::MAX,"unknown_owner");}self.advance(1);}return Ok(false);
        }
        if self.stage==1&&n==if owner{2}else{1}{ensure!(trace.job_results.iter().all(|v|*v!=0)&&p.job_level==1&&p.jp==if owner{8}else{0},"forged level request changed state");self.record(app,"Probed");self.mark(&format!("{}.probed",self.role))?;self.advance(2);return Ok(false);}
        if self.stage==22 {
            let Some(peer)=w.visible_player(&self.peer).filter(|v|v.handle!=self.initial_other) else{return Ok(false);};let handle=w.addon_state().player.context("reentry")?.handle;
            if handle!=self.initial&&p.job_level==if owner{3}else{1}&&p.jp==0 {self.own=handle;self.other=peer.handle;app.world.as_mut().unwrap().show_skills(true);self.record(app,"Reentered");self.mark(&format!("{}.reentered",self.role))?;self.advance(23);}return Ok(false);
        }
        if self.stage==23&&self.since.elapsed()>Duration::from_millis(1200)&&self.ready(&format!("{peer_role}.reentered")){
            ensure!(p.job_level==if owner{3}else{1}&&p.jp==0&&p.status==status::JP,"reentry lost durable progress");ensure!(app.world.as_mut().unwrap().increase_job_level().is_none(),"disabled helper accepted");self.record(app,"Saved");self.capture("saved")?;self.mark(&format!("{}.finish_ready",self.role))?;self.advance(24);return Ok(false);
        }
        if self.stage==24&&self.ready(&format!("{peer_role}.finish_ready")){app.send(NetCmd::ReturnLobby);self.advance(25);return Ok(false);}
        if owner {
            match self.stage {
                2 if self.ready("release-progression")=>{self.increase(app,"level2")?;self.advance(3);}
                3 if n==3&&p.job_level==2&&p.jp==5=>{ensure!(trace.job_results[2]==0&&p.cost==Some(5),"first cost");self.record(app,"Level2");self.mark("owner.level2")?;self.advance(4);}
                4 if self.ready("level2-verified")=>{self.increase(app,"level3")?;self.advance(5);}
                5 if n==4&&p.job_level==3&&p.jp==0=>{ensure!(trace.job_results[3]==0&&p.cost==Some(8)&&p.status==status::JP,"second cost");self.record(app,"Level3");self.mark("owner.level3")?;ensure!(app.world.as_mut().unwrap().increase_job_level().is_none(),"insufficient helper accepted");self.raw(app,self.own,"insufficient_jp");self.advance(6);}
                6 if n==5=>{ensure!(trace.job_results[4]!=0&&p.job_level==3&&p.jp==0,"overdraw");self.record(app,"Exhausted");self.mark("owner.exhausted")?;self.advance(7);}
                7 if self.ready("witness.denied")&&self.ready("level3-verified")=>{self.record(app,"BeforeLobby");self.mark("owner.ready_lobby")?;app.send(NetCmd::ReturnLobby);self.advance(20);}
                _=>{}
            }
        } else {
            match self.stage {
                2 if self.ready("release-progression")=>{ensure!(app.world.as_mut().unwrap().increase_job_level().is_none(),"witness helper accepted");self.raw(app,self.own,"insufficient_jp");self.advance(3);}
                3 if n==2=>{ensure!(trace.job_results[1]!=0&&p.job_level==1&&p.jp==0,"witness changed");self.record(app,"Denied");self.mark("witness.denied")?;self.advance(4);}
                4 if self.ready("owner.ready_lobby")=>{self.record(app,"BeforeLobby");app.send(NetCmd::ReturnLobby);self.advance(20);}
                _=>{}
            }
        }
        Ok(false)
    }
    pub fn fail(&mut self,error:anyhow::Error){self.failure=Some(format!("{error:#}"));let _=self.capture("failure");}
    pub fn finish(&self,app:&App)->Result<bool>{let success=self.stage==26&&self.failure.is_none();mark(&self.directory,&format!("{}.json",self.role),json!({"scenario":"job-progression","role":self.role,"success":success,"failure":self.failure,"own":self.own,"initial":self.initial,"other":self.other,"initial_other":self.initial_other,"requests":self.requests,"steps":self.steps,"events":self.events,"messages":self.messages,"profile_counts":[3923,300,200],"wide_exp300":"272233208401492000","trace":app.combat_trace.as_ref().context("trace")?.json()}))?;Ok(success)}
}

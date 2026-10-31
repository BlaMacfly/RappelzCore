//! Two real clients: expired corpse, competing casts, consumption and original respawn timer.
use super::{game,id,mark,read,unit};
use crate::{App,NetCmd};
use anyhow::{ensure,Context,Result};
use rz_proto::Message;
use serde_json::{json,Value};
use std::{collections::BTreeMap,path::PathBuf,time::{Duration,Instant}};

pub struct CorpseSkillValidation {
    directory:PathBuf,role:String,peer:String,character:String,stage:u8,started:Instant,since:Instant,
    own:u32,initial:u32,other:u32,initial_other:u32,body:u32,announced:bool,
    requests:Vec<Value>,steps:Vec<Value>,events:BTreeMap<String,usize>,messages:Vec<String>,failure:Option<String>,
}
impl CorpseSkillValidation {
    pub fn new(plan:PathBuf,app:&App)->Result<Self>{
        super::super::guard(app)?;let p:Value=serde_json::from_slice(&std::fs::read(plan)?)?;
        let role=p["role"].as_str().context("corpse role")?.to_string();ensure!(role=="owner"||role=="witness","role");
        let peer=p["peer"].as_str().context("peer")?.to_string();ensure!(peer.starts_with("Lua"),"dedicated peer");
        let directory=PathBuf::from(p["directory"].as_str().context("directory")?);std::fs::create_dir_all(&directory)?;
        ensure!(!directory.join(format!("{role}.json")).exists(),"fresh attempt required");
        Ok(Self{directory,role,peer,character:app.auto_enter.clone().context("character")?,stage:0,started:Instant::now(),since:Instant::now(),own:0,initial:0,other:0,initial_other:0,body:0,announced:false,requests:vec![],steps:vec![],events:BTreeMap::new(),messages:vec![],failure:None})
    }
    pub fn message(&mut self,line:&str){
        if let Some(event)=line.strip_prefix("[RappelzDebug] EVENT ").and_then(|s|s.split_whitespace().next()){*self.events.entry(event.into()).or_default()+=1;}
        if self.messages.len()<512 {self.messages.push(line.into());}
    }
    fn advance(&mut self,stage:u8){self.stage=stage;self.since=Instant::now();}
    fn ready(&self,key:&str)->bool{read(&self.directory.join(key)).is_some()}
    fn mark(&self,key:&str)->Result<()>{mark(&self.directory,key,json!({"ready":true}))}
    fn record(&mut self,app:&App,name:&str){
        let w=app.world.as_ref();let s=w.map(|w|w.addon_state()).unwrap_or_default();
        self.steps.push(json!({"stage":name,"elapsed_ms":self.started.elapsed().as_millis(),"player":s.player.as_ref().map(unit),"target":s.target.as_ref().map(unit),
            "body":w.and_then(|w|w.unit(self.body)).as_ref().map(unit),"body_renderable":w.and_then(|w|w.unit_renderable(self.body)),
            "learned":s.learned_skills,"jp":s.job_points,"xp_jp":w.map(|w|w.experience()),"money":s.money,"bag_revision":s.bag_revision,
            "remaining":w.map(|w|w.skill_remaining(3212)),"casting":w.map(|w|w.is_casting()),"position":w.and_then(|w|w.unit_position(self.own)),
            "display":super::super::AddonValidation::display(app),"events":self.events}));
    }
    fn capture(&self,name:&str)->Result<()>{let p=self.directory.join(format!("{name}.png"));macroquad::prelude::get_screen_data().export_png(p.to_str().context("capture")?);Ok(())}
    fn request(&mut self,app:&App,mut r:game::CsSkill,label:&str){
        r.layer=0;self.requests.push(json!({"label":label,"id":r.skill_id,"level":r.skill_level,"caster":r.caster,"target":r.target,"layer":r.layer}));app.send(NetCmd::Raw(r.to_packet()));
    }
    fn raw(&mut self,app:&App,label:&str){self.request(app,game::CsSkill{skill_id:3212,skill_level:1,caster:self.own,target:self.body,..Default::default()},label);}
    fn query(&self,app:&App){app.send(NetCmd::Raw(game::Handle::<{id::TS_CS_QUERY}>(self.body).to_packet()));}
    fn select(&self,app:&mut App)->Result<()>{let c=app.world.as_mut().context("world")?.select_target(self.body).context("visible monster")?;app.send(c);Ok(())}
    fn attack(&self,app:&mut App)->Result<()>{self.select(app)?;app.send(app.world.as_ref().unwrap().attack_target(self.body).context("normal attack")?);Ok(())}
    fn cast(&mut self,app:&App,label:&str)->Result<()>{
        let q=app.assets.as_ref().context("assets")?.restoration_quote(3212,1).context("archive corpse profile")?;
        ensure!(q.corpse && q.amount==320 && q.cooldown==5800,"wrong source profile");
        let NetCmd::Raw(packet)=app.world.as_ref().context("world")?.cast_skill(&q).context("production helper refused corpse")? else {anyhow::bail!("cast command");};
        self.request(app,game::CsSkill::from_packet(&packet)?,label);Ok(())
    }
    pub fn tick(&mut self,app:&mut App)->Result<bool>{
        ensure!(app.error.is_none() && app.addons.error.is_none() && app.addons.runtime.errors().is_empty(),"client/addon error");
        ensure!(self.started.elapsed()<Duration::from_secs(180),"corpse stage {} timed out",self.stage);
        let trace=app.combat_trace.as_ref().context("trace")?;ensure!(trace.errors.is_empty(),"trace decoding error");
        let peer_role=if self.role=="owner" {"witness"} else {"owner"};
        if let Some(done)=read(&self.directory.join(format!("{peer_role}.json"))){ensure!(done["success"]==true,"peer failed");}
        if self.stage==17 {
            let count=if self.role=="owner" {2}else{1};
            if app.world.is_none() && self.events.get("PLAYER_LOGOUT")==Some(&count){self.record(app,"Logout");self.stage=18;return Ok(true);}return Ok(false);
        }
        if self.role=="owner" && self.stage==9 {
            if app.world.is_none() && self.events.get("PLAYER_LOGOUT")==Some(&1){app.send(NetCmd::EnterWorld(self.character.clone()));self.advance(10);}return Ok(false);
        }
        let Some(w)=app.world.as_ref() else{return Ok(false);};if app.loading.is_some(){return Ok(false);}
        let Some(known)=w.learned_skills() else{return Ok(false);};let Some(_catalog)=w.skill_catalog() else{return Ok(false);};
        let alive=w.unit(self.body).is_some_and(|u|u.health.is_some_and(|hp|hp>0));let dead=w.unit(self.body).is_some_and(|u|u.health==Some(0));
        let phase_count=|phase:&str,caster:u32|trace.cast_events.iter().filter(|e|e["phase"]==phase && e["caster"]==caster).count();
        let leaves=trace.leaves.iter().filter(|h|**h==self.body).count();let replies=trace.cast_results.len();
        if self.stage==0 {
            if !self.announced {
                let Some(peer)=w.visible_player(&self.peer) else{return Ok(false);};let Some(monster)=w.living_monster() else{return Ok(false);};
                ensure!(known.get(&3212)==Some(&1) && known.len()==2,"dedicated learned fixture");
                self.own=w.addon_state().player.context("player")?.handle;self.initial=self.own;self.other=peer.handle;self.initial_other=peer.handle;self.body=monster.handle;
                app.auto_enter=None;app.world.as_mut().unwrap().focus_skill(3212);self.select(app)?;self.record(app,"Initial");self.mark(&format!("{}.ready",self.role))?;self.announced=true;
            }
            if !self.ready(&format!("{peer_role}.ready")){return Ok(false);}
            let base=game::CsSkill{skill_id:3212,skill_level:1,caster:self.own,target:self.body,..Default::default()};
            if self.role=="owner" {
                let npc=app.world.as_ref().unwrap().visible_npc().context("visible NPC probe")?.handle;
                for (r,label) in [(game::CsSkill{target:self.own,..base.clone()},"self"),(game::CsSkill{target:self.other,..base.clone()},"player"),(base.clone(),"living"),(game::CsSkill{target:0x2fffffff,..base.clone()},"unknown"),(game::CsSkill{target:npc,..base.clone()},"npc"),(game::CsSkill{caster:self.other,..base},"foreign_caster")] {self.request(app,r,label);}
            }else{self.request(app,game::CsSkill{target:self.own,..base.clone()},"self");self.request(app,base,"living");}
            self.advance(1);return Ok(false);
        }
        if self.role=="witness" {
            match self.stage {
                1=>{
                    if replies>=2 {ensure!(trace.cast_results[..2].iter().all(|r|*r!=0),"non-corpse cast accepted");}
                    if replies==2 && leaves>=1 && w.unit(self.body).is_none(){self.record(app,"ExpiredHidden");self.query(app);self.raw(app,"expired");self.advance(2);}
                }
                2 if replies==3=>{ensure!(trace.cast_results[2]!=0,"expired body accepted");self.record(app,"ExpiredQueryRefused");self.mark("witness.expired_query")?;self.advance(3);}
                3=>{if self.ready("owner.racing") && dead {self.select(app)?;self.record(app,"Racing");self.cast(app,"race")?;self.advance(4);}}
                4=>{if phase_count("CANCEL",self.own)==1 && leaves>=2 && w.unit(self.body).is_none(){self.record(app,"RaceLost");self.query(app);self.raw(app,"consumed");self.advance(5);}}
                5=>{if replies==5 && self.ready("owner.reentered") {
                    ensure!(trace.cast_results[4]!=0,"consumed body accepted");let Some(peer)=w.visible_player(&self.peer).filter(|p|p.handle!=self.initial_other) else{return Ok(false);};
                    self.other=peer.handle;self.record(app,"PeerReload");self.mark("witness.reloaded")?;self.advance(6);
                }}
                6=>{if alive && self.ready("owner.alive_probed"){self.select(app)?;self.raw(app,"respawn_alive");self.advance(7);}}
                7=>{if replies==6 && self.ready("owner.alive_denied"){ensure!(trace.cast_results[5]!=0,"living respawn accepted");self.record(app,"Respawn2");self.attack(app)?;self.advance(8);}}
                8=>{if dead && self.ready("owner.cooldown_denied"){self.record(app,"Kill3");self.cast(app,"fresh_corpse")?;self.advance(9);}}
                9=>{if phase_count("FIRE",self.own)==1 && w.addon_state().player.as_ref().is_some_and(|p|p.mana==Some(140)) && leaves>=3 {
                    self.record(app,"SecondConsumed");self.capture("witness-consumed")?;self.mark("witness.done")?;self.advance(10);
                }}
                10=>{if self.ready("owner.done") && w.unit(self.other).is_none(){self.record(app,"Saved");app.send(NetCmd::ReturnLobby);self.advance(17);}}
                _=>{}
            }
            return Ok(false);
        }
        match self.stage {
            1 if replies==6=>{ensure!(trace.cast_results.iter().all(|r|*r!=0),"non-corpse cast accepted");self.record(app,"Refused");self.attack(app)?;self.advance(2);}
            2 if dead=>{self.record(app,"Kill1");self.advance(3);}
            3 if self.since.elapsed()>=Duration::from_millis(2050)=>{ensure!(dead,"corpse disappeared too early");self.record(app,"LateCast");self.cast(app,"late_expiring")?;self.advance(4);}
            4=>{if replies==7 {ensure!(trace.cast_results[6]==0,"valid late corpse start denied");}
                if phase_count("CANCEL",self.own)==1 && w.unit(self.body).is_none(){ensure!(w.skill_remaining(3212)==0.,"canceled cast charged cooldown");self.record(app,"Expired");self.advance(5);}
            }
            5 if alive && self.ready("witness.expired_query")=>{self.record(app,"Respawn1");self.attack(app)?;self.advance(6);}
            6 if dead=>{self.record(app,"Kill2");self.cast(app,"race")?;self.advance(7);}
            7=>{
                if phase_count("CASTING",self.own)==2 && w.is_casting() && !self.ready("owner.racing"){self.mark("owner.racing")?;}
                if phase_count("FIRE",self.own)==1 && w.addon_state().player.as_ref().is_some_and(|p|p.mana==Some(140)) && leaves>=2 {
                    self.record(app,"Consumed");self.capture("owner-consumed")?;self.query(app);self.raw(app,"consumed");self.advance(8);
                }
            }
            8 if replies==9=>{ensure!(trace.cast_results[8]!=0,"consumed body accepted");self.record(app,"ConsumedQueryRefused");app.send(NetCmd::ReturnLobby);self.advance(9);}
            10=>{if known.get(&3212)==Some(&1) && w.skill_remaining(3212)>0. {
                self.own=w.addon_state().player.context("reentered player")?.handle;ensure!(self.own!=self.initial,"new handle required");app.world.as_mut().unwrap().focus_skill(3212);self.record(app,"Reentered");self.mark("owner.reentered")?;self.advance(11);
            }}
            11 if alive && self.ready("witness.reloaded")=>{self.record(app,"Respawn2");self.select(app)?;self.raw(app,"respawn_alive");self.mark("owner.alive_probed")?;self.advance(12);}
            12 if replies==10=>{ensure!(trace.cast_results[9]!=0,"living respawn accepted");self.record(app,"AliveRefused");self.mark("owner.alive_denied")?;self.advance(13);}
            13 if dead=>{ensure!(w.skill_remaining(3212)>0.,"owner recharge unexpectedly elapsed");self.raw(app,"cooldown_fresh_body");self.advance(14);}
            14 if replies==11=>{ensure!(trace.cast_results[10]!=0,"cooldown bypassed with new corpse");self.record(app,"CooldownRefused");self.mark("owner.cooldown_denied")?;self.advance(15);}
            15 if self.ready("witness.done") && phase_count("FIRE",self.other)==1=>{self.record(app,"Saved");self.capture("owner-reentered")?;self.mark("owner.done")?;app.send(NetCmd::ReturnLobby);self.advance(17);}
            _=>{}
        }
        Ok(false)
    }
    pub fn fail(&mut self,error:anyhow::Error){self.failure=Some(format!("{error:#}"));let _=self.capture(&format!("{}-failure",self.role));}
    pub fn finish(&self,app:&App)->Result<bool>{
        let success=self.stage==18 && self.failure.is_none();
        mark(&self.directory,&format!("{}.json",self.role),json!({"scenario":"corpse_skill","role":self.role,"success":success,"failure":self.failure,"own":self.own,"initial":self.initial,"other":self.other,"initial_other":self.initial_other,"body":self.body,"requests":self.requests,"steps":self.steps,"events":self.events,"messages":self.messages,"trace":app.combat_trace.as_ref().context("trace")?.json()}))?;Ok(success)
    }
}

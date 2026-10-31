//! Real ordinary observers cross the visibility boundary during archived corpse casts.
use super::super::{game,id,mark,read,unit};
use crate::{App,NetCmd};
use anyhow::{ensure,Context,Result};
use rz_proto::Message;
use serde_json::{json,Value};
use std::{collections::BTreeMap,path::PathBuf,time::{Duration,Instant,SystemTime,UNIX_EPOCH}};

pub struct LateCastValidation {
    directory:PathBuf,role:String,peer:String,stage:u8,started:Instant,since:Instant,
    own:u32,other:u32,body:u32,near:(f32,f32),far:(f32,f32),cancel_sent:bool,
    steps:Vec<Value>,events:BTreeMap<String,usize>,messages:Vec<String>,failure:Option<String>,
}
impl LateCastValidation {
    pub fn new(p:&Value,app:&App)->Result<Self>{
        super::super::super::guard(app)?;
        let point=|key:&str|->Result<(f32,f32)>{let a=p[key].as_array().context("movement point")?;ensure!(a.len()==2,"two coordinates");let x=a[0].as_f64().context("x")? as f32;let y=a[1].as_f64().context("y")? as f32;ensure!(x.is_finite()&&y.is_finite()&&(9000.0..11000.0).contains(&x)&&(7000.0..7500.0).contains(&y),"isolated fixture region");Ok((x,y))};
        let role=p["role"].as_str().context("role")?.to_owned();ensure!(role=="owner"||role=="witness","role");
        let peer=p["peer"].as_str().context("peer")?.to_owned();ensure!(peer.starts_with("Lua"),"dedicated peer");
        let directory=PathBuf::from(p["directory"].as_str().context("directory")?);ensure!(!directory.join(format!("{role}.json")).exists(),"fresh attempt");
        Ok(Self{directory,role,peer,near:point("near")?,far:point("far")?,stage:0,started:Instant::now(),since:Instant::now(),own:0,other:0,body:0,cancel_sent:false,steps:vec![],events:BTreeMap::new(),messages:vec![],failure:None})
    }
    pub fn message(&mut self,line:&str){
        if let Some(event)=line.strip_prefix("[RappelzDebug] EVENT ").and_then(|s|s.split_whitespace().next()){*self.events.entry(event.into()).or_default()+=1;}
        if self.messages.len()<512 {self.messages.push(line.into());}
    }
    fn advance(&mut self,stage:u8){self.stage=stage;self.since=Instant::now();}
    fn ready(&self,key:&str)->bool{read(&self.directory.join(key)).is_some()}
    fn mark(&self,key:&str)->Result<()>{mark(&self.directory,key,json!({"unix_ms":SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis(),"handle":self.own}))}
    fn record(&mut self,app:&App,name:&str){let w=app.world.as_ref();let s=w.map(|w|w.addon_state()).unwrap_or_default();
        self.steps.push(json!({"stage":name,"elapsed_ms":self.started.elapsed().as_millis(),"player":s.player.as_ref().map(unit),"peer":w.and_then(|w|w.unit(self.other)).as_ref().map(unit),"body":w.and_then(|w|w.unit(self.body)).as_ref().map(unit),"position":w.and_then(|w|w.unit_position(self.own)),"xp_jp":w.map(|w|w.experience()),"money":s.money,"remaining":w.map(|w|w.skill_remaining(3212)),"casting":w.map(|w|w.is_casting()),"skill_event":s.skill_event.as_ref().map(|e|json!({"sequence":e.sequence,"skill":e.skill,"phase":e.phase,"caster":e.caster,"amount":e.amount})),"events":self.events,"display":super::super::super::AddonValidation::display(app)}));
    }
    fn capture(&self,label:&str)->Result<()>{let p=self.directory.join(format!("{}-{label}.png",self.role));macroquad::prelude::get_screen_data().export_png(p.to_str().context("capture")?);Ok(())}
    fn at(&self,app:&App,p:(f32,f32))->bool{app.world.as_ref().and_then(|w|w.unit_position(self.own)).is_some_and(|v|(v.0-p.0).abs()<1.&&(v.1-p.1).abs()<1.)}
    fn move_to(&self,app:&mut App,p:(f32,f32))->Result<()>{let command=app.world.as_mut().context("world")?.move_path(vec![rz_proto::msg::MovePoint{x:p.0,y:p.1}]).context("ordinary movement")?;app.send(command);Ok(())}
    fn attack(&self,app:&mut App)->Result<()>{let command=app.world.as_mut().context("world")?.select_target(self.body).context("target")?;app.send(command);app.send(app.world.as_ref().unwrap().attack_target(self.body).context("attack")?);Ok(())}
    fn cast(&self,app:&mut App)->Result<()>{
        let q=app.assets.as_ref().context("assets")?.restoration_quote(3212,1).context("archive profile")?;ensure!(q.corpse&&q.cast==145&&q.cooldown==5800&&q.amount==320,"archive rule");
        let command=app.world.as_mut().context("world")?.select_target(self.body).context("corpse")?;app.send(command);app.send(app.world.as_ref().unwrap().cast_skill(&q).context("production cast helper")?);Ok(())
    }
    pub fn tick(&mut self,app:&mut App)->Result<bool>{
        ensure!(app.error.is_none()&&app.addons.error.is_none()&&app.addons.runtime.errors().is_empty(),"client/addon error");
        ensure!(self.started.elapsed()<Duration::from_secs(120),"late observer stage {} timed out",self.stage);
        let peer_role=if self.role=="owner"{"witness"}else{"owner"};if let Some(done)=read(&self.directory.join(format!("{peer_role}.json"))){ensure!(done["success"]==true,"peer failed");}
        if self.stage==20 {if app.world.is_none()&&self.events.get("PLAYER_LOGOUT")==Some(&1){self.record(app,"Logout");self.stage=21;return Ok(true);}return Ok(false);}
        let Some(w)=app.world.as_ref() else{return Ok(false);};if app.loading.is_some(){return Ok(false);}
        let trace=app.combat_trace.as_ref().context("trace")?;ensure!(trace.errors.is_empty(),"trace errors");
        let phase=|name:&str|trace.cast_events.iter().filter(|e|e["phase"]==name&&e["caster"]==if self.role=="owner"{self.own}else{self.other}).count();
        let hidden=w.unit(self.other).is_none()&&w.unit(self.body).is_none();
        let alive=w.unit(self.body).is_some_and(|u|u.health.is_some_and(|v|v>0));let dead=w.unit(self.body).is_some_and(|u|u.health==Some(0));
        if self.stage==0 {
            if self.own==0 {let Some(peer)=w.visible_player(&self.peer) else{return Ok(false);};let Some(body)=w.living_monster() else{return Ok(false);};
                let Some(known)=w.learned_skills() else{return Ok(false);};ensure!(known.len()==2&&known.get(&3212)==Some(&1),"learned fixture");
                self.own=w.addon_state().player.context("player")?.handle;self.other=peer.handle;self.body=body.handle;app.auto_enter=None;self.record(app,"Initial");self.mark(&format!("{}.ready",self.role))?;
            }
            if self.ready(&format!("{peer_role}.ready")){if self.role=="witness"{self.move_to(app,self.far)?;}self.advance(1);}return Ok(false);
        }
        if self.role=="witness" {
            match self.stage {
                1 if hidden&&self.at(app,self.far)=>{ensure!(trace.cast_events.is_empty(),"hidden skill leak");self.record(app,"Hidden1");self.mark("witness.hidden1")?;self.advance(2);}
                2 if self.ready("owner.cast1")=>{ensure!(trace.cast_events.is_empty(),"hidden cast leaked");self.move_to(app,self.near)?;self.advance(3);}
                3 if phase("CASTING")==1=>{ensure!(dead&&w.unit(self.other).is_some(),"entities must precede late cast");self.record(app,"Late1");self.capture("late1")?;for h in [self.other,self.body,self.other,self.body]{app.send(NetCmd::Raw(game::Handle::<{id::TS_CS_QUERY}>(h).to_packet()));}self.mark("witness.seen1")?;self.advance(4);}
                4 if phase("CANCEL")==1=>{ensure!(phase("FIRE")==0&&w.skill_remaining(3212)==0.,"canceled cast changed witness");self.record(app,"Canceled");self.move_to(app,self.far)?;self.advance(5);}
                5 if hidden&&self.at(app,self.far)=>{self.record(app,"Hidden2");self.mark("witness.hidden2")?;self.advance(6);}
                6 if self.ready("owner.cast2")=>{ensure!(trace.cast_events.len()==2,"hidden second cast leaked");self.move_to(app,self.near)?;self.advance(7);}
                7 if phase("CASTING")==2=>{ensure!(dead&&w.unit(self.other).is_some(),"entities must precede second late cast");self.record(app,"Late2");self.mark("witness.seen2")?;self.advance(8);}
                8 if phase("FIRE")==1&&w.unit(self.body).is_none()=>{ensure!(w.skill_remaining(3212)==0.&&w.experience()==(0,0),"private state leaked");self.record(app,"Consumed");self.capture("consumed")?;self.move_to(app,self.far)?;self.advance(9);}
                9 if hidden&&self.at(app,self.far)=>{self.record(app,"HiddenAfter");self.mark("witness.hidden_after")?;self.move_to(app,self.near)?;self.advance(10);}
                10 if self.at(app,self.near)&&w.unit(self.other).is_some_and(|p|p.mana==Some(140))&&self.since.elapsed()>Duration::from_millis(1200)=>{ensure!(trace.cast_events.len()==4,"old effect replay");self.record(app,"Quiet");self.capture("quiet")?;self.mark("witness.done")?;self.advance(11);}
                11 if self.ready("owner.done")=>{self.record(app,"Saved");app.send(NetCmd::ReturnLobby);self.advance(20);}
                _=>{}
            }
        } else {
            match self.stage {
                1 if self.ready("witness.hidden1")=>{self.attack(app)?;self.advance(2);}
                2 if dead=>{self.record(app,"Kill1");self.cast(app)?;self.advance(3);}
                3 if phase("CASTING")==1=>{self.mark("owner.cast1")?;self.advance(4);}
                4=>{if self.ready("witness.seen1")&&!self.cancel_sent{app.send(w.cancel_skill());self.cancel_sent=true;}
                    if phase("CANCEL")==1{ensure!(w.skill_remaining(3212)==0.&&phase("FIRE")==0,"canceled cast charged");self.record(app,"Canceled");self.advance(5);}}
                5 if self.ready("witness.hidden2")&&alive=>{self.attack(app)?;self.advance(6);}
                6 if dead=>{self.record(app,"Kill2");self.cast(app)?;self.advance(7);}
                7 if phase("CASTING")==2=>{self.mark("owner.cast2")?;self.advance(8);}
                8 if phase("FIRE")==1&&w.unit(self.body).is_none()=>{ensure!(w.skill_remaining(3212)>0.&&w.addon_state().player.is_some_and(|p|p.mana==Some(140)),"missing confirmed restoration");self.record(app,"Consumed");self.capture("consumed")?;self.advance(9);}
                9 if self.ready("witness.done")&&w.unit(self.other).is_some()=>{ensure!(trace.cast_events.len()==4&&w.experience()==(10,6),"duplicated effect/reward");self.record(app,"Quiet");self.capture("quiet")?;self.mark("owner.done")?;self.advance(10);}
                10=>{self.record(app,"Saved");app.send(NetCmd::ReturnLobby);self.advance(20);}
                _=>{}
            }
        }
        Ok(false)
    }
    pub fn fail(&mut self,error:anyhow::Error){self.failure=Some(format!("{error:#}"));let _=self.capture("failure");}
    pub fn finish(&self,app:&App)->Result<bool>{let success=self.stage==21&&self.failure.is_none();mark(&self.directory,&format!("{}.json",self.role),json!({"scenario":"late-cast","success":success,"failure":self.failure,"role":self.role,"own":self.own,"other":self.other,"body":self.body,"near":self.near,"far":self.far,"steps":self.steps,"events":self.events,"messages":self.messages,"trace":app.combat_trace.as_ref().context("trace")?.json()}))?;Ok(success)}
}

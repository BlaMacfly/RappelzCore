//! Real owner-only stat refresh, XP level-up, peer privacy and persisted reentry.
use super::super::{game,id,mark,read,unit};
use crate::{App,NetCmd};
use anyhow::{ensure,Context,Result};
use rz_proto::Message;
use serde_json::{json,Value};
use std::{collections::BTreeMap,path::PathBuf,time::{Duration,Instant}};
pub struct StatsValidation {
    directory:PathBuf,role:String,peer:String,character:String,stage:u8,started:Instant,since:Instant,
    own:u32,initial:u32,other:u32,initial_other:u32,body:u32,steps:Vec<Value>,requests:Vec<Value>,events:BTreeMap<String,usize>,messages:Vec<String>,failure:Option<String>,
}
impl StatsValidation {
    pub fn new(p:&Value,app:&App)->Result<Self>{
        super::super::super::guard(app)?;let role=p["role"].as_str().context("role")?.to_owned();ensure!(role=="owner"||role=="witness","role");let peer=p["peer"].as_str().context("peer")?.to_owned();ensure!(peer.starts_with("Lua"),"dedicated peer");
        let directory=PathBuf::from(p["directory"].as_str().context("directory")?);ensure!(!directory.join(format!("{role}.json")).exists(),"fresh attempt");
        Ok(Self{directory,role,peer,character:app.auto_enter.clone().context("character")?,stage:0,started:Instant::now(),since:Instant::now(),own:0,initial:0,other:0,initial_other:0,body:0,steps:vec![],requests:vec![],events:BTreeMap::new(),messages:vec![],failure:None})
    }
    pub fn message(&mut self,line:&str){if let Some(e)=line.strip_prefix("[RappelzDebug] EVENT ").and_then(|s|s.split_whitespace().next()){*self.events.entry(e.into()).or_default()+=1;}if self.messages.len()<512{self.messages.push(line.into());}}
    fn ready(&self,key:&str)->bool{read(&self.directory.join(key)).is_some()}
    fn mark(&self,key:&str)->Result<()>{mark(&self.directory,key,json!({"handle":self.own,"elapsed_ms":self.started.elapsed().as_millis()}))}
    fn advance(&mut self,stage:u8){self.stage=stage;self.since=Instant::now();}
    fn record(&mut self,app:&App,name:&str){let w=app.world.as_ref();let state=w.map(|w|w.addon_state()).unwrap_or_default();self.steps.push(json!({"stage":name,"elapsed_ms":self.started.elapsed().as_millis(),"player":state.player.as_ref().map(unit),"peer":w.and_then(|w|w.unit(self.other)).as_ref().map(unit),"stats":state.combat_stats.map(|s|json!({"attack":s.attack,"defence":s.defence,"speed":s.move_speed})),"xp_jp":w.map(|w|w.experience()),"money":state.money,"position":w.and_then(|w|w.unit_position(self.own)),"events":self.events,"display":super::super::super::AddonValidation::display(app)}));}
    fn capture(&self,label:&str)->Result<()>{let p=self.directory.join(format!("{}-{label}.png",self.role));macroquad::prelude::get_screen_data().export_png(p.to_str().context("capture")?);Ok(())}
    fn refresh(&mut self,app:&App,label:&str)->Result<()>{let cmd=app.world.as_ref().context("world")?.request_combat_stats().context("production refresh")?;self.requests.push(json!({"label":label,"target":self.own,"helper":true}));app.send(cmd);Ok(())}
    fn foreign(&mut self,app:&App){self.requests.push(json!({"label":"peer_query","target":self.other,"helper":false}));app.send(NetCmd::Raw(game::Handle::<{id::TS_CS_QUERY}>(self.other).to_packet()));}
    fn attack(&self,app:&mut App)->Result<()>{let cmd=app.world.as_mut().context("world")?.select_target(self.body).context("target")?;app.send(cmd);app.send(app.world.as_ref().unwrap().attack_target(self.body).context("ordinary attack")?);Ok(())}
    pub fn tick(&mut self,app:&mut App)->Result<bool>{
        ensure!(self.started.elapsed()<Duration::from_secs(120),"stats stage {} timed out",self.stage);ensure!(app.error.is_none()&&app.addons.error.is_none()&&app.addons.runtime.errors().is_empty(),"client/addon error");
        let owner=self.role=="owner";let peer_role=if owner{"witness"}else{"owner"};if let Some(v)=read(&self.directory.join(format!("{peer_role}.json"))){ensure!(v["success"]==true,"peer failed");}
        if self.stage==20 {if app.world.is_none()&&self.events.get("PLAYER_LOGOUT")==Some(&1){self.record(app,"FirstLobby");self.mark(&format!("{}.saved_lobby",self.role))?;self.advance(21);}return Ok(false);}
        if self.stage==21 {if self.ready("saved-verified"){app.send(NetCmd::EnterWorld(self.character.clone()));self.advance(22);}return Ok(false);}
        if self.stage==25 {if app.world.is_none()&&self.events.get("PLAYER_LOGOUT")==Some(&2){self.record(app,"Logout");self.stage=26;return Ok(true);}return Ok(false);}
        let Some(w)=app.world.as_ref()else{return Ok(false);};if app.loading.is_some(){return Ok(false);}
        let Some(stats)=w.combat_stats()else{return Ok(false);};let trace=app.combat_trace.as_ref().context("trace")?;ensure!(trace.errors.is_empty(),"trace error");
        let count=self.events.get("RAPPELZ_COMBAT_STATS_CHANGED").copied().unwrap_or(0);
        if self.stage==0 {
            if self.own==0 {
                let Some(peer)=w.visible_player(&self.peer)else{return Ok(false);};let Some(body)=w.living_monster()else{return Ok(false);};let expected=if owner{(20,8)}else{(16,6)};
                ensure!((stats.attack,stats.defence,stats.move_speed)==(expected.0,expected.1,120),"initial combat stats disagree with real server combat: {:?}",stats);
                self.own=w.addon_state().player.context("player")?.handle;self.initial=self.own;self.other=peer.handle;self.initial_other=peer.handle;self.body=body.handle;app.auto_enter=None;app.world.as_mut().unwrap().show_character(true);self.record(app,"Initial");
                self.refresh(app,"refresh1")?;self.refresh(app,"refresh2")?;self.foreign(app);self.foreign(app);self.mark(&format!("{}.ready",self.role))?;
            }
            self.advance(1);return Ok(false);
        }
        if self.stage==1&&trace.stats.len()>=3&&self.since.elapsed()>Duration::from_millis(700){ensure!(trace.stats.len()==3&&count==1,"duplicate refresh event or private stat leak");self.record(app,"Queried");self.mark(&format!("{}.queries_done",self.role))?;self.advance(2);return Ok(false);}
        if self.stage==2&&self.ready("release-combat") {if owner{self.attack(app)?;}self.advance(3);return Ok(false);}
        if self.stage==3 {
            if owner {
                if w.experience()==(4,10)&&stats.attack==24&&stats.defence==10&&count==2 {
                    ensure!(trace.stats.len()==4,"level-up snapshot count");self.record(app,"Leveled");self.mark("owner.leveled")?;self.advance(4);
                }
            }else if self.ready("owner.leveled")&&w.unit(self.other).is_some_and(|p|p.level==Some(3)&&p.max_health==Some(140)&&p.max_mana==Some(70)) {
                ensure!((stats.attack,stats.defence,stats.move_speed)==(16,6,120)&&trace.stats.len()==3&&count==1,"peer update leaked into own combat stats");self.record(app,"Observed");self.mark("witness.observed")?;self.advance(4);
            }
            return Ok(false);
        }
        if self.stage==4&&self.since.elapsed()>Duration::from_millis(1200)&&self.ready(if owner{"witness.observed"}else{"owner.leveled"}) {
            self.record(app,"BeforeLobby");self.capture("leveled")?;self.mark(&format!("{}.lobby_ready",self.role))?;self.advance(5);return Ok(false);
        }
        if self.stage==5&&self.ready(&format!("{peer_role}.lobby_ready")){app.send(NetCmd::ReturnLobby);self.advance(20);return Ok(false);}
        if self.stage==22 {
            let Some(peer)=w.visible_player(&self.peer).filter(|v|v.handle!=self.initial_other)else{return Ok(false);};let handle=w.addon_state().player.context("reentry")?.handle;
            let expected=if owner{(24,10)}else{(16,6)};
            if handle!=self.initial&&(stats.attack,stats.defence,stats.move_speed)==(expected.0,expected.1,120){self.own=handle;self.other=peer.handle;app.world.as_mut().unwrap().show_character(true);self.refresh(app,"reentry_refresh")?;self.record(app,"Reentered");self.mark(&format!("{}.reentered",self.role))?;self.advance(23);}return Ok(false);
        }
        if self.stage==23&&self.since.elapsed()>Duration::from_millis(1200)&&self.ready(&format!("{peer_role}.reentered")){
            ensure!(trace.stats.len()==if owner{6}else{5},"missing or replayed private snapshot");ensure!(count==if owner{3}else{2},"duplicate stat callback");ensure!(w.addon_state().target.is_none(),"old target replayed");self.record(app,"Saved");self.capture("saved")?;self.mark(&format!("{}.finish_ready",self.role))?;self.advance(24);return Ok(false);
        }
        if self.stage==24&&self.ready(&format!("{peer_role}.finish_ready")){app.send(NetCmd::ReturnLobby);self.advance(25);}
        Ok(false)
    }
    pub fn fail(&mut self,error:anyhow::Error){self.failure=Some(format!("{error:#}"));let _=self.capture("failure");}
    pub fn finish(&self,app:&App)->Result<bool>{let success=self.stage==26&&self.failure.is_none();mark(&self.directory,&format!("{}.json",self.role),json!({"scenario":"character-stats","role":self.role,"success":success,"failure":self.failure,"own":self.own,"initial":self.initial,"other":self.other,"initial_other":self.initial_other,"body":self.body,"requests":self.requests,"steps":self.steps,"events":self.events,"messages":self.messages,"trace":app.combat_trace.as_ref().context("trace")?.json()}))?;Ok(success)}
}

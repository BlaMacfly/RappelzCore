//! Two real clients keep unsaved movement in world until administrator SOAP shutdown.
use super::{guard, combat::{mark, read}};
use crate::{App, NetCmd, Screen};
use anyhow::{ensure, Context, Result};
use rz_proto::msg::MovePoint;
use serde_json::{json, Value};
use std::{collections::BTreeMap, path::PathBuf, time::{Duration, Instant}};

pub struct SoapShutdownValidation {
    directory: PathBuf, role: String, peer: String, character: String, password: String,
    position: [f32;2], negative: bool, stage: u8, started: Instant, since: Instant,
    events: BTreeMap<String,usize>, messages: Vec<String>, steps: Vec<Value>, failure: Option<String>,
}
impl SoapShutdownValidation {
    pub fn new(plan: PathBuf, app: &App) -> Result<Self> {
        guard(app)?;let p:Value=serde_json::from_slice(&std::fs::read(plan)?)?;
        let role=p["role"].as_str().context("role")?.to_owned();ensure!(matches!(role.as_str(),"owner"|"witness"),"role");
        let peer=p["peer"].as_str().context("peer")?.to_owned();ensure!(peer.starts_with("Lua"),"dedicated peer");
        let directory=PathBuf::from(p["directory"].as_str().context("directory")?);ensure!(!directory.join(format!("{role}.json")).exists(),"fresh report");
        let position=[p["position"][0].as_f64().context("x")? as f32,p["position"][1].as_f64().context("y")? as f32];
        let password=crate::arg_value("-login").and_then(|v|v.split_once(':').map(|(_,p)|p.to_string())).context("dedicated login")?;
        Ok(Self{directory,role,peer,character:app.auto_enter.clone().context("character")?,password,position,negative:p["negative"]==true,stage:0,started:Instant::now(),since:Instant::now(),events:BTreeMap::new(),messages:vec![],steps:vec![],failure:None})
    }
    fn ready(&self,name:&str)->bool {read(&self.directory.join(name)).is_some()}
    fn mark(&self,name:&str)->Result<()> {mark(&self.directory,name,json!({"elapsed_ms":self.started.elapsed().as_millis()}))}
    fn advance(&mut self,stage:u8){self.stage=stage;self.since=Instant::now();}
    pub fn message(&mut self,line:&str){
        if let Some(e)=line.strip_prefix("[RappelzDebug] EVENT ").and_then(|s|s.split_whitespace().next()){*self.events.entry(e.into()).or_default()+=1;}
        if self.messages.len()<512{self.messages.push(line.into());}
    }
    fn record(&mut self,app:&App,name:&str){
        let w=app.world.as_ref();let s=w.map(|w|w.addon_state()).unwrap_or_default();
        self.steps.push(json!({"stage":name,"elapsed_ms":self.started.elapsed().as_millis(),"player":s.player.as_ref().map(|p|json!({"handle":p.handle,"name":p.name,"level":p.level,"hp":p.health,"mp":p.mana,"max_hp":p.max_health,"max_mp":p.max_mana})),
            "position":w.and_then(|w|w.unit_position(w.addon_state().player?.handle)),"peer":w.and_then(|w|w.visible_player(&self.peer)).map(|p|json!({"handle":p.handle,"name":p.name})),
            "peer_position":w.and_then(|w|w.visible_player(&self.peer).and_then(|p|w.unit_position(p.handle))),"xp_jp":w.map(|w|w.experience()),"money":s.money,"job":w.and_then(|w|w.job_progress()).map(|p|json!({"job":p.job,"jlv":p.job_level,"jp":p.jp,"base":p.base})),"events":self.events,"display":super::AddonValidation::display(app)}));
    }
    fn capture(&self,label:&str)->Result<()> {macroquad::prelude::get_screen_data().export_png(self.directory.join(format!("{}-{label}.png",self.role)).to_str().context("capture")?);Ok(())}
    fn progress_ready(&self,app:&App)->bool{
        let Some(w)=&app.world else{return false};let Some(p)=w.job_progress()else{return false};
        p.job==if self.role=="owner"{100}else{200} && p.job_level==if self.role=="owner"{2}else{1} && p.jp==if self.role=="owner"{5}else{0}
    }
    pub fn tick(&mut self,app:&mut App)->Result<bool>{
        ensure!(self.started.elapsed()<Duration::from_secs(150),"SOAP stage {} timed out",self.stage);
        ensure!(app.addons.error.is_none()&&app.addons.runtime.errors().is_empty(),"addon errors");
        ensure!(app.combat_trace.as_ref().context("trace")?.json()["errors"].as_array().context("trace errors")?.is_empty(),"decode errors");
        if !matches!(self.stage,4|5){ensure!(app.error.is_none(),"unexpected client error: {:?}",app.error);}
        match self.stage {
            0=>{
                if app.loading.is_some(){return Ok(false);}let Some(w)=&app.world else{return Ok(false)};
                if w.visible_player(&self.peer).is_none()||w.job_progress().is_none()||w.addon_state().money.is_none(){return Ok(false);}
                self.record(app,"Initial");app.auto_enter=None;
                if self.role=="owner"{let cmd=app.world.as_mut().unwrap().increase_job_level().context("job purchase")?;app.send(cmd);}
                self.advance(1);
            }
            1 if self.progress_ready(app)=>{self.record(app,"ProgressCommitted");self.mark(&format!("{}.progress",self.role))?;self.advance(2);}
            2 if self.ready("move-allowed")=>{
                let cmd=app.world.as_mut().context("world")?.move_path(vec![MovePoint{x:self.position[0],y:self.position[1]}]).context("production movement")?;app.send(cmd);self.advance(3);
            }
            3=>{
                let Some(w)=&app.world else{return Ok(false)};let own=w.addon_state().player.context("player")?.handle;
                if w.unit_position(own).is_some_and(|p|(p.0-self.position[0]).abs()<0.05&&(p.1-self.position[1]).abs()<0.05)&&self.since.elapsed()>Duration::from_millis(800){
                    self.record(app,"MovedLive");self.capture("live")?;self.mark(&format!("{}.shutdown-ready",self.role))?;self.advance(4);
                }
            }
            4 if app.world.is_none()=>{
                ensure!(matches!(app.screen,Screen::Login)&&self.events.get("PLAYER_LOGOUT")==Some(&1),"missing actual shutdown logout");
                self.record(app,"Disconnected");app.error=None;self.mark(&format!("{}.dropped",self.role))?;self.advance(5);
            }
            5=>{
                if self.negative&&self.ready("negative-verified"){self.advance(9);return Ok(true);}
                if self.ready("server-restarted"){
                    let(host,port)=crate::config::realmlist(&app.root,&app.config.get("locale","frFR"));
                    app.auto_enter=Some(self.character.clone());app.send(NetCmd::Login{host,port,account:app.account.clone(),password:self.password.clone()});self.advance(6);
                }
            }
            6 if self.progress_ready(app)&&app.loading.is_none()=>{
                let w=app.world.as_ref().unwrap();let own=w.addon_state().player.context("player")?.handle;
                if w.visible_player(&self.peer).is_some()&&w.unit_position(own).is_some_and(|p|(p.0-self.position[0]).abs()<0.05&&(p.1-self.position[1]).abs()<0.05){
                    app.auto_enter=None;self.record(app,"Reentered");self.capture("reentered")?;self.mark(&format!("{}.reentered",self.role))?;self.advance(7);
                }
            }
            7 if self.ready("reentry-verified")=>{app.send(NetCmd::ReturnLobby);self.advance(8);}
            8 if app.world.is_none()&&self.events.get("PLAYER_LOGOUT")==Some(&2)=>{self.record(app,"SavedLobby");self.advance(9);return Ok(true);}
            _=>{}
        }
        Ok(false)
    }
    pub fn fail(&mut self,error:anyhow::Error){self.failure=Some(format!("{error:#}"));let _=self.capture("failure");}
    pub fn finish(&self,app:&App)->Result<bool>{
        let success=self.stage==9&&self.failure.is_none();
        mark(&self.directory,&format!("{}.json",self.role),json!({"scenario":"soap-player-shutdown","role":self.role,"negative":self.negative,"success":success,"failure":self.failure,"steps":self.steps,"events":self.events,"messages":self.messages,"trace":app.combat_trace.as_ref().context("trace")?.json()}))?;Ok(success)
    }
}

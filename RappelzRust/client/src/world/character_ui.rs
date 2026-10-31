use super::*;
impl WorldState {
    pub fn combat_stats(&self)->Option<msg::CombatStats>{self.combat_stats}
    pub fn show_character(&mut self,show:bool){self.show_character=show;if show{self.show_skills=false;}}
    pub fn request_combat_stats(&self)->Option<NetCmd>{(!self.offline&&self.handle!=0).then(||NetCmd::Raw(game::Handle::<{gid::TS_CS_QUERY}>(self.handle).to_packet()))}
    pub(super) fn draw_character(&mut self,assets:&mut Assets,ui:&RzUi,out:&mut Vec<NetCmd>)->bool{
        let r=Rect::new((screen_width()-490.).max(8.),100.,480.,400.);let mouse:Vec2=mouse_position().into();
        draw_rectangle(r.x,r.y,r.w,r.h,Color::from_rgba(12,18,30,245));draw_rectangle_lines(r.x,r.y,r.w,r.h,1.,SKYBLUE);ui.label("Personnage (C)",r.x+12.,r.y+23.,SKYBLUE);
        let close=Rect::new(r.x+r.w-35.,r.y+5.,28.,25.);ui.label_in("X",close,WHITE);if close.contains(mouse)&&is_mouse_button_pressed(MouseButton::Left){self.show_character=false;}
        ui.label(&format!("{} · niveau {}",self.name,self.addon_level.map(|v|v.to_string()).unwrap_or_else(||"?".into())),r.x+12.,r.y+53.,WHITE);
        let job=self.job_progress.as_ref().map(|p|p.job).or_else(||self.skill_catalog.as_ref().map(|p|p.job));let jlv=self.job_progress.as_ref().map(|p|p.job_level).or_else(||self.skill_catalog.as_ref().map(|p|p.job_level));
        ui.label(&format!("{} · métier niv. {}",job.map(|id|assets.job_name(id)).unwrap_or_else(||"Métier inconnu".into()),jlv.map(|v|v.to_string()).unwrap_or_else(||"?".into())),r.x+12.,r.y+78.,LIGHTGRAY);
        ui.label(&format!("PV : {} / {}     PM : {} / {}",self.hp,self.max_hp,self.mp,self.max_mp),r.x+12.,r.y+111.,WHITE);
        ui.label(&format!("JP : {}     Monnaie : {}",self.addon_jp.map(|v|v.to_string()).unwrap_or_else(||"?".into()),self.addon_money.map(|v|v.to_string()).unwrap_or_else(||"?".into())),r.x+12.,r.y+136.,WHITE);
        if let Some(s)=self.combat_stats {
            ui.label(&format!("Attaque physique : {}",s.attack),r.x+12.,r.y+177.,WHITE);ui.label(&format!("Défense physique : {}",s.defence),r.x+12.,r.y+204.,WHITE);ui.label(&format!("Vitesse de déplacement : {}",s.move_speed),r.x+12.,r.y+231.,WHITE);
        }else{ui.label("En attente des caractéristiques…",r.x+12.,r.y+177.,LIGHTGRAY);}
        if let Some((_,a))=self.job_progress.as_ref().and_then(|p|p.base){
            ui.label("Base du métier",r.x+12.,r.y+274.,SKYBLUE);ui.label(&format!("FOR {} · VIT {} · DEX {} · AGI {}",a[0],a[1],a[2],a[3]),r.x+12.,r.y+297.,LIGHTGRAY);ui.label(&format!("INT {} · ESP {} · Chance {}",a[4],a[5],a[6]),r.x+12.,r.y+320.,LIGHTGRAY);
        }
        let refresh=Rect::new(r.x+r.w-142.,r.y+r.h-43.,130.,29.);draw_rectangle(refresh.x,refresh.y,refresh.w,refresh.h,Color::from_rgba(35,60,85,255));ui.label_in("Actualiser",refresh,WHITE);
        if refresh.contains(mouse)&&is_mouse_button_pressed(MouseButton::Left){if let Some(command)=self.request_combat_stats(){out.push(command);}}
        r.contains(mouse)
    }
}
#[cfg(test)]mod tests{
    use super::*;
    fn packet(handle:u32,attack:i32,defence:i32,move_speed:i16,kind:u8)->RawPacket{msg::ScStatInfo::from_stats(handle,&msg::BasicStats{attributes:[10;7],attack,defence,magic_point:0,magic_defence:0,move_speed,attack_speed:100,attack_range:50},kind).to_packet()}
    #[test]fn private_complete_snapshots_update_atomically_and_ignore_foreign_delta_or_malformed_packets(){
        let mut w=WorldState::from_login(&msg::ScLoginResult{handle:7,hp:120,max_hp:120,mp:60,max_mp:60,..Default::default()},Vec2::ZERO);assert!(w.combat_stats().is_none());
        for p in [packet(8,999,999,255,0),packet(7,20,8,120,1),packet(7,-1,8,120,0)]{w.on_packet(p,0.);assert!(w.combat_stats().is_none());}
        w.on_packet(packet(7,20,8,120,0),0.);let confirmed=w.addon_state().combat_stats.unwrap();assert_eq!((confirmed.attack,confirmed.defence,confirmed.move_speed),(20,8,120));
        let mut bad=packet(7,99,99,99,0);bad.body.push(0);w.on_packet(bad,0.);assert_eq!(w.addon_state().combat_stats,Some(confirmed));
        w.on_packet(packet(7,24,10,120,0),0.);assert_eq!(w.combat_stats().unwrap().attack,24);assert_eq!((w.hp,w.mp),(120,60));
    }
    #[test]fn character_refresh_queries_only_self_and_new_world_discards_old_stats(){
        let mut w=WorldState::from_login(&msg::ScLoginResult{handle:7,..Default::default()},Vec2::ZERO);w.show_skills=true;w.show_character(true);assert!(w.show_character&&!w.show_skills);
        let NetCmd::Raw(p)=w.request_combat_stats().unwrap()else{panic!("raw")};assert_eq!(game::CsQuery::from_packet(&p).unwrap().0,7);
        w.on_packet(packet(7,24,10,120,0),0.);let next=WorldState::from_login(&msg::ScLoginResult{handle:8,..Default::default()},Vec2::ZERO);assert!(next.combat_stats().is_none());w.offline=true;assert!(w.request_combat_stats().is_none());
    }
}

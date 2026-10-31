use super::*;
use rz_proto::job::{self,status};
impl WorldState {
    pub fn job_progress(&self)->Option<&job::ScJobProgress>{self.job_progress.as_ref()}
    /// The offer is advisory; the server checks ownership, live state, current cost and JP again.
    pub fn increase_job_level(&mut self)->Option<NetCmd>{
        let p=self.job_progress.as_ref()?;
        if self.offline || self.hp<=0 || self.job_pending || self.is_casting() || p.target!=self.handle || p.status!=status::READY {return None;}
        self.job_pending=true;Some(NetCmd::Raw(job::CsJobLevelUp{target:self.handle}.to_packet()))
    }
    pub(super) fn draw_job_progression(&mut self,p:&job::ScJobProgress,r:Rect,ui:&RzUi,out:&mut Vec<NetCmd>){
        let button=Rect::new(r.x+12.,r.y+62.,210.,26.);let enabled=p.status==status::READY&&!self.job_pending&&self.hp>0&&!self.is_casting();
        draw_rectangle(button.x,button.y,button.w,button.h,Color::from_rgba(35,60,85,255));ui.label_in("Augmenter le métier",button,if enabled{WHITE}else{GRAY});
        if enabled&&button.contains(mouse_position().into())&&is_mouse_button_pressed(MouseButton::Left){if let Some(command)=self.increase_job_level(){out.push(command);}}
        let note=match p.status {status::READY=>format!("{} JP",p.cost.unwrap_or(0)),status::JP=>format!("{} JP requis",p.cost.unwrap_or(0)),status::MAXIMUM=>"Niveau maximal atteint".into(),status::UNSUPPORTED=>"Progression non prise en charge".into(),_=>"Progression indisponible".into()};
        ui.label(&note,r.x+232.,r.y+81.,if p.status==status::READY{GREEN}else{YELLOW});
        if let Some((_,a))=p.base {
            ui.label(&format!("Base métier : FOR {} · VIT {} · DEX {} · AGI {}",a[0],a[1],a[2],a[3]),r.x+12.,r.y+109.,LIGHTGRAY);
            ui.label(&format!("INT {} · ESP {} · Chance {}",a[4],a[5],a[6]),r.x+12.,r.y+130.,LIGHTGRAY);
        }
    }
}
#[cfg(test)]mod tests{
    use super::*;
    #[test]fn authoritative_job_offer_is_private_and_suppresses_duplicate_or_disabled_requests(){
        let mut w=WorldState::from_login(&msg::ScLoginResult{handle:7,hp:100,max_hp:100,mp:20,max_mp:50,..Default::default()},Vec2::ZERO);
        let mut p=job::ScJobProgress{target:8,job:100,job_level:1,jp:8,cost:Some(3),status:status::READY,base:Some((100,[10;7]))};
        w.on_packet(p.to_packet(),0.);assert!(w.job_progress().is_none());p.target=7;w.on_packet(p.to_packet(),0.);
        let NetCmd::Raw(raw)=w.increase_job_level().unwrap() else{panic!("raw")};assert_eq!(job::CsJobLevelUp::from_packet(&raw).unwrap().target,7);assert!(w.increase_job_level().is_none());assert_eq!(w.addon_state().job_level,Some(1));
        p.job_level=2;p.jp=5;p.cost=Some(5);w.on_packet(p.to_packet(),0.);assert_eq!(w.addon_state().job_level,Some(2));assert!(w.increase_job_level().is_some());
        p.job_level=3;p.jp=0;p.cost=Some(8);p.status=status::JP;w.on_packet(p.to_packet(),0.);assert!(w.increase_job_level().is_none());assert_eq!(w.addon_state().job_points,Some(0));assert_eq!(w.hp,100);
    }
}

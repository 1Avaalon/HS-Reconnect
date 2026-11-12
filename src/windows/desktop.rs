//! Independent native control window + game-bound popup. No hooks or injection.
use super::*;
#[path = "startup.rs"] mod startup;

#[repr(C)] #[derive(Clone,Copy,Default)] struct Rect { left:i32,top:i32,right:i32,bottom:i32 }
impl Rect {
    fn new(x:i32,y:i32,w:i32,h:i32)->Self{Self{left:x,top:y,right:x+w,bottom:y+h}}
    fn w(self)->i32{self.right-self.left} fn h(self)->i32{self.bottom-self.top}
    fn contains(self,p:Point)->bool{p.x>=self.left&&p.x<self.right&&p.y>=self.top&&p.y<self.bottom}
}
#[repr(C)] struct Paint { dc:Handle,erase:i32,rect:Rect,restore:i32,inc_update:i32,reserved:[u8;32] }
#[link(name="user32")] extern "system" {
    fn MonitorFromWindow(h:Hwnd,flags:u32)->Handle;
    fn GetMonitorInfoW(monitor:Handle,info:*mut MonitorInfo)->i32;
    fn FindWindowExW(parent:Hwnd,after:Hwnd,class:*const u16,title:*const u16)->Hwnd;
    fn AllowSetForegroundWindow(pid:u32)->i32;
    fn UpdateLayeredWindow(h:Hwnd,dst:Handle,position:*const Point,size:*const LayerSize,src:Handle,origin:*const Point,key:u32,blend:*const LayerBlend,flags:u32)->i32;
    fn GetSystemMetrics(index:i32)->i32;
    fn SetCursor(cursor:Handle)->Handle;
    fn RegisterWindowMessageW(name:*const u16)->u32;
    fn SetForegroundWindow(h:Hwnd)->i32;
    fn CreatePopupMenu()->Handle;fn DestroyMenu(menu:Handle)->i32;
    fn AppendMenuW(menu:Handle,flags:u32,id:usize,text:*const u16)->i32;
    fn TrackPopupMenu(menu:Handle,flags:u32,x:i32,y:i32,reserved:i32,owner:Hwnd,rect:*const Rect)->u32;
    fn PostMessageW(h:Hwnd,msg:u32,w:usize,l:isize)->i32;
    fn LoadImageW(instance:Handle,name:*const u16,kind:u32,cx:i32,cy:i32,flags:u32)->Handle;
    fn DrawIconEx(dc:Handle,x:i32,y:i32,icon:Handle,w:i32,h:i32,step:u32,brush:Handle,flags:u32)->i32;
    fn BeginPaint(h:Hwnd,p:*mut Paint)->Handle; fn EndPaint(h:Hwnd,p:*const Paint)->i32;
    fn GetClientRect(h:Hwnd,r:*mut Rect)->i32; fn ClientToScreen(h:Hwnd,p:*mut Point)->i32;
    fn GetForegroundWindow()->Hwnd; fn GetWindowThreadProcessId(h:Hwnd,pid:*mut u32)->u32;
    fn IsIconic(h:Hwnd)->i32; fn IsWindowVisible(h:Hwnd)->i32; fn IsWindow(h:Hwnd)->i32;
    fn ShowWindow(h:Hwnd,how:i32)->i32; fn InvalidateRect(h:Hwnd,r:*const Rect,erase:i32)->i32;
    fn DrawTextW(dc:Handle,t:*const u16,len:i32,r:*mut Rect,flags:u32)->i32;
    fn FillRect(dc:Handle,r:*const Rect,brush:Handle)->i32;
    fn GetCursorPos(p:*mut Point)->i32; fn ScreenToClient(h:Hwnd,p:*mut Point)->i32;
    fn SetCapture(h:Hwnd)->Hwnd; fn GetCapture()->Hwnd; fn ReleaseCapture()->i32;
    fn SystemParametersInfoW(action:u32,param:u32,out:*mut c_void,flags:u32)->i32;
}
#[link(name="gdi32")] extern "system" {
    fn CreateDIBSection(dc:Handle,info:*const DibInfo,usage:u32,bits:*mut *mut c_void,section:Handle,offset:u32)->Handle;
    fn GdiFlush()->i32;
    fn GetViewportOrgEx(dc:Handle,point:*mut Point)->i32;
    fn SetViewportOrgEx(dc:Handle,x:i32,y:i32,old:*mut Point)->i32;
    fn StretchBlt(dst:Handle,x:i32,y:i32,w:i32,h:i32,src:Handle,sx:i32,sy:i32,sw:i32,sh:i32,rop:u32)->i32;
    fn SetStretchBltMode(dc:Handle,mode:i32)->i32;
    fn SetBrushOrgEx(dc:Handle,x:i32,y:i32,old:*mut Point)->i32;
    fn SaveDC(dc:Handle)->i32; fn RestoreDC(dc:Handle,saved:i32)->i32;
    fn SelectClipRgn(dc:Handle,rgn:Handle)->i32;
    fn Arc(dc:Handle,l:i32,t:i32,r:i32,b:i32,sx:i32,sy:i32,ex:i32,ey:i32)->i32;
    fn CreateCompatibleDC(dc:Handle)->Handle; fn DeleteDC(dc:Handle)->i32;
    fn CreateCompatibleBitmap(dc:Handle,w:i32,h:i32)->Handle;
    fn SelectObject(dc:Handle,obj:Handle)->Handle; fn DeleteObject(obj:Handle)->i32;
    fn BitBlt(dst:Handle,x:i32,y:i32,w:i32,h:i32,src:Handle,sx:i32,sy:i32,rop:u32)->i32;
    fn CreateSolidBrush(color:u32)->Handle; fn CreatePen(style:i32,width:i32,color:u32)->Handle;
    fn RoundRect(dc:Handle,l:i32,t:i32,r:i32,b:i32,ew:i32,eh:i32)->i32;
    fn Ellipse(dc:Handle,l:i32,t:i32,r:i32,b:i32)->i32;
    fn MoveToEx(dc:Handle,x:i32,y:i32,old:*mut Point)->i32; fn LineTo(dc:Handle,x:i32,y:i32)->i32;
    fn SetTextColor(dc:Handle,c:u32)->u32; fn SetBkMode(dc:Handle,mode:i32)->i32;
    fn CreateFontW(height:i32,width:i32,esc:i32,orientation:i32,weight:i32,italic:u32,under:u32,strike:u32,charset:u32,out:u32,clip:u32,quality:u32,pitch:u32,face:*const u16)->Handle;
    fn CreateRectRgn(l:i32,t:i32,r:i32,b:i32)->Handle;
    fn CreateRoundRectRgn(l:i32,t:i32,r:i32,b:i32,ew:i32,eh:i32)->Handle;
}
#[link(name="kernel32")] extern "system"{
    fn CreateMutexW(security:*const c_void,owner:i32,name:*const u16)->Handle;
    fn GetLastError()->u32;
}
// Local namespace scopes this lock to the interactive Windows session.
// RAII retains it through the entire message loop, including a hidden tray window.
unsafe fn instance_lock()->Result<Option<Owned>,String>{
    let handle=CreateMutexW(null(),0,wide("Local\\HSRustReconnectDesktopV2.SingleInstance").as_ptr());
    let error=GetLastError();
    if handle.is_null(){return Err(format!("Single-instance lock: {}",std::io::Error::from_raw_os_error(error as i32)));}
    let lock=Owned(handle);if error==183{drop(lock);Ok(None)}else{Ok(Some(lock))}
}
unsafe fn restore_existing()->bool{
    let class=wide("HSRustReconnectDesktopV2");
    for _ in 0..100{
        let mut after=null_mut();
        loop{
            let h=FindWindowExW(null_mut(),after,class.as_ptr(),null());if h.is_null(){break;}after=h;
            // Main and overlay share a class. Never restore the no-activate tool overlay.
            if GetWindowLongPtrW(h,-20)&0x80==0&&GetWindowLongPtrW(h,-21)!=0{
                let mut pid=0;GetWindowThreadProcessId(h,&mut pid);if pid!=0{AllowSetForegroundWindow(pid);}
                if PostMessageW(h,OPEN_MAIN,0,0)!=0{return true;}
            }
        }
        thread::sleep(Duration::from_millis(30));
    }false
}
#[repr(C)] struct MonitorInfo{size:u32,monitor:Rect,work:Rect,flags:u32}
#[repr(C)] struct LayerSize{cx:i32,cy:i32}
#[repr(C)] struct LayerBlend{op:u8,flags:u8,alpha:u8,format:u8}
#[repr(C)] struct DibHeader{size:u32,width:i32,height:i32,planes:u16,bits:u16,compression:u32,image_size:u32,xppm:i32,yppm:i32,used:u32,important:u32}
#[repr(C)] struct DibInfo{header:DibHeader,colors:[u32;1]}

#[repr(C)] struct NotifyIconData{
    size:u32,hwnd:Hwnd,id:u32,flags:u32,callback:u32,icon:Handle,tip:[u16;128],
    state:u32,state_mask:u32,info:[u16;256],timeout_version:u32,info_title:[u16;64],info_flags:u32,guid:[u8;16],balloon_icon:Handle,
}
#[link(name="shell32")] extern "system"{fn Shell_NotifyIconW(message:u32,data:*mut NotifyIconData)->i32;}
const TRAY_CALLBACK:u32=0x8001;const QUIT_APP:u32=0x8002;const OPEN_MAIN:u32=0x8003;

#[link(name="dwmapi")] extern "system" { fn DwmSetWindowAttribute(h:Hwnd,attr:u32,value:*const c_void,size:u32)->i32; }
fn rgb(v:u32)->u32{((v>>16)&255)|(v&0xff00)|((v&255)<<16)}
fn mix(a:u32,b:u32,t:f32)->u32{
    let mut out=0;for shift in [0,8,16]{let x=((a>>shift)&255) as f32;let y=((b>>shift)&255) as f32;out|=((x+(y-x)*t.clamp(0.0,1.0)) as u32)<<shift;}out
}
// Render at twice the output resolution and average down, with no sharpening.
const SS:i32=2;
fn pixels(r:Rect)->Rect{Rect{left:r.left*SS,top:r.top*SS,right:r.right*SS,bottom:r.bottom*SS}}
const BG:u32=0x171a20; const CARD:u32=0x1e232c; const BORDER:u32=0x292f38;
const TEXT:u32=0xdce2eb; const MUTED:u32=0xa4adba; const BLUE:u32=0x9cbce5;
unsafe fn fill(dc:Handle,r:Rect,color:u32){let r=pixels(r);let br=CreateSolidBrush(rgb(color));FillRect(dc,&r,br);DeleteObject(br);}
unsafe fn rounded(dc:Handle,r:Rect,rad:i32,bg:u32,stroke:u32){
    let r=pixels(r);let rad=rad*SS;
    let br=CreateSolidBrush(rgb(bg));let pen=CreatePen(0,SS,rgb(stroke));let ob=SelectObject(dc,br);let op=SelectObject(dc,pen);
    RoundRect(dc,r.left,r.top,r.right,r.bottom,rad,rad);SelectObject(dc,op);SelectObject(dc,ob);DeleteObject(pen);DeleteObject(br);
}
unsafe fn text(dc:Handle,r:Rect,s:&str,size:i32,bold:bool,color:u32,center:bool){
    let r=pixels(r);
    let font=CreateFontW(-size*SS,0,0,0,if bold{500}else{400},0,0,0,1,0,0,4,0,wide("Segoe UI").as_ptr());
    let old=SelectObject(dc,font);SetBkMode(dc,1);SetTextColor(dc,rgb(color));let mut r=r;
    DrawTextW(dc,wide(s).as_ptr(),-1,&mut r,0x20|0x4|0x8000|if center{1}else{0});
    SelectObject(dc,old);DeleteObject(font);
}
unsafe fn line(dc:Handle,a:Point,b:Point,color:u32,width:i32){let pen=CreatePen(0,width*SS,rgb(color));let old=SelectObject(dc,pen);MoveToEx(dc,a.x*SS,a.y*SS,null_mut());LineTo(dc,b.x*SS,b.y*SS);SelectObject(dc,old);DeleteObject(pen);}
unsafe fn dot(dc:Handle,x:i32,y:i32,r:i32,color:u32){let (x,y,r)=(x*SS,y*SS,r*SS);let br=CreateSolidBrush(rgb(color));let old=SelectObject(dc,br);let op=SelectObject(dc,GetStockObject(8));Ellipse(dc,x-r,y-r,x+r,y+r);SelectObject(dc,op);SelectObject(dc,old);DeleteObject(br);}
unsafe fn symbol(dc:Handle,r:Rect,plus:bool,color:u32){
    let x=(r.left+r.right)/2;let y=(r.top+r.bottom)/2;
    line(dc,Point{x:x-5,y},Point{x:x+5,y},color,2);
    dot(dc,x-5,y,1,color);dot(dc,x+5,y,1,color);
    if plus{line(dc,Point{x,y:y-5},Point{x,y:y+5},color,2);dot(dc,x,y-5,1,color);dot(dc,x,y+5,1,color);}
}
unsafe fn lock_icon(dc:Handle,r:Rect,locked:bool,color:u32){
    let x=(r.left+r.right)/2;let y=(r.top+r.bottom)/2;
    let pen=CreatePen(0,2*SS,rgb(color));let op=SelectObject(dc,pen);let ob=SelectObject(dc,GetStockObject(5));
    // Curved shackle and outline body; no tiny text glyph or square wire drawing.
    let dx=if locked{0}else{2};
    Arc(dc,(x-6+dx)*SS,(y-11)*SS,(x+6+dx)*SS,(y+1)*SS,(x+6+dx)*SS,(y-5)*SS,(x-6+dx)*SS,(y-5)*SS);
    MoveToEx(dc,(x-6+dx)*SS,(y-5)*SS,null_mut());LineTo(dc,(x-6+dx)*SS,y*SS);
    if locked{MoveToEx(dc,(x+6)*SS,(y-5)*SS,null_mut());LineTo(dc,(x+6)*SS,y*SS);}
    RoundRect(dc,(x-8)*SS,y*SS,(x+8)*SS,(y+12)*SS,5*SS,5*SS);
    SelectObject(dc,ob);SelectObject(dc,op);DeleteObject(pen);
    dot(dc,x,y+5,1,color);
}
unsafe fn logo(dc:Handle,x:i32,y:i32,size:i32,icon:Handle){
    if !icon.is_null(){DrawIconEx(dc,x*SS,y*SS,icon,size*SS,size*SS,0,null_mut(),3);}
}

#[derive(Clone,Copy,PartialEq)] enum Lang{Ru,En}
fn tr(lang:Lang,ru:&'static str,en:&'static str)->&'static str{if lang==Lang::Ru{ru}else{en}}

#[derive(Clone)] struct Settings{enabled:bool,locked:bool,scale:f32,ax:f32,ay:f32,animations:bool,lang:Lang}
impl Default for Settings{fn default()->Self{Self{enabled:true,locked:false,scale:1.0,ax:0.92,ay:0.18,animations:true,lang:Lang::Ru}}}
fn settings_path()->Option<PathBuf>{std::env::var_os("LOCALAPPDATA").map(|p|PathBuf::from(p).join("HSRustReconnect").join("overlay-v2.txt"))}
impl Settings{
    fn load()->Self{
        let mut s=Self::default();
        if let Some(t)=settings_path().and_then(|p|fs::read_to_string(p).ok()){
            for line in t.lines(){if let Some((k,v))=line.split_once('='){match k{
                "language"=>s.lang=if v=="en"{Lang::En}else{Lang::Ru},
                "enabled"=>s.enabled=v=="true","locked"=>s.locked=v=="true","animations"=>s.animations=v=="true",
                "scale"=>if let Ok(n)=v.parse::<f32>(){if n.is_finite(){s.scale=n.clamp(0.85,1.6);}},
                "x"=>if let Ok(n)=v.parse::<f32>(){if n.is_finite(){s.ax=n.clamp(0.0,1.0);}},
                "y"=>if let Ok(n)=v.parse::<f32>(){if n.is_finite(){s.ay=n.clamp(0.0,1.0);}},_=>{}
            }}}
        }s
    }
    fn save(&self){if let Some(p)=settings_path(){if let Some(d)=p.parent(){let _=fs::create_dir_all(d);}let _=fs::write(p,format!("enabled={}\nlocked={}\nscale={}\nx={}\ny={}\nanimations={}\nlanguage={}\n",self.enabled,self.locked,self.scale,self.ax,self.ay,self.animations,if self.lang==Lang::Ru{"ru"}else{"en"}));}}

}
struct Drag{start:Point,left:i32,top:i32,moved:bool,resize:bool,scale:f32}
struct Ui{
    main:Hwnd,overlay:Hwnd,notice:Hwnd,notice_until:Option<Instant>,tx:SyncSender<Command>,rx:Receiver<Update>,stop:Arc<AtomicBool>,settings:Settings,
    ready:bool,busy:bool,status:String,raw_status:String,hover:i32,pressed:i32,drag:Option<Drag>,game:Option<Rect>,
    frame:u64,over_w:i32,over_h:i32,visible:bool,win_animations:bool,
    logo:Handle,main_logo:Handle,hover_main:i32,slider_drag:bool,toggle_on:f32,toggle_motion:f32,scale_visual:f32,lock_hover:f32,tray_added:bool,tray_icon:Handle,taskbar_message:u32,startup_enabled:bool,startup_busy:bool,toggle_startup:f32,startup_error:String,startup_tx:mpsc::Sender<(bool,Option<String>)>,startup_rx:Receiver<(bool,Option<String>)>,
}
fn overlay_size(scale:f32)->(i32,i32){((232.0*scale).round() as i32,(94.0*scale).round() as i32)}
fn overlay_lock(_w:i32)->Rect{Rect::new(4,2,22,20)}
fn overlay_action(w:i32,h:i32)->Rect{Rect::new(10,24,w-20,h-32)}
fn overlay_grip(w:i32)->Rect{Rect::new(w-26,2,22,20)}
fn overlay_hit(s:&Ui,p:Point)->i32{
    if overlay_lock(s.over_w).contains(p){2}else if !s.settings.locked&&overlay_grip(s.over_w).contains(p){3}else if overlay_action(s.over_w,s.over_h).contains(p){1}else{0}
}
fn scale_slider()->Rect{Rect::new(40,224,456,44)}
fn main_hit(p:Point)->i32{
    for (id,r) in [(10,Rect::new(440,300,64,44)),(11,Rect::new(452,164,44,44)),(12,Rect::new(340,164,44,44)),(13,Rect::new(396,164,44,44)),(14,Rect::new(440,360,64,44)),(15,Rect::new(24,496,236,48)),(20,Rect::new(440,420,64,44)),(21,Rect::new(276,496,236,48)),(16,scale_slider()),(19,Rect::new(448,40,44,44))]{if r.contains(p){return id;}}
    0
}
const CHROME:i32=36;
fn chrome_hit(p:Point)->i32{
    if Rect::new(448,6,36,24).contains(p){17}else if Rect::new(490,6,36,24).contains(p){18}else{0}
}
fn main_window_hit(p:Point)->i32{
    if p.y<CHROME{chrome_hit(p)}else{main_hit(Point{x:p.x,y:p.y-CHROME})}
}

fn slider_value(x:i32)->f32{let r=scale_slider();0.85+((x-r.left-8) as f32/(r.w()-16) as f32).clamp(0.0,1.0)*0.75}
unsafe fn slider_move(s:&mut Ui,x:i32){if !s.settings.locked{s.settings.scale=slider_value(x);position(s);InvalidateRect(s.main,null(),0);}}

unsafe fn tray_data(s:&Ui)->NotifyIconData{
    let mut data:NotifyIconData=zeroed();data.size=size_of::<NotifyIconData>() as u32;
    data.hwnd=s.main;data.id=1;data.flags=1|2|4;data.callback=TRAY_CALLBACK;data.icon=s.tray_icon;
    for (to,from) in data.tip.iter_mut().take(127).zip(tr(s.settings.lang,"HS/RECONNECT — двойной щелчок: открыть","HS/RECONNECT — double-click to open").encode_utf16()){*to=from;}
    data
}
unsafe fn tray_install(s:&mut Ui)->bool{
    if s.tray_added{return true;}
    let mut data=tray_data(s);s.tray_added=Shell_NotifyIconW(0,&mut data)!=0;s.tray_added
}
unsafe fn tray_notice(s:&mut Ui){
    if !s.tray_added{return;}
    // An app-owned no-activate notice, not a best-effort Windows balloon.
    let monitor=MonitorFromWindow(s.main,2);let mut info=MonitorInfo{size:size_of::<MonitorInfo>() as u32,monitor:Rect::default(),work:Rect::default(),flags:0};
    let work=if !monitor.is_null()&&GetMonitorInfoW(monitor,&mut info)!=0{info.work}else{
        let mut work=Rect::default();if SystemParametersInfoW(0x30,0,(&mut work as *mut Rect).cast(),0)!=0{work}else{Rect::new(0,0,GetSystemMetrics(0),GetSystemMetrics(1))}
    };
    s.notice_until=Some(Instant::now()+Duration::from_secs(5));
    SetWindowPos(s.notice,-1isize as Hwnd,(work.right-376).max(work.left),(work.bottom-128).max(work.top),360,112,0x10|0x40);
    InvalidateRect(s.notice,null(),0);PostMessageW(s.notice,0xf,0,0);
}

unsafe fn tray_remove(s:&mut Ui){
    if s.tray_added{let mut data=tray_data(s);let _=Shell_NotifyIconW(2,&mut data);s.tray_added=false;}
}
unsafe fn tray_menu(h:Hwnd){
    // TrackPopupMenu runs a nested message loop: do not hold a Ui borrow here.
    let ptr=GetWindowLongPtrW(h,-21) as *mut Ui;let lang=if ptr.is_null(){Lang::Ru}else{(*ptr).settings.lang};
    let menu=CreatePopupMenu();if menu.is_null(){return;}
    AppendMenuW(menu,0,1,wide(tr(lang,"Открыть HS/RECONNECT","Open HS/RECONNECT")).as_ptr());
    AppendMenuW(menu,0x800,0,null());AppendMenuW(menu,0,2,wide(tr(lang,"Выход","Exit")).as_ptr());
    let mut p=Point::default();GetCursorPos(&mut p);SetForegroundWindow(h);
    let id=TrackPopupMenu(menu,0x100|0x2,p.x,p.y,0,h,null());DestroyMenu(menu);PostMessageW(h,0,0,0);
    if id==1{PostMessageW(h,OPEN_MAIN,0,0);}else if id==2{PostMessageW(h,QUIT_APP,0,0);}
}
unsafe fn main_outer_border(dc:Handle,w:i32,h:i32){
    let pen=CreatePen(0,SS,rgb(MAIN_GOLD));let old_pen=SelectObject(dc,pen);let old_brush=SelectObject(dc,GetStockObject(5));
    RoundRect(dc,SS,SS,(w-1)*SS,(h-1)*SS,24*SS,24*SS);
    SelectObject(dc,old_brush);SelectObject(dc,old_pen);DeleteObject(pen);
}

fn friendly(raw:&str,lang:Lang)->String{
    let pair=if raw.starts_with("Ready:"){("Подключено","Connected")}
    else if raw.starts_with("Reset sent"){("Переподключение","Reconnecting")}
    else if raw.starts_with("Checking connection"){("Проверка соединения","Checking connection")}
    else if raw.contains("Waiting for Hearthstone"){("Ожидание Hearthstone","Waiting for Hearthstone")}
    else if raw.contains("No fresh match")||raw.contains("No current-session match"){("Ожидание матча","Waiting for match")}
    else if raw.contains("No matching established"){("Игра обнаружена","Game detected")}
    else if raw.contains("Ambiguous")||raw.contains("Multiple"){("Сброс отменён: неоднозначное соединение","Reset cancelled: ambiguous connection")}
    else if raw.contains("Administrator"){("Нужны права администратора","Administrator rights required")}
    else if raw.contains("Query game path")||raw.contains("Query game start"){("Не удалось прочитать сведения об игре","Cannot read game process details")}
    else if raw.contains("TCP table"){("Не удалось обновить TCP-соединения","Cannot refresh TCP connections")}
    else if raw.contains("Read log"){("Лог игры временно недоступен","Game log temporarily unavailable")}
    else if raw.starts_with("Not reset:"){("Сброс отменён: соединение изменилось","Reset cancelled: connection changed")}
    else{("Матч не определён: проверьте логи","Match not detected: check game logs")};
    tr(lang,pair.0,pair.1).into()
}

unsafe fn foreground_game()->Option<Rect>{
    let h=GetForegroundWindow();if h.is_null()||IsIconic(h)!=0||IsWindowVisible(h)==0{return None;}
    let mut pid=0;GetWindowThreadProcessId(h,&mut pid);if pid==0{return None;}
    let raw=OpenProcess(0x1000,0,pid);if raw.is_null(){return None;}let proc=Owned(raw);
    let mut path=vec![0u16;32768];let mut len=path.len() as u32;
    if QueryFullProcessImageNameW(proc.0,0,path.as_mut_ptr(),&mut len)==0{return None;}
    let path=PathBuf::from(String::from_utf16_lossy(&path[..len as usize]));
    if !path.file_name()?.to_string_lossy().eq_ignore_ascii_case("Hearthstone.exe"){return None;}
    let mut r=Rect::default();if GetClientRect(h,&mut r)==0||r.w()<100||r.h()<100{return None;}
    let mut origin=Point{x:0,y:0};if ClientToScreen(h,&mut origin)==0{return None;}
    Some(Rect::new(origin.x,origin.y,r.w(),r.h()))
}
unsafe fn position(s:&mut Ui){
    let game=foreground_game();s.game=game;
    if !s.settings.enabled||game.is_none(){
        if s.visible{ShowWindow(s.overlay,0);s.visible=false;}
        if s.drag.take().is_some(){ReleaseCapture();}
        return;
    }
    let g=game.unwrap();let (w,h)=overlay_size(s.settings.scale);
    // Do not paint outside a client area too small for the minimum usable widget.
    if g.w()<w||g.h()<h{if s.visible{ShowWindow(s.overlay,0);s.visible=false;}return;}
    let x=g.left+((g.w()-w) as f32*s.settings.ax).round() as i32;
    let y=g.top+((g.h()-h) as f32*s.settings.ay).round() as i32;
    let flags=0x10|if !s.visible{0x40}else{0}; // NOACTIVATE, optional SHOWWINDOW
    SetWindowPos(s.overlay,(-1isize) as Hwnd,x,y,w,h,flags);
    s.over_w=w;s.over_h=h;
    s.visible=true;
}
unsafe fn sheen(dc:Handle,r:Rect,bg:u32,s:&Ui,offset:f32,hover:bool){
    if !(s.settings.animations&&s.win_animations)||r.w()<24||r.h()<20{return;}
    // Match the actual control's full silhouette, including its round corners.
    // Regions are in device coordinates, so explicitly apply supersampling.
    let radius=if r.h()<=28{r.h()/2}else{8};
    let mut inner=pixels(Rect::new(r.left+1,r.top+1,r.w()-2,r.h()-2));
    let mut origin=Point::default();GetViewportOrgEx(dc,&mut origin);inner.left+=origin.x;inner.right+=origin.x;inner.top+=origin.y;inner.bottom+=origin.y;
    let clip=CreateRoundRectRgn(inner.left,inner.top,inner.right,inner.bottom,(radius*2-2)*SS,(radius*2-2)*SS);
    if clip.is_null(){return;}
    let saved=SaveDC(dc);if saved==0{DeleteObject(clip);return;}
    if SelectClipRgn(dc,clip)==0{RestoreDC(dc,saved);DeleteObject(clip);return;}
    let progress=(s.frame as f32*0.003+offset).fract();
    let band=(r.w() as f32*0.32).clamp(14.0,70.0);
    let center=r.left as f32-band*2.5+progress*(r.w() as f32+band*5.0);
    let strength=if hover{0.075}else{0.035};
    // Full-height Gaussian wash, not a short inset stripe. No dithering/noise.
    for x in r.left..r.right{
        let distance=(x as f32-center)/band;
        let weight=(-distance*distance*1.7).exp()*strength;
        if weight>0.0005{fill(dc,Rect::new(x,r.top,1,r.h()),mix(bg,0xdde8f5,weight));}
    }
    RestoreDC(dc,saved);DeleteObject(clip);
}

unsafe fn soft_button(dc:Handle,r:Rect,s:&Ui,id:i32){
    let hover=s.hover_main==id;let bg=if s.pressed==id{0x2b3441}else if hover{0x2a313d}else{0x242b35};
    rounded(dc,r,16,bg,if hover{0x3b4756}else{0x2d3641});sheen(dc,r,bg,s,id as f32*0.071,hover);
}
unsafe fn toggle(dc:Handle,r:Rect,progress:f32,s:&Ui,id:i32){
    let bg=mix(0x303640,0x527ba7,progress);let track=Rect::new(r.left+6,r.top+9,r.w()-12,26);
    rounded(dc,track,26,bg,mix(0x3b434f,0x6b92bd,progress));sheen(dc,track,bg,s,id as f32*0.09,s.hover_main==id);
    let x=(r.left+20) as f32+(r.w()-40) as f32*progress;
    dot(dc,x.round() as i32,r.top+22,9,0xe4eaf3);
}
// Main-window-only Dark Gold theme. Shared overlay primitives remain untouched.
const MAIN_BG:u32=0x13171d;const MAIN_IVORY:u32=0xead9b7;const MAIN_GOLD:u32=0xad8550;
unsafe fn main_text(dc:Handle,r:Rect,s:&str,size:i32,bold:bool,color:u32,center:bool){
    let r=pixels(r);let font=CreateFontW(-size*SS,0,0,0,if bold{600}else{400},0,0,0,1,0,0,4,0,wide("Georgia").as_ptr());
    let old=SelectObject(dc,font);SetBkMode(dc,1);SetTextColor(dc,rgb(color));let mut r=r;
    DrawTextW(dc,wide(s).as_ptr(),-1,&mut r,0x20|0x4|0x8000|if center{1}else{0});
    SelectObject(dc,old);DeleteObject(font);
}
unsafe fn gold_panel(dc:Handle,r:Rect,strong:bool){
    rounded(dc,r,16,0x10141a,if strong{0x8b6c43}else{0x594b37});
    let inset=Rect::new(r.left+2,r.top+2,r.w()-4,r.h()-4);
    rounded(dc,inset,12,0x191e25,0x292a29);
    // Smooth, very low-contrast vertical shading. No grain or sharpen filter.
    for y in r.top+8..r.bottom-8{
        let t=(y-r.top) as f32/r.h() as f32;
        fill(dc,Rect::new(r.left+4,y,r.w()-8,1),mix(0x1b2028,0x13171d,t));
    }

}
unsafe fn gold_button(dc:Handle,r:Rect,s:&Ui,id:i32,primary:bool){
    let hover=s.hover_main==id;
    let bg=if s.pressed==id{0x172c43}else if hover{0x243b54}else if primary{0x1b3048}else{0x21272f};
    rounded(dc,r,16,0x0f1319,if hover{0xd0aa68}else{MAIN_GOLD});
    rounded(dc,Rect::new(r.left+2,r.top+2,r.w()-4,r.h()-4),12,bg,if primary{0x3f5a74}else{0x42494f});
    sheen(dc,Rect::new(r.left+2,r.top+2,r.w()-4,r.h()-4),bg,s,id as f32*0.071,hover);
}
unsafe fn gold_toggle(dc:Handle,r:Rect,progress:f32,s:&Ui,id:i32){
    let track=Rect::new(r.left+6,r.top+9,r.w()-12,26);
    let bg=mix(0x2a2e35,0x087bbb,progress);
    rounded(dc,track,26,bg,mix(0x766349,0x67c5e8,progress));
    sheen(dc,track,bg,s,id as f32*0.09,s.hover_main==id);
    let x=(r.left+20) as f32+(r.w()-40) as f32*progress;
    dot(dc,x.round() as i32,r.top+22,9,0xcde8f3);
    dot(dc,x.round() as i32,r.top+21,6,0xe0f2f8);
}
unsafe fn language_flag(dc:Handle,s:&Ui){
    let r=Rect::new(448,40,44,44);let inner=Rect::new(450,42,40,40);
    rounded(dc,r,16,0x11151b,if s.hover_main==19{0xd0aa68}else{MAIN_GOLD});
    let mut area=pixels(inner);let mut org=Point::default();GetViewportOrgEx(dc,&mut org);
    area.left+=org.x;area.right+=org.x;area.top+=org.y;area.bottom+=org.y;
    let region=CreateRoundRectRgn(area.left,area.top,area.right,area.bottom,12*SS,12*SS);let saved=SaveDC(dc);
    if !region.is_null()&&saved!=0&&SelectClipRgn(dc,region)!=0{
        let x=inner.left;let y=inner.top;
        if s.settings.lang==Lang::Ru{
            fill(dc,Rect::new(x,y,40,14),0xf0f1f4);fill(dc,Rect::new(x,y+14,40,13),0x2455ad);fill(dc,Rect::new(x,y+27,40,13),0xc83b45);
        }else{
            fill(dc,inner,0x244a86);
            for (a,b) in [(Point{x,y},Point{x:x+40,y:y+40}),(Point{x:x+40,y},Point{x,y:y+40})]{line(dc,a,b,0xf0ece2,8);line(dc,a,b,0xc43c4a,3);}
            fill(dc,Rect::new(x+15,y,10,40),0xf0ece2);fill(dc,Rect::new(x,y+15,40,10),0xf0ece2);
            fill(dc,Rect::new(x+17,y,6,40),0xc43c4a);fill(dc,Rect::new(x,y+17,40,6),0xc43c4a);
        }
    }
    if saved!=0{RestoreDC(dc,saved);}if !region.is_null(){DeleteObject(region);}
}

unsafe fn paint_main(dc:Handle,s:&Ui,w:i32,h:i32){
    fill(dc,Rect::new(0,0,w,h),MAIN_BG);
    rounded(dc,Rect::new(7,7,w-14,h-14),18,MAIN_BG,MAIN_GOLD);
    rounded(dc,Rect::new(10,10,w-20,h-20),14,MAIN_BG,0x4f4230);
    gold_panel(dc,Rect::new(24,22,488,114),true);
    logo(dc,30,33,84,s.main_logo);
    main_text(dc,Rect::new(124,40,304,38),"HS/RECONNECT",23,true,MAIN_IVORY,false);
    language_flag(dc,s);
    dot(dc,130,102,3,if s.ready{0x78b98b}else{0xbca06a});
    text(dc,Rect::new(142,86,350,32),&s.status,13,false,0xc2b59d,false);
    gold_panel(dc,Rect::new(24,152,488,128),false);
    main_text(dc,Rect::new(40,164,190,44),tr(s.settings.lang,"Размер оверлея","Overlay size"),16,true,MAIN_IVORY,false);
    main_text(dc,Rect::new(230,164,96,44),&format!("{}%",(s.settings.scale*100.0).round() as u32),15,false,MAIN_IVORY,true);
    for (x,plus,id) in [(340,false,12),(396,true,13)]{
        let r=Rect::new(x,164,44,44);gold_button(dc,r,s,id,false);symbol(dc,r,plus,if s.settings.locked{0x807769}else{MAIN_IVORY});
    }
    let r=Rect::new(452,164,44,44);gold_button(dc,r,s,11,false);lock_icon(dc,r,s.settings.locked,if s.settings.locked{0x8ad5f4}else{MAIN_IVORY});
    let track=scale_slider();let cy=track.top+22;let left=track.left+8;let right=track.right-8;
    let thumb=left+(((s.scale_visual-0.85)/0.75)*(right-left) as f32).round() as i32;
    rounded(dc,Rect::new(left,cy-4,right-left,8),8,0x0c1118,0x3c4246);
    rounded(dc,Rect::new(left+1,cy-3,(thumb-left).max(2),6),6,if s.settings.locked{0x4a6577}else{0x138fcc},if s.settings.locked{0x4a6577}else{0x52c1ea});
    dot(dc,thumb,cy,11,0x233747);dot(dc,thumb,cy,9,if s.settings.locked{0x7f939f}else{0x8edcf4});dot(dc,thumb,cy,6,if s.settings.locked{0x99a8b0}else{0x49b9e6});
    gold_panel(dc,Rect::new(24,292,488,188),false);
    main_text(dc,Rect::new(40,300,350,44),tr(s.settings.lang,"Оверлей в игре","In-game overlay"),16,true,MAIN_IVORY,false);
    gold_toggle(dc,Rect::new(440,300,64,44),s.toggle_on,s,10);
    line(dc,Point{x:40,y:356},Point{x:496,y:356},0x343331,1);
    main_text(dc,Rect::new(40,360,350,44),tr(s.settings.lang,"Анимация","Animations"),16,true,MAIN_IVORY,false);
    gold_toggle(dc,Rect::new(440,360,64,44),s.toggle_motion,s,14);
    line(dc,Point{x:40,y:416},Point{x:496,y:416},0x343331,1);
    main_text(dc,Rect::new(40,420,350,44),tr(s.settings.lang,"Запуск с Windows","Start with Windows"),16,true,MAIN_IVORY,false);
    gold_toggle(dc,Rect::new(440,420,64,44),s.toggle_startup,s,20);
    if s.startup_busy{dot(dc,426,442,2,0xbca06a);}
    let reset=Rect::new(24,496,236,48);gold_button(dc,reset,s,15,true);
    main_text(dc,reset,tr(s.settings.lang,"Сбросить расположение","Reset position"),14,false,MAIN_IVORY,true);
    let reconnect=Rect::new(276,496,236,48);gold_button(dc,reconnect,s,21,true);
    main_text(dc,reconnect,"RECONNECT",18,true,if s.ready&&!s.busy{MAIN_IVORY}else{0x7f817f},true);

}

fn overlay_bg(s:&Ui)->u32{
    if s.pressed==1{0x172538}else if s.hover==1&&s.ready&&!s.busy{0x243b54}else if s.ready&&!s.busy{0x1b3048}else{0x1b3048}
}
unsafe fn tiny_lock(dc:Handle,r:Rect,locked:bool,color:u32){
    let x=(r.left+r.right)/2;let y=(r.top+r.bottom)/2;
    let pen=CreatePen(0,SS,rgb(color));let op=SelectObject(dc,pen);let ob=SelectObject(dc,GetStockObject(5));
    let dx=if locked{0}else{1};
    Arc(dc,(x-3+dx)*SS,(y-6)*SS,(x+3+dx)*SS,y*SS,(x+3+dx)*SS,(y-3)*SS,(x-3+dx)*SS,(y-3)*SS);
    MoveToEx(dc,(x-3+dx)*SS,(y-3)*SS,null_mut());LineTo(dc,(x-3+dx)*SS,(y+1)*SS);
    if locked{MoveToEx(dc,(x+3)*SS,(y-3)*SS,null_mut());LineTo(dc,(x+3)*SS,(y+1)*SS);}
    RoundRect(dc,(x-4)*SS,y*SS,(x+4)*SS,(y+6)*SS,2*SS,2*SS);
    SelectObject(dc,ob);SelectObject(dc,op);DeleteObject(pen);
}
unsafe fn paint_overlay(dc:Handle,s:&Ui,w:i32,h:i32){
    // Compact black/gold shell with the lock OUTSIDE the inner action button.
    fill(dc,Rect::new(0,0,w,h),MAIN_BG);
    rounded(dc,Rect::new(0,0,w,h),24,MAIN_BG,MAIN_GOLD);
    rounded(dc,Rect::new(2,2,w-4,h-4),20,MAIN_BG,0x40392e);
    let r=overlay_action(w,h);let bg=overlay_bg(s);
    // Match the blue/gold treatment of the main window's reset button.
    rounded(dc,r,16,0x0f1319,if s.hover==1&&s.ready{0xd0aa68}else{MAIN_GOLD});
    let inner=Rect::new(r.left+2,r.top+2,r.w()-4,r.h()-4);
    rounded(dc,inner,12,bg,0x3f5a74);sheen(dc,inner,bg,s,0.0,s.hover==1);
    let top_h=(r.h()/2).max(24);
    main_text(dc,Rect::new(r.left+8,r.top+3,r.w()-16,top_h-3),"RECONNECT",((20.0*s.settings.scale).round() as i32).clamp(17,27),true,if s.ready&&!s.busy{MAIN_IVORY}else{0xbab09e},true);
    text(dc,Rect::new(r.left+8,r.top+top_h,r.w()-16,r.h()-top_h-4),&s.status,((12.0*s.settings.scale).round() as i32).clamp(11,16),false,0xc2b59d,true);
    let alpha=0.22+s.lock_hover*0.73;
    tiny_lock(dc,overlay_lock(w),s.settings.locked,mix(MAIN_BG,MAIN_IVORY,alpha));
    // Top-right grip is separate from both the lock and the action button.
    let grip_color=mix(MAIN_BG,MAIN_IVORY,if s.settings.locked{0.10}else if s.hover==3{0.85}else{0.35});
    for i in [0,4,8]{line(dc,Point{x:w-20+i,y:7},Point{x:w-7,y:20-i},grip_color,1);}

}

unsafe fn paint_chrome(dc:Handle,s:&Ui,w:i32){
    fill(dc,Rect::new(0,0,w,CHROME),MAIN_BG);
    if s.hover_main==17{rounded(dc,Rect::new(448,6,36,24),8,0x242b35,0x242b35);}
    if s.hover_main==18{rounded(dc,Rect::new(490,6,36,24),8,0x673333,0x673333);}
    line(dc,Point{x:460,y:18},Point{x:472,y:18},MAIN_IVORY,1);
    line(dc,Point{x:503,y:13},Point{x:513,y:23},MAIN_IVORY,1);
    line(dc,Point{x:513,y:13},Point{x:503,y:23},MAIN_IVORY,1);
}

unsafe fn present_layered(h:Hwnd,dc:Handle,source:Handle,w:i32,height:i32){
    let info=DibInfo{header:DibHeader{size:40,width:w,height:-height,planes:1,bits:32,compression:0,image_size:0,xppm:0,yppm:0,used:0,important:0},colors:[0]};
    let target=CreateCompatibleDC(dc);let mut bits:*mut c_void=null_mut();let bitmap=CreateDIBSection(dc,&info,0,&mut bits,null_mut(),0);
    if target.is_null()||bitmap.is_null()||bits.is_null(){if !bitmap.is_null(){DeleteObject(bitmap);}if !target.is_null(){DeleteDC(target);}return;}
    let old=SelectObject(target,bitmap);SetStretchBltMode(target,4);SetBrushOrgEx(target,0,0,null_mut());
    StretchBlt(target,0,0,w,height,source,0,0,w*SS,height*SS,0x00cc0020);GdiFlush();
    let pixels=std::slice::from_raw_parts_mut(bits.cast::<u8>(),w as usize*height as usize*4);
    let radius=12.0f32.min(w as f32*0.5).min(height as f32*0.5);
    for y in 0..height{for x in 0..w{
        let corner=((x as f32)<radius||x as f32>=w as f32-radius)&&((y as f32)<radius||y as f32>=height as f32-radius);
        let alpha=if corner{
            let mut count=0u32;
            for sy in 0..4{for sx in 0..4{
                let px=x as f32+(sx as f32+0.5)/4.0;let py=y as f32+(sy as f32+0.5)/4.0;
                let cx=px.clamp(radius,w as f32-radius);let cy=py.clamp(radius,height as f32-radius);
                if (px-cx).powi(2)+(py-cy).powi(2)<=radius*radius{count+=1;}
            }}(count*255/16) as u8
        }else{255};
        let i=(y as usize*w as usize+x as usize)*4;
        for c in 0..3{pixels[i+c]=(pixels[i+c] as u32*alpha as u32/255) as u8;}pixels[i+3]=alpha;
    }}
    let size=LayerSize{cx:w,cy:height};let origin=Point{x:0,y:0};let blend=LayerBlend{op:0,flags:0,alpha:255,format:1};
    UpdateLayeredWindow(h,null_mut(),null(),&size,target,&origin,0,&blend,2);
    SelectObject(target,old);DeleteObject(bitmap);DeleteDC(target);
}
unsafe fn paint_notice(dc:Handle,s:&Ui,w:i32,h:i32){
    fill(dc,Rect::new(0,0,w,h),MAIN_BG);
    rounded(dc,Rect::new(1,1,w-2,h-2),24,MAIN_BG,MAIN_GOLD);
    rounded(dc,Rect::new(4,4,w-8,h-8),20,MAIN_BG,0x40392e);
    main_text(dc,Rect::new(20,14,w-40,28),tr(s.settings.lang,"Программа свёрнута в трей","Running in the system tray"),16,true,MAIN_IVORY,false);
    text(dc,Rect::new(20,46,w-40,24),tr(s.settings.lang,"Работа продолжается. Нажмите здесь,","Still running. Click here to reopen"),13,false,0xc2b59d,false);
    text(dc,Rect::new(20,69,w-40,24),tr(s.settings.lang,"чтобы снова открыть окно.","the main window."),13,false,0xc2b59d,false);
}
unsafe fn paint(h:Hwnd,s:&Ui,overlay:bool){
    let mut ps:Paint=zeroed();let dc=BeginPaint(h,&mut ps);if dc.is_null(){return;}if IsIconic(h)!=0{EndPaint(h,&ps);return;}
    let mut r=Rect::default();GetClientRect(h,&mut r);let w=r.w().max(1);let height=r.h().max(1);
    let mem=CreateCompatibleDC(dc);let bitmap=CreateCompatibleBitmap(dc,w*SS,height*SS);
    if mem.is_null()||bitmap.is_null(){if !bitmap.is_null(){DeleteObject(bitmap);}if !mem.is_null(){DeleteDC(mem);}EndPaint(h,&ps);return;}
    let old=SelectObject(mem,bitmap);
    if h==s.notice{paint_notice(mem,s,w,height);}else if overlay{paint_overlay(mem,s,w,height);}else{
        paint_chrome(mem,s,w);SetViewportOrgEx(mem,0,CHROME*SS,null_mut());
        paint_main(mem,s,w,height-CHROME);SetViewportOrgEx(mem,0,0,null_mut());main_outer_border(mem,w,height);
    }
    // Main, overlay and notice all use full-window per-pixel alpha: no binary region clipping.
    present_layered(h,dc,mem,w,height);
    SelectObject(mem,old);DeleteObject(bitmap);DeleteDC(mem);EndPaint(h,&ps);
}

unsafe fn request_reset(s:&mut Ui){
    if !s.ready||s.busy{return;}
    // Clear pre-click status updates so an old Ready message cannot re-enable the button.
    while s.rx.try_recv().is_ok(){}
    if s.tx.try_send(Command::Reconnect).is_ok(){s.ready=false;s.busy=true;s.raw_status="Checking connection".into();s.status=friendly(&s.raw_status,s.settings.lang);}
}
unsafe fn change_scale(s:&mut Ui,delta:f32){
    if s.settings.locked{return;}
    s.settings.scale=(s.settings.scale+delta).clamp(0.85,1.6);s.settings.save();position(s);InvalidateRect(s.main,null(),0);InvalidateRect(s.overlay,null(),0);
}
unsafe extern "system" fn proc(h:Hwnd,msg:u32,w:usize,l:isize)->isize{
    // Use the entire window as its client area, even with minimize/system-menu styles.
    if msg==0x83{return 0;}
    let state_ptr=GetWindowLongPtrW(h,-21) as *mut Ui;
    if !state_ptr.is_null()&&h==(*state_ptr).notice{
        match msg{
            0xf=>{paint(h,&*state_ptr,false);return 0;},
            0x14=>return 1,0x21=>return 3,0x84=>return 1,
            0x20=>{SetCursor(LoadCursorW(null_mut(),32649usize as *const u16));return 1;},
            0x202=>{PostMessageW((*state_ptr).main,OPEN_MAIN,0,0);return 0;},
            0x10=>{ShowWindow(h,0);(*state_ptr).notice_until=None;return 0;},
            _=>return DefWindowProcW(h,msg,w,l)
        }
    }

    if msg==0x20&&!state_ptr.is_null()&&h==(*state_ptr).overlay&&!(*state_ptr).settings.locked{
        let mut p=Point::default();GetCursorPos(&mut p);ScreenToClient(h,&mut p);
        if overlay_grip((*state_ptr).over_w).contains(p){SetCursor(LoadCursorW(null_mut(),32643usize as *const u16));return 1;}
    }
    if !state_ptr.is_null()&&h==(*state_ptr).main{
        if msg==TRAY_CALLBACK{
            if l as u32==0x203||l as u32==0x405{PostMessageW(h,OPEN_MAIN,0,0);}else if l as u32==0x205||l as u32==0x7b{tray_menu(h);}return 0;
        }
        if msg==0x8004{MessageBoxW(h,wide(tr((*state_ptr).settings.lang,"Не удалось добавить значок в трей. Окно оставлено открытым, чтобы программа не стала недоступна.","Could not create the tray icon. The window remains open so the app stays accessible.")).as_ptr(),wide("HS/RECONNECT").as_ptr(),0x30);return 0;}
        if msg==0x8005{
            let lang=(*state_ptr).settings.lang;let detail=(&(*state_ptr).startup_error).clone();
            let text=format!("{}\n\n{}",tr(lang,"Не удалось изменить запуск с Windows.","Could not change Windows startup."),detail);
            MessageBoxW(h,wide(&text).as_ptr(),wide("HS/RECONNECT").as_ptr(),0x30);return 0;
        }
        if msg==OPEN_MAIN{ShowWindow((*state_ptr).notice,0);(*state_ptr).notice_until=None;ShowWindow(h,9);SetForegroundWindow(h);InvalidateRect(h,null(),0);PostMessageW(h,0xf,0,0);return 0;}
        if (*state_ptr).taskbar_message!=0&&msg==(*state_ptr).taskbar_message{
            (*state_ptr).tray_added=false;if !tray_install(&mut *state_ptr){PostMessageW(h,OPEN_MAIN,0,0);PostMessageW(h,0x8004,0,0);}return 0;
        }
        if msg==0x16&&w!=0{PostMessageW(h,QUIT_APP,0,0);return 0;}
    }

    if msg==0x84{
        let raw=GetWindowLongPtrW(h,-21) as *mut Ui;
        if !raw.is_null()&&h==(*raw).main{
            let mut p=Point{x:(l as u16 as i16) as i32,y:((l>>16) as u16 as i16) as i32};ScreenToClient(h,&mut p);
            if p.y>=0&&p.y<CHROME&&chrome_hit(p)==0{return 2;}return 1;
        }
        return DefWindowProcW(h,msg,w,l);
    }
    // Avoid obtaining a mutable state reference for unrelated/reentrant Win32 messages.
    if !matches!(msg,0xf|0x14|0x21|0x113|0x201|0x202|0x200|0x20a|0x215|0x10|0x2|QUIT_APP){return DefWindowProcW(h,msg,w,l);}
    if msg==0x14{return 1;}
    if msg==0x215{return 0;}
    let ptr=GetWindowLongPtrW(h,-21) as *mut Ui;if ptr.is_null(){return DefWindowProcW(h,msg,w,l);}
    let s=&mut *ptr;let over=h==s.overlay;
    match msg{
        0x21 if over=>3,
        0xf=>{paint(h,s,over);0}
        0x113 if !over=>{
            s.frame=s.frame.wrapping_add(1);
            if s.notice_until.map(|deadline|Instant::now()>=deadline).unwrap_or(false){ShowWindow(s.notice,0);s.notice_until=None;}

            if s.slider_drag&&GetCapture()!=s.main{s.slider_drag=false;s.pressed=0;s.settings.save();}
            for u in s.rx.try_iter(){s.ready=u.ready;s.busy=u.text.starts_with("Reset sent");s.raw_status=u.text;s.status=friendly(&s.raw_status,s.settings.lang);}
            for (enabled,error) in s.startup_rx.try_iter(){s.startup_enabled=enabled;s.startup_busy=false;if let Some(error)=error{s.startup_error=error;PostMessageW(s.main,0x8005,0,0);}}
            let animate=s.settings.animations&&s.win_animations;
            let ease=if animate{0.23}else{1.0};
            let on=if s.settings.enabled{1.0}else{0.0};let motion=if s.settings.animations{1.0}else{0.0};
            let startup=if s.startup_enabled{1.0}else{0.0};s.toggle_startup+=(startup-s.toggle_startup)*ease;
            s.toggle_on+=(on-s.toggle_on)*ease;s.toggle_motion+=(motion-s.toggle_motion)*ease;
            s.scale_visual+=(s.settings.scale-s.scale_visual)*ease;
            let mut cursor=Point::default();GetCursorPos(&mut cursor);ScreenToClient(s.main,&mut cursor);
            s.hover_main=if GetForegroundWindow()==s.main{main_window_hit(cursor)}else{0};
            position(s);
            let mut over_cursor=Point::default();GetCursorPos(&mut over_cursor);ScreenToClient(s.overlay,&mut over_cursor);
            s.hover=if s.visible{overlay_hit(s,over_cursor)}else{0};
            let lock_target=if s.hover==2{1.0}else{0.0};s.lock_hover+=(lock_target-s.lock_hover)*ease;
            if s.drag.is_some()&&GetCapture()!=s.overlay{let moved=s.drag.take().map(|d|d.moved).unwrap_or(false);s.pressed=0;if moved{s.settings.save();}}
            if s.visible{InvalidateRect(s.overlay,null(),0);}
            if IsIconic(s.main)==0&&IsWindowVisible(s.main)!=0{InvalidateRect(s.main,null(),0);}0
        }
        0x201=>{
            let p=Point{x:(l as u16 as i16) as i32,y:((l>>16) as u16 as i16) as i32};
            if over{
                if overlay_lock(s.over_w).contains(p){s.pressed=2;SetCapture(h);}
                else if !s.settings.locked&&overlay_grip(s.over_w).contains(p){
                    s.pressed=3;let mut cur=Point::default();GetCursorPos(&mut cur);let mut origin=Point::default();ClientToScreen(h,&mut origin);
                    s.drag=Some(Drag{start:cur,left:origin.x,top:origin.y,moved:false,resize:true,scale:s.settings.scale});SetCapture(h);
                }
                else if overlay_action(s.over_w,s.over_h).contains(p){
                    s.pressed=1;
                    if !s.settings.locked{let mut cur=Point::default();GetCursorPos(&mut cur);let mut origin=Point::default();ClientToScreen(h,&mut origin);
                        s.drag=Some(Drag{start:cur,left:origin.x,top:origin.y,moved:false,resize:false,scale:s.settings.scale});}
                    SetCapture(h);
                }else if !s.settings.locked&&Rect::new(0,0,s.over_w,s.over_h).contains(p){
                    s.pressed=0;let mut cur=Point::default();GetCursorPos(&mut cur);let mut origin=Point::default();ClientToScreen(h,&mut origin);
                    s.drag=Some(Drag{start:cur,left:origin.x,top:origin.y,moved:false,resize:false,scale:s.settings.scale});SetCapture(h);
                }
            }else{s.pressed=main_window_hit(p);if s.pressed==16&&!s.settings.locked{s.slider_drag=true;slider_move(s,p.x);}SetCapture(h);}0
        }
        0x200=>{
            let p=Point{x:(l as u16 as i16) as i32,y:((l>>16) as u16 as i16) as i32};
            if over{
                s.hover=overlay_hit(s,p);
                if let (Some(d),Some(g))=(&mut s.drag,s.game){
                    let mut cur=Point::default();GetCursorPos(&mut cur);let dx=cur.x-d.start.x;let dy=cur.y-d.start.y;
                    if dx.abs()>=5||dy.abs()>=5{d.moved=true;}
                    if d.moved{
                        if d.resize{
                            // Project the top-right handle motion onto the size diagonal.
                            let delta=(dx as f32*232.0-dy as f32*94.0)/(232.0*232.0+94.0*94.0);
                            s.settings.scale=(d.scale+delta).clamp(0.85,1.6);
                            let (_,start_h)=overlay_size(d.scale);let (ow,oh)=overlay_size(s.settings.scale);
                            // Keep bottom-left fixed while top-right is stretched.
                            s.settings.ax=((d.left-g.left) as f32/(g.w()-ow).max(1) as f32).clamp(0.0,1.0);
                            s.settings.ay=((d.top+start_h-oh-g.top) as f32/(g.h()-oh).max(1) as f32).clamp(0.0,1.0);
                        }else{let (ow,oh)=overlay_size(s.settings.scale);s.settings.ax=((d.left+dx-g.left) as f32/(g.w()-ow).max(1) as f32).clamp(0.0,1.0);s.settings.ay=((d.top+dy-g.top) as f32/(g.h()-oh).max(1) as f32).clamp(0.0,1.0);}
                    }
                }
                position(s);InvalidateRect(h,null(),0);
            }else{s.hover_main=main_window_hit(p);if s.slider_drag{slider_move(s,p.x);}InvalidateRect(h,null(),0);}0
        }
        0x202=>{
            let p=Point{x:(l as u16 as i16) as i32,y:((l>>16) as u16 as i16) as i32};
            let id=s.pressed;s.pressed=0;let slider_dragged=s.slider_drag;s.slider_drag=false;let dragged=s.drag.take().map(|d|d.moved).unwrap_or(false);
            // ReleaseCapture may synchronously send CAPTURECHANGED; that branch does not borrow state.
            ReleaseCapture();
            if dragged||slider_dragged{s.settings.save();}
            else if over{
                if id==2&&overlay_lock(s.over_w).contains(p){s.settings.locked=!s.settings.locked;s.settings.save();}
                else if id==1&&overlay_action(s.over_w,s.over_h).contains(p)&&!overlay_lock(s.over_w).contains(p){request_reset(s);}
            }else if id!=0&&id==main_window_hit(p){match id{
                17=>{ShowWindow(s.main,6);},18=>{PostMessageW(s.main,0x10,0,0);},
                19=>{s.settings.lang=if s.settings.lang==Lang::Ru{Lang::En}else{Lang::Ru};s.status=friendly(&s.raw_status,s.settings.lang);if s.tray_added{let mut data=tray_data(s);let _=Shell_NotifyIconW(1,&mut data);}},
                20=>{if !s.startup_busy{s.startup_busy=true;let tx=s.startup_tx.clone();let enable=!s.startup_enabled;
                    thread::spawn(move||{let result=startup::configure(enable);let enabled=if result.is_ok(){enable}else{startup::query()};let _=tx.send((enabled,result.err()));});}},
                21=>{request_reset(s);},
                10=>s.settings.enabled=!s.settings.enabled,
                11=>s.settings.locked=!s.settings.locked,
                12=>change_scale(s,-0.05),13=>change_scale(s,0.05),14=>s.settings.animations=!s.settings.animations,
                15=>{s.settings.scale=1.0;s.settings.ax=0.92;s.settings.ay=0.18;s.settings.locked=false;},_=>{}
            }s.settings.save();position(s);}
            InvalidateRect(s.main,null(),0);InvalidateRect(s.overlay,null(),0);0
        }
        0x20a if over=>{let delta=((w>>16) as u16 as i16) as i32;if delta!=0{change_scale(s,if delta>0{0.05}else{-0.05});}0}
        0x215=>0,
        0x10 if !over=>{
            s.settings.save();
            if tray_install(s){let was_visible=IsWindowVisible(s.main)!=0;ShowWindow(s.main,0);if was_visible{tray_notice(s);}}else{
                PostMessageW(s.main,0x8004,0,0);
            }0
        }
        QUIT_APP if !over=>{tray_remove(s);s.settings.save();s.stop.store(true,Ordering::Release);let _=s.tx.try_send(Command::Stop);KillTimer(h,1);if IsWindow(s.notice)!=0{SetWindowLongPtrW(s.notice,-21,0);DestroyWindow(s.notice);}if IsWindow(s.overlay)!=0{SetWindowLongPtrW(s.overlay,-21,0);DestroyWindow(s.overlay);}SetWindowLongPtrW(h,-21,0);DestroyWindow(h);PostQuitMessage(0);0}
        0x10=>0,
        0x2=>0,
        _=>DefWindowProcW(h,msg,w,l)
    }
}
pub(super) fn run()->Result<(),String>{unsafe{
    let launched_at_logon=std::env::args().any(|a|a=="--startup");
    let _instance_guard=match instance_lock()?{
        Some(lock)=>lock,
        None=>{
            if !launched_at_logon&&!restore_existing(){
                let lang=Settings::load().lang;
                MessageBoxW(null_mut(),wide(tr(lang,"HS/RECONNECT уже запущен или ещё запускается. Откройте его через значок в трее.","HS/RECONNECT is already running or still starting. Open it using the tray icon.")).as_ptr(),wide("HS/RECONNECT").as_ptr(),0x40);
            }return Ok(());
        }
    };

    SetProcessDPIAware();let instance=GetModuleHandleW(null());let class=wide("HSRustReconnectDesktopV2");
    let logo=LoadImageW(instance,102usize as *const u16,1,256,256,0x8000);
    let main_logo=LoadImageW(instance,101usize as *const u16,1,256,256,0x8000);
    let wc=WndClass{style:0,proc:Some(proc),cls_extra:0,wnd_extra:0,instance,icon:main_logo,cursor:LoadCursorW(null_mut(),32512usize as *const u16),background:null_mut(),menu:null(),name:class.as_ptr()};
    if RegisterClassW(&wc)==0{return Err(os_error("Register UI"));}
    // Main settings window is NOT topmost; overlay is a separate no-activate popup.
    let main=CreateWindowExW(0x000c0000,class.as_ptr(),wide("HS/RECONNECT · 0.6.3").as_ptr(),0x800a0000,120,100,536,568+CHROME,null_mut(),null_mut(),instance,null_mut());
    if main.is_null(){return Err(os_error("Create settings window"));}
    let overlay=CreateWindowExW(0x08080088,class.as_ptr(),wide("HS Reconnect Overlay").as_ptr(),0x80000000,0,0,232,94,null_mut(),null_mut(),instance,null_mut());
    if overlay.is_null(){DestroyWindow(main);return Err(os_error("Create overlay"));}
    let notice=CreateWindowExW(0x08080088,class.as_ptr(),wide("HS Reconnect Tray Notice").as_ptr(),0x80000000,0,0,360,112,null_mut(),null_mut(),instance,null_mut());
    if notice.is_null(){DestroyWindow(overlay);DestroyWindow(main);return Err(os_error("Create tray notice"));}
    let small=LoadImageW(instance,101usize as *const u16,1,GetSystemMetrics(49).max(16),GetSystemMetrics(50).max(16),0x8000);
    let big=LoadImageW(instance,101usize as *const u16,1,GetSystemMetrics(11).max(32),GetSystemMetrics(12).max(32),0x8000);
    SendMessageW(main,0x80,0,small as isize);SendMessageW(main,0x80,1,big as isize);
    let dark=1i32;let _=DwmSetWindowAttribute(main,20,(&dark as *const i32).cast(),4);
    // Captionless main window: title stays available to the taskbar/Alt+Tab, not painted.
    // A blank draggable strip with custom minimize/close replaces native chrome.
    let mut animate=1i32;SystemParametersInfoW(0x1042,0,(&mut animate as *mut i32).cast(),0);
    let (tx,rx)=mpsc::sync_channel(1);let (utx,urx)=mpsc::channel();let stop=Arc::new(AtomicBool::new(false));let ws=Arc::clone(&stop);
    let settings=Settings::load();let toggle_on=if settings.enabled{1.0}else{0.0};let toggle_motion=if settings.animations{1.0}else{0.0};let scale_visual=settings.scale;
    let taskbar_message=RegisterWindowMessageW(wide("TaskbarCreated").as_ptr());
    let (startup_tx,startup_rx)=mpsc::channel();
    let initial_status=friendly("Waiting for Hearthstone",settings.lang);
    let state=Box::new(Ui{main,overlay,notice,notice_until:None,tx,rx:urx,stop,settings,ready:false,busy:false,status:initial_status,raw_status:"Waiting for Hearthstone".into(),hover:0,pressed:0,drag:None,game:None,frame:0,over_w:0,over_h:0,visible:false,win_animations:animate!=0,logo,main_logo,hover_main:0,slider_drag:false,toggle_on,toggle_motion,scale_visual,lock_hover:0.0,tray_added:false,tray_icon:big,taskbar_message,startup_enabled:false,startup_busy:true,toggle_startup:0.0,startup_error:String::new(),startup_tx,startup_rx});
    let p=Box::into_raw(state);SetWindowLongPtrW(main,-21,p as isize);SetWindowLongPtrW(overlay,-21,p as isize);SetWindowLongPtrW(notice,-21,p as isize);
    let background=thread::spawn(move||worker(rx,utx,ws));
    let _=tray_install(&mut *p);
    let startup_sender=(&(*p).startup_tx).clone();thread::spawn(move||{let _=startup_sender.send((startup::query(),None));});
    if !launched_at_logon||!(*p).tray_added{ShowWindow(main,1);InvalidateRect(main,null(),0);PostMessageW(main,0xf,0,0);}
    if SetTimer(main,1,33,null())==0{SendMessageW(main,QUIT_APP,0,0);drop(Box::from_raw(p));let _=background.join();return Err(os_error("UI refresh timer"));}
    let mut msg:Msg=zeroed();let mut failed=false;
    loop{let r=GetMessageW(&mut msg,null_mut(),0,0);if r==0{break;}if r<0{failed=true;SendMessageW(main,QUIT_APP,0,0);break;}TranslateMessage(&msg);DispatchMessageW(&msg);}
    drop(Box::from_raw(p));let _=background.join();if failed{Err(os_error("Message loop"))}else{Ok(())}
}}

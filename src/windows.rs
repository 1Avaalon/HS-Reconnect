//! Native Win32 UI + IP Helper API. No injection, hooks, drivers or firewall changes.
use hs_rust_reconnect::{cached_connection_valid, latest_endpoint, select_connection, Connection, Endpoint};
use std::{ffi::c_void, fs::{self, OpenOptions}, io::{Read, Seek, SeekFrom, Write},
    mem::{size_of, zeroed}, os::windows::fs::OpenOptionsExt, path::{Path, PathBuf},
    ptr::{null, null_mut}, sync::{mpsc::{self, Receiver, SyncSender}, Arc, atomic::{AtomicBool, Ordering}}, thread,
    time::{Duration, Instant, SystemTime}};

type Handle = *mut c_void;
type Hwnd = Handle;
#[repr(C)] #[derive(Default, Copy, Clone)] struct Point { x:i32, y:i32 }
#[repr(C)] struct Msg { hwnd:Hwnd, message:u32, wparam:usize, lparam:isize, time:u32, pt:Point, private:u32 }
#[repr(C)] struct WndClass { style:u32, proc:Option<unsafe extern "system" fn(Hwnd,u32,usize,isize)->isize>, cls_extra:i32, wnd_extra:i32, instance:Handle, icon:Handle, cursor:Handle, background:Handle, menu:*const u16, name:*const u16 }
#[repr(C)] struct ProcessEntry { size:u32, usage:u32, pid:u32, heap:usize, module:u32, threads:u32, parent:u32, priority:i32, flags:u32, exe:[u16;260] }
#[repr(C)] #[derive(Copy,Clone)] struct TcpOwner { state:u32, local_addr:u32, local_port:u32, remote_addr:u32, remote_port:u32, pid:u32 }
#[repr(C)] struct TcpBasic { state:u32, local_addr:u32, local_port:u32, remote_addr:u32, remote_port:u32 }
#[repr(C)] #[derive(Copy,Clone)] struct FileTime { low:u32, high:u32 }

#[link(name="kernel32")] extern "system" {
    fn GetModuleHandleW(name:*const u16)->Handle;
    fn CreateToolhelp32Snapshot(flags:u32,pid:u32)->Handle;
    fn Process32FirstW(h:Handle,p:*mut ProcessEntry)->i32;
    fn Process32NextW(h:Handle,p:*mut ProcessEntry)->i32;
    fn CloseHandle(h:Handle)->i32;
    fn OpenProcess(access:u32,inherit:i32,pid:u32)->Handle;
    fn QueryFullProcessImageNameW(h:Handle,flags:u32,buf:*mut u16,len:*mut u32)->i32;
    fn GetProcessTimes(h:Handle,create:*mut FileTime,exit:*mut FileTime,kernel:*mut FileTime,user:*mut FileTime)->i32;
}
#[link(name="iphlpapi")] extern "system" {
    fn GetExtendedTcpTable(table:*mut c_void,size:*mut u32,order:i32,family:u32,class:u32,reserved:u32)->u32;
    fn SetTcpEntry(row:*const TcpBasic)->u32;
}
#[link(name="user32")] extern "system" {
    fn RegisterClassW(c:*const WndClass)->u16;
    fn CreateWindowExW(ex:u32,class:*const u16,title:*const u16,style:u32,x:i32,y:i32,w:i32,h:i32,parent:Hwnd,menu:Handle,instance:Handle,param:*mut c_void)->Hwnd;
    fn DefWindowProcW(h:Hwnd,msg:u32,w:usize,l:isize)->isize;
    fn GetMessageW(msg:*mut Msg,h:Hwnd,min:u32,max:u32)->i32;
    fn TranslateMessage(msg:*const Msg)->i32;
    fn DispatchMessageW(msg:*const Msg)->isize;
    fn PostQuitMessage(code:i32);
    fn DestroyWindow(h:Hwnd)->i32;
    fn SetWindowLongPtrW(h:Hwnd,index:i32,value:isize)->isize;
    fn GetWindowLongPtrW(h:Hwnd,index:i32)->isize;
    fn SetWindowTextW(h:Hwnd,text:*const u16)->i32;
    fn EnableWindow(h:Hwnd,on:i32)->i32;
    fn SetTimer(h:Hwnd,id:usize,ms:u32,callback:*const c_void)->usize;
    fn KillTimer(h:Hwnd,id:usize)->i32;
    fn SetWindowPos(h:Hwnd,after:Hwnd,x:i32,y:i32,w:i32,height:i32,flags:u32)->i32;
    fn LoadCursorW(instance:Handle,name:*const u16)->Handle;
    fn SendMessageW(h:Hwnd,msg:u32,w:usize,l:isize)->isize;
    fn MessageBoxW(h:Hwnd,text:*const u16,title:*const u16,kind:u32)->i32;
    fn SetProcessDPIAware()->i32;
}
#[link(name="gdi32")] extern "system" { fn GetStockObject(index:i32)->Handle; }
fn wide(s:&str)->Vec<u16> { s.encode_utf16().chain(Some(0)).collect() }
fn os_error(context:&str)->String { format!("{context}: {}",std::io::Error::last_os_error()) }
struct Owned(Handle);
impl Drop for Owned { fn drop(&mut self) { unsafe { CloseHandle(self.0); } } }
#[derive(Clone)] struct Game { pid:u32, path:PathBuf, started:SystemTime }
fn find_game()->Result<Game,String> { unsafe {
    let raw=CreateToolhelp32Snapshot(2,0);
    if raw as isize == -1 { return Err(os_error("Process list")); }
    let snap=Owned(raw);let mut entry:ProcessEntry=zeroed();entry.size=size_of::<ProcessEntry>() as u32;
    let mut ok=Process32FirstW(snap.0,&mut entry); let mut pids=Vec::new();
    while ok!=0 {
        let end=entry.exe.iter().position(|c|*c==0).unwrap_or(260);
        if String::from_utf16_lossy(&entry.exe[..end]).eq_ignore_ascii_case("Hearthstone.exe") {pids.push(entry.pid);}
        ok=Process32NextW(snap.0,&mut entry);
    }
    if pids.len()!=1 { return Err(if pids.is_empty(){"Waiting for Hearthstone"}else{"Multiple game processes; nothing disconnected"}.into()); }
    let h=OpenProcess(0x1000,0,pids[0]); // QUERY_LIMITED_INFORMATION, never VM_READ/VM_WRITE
    if h.is_null(){return Err(os_error("Query game path"));}let h=Owned(h);
    let mut buf=vec![0u16;32768];let mut len=buf.len() as u32;
    if QueryFullProcessImageNameW(h.0,0,buf.as_mut_ptr(),&mut len)==0{return Err(os_error("Query game path"));}
    let mut ct:FileTime=zeroed();let mut et=ct;let mut kt=ct;let mut ut=ct;
    if GetProcessTimes(h.0,&mut ct,&mut et,&mut kt,&mut ut)==0{return Err(os_error("Query game start time"));}
    let ticks=((ct.high as u64)<<32)|ct.low as u64;
    let secs=ticks/10_000_000;
    let unix=secs.checked_sub(11_644_473_600).ok_or("Invalid process start time")?;
    Ok(Game{pid:pids[0],path:PathBuf::from(String::from_utf16_lossy(&buf[..len as usize])),started:SystemTime::UNIX_EPOCH+Duration::new(unix,((ticks%10_000_000)*100) as u32)})
} }
fn tcp_rows()->Result<Vec<Connection>,String> { unsafe {
    let mut bytes=0u32;
    let code=GetExtendedTcpTable(null_mut(),&mut bytes,0,2,5,0); // AF_INET, OWNER_PID_ALL
    if code!=122 && code!=0{return Err(format!("TCP table error {code}"));}
    for _ in 0..4 {
        if bytes>16*1024*1024{return Err("TCP table too large".into());}
        let mut aligned=vec![0u32;((bytes as usize+3)/4).max(1)];
        let cap=aligned.len()*4;let mut supplied=cap as u32;
        let code=GetExtendedTcpTable(aligned.as_mut_ptr().cast(),&mut supplied,0,2,5,0);
        if code==122 {bytes=supplied;continue;}
        if code!=0{return Err(format!("TCP table error {code}"));}
        let count=aligned[0] as usize;
        if 4+count.saturating_mul(size_of::<TcpOwner>())>cap{return Err("Invalid TCP table length".into());}
        let base=aligned.as_ptr().cast::<u8>().add(4);
        let mut out=Vec::with_capacity(count);
        for i in 0..count {
            let r=std::ptr::read_unaligned(base.add(i*size_of::<TcpOwner>()).cast::<TcpOwner>());
            out.push(Connection{pid:r.pid,state:r.state,local_addr:r.local_addr,local_port:r.local_port,remote_addr:r.remote_addr,remote_port:r.remote_port});
        }
        return Ok(out);
    }
    Err("TCP table changed repeatedly; try again".into())
} }
fn walk_logs(dir:&Path,depth:u8,budget:&mut usize,out:&mut Vec<(SystemTime,PathBuf)>){
    if depth>3 || *budget==0{return;}
    let Ok(entries)=fs::read_dir(dir) else{return;};
    for item in entries.flatten() {
        if *budget==0 {break;}*budget-=1;
        let Ok(ft)=item.file_type() else{continue;};
        if ft.is_symlink(){continue;}
        let p=item.path();
        if ft.is_dir(){walk_logs(&p,depth+1,budget,out);}
        else if ft.is_file() && ["GameNetLogger.log","Hearthstone.log"].iter().any(|n|item.file_name().to_string_lossy().eq_ignore_ascii_case(n)) {
            if let Ok(t)=item.metadata().and_then(|m|m.modified()){out.push((t,p));}
        }
    }
}
#[derive(Clone)] struct Target { game:Game, row:Connection, endpoint:Endpoint }
fn read_endpoint(path:&Path)->Result<Option<Endpoint>,String>{
    // Share log access with the game. Only the bounded tail is read.
    let mut f=OpenOptions::new().read(true).share_mode(1|2|4).open(path).map_err(|e|format!("Read log: {e}"))?;
    let len=f.metadata().map_err(|e|e.to_string())?.len();let start=len.saturating_sub(262144);
    f.seek(SeekFrom::Start(start)).map_err(|e|e.to_string())?;
    let mut bytes=Vec::new();f.take(262144).read_to_end(&mut bytes).map_err(|e|e.to_string())?;
    // Drop a partial first line so it cannot create a spurious marker.
    let text=String::from_utf8_lossy(&bytes);
    let text=if start>0 {text.split_once('\n').map(|(_,t)|t).unwrap_or("")}else{&text};
    Ok(latest_endpoint(text))
}
fn same_game(a:&Game,b:&Game)->bool {a.pid==b.pid && a.path==b.path && a.started==b.started}
fn current_log_candidate(game:&Game)->Result<Option<(SystemTime,PathBuf,Endpoint)>,String>{
    let root=game.path.parent().ok_or("Missing game directory")?.join("Logs");
    let mut files=Vec::new();walk_logs(&root,0,&mut 2000,&mut files);
    // Prefer GameNetLogger over the general log to avoid unrelated address markers.
    // Within the same class use newest modified file, only from the current process session.
    files.retain(|(modified,_)|*modified>=game.started);
    files.sort_by(|a,b|{
        let a_net=a.1.file_name().map(|n|n.to_string_lossy().eq_ignore_ascii_case("GameNetLogger.log")).unwrap_or(false);
        let b_net=b.1.file_name().map(|n|n.to_string_lossy().eq_ignore_ascii_case("GameNetLogger.log")).unwrap_or(false);
        b_net.cmp(&a_net).then_with(||b.0.cmp(&a.0))
    });
    let mut last_read_error=None;
    for (modified,path) in files.into_iter().take(1) {
        match read_endpoint(&path){
            Ok(Some(ep))=>return Ok(Some((modified,path,ep))),
            Ok(None)=>{},
            Err(e)=>last_read_error=Some(e),
        }
    }
    if let Some(e)=last_read_error {Err(e)}else{Ok(None)}
}
fn discover(previous:Option<&Target>)->Result<Target,String>{
    let game=find_game()?;
    let rows=tcp_rows()?;
    let log_candidate=current_log_candidate(&game);
    if let Ok(Some((_,_,endpoint)))=&log_candidate {
        // A new explicit endpoint takes priority over the old cache. Never
        // silently reset a cached old match if the logs name a different match.
        let row=select_connection(&rows,game.pid,*endpoint)?;
        return Ok(Target{game,row,endpoint:*endpoint});
    }
    if let Some(old)=previous {
        if same_game(&old.game,&game) && cached_connection_valid(&rows,game.pid,old.row) {
            // No rolling-tail match / temporary log read failure: keep only
            // the exact tuple already established earlier in this process.
            return Ok(old.clone());
        }
    }
    match log_candidate {
        Err(e)=>Err(e),
        _=>Err("No current-session match address. Waiting for game logs".into()),
    }
}
fn reset(target:&Target)->Result<String,String>{
    let game=find_game()?;
    if !same_game(&game,&target.game){return Err("Game process changed; nothing disconnected".into());}
    // No expiry by wall-clock/log age: validate reality instead.
    // A newly observed explicit address must still refer to this target.
    if let Ok(Some((_,_,ep)))=current_log_candidate(&game){
        if ep!=target.endpoint{return Err("Match address changed; waiting for rediscovery".into());}
    }
    let rows=tcp_rows()?;
    if !cached_connection_valid(&rows,game.pid,target.row){return Err("Connection changed; wait for refresh".into());}
    let selected=select_connection(&rows,game.pid,target.endpoint)?;
    if selected!=target.row{return Err("Connection changed; wait for refresh".into());}
    // One DELETE_TCB request, no automatic resets or broad connection kill.
    let row=TcpBasic{state:12,local_addr:selected.local_addr,local_port:selected.local_port,remote_addr:selected.remote_addr,remote_port:selected.remote_port};
    let code=unsafe {SetTcpEntry(&row)};
    if code!=0{return Err(match code{5|740=>"Administrator rights required".into(),50=>"TCP reset not supported".into(),87=>"Connection ended or parameters rejected".into(),_=>format!("SetTcpEntry error {code}")});}
    Ok("Reset sent. The game reconnects itself.".into())
}

fn local_log(text:&str){
    // Optional bounded local text log only; never transmits anything.
    if let Some(base)=std::env::var_os("LOCALAPPDATA") {
        let dir=PathBuf::from(base).join("HSRustReconnect");let _=fs::create_dir_all(&dir);let p=dir.join("events.log");
        if fs::metadata(&p).map(|m|m.len()>262144).unwrap_or(false){let _=fs::remove_file(&p);}
        if let Ok(mut f)=OpenOptions::new().create(true).append(true).open(p){let _=writeln!(f,"{:?}: {}",SystemTime::now(),text.replace(['\r','\n']," "));}
    }
}
enum Command { Reconnect, Stop }
struct Update { text:String, ready:bool }
fn worker(rx:Receiver<Command>,tx:mpsc::Sender<Update>,stop:Arc<AtomicBool>){
    local_log("version=0.3.0; worker started; discovery independent of foreground window");
    let mut target:Option<Target>=None;let mut last_state=String::new();let mut last_reset=Instant::now()-Duration::from_secs(10);
    loop {
        if stop.load(Ordering::Acquire){break;}
        let command=rx.recv_timeout(Duration::from_millis(750));
        if stop.load(Ordering::Acquire){break;}
        match command {
            Ok(Command::Stop)|Err(mpsc::RecvTimeoutError::Disconnected)=>break,
            Ok(Command::Reconnect)=>{
                if last_reset.elapsed()<Duration::from_secs(2){continue;}
                if stop.load(Ordering::Acquire){break;}
                let result=target.as_ref().ok_or_else(||"No verified target".to_string()).and_then(reset);
                let text=match result{Ok(s)=>s,Err(s)=>format!("Not reset: {s}")};
                local_log(&text);let _=tx.send(Update{text,ready:false});target=None;
                last_reset=Instant::now();
            }
            Err(mpsc::RecvTimeoutError::Timeout)=>{}
        }
        if last_reset.elapsed()<Duration::from_secs(2){continue;}
        match discover(target.as_ref()){
            Ok(t)=>{let text="Ready: verified match connection".to_string();
                if last_state!="ready"{local_log("state=ready; current game process and connection verified");last_state="ready".into();}
                target=Some(t);if tx.send(Update{text,ready:true}).is_err(){break;}}
            Err(e)=>{
                // If an explicit new endpoint is invalid/ambiguous, invalidate
                // the old identity too. Temporary unreadable logs with a live
                // exact cached tuple were already accepted inside discover().
                target=None;
                if last_state!=e{local_log(&format!("state=not_ready; {e}"));last_state=e.clone();}
                if tx.send(Update{text:e,ready:false}).is_err(){break;}}
        }
    }
}
mod desktop;
pub fn run()->Result<(),String>{desktop::run()}
pub fn error(text:&str){unsafe{MessageBoxW(null_mut(),wide(text).as_ptr(),wide("HS Rust Reconnect").as_ptr(),0x10);}}

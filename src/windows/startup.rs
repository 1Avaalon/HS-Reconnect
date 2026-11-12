//! Optional, current-user logon startup. No task is created until explicitly enabled.
use std::{fs,os::windows::process::CommandExt,path::PathBuf,process::{Command,Output,Stdio},thread,time::{Duration,Instant}};
#[link(name="secur32")] extern "system"{fn GetUserNameExW(format:u32,name:*mut u16,size:*mut u32)->u8;}
#[link(name="kernel32")] extern "system"{
    fn GetOEMCP()->u32;
    fn MultiByteToWideChar(codepage:u32,flags:u32,input:*const u8,bytes:i32,output:*mut u16,chars:i32)->i32;
}
fn account()->Result<String,String>{unsafe{
    let mut name=vec![0u16;32768];let mut size=name.len() as u32;
    if GetUserNameExW(2,name.as_mut_ptr(),&mut size)==0{return Err(format!("Read Windows account: {}",std::io::Error::last_os_error()));}
    Ok(String::from_utf16_lossy(&name[..size as usize]).trim_end_matches('\0').to_string())
}}
fn task_name()->Result<String,String>{Ok(format!("HSRustReconnect-{}",account()?.chars().map(|c|if c.is_alphanumeric(){c}else{'_'}).collect::<String>()))}
fn run(args:&[&str])->Result<Output,String>{
    let exe=std::env::var_os("SystemRoot").map(PathBuf::from).unwrap_or_else(||PathBuf::from(r"C:\Windows")).join("System32/schtasks.exe");
    let mut child=Command::new(exe).args(args).stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::piped()).creation_flags(0x08000000).spawn().map_err(|e|e.to_string())?;
    let deadline=Instant::now()+Duration::from_secs(10);
    loop{
        match child.try_wait(){
            Ok(Some(_))=>return child.wait_with_output().map_err(|e|e.to_string()),
            Ok(None)=>{},Err(e)=>{let _=child.kill();let _=child.wait();return Err(e.to_string());}
        }
        if Instant::now()>=deadline{let _=child.kill();let _=child.wait();return Err("Windows Task Scheduler did not respond in 10 seconds".into());}
        thread::sleep(Duration::from_millis(50));
    }
}
// schtasks can return Unicode XML or localized text in the Windows OEM code page.
fn decode(bytes:&[u8])->String{
    let le=bytes.starts_with(&[0xff,0xfe]);let be=bytes.starts_with(&[0xfe,0xff]);
    let bare_le=bytes.len()>=4&&bytes[1]==0&&bytes[3]==0;
    let bare_be=bytes.len()>=4&&bytes[0]==0&&bytes[2]==0;
    if le||be||bare_le||bare_be{
        let data=if le||be{&bytes[2..]}else{bytes};
        let chars=data.chunks_exact(2).map(|a|if be||bare_be{u16::from_be_bytes([a[0],a[1]])}else{u16::from_le_bytes([a[0],a[1]])}).collect::<Vec<_>>();
        return String::from_utf16_lossy(&chars).trim_end_matches('\0').to_string();
    }
    if let Ok(text)=std::str::from_utf8(bytes){return text.trim_start_matches('\u{feff}').to_string();}
    if bytes.len()<=i32::MAX as usize{unsafe{
        let cp=GetOEMCP();let n=MultiByteToWideChar(cp,0,bytes.as_ptr(),bytes.len() as i32,std::ptr::null_mut(),0);
        if n>0{let mut chars=vec![0u16;n as usize];let used=MultiByteToWideChar(cp,0,bytes.as_ptr(),bytes.len() as i32,chars.as_mut_ptr(),n);
            if used>0{return String::from_utf16_lossy(&chars[..used as usize]);}
        }
    }}
    String::from_utf8_lossy(bytes).into_owned()
}
fn task_file_bytes(text:&str)->Vec<u8>{
    // Task Scheduler reliably accepts UTF-16LE with an explicit BOM and matching declaration.
    let mut bytes=Vec::with_capacity(2+text.len()*2);bytes.extend_from_slice(&[0xff,0xfe]);
    for unit in text.encode_utf16(){bytes.extend_from_slice(&unit.to_le_bytes());}bytes
}

pub(super) fn query()->bool{
    let Ok(name)=task_name()else{return false;};let Ok(out)=run(&["/Query","/TN",&name,"/XML"])else{return false;};
    out.status.success()&&decode(&out.stdout).contains("<Enabled>true</Enabled>")
}
fn xml(s:&str)->String{s.replace('&',"&amp;").replace('<',"&lt;").replace('>',"&gt;").replace('"',"&quot;").replace('\'',"&apos;")}
pub(super) fn configure(enable:bool)->Result<(),String>{
    let name=task_name()?;
    let output=if enable{
        let user=account()?;let exe=std::env::current_exe().map_err(|e|e.to_string())?;
        let folder=std::env::var_os("LOCALAPPDATA").map(PathBuf::from).ok_or("LOCALAPPDATA unavailable")?.join("HSRustReconnect");
        fs::create_dir_all(&folder).map_err(|e|e.to_string())?;
        let file=folder.join(format!("startup-task-{}.xml",std::process::id()));
        let command=xml(&exe.to_string_lossy());let directory=xml(&exe.parent().ok_or("Executable directory unavailable")?.to_string_lossy());let user=xml(&user);
        let text=format!(r#"<?xml version="1.0" encoding="UTF-16"?>
<Task version="1.2" xmlns="http://schemas.microsoft.com/windows/2004/02/mit/task">
<RegistrationInfo><Description>HS/RECONNECT: optional startup for the current Windows account.</Description></RegistrationInfo>
<Triggers><LogonTrigger><Enabled>true</Enabled><UserId>{user}</UserId><Delay>PT10S</Delay></LogonTrigger></Triggers>
<Principals><Principal id="Owner"><UserId>{user}</UserId><LogonType>InteractiveToken</LogonType><RunLevel>HighestAvailable</RunLevel></Principal></Principals>
<Settings><MultipleInstancesPolicy>IgnoreNew</MultipleInstancesPolicy><DisallowStartIfOnBatteries>false</DisallowStartIfOnBatteries><StopIfGoingOnBatteries>false</StopIfGoingOnBatteries><StartWhenAvailable>true</StartWhenAvailable><AllowStartOnDemand>true</AllowStartOnDemand><Enabled>true</Enabled><ExecutionTimeLimit>PT0S</ExecutionTimeLimit></Settings>
<Actions Context="Owner"><Exec><Command>{command}</Command><Arguments>--startup</Arguments><WorkingDirectory>{directory}</WorkingDirectory></Exec></Actions>
</Task>"#);
        fs::write(&file,task_file_bytes(&text)).map_err(|e|e.to_string())?;
        let result=run(&["/Create","/TN",&name,"/XML",&file.to_string_lossy(),"/F"]);let _=fs::remove_file(file);result?
    }else{run(&["/Delete","/TN",&name,"/F"])?};
    if output.status.success(){Ok(())}else{Err(format!("{} {}",decode(&output.stdout),decode(&output.stderr)).trim().to_string())}
}

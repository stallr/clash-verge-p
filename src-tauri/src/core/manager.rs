use std::borrow::Cow;

/// 给clash内核的tun模式授权
#[cfg(any(target_os = "macos", target_os = "linux"))]
pub fn grant_permission(core: String) -> anyhow::Result<()> {
    use std::process::Command;
    use tauri::utils::platform::current_exe;

    let path = current_exe()?.with_file_name(core).canonicalize()?;
    let path = path.display().to_string();

    log::debug!("grant_permission path: {path}");

    #[cfg(target_os = "macos")]
    if getcore_path(&path) {
        Ok(())
    }else{
        let output = {
            // the path of clash /Applications/Clash Verge.app/Contents/MacOS/clash
            // https://apple.stackexchange.com/questions/82967/problem-with-empty-spaces-when-executing-shell-commands-in-applescript
            // let path = escape(&path);
            let path = path.replace(' ', "\\\\ ");
            let shell = format!("chown root:admin {path}\nchmod +sx {path}");
            let command = format!(r#"do shell script "{shell}" with administrator privileges"#);
            Command::new("osascript")
                .args(vec!["-e", &command])
                .output()?
        };
        if output.status.success() {
            Ok(())
        } else {
            let stderr = std::str::from_utf8(&output.stderr).unwrap_or("");
            anyhow::bail!("{stderr}");
        }
    }

    #[cfg(target_os = "linux")]
    let output = {
        let path = path.replace(' ', "\\ "); // 避免路径中有空格
        let shell = format!("setcap cap_net_bind_service,cap_net_admin=+ep {path}");

        let sudo = match Command::new("which").arg("pkexec").output() {
            Ok(output) => {
                if output.stdout.is_empty() {
                    "sudo"
                } else {
                    "pkexec"
                }
            }
            Err(_) => "sudo",
        };

        Command::new(sudo).arg("sh").arg("-c").arg(shell).output()?
    };

    #[cfg(target_os = "linux")]
    if output.status.success() {
        Ok(())
    } else {
        let stderr = std::str::from_utf8(&output.stderr).unwrap_or("");
        anyhow::bail!("{stderr}");
    }
}

#[cfg(target_os = "macos")]
pub fn getcore_path(path: &str) -> bool {
    use std::fs;
    use std::os::unix::fs::MetadataExt;
    match fs::metadata(path) {
        Ok(metadata) => {
            let is_owner_root = metadata.uid() == 0;
            let is_group_admin = metadata.gid() == 80;
            let permissions = metadata.mode();
            let is_setuid_set = permissions & 0o4000 != 0;
            let is_setgid_set = permissions & 0o2000 != 0;
            is_owner_root && is_group_admin && is_setuid_set && is_setgid_set
        }
        Err(_) => false,
    }
}

#[allow(unused)]
pub fn escape<'a>(text: &'a str) -> Cow<'a, str> {
    let bytes = text.as_bytes();

    let mut owned = None;

    for pos in 0..bytes.len() {
        let special = match bytes[pos] {
            b' ' => Some(b' '),
            _ => None,
        };
        if let Some(s) = special {
            if owned.is_none() {
                owned = Some(bytes[0..pos].to_owned());
            }
            owned.as_mut().unwrap().push(b'\\');
            owned.as_mut().unwrap().push(b'\\');
            owned.as_mut().unwrap().push(s);
        } else if let Some(owned) = owned.as_mut() {
            owned.push(bytes[pos]);
        }
    }

    if let Some(owned) = owned {
        unsafe { Cow::Owned(String::from_utf8_unchecked(owned)) }
    } else {
        unsafe { Cow::Borrowed(std::str::from_utf8_unchecked(bytes)) }
    }
}

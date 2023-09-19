use std::{
    env,
    ffi::CStr,
    fs::Metadata,
    mem,
    os::unix::{fs::MetadataExt, prelude::PermissionsExt},
    process::exit,
    ptr,
};

use ansiterm::{
    ANSIGenericString,
    Color::{Blue, Green, Purple, Red, Yellow},
};
use chrono::prelude::*;

mod permission;

fn main() {
    let dir = match env::current_dir() {
        Ok(path) => path,
        Err(_) => {
            println!("Error: cannot get current directory");
            exit(1);
        }
    };

    let content = match std::fs::read_dir(dir) {
        Ok(content) => content,
        Err(_) => {
            println!("Error: cannot read current directory");
            exit(1);
        }
    };

    for entry in content {
        let entry = match entry {
            Ok(entry) => entry,
            Err(_) => {
                println!("Error: cannot read current directory");
                exit(1);
            }
        };

        let metadata = match entry.metadata() {
            Ok(metadata) => metadata,
            Err(_) => {
                println!("Error: cannot read current directory");
                exit(1);
            }
        };

        let is_dir = match metadata.is_dir() {
            true => ".",
            false => "d",
        };

        let mode = metadata.permissions().mode();
        let perm = colorize_perm(mode);
        let filename = entry.file_name();
        let size = metadata.size().to_string();

        let owner = get_unix_username(metadata.uid()).unwrap();
        let group = get_unix_username(metadata.gid()).unwrap();

        println!(
            "{0}{1}{2}{3} {4: >22} {5} {6} {7: >7} {8: >10}",
            Blue.bold().paint(is_dir).to_string(),
            perm.0,
            perm.1,
            perm.2,
            Green.bold().paint(size).to_string(),
            Yellow.bold().paint(owner).to_string(),
            Yellow.bold().paint(group).to_string(),
            get_date_time(metadata),
            Purple.paint(filename.to_str().unwrap()).to_string()
        );
    }
}

fn get_unix_username(uid: u32) -> Option<String> {
    unsafe {
        let mut result = ptr::null_mut();
        let amt = match libc::sysconf(libc::_SC_GETPW_R_SIZE_MAX) {
            n if n < 0 => 512usize,
            n => n as usize,
        };
        let mut buf = Vec::with_capacity(amt);
        let mut passwd: libc::passwd = mem::zeroed();

        match libc::getpwuid_r(
            uid,
            &mut passwd,
            buf.as_mut_ptr(),
            buf.capacity() as libc::size_t,
            &mut result,
        ) {
            0 if !result.is_null() => {
                let ptr = passwd.pw_name as *const _;
                let username = CStr::from_ptr(ptr).to_str().unwrap().to_owned();
                Some(username)
            }
            _ => None,
        }
    }
}

fn get_date_time(metadata: Metadata) -> ANSIGenericString<'static, str> {
    let modified_date = metadata.mtime();
    let native = NaiveDateTime::from_timestamp_opt(modified_date, 0);
    let time: &DateTime<FixedOffset> = &DateTime::from_naive_utc_and_offset(
        native.unwrap(),
        FixedOffset::east_opt(9 * 3600).unwrap(),
    );

    let locale: locale::Time =
        locale::Time::load_user_locale().unwrap_or_else(|_| locale::Time::english());
    let day = time.day0().to_string() + " ";
    let month = &*locale.short_month_name(time.month0() as usize).to_string();
    return Blue.paint(day + month);
}

fn colorize_perm(mode: u32) -> (String, String, String) {
    let user_perm = Yellow
        .bold()
        .paint(if (mode & (0x1 << 8)) >= 1 { "r" } else { " " })
        .to_string()
        + &Red
            .bold()
            .paint(if (mode & (0x1 << 7)) >= 1 { "w" } else { " " })
            .to_string()
        + &Green
            .bold()
            .paint(if (mode & (0x1 << 6)) >= 1 { "x" } else { " " })
            .to_string();
    let group_perm = Yellow
        .bold()
        .paint(if (mode & (0x1 << 5)) >= 1 { "r" } else { " " })
        .to_string()
        + &Red
            .bold()
            .paint(if (mode & (0x1 << 4)) >= 1 { "w" } else { " " })
        + &Green
            .bold()
            .paint(if (mode & (0x1 << 3)) >= 1 { "x" } else { " " })
            .to_string();

    let other_perm = Yellow
        .bold()
        .paint(if (mode & (0x1 << 2)) >= 1 { "r" } else { " " })
        .to_string()
        + &Red
            .bold()
            .paint(if (mode & (0x1 << 1)) >= 1 { "w" } else { " " })
            .to_string()
        + &Green
            .bold()
            .paint(if (mode & 0x1) >= 1 { "x" } else { " " })
            .to_string();
    return (user_perm, group_perm, other_perm);
}

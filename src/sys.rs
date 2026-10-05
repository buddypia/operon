use crate::prelude::*;
use crate::*;

pub(crate) fn reveal_path(path: &Path) -> Result<()> {
    let mut command = Command::new("open");
    command.arg("-R").arg(path);
    run_system_command(&mut command)
        .with_context(|| tf!("{p0} を Finder で開く処理", p0 = path.display()))
}
pub(crate) fn open_path(path: &Path) -> Result<()> {
    let mut command = Command::new("open");
    command.arg(path);
    run_system_command(&mut command).with_context(|| format!("opening {}", path.display()))
}
pub(crate) fn open_url(url: &str) -> Result<()> {
    let mut command = Command::new("open");
    command.arg(url);
    run_system_command(&mut command).with_context(|| format!("opening {url}"))
}
use anyhow::bail;
use std::ffi::{c_char, c_void, CString};

#[repr(C)]
struct BlockDescriptor {
    reserved: libc::c_ulong,
    size: libc::c_ulong,
}

static BLOCK_DESCRIPTOR: BlockDescriptor = BlockDescriptor {
    reserved: 0,
    size: std::mem::size_of::<BlockLiteral>() as libc::c_ulong,
};

#[repr(C)]
struct BlockLiteral {
    isa: *const c_void,
    flags: libc::c_int,
    reserved: libc::c_int,
    invoke: unsafe extern "C" fn(*mut BlockLiteral, libc::c_int, *mut c_void),
    descriptor: *const BlockDescriptor,
}

unsafe extern "C" fn auth_completion_invoke(
    _block: *mut BlockLiteral,
    _granted: libc::c_int,
    _error: *mut c_void,
) {
}

extern "C" {
    fn objc_getClass(name: *const c_char) -> *mut c_void;
    fn sel_registerName(name: *const c_char) -> *mut c_void;
    fn objc_msgSend();
    fn objc_autoreleasePoolPush() -> *mut c_void;
    fn objc_autoreleasePoolPop(pool: *mut c_void);
    static _NSConcreteGlobalBlock: c_void;
}

type MsgSend0 = unsafe extern "C" fn(*mut c_void, *mut c_void) -> *mut c_void;
type MsgSend1 = unsafe extern "C" fn(*mut c_void, *mut c_void, *mut c_void) -> *mut c_void;
type MsgSend2 =
    unsafe extern "C" fn(*mut c_void, *mut c_void, *mut c_void, *mut c_void) -> *mut c_void;
type MsgSend3 = unsafe extern "C" fn(
    *mut c_void,
    *mut c_void,
    *mut c_void,
    *mut c_void,
    *mut c_void,
) -> *mut c_void;
type MsgSend4 = unsafe extern "C" fn(
    *mut c_void,
    *mut c_void,
    *mut c_void,
    *mut c_void,
    *mut c_void,
    *mut c_void,
) -> *mut c_void;

enum NativeNotificationAction<'a> {
    CheckBundle,
    RequestAuthorization,
    Send { title: &'a str, body: &'a str },
}

unsafe fn to_nsstring(s: &str) -> *mut c_void {
    let cls_nsstring = objc_getClass(c"NSString".as_ptr());
    let sel_string_with_utf8 = sel_registerName(c"stringWithUTF8String:".as_ptr());
    let c_str = match CString::new(s) {
        Ok(c) => c,
        Err(_) => {
            let sanitized: String = s.chars().filter(|&c| c != '\0').collect();
            CString::new(sanitized).unwrap_or_default()
        }
    };
    let msg_send_1: MsgSend1 = std::mem::transmute(objc_msgSend as *const ());
    msg_send_1(
        cls_nsstring,
        sel_string_with_utf8,
        c_str.as_ptr() as *mut c_void,
    )
}

unsafe fn dispatch_macos_notification_native_impl(
    action: NativeNotificationAction<'_>,
) -> Result<bool> {
    let pool = objc_autoreleasePoolPush();
    let result = (|| -> Result<bool> {
        let cls_bundle = objc_getClass(c"NSBundle".as_ptr());
        if cls_bundle.is_null() {
            return Ok(false);
        }
        let sel_main_bundle = sel_registerName(c"mainBundle".as_ptr());
        let msg_send_0: MsgSend0 = std::mem::transmute(objc_msgSend as *const ());
        let main_bundle = msg_send_0(cls_bundle, sel_main_bundle);
        if main_bundle.is_null() {
            return Ok(false);
        }
        let sel_bundle_id = sel_registerName(c"bundleIdentifier".as_ptr());
        let bundle_id = msg_send_0(main_bundle, sel_bundle_id);
        if bundle_id.is_null() {
            return Ok(false);
        }
        if matches!(action, NativeNotificationAction::CheckBundle) {
            return Ok(true);
        }

        libc::dlopen(
            c"/System/Library/Frameworks/UserNotifications.framework/UserNotifications".as_ptr(),
            libc::RTLD_LAZY,
        );

        let cls_center = objc_getClass(c"UNUserNotificationCenter".as_ptr());
        if cls_center.is_null() {
            bail!("UNUserNotificationCenter class not available");
        }
        let sel_current_center = sel_registerName(c"currentNotificationCenter".as_ptr());
        let center = msg_send_0(cls_center, sel_current_center);
        if center.is_null() {
            bail!("currentNotificationCenter returned nil");
        }

        let sel_request_auth =
            sel_registerName(c"requestAuthorizationWithOptions:completionHandler:".as_ptr());
        let msg_send_2: MsgSend2 = std::mem::transmute(objc_msgSend as *const ());

        let auth_block = Box::leak(Box::new(BlockLiteral {
            isa: &raw const _NSConcreteGlobalBlock,
            flags: (1 << 28) | (1 << 29),
            reserved: 0,
            invoke: auth_completion_invoke,
            descriptor: &BLOCK_DESCRIPTOR,
        }));
        let options: usize = 7; // Alert | Sound | Badge
        msg_send_2(
            center,
            sel_request_auth,
            options as *mut c_void,
            auth_block as *mut _ as *mut c_void,
        );

        match action {
            NativeNotificationAction::CheckBundle => unreachable!(),
            NativeNotificationAction::RequestAuthorization => Ok(true),
            NativeNotificationAction::Send { title, body } => {
                let cls_content = objc_getClass(c"UNMutableNotificationContent".as_ptr());
                if cls_content.is_null() {
                    bail!("UNMutableNotificationContent class not available");
                }
                let sel_alloc = sel_registerName(c"alloc".as_ptr());
                let sel_init = sel_registerName(c"init".as_ptr());
                let content_alloc = msg_send_0(cls_content, sel_alloc);
                let content = msg_send_0(content_alloc, sel_init);
                if content.is_null() {
                    bail!("Failed to allocate UNMutableNotificationContent");
                }
                let sel_autorelease = sel_registerName(c"autorelease".as_ptr());
                msg_send_0(content, sel_autorelease);

                let sel_set_title = sel_registerName(c"setTitle:".as_ptr());
                let msg_send_1: MsgSend1 = std::mem::transmute(objc_msgSend as *const ());
                msg_send_1(content, sel_set_title, to_nsstring(title));

                let sel_set_body = sel_registerName(c"setBody:".as_ptr());
                msg_send_1(content, sel_set_body, to_nsstring(body));

                let cls_sound = objc_getClass(c"UNNotificationSound".as_ptr());
                if !cls_sound.is_null() {
                    let sel_default_sound = sel_registerName(c"defaultSound".as_ptr());
                    let sound = msg_send_0(cls_sound, sel_default_sound);
                    if !sound.is_null() {
                        let sel_set_sound = sel_registerName(c"setSound:".as_ptr());
                        msg_send_1(content, sel_set_sound, sound);
                    }
                }

                let sel_path_for_res = sel_registerName(c"pathForResource:ofType:".as_ptr());
                let name_str = to_nsstring(concat!("operon", "-icon-1024"));
                let type_str = to_nsstring("png");
                let icon_path = msg_send_2(main_bundle, sel_path_for_res, name_str, type_str);
                if !icon_path.is_null() {
                    let cls_url = objc_getClass(c"NSURL".as_ptr());
                    let sel_file_url = sel_registerName(c"fileURLWithPath:".as_ptr());
                    let file_url = msg_send_1(cls_url, sel_file_url, icon_path);
                    if !file_url.is_null() {
                        let cls_attachment = objc_getClass(c"UNNotificationAttachment".as_ptr());
                        if !cls_attachment.is_null() {
                            let sel_attach = sel_registerName(
                                c"attachmentWithIdentifier:URL:options:error:".as_ptr(),
                            );
                            let attach_id = to_nsstring("notification-icon");
                            let msg_send_4: MsgSend4 =
                                std::mem::transmute(objc_msgSend as *const ());
                            let attachment = msg_send_4(
                                cls_attachment,
                                sel_attach,
                                attach_id,
                                file_url,
                                std::ptr::null_mut(),
                                std::ptr::null_mut(),
                            );
                            if !attachment.is_null() {
                                let cls_array = objc_getClass(c"NSArray".as_ptr());
                                let sel_array_with_obj =
                                    sel_registerName(c"arrayWithObject:".as_ptr());
                                let attachments_array =
                                    msg_send_1(cls_array, sel_array_with_obj, attachment);
                                if !attachments_array.is_null() {
                                    let sel_set_attachments =
                                        sel_registerName(c"setAttachments:".as_ptr());
                                    msg_send_1(content, sel_set_attachments, attachments_array);
                                }
                            }
                        }
                    }
                }

                let req_id_str = Uuid::new_v4().to_string();
                let req_id = to_nsstring(&req_id_str);
                let cls_req = objc_getClass(c"UNNotificationRequest".as_ptr());
                if cls_req.is_null() {
                    bail!("UNNotificationRequest class not available");
                }
                let sel_req_with_id =
                    sel_registerName(c"requestWithIdentifier:content:trigger:".as_ptr());
                let msg_send_3: MsgSend3 = std::mem::transmute(objc_msgSend as *const ());
                let request = msg_send_3(
                    cls_req,
                    sel_req_with_id,
                    req_id,
                    content,
                    std::ptr::null_mut(),
                );
                if request.is_null() {
                    bail!("Failed to allocate UNNotificationRequest");
                }

                let sel_add_req =
                    sel_registerName(c"addNotificationRequest:withCompletionHandler:".as_ptr());
                msg_send_2(center, sel_add_req, request, std::ptr::null_mut());
                Ok(true)
            }
        }
    })();
    objc_autoreleasePoolPop(pool);
    result
}

fn dispatch_macos_notification_native(action: NativeNotificationAction<'_>) -> Result<bool> {
    // SAFETY: Interacts with macOS UserNotifications Objective-C runtime via FFI within an autorelease pool.
    unsafe { dispatch_macos_notification_native_impl(action) }
}

pub(crate) fn is_bundled_app() -> bool {
    dispatch_macos_notification_native(NativeNotificationAction::CheckBundle).unwrap_or(false)
}

pub(crate) fn request_notification_authorization() {
    if is_bundled_app() {
        let _ = dispatch_macos_notification_native(NativeNotificationAction::RequestAuthorization);
    }
}

pub(crate) fn send_macos_notification(title: &str, body: &str) -> Result<()> {
    if is_bundled_app() {
        if let Ok(true) =
            dispatch_macos_notification_native(NativeNotificationAction::Send { title, body })
        {
            return Ok(());
        }
    }
    send_macos_notification_osascript(title, body)
}

pub(crate) fn send_macos_notification_osascript(title: &str, body: &str) -> Result<()> {
    let title = escape_applescript_string(title);
    let body = escape_applescript_string(body);
    let script = format!("display notification \"{body}\" with title \"{title}\"");
    let mut command = Command::new("osascript");
    command.args(["-e", &script]);
    run_system_command(&mut command).context(tr("macOS 通知の送信"))
}
pub(crate) fn run_system_command(command: &mut Command) -> Result<()> {
    let output = run_command_with_timeout(command, Duration::from_secs(10))?;
    if output.status.success() {
        Ok(())
    } else {
        Err(anyhow!(String::from_utf8_lossy(&output.stderr)
            .trim()
            .to_owned()))
    }
}
pub(crate) fn escape_applescript_string(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace(['\n', '\r'], " ")
}
pub(crate) fn command_output_in_dir<const N: usize>(
    program: &str,
    args: [&str; N],
    cwd: &Path,
) -> Result<String> {
    let mut command = Command::new(program);
    command.args(args).current_dir(cwd);
    let output = run_command_with_timeout(&mut command, Duration::from_secs(20))?;
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(anyhow!(String::from_utf8_lossy(&output.stderr).to_string()))
    }
}
pub(crate) fn git_output(path: &Path, arguments: &[&str]) -> Result<String> {
    let mut command = crate::git::git_command(path);
    command.args(arguments);
    let output = run_command_with_timeout(&mut command, Duration::from_secs(20))?;
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(anyhow!(String::from_utf8_lossy(&output.stderr).to_string()))
    }
}

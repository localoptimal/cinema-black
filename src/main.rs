//! Thin, dependency-free AppKit bridge. All UI calls stay on the main thread.
#![cfg_attr(not(target_os = "macos"), allow(dead_code))]
#[cfg(not(all(target_os = "macos", target_arch = "aarch64")))]
compile_error!("This release targets Apple Silicon macOS. The Objective-C struct ABI must be adapted before building for Intel.");

use std::ffi::{c_char, c_void, CString};
type Id = *mut c_void;
type Sel = *mut c_void;
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
struct Point {
    x: f64,
    y: f64,
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
struct Size {
    width: f64,
    height: f64,
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
struct Rect {
    origin: Point,
    size: Size,
}

#[link(name = "objc")]
extern "C" {
    fn objc_getClass(name: *const c_char) -> Id;
    fn sel_registerName(name: *const c_char) -> Sel;
    fn objc_msgSend();
    fn objc_allocateClassPair(superclass: Id, name: *const c_char, extra: usize) -> Id;
    fn objc_registerClassPair(class: Id);
    fn class_addMethod(class: Id, selector: Sel, imp: *const c_void, types: *const c_char) -> bool;
}
#[link(name = "AppKit", kind = "framework")]
extern "C" {}
#[link(name = "QuartzCore", kind = "framework")]
extern "C" {}
#[link(name = "CoreGraphics", kind = "framework")]
extern "C" {
    fn CGColorGetNumberOfComponents(color: Id) -> usize;
    fn CGColorGetComponents(color: Id) -> *const f64;
}

// Typed sends are required: varargs would corrupt floating-point/struct arguments.
unsafe fn sel(s: &str) -> Sel {
    sel_registerName(CString::new(s).unwrap().as_ptr())
}
unsafe fn class(s: &str) -> Id {
    objc_getClass(CString::new(s).unwrap().as_ptr())
}
macro_rules! send {
    ($obj:expr, $name:expr, $ret:ty $(, $ty:ty => $arg:expr)*) => {{
        let f: unsafe extern "C" fn(Id, Sel $(, $ty)*) -> $ret =
            std::mem::transmute(objc_msgSend as *const ());
        f($obj, sel($name) $(, $arg)*)
    }};
}
unsafe fn string(s: &str) -> Id {
    send!(class("NSString"), "stringWithUTF8String:", Id, *const c_char => CString::new(s).unwrap().as_ptr())
}
unsafe fn rust_string(s: Id) -> String {
    let p = send!(s, "UTF8String", *const c_char);
    if p.is_null() {
        String::new()
    } else {
        std::ffi::CStr::from_ptr(p).to_string_lossy().into_owned()
    }
}
static mut APP: Id = std::ptr::null_mut();
static mut WINDOW: Id = std::ptr::null_mut();
static mut SCREENS: Id = std::ptr::null_mut();

#[derive(Debug, Default, PartialEq)]
struct Options {
    screen: Option<usize>,
    fullscreen: bool,
    list: bool,
    smoke: bool,
    help: bool,
}
fn parse(args: impl Iterator<Item = String>) -> Result<Options, String> {
    let mut opts = Options::default();
    let mut args = args;
    while let Some(a) = args.next() {
        match a.as_str() {
            "--screen" => {
                let n: usize = args
                    .next()
                    .ok_or("--screen requires a 1-based number")?
                    .parse()
                    .map_err(|_| "Invalid screen number")?;
                if n == 0 {
                    return Err("Screen numbers start at 1".into());
                }
                opts.screen = Some(n - 1);
            }
            "--fullscreen" => opts.fullscreen = true,
            "--list-screens" => opts.list = true,
            "--smoke-test" => opts.smoke = true,
            "--help" | "-h" => opts.help = true,
            s if s.starts_with("-psn_") => {} // Finder compatibility.
            _ => return Err(format!("Unknown argument: {a}")),
        }
    }
    Ok(opts)
}
fn centered(frame: Rect) -> Rect {
    let width = 800.0_f64.min(frame.size.width * 0.8);
    let height = 450.0_f64.min(frame.size.height * 0.8);
    Rect {
        origin: Point {
            x: frame.origin.x + (frame.size.width - width) / 2.0,
            y: frame.origin.y + (frame.size.height - height) / 2.0,
        },
        size: Size { width, height },
    }
}
unsafe extern "C" fn quit(_: Id, _: Sel, _: Id) {
    send!(APP, "terminate:", (), Id => APP);
}
unsafe extern "C" fn close(_: Id, _: Sel, _: Id) -> bool {
    send!(APP, "terminate:", (), Id => APP);
    true
}
unsafe extern "C" fn keys(this: Id, _: Sel, event: Id) {
    if send!(event, "keyCode", u16) == 53 {
        quit(this, std::ptr::null_mut(), event);
    }
}
unsafe extern "C" fn click(this: Id, _: Sel, event: Id) {
    if send!(event, "clickCount", isize) == 2 {
        send!(this, "toggleFullScreen:", (), Id => this);
    }
}
unsafe extern "C" fn zoom(this: Id, _: Sel, _: Id) {
    send!(this, "toggleFullScreen:", (), Id => this);
}
unsafe extern "C" fn toggle(_: Id, _: Sel, _: Id) {
    send!(WINDOW, "toggleFullScreen:", (), Id => WINDOW);
}
unsafe extern "C" fn verify(_: Id, _: Sel, _: Id) {
    let frame = send!(WINDOW, "frame", Rect);
    let screen = send!(WINDOW, "screen", Id);
    let bounds = send!(screen, "frame", Rect);
    let view = send!(WINDOW, "contentView", Id);
    let color = send!(send!(view, "layer", Id), "backgroundColor", Id);
    let n = CGColorGetNumberOfComponents(color);
    let values = std::slice::from_raw_parts(CGColorGetComponents(color), n);
    let black = n >= 2 && values[..n - 1].iter().all(|v| *v == 0.0) && values[n - 1] == 1.0;
    let covers = (frame.origin.x - bounds.origin.x).abs() < 0.5
        && (frame.origin.y - bounds.origin.y).abs() < 0.5
        && (frame.size.width - bounds.size.width).abs() < 0.5
        && (frame.size.height - bounds.size.height).abs() < 0.5;
    println!("native-check: display={}, fullscreen={}, covers-display={}, black={}, opaque={}, frame={:?}",
        rust_string(send!(screen, "localizedName", Id)), send!(WINDOW, "styleMask", usize) & (1 << 14) != 0,
        covers, black, send!(WINDOW, "isOpaque", bool), frame);
}
unsafe extern "C" fn choose(_: Id, _: Sel, item: Id) {
    // A screen selection is a move, not a global display setting change.
    if send!(WINDOW, "styleMask", usize) & (1 << 14) != 0 {
        return;
    }
    let index = send!(item, "tag", isize) as usize;
    let count = send!(SCREENS, "count", usize);
    if index >= count {
        return;
    }
    let screen = send!(SCREENS, "objectAtIndex:", Id, usize => index);
    let frame = send!(screen, "visibleFrame", Rect);
    send!(WINDOW, "setFrame:display:", (), Rect => centered(frame), bool => true);
    send!(WINDOW, "makeKeyAndOrderFront:", (), Id => WINDOW);
}
unsafe fn method(c: Id, name: &str, imp: *const c_void, encoding: &str) {
    assert!(class_addMethod(
        c,
        sel(name),
        imp,
        CString::new(encoding).unwrap().as_ptr()
    ));
}
unsafe fn menu_item(menu: Id, title: &str, action: &str, key: &str, target: Id) -> Id {
    let item = send!(send!(class("NSMenuItem"), "alloc", Id), "initWithTitle:action:keyEquivalent:", Id,
        Id => string(title), Sel => sel(action), Id => string(key));
    send!(item, "setTarget:", (), Id => target);
    send!(menu, "addItem:", (), Id => item);
    item
}
unsafe fn submenu(bar: Id, name: &str) -> Id {
    let root = send!(send!(class("NSMenuItem"), "alloc", Id), "initWithTitle:action:keyEquivalent:", Id,
        Id => string(name), Sel => std::ptr::null_mut(), Id => string(""));
    let sub = send!(send!(class("NSMenu"), "alloc", Id), "initWithTitle:", Id, Id => string(name));
    send!(root, "setSubmenu:", (), Id => sub);
    send!(bar, "addItem:", (), Id => root);
    sub
}
fn main() {
    let opts = match parse(std::env::args().skip(1)) {
        Ok(o) => o,
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(2);
        }
    };
    if opts.help {
        println!("Cinema Black 1.0.0 — Apple Silicon macOS\n\n--list-screens\n--screen N (1-based)\n--fullscreen\n--smoke-test (auto exits after 2 seconds)\n\nDrag to your chosen display; green button, double-click or Cmd+F blackens it.\nEsc / Cmd+Q exits. Displays menu moves the window before fullscreen.");
        return;
    }
    unsafe {
        let pool = send!(send!(class("NSAutoreleasePool"), "alloc", Id), "init", Id);
        APP = send!(class("NSApplication"), "sharedApplication", Id);
        SCREENS = send!(class("NSScreen"), "screens", Id);
        let count = send!(SCREENS, "count", usize);
        if count == 0 {
            eprintln!("No connected screens.");
            std::process::exit(1);
        }
        if opts.list {
            for i in 0..count {
                let s = send!(SCREENS, "objectAtIndex:", Id, usize => i);
                let f = send!(s, "frame", Rect);
                let scale = send!(s, "backingScaleFactor", f64);
                println!(
                    "{}: {} — {:.0}×{:.0} points, scale {}",
                    i + 1,
                    rust_string(send!(s, "localizedName", Id)),
                    f.size.width,
                    f.size.height,
                    scale
                );
            }
            send!(pool, "drain", ());
            return;
        }
        let index = opts.screen.unwrap_or(0);
        if index >= count {
            eprintln!("Screen {} unavailable; {} connected.", index + 1, count);
            std::process::exit(2);
        }
        let screen = send!(SCREENS, "objectAtIndex:", Id, usize => index);
        let frame = send!(screen, "visibleFrame", Rect);
        let cls = objc_allocateClassPair(
            class("NSWindow"),
            CString::new("CinemaBlackWindow").unwrap().as_ptr(),
            0,
        );
        method(cls, "keyDown:", keys as *const c_void, "v@:@");
        method(cls, "mouseDown:", click as *const c_void, "v@:@");
        method(cls, "zoom:", zoom as *const c_void, "v@:@");
        objc_registerClassPair(cls);
        let dc = objc_allocateClassPair(
            class("NSObject"),
            CString::new("CinemaBlackDelegate").unwrap().as_ptr(),
            0,
        );
        method(dc, "quit:", quit as *const c_void, "v@:@");
        method(dc, "windowShouldClose:", close as *const c_void, "B@:@");
        method(dc, "toggle:", toggle as *const c_void, "v@:@");
        method(dc, "choose:", choose as *const c_void, "v@:@");
        method(dc, "verify:", verify as *const c_void, "v@:@");
        objc_registerClassPair(dc);
        let delegate = send!(send!(dc, "alloc", Id), "init", Id);
        WINDOW = send!(send!(cls, "alloc", Id), "initWithContentRect:styleMask:backing:defer:screen:", Id,
            Rect => centered(frame), usize => 1|2|4|8, usize => 2, bool => false, Id => screen);
        send!(WINDOW, "setTitle:", (), Id => string("Cinema Black · 拖到目标屏幕，绿色按钮 / ⌘F 全屏 · Esc 退出"));
        send!(WINDOW, "setBackgroundColor:", (), Id => send!(class("NSColor"), "blackColor", Id));
        send!(WINDOW, "setOpaque:", (), bool => true);
        send!(WINDOW, "setHasShadow:", (), bool => false);
        send!(WINDOW, "setReleasedWhenClosed:", (), bool => false);
        send!(WINDOW, "setCollectionBehavior:", (), usize => 1 << 7);
        send!(WINDOW, "setDelegate:", (), Id => delegate);
        let view = send!(WINDOW, "contentView", Id);
        send!(view, "setWantsLayer:", (), bool => true);
        let layer = send!(view, "layer", Id);
        let color = send!(class("NSColor"), "blackColor", Id);
        send!(layer, "setBackgroundColor:", (), Id => send!(color, "CGColor", Id));
        let bar = send!(send!(class("NSMenu"), "alloc", Id), "init", Id);
        let app_menu = submenu(bar, "Cinema Black");
        menu_item(app_menu, "全屏黑屏 / 恢复窗口", "toggle:", "f", delegate);
        menu_item(app_menu, "退出 Cinema Black", "quit:", "q", delegate);
        let displays = submenu(bar, "选择屏幕 Displays");
        for i in 0..count {
            let s = send!(SCREENS, "objectAtIndex:", Id, usize => i);
            let item = menu_item(
                displays,
                &format!("{} · {}", i + 1, rust_string(send!(s, "localizedName", Id))),
                "choose:",
                "",
                delegate,
            );
            send!(item, "setTag:", (), isize => i as isize);
        }
        send!(APP, "setMainMenu:", (), Id => bar);
        send!(APP, "setActivationPolicy:", bool, isize => if opts.smoke { 1 } else { 0 });
        send!(APP, "finishLaunching", ());
        if opts.smoke {
            // Test native construction without interrupting the user's foreground app.
            send!(WINDOW, "orderBack:", (), Id => WINDOW);
            println!(
                "native-window: screen={}, opaque={}, black-layer=true, frame={:?}",
                index + 1,
                send!(WINDOW, "isOpaque", bool),
                send!(WINDOW, "frame", Rect)
            );
            send!(delegate, "performSelector:withObject:afterDelay:", (), Sel => sel("verify:"), Id => APP, f64 => 1.5);
            send!(delegate, "performSelector:withObject:afterDelay:", (), Sel => sel("quit:"), Id => APP, f64 => 2.0);
        } else {
            send!(WINDOW, "makeKeyAndOrderFront:", (), Id => WINDOW);
            send!(APP, "activateIgnoringOtherApps:", (), bool => true);
        }
        if opts.fullscreen {
            send!(delegate, "performSelector:withObject:afterDelay:", (), Sel => sel("toggle:"), Id => WINDOW, f64 => 0.25);
        }
        send!(APP, "run", ());
        send!(pool, "drain", ());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn args<'a>(s: &'a [&'a str]) -> impl Iterator<Item = String> + 'a {
        s.iter().map(|s| s.to_string())
    }
    #[test]
    fn defaults_are_windowed() {
        assert_eq!(parse(args(&[])).unwrap(), Options::default());
    }
    #[test]
    fn screen_is_one_based() {
        assert_eq!(parse(args(&["--screen", "2"])).unwrap().screen, Some(1));
    }
    #[test]
    fn bad_screen_rejected() {
        for a in [&["--screen", "0"][..], &["--screen", "x"], &["--screen"]] {
            assert!(parse(args(a)).is_err());
        }
    }
    #[test]
    fn unknown_rejected() {
        assert!(parse(args(&["--nonsense"])).is_err());
    }
    #[test]
    fn flags() {
        let o = parse(args(&["--fullscreen", "--smoke-test"])).unwrap();
        assert!(o.fullscreen && o.smoke);
    }
    #[test]
    fn centering_respects_negative_display_origin() {
        let r = centered(Rect {
            origin: Point { x: -1920.0, y: 0.0 },
            size: Size {
                width: 1920.0,
                height: 1080.0,
            },
        });
        assert_eq!(
            r.origin,
            Point {
                x: -1360.0,
                y: 315.0
            }
        );
    }
    #[test]
    fn small_screens_fit() {
        let r = centered(Rect {
            origin: Point::default(),
            size: Size {
                width: 400.0,
                height: 200.0,
            },
        });
        assert_eq!(
            r.size,
            Size {
                width: 320.0,
                height: 160.0
            }
        );
    }
}

/******************************************************************************************************************
 * Source: https://raw.githubusercontent.com/bekker/msgbox-rs/cff1e50e6e2de971a6995d9ce062d5e394a45d45/src/macos.rs
 * License: Distributed under MIT License
 * Author: Jang Ryeol (https://github.com/bekker)
 ******************************************************************************************************************/

use cocoa::base::id;
use cocoa::foundation::NSString;
use objc::*;


/**
 * cocoa-rs doesn't implement NSAlert yet (0.14.0)
 * Then implement it!
 * Someone would stub and implement all methods for NSAlert, and make it to the upstream?
 */

/**
 * NSAlert.Style
 * https://developer.apple.com/documentation/appkit/nsalert.style
 */
#[repr(u64)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum NSAlertStyle {
    warning         = 0, // Same visual as informational
    informational   = 1,
    critical        = 2
}

/**
 * NSAlert
 * https://developer.apple.com/documentation/appkit/nsalert
 */
pub trait NSAlert: Sized {
    unsafe fn alloc(_: Self) -> id {
        msg_send![class!(NSAlert), alloc]
    }

    unsafe fn init(self) -> id;
    unsafe fn autorelease(self) -> id;

    unsafe fn setAlertStyle(self, style: NSAlertStyle);
    unsafe fn setMessageText(self, messageText: id);
    unsafe fn setInformativeText(self, informativeText: id);
    unsafe fn addButton(self, withTitle: id);
    unsafe fn runModal(self) -> id;
}

impl NSAlert for id {
    unsafe fn init(self) -> id {
        msg_send![self, init]
    }

    unsafe fn autorelease(self) -> id {
        msg_send![self, autorelease]
    }

    unsafe fn setAlertStyle(self, alertStyle: NSAlertStyle) {
        msg_send![self, setAlertStyle: alertStyle]
    }

    unsafe fn setMessageText(self, messageText: id) {
        msg_send![self, setMessageText: messageText]
    }

    unsafe fn setInformativeText(self, informativeText: id) {
        msg_send![self, setInformativeText: informativeText]
    }

    unsafe fn addButton(self, withTitle: id) {
        msg_send![self, addButtonWithTitle: withTitle]
    }

    unsafe fn runModal(self) -> id {
        msg_send![self, runModal]
    }
}
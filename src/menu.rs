//! Lboot's grub2-like selection menu
extern crate alloc;

use core::{ffi::c_void, ptr::NonNull};

use crate::config::Entry;
use alloc::vec::Vec;
use uefi::{
    proto::console::text::{Key, ScanCode},
    table::{
        boot::{EventType, TimerTrigger, Tpl},
        Boot, SystemTable,
    },
    Event,
};
use uefi_services::println;

fn show_menu(entries: &[Entry], selected: usize) {
    for (i, entry) in entries.iter().enumerate() {
        if i == selected {
            println!("> {}", entry);
        } else {
            println!("  {}", entry);
        }
    }
}

const WAIT_TIME: u64 = 30_000_000; // * 100ns

/// A selection menu similar to grub2.
/// 
/// Use up and down or jk to select, and press enter or right or l
/// to start the entry.
pub fn select_in_menu<'a>(st: &mut SystemTable<Boot>, entries: &'a Vec<Entry>) -> &'a Entry<'a> {
    let mut selected: usize = 0;
    let mut last_selected: usize = 0;
    let mut time_out = false;
    let event_type = EventType::union(EventType::TIMER, EventType::NOTIFY_SIGNAL);
    let ctx: *mut bool = &mut time_out;
    let ctx = NonNull::new(ctx.cast::<c_void>()).unwrap();
    extern "efiapi" fn callback(_event: Event, ctx: Option<NonNull<c_void>>) {
        unsafe {
            let time_out = ctx.unwrap().as_ptr().cast::<bool>();
            *time_out = true;
        }
    }
    let event = unsafe {
        st.boot_services()
            .create_event(event_type, Tpl::CALLBACK, Some(callback), Some(ctx))
    }
    .unwrap();
    st.boot_services()
        .set_timer(&event, TimerTrigger::Relative(WAIT_TIME))
        .unwrap();
    let len = entries.len();
    st.stdout().clear().unwrap();
    show_menu(entries, selected);
    loop {
        if time_out {
            st.stdout().clear().unwrap();
            return &entries[selected];
        }
        if selected != last_selected {
            st.stdout().clear().unwrap();
            show_menu(entries, selected);
            last_selected = selected;
        }
        if let Ok(Some(key)) = st.stdin().read_key() {
            match key {
                Key::Special(special) => match special {
                    ScanCode::UP => {
                        selected += len - 1;
                    }
                    ScanCode::DOWN => {
                        selected += 1;
                    }
                    ScanCode::RIGHT => {
                        st.stdout().clear().unwrap();
                        return &entries[selected];
                    }
                    _ => (),
                },
                Key::Printable(c) => {
                    let ch = unsafe { char::from_u32_unchecked(u16::from(c) as u32) };
                    match ch {
                        'k' => {
                            selected += len - 1;
                        }
                        'j' => {
                            selected += 1;
                        }
                        '\r' | '\n' | 'l' => {
                            st.stdout().clear().unwrap();
                            return &entries[selected];
                        }
                        _ => (),
                    }
                }
            }
        }
        selected %= len;
    }
}

//! Isolated measurement fixture. This does not modify the student starter.
#![allow(dead_code)]
use std::mem::{align_of, offset_of, size_of};
use std::num::NonZeroU8;

#[repr(C)]
struct OriginalHeader { flag: u8, timestamp: u64, kind: u16, payload_len: u32 }
#[repr(C)]
struct ReorderedHeader { timestamp: u64, payload_len: u32, kind: u16, flag: u8 }
enum OriginalMessage { Ping, Ack(u8), Data([u8; 64]), Error(String) }
enum BoxedMessage { Ping, Ack(u8), Data(Box<[u8; 64]>), Error(String) }
#[repr(transparent)]
struct PayloadLen(u32);
#[repr(u8)]
enum LinkStatus { Idle, Connecting, Connected, Closing }

fn main() {
    println!("OriginalHeader size={} align={}", size_of::<OriginalHeader>(), align_of::<OriginalHeader>());
    println!("OriginalHeader offsets flag={} timestamp={} kind={} payload_len={}", offset_of!(OriginalHeader,flag), offset_of!(OriginalHeader,timestamp), offset_of!(OriginalHeader,kind), offset_of!(OriginalHeader,payload_len));
    println!("ReorderedHeader size={} align={}", size_of::<ReorderedHeader>(), align_of::<ReorderedHeader>());
    println!("ReorderedHeader offsets timestamp={} payload_len={} kind={} flag={}", offset_of!(ReorderedHeader,timestamp), offset_of!(ReorderedHeader,payload_len), offset_of!(ReorderedHeader,kind), offset_of!(ReorderedHeader,flag));
    println!("OriginalMessage={} BoxedMessage={}", size_of::<OriginalMessage>(), size_of::<BoxedMessage>());
    println!("String={} BoxArray={} Array={}", size_of::<String>(), size_of::<Box<[u8;64]>>(), size_of::<[u8;64]>());
    println!("PayloadLen={} u32={}", size_of::<PayloadLen>(), size_of::<u32>());
    println!("Box<u32>={} Option<Box<u32>>={}", size_of::<Box<u32>>(), size_of::<Option<Box<u32>>>());
    println!("NonZeroU8={} Option<NonZeroU8>={} Option<u8>={}", size_of::<NonZeroU8>(), size_of::<Option<NonZeroU8>>(), size_of::<Option<u8>>());
    println!("LinkStatus={} Option<LinkStatus>={}", size_of::<LinkStatus>(), size_of::<Option<LinkStatus>>());
    assert_eq!(size_of::<OriginalHeader>(), 24);
    assert_eq!(size_of::<ReorderedHeader>(), 16);
    assert_eq!(size_of::<OriginalMessage>(), 72);
    assert_eq!(size_of::<BoxedMessage>(), 24);
    assert_eq!(size_of::<PayloadLen>(), size_of::<u32>());
    assert_eq!(size_of::<Option<NonZeroU8>>(), 1);
    assert_eq!(size_of::<Option<Box<u32>>>(), size_of::<Box<u32>>());
}

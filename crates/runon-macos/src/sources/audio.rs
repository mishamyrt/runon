use crate::native::Source;
use block2::RcBlock;
use dispatch2::DispatchQueue;
use objc2_core_audio::{
    AudioObjectAddPropertyListenerBlock, AudioObjectGetPropertyData,
    AudioObjectGetPropertyDataSize, AudioObjectPropertyAddress,
    AudioObjectRemovePropertyListenerBlock, kAudioDevicePropertyDeviceUID,
    kAudioHardwarePropertyDevices, kAudioObjectPropertyElementMain, kAudioObjectPropertyName,
    kAudioObjectPropertyScopeGlobal, kAudioObjectSystemObject,
};
use objc2_core_foundation::{CFRetained, CFString};
use runon_core::event::{Event, Kind};
use std::{collections::BTreeMap, ptr::NonNull, sync::Arc};

type AudioBlock = RcBlock<dyn Fn(u32, NonNull<AudioObjectPropertyAddress>)>;

pub(super) struct Listener {
    block: AudioBlock,
}
impl Drop for Listener {
    fn drop(&mut self) {
        let mut address = address(kAudioHardwarePropertyDevices);
        let code = unsafe {
            AudioObjectRemovePropertyListenerBlock(
                kAudioObjectSystemObject as u32,
                NonNull::from(&mut address),
                Some(DispatchQueue::main()),
                RcBlock::as_ptr(&self.block),
            )
        };
        if code != 0 {
            log::error!("removing audio listener: OSStatus {code}");
        }
    }
}

impl Listener {
    pub(super) fn new(refresh: Arc<Source>) -> Result<Self, String> {
        let block: AudioBlock =
            RcBlock::new(move |_: u32, _: NonNull<AudioObjectPropertyAddress>| {
                refresh.wake();
            });
        let mut address = address(kAudioHardwarePropertyDevices);
        let code = unsafe {
            AudioObjectAddPropertyListenerBlock(
                kAudioObjectSystemObject as u32,
                NonNull::from(&mut address),
                Some(DispatchQueue::main()),
                RcBlock::as_ptr(&block),
            )
        };
        if code != 0 {
            return Err(format!("subscribing to audio devices: OSStatus {code}"));
        }
        Ok(Self { block })
    }
}

fn address(selector: u32) -> AudioObjectPropertyAddress {
    AudioObjectPropertyAddress {
        mSelector: selector,
        mScope: kAudioObjectPropertyScopeGlobal,
        mElement: kAudioObjectPropertyElementMain,
    }
}

pub(super) fn snapshot() -> Result<BTreeMap<String, Event>, String> {
    let mut addr = address(kAudioHardwarePropertyDevices);
    // Device lists can change between the size query and data query; bounded retry.
    for _ in 0..3 {
        let mut size = 0u32;
        let code = unsafe {
            AudioObjectGetPropertyDataSize(
                kAudioObjectSystemObject as u32,
                NonNull::from(&mut addr),
                0,
                std::ptr::null(),
                NonNull::from(&mut size),
            )
        };
        if code != 0 {
            return Err(format!("device list size: OSStatus {code}"));
        }
        if size == 0 {
            return Ok(BTreeMap::new());
        }
        if !size.is_multiple_of(4) {
            return Err("invalid audio device list size".into());
        }
        let mut ids = vec![0u32; size as usize / 4];
        let code = unsafe {
            AudioObjectGetPropertyData(
                kAudioObjectSystemObject as u32,
                NonNull::from(&mut addr),
                0,
                std::ptr::null(),
                NonNull::from(&mut size),
                NonNull::new(ids.as_mut_ptr().cast()).unwrap(),
            )
        };
        if code != 0 {
            continue;
        }
        ids.truncate(size as usize / 4);
        let mut result = BTreeMap::new();
        let mut complete = true;
        for id in ids {
            match (
                audio_string(id, kAudioDevicePropertyDeviceUID),
                audio_string(id, kAudioObjectPropertyName),
            ) {
                (Ok(uid), Ok(name)) => {
                    result.insert(
                        uid.clone(),
                        Event::new(Kind::AudioConnected)
                            .text("uid", uid)
                            .text("name", name),
                    );
                }
                _ => {
                    complete = false;
                    break;
                }
            }
        }
        if complete {
            return Ok(result);
        }
    }
    Err("audio devices changed during enumeration; keeping previous snapshot".into())
}

fn audio_string(id: u32, selector: u32) -> Result<String, String> {
    let mut addr = address(selector);
    let mut value: *mut CFString = std::ptr::null_mut();
    let mut size = u32::try_from(std::mem::size_of_val(&value)).unwrap();
    let code = unsafe {
        AudioObjectGetPropertyData(
            id,
            NonNull::from(&mut addr),
            0,
            std::ptr::null(),
            NonNull::from(&mut size),
            NonNull::from(&mut value).cast(),
        )
    };
    if code != 0 {
        return Err(format!("device {id} property {selector}: OSStatus {code}"));
    }
    let ptr = NonNull::new(value).ok_or("audio property is null")?;
    // CoreAudio transfers ownership of CF-valued properties to the caller.
    Ok(unsafe { CFRetained::from_raw(ptr) }.to_string())
}

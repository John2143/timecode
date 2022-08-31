use wasm_bindgen::prelude::*;

use crate::{Convert, DynFramerate, FrameCount, Framerate, Frames, Timecode, ToFrames};

///An immutable timecode object
#[wasm_bindgen]
pub struct JSTimecode(Timecode<DynFramerate>);

#[wasm_bindgen]
impl JSTimecode {
    ///Construct a new timecode from timecode and framerate
    #[wasm_bindgen(constructor)]
    pub fn new(timecode: &str, framerate: &str) -> Result<JSTimecode, JsValue> {
        let f = Timecode::new_with_fr(timecode, framerate)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        Ok(Self(f))
    }

    ///Construct a new timecode from frame count and framerate
    pub fn from_frames(frames: FrameCount, framerate: &str) -> Result<JSTimecode, JsValue> {
        let d: DynFramerate = framerate.parse().map_err(|e: &str| JsValue::from_str(e))?;
        let f = Timecode::from_frames(&Frames(frames), &d);
        Ok(Self(f))
    }

    ///Display timecode as string
    pub fn ts(&self) -> String {
        format!("{}", self.0)
    }

    ///Add two timecodes together, returning a new timecode object
    pub fn add_timecode(&self, tc: JSTimecode) -> JSTimecode {
        JSTimecode(self.0 + tc.0)
    }

    ///Advance this timecode forward by frames
    pub fn add_frames(&self, frames: FrameCount) -> JSTimecode {
        JSTimecode(self.0 + Frames(frames))
    }

    ///Move this timecode backward by frames. Throws an exception if timecode would go below 0
    ///frames.
    pub fn sub_frames(&self, frames: FrameCount) -> Result<JSTimecode, JsValue> {
        if self.0.to_frame_count() < frames {
            return Err(JsValue::from_str("Not enough frames"));
        }
        Ok(JSTimecode(self.0 - Frames(frames)))
    }

    ///Return the number of frames since 00:00:00:00
    pub fn frame_count(&self) -> FrameCount {
        self.0.to_frame_count()
    }

    ///Convert timecode to another framerate, with 00:00:00:00 as the basis
    pub fn convert_to(&self, framerate: &str) -> Result<JSTimecode, JsValue> {
        let d: DynFramerate = framerate.parse().map_err(|e: &str| JsValue::from_str(e))?;
        Ok(Self(self.0.convert_with_fr(&d)))
    }

    ///Convert tiemcode to another framerate, with any timecode as the basis. `start` must be valid
    ///in both the source and destination framerates.
    pub fn convert_with_start(
        &self,
        framerate: &str,
        start: &JSTimecode,
    ) -> Result<JSTimecode, JsValue> {
        let d: DynFramerate = framerate.parse().map_err(|e: &str| JsValue::from_str(e))?;
        Ok(Self(self.0.convert_with_start_fr(start.0, &d)))
    }

    ///Return the framerate (calculated by framerate ratio)
    pub fn framerate(&self) -> String {
        format!("{:.3}", self.0.framerate().fr_ratio())
    }

    ///Return true if this timecode is dropframe
    pub fn is_dropframe(&self) -> bool {
        self.0.framerate().is_dropframe()
    }

    ///The hours part of the timecode
    pub fn h(&self) -> u8 {
        self.0.h()
    }
    ///The minutes part of the timecode
    pub fn m(&self) -> u8 {
        self.0.m()
    }
    ///The seconds part of the timecode
    pub fn s(&self) -> u8 {
        self.0.s()
    }
    ///The frames part of the timecode
    pub fn f(&self) -> FrameCount {
        self.0.f()
    }
}

#[wasm_bindgen]
extern "C" {
    #[allow(non_camel_case_types)]
    type console;

    pub fn log(text: &str);
}

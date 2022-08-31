use wasm_bindgen::prelude::*;

use crate::{Convert, DynFramerate, FrameCount, Framerate, Frames, Timecode, ToFrames};

#[wasm_bindgen]
pub struct JSTimecode(Timecode<DynFramerate>);

#[wasm_bindgen]
impl JSTimecode {
    #[wasm_bindgen(constructor)]
    pub fn new(s: &str, fr: &str) -> Result<JSTimecode, JsValue> {
        let f = Timecode::new_with_fr(s, fr).map_err(|e| JsValue::from_str(&e.to_string()))?;
        Ok(Self(f))
    }

    pub fn from_frames(frames: FrameCount, fr: &str) -> Result<JSTimecode, JsValue> {
        let d: DynFramerate = fr.parse().map_err(|e: &str| JsValue::from_str(e))?;
        let f = Timecode::from_frames(&Frames(frames), &d);
        Ok(Self(f))
    }

    pub fn ts(&self) -> String {
        format!("{}", self.0)
    }

    pub fn add(&self, tc: JSTimecode) -> JSTimecode {
        JSTimecode(self.0 + tc.0)
    }

    pub fn add_frames(&self, frames: FrameCount) -> JSTimecode {
        JSTimecode(self.0 + Frames(frames))
    }

    pub fn sub_frames(&self, frames: FrameCount) -> Result<JSTimecode, JsValue> {
        if self.0.to_frame_count() < frames {
            return Err(JsValue::from_str("Not enough frames"));
        }
        Ok(JSTimecode(self.0 - Frames(frames)))
    }

    pub fn frame_count(&self) -> FrameCount {
        self.0.to_frame_count()
    }

    pub fn convert_to(&self, fr: &str) -> Result<JSTimecode, JsValue> {
        let d: DynFramerate = fr.parse().map_err(|e: &str| JsValue::from_str(e))?;
        Ok(Self(self.0.convert_to_dyn(&d)))
    }

    pub fn convert_with_start(&self, fr: &str, start: &JSTimecode) -> Result<JSTimecode, JsValue> {
        let d: DynFramerate = fr.parse().map_err(|e: &str| JsValue::from_str(e))?;
        Ok(Self(self.0.convert_with_start_dyn(start.0, &d)))
    }

    pub fn framerate(&self) -> String {
        format!("{:.3}", self.0.framerate().fr_ratio())
    }

    pub fn is_dropframe(&self) -> bool {
        self.0.framerate().is_dropframe()
    }

    pub fn h(&self) -> u8 {
        self.0.h()
    }
    pub fn m(&self) -> u8 {
        self.0.m()
    }
    pub fn s(&self) -> u8 {
        self.0.s()
    }
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

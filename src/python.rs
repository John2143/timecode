use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

#[pymodule]
fn timecode(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_class::<JSTimecode>()?;
    Ok(())
}

use crate::{Convert, DynFramerate, FrameCount, Framerate, Frames, Timecode, ToFrames};

#[pyclass]
#[derive(Clone)]
pub struct JSTimecode(Timecode<DynFramerate>);

#[pymethods]
impl JSTimecode {
    #[new]
    pub fn new(s: &str, fr: &str) -> PyResult<JSTimecode> {
        let d: DynFramerate = fr.parse().map_err(|e: &str| PyValueError::new_err(e))?;
        match Timecode::new_with_fr(s, fr) {
            Ok(tc) => return Ok(JSTimecode(tc)),
            Err(_) => {}
        };

        let frames: FrameCount = match s.parse() {
            Ok(frames) => frames,
            Err(e) => return Err(PyValueError::new_err(e.to_string())),
        };

        let f = Timecode::from_frames(&Frames(frames), &d);
        Ok(JSTimecode(f))
    }

    //pub fn from_frames(frames: FrameCount, fr: &str) -> PyResult<JSTimecode> {
    //let d: DynFramerate = fr.parse().map_err(|e: &str| PyValueError::from_str(e))?;
    //let f = Timecode::from_frames(&Frames(frames), &d);
    //Ok(Self(f))
    //}

    //pub fn ts(&self) -> String {
    //format!("{}", self.0)
    //}

    //pub fn add(&self, tc: JSTimecode) -> JSTimecode {
    //JSTimecode(self.0 + tc.0)
    //}

    //pub fn add_frames(&self, frames: FrameCount) -> JSTimecode {
    //JSTimecode(self.0 + Frames(frames))
    //}

    //pub fn sub_frames(&self, frames: FrameCount) -> PyResult<JSTimecode> {
    //if self.0.to_frame_count() < frames {
    //return Err(PyValueError::new_err("Not enough frames"));
    //}
    //Ok(JSTimecode(self.0 - Frames(frames)))
    //}

    //pub fn frame_count(&self) -> FrameCount {
    //self.0.to_frame_count()
    //}

    //pub fn convert_to(&self, fr: &str) -> PyResult<JSTimecode> {
    //let d: DynFramerate = fr.parse().map_err(|e: &str| PyValueError::new_err(e))?;
    //Ok(Self(self.0.convert_to_dyn(&d)))
    //}

    //pub fn convert_with_start(&self, fr: &str, start: &JSTimecode) -> PyResult<JSTimecode> {
    //let d: DynFramerate = fr.parse().map_err(|e: &str| PyValueError::new_err(e))?;
    //Ok(Self(self.0.convert_with_start_dyn(start.0, &d)))
    //}

    //pub fn framerate(&self) -> String {
    //format!("{:.3}", self.0.framerate().fr_ratio())
    //}

    //pub fn is_dropframe(&self) -> bool {
    //self.0.framerate().is_dropframe()
    //}
}

use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

#[pymodule]
fn timecode(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_class::<Timecode>()?;
    Ok(())
}

//rename Timecode to TC because I don't know how to rename a function for pyo3
use crate::{Convert, DynFramerate, FrameCount, Framerate, Frames, Timecode as TC, ToFrames};

#[pyclass]
#[derive(Clone)]
pub struct Timecode(TC<DynFramerate>);

#[pymethods]
impl Timecode {
    #[new]
    pub fn new(s: &str, fr: &str) -> PyResult<Timecode> {
        let d: DynFramerate = fr.parse().map_err(|e: &str| PyValueError::new_err(e))?;
        match TC::new_with_fr(s, fr) {
            Ok(tc) => return Ok(Timecode(tc)),
            Err(_) => {}
        };

        let frames: FrameCount = match s.parse() {
            Ok(frames) => frames,
            Err(e) => return Err(PyValueError::new_err(e.to_string())),
        };

        let f = TC::from_frames(&Frames(frames), &d);
        Ok(Timecode(f))
    }

    pub fn __str__(&self) -> String {
        self.ts()
    }

    pub fn __repr__(&self) -> String {
        self.ts()
    }

    pub fn ts(&self) -> String {
        format!("{}", self.0)
    }

    pub fn add(&self, tc: Timecode) -> Timecode {
        Timecode(self.0 + tc.0)
    }

    pub fn add_frames(&self, frames: FrameCount) -> Timecode {
        Timecode(self.0 + Frames(frames))
    }

    pub fn sub_frames(&self, frames: FrameCount) -> PyResult<Timecode> {
        if self.0.to_frame_count() < frames {
            return Err(PyValueError::new_err("Not enough frames"));
        }
        Ok(Timecode(self.0 - Frames(frames)))
    }

    pub fn frame_count(&self) -> FrameCount {
        self.0.to_frame_count()
    }

    pub fn convert_to(&self, fr: &str) -> PyResult<Timecode> {
        let d: DynFramerate = fr.parse().map_err(|e: &str| PyValueError::new_err(e))?;
        Ok(Self(self.0.convert_with_fr(&d)))
    }

    pub fn convert_with_start(&self, fr: &str, start: &Timecode) -> PyResult<Timecode> {
        let d: DynFramerate = fr.parse().map_err(|e: &str| PyValueError::new_err(e))?;
        Ok(Self(self.0.convert_with_start_fr(&start.0, &d)))
    }

    pub fn framerate(&self) -> f32 {
        self.0.framerate().fr_ratio()
    }

    pub fn is_dropframe(&self) -> bool {
        self.0.framerate().is_dropframe()
    }
}

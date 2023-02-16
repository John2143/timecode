// This is the interface to the JVM that we'll call the majority of our
// methods on.
use jni::JNIEnv;

// These objects are what you should use as arguments to your native
// function. They carry extra lifetime information to prevent them escaping
// this context and getting used after being GC'd.
use jni::objects::{JClass, JString};

// This is just a pointer. We'll be returning it from our function. We
// can't return one of the objects with lifetime information because the
// lifetime checker won't let us.
use jni::sys::{jlong, jstring};

use crate::{DynFramerate, Timecode};

type JavaTimecode = Timecode<DynFramerate>;

#[no_mangle]
pub extern "system" fn Java_RTimecode_timecodeNew<'local>(
    mut env: JNIEnv,
    _class: JClass,
    timecode: JString,
    framerate: JString,
) -> jlong {
    //TODO alloc
    let fr_s: String = env
        .get_string(&framerate)
        .expect("JNI Interface Error: invalid framerate input (null pointer?)")
        .into();

    let tc_s: String = env
        .get_string(&timecode)
        .expect("JNI Interface Error: invalid timecode input (null pointer?)")
        .into();

    let tco: Timecode<DynFramerate> =
        Timecode::new_with_fr(&tc_s, &fr_s).expect("invalid timecode");

    Box::into_raw(Box::new(tco)) as jlong
}

#[no_mangle]
pub extern "system" fn Java_RTimecode_timecodeFree<'local>(
    _env: JNIEnv,
    _class: JClass,
    tc_ptr: jlong,
) {
    let _ = unsafe { Box::from_raw(tc_ptr as *mut JavaTimecode) };
}

import * as t from "./pkg/timecode.js";

console.log(t);
console.log(t.JSTimecode);

console.log("before")
let x = new t.JSTimecode("00:01:02:03", "25");
console.log("after")
console.log(x);
console.log(x.ts());
x = x.add_frames(3);
console.log(x.ts());
console.log(x.frame_count());
let x2 = new t.JSTimecode("00:00:00:03", "25");
console.log(x2.frame_count());
console.log("========");
let xb = x.convert_to("2398");
console.log(xb.ts());
console.log(xb.framerate());
console.log(xb.frame_count());
console.log(xb.add_frames(1000002).ts());
console.log(xb.add_frames(1000002).frame_count());
try{
console.log(xb.sub_frames(3000).frame_count());
}catch(e){
    console.log(e)
}

console.log((new t.JSTimecode("01:00:00:04", "60")).frame_count())

import { Timecode } from "./timecode_js_node/timecode.js";


console.log("before")
let x = new Timecode("00:01:02:03", "25");
console.log("after")
console.log(x);
console.log(x.tc());
x = x.add_frames(3);
console.log(x.tc());
console.log(x.frame_count());
let x2 = new Timecode("00:00:00:03", "25");
console.log(x2.frame_count());
console.log("========");
let xb = x.convert_to("23.98");
console.log(xb.tc());
console.log(xb.framerate());
console.log(xb.frame_count());
console.log(xb.add_frames(1000002).tc());
console.log(xb.add_frames(1000002).frame_count());
try{
    console.log(xb.sub_frames(3000).frame_count());
}catch(e){
    console.log("Failed to sub frames:", e)
}

console.log((new Timecode("01:00:00:04", "60")).frame_count())


let tc = new Timecode("00:01:25:41", "47.96");
let ntc = tc.convert_to("50");
console.log(ntc.tc());

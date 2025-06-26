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


//let tc = new Timecode("00:01:25:41", "47.96");
let tc = new Timecode("00:01:25:42", "47.40");
console.log(tc.framerate());
console.log(tc.convert_to("23.98").tc());



/// $GPRMC,205942.00,V,,,,,,,030724,,,N*77
/// $GPVTG,,,,,,,,,N*30
/// $GPGGA,205942.00,,,,,0,00,99.99,,,,,,*6E
/// $GPGSA,A,1,,,,,,,,,,,,,99.99,99.99,99.99*30
/// $GPGSV,3,1,11,05,16,094,31,10,,,25,13,21,047,29,15,,,29*76
/// $GPGSV,3,2,11,18,,,43,23,,,26,24,,,38,29,,,29*7E
/// $GPGSV,3,3,11,32,08,234,26,46,20,244,30,48,23,240,39*41
/// $GPGLL,,,,,205942.00,V,N*42
//
/// $GPRMC,210045.00,A,3901.98089,N,07703.37097,W,0.052,,030724,,,A*67
/// $GPVTG,,T,,M,0.052,N,0.097,K,A*2A
/// $GPGGA,210045.00,3901.98089,N,07703.37097,W,1,08,0.91,150.0,M,-34.6,M,,*66
/// $GPGSA,A,3,24,13,29,05,10,32,18,23,,,,,1.52,0.91,1.21*0C
/// $GPGSV,3,1,10,05,16,094,31,10,21,292,27,13,21,047,28,18,78,213,41*72
/// $GPGSV,3,2,10,23,52,312,20,24,48,136,35,29,07,194,28,32,09,235,30*70
/// $GPGSV,3,3,10,46,20,244,30,48,23,240,37*76
/// $GPGLL,3901.98089,N,07703.37097,W,210045.00,A,A*7B


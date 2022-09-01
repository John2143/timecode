from timecode import Timecode

start = Timecode("01:00:00:00", "25")
tc = Timecode("01:02:03:04", "25")

tc = tc.add_frames(4)
assert str(tc) == "01:02:03:08";
print(str(tc))

tc59 = tc.convert_to("59.94")
print(str(tc59))
assert str(tc59) == "01:02:03;20";

tc592 = tc.convert_with_start("59.94", start)
print(str(tc592))
assert str(tc592) == "01:02:03;19";

print(tc.frame_count())
assert tc.frame_count() == 93083

print(tc.is_dropframe())
assert not tc.is_dropframe()
assert tc59.is_dropframe()
assert tc592.is_dropframe()

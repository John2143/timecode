from timecode import Timecode

# start = Timecode("01:00:00:00", "25")
# tc = Timecode("01:02:03:04", "25")

# tc = tc.add_frames(4)
# assert str(tc) == "01:02:03:08";
# print(str(tc))

# tc59 = tc.convert_to("59.94")
# print(str(tc59))
# assert str(tc59) == "01:02:03;20";

# tc592 = tc.convert_with_start("59.94", start)
# print(str(tc592))
# assert str(tc592) == "01:02:03;19";

# print(tc.frame_count())
# assert tc.frame_count() == 93083

# print(tc.is_dropframe())
# assert not tc.is_dropframe()
# assert tc59.is_dropframe()
# assert tc592.is_dropframe()




start = Timecode("09:59:40:00", "25")
tc_to_convert = Timecode("10:00:00:00", "25")

num_frames = tc_to_convert.frame_count() - start.frame_count()
print(f"Num frames in {tc_to_convert} from start: {num_frames}")

tc_converted = tc_to_convert.convert_with_start("29.97", start)

num_frames_converted = tc_converted.frame_count() - start.convert_to("29.97").frame_count()
print(f"Converted approx tc is {tc_converted}, approx num frames from start: {num_frames_converted}")

exact_frame_diff = num_frames * (29.97 / 25.0)
error = exact_frame_diff - num_frames_converted
print(f"The exact number of converted frames is f{exact_frame_diff}.")
print(f"Frame error is {error:.4f}")

class RTimecode {
    public static native long timecodeNew(String tc, String framerate);
    public static native void timecodeFree(long tc_obj);
    public static native int timecode_h(long tc_obj);
    public static native int timecode_m(long tc_obj);
    public static native int timecode_s(long tc_obj);
    public static native int timecode_f(long tc_obj);
    public static native int timecodeFrameCount(long tc_obj);

    static {
        System.loadLibrary("timecode");
    }

    public static void main(String[] args) {
        Timecode output = new Timecode("10:01:00:03", "29.97");
        //System.out.println(output);
        System.out.println(output.h());
        System.out.println(output.m());
        System.out.println(output.s());
        System.out.println(output.f());
        System.out.println(output.frameCount());
        System.out.println("yo");
    }
}

class Timecode {
    long tc_ptr;

    public Timecode(String timecode, String framerate) {
        tc_ptr = RTimecode.timecodeNew(timecode, framerate);
    }

    public int h() {
        return RTimecode.timecode_h(tc_ptr);
    }
    public int m() {
        return RTimecode.timecode_m(tc_ptr);
    }
    public int s() {
        return RTimecode.timecode_s(tc_ptr);
    }
    public int f() {
        return RTimecode.timecode_f(tc_ptr);
    }

    public int frameCount() {
        return RTimecode.timecodeFrameCount(tc_ptr);
    }

    @Override
    public void finalize() {
        RTimecode.timecodeFree(tc_ptr);
    }
}

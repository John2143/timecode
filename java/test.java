class RTimecode {
    private static native long timecodeNew(String tc, String framerate);
    private static native void timecodeFree(long tc_obj);

    static {
        System.loadLibrary("timecode");
    }

    public static void main(String[] args) {
        long output = RTimecode.timecodeNew("10:01:00:03", "29.97");
        System.out.println(output);
        RTimecode.timecodeFree(output);
        System.out.println("yo");
    }
}

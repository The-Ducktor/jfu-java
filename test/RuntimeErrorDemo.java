/*
Demonstrates a NullPointerException runtime error with code context display.
The error formatter will show the source code line where the error occurred.
*/
public class RuntimeErrorDemo {
    public static void main(String[] args) {
        String text = null;
        // This will throw NullPointerException at runtime
        int length = text.length();
        System.out.println("Length: " + length);
    }
}

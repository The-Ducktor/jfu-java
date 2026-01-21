/*
Demonstrates a compilation error with missing method call.
The error formatter will show the problematic line and suggest fixes.
*/
public class CompilationErrorDemo {
    public static void main(String[] args) {
        String text = "hello";
        // This method doesn't exist - compiler will catch it
        text.toUppercaes();
        System.out.println(text);
    }
}

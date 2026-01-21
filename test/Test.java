/*
Basic test file demonstrating simple Java syntax.
This is a valid program that compiles and runs successfully.
*/
public class Test {
    public static void main(String[] args) {
        int x = 10;
        if (x > 5) {
            System.out.println("Hello World");
        } else {
            System.out.println("Goodbye");
        }
        
        // This is a comment
        String str = "test string";
        String upper = str.toUpperCase();
        System.out.println("Uppercase: " + upper);
        
        // Loop test
        for (int i = 0; i < 3; i++) {
            System.out.println("Loop iteration: " + i);
        }
    }
}

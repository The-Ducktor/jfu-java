/*
Demonstrates compilation errors and how jfu formats them.
This file has intentional errors to showcase error formatting.
*/
public class CompilationErrorExample {
    public static void main(String[] args) {
        String greeting = "Hello";
        // Method name typo - compiler catches it
        System.out.println(greeting.toUppercaes());
    }
}

/*
Demonstrates an ArrayIndexOutOfBoundsException.
The error formatter will display the code line that threw the exception
with syntax highlighting and proper context.
*/
public class ArrayErrorDemo {
    public static void main(String[] args) {
        int[] numbers = {1, 2, 3};
        
        // Accessing index that doesn't exist
        int value = numbers[10];
        System.out.println("Value: " + value);
    }
}

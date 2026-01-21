/*
Demonstrates a ClassCastException when casting to wrong type.
Shows how the error formatter displays the problematic line with context.
*/
public class ClassCastErrorDemo {
    public static void main(String[] args) {
        Object obj = "Hello";
        
        // Attempting to cast String to Integer
        Integer num = (Integer) obj;
        System.out.println("Number: " + num);
    }
}

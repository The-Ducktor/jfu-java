/*
Demonstrates a StackOverflowError from infinite recursion.
The error formatter provides special handling for this common mistake,
showing hints about base cases and exit conditions.
*/
public class RecursionErrorDemo {
    public static void main(String[] args) {
        countDown(1000000);
    }
    
    // This will overflow the stack - missing base case!
    public static void countDown(int n) {
        System.out.println("Count: " + n);
        countDown(n - 1);  // Never stops!
    }
}

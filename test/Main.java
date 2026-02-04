/*
using "Runner.java"
using "Cool.java"
*/

public class Main {

    public static void main(String[] args) {
        System.out.println("=== JFU Build System Demo ===");

        Runner runner = new Runner();
        runner.execute();
        System.out.println("test");

        Cool cool = new Cool();
        cool.doCoolStuff();

        System.out.println("\n✅ All dependencies working!");
    }
}

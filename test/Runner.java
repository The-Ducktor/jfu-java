/*
using "Helper.java"
using "Nothing.java"
*/
class Runner {

    public void execute() {
        System.out.println("\n🏃 Runner executing...");

        Helper helper = new Helper();
        String result = helper.help();

        System.out.println("  Helper says: " + result);
        helper.assist();
    }
}

/*
Example demonstrating how to use external JAR files with jfu.
This file shows how to import and use classes from external libraries.

To use external libraries, add to jfu.toml:
    classpath = ["libs/*.jar"]

Then jfu will automatically compile and run with those JARs on the classpath.
*/
import java.util.ArrayList;
import java.util.HashMap;

public class WithExternalLibrary {
    public static void main(String[] args) {
        // Using standard library (always available)
        ArrayList<String> items = new ArrayList<>();
        items.add("First");
        items.add("Second");
        
        HashMap<String, Integer> map = new HashMap<>();
        map.put("count", items.size());
        
        System.out.println("Items: " + items);
        System.out.println("Map: " + map);
        
        // External libraries would be used similarly:
        // import com.external.SomeClass;
        // SomeClass instance = new SomeClass();
    }
}

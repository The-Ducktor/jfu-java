make runtime exceptions look nicer show code 
```bash
🔥 Exception in thread "main" java.lang.NullPointerException: Cannot invoke "DoubleListNode.setPrev(DoubleListNode)" because "this.head" is null
  → at DLList.addFirst(DLList.java:30)
  → at DLList.addLast(DLList.java:39)
  → at DLLDriver.main(DLLDriver.java:10)

───────────────────────────────────────────
```

fix when using scanner and java scanner crash passthrough correctluy
```bash
jfu run MusicPlayerTester.java
...
💥 Runtime Error
─────────────────────────────────────────────────────────────────────────────────────────────────

  🔥 Exception in thread "main" java.util.NoSuchElementException
    → at java.base/java.util.Scanner.throwFor(Scanner.java:975)
    → at java.base/java.util.Scanner.next(Scanner.java:1632)
    → at java.base/java.util.Scanner.nextInt(Scanner.java:2297)
    → at java.base/java.util.Scanner.nextInt(Scanner.java:2251)
    → at MusicPlayerTester.main(MusicPlayerTester.java:32)

─────────────────────────────────────────────────────────────────────────────────────────────────
💡 Check the stack trace above to find the issue.


❌ Program exited with status code: 1
Choose an option: ⏎
~/Downloads/formative-assessment
❯
```
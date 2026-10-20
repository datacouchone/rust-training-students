# Defend your derive

1. Why is formatting and then hiding a secret incorrect?
2. Must a redacted type implement Debug?
3. Why is a Debug bound on every type parameter too strong?
4. Which type should receive the bound for a field of type Option<T>?
5. What does split_for_impl do with generic defaults?
6. What happens to the user's existing where clause?
7. Can two separate summary attributes repeat the same option?
8. Does a skipped field reserve an output label?
9. Can a renamed label contain braces or quotes?
10. Why should malformed attributes produce a compile error instead of a panic?
11. What breaks if generated code uses an unqualified String or Vec?
12. Why should a visible field be formatted exactly once?
13. What is the output for an empty named struct?
14. What is the default label for a raw identifier such as r#type?
15. Does the derive modify or consume the original struct's fields?
16. Why do baseline tests passing not establish that wire_summary works?

# Defend your journal

1. Why must rejecting stage return the exact original owner?
2. Does prepare publish any item?
3. Why does failed preparation return Open rather than Prepared?
4. What happens when a prepared transaction is dropped?
5. Which field owns the staged values during panic unwinding?
6. Can a caller inspect the journal while a transaction still borrows it?
7. Why is head different from the core lab's head?
8. Which slot receives the next value when the journal is partially full?
9. Who owns an evicted value after commit returns?
10. Why reserve output space before the first committed change?
11. Why is N = 0 legal here even though the core ring rejected it?
12. Why can a wrapped logical window not be one contiguous slice?
13. When may the caller request the next overlapping mutable window?
14. Does a skipped get_mut index advance the window?
15. Can a zero-sized value still have a destructor?
16. Does this transaction provide crash recovery or cross-thread isolation?

### Object
```
super pointer
data
```

data will vary for primitives, for a normal object it is a hashtable of strings and pointers to other objects

GC tracing must follow super pointer and also the other hashtable values in the case of a normal object.
primitive data and hashtable keys will be cleaned up along with the object (freed together)
Variable declaration
`let number = 10`
let must be used when assigning to something that is currently undefined

### Primitives
- Boolean
  - `true` or `false`
- Integer
  - eg `1` `0xFF`
  - Always a long (64 bit)

- Float
 - eg `0.1` `434.`
 - Always double precision

- String
 - eg `"Hello World"`
 - a UTF-8 encoded string

- Function
 - see below

### Objects
Create an instance by calling an existing object
`let test = Object()`.
Object is top-level.
Objects act as "classes" because if `test.field` is undefined, the statement will return `Object.field`.
Likewise when calling `test.func()`, as this operation just gets the function contained by func and calls it.
Functions can be declared with `test.newFunc = <Function Declaration>`. the field `super` is reserved. `test.super == Object`

### Scope
Importing is done by file. Import `a/file/path/server.pusl` with `import a.file.path.server`,
which is used with `a.file.path.server` in a file. Can also add an as statement `import a.file.path.server as the.server`

Top level names in a file must be marked with the export keyword. All marked names are imported when the file is imported.
you may also use the as keyword: `export test as outwardName`.

### Controlling Flow
for all blocks, one liners are allowed like so
```
if <expr>: <expr>
```

#### if block
```
if <expr>:
    <expr...>
```

#### if else if else block
```
if <expr>:
    <expr...>
else if <expr>:
    <expr...>
else:
    <expr...>
```

#### while loop
```
while <expr>:
    <expr...>
```

#### for loop
```
for <name> in <expr returning iterable>:
    <expr...>
```

#### compare block
```
cmp <expr> to <expr>:
    >:
        <expr...>
    <=:
        <expr...>
```

### Symbols and Operators
 - `;` -> makes an expression return 'undefined'
 - `?:` -> elvis operator: if the left is undefined, return right side, else return left side
 - `?=` -> conditional assignment: if name to left is undefined, assign right side to it
 - `+` `-` `*` `/` -> math operators, division promotes to float unambiguously, for truncation use `//`
 - `//` -> truncating division
 - '%' -> Modulus Operator
 - `+` -> string concatenation
 - `=` -> assignment operator
 - `==` `!=` `>` `<` `>=` `<=` -> comparison operators
 - `**` -> exponent
 - `#` -> comment
 - `:` -> Start a block
 - `!` -> Not
 - '&' -> Logical And
 - '|' -> Logical Or

### Defining a Function
```
let topLevelFunction = (param1, param2):
    <expr...>

let noArgs = ():
    <expr...>

```

the reserved name `self` is used to refer to this object

### Order of Operations
 - Field Access - parent `.` member
 - Function Call - `(`..`)`
 - Unary - `-`
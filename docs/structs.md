## src/parser/mod.rs
### `pub struct ParsedFile {`
A the complete ast of a file in an `ExpRef` and a list of imports that are at the top of the file

### `pub struct Import {`
The string path that the import is targeting and its alias

## src/backend/mod.rs
`pub struct BoundFunction {`
A function which has values bound to its bind targets

`struct Variable {`
A name value pair

`pub struct StackFrame {`
A stack frame during runtime. Has a this_obj, a bound function that is in the GC, a stack of variables with scope sentinels, a stack of values, and a current index into the function's bytecode.

`pub struct ExecContext<'a> {`
A function which resolves imports and a reference to the stream to use as stdout

`pub struct ExecutionState<'a> {`
All state for program execution

## src/backend/object.rs
`struct ObjectFmtWrapper<'a>(&'a ObjectPtr);`
A wrapper that allows formatting a Gc<RefCell<Object>> even if it cannot be borrowed

`pub struct PuslObject {`
An object in runtime which may have a super object and a hashtable of fields

## src/backend/linearize.rs
`pub struct ByteCodeFile {`
A basic function and a list of imports

`pub struct ResolvedFunction {`
A function which has subfunctions and also its imports have been resolved to loaded objects

`pub struct ErrorCatch {`
Defines a catch zone (catches from offset begin to offset yoink), where offset yoink is the start of the error handling section. Also has the index of the filter and the variable name to store the error in

`pub struct Function {`
A compiled function. Has argument names, bind names, a pool of literals and references, catch ranges, and code. A flag indicates if it is a generator or not.

`pub struct BasicFunction {`
A function that has sub functions (which are themselves basic functions)

## src/backend/list.rs
`struct List {`
A list native object (implemented on top of a vec)

`struct ListBuiltin {`

## src/backend/opcode.rs
`pub struct ByteCodeArray(Vec<ByteCode>);`
An array of ByteCode which only exposes safe interfaces via OpCode

`pub struct OpCodeIter<'a> {`
An iterator over the decoded opcodes inside a ByteCodeArray

`struct ByteCodeArrayVisitor;`
Part of the ByteCodeArray deserialize impl

## src/backend/generator.rs
`struct Generator {`
the Generator native object

`struct IterationEnd;`
The sentinel object for generator iteration

`struct GeneratorBuiltin {`

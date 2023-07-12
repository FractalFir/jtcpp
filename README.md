# jtcpp - JVM bytecode to C++ converter
`jtcpp` is a tool that takes java class and jar files, and converts them to valid C++ code. Resulting `.cpp` and `.hpp` files can then be compiled for a wide range of targets. **NOTE:** Not all targets provided by supported compilers(`gcc` and `clang`) can be used, because they must be also supported by the garbage collector. Other compilers and targets can be used, but it requires far more effort to get java code up and running.
# What does it mean that a target is not supported?
`jtcpp` generated code does not use any compiler - specific intrinsic or extensions. Compilers such as `msvc` are not supported because they require different flags for compilation, and those flags are not provided. 
A target platform is not supported if the used garbage collector does not support it. In such case either porting of the used GC, or replacing it is required. All `jtcpp` classes derive form class `gc`, and this class should be replaced to add alternative, platform-specific `gc`'s.
//TODO: more on usage, examples, effects on performance
# Why translation can fail?
`jtcpp` assumes certain invariants hold true for all JVM bytecode. If those invaraints are broken, translation will continue, but produced code can contain errors. Breaking of those invariants is *very, very* rare, and code that breaks them would, in many cases, have to be maliciously designed to do so.
## Examples of invariants
### Example bytecode notation
Beware: in all examples, a special notation is used to aid reading and understanding them. All jumps addresses are absolute, and in described in terms of ops, not offsets in bytecode. Instead of using indices to constant pool and type descriptions on the side, types are used(Instead of `new #10 // class SomeClass`, notation like that: `New(SomeClass)` is used).
### One local, one type
`jtcpp` assumes each local variable has one, distinct type for the entire method. So, following JVM bytecode sinppet:
```
FLoad(1), // Local 1 is float
F2I,
IStore(1), // Local 1 is int
```
while technically speaking valid, and executable by JVM, would break this invariant. There are, however, some mitigations in place, designed to prevent most of issues arising form this kind of unusual bytecode. This invariant can be broken for types of different kinds. `Int` and `Float` are diffrent kinds. `ObjectRef(String)` and `Int` are diffrent kinds of types. But ObjectRef(String) and ObjectRef(Class) are the same kind of type, and could cause a collision and emission of invalid `.cpp` code.  This means that the snippet presented above would still work, but one like that:
```
New(Vector3),
AStore(1), //Local 1 is ObjectRef(Vector3)
...
IConst(-1),
IStore(1), // Ok, mitigations can see that two locals have differnent kinds, and resolve the issue.
...
New(Bird),
AStore(1), // `jtcpp` can't decide if local 1 is ObjectRef(Vector3), or ObjectRef(Bird). Invalid .cpp code may be generated.
```
Would fail.
## Locals are not read before they are set
This may seem like something that should not be allowed by the JVM specification and such invalid bytecode could never be emitted. If we are talking about *chronolgical* order of reads and writes, then yes. But what about the order of ops in bytecode? Consider following:
```
1: Goto(5) // Executed 1st, Jumps to 5:
2: ALoad(0) //Executed 5th,  Loads Local 0, but `jtcpp` , does not know its type(since it goes trough ops in order), so it is assumed to be Unknown. This is not an error yet!
3: AStore(1) //Executed 6th, Stores contents of local 0 in local 1. `jtcpp` tries to deduce the type of local 1. It knows it is the same as local0,
// but since local 0 is unknown, local 1 is unknown too. `Unknown` type is placed in generated C++ code, to be fixed by the user
4: Goto(8) //Executed 7th, jumps to 8 
5: New(Vector3) // Executed 2st, Pushes a reference to new Vector3 on stack
6: AStore(0) // Executed 3rd, Stores the reference to the new Vector3 in local 0
7: Goto(2) // Executed 4th, Jumps to 2:
8: ALoad(1) // Executed 8th,  loads local 1, (type of which jtcpp does not know).
9: AReturn// Executed 9th, returns value of local 9
```
It is a very convoluted mess of jumps. All the bytecode that breaks the invariant, in such a way, to cause issues in generated C++ code looks at least as insane as this. This is why I think it is fair to say JVM bytecode that breaks this particular invariant must be created maliciously, to break stuff.

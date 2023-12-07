# brainf-k
a simple bainf**k interpret just run:\
`cargo run -- [-b/-1/-r] [the name of the files you like to run]`\
- -b: Compile Files to ByteCode
- -i run ByteCode
- -r run PlainText Brainf**k
- -l compile it to nativ code using llvm and clang

i recommend to compile it to ByteCode and then Run the ByteCode  
because:

- a. The ByteCode is smaller in Size
- b. The ByteCode is optimized
- c. The Runtime is shorter

import dis
def fib(n): 
    if n <= 1:
        return n
    else:
        return fib(n - 1) + fib(n - 2)

bytecode = dis.Bytecode(fib)
for instr in bytecode:
    print(instr)

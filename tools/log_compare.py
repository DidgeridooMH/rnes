from dataclasses import dataclass

@dataclass
class Instruction:
    cycle: int
    a: int 
    x: int 
    y: int 
    # p: str
    pc: int

def read_log(filename: str) -> list[Instruction]:
    instructions: list[Instruction] = []
    with open(filename) as f:
        lines = f.readlines()
        for line in lines:
            if line[0] == 'c':
                tokens = line.split()
                cycles = int(tokens[0][1:]) + 7
                a = int(tokens[1].split(':')[1], 16)
                x = int(tokens[2].split(':')[1], 16)
                y = int(tokens[3].split(':')[1], 16)
                # p = tokens[5].split(':')[1]
                pc = int(tokens[6][1:5], 16)
                instructions.append(Instruction(cycles, a, x, y, pc))
    return instructions

def read_nestest_log(filename: str) -> list[Instruction]:
    instructions: list[Instruction] = []
    with open(filename) as f:
        lines = f.readlines()
        for line in lines:
            tokens = line.split()
            for i, token in enumerate(tokens):
                if i > 0:
                    comp = token.split(':')
                    if len(comp) == 2:
                        if comp[0] == 'A':
                            a = int(comp[1], 16)
                        if comp[0] == 'X':
                            x = int(comp[1], 16)
                        if comp[0] == 'Y':
                            y = int(comp[1], 16)
                        if comp[0] == 'CYC':
                            cycles = int(comp[1])
                else:
                    pc = int(token, 16)
            
            instructions.append(Instruction(cycles, a, x, y, pc))
    return instructions

rnes = read_log('output.txt')
print('RNES read: ', len(rnes))
fceux = read_nestest_log('nestest.log.txt')
print('FCEUX read: ', len(fceux))

for i in range(0, min(len(rnes), len(fceux))):
    if rnes[i] != fceux[i]:
        print(f'Discrepancy found on line {i}')
        print("\tRNES -> ", rnes[i])
        print("\tORIG -> ", fceux[i])
        break

